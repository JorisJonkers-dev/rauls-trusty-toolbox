// SPDX-License-Identifier: GPL-3.0-only
//! Drives the wizards of `raul-ui-model` with a [`Service`]. Each function does one transition of the shared
//! plan, review, apply machine: the wizard is in the step the function needs and ends in the step the result
//! calls for. The service's reports and errors are the only source of what the wizards show.

use crate::dto::ModsReport;
use crate::service::{
    ImportRequest, InstallRequest, InstallResult, ModsRequest, RollbackRequest, Service,
    ServiceError, TtwRequest,
};
use raul_ui_model::error::{ExitClass, ViewError};
use raul_ui_model::job::Outcome;
use raul_ui_model::mods::{ModRow, ModStep, ModsScreen};
use raul_ui_model::wizard::{Blocker, Flow, FlowError, ImportWizard, InstallWizard, TtwWizard};

/// What the install wizard needs besides what it holds itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallContext {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
    /// The Mod's folder name, `None` to derive it.
    pub name: Option<String>,
}

/// What the import wizard needs besides what it holds itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportContext {
    /// The new Instance folder.
    pub instance: String,
    /// The base copy of the game.
    pub base: String,
}

/// What the TTW wizard needs besides what it holds itself.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TtwContext {
    /// The Instance folder; `None` for the one `raul config` derives.
    pub instance: Option<String>,
    /// The profile.
    pub profile: Option<String>,
    /// The New Vegas folder; `None` for the one in `raul config`.
    pub newvegas: Option<String>,
    /// The TTW package file when the wizard has none; `None` for the one in `raul config`.
    pub package: Option<String>,
    /// Continue an interrupted run.
    pub resume: bool,
}

/// How an apply ended, by the exit class of the command.
pub fn outcome_of(class: ExitClass) -> Outcome {
    match class {
        ExitClass::Ok => Outcome::Completed,
        ExitClass::Refused | ExitClass::VerificationFailed | ExitClass::Usage => Outcome::Refused,
        ExitClass::StepFailedRolledBack => Outcome::RolledBack,
        ExitClass::ProblemsFound | ExitClass::Internal => Outcome::Failed,
    }
}

fn finish_failed(flow: &mut Flow, e: &ViewError) -> Result<(), FlowError> {
    flow.finish(e.exit, outcome_of(e.exit))
}

fn install_request(w: &InstallWizard, ctx: &InstallContext, dry_run: bool) -> InstallRequest {
    InstallRequest {
        archive: w.archive().unwrap_or_default().to_owned(),
        instance: ctx.instance.clone(),
        profile: ctx.profile.clone(),
        name: ctx.name.clone(),
        dry_run,
        answers: w
            .fomod()
            .map(raul_ui_model::wizard::FomodWizard::answers_json),
    }
}

/// Plans an install with a dry run. The wizard is planning (after `choose_archive` or `submit_answers`).
/// A FOMOD that needs answers hands over to the FOMOD wizard; a refusal or failure returns to the input step
/// with the error; a dry run goes to the review with the files it listed.
///
/// # Errors
/// [`FlowError::WrongStep`] when the wizard is not planning.
pub fn plan_install(
    w: &mut InstallWizard,
    service: &dyn Service,
    ctx: &InstallContext,
) -> Result<(), FlowError> {
    let request = install_request(w, ctx, true);
    match service.install_mod(&request) {
        Ok(InstallResult::Questions(questions)) => w.needs_answers(questions),
        Ok(InstallResult::Report(report)) => {
            w.set_planned_files(report.files);
            w.plan_ready(Vec::new())
        }
        Err(e) => w.flow.plan_failed(e.view_error()),
    }
}

/// Applies a reviewed install.
///
/// # Errors
/// The wizard is not in a reviewed plan ([`Flow::apply`]).
pub fn apply_install(
    w: &mut InstallWizard,
    service: &dyn Service,
    ctx: &InstallContext,
) -> Result<(), FlowError> {
    w.flow.apply(false)?;
    let request = install_request(w, ctx, false);
    match service.install_mod(&request) {
        Ok(InstallResult::Report(_)) => w.flow.finish(ExitClass::Ok, Outcome::Completed),
        Ok(InstallResult::Questions(_)) => w.flow.finish(ExitClass::ProblemsFound, Outcome::Failed),
        Err(e) => finish_failed(&mut w.flow, &e.view_error()),
    }
}

