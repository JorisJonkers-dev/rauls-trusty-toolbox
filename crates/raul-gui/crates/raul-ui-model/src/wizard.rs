// SPDX-License-Identifier: GPL-3.0-only
//! The install and FOMOD, MO2 import and TTW wizards (documents 01 and 04).
//!
//! All three share one rule: a wizard plans first (nothing is written), the user reviews the plan,
//! and only then can it apply. [`Flow`] holds that rule once; each wizard adds its own inputs and blockers.

use crate::error::{ExitClass, ViewError};
use crate::job::Outcome;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Where a wizard is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    /// Collecting what the plan needs.
    Input,
    /// The plan is being computed.
    Planning,
    /// The plan is shown; the user reads it.
    Review,
    /// The plan is being applied.
    Applying,
    /// Cancel was requested; waiting for the job to finish.
    Cancelling,
    /// Over.
    Done,
}

/// Why a transition was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowError {
    /// The wizard is not in the step the transition needs.
    WrongStep(Step),
    /// The plan was not shown to the user yet.
    NotReviewed,
    /// Blocking rows remain (the count).
    Blocked(usize),
    /// The running job cannot be cancelled.
    NotCancellable,
}

/// The plan, review, apply, done machine every wizard uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    step: Step,
    reviewed: bool,
    blockers: usize,
    cancellable: bool,
    error: Option<ViewError>,
    result: Option<(ExitClass, Outcome)>,
}

impl Default for Flow {
    fn default() -> Self {
        Self {
            step: Step::Input,
            reviewed: false,
            blockers: 0,
            cancellable: false,
            error: None,
            result: None,
        }
    }
}

impl Flow {
    /// A flow at the input step.
    pub fn new() -> Self {
        Self::default()
    }

    /// The step.
    pub fn step(&self) -> Step {
        self.step
    }

    /// The error of the last failed plan, if any.
    pub fn error(&self) -> Option<&ViewError> {
        self.error.as_ref()
    }

    /// How the apply ended.
    pub fn result(&self) -> Option<(ExitClass, Outcome)> {
        self.result
    }

    /// Blocking rows of the shown plan.
    pub fn blockers(&self) -> usize {
        self.blockers
    }

    fn expect(&self, step: Step) -> Result<(), FlowError> {
        if self.step == step {
            Ok(())
        } else {
            Err(FlowError::WrongStep(self.step))
        }
    }

    /// Input to planning.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] outside the input step.
    pub fn begin_plan(&mut self) -> Result<(), FlowError> {
        self.expect(Step::Input)?;
        self.error = None;
        self.step = Step::Planning;
        Ok(())
    }

    /// The plan is ready with `blockers` rows that stop it from being applied.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] outside planning.
    pub fn plan_ready(&mut self, blockers: usize) -> Result<(), FlowError> {
        self.expect(Step::Planning)?;
        self.blockers = blockers;
        self.reviewed = false;
        self.step = Step::Review;
        Ok(())
    }

    /// Planning failed: back to the input with the error shown.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] outside planning.
    pub fn plan_failed(&mut self, error: ViewError) -> Result<(), FlowError> {
        self.expect(Step::Planning)?;
        self.error = Some(error);
        self.step = Step::Input;
        Ok(())
    }

