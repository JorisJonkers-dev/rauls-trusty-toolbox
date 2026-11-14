// SPDX-License-Identifier: GPL-3.0-only
//! The [`Service`] trait and its real implementation, [`CliService`].

use crate::conflicts::{ConflictsReport, ConflictsRequest, TreeReport, TreeRequest};
use crate::dto::{
    DisagreeDoc, DoctorReport, Finding, ImportReport, InstallReport, ModsReport, QuestionsDoc,
    RollbackReport, SetupReport, SortReport, TargetReport, ToolRow, ToolsReport, TtwReport,
    VerifyReport,
};
use crate::exec::{Exec, Output};
use crate::registry::Action;
use nvmod_ui_model::error::{ExitClass, Retry, ViewError};
use nvmod_ui_model::mods::ModStep;
use nvmod_ui_model::setup::{FinishStatus, Phase, SetupWizard};
use nvmod_ui_model::wizard::FomodQuestion;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

/// Why a service call did not produce its report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    /// `import-mo2` found that the profile files contradict each other and wrote nothing.
    Disagree(Vec<Finding>),
    /// The call failed; the error is what a screen shows.
    Failed(ViewError),
}

impl ServiceError {
    /// The error as a screen shows it. A disagreement becomes a problems-found error with one line per finding.
    pub fn view_error(&self) -> ViewError {
        match self {
            Self::Failed(e) => e.clone(),
            Self::Disagree(findings) => ViewError::new(
                ExitClass::ProblemsFound,
                findings
                    .iter()
                    .map(|f| format!("{}: {}", f.subject, f.message))
                    .collect::<Vec<_>>()
                    .join("; "),
                "Nothing was written",
            ),
        }
    }
}

/// What `nvmod import-mo2` needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRequest {
    /// The MO2 instance folder (only read).
    pub mo2: String,
    /// The new Instance folder.
    pub instance: String,
    /// The base copy of the game.
    pub base: String,
    /// The MO2 profile, `None` for the one MO2 selected.
    pub profile: Option<String>,
    /// Copy every file instead of hard-linking.
    pub copy_only: bool,
}

/// What `nvmod install` needs.
#[derive(Debug, Clone, PartialEq)]
pub struct InstallRequest {
    /// The archive.
    pub archive: String,
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
    /// The Mod's folder name, `None` to derive it from the archive.
    pub name: Option<String>,
    /// Plan only: nothing is written.
    pub dry_run: bool,
    /// The FOMOD answers (question id to chosen option names), as `FomodWizard::answers_json` gives them.
    pub answers: Option<serde_json::Value>,
}

/// What `nvmod ttw` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TtwRequest {
    /// The Instance folder.
    pub instance: String,
    /// The Fallout 3 folder.
    pub fallout3: String,
    /// The TTW package file.
    pub package: Option<String>,
    /// The installer to run instead of the fetched one.
    pub installer: Option<String>,
    /// The profile.
    pub profile: Option<String>,
    /// The lock file.
    pub lock: Option<String>,
    /// The tool store.
    pub store: Option<String>,
}

/// How `nvmod setup` is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupMode {
    /// A first run from the base copy.
    #[default]
    Fresh,
    /// `--resume`: continue the unfinished setup of the Instance.
    Resume,
    /// `--restart`: remove the unfinished Instance. The command removes nothing without `--yes`.
    Restart {
        /// Whether the person confirmed the removal (`--yes`).
        confirmed: bool,
    },
}