/// Plans an MO2 import. The command line has no dry run for it, so the plan has no blocking rows yet: the
/// ones the profile files cause come back when Create runs ([`apply_import`]).
///
/// # Errors
/// See [`ImportWizard::plan`] and [`ImportWizard::plan_ready`].
pub fn plan_import(w: &mut ImportWizard) -> Result<(), FlowError> {
    w.plan()?;
    w.plan_ready(Vec::new())
}

/// Creates the Instance from the MO2 profile. When the profile files disagree the command wrote nothing and the
/// wizard returns to the review with one blocking row per disagreement.
///
/// # Errors
/// The wizard is not in a reviewed plan ([`Flow::apply`]).
pub fn apply_import(
    w: &mut ImportWizard,
    service: &dyn Service,
    ctx: &ImportContext,
) -> Result<(), FlowError> {
    w.flow.apply(false)?;
    let request = ImportRequest {
        mo2: w.mo2().unwrap_or_default().to_owned(),
        instance: ctx.instance.clone(),
        base: ctx.base.clone(),
        profile: w.profile().map(str::to_owned),
        copy_only: w.copy_only(),
    };
    match service.import_mo2(&request) {
        Ok(_) => w.flow.finish(ExitClass::Ok, Outcome::Completed),
        Err(ServiceError::Disagree(findings)) => w.disagreed(
            findings
                .into_iter()
                .map(|f| Blocker {
                    text: format!("{}: {}", f.subject, f.message),
                })
                .collect(),
        ),
        Err(e) => finish_failed(&mut w.flow, &e.view_error()),
    }
}

/// Plans a TTW install: what is needed is checked by the wizard, and the review shows the command.
///
/// # Errors
/// See [`TtwWizard::plan`].
pub fn plan_ttw(w: &mut TtwWizard) -> Result<(), FlowError> {
    w.plan()?;
    w.flow.plan_ready(0)
}

/// Runs the TTW install. The report's status is the outcome: a refusal or an unsupported package is
/// [`Outcome::Refused`], a failure [`Outcome::RolledBack`] (the Instance is as it was) and a cancel
/// [`Outcome::Cancelled`].
///
/// # Errors
/// See [`TtwWizard::apply`].
pub fn apply_ttw(
    w: &mut TtwWizard,
    service: &dyn Service,
    ctx: &TtwContext,
) -> Result<(), FlowError> {
    w.apply()?;
    let request = TtwRequest {
        instance: ctx.instance.clone(),
        profile: ctx.profile.clone(),
        fallout3: w.fallout3().map(str::to_owned),
        newvegas: ctx.newvegas.clone(),
        package: w
            .package()
            .map(str::to_owned)
            .or_else(|| ctx.package.clone()),
        dry_run: false,
        resume: ctx.resume,
    };
    match service.ttw(&request) {
        Ok(report) => {
            let outcome = report.status.outcome(report.dry_run);
            let class = match report.status {
                raul_ui_model::events::Status::Success => ExitClass::Ok,
                raul_ui_model::events::Status::Refused
                | raul_ui_model::events::Status::Unsupported => ExitClass::Refused,
                raul_ui_model::events::Status::Unverifiable => ExitClass::VerificationFailed,
                raul_ui_model::events::Status::Failed
                | raul_ui_model::events::Status::Cancelled => ExitClass::StepFailedRolledBack,
            };
            w.flow.finish(class, outcome)
        }
        Err(e) => finish_failed(&mut w.flow, &e.view_error()),
    }
}

/// What the Mods screen needs besides what it holds itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsContext {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
}

fn mods_request(ctx: &ModsContext, step: ModStep, dry_run: bool) -> ModsRequest {
    ModsRequest {
        instance: ctx.instance.clone(),
        profile: ctx.profile.clone(),
        step,
        dry_run,
    }
}

/// What the staged Mods edits would do, from a dry run of each step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsPreview {
    /// Each step with the report of its dry run, in the order they would run.
    pub steps: Vec<(ModStep, ModsReport)>,
    /// True when every dry run was made against the list the step would really meet. The enable and the disable
    /// step always are, and so is the first move; a later move is checked against the list as it is now, before
    /// the moves ahead of it. Apply stops at the first step the command refuses and undoes the earlier ones.
    pub exact: bool,
}

impl ModsPreview {
    /// The Mods the steps would touch, in step order and without repeats.
    pub fn changed(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for (_, report) in &self.steps {
            for n in &report.changed {
                if !names.contains(n) {
                    names.push(n.clone());
                }
            }
        }
        names
    }
}