    /// The adapter showed the review panel (`Action::Reviewed`).
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] outside the review step.
    pub fn mark_reviewed(&mut self) -> Result<(), FlowError> {
        self.expect(Step::Review)?;
        self.reviewed = true;
        Ok(())
    }

    /// Whether Apply is enabled.
    pub fn can_apply(&self) -> bool {
        self.step == Step::Review && self.reviewed && self.blockers == 0
    }

    /// Review to applying.
    ///
    /// # Errors
    /// Wrong step, not reviewed, or blocking rows.
    pub fn apply(&mut self, cancellable: bool) -> Result<(), FlowError> {
        self.expect(Step::Review)?;
        if !self.reviewed {
            return Err(FlowError::NotReviewed);
        }
        if self.blockers > 0 {
            return Err(FlowError::Blocked(self.blockers));
        }
        self.cancellable = cancellable;
        self.step = Step::Applying;
        Ok(())
    }

    /// The command refused to start because of blocking rows it found only while applying (an exit 1 that wrote
    /// nothing): back to the review with those rows, which must be seen again.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] outside the applying step.
    pub fn apply_blocked(&mut self, blockers: usize) -> Result<(), FlowError> {
        self.expect(Step::Applying)?;
        self.blockers = blockers;
        self.reviewed = false;
        self.step = Step::Review;
        Ok(())
    }

    /// Asks the running job to stop.
    ///
    /// # Errors
    /// Wrong step, or the job is not cancellable.
    pub fn cancel(&mut self) -> Result<(), FlowError> {
        self.expect(Step::Applying)?;
        if !self.cancellable {
            return Err(FlowError::NotCancellable);
        }
        self.step = Step::Cancelling;
        Ok(())
    }

    /// The job finished.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] unless applying or cancelling.
    pub fn finish(&mut self, exit: ExitClass, outcome: Outcome) -> Result<(), FlowError> {
        if !matches!(self.step, Step::Applying | Step::Cancelling) {
            return Err(FlowError::WrongStep(self.step));
        }
        self.result = Some((exit, outcome));
        self.step = Step::Done;
        Ok(())
    }

    /// Review back to the input (the plan is dropped, the review is forgotten).
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] outside the review step.
    pub fn back(&mut self) -> Result<(), FlowError> {
        self.expect(Step::Review)?;
        self.reviewed = false;
        self.blockers = 0;
        self.step = Step::Input;
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// FOMOD

/// How an option of a FOMOD group is typed by its author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OptionKind {
    /// Must be installed.
    Required,
    /// Suggested.
    Recommended,
    /// Free to choose.
    Optional,
    /// Cannot be installed.
    NotUsable,
    /// Free to choose, with a warning.
    CouldBeUsable,
}

/// A group's selection rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupRule {
    /// Any number.
    SelectAny,
    /// All of them.
    SelectAll,
    /// Exactly one.
    SelectExactlyOne,
    /// None or one.
    SelectAtMostOne,
    /// One or more.
    SelectAtLeastOne,
}

/// One choice of a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FomodOption {
    /// The name given back in the answers file.
    pub name: String,
    /// The author's description.
    pub description: String,
    /// The type, when known.
    pub kind: Option<OptionKind>,
}

/// A group the installer asks (the question shape of `nvmod-modinstall`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FomodQuestion {
    /// `step name/group name`, the key of the answers file.
    pub id: String,
    /// The install step.
    pub step: String,
    /// The group.
    pub group: String,
    /// The rule.
    pub rule: GroupRule,
    /// Fewest choices.
    pub min: usize,
    /// Most choices.
    pub max: usize,
    /// The choices.
    pub options: Vec<FomodOption>,
}

/// Why an answer was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerError {
    /// No such question.
    UnknownQuestion(String),
    /// The group has no option of that name.
    UnknownOption(String),
    /// Fewer than the group's minimum.
    TooFew {
        /// Fewest allowed.
        min: usize,
    },
    /// More than the group's maximum.
    TooMany {
        /// Most allowed.
        max: usize,
    },
    /// An option that cannot be installed was chosen.
    NotUsable(String),
    /// An option that must be installed was left out.
    MissingRequired(String),
}

/// The FOMOD questions and the user's answers, one question at a time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FomodWizard {
    questions: Vec<FomodQuestion>,
    answers: BTreeMap<String, Vec<String>>,
    at: usize,
}

impl FomodWizard {
    /// A wizard over the questions the planner could not answer.
    pub fn new(questions: Vec<FomodQuestion>) -> Self {
        Self {
            questions,
            answers: BTreeMap::new(),
            at: 0,
        }
    }

    /// The question shown, none when there are none.
    pub fn current(&self) -> Option<&FomodQuestion> {
        self.questions.get(self.at)
    }

    /// 1-based position and total: "2 of 5".
    pub fn position(&self) -> (usize, usize) {
        (
            self.at.min(self.questions.len().saturating_sub(1)) + 1,
            self.questions.len(),
        )
    }

    /// What is chosen for a question, with the options that must be installed pre-selected.
    pub fn selection(&self, id: &str) -> Vec<String> {
        if let Some(a) = self.answers.get(id) {
            return a.clone();
        }
        self.questions
            .iter()
            .find(|q| q.id == id)
            .map_or_default(|q| {
                q.options
                    .iter()
                    .filter(|o| {
                        o.kind == Some(OptionKind::Required) || q.rule == GroupRule::SelectAll
                    })
                    .map(|o| o.name.clone())
                    .collect()
            })
    }