/// What `nvmod setup` needs. A resume or a restart takes the Instance and nothing else, so [`SetupRequest::args`]
/// leaves every other field out for them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)] // One bool per flag of the command line.
pub struct SetupRequest {
    /// How it is started.
    pub mode: SetupMode,
    /// The Instance folder.
    pub instance: String,
    /// The base copy of the game.
    pub base: Option<String>,
    /// The masterlist file.
    pub masterlist: Option<String>,
    /// The profile.
    pub profile: Option<String>,
    /// An MO2 instance folder to import.
    pub import_mo2: Option<String>,
    /// Mod archives to install, in order.
    pub mods: Vec<String>,
    /// The Fallout 3 folder, which adds the TTW step.
    pub fallout3: Option<String>,
    /// The TTW package file.
    pub package: Option<String>,
    /// The TTW installer to run instead of the fetched one.
    pub installer: Option<String>,
    /// Fetch the nv-rs release (`--release`).
    pub release: bool,
    /// Build nv-rs from this source folder (`--source`).
    pub source: Option<String>,
    /// Leave out the nv-rs step.
    pub skip_nv_rs: bool,
    /// Copy every file instead of hard-linking.
    pub copy_only: bool,
    /// A failing step leaves the Instance where it is, to be resumed (`--keep-partial`).
    pub keep_partial: bool,
    /// Plan only: nothing is written (`--dry-run`).
    pub dry_run: bool,
}

impl SetupRequest {
    /// A first run.
    pub fn fresh(instance: &str, base: &str, masterlist: &str) -> Self {
        Self {
            instance: instance.to_owned(),
            base: Some(base.to_owned()),
            masterlist: Some(masterlist.to_owned()),
            ..Self::default()
        }
    }

    /// Continues the unfinished setup of `instance`.
    pub fn resume(instance: &str) -> Self {
        Self {
            mode: SetupMode::Resume,
            instance: instance.to_owned(),
            ..Self::default()
        }
    }

    /// Removes the unfinished setup of `instance`; `confirmed` is the person's yes.
    pub fn restart(instance: &str, confirmed: bool) -> Self {
        Self {
            mode: SetupMode::Restart { confirmed },
            instance: instance.to_owned(),
            ..Self::default()
        }
    }

    /// The words after `nvmod setup`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        match self.mode {
            SetupMode::Resume => rest.push("--resume".to_owned()),
            SetupMode::Restart { confirmed } => {
                rest.push("--restart".to_owned());
                push(&mut rest, "instance", &self.instance);
                if confirmed {
                    rest.push("--yes".to_owned());
                }
                return rest;
            }
            SetupMode::Fresh => {}
        }
        push(&mut rest, "instance", &self.instance);
        if self.mode == SetupMode::Resume {
            return rest;
        }
        push_opt(&mut rest, "base", self.base.as_ref());
        push_opt(&mut rest, "masterlist", self.masterlist.as_ref());
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push_opt(&mut rest, "import-mo2", self.import_mo2.as_ref());
        for archive in &self.mods {
            push(&mut rest, "mod", archive);
        }
        push_opt(&mut rest, "fallout3", self.fallout3.as_ref());
        push_opt(&mut rest, "package", self.package.as_ref());
        push_opt(&mut rest, "installer", self.installer.as_ref());
        push_opt(&mut rest, "source", self.source.as_ref());
        for (on, flag) in [
            (self.release, "--release"),
            (self.skip_nv_rs, "--skip-nv-rs"),
            (self.copy_only, "--copy"),
            (self.keep_partial, "--keep-partial"),
            (self.dry_run, "--dry-run"),
        ] {
            if on {
                rest.push(flag.to_owned());
            }
        }
        rest
    }

    /// The job that runs this request: it writes unless it only plans, and Cancel is offered for a first run or a
    /// resume (a restart is a removal that is not stopped half way).
    pub fn job_spec(&self, id: nvmod_ui_model::job::JobId) -> crate::jobs::JobSpec {
        let mut spec = crate::jobs::JobSpec::new(id, Action::Setup, self.args());
        spec.writes = !(self.dry_run && self.mode == SetupMode::Fresh);
        spec.dry_run = self.dry_run && self.mode == SetupMode::Fresh;
        spec.cancellable = spec.writes && !matches!(self.mode, SetupMode::Restart { .. });
        spec
    }
}

/// What one `nvmod mods enable|disable|reorder` call needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
    /// What to change.
    pub step: ModStep,
    /// Plan only: nothing is written.
    pub dry_run: bool,
}