/// Dry-runs the staged edits of the Mods screen, one direct dry run per step, and writes nothing.
/// An empty answer means nothing is staged.
///
/// # Errors
/// The error of the first step the command refused.
pub fn preview_mods(
    screen: &ModsScreen,
    service: &dyn Service,
    ctx: &ModsContext,
) -> Result<ModsPreview, ServiceError> {
    let plan = screen.plan();
    let moves = plan
        .iter()
        .filter(|s| matches!(s, ModStep::Move { .. }))
        .count();
    let mut steps = Vec::new();
    for step in plan {
        let report = service.mods_change(&mods_request(ctx, step.clone(), true))?;
        steps.push((step, report));
    }
    Ok(ModsPreview {
        steps,
        exact: moves <= 1,
    })
}

/// How applying the staged Mods edits ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsApplied {
    /// The steps that were carried out and stay (all of them, or none after an undo).
    pub applied: usize,
    /// The steps of the plan.
    pub total: usize,
    /// Why a step failed. The earlier steps were undone ([`ModsApplied::undone`]) so the Instance is as it was.
    pub error: Option<ViewError>,
    /// The earlier steps that were undone (the rollback job) after a failure.
    pub undone: usize,
}

/// Applies the staged edits of the Mods screen, one `raul mods ...` per step. When every step went through the
/// screen is reloaded from the last report (so nothing is staged any more). When one fails, the steps before it
/// are undone by the rollback job (each step has its own journal), the edits stay staged, and the error says
/// what happened; an undo that fails too is said in the error, with how many steps are still applied.
pub fn apply_mods(
    screen: &mut ModsScreen,
    service: &dyn Service,
    ctx: &ModsContext,
) -> ModsApplied {
    let plan = screen.plan();
    let total = plan.len();
    let mut journaled = 0;
    let mut last: Option<ModsReport> = None;
    for step in plan {
        match service.mods_change(&mods_request(ctx, step, false)) {
            Ok(report) => {
                journaled += usize::from(report.journal.is_some());
                last = Some(report);
            }
            Err(e) => {
                let mut error = e.view_error();
                let mut undone = 0;
                for _ in 0..journaled {
                    let undo = RollbackRequest {
                        instance: ctx.instance.clone(),
                    };
                    if let Err(why) = service.rollback(&undo) {
                        error.effect = format!(
                            "{} step(s) are still applied and could not be undone; check the Mods list. {}",
                            journaled - undone,
                            why.view_error().what
                        );
                        return ModsApplied {
                            applied: journaled - undone,
                            total,
                            error: Some(error),
                            undone,
                        };
                    }
                    undone += 1;
                }
                error.effect = if undone == 0 {
                    "Nothing was changed".to_owned()
                } else {
                    format!("{undone} earlier step(s) were undone; nothing was changed")
                };
                return ModsApplied {
                    applied: 0,
                    total,
                    error: Some(error),
                    undone,
                };
            }
        }
    }
    if let Some(report) = last {
        screen.loaded(report.mods.iter().map(ModRow::from).collect());
    }
    ModsApplied {
        applied: total,
        total,
        error: None,
        undone: 0,
    }
}

#[cfg(test)]
mod mods_tests {
    use super::*;
    use crate::dto::ModEntry;
    use crate::fake::FakeService;
    use crate::service::RollbackRequest;

    fn entry(position: usize, name: &str, state: &str) -> ModEntry {
        ModEntry {
            position,
            name: name.to_owned(),
            state: state.to_owned(),
            kind: "mod".to_owned(),
            installed: true,
        }
    }

    fn report(action: &str, dry: bool, changed: &[&str], mods: Vec<ModEntry>) -> ModsReport {
        ModsReport {
            status: if dry { "dry-run" } else { "changed" }.to_owned(),
            dry_run: dry,
            action: action.to_owned(),
            profile: "Default".to_owned(),
            changed: changed.iter().map(|c| (*c).to_owned()).collect(),
            mods,
            staged_files: None,
            journal: (!dry).then(|| format!("j-{action}")),
        }
    }

    fn screen() -> ModsScreen {
        let mut s = ModsScreen::new();
        s.loaded(
            ["A", "B", "C"]
                .iter()
                .map(|n| ModRow {
                    name: (*n).to_owned(),
                    enabled: true,
                })
                .collect(),
        );
        s
    }

    fn ctx() -> ModsContext {
        ModsContext {
            instance: "/i".into(),
            profile: Some("P".into()),
        }
    }

    fn failure(why: &str) -> ServiceError {
        ServiceError::Failed(ViewError::new(
            ExitClass::StepFailedRolledBack,
            why,
            "The step was rolled back",
        ))
    }