    /// Records the choice for a question.
    ///
    /// # Errors
    /// An [`AnswerError`] describing why the choice breaks the group's rule; nothing is recorded then.
    pub fn answer(&mut self, id: &str, chosen: &[&str]) -> Result<(), AnswerError> {
        let q = self
            .questions
            .iter()
            .find(|q| q.id == id)
            .ok_or_else(|| AnswerError::UnknownQuestion(id.to_owned()))?;
        for c in chosen {
            let o = q
                .options
                .iter()
                .find(|o| o.name == *c)
                .ok_or_else(|| AnswerError::UnknownOption((*c).to_owned()))?;
            if o.kind == Some(OptionKind::NotUsable) {
                return Err(AnswerError::NotUsable(o.name.clone()));
            }
        }
        if let Some(o) = q.options.iter().find(|o| {
            (o.kind == Some(OptionKind::Required) || q.rule == GroupRule::SelectAll)
                && !chosen.contains(&o.name.as_str())
        }) {
            return Err(AnswerError::MissingRequired(o.name.clone()));
        }
        if chosen.len() < q.min {
            return Err(AnswerError::TooFew { min: q.min });
        }
        if chosen.len() > q.max {
            return Err(AnswerError::TooMany { max: q.max });
        }
        self.answers.insert(
            id.to_owned(),
            chosen.iter().map(|c| (*c).to_owned()).collect(),
        );
        Ok(())
    }

    /// Whether the question was answered.
    pub fn is_answered(&self, id: &str) -> bool {
        self.answers.contains_key(id)
    }

    /// Next question; false when the current one is unanswered or it is the last.
    pub fn advance(&mut self) -> bool {
        let answered = self.current().is_some_and(|q| self.is_answered(&q.id));
        if answered && self.at + 1 < self.questions.len() {
            self.at += 1;
            true
        } else {
            false
        }
    }

    /// Previous question; false on the first.
    pub fn previous(&mut self) -> bool {
        if self.at > 0 {
            self.at -= 1;
            true
        } else {
            false
        }
    }

    /// Whether every question has an answer.
    pub fn is_complete(&self) -> bool {
        self.questions.iter().all(|q| self.is_answered(&q.id))
    }

    /// The answers file `raul install-mod --answers` reads: question id to the chosen option names.
    pub fn answers_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.answers).unwrap_or_default()
    }
}

/// One mod install: archive, FOMOD questions if any, review, apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallWizard {
    /// The shared machine.
    pub flow: Flow,
    archive: Option<String>,
    fomod: Option<FomodWizard>,
    overwrites: Vec<String>,
    #[serde(default)]
    planned: Vec<String>,
}

impl Default for InstallWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallWizard {
    /// A wizard waiting for an archive.
    pub fn new() -> Self {
        Self {
            flow: Flow::new(),
            archive: None,
            fomod: None,
            overwrites: Vec::new(),
            planned: Vec::new(),
        }
    }

    /// The files the dry run listed (shown in the review).
    pub fn planned_files(&self) -> &[String] {
        &self.planned
    }

    /// Records the files of the dry run.
    pub fn set_planned_files(&mut self, files: Vec<String>) {
        self.planned = files;
    }

    /// The chosen archive.
    pub fn archive(&self) -> Option<&str> {
        self.archive.as_deref()
    }

    /// The FOMOD questions, when the archive has any.
    pub fn fomod(&self) -> Option<&FomodWizard> {
        self.fomod.as_ref()
    }

    /// Mutable access for answering.
    pub fn fomod_mut(&mut self) -> Option<&mut FomodWizard> {
        self.fomod.as_mut()
    }

    /// Files the plan would overwrite (shown in the review).
    pub fn overwrites(&self) -> &[String] {
        &self.overwrites
    }

    /// Picks the archive and starts planning.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when not at the input step.
    pub fn choose_archive(&mut self, path: impl Into<String>) -> Result<(), FlowError> {
        self.flow.begin_plan()?;
        self.archive = Some(path.into());
        self.fomod = None;
        Ok(())
    }

    /// The planner exited 1 with questions: the FOMOD wizard takes over (the flow returns to the input step).
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when not planning.
    pub fn needs_answers(&mut self, questions: Vec<FomodQuestion>) -> Result<(), FlowError> {
        self.flow.plan_failed(ViewError::new(
            ExitClass::ProblemsFound,
            "The FOMOD needs answers",
            "Nothing was written",
        ))?;
        self.fomod = Some(FomodWizard::new(questions));
        Ok(())
    }