impl ModsRequest {
    /// The words after `nvmod mods`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        match &self.step {
            ModStep::Enable(names) => {
                rest.push("enable".to_owned());
                rest.extend(names.iter().cloned());
            }
            ModStep::Disable(names) => {
                rest.push("disable".to_owned());
                rest.extend(names.iter().cloned());
            }
            ModStep::Move { name, position } => {
                rest.push("reorder".to_owned());
                rest.push(name.clone());
                push(&mut rest, "position", &position.to_string());
            }
        }
        push(&mut rest, "instance", &self.instance);
        push_opt(&mut rest, "profile", self.profile.as_ref());
        if self.dry_run {
            rest.push("--dry-run".to_owned());
        }
        rest
    }
}

/// What `nvmod doctor --instance` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DoctorRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile.
    pub profile: Option<String>,
    /// The masterlist.
    pub masterlist: Option<String>,
}

/// What `nvmod sort` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SortRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
    /// The masterlist file.
    pub masterlist: String,
    /// Write the proposal (`--apply`); without it nothing is written.
    pub apply: bool,
}

impl SortRequest {
    /// The words after `nvmod sort`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &self.instance);
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push(&mut rest, "masterlist", &self.masterlist);
        if self.apply {
            rest.push("--apply".to_owned());
        }
        rest
    }
}

/// What `nvmod verify` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerifyRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile.
    pub profile: Option<String>,
    /// The masterlist file.
    pub masterlist: Option<String>,
    /// The catalogue file.
    pub catalog: Option<String>,
    /// The Proton program the launch plan names.
    pub proton: Option<String>,
    /// The lowest severity that fails the verification (`info`, `warning`, `error`).
    pub fail_on: Option<String>,
}

impl VerifyRequest {
    /// The words after `nvmod verify`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &self.instance);
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push_opt(&mut rest, "masterlist", self.masterlist.as_ref());
        push_opt(&mut rest, "catalog", self.catalog.as_ref());
        push_opt(&mut rest, "proton", self.proton.as_ref());
        push_opt(&mut rest, "fail-on", self.fail_on.as_ref());
        rest
    }
}

/// What an Instance runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTarget {
    /// The vanilla game with xNVSE.
    Vanilla,
    /// The nv-rs engine.
    NvRs,
}

impl RuntimeTarget {
    /// The word of the command line.
    pub fn word(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::NvRs => "nv-rs",
        }
    }
}

/// What `nvmod target` needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRequest {
    /// The Instance folder.
    pub instance: String,
    /// What it should run on.
    pub target: RuntimeTarget,
}

impl TargetRequest {
    /// The words after `nvmod target`. The command has no `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = vec![self.target.word().to_owned()];
        push(&mut rest, "instance", &self.instance);
        rest
    }
}

/// What `nvmod tools list` and `nvmod tools verify` need.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolsRequest {
    /// The lock file, `None` for the built-in one.
    pub lock: Option<String>,
    /// The tool store, `None` for the default one.
    pub store: Option<String>,
    /// The Instance folder, whose trusted checksums count.
    pub instance: Option<String>,
}

impl ToolsRequest {
    /// The words after `nvmod tools <verb>`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push_opt(&mut rest, "lock", self.lock.as_ref());
        push_opt(&mut rest, "store", self.store.as_ref());
        push_opt(&mut rest, "instance", self.instance.as_ref());
        rest
    }
}

/// What `nvmod rollback` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RollbackRequest {
    /// The Instance folder.
    pub instance: String,
}

impl RollbackRequest {
    /// The words after `nvmod rollback`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &self.instance);
        rest
    }
}

/// The answer of an install call.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallResult {
    /// The install, or its dry run, went through.
    Report(InstallReport),
    /// The FOMOD needs answers; nothing was written.
    Questions(Vec<FomodQuestion>),
}

/// What the screens and wizards ask of the world. [`CliService`] answers by running the command line,
/// [`crate::FakeService`] from a script.
pub trait Service {
    /// `nvmod import-mo2`.
    ///
    /// # Errors
    /// [`ServiceError::Disagree`] when the profile files disagree, [`ServiceError::Failed`] otherwise.
    fn import_mo2(&self, request: &ImportRequest) -> Result<ImportReport, ServiceError>;