    #[test]
    fn a_preview_dry_runs_every_step_and_says_when_it_is_not_exact() {
        let fake = FakeService::new();
        let mut s = screen();
        s.set_enabled("B", false).unwrap();
        s.move_to("A", 2).unwrap();
        fake.on_mods(Ok(report("disable", true, &["B"], vec![])))
            .on_mods(Ok(report("reorder", true, &["A"], vec![])));
        let preview = preview_mods(&s, &fake, &ctx()).unwrap();
        assert!(preview.exact);
        assert_eq!(preview.changed(), ["B", "A"]);
        let calls = fake.mods_calls();
        assert!(calls.iter().all(|c| c.dry_run && c.instance == "/i"));
        assert_eq!(calls[0].step, ModStep::Disable(vec!["B".into()]));

        // Two moves: the second is checked against the list before the first.
        let mut s = screen();
        s.move_to("C", 0).unwrap();
        s.move_to("B", 1).unwrap();
        assert_eq!(s.plan().len(), 2);
        fake.on_mods(Ok(report("reorder", true, &["C"], vec![])))
            .on_mods(Ok(report("reorder", true, &["C"], vec![])));
        assert!(!preview_mods(&s, &fake, &ctx()).unwrap().exact);

        // A refusal of the dry run is the error of the preview; nothing staged previews as nothing.
        let mut s = screen();
        s.set_enabled("A", false).unwrap();
        fake.on_mods(Err(failure("no such Mod")));
        assert!(preview_mods(&s, &fake, &ctx()).is_err());
        assert!(
            preview_mods(&screen(), &fake, &ctx())
                .unwrap()
                .steps
                .is_empty()
        );
    }

    #[test]
    fn a_complete_apply_reloads_the_screen_from_the_last_report() {
        let fake = FakeService::new();
        let mut s = screen();
        s.set_enabled("B", false).unwrap();
        s.move_to("A", 2).unwrap();
        fake.on_mods(Ok(report("disable", false, &["B"], vec![])));
        fake.on_mods(Ok(report(
            "reorder",
            false,
            &["A"],
            vec![
                entry(1, "B", "disabled"),
                entry(2, "C", "enabled"),
                entry(3, "A", "enabled"),
            ],
        )));
        let done = apply_mods(&mut s, &fake, &ctx());
        assert_eq!((done.applied, done.total, done.undone), (2, 2, 0));
        assert!(done.error.is_none() && !s.is_dirty());
        let rows: Vec<_> = s
            .visible()
            .iter()
            .map(|r| (r.name.clone(), r.enabled))
            .collect();
        assert_eq!(
            rows,
            [
                ("B".to_owned(), false),
                ("C".to_owned(), true),
                ("A".to_owned(), true)
            ]
        );
        assert!(fake.mods_calls().iter().all(|c| !c.dry_run));
    }

    #[test]
    fn a_failed_step_undoes_the_earlier_ones_and_an_undo_that_fails_is_said() {
        let fake = FakeService::new();
        let mut s = screen();
        s.set_enabled("B", false).unwrap();
        s.move_to("A", 2).unwrap();
        fake.on_mods(Ok(report("disable", false, &["B"], vec![])))
            .on_mods(Err(failure("the write failed")));
        fake.on_rollback(Ok(crate::fake::sample::rollback()));
        let done = apply_mods(&mut s, &fake, &ctx());
        assert_eq!((done.applied, done.undone), (0, 1));
        let e = done.error.unwrap();
        assert_eq!(e.what, "the write failed");
        assert!(e.effect.contains("1 earlier step(s) were undone"));
        assert!(s.is_dirty());
        assert_eq!(
            fake.rollback_calls(),
            [RollbackRequest {
                instance: "/i".into()
            }]
        );

        fake.on_mods(Ok(report("disable", false, &["B"], vec![])))
            .on_mods(Err(failure("the write failed")));
        fake.on_rollback(Err(failure("the undo failed")));
        let done = apply_mods(&mut s, &fake, &ctx());
        assert_eq!((done.applied, done.undone), (1, 0));
        assert!(
            done.error
                .unwrap()
                .effect
                .contains("1 step(s) are still applied")
        );

        // The first step failing has nothing to undo.
        fake.on_mods(Err(failure("refused")));
        let done = apply_mods(&mut s, &fake, &ctx());
        assert_eq!((done.applied, done.undone), (0, 0));
        assert_eq!(done.error.unwrap().effect, "Nothing was changed");
    }
}