    /// Submits the answers and plans again.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when not at the input step, or [`FlowError::Blocked`] with the number of
    /// unanswered questions.
    pub fn submit_answers(&mut self) -> Result<(), FlowError> {
        if let Some(f) = &self.fomod {
            let open = f.questions.iter().filter(|q| !f.is_answered(&q.id)).count();
            if open > 0 {
                return Err(FlowError::Blocked(open));
            }
        }
        self.flow.begin_plan()
    }

    /// The plan is ready; `overwrites` are the files it would replace.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when not planning.
    pub fn plan_ready(&mut self, overwrites: Vec<String>) -> Result<(), FlowError> {
        self.flow.plan_ready(0)?;
        self.overwrites = overwrites;
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// MO2 import

/// A row that stops the import from being created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    /// What is wrong, from the report.
    pub text: String,
}

/// The MO2 import: pick the MO2 instance and profile, read the plan, create the Instance. MO2 is only read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportWizard {
    /// The shared machine.
    pub flow: Flow,
    mo2: Option<String>,
    profile: Option<String>,
    copy_only: bool,
    blockers: Vec<Blocker>,
}

impl Default for ImportWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportWizard {
    /// A wizard waiting for the MO2 folder.
    pub fn new() -> Self {
        Self {
            flow: Flow::new(),
            mo2: None,
            profile: None,
            copy_only: false,
            blockers: Vec::new(),
        }
    }

    /// Chooses the MO2 instance folder.
    pub fn set_mo2(&mut self, path: impl Into<String>) {
        self.mo2 = Some(path.into());
    }

    /// Chooses the profile.
    pub fn set_profile(&mut self, name: impl Into<String>) {
        self.profile = Some(name.into());
    }

    /// Hard links are not used; every file is copied.
    pub fn set_copy_only(&mut self, copy_only: bool) {
        self.copy_only = copy_only;
    }

    /// The chosen MO2 folder.
    pub fn mo2(&self) -> Option<&str> {
        self.mo2.as_deref()
    }

    /// The chosen profile, `None` for the MO2 instance's selected one.
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Whether every file is copied.
    pub fn copy_only(&self) -> bool {
        self.copy_only
    }

    /// The command found that the profile files disagree and wrote nothing: back to the review with those rows.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when not applying.
    pub fn disagreed(&mut self, blockers: Vec<Blocker>) -> Result<(), FlowError> {
        self.flow.apply_blocked(blockers.len())?;
        self.blockers = blockers;
        Ok(())
    }

    /// The rows that disable Create.
    pub fn blockers(&self) -> &[Blocker] {
        &self.blockers
    }

    /// The command that does the same, for the equivalent-command strip.
    pub fn equivalent_command(&self) -> Option<String> {
        let mo2 = self.mo2.as_ref()?;
        let mut c = format!("raul import-mo2 --mo2 {mo2} --instance <dir> --base <dir>");
        if let Some(p) = &self.profile {
            c.push_str(" --profile ");
            c.push_str(p);
        }
        if self.copy_only {
            c.push_str(" --copy");
        }
        Some(c)
    }

    /// Starts planning once the MO2 folder is chosen.
    ///
    /// # Errors
    /// [`FlowError::Blocked`] when no folder is chosen, or [`FlowError::WrongStep`].
    pub fn plan(&mut self) -> Result<(), FlowError> {
        if self.mo2.is_none() {
            return Err(FlowError::Blocked(1));
        }
        self.flow.begin_plan()
    }

    /// The plan is ready with its blocking rows (for example an order the profile and the plugin list disagree on).
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when not planning.
    pub fn plan_ready(&mut self, blockers: Vec<Blocker>) -> Result<(), FlowError> {
        self.flow.plan_ready(blockers.len())?;
        self.blockers = blockers;
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// TTW

/// What the TTW install needs before it can be planned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtwNeed {
    /// The Fallout 3 folder.
    Fallout3,
    /// The TTW package the player downloaded (`ttw.package`).
    Package,
}

/// The Tale of Two Wastelands install. It is long, so it can be cancelled and then rolls back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtwWizard {
    /// The shared machine.
    pub flow: Flow,
    fallout3: Option<String>,
    package: Option<String>,
}