    /// `nvmod install`, also as a dry run.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed.
    fn install_mod(&self, request: &InstallRequest) -> Result<InstallResult, ServiceError>;

    /// `nvmod ttw`.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed.
    fn ttw(&self, request: &TtwRequest) -> Result<TtwReport, ServiceError>;

    /// `nvmod doctor --instance`. Findings of error severity are a report, not an error.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the Doctor could not run.
    fn doctor(&self, request: &DoctorRequest) -> Result<DoctorReport, ServiceError>;

    /// `nvmod sort`, a proposal or with `apply` the written order.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed (a failed apply is rolled back).
    fn sort(&self, request: &SortRequest) -> Result<SortReport, ServiceError>;

    /// `nvmod verify`. A verification that fails is a report with `verified` false, not an error.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the verification could not run.
    fn verify(&self, request: &VerifyRequest) -> Result<VerifyReport, ServiceError>;

    /// `nvmod target`.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed.
    fn set_target(&self, request: &TargetRequest) -> Result<TargetReport, ServiceError>;

    /// `nvmod tools list`.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the lock or the Instance could not be read.
    fn tools_list(&self, request: &ToolsRequest) -> Result<ToolsReport, ServiceError>;

    /// `nvmod tools verify`. A corrupt stored tool is a report with `verified` false, not an error.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the lock or the Instance could not be read.
    fn tools_verify(&self, request: &ToolsRequest) -> Result<ToolsReport, ServiceError>;

    /// `nvmod rollback`: undoes the last install or sort.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when there is nothing to undo or the undo failed.
    fn rollback(&self, request: &RollbackRequest) -> Result<RollbackReport, ServiceError>;

    /// One `nvmod mods enable|disable|reorder` call, also as a dry run.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed (a failed change is rolled back).
    fn mods_change(&self, request: &ModsRequest) -> Result<ModsReport, ServiceError>;

    /// `nvmod conflicts`: the file-level conflicts of a profile (read only).
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused.
    fn conflicts(&self, request: &ConflictsRequest) -> Result<ConflictsReport, ServiceError>;

    /// `nvmod tree`: the virtual Data folder of a profile (read only).
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused.
    fn tree(&self, request: &TreeRequest) -> Result<TreeReport, ServiceError>;

    /// `nvmod setup`, run to its end and read from its event stream. A run that fails is an error carrying the
    /// card of the setup wizard; a dry run is a report. To follow a run while it goes, use
    /// [`CliService::run_job`] with [`SetupRequest::job_spec`].
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the run was refused or failed.
    fn setup(&self, request: &SetupRequest) -> Result<SetupReport, ServiceError>;
}

fn word(value: &str) -> String {
    if value.is_empty() || value.contains(char::is_whitespace) {
        format!("'{value}'")
    } else {
        value.to_owned()
    }
}

/// The command line as a person would type it, for the error's repro.
pub(crate) fn repro(args: &[String]) -> String {
    let mut s = String::from("nvmod");
    for a in args {
        s.push(' ');
        s.push_str(&word(a));
    }
    s
}

/// What changed (or did not) for an exit class, in the words of the design.
fn effect_of(class: ExitClass) -> &'static str {
    match class {
        ExitClass::Ok => "Nothing went wrong",
        ExitClass::Refused | ExitClass::VerificationFailed | ExitClass::Usage => {
            "Nothing was changed"
        }
        ExitClass::StepFailedRolledBack => "The step was rolled back",
        ExitClass::ProblemsFound => "Problems were found",
        ExitClass::Internal => "The state is not known; run the Doctor",
    }
}

/// The cause from standard error: the first non-empty line without the `nvmod <command>:` and `refused:`
/// prefixes the command line puts in front.
pub(crate) fn cause_of(stderr: &str, command: &str) -> String {
    let line = stderr.lines().map(str::trim).find(|l| !l.is_empty());
    let Some(line) = line else {
        return String::from("The command failed without saying why");
    };
    let line = line
        .strip_prefix(&format!("nvmod {command}:"))
        .map_or(line, str::trim);
    line.strip_prefix("refused:")
        .map_or(line, str::trim)
        .to_owned()
}

