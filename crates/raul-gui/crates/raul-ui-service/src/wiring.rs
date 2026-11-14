// SPDX-License-Identifier: GPL-3.0-only
//! Drives the wizards of `nvmod-ui-model` with a [`Service`]. Each function does one transition of the shared
//! plan, review, apply machine: the wizard is in the step the function needs and ends in the step the result
//! calls for. The service's reports and errors are the only source of what the wizards show.

use crate::service::{
    ImportRequest, InstallRequest, InstallResult, Service, ServiceError, TtwRequest,
};
use nvmod_ui_model::error::{ExitClass, ViewError};
use nvmod_ui_model::job::Outcome;
use nvmod_ui_model::wizard::{Blocker, Flow, FlowError, ImportWizard, InstallWizard, TtwWizard};

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
    /// The Instance folder.
    pub instance: String,
    /// The profile.
    pub profile: Option<String>,
    /// The lock file.
    pub lock: Option<String>,
    /// The tool store.
    pub store: Option<String>,
    /// The TTW package file.
    pub package: Option<String>,
    /// The installer to run instead of the pinned one (a path), `None` for the pinned one.
    pub installer: Option<String>,
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
            .map(nvmod_ui_model::wizard::FomodWizard::answers_json),
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

/// Runs the TTW install. A report whose installer record says `failed` ends as a failure.
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
        fallout3: w.fallout3().unwrap_or_default().to_owned(),
        package: ctx.package.clone(),
        installer: ctx.installer.clone(),
        profile: ctx.profile.clone(),
        lock: ctx.lock.clone(),
        store: ctx.store.clone(),
    };
    match service.ttw(&request) {
        Ok(report) if report.result.outcome == "success" => {
            w.flow.finish(ExitClass::Ok, Outcome::Completed)
        }
        Ok(_) => w.flow.finish(ExitClass::Internal, Outcome::Failed),
        Err(e) => finish_failed(&mut w.flow, &e.view_error()),
    }
}