impl Default for TtwWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl TtwWizard {
    /// A wizard with nothing chosen.
    pub fn new() -> Self {
        Self {
            flow: Flow::new(),
            fallout3: None,
            package: None,
        }
    }

    /// Chooses the Fallout 3 folder.
    pub fn set_fallout3(&mut self, path: impl Into<String>) {
        self.fallout3 = Some(path.into());
    }

    /// The chosen Fallout 3 folder.
    pub fn fallout3(&self) -> Option<&str> {
        self.fallout3.as_deref()
    }

    /// Chooses the TTW package file.
    pub fn set_package(&mut self, path: impl Into<String>) {
        self.package = Some(path.into());
    }

    /// The chosen TTW package file.
    pub fn package(&self) -> Option<&str> {
        self.package.as_deref()
    }

    /// What is still missing.
    pub fn needs(&self) -> Vec<TtwNeed> {
        let mut n = Vec::new();
        if self.fallout3.is_none() {
            n.push(TtwNeed::Fallout3);
        }
        if self.package.is_none() {
            n.push(TtwNeed::Package);
        }
        n
    }

    /// Starts planning once nothing is missing.
    ///
    /// # Errors
    /// [`FlowError::Blocked`] with the number of missing things, or [`FlowError::WrongStep`].
    pub fn plan(&mut self) -> Result<(), FlowError> {
        let missing = self.needs().len();
        if missing > 0 {
            return Err(FlowError::Blocked(missing));
        }
        self.flow.begin_plan()
    }

    /// Applies the reviewed plan; TTW can always be cancelled.
    ///
    /// # Errors
    /// See [`Flow::apply`].
    pub fn apply(&mut self) -> Result<(), FlowError> {
        self.flow.apply(true)
    }