fn failure(out: &Output, command: &str, args: &[String]) -> ServiceError {
    let class = out.code.map_or(ExitClass::Internal, ExitClass::from_code);
    let mut cause = cause_of(&out.stderr, command);
    if out.stderr.trim().is_empty() {
        // With --json the commands that refuse put the reason on standard output.
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&out.stdout)
            && let Some(why) = v["reason"].as_str().or_else(|| v["error"].as_str())
        {
            why.clone_into(&mut cause);
        }
    }
    let mut e = ViewError::new(class, cause, effect_of(class));
    e.repro = Some(repro(args));
    e.retry = match class {
        ExitClass::StepFailedRolledBack => Retry::Yes,
        ExitClass::Refused | ExitClass::Usage | ExitClass::ProblemsFound => {
            Retry::AfterFix(String::from("fix what the message names, then try again"))
        }
        _ => Retry::No,
    };
    ServiceError::Failed(e)
}

fn unreadable(what: &str, why: &serde_json::Error, args: &[String]) -> ServiceError {
    let mut e = ViewError::new(
        ExitClass::Internal,
        format!("The output of `nvmod {what}` could not be read: {why}"),
        effect_of(ExitClass::Internal),
    );
    e.repro = Some(repro(args));
    ServiceError::Failed(e)
}

/// A scratch file holding the FOMOD answers, removed when dropped.
struct AnswersFile(PathBuf);

impl AnswersFile {
    fn write(answers: &serde_json::Value) -> Result<Self, ServiceError> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path =
            std::env::temp_dir().join(format!("nvmod-answers-{}-{nanos}.json", std::process::id()));
        std::fs::write(&path, answers.to_string()).map_err(|e| {
            ServiceError::Failed(ViewError::new(
                ExitClass::Internal,
                format!("could not write the answers file {}: {e}", path.display()),
                "Nothing was changed",
            ))
        })?;
        Ok(Self(path))
    }
}

impl Drop for AnswersFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Runs the `nvmod` command line and reads its `--json` output.
#[derive(Debug, Clone)]
pub struct CliService<E: Exec> {
    exec: E,
}

impl<E: Exec> CliService<E> {
    /// A service over `exec`.
    pub fn new(exec: E) -> Self {
        Self { exec }
    }

    /// The `nvmod` words of a call: the command, `rest` and `--json` (commands without a JSON output do not get it).
    pub(crate) fn words(action: Action, rest: Vec<String>, json: bool) -> Vec<String> {
        let mut args = vec![action.command().to_owned()];
        args.extend(rest);
        if json {
            args.push("--json".to_owned());
        }
        args
    }

    /// The program runner, for the job runner of this crate.
    pub(crate) fn exec(&self) -> &E {
        &self.exec
    }

    fn invoke(
        &self,
        action: Action,
        rest: Vec<String>,
    ) -> Result<(Output, Vec<String>), ServiceError> {
        self.invoke_with(action, rest, true)
    }

    fn invoke_with(
        &self,
        action: Action,
        rest: Vec<String>,
        json: bool,
    ) -> Result<(Output, Vec<String>), ServiceError> {
        let args = Self::words(action, rest, json);
        match self.exec.run(&args) {
            Ok(out) => Ok((out, args)),
            Err(why) => {
                let mut e = ViewError::new(ExitClass::Internal, why, "Nothing was started");
                e.repro = Some(repro(&args));
                Err(ServiceError::Failed(e))
            }
        }
    }

    fn parse<T: DeserializeOwned>(
        action: Action,
        out: &Output,
        args: &[String],
    ) -> Result<T, ServiceError> {
        serde_json::from_str(&out.stdout).map_err(|e| unreadable(action.command(), &e, args))
    }
}

pub(crate) fn push(args: &mut Vec<String>, name: &str, value: &str) {
    args.push(format!("--{name}"));
    args.push(value.to_owned());
}

pub(crate) fn push_opt(args: &mut Vec<String>, name: &str, value: Option<&String>) {
    if let Some(v) = value {
        push(args, name, v);
    }
}

impl<E: Exec> Service for CliService<E> {
    fn import_mo2(&self, r: &ImportRequest) -> Result<ImportReport, ServiceError> {
        let mut rest = Vec::new();
        push(&mut rest, "mo2", &r.mo2);
        push(&mut rest, "instance", &r.instance);
        push(&mut rest, "base", &r.base);
        push_opt(&mut rest, "profile", r.profile.as_ref());
        if r.copy_only {
            rest.push("--copy".to_owned());
        }
        let (out, args) = self.invoke(Action::ImportMo2, rest)?;
        match out.code {
            Some(0) => Self::parse(Action::ImportMo2, &out, &args),
            Some(1) => match serde_json::from_str::<DisagreeDoc>(&out.stdout) {
                Ok(doc) => Err(ServiceError::Disagree(doc.disagreements)),
                Err(_) => Err(failure(&out, "import-mo2", &args)),
            },
            _ => Err(failure(&out, "import-mo2", &args)),
        }
    }

    fn install_mod(&self, r: &InstallRequest) -> Result<InstallResult, ServiceError> {
        let answers = r.answers.as_ref().map(AnswersFile::write).transpose()?;
        let mut rest = vec![r.archive.clone()];
        push(&mut rest, "instance", &r.instance);
        push_opt(&mut rest, "profile", r.profile.as_ref());
        push_opt(&mut rest, "name", r.name.as_ref());
        if r.dry_run {
            rest.push("--dry-run".to_owned());
        }
        if let Some(file) = &answers {
            push(&mut rest, "answers", &file.0.display().to_string());
        }
        let (out, args) = self.invoke(Action::InstallMod, rest)?;
        match out.code {
            Some(0) => Self::parse::<InstallReport>(Action::InstallMod, &out, &args)
                .map(InstallResult::Report),
            Some(1) => match serde_json::from_str::<QuestionsDoc>(&out.stdout) {
                Ok(doc) => Ok(InstallResult::Questions(
                    doc.questions.into_iter().map(Into::into).collect(),
                )),
                Err(_) => Err(failure(&out, "install", &args)),
            },
            _ => Err(failure(&out, "install", &args)),
        }
    }