    /// The command that does the same.
    pub fn equivalent_command(&self) -> Option<String> {
        let f3 = self.fallout3.as_ref()?;
        let package = self.package.as_ref()?;
        Some(format!(
            "raul ttw --instance <dir> --fallout3 {f3} --package {package}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(name: &str, kind: Option<OptionKind>) -> FomodOption {
        FomodOption {
            name: name.into(),
            description: String::new(),
            kind,
        }
    }

    fn question(
        id: &str,
        rule: GroupRule,
        min: usize,
        max: usize,
        options: Vec<FomodOption>,
    ) -> FomodQuestion {
        FomodQuestion {
            id: id.into(),
            step: "s".into(),
            group: "g".into(),
            rule,
            min,
            max,
            options,
        }
    }

    fn reviewed_flow(blockers: usize) -> Flow {
        let mut f = Flow::new();
        f.begin_plan().unwrap();
        f.plan_ready(blockers).unwrap();
        f
    }

    #[test]
    fn apply_needs_a_reviewed_plan() {
        let mut f = reviewed_flow(0);
        assert!(!f.can_apply());
        assert_eq!(f.apply(false), Err(FlowError::NotReviewed));
        f.mark_reviewed().unwrap();
        assert!(f.can_apply());
        f.apply(false).unwrap();
        assert_eq!(f.step(), Step::Applying);
        assert_eq!(f.cancel(), Err(FlowError::NotCancellable));
        f.finish(ExitClass::Ok, Outcome::Completed).unwrap();
        assert_eq!(f.result(), Some((ExitClass::Ok, Outcome::Completed)));
        assert_eq!(
            f.finish(ExitClass::Ok, Outcome::Completed),
            Err(FlowError::WrongStep(Step::Done))
        );
    }

    #[test]
    fn blockers_disable_apply_and_wrong_steps_are_refused() {
        let mut f = reviewed_flow(2);
        f.mark_reviewed().unwrap();
        assert!(!f.can_apply());
        assert_eq!(f.apply(true), Err(FlowError::Blocked(2)));
        assert_eq!(f.blockers(), 2);
        f.back().unwrap();
        assert_eq!(f.step(), Step::Input);
        assert_eq!(f.mark_reviewed(), Err(FlowError::WrongStep(Step::Input)));
        assert_eq!(f.plan_ready(0), Err(FlowError::WrongStep(Step::Input)));
        assert_eq!(f.back(), Err(FlowError::WrongStep(Step::Input)));
        assert_eq!(f.cancel(), Err(FlowError::WrongStep(Step::Input)));
    }

    #[test]
    fn plan_failure_returns_to_input_with_the_error() {
        let mut f = Flow::new();
        f.begin_plan().unwrap();
        assert_eq!(f.begin_plan(), Err(FlowError::WrongStep(Step::Planning)));
        f.plan_failed(ViewError::new(
            ExitClass::Refused,
            "no",
            "Nothing was changed",
        ))
        .unwrap();
        assert_eq!(f.step(), Step::Input);
        assert_eq!(f.error().unwrap().exit, ExitClass::Refused);
        f.begin_plan().unwrap();
        assert!(f.error().is_none());
    }

    #[test]
    fn cancel_then_finish_rolled_back() {
        let mut f = reviewed_flow(0);
        f.mark_reviewed().unwrap();
        f.apply(true).unwrap();
        f.cancel().unwrap();
        assert_eq!(f.step(), Step::Cancelling);
        f.finish(ExitClass::StepFailedRolledBack, Outcome::RolledBack)
            .unwrap();
        assert_eq!(f.step(), Step::Done);
    }

    #[test]
    fn fomod_answers_follow_the_group_rules() {
        let q = question(
            "a/b",
            GroupRule::SelectAtLeastOne,
            1,
            2,
            vec![
                opt("core", Some(OptionKind::Required)),
                opt("extra", Some(OptionKind::Optional)),
                opt("more", None),
                opt("bad", Some(OptionKind::NotUsable)),
            ],
        );
        let mut w = FomodWizard::new(vec![q]);
        assert_eq!(w.selection("a/b"), vec!["core"]);
        assert!(w.selection("zzz").is_empty());
        assert_eq!(
            w.answer("x", &[]),
            Err(AnswerError::UnknownQuestion("x".into()))
        );
        assert_eq!(
            w.answer("a/b", &["nope"]),
            Err(AnswerError::UnknownOption("nope".into()))
        );
        assert_eq!(
            w.answer("a/b", &["core", "bad"]),
            Err(AnswerError::NotUsable("bad".into()))
        );
        assert_eq!(
            w.answer("a/b", &["extra"]),
            Err(AnswerError::MissingRequired("core".into()))
        );
        assert_eq!(
            w.answer("a/b", &["core", "extra", "more"]),
            Err(AnswerError::TooMany { max: 2 })
        );
        assert!(!w.is_complete());
        w.answer("a/b", &["core", "extra"]).unwrap();
        assert!(w.is_complete());
        assert_eq!(w.selection("a/b"), vec!["core", "extra"]);
        assert_eq!(
            w.answers_json(),
            serde_json::json!({"a/b": ["core", "extra"]})
        );
    }

    #[test]
    fn fomod_too_few_and_select_all() {
        let q = question(
            "q",
            GroupRule::SelectExactlyOne,
            1,
            1,
            vec![opt("a", None), opt("b", None)],
        );
        let all = question(
            "all",
            GroupRule::SelectAll,
            2,
            2,
            vec![opt("x", None), opt("y", None)],
        );
        let mut w = FomodWizard::new(vec![q, all]);
        assert_eq!(w.answer("q", &[]), Err(AnswerError::TooFew { min: 1 }));
        assert_eq!(w.selection("all"), vec!["x", "y"]);
        assert_eq!(
            w.answer("all", &["x"]),
            Err(AnswerError::MissingRequired("y".into()))
        );
    }

    #[test]
    fn fomod_navigation_needs_answers() {
        let qs = vec![
            question("1", GroupRule::SelectAny, 0, 2, vec![opt("a", None)]),
            question("2", GroupRule::SelectAny, 0, 2, vec![opt("b", None)]),
        ];
        let mut w = FomodWizard::new(qs);
        assert_eq!(w.position(), (1, 2));
        assert!(!w.advance());
        w.answer("1", &["a"]).unwrap();
        assert!(w.advance());
        assert_eq!(w.current().unwrap().id, "2");
        assert_eq!(w.position(), (2, 2));
        assert!(!w.advance());
        assert!(w.previous());
        assert!(!w.previous());
        let empty = FomodWizard::new(vec![]);
        assert!(empty.current().is_none());
        assert!(empty.is_complete());
        assert_eq!(empty.position(), (1, 0));
    }

    #[test]
    fn install_with_fomod_questions_round_trip() {
        let mut w = InstallWizard::new();
        w.choose_archive("mod.7z").unwrap();
        assert_eq!(w.archive(), Some("mod.7z"));
        assert_eq!(
            w.choose_archive("x"),
            Err(FlowError::WrongStep(Step::Planning))
        );
        let q = question("1", GroupRule::SelectExactlyOne, 1, 1, vec![opt("a", None)]);
        w.needs_answers(vec![q]).unwrap();
        assert_eq!(w.flow.step(), Step::Input);
        assert_eq!(w.submit_answers(), Err(FlowError::Blocked(1)));
        w.fomod_mut().unwrap().answer("1", &["a"]).unwrap();
        assert!(w.fomod().unwrap().is_complete());
        w.submit_answers().unwrap();
        w.plan_ready(vec!["Data/x.esp".into()]).unwrap();
        assert_eq!(w.overwrites(), ["Data/x.esp"]);
        w.flow.mark_reviewed().unwrap();
        w.flow.apply(false).unwrap();
    }

    #[test]
    fn install_without_fomod_plans_directly() {
        let mut w = InstallWizard::default();
        w.choose_archive("a.zip").unwrap();
        w.plan_ready(vec![]).unwrap();
        assert!(w.fomod().is_none());
        assert_eq!(w.flow.step(), Step::Review);
    }

    #[test]
    fn import_blocks_create_until_resolved() {
        let mut w = ImportWizard::new();
        assert_eq!(w.plan(), Err(FlowError::Blocked(1)));
        assert!(w.equivalent_command().is_none());
        w.set_mo2("/mo2");
        w.set_profile("Default");
        w.set_copy_only(true);
        assert_eq!(
            w.equivalent_command().unwrap(),
            "raul import-mo2 --mo2 /mo2 --instance <dir> --base <dir> --profile Default --copy"
        );
        w.plan().unwrap();
        w.plan_ready(vec![Blocker {
            text: "order disagrees".into(),
        }])
        .unwrap();
        assert_eq!(w.blockers().len(), 1);
        w.flow.mark_reviewed().unwrap();
        assert!(!w.flow.can_apply());
        let mut ok = ImportWizard::default();
        ok.set_mo2("/m");
        ok.plan().unwrap();
        ok.plan_ready(vec![]).unwrap();
        ok.flow.mark_reviewed().unwrap();
        assert!(ok.flow.can_apply());
        assert_eq!(
            ok.equivalent_command().unwrap(),
            "raul import-mo2 --mo2 /m --instance <dir> --base <dir>"
        );
    }

    #[test]
    fn ttw_needs_inputs_and_cancels_to_rollback() {
        let mut w = TtwWizard::new();
        assert_eq!(w.needs(), vec![TtwNeed::Fallout3, TtwNeed::Package]);
        assert_eq!(w.plan(), Err(FlowError::Blocked(2)));
        assert!(w.equivalent_command().is_none());
        w.set_fallout3("/fo3");
        assert!(w.equivalent_command().is_none(), "the package is missing");
        w.set_package("/ttw.mpi");
        assert_eq!(w.package(), Some("/ttw.mpi"));
        assert!(w.needs().is_empty());
        w.plan().unwrap();
        w.flow.plan_ready(0).unwrap();
        w.flow.mark_reviewed().unwrap();
        w.apply().unwrap();
        w.flow.cancel().unwrap();
        w.flow
            .finish(ExitClass::StepFailedRolledBack, Outcome::RolledBack)
            .unwrap();
        assert_eq!(
            w.equivalent_command().unwrap(),
            "raul ttw --instance <dir> --fallout3 /fo3 --package /ttw.mpi"
        );
        assert_eq!(TtwWizard::default().needs().len(), 2);
    }

    #[test]
    fn rows_found_while_applying_send_the_import_back_to_a_new_review() {
        let mut w = ImportWizard::new();
        w.set_mo2("/mo2");
        w.plan().unwrap();
        w.plan_ready(Vec::new()).unwrap();
        w.flow.mark_reviewed().unwrap();
        assert!(w.disagreed(Vec::new()).is_err(), "only while applying");
        w.flow.apply(false).unwrap();
        w.disagreed(vec![Blocker { text: "x".into() }]).unwrap();
        assert_eq!(w.flow.step(), Step::Review);
        assert!(!w.flow.can_apply(), "blocked and not reviewed again");
        assert_eq!(w.blockers().len(), 1);
    }

    #[test]
    fn the_install_wizard_keeps_the_files_of_the_dry_run() {
        let mut w = InstallWizard::new();
        w.set_planned_files(vec!["a.esp".into()]);
        assert_eq!(w.planned_files(), ["a.esp"]);
    }
}