    fn ttw(&self, r: &TtwRequest) -> Result<TtwReport, ServiceError> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &r.instance);
        push(&mut rest, "fallout3", &r.fallout3);
        push_opt(&mut rest, "package", r.package.as_ref());
        push_opt(&mut rest, "installer", r.installer.as_ref());
        push_opt(&mut rest, "profile", r.profile.as_ref());
        push_opt(&mut rest, "lock", r.lock.as_ref());
        push_opt(&mut rest, "store", r.store.as_ref());
        let (out, args) = self.invoke(Action::Ttw, rest)?;
        match out.code {
            Some(0) => Self::parse(Action::Ttw, &out, &args),
            _ => Err(failure(&out, "ttw", &args)),
        }
    }

    fn doctor(&self, r: &DoctorRequest) -> Result<DoctorReport, ServiceError> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &r.instance);
        push_opt(&mut rest, "profile", r.profile.as_ref());
        push_opt(&mut rest, "masterlist", r.masterlist.as_ref());
        let (out, args) = self.invoke(Action::Doctor, rest)?;
        match out.code {
            // Exit 1 is "findings at or above the failing severity": the report is on standard output.
            Some(0 | 1) => match serde_json::from_str::<DoctorReport>(&out.stdout) {
                Ok(report) => Ok(report),
                Err(e) if out.code == Some(1) => {
                    let message = serde_json::from_str::<serde_json::Value>(&out.stdout)
                        .ok()
                        .and_then(|v| v["error"].as_str().map(str::to_owned));
                    match message {
                        Some(m) => {
                            let mut v = ViewError::new(
                                ExitClass::ProblemsFound,
                                m,
                                effect_of(ExitClass::ProblemsFound),
                            );
                            v.repro = Some(repro(&args));
                            Err(ServiceError::Failed(v))
                        }
                        None => Err(unreadable("doctor", &e, &args)),
                    }
                }
                Err(e) => Err(unreadable("doctor", &e, &args)),
            },
            _ => Err(failure(&out, "doctor", &args)),
        }
    }

    fn sort(&self, r: &SortRequest) -> Result<SortReport, ServiceError> {
        let (out, args) = self.invoke(Action::Sort, r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Sort, &out, &args),
            _ => Err(failure(&out, "sort", &args)),
        }
    }

    fn verify(&self, r: &VerifyRequest) -> Result<VerifyReport, ServiceError> {
        let (out, args) = self.invoke(Action::Verify, r.args())?;
        match out.code {
            // Exit 3 is "not verified": the report is on standard output.
            Some(0 | 3) => Self::parse(Action::Verify, &out, &args),
            _ => Err(failure(&out, "verify", &args)),
        }
    }

    fn set_target(&self, r: &TargetRequest) -> Result<TargetReport, ServiceError> {
        let (out, args) = self.invoke_with(Action::Target, r.args(), false)?;
        match out.code {
            Some(0) => Ok(TargetReport {
                instance: r.instance.clone(),
                target: r.target,
            }),
            _ => Err(failure(&out, "target", &args)),
        }
    }

    fn tools_list(&self, r: &ToolsRequest) -> Result<ToolsReport, ServiceError> {
        self.tools("list", r)
    }

    fn tools_verify(&self, r: &ToolsRequest) -> Result<ToolsReport, ServiceError> {
        self.tools("verify", r)
    }

    fn rollback(&self, r: &RollbackRequest) -> Result<RollbackReport, ServiceError> {
        let (out, args) = self.invoke(Action::Rollback, r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Rollback, &out, &args),
            _ => Err(failure(&out, "rollback", &args)),
        }
    }

    fn mods_change(&self, r: &ModsRequest) -> Result<ModsReport, ServiceError> {
        let (out, args) = self.invoke(Action::Mods, r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Mods, &out, &args),
            _ => Err(failure(&out, "mods", &args)),
        }
    }

    fn conflicts(&self, r: &ConflictsRequest) -> Result<ConflictsReport, ServiceError> {
        let (out, args) = self.invoke(Action::Conflicts, r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Conflicts, &out, &args),
            _ => Err(failure(&out, "conflicts", &args)),
        }
    }

    fn tree(&self, r: &TreeRequest) -> Result<TreeReport, ServiceError> {
        let (out, args) = self.invoke(Action::Tree, r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Tree, &out, &args),
            _ => Err(failure(&out, "tree", &args)),
        }
    }

    fn setup(&self, r: &SetupRequest) -> Result<SetupReport, ServiceError> {
        self.setup_report(r)
    }
}

impl<E: Exec> CliService<E> {
    fn setup_report(&self, r: &SetupRequest) -> Result<SetupReport, ServiceError> {
        let (out, args) = self.invoke(Action::Setup, r.args())?;
        let mut wizard = SetupWizard::new();
        for line in out.stdout.lines() {
            let _ = wizard.feed_line(line);
        }
        let Phase::Finished(finish) = wizard.phase().clone() else {
            return Err(failure(&out, "setup", &args));
        };
        if let Some(mut card) = wizard.error() {
            card.repro = Some(repro(&args));
            return Err(ServiceError::Failed(card));
        }
        debug_assert!(matches!(
            finish.status,
            FinishStatus::Ok | FinishStatus::DryRun | FinishStatus::Restarted
        ));
        Ok(SetupReport {
            steps: wizard.steps().to_vec(),
            finish,
            dry_run: wizard.is_dry_run(),
            resume: wizard.is_resume(),
        })
    }

    fn tools(&self, verb: &str, r: &ToolsRequest) -> Result<ToolsReport, ServiceError> {
        let mut rest = vec![verb.to_owned()];
        rest.extend(r.args());
        let (out, args) = self.invoke(Action::Tools, rest)?;
        match out.code {
            Some(code @ (0 | 3)) => {
                let rows: Vec<ToolRow> = Self::parse(Action::Tools, &out, &args)?;
                Ok(ToolsReport {
                    rows,
                    verified: code == 0,
                })
            }
            _ => Err(failure(&out, "tools", &args)),
        }
    }
}
