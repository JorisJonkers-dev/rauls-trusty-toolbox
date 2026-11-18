// SPDX-License-Identifier: GPL-3.0-only
//! The [`Service`] trait and its real implementation, [`CliService`].

use crate::conflicts::{
    ConflictsReport, ConflictsRequest, PluginsReport, PluginsRequest, TreeReport, TreeRequest,
};
use crate::direct::run_direct;
use crate::dto::{
    DisagreeDoc, DoctorReport, Finding, ImportReport, InstallReport, ModsReport, QuestionsDoc,
    RollbackReport, SetupReport, SortReport, TargetReport, ToolRow, ToolsReport, TtwReport,
    VerifyReport,
};
use crate::exec::{CancelHandle, Exec, Output};
use crate::jobs::JobSpec;
use crate::registry::Action;
use raul_ui_model::error::{ExitClass, Retry, ViewError};
use raul_ui_model::job::JobId;
use raul_ui_model::mods::ModStep;
use raul_ui_model::setup::{FinishStatus, Phase, SetupWizard};
use raul_ui_model::wizard::{DEFAULT_PROFILE, FomodQuestion};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// Why a service call did not produce its report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    /// `import-mo2` found that the profile files contradict each other and wrote nothing.
    Disagree(Vec<Finding>),
    /// The call failed; the error is what a screen shows.
    Failed(ViewError),
}

/// What a screen says when an Instance has no profile and making the default one did not work either. The
/// headline and the guidance are plain words: the internal name of a profile never reaches a person.
pub const NO_PROFILE_WHAT: &str = "This Instance has no mod list yet";
/// What the same screen says about the effect: nothing changed, and what to do.
pub const NO_PROFILE_EFFECT: &str =
    "Nothing changed. Set the storage folder in Settings, then try again.";

/// Whether the words of a command name the profile an Instance does not have. The commands of nvmod refuse with
/// `the Instance has no profile`; that is the one refusal the Toolbox fixes itself.
pub(crate) fn names_a_missing_profile(words: &str) -> bool {
    let lower = words.to_lowercase();
    lower.contains("profile")
        && (lower.contains("no profile")
            || lower.contains("not have")
            || lower.contains("does not exist"))
}

/// The error a screen shows for a refusal that names a missing profile: the plain words above, with the command's
/// own line kept as the detail.
pub(crate) fn no_profile_error(said: &str) -> ViewError {
    let mut e = ViewError::new(ExitClass::Refused, NO_PROFILE_WHAT, NO_PROFILE_EFFECT);
    e.retry = Retry::AfterFix("set the storage folder, then try again".to_owned());
    let said = said.trim();
    if !said.is_empty() && !said.to_lowercase().contains("profile") {
        e.effect = format!("{NO_PROFILE_EFFECT} {said}");
    }
    e
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

/// What `raul import-mo2` needs.
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

/// What `raul install` needs.
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

/// What `raul ttw` needs. Anything left `None` comes from `raul config` (the game folders and the TTW package).
/// ttwright runs in raul's own process, so there is no program to locate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)] // One bool per flag of the command line.
pub struct TtwRequest {
    /// The Instance folder; `None` for the one `raul config` derives.
    pub instance: Option<String>,
    /// The Profile that gets the Mod; `None` for `Default`.
    pub profile: Option<String>,
    /// The Fallout 3 folder.
    pub fallout3: Option<String>,
    /// The New Vegas folder.
    pub newvegas: Option<String>,
    /// The TTW package file.
    pub package: Option<String>,
    /// Read the package and check the game copies only; nothing is written.
    pub dry_run: bool,
    /// Continue an interrupted run.
    pub resume: bool,
}

impl TtwRequest {
    /// The words after `raul ttw`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push_opt(&mut rest, "instance", self.instance.as_ref());
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push_opt(&mut rest, "fallout3", self.fallout3.as_ref());
        push_opt(&mut rest, "newvegas", self.newvegas.as_ref());
        push_opt(&mut rest, "package", self.package.as_ref());
        if self.dry_run {
            rest.push("--dry-run".to_owned());
        }
        if self.resume {
            rest.push("--resume".to_owned());
        }
        rest
    }
}

/// How `raul setup` is started.
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

/// What `raul setup` needs. A resume or a restart takes the Instance and nothing else, so [`SetupRequest::args`]
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
    /// Sort an imported MO2 Instance too (`--sort`); by default its order is kept.
    pub sort: bool,
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

    /// The words after `raul setup`, before `--json`.
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
            (self.sort, "--sort"),
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
    pub fn job_spec(&self, id: raul_ui_model::job::JobId) -> crate::jobs::JobSpec {
        let mut spec = crate::jobs::JobSpec::new(id, Action::Setup, self.args());
        spec.writes = !(self.dry_run && self.mode == SetupMode::Fresh);
        spec.dry_run = self.dry_run && self.mode == SetupMode::Fresh;
        spec.cancellable = spec.writes && !matches!(self.mode, SetupMode::Restart { .. });
        spec
    }
}

/// What one `raul mods enable|disable|reorder` call needs.
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
    /// The words after `raul mods`, before `--json`.
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

/// What `raul doctor --instance` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DoctorRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile.
    pub profile: Option<String>,
    /// The masterlist.
    pub masterlist: Option<String>,
}

/// What `raul sort` needs.
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
    /// The words after `raul sort`, before `--json`.
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

/// What `raul verify` needs.
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

/// What `raul target` needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRequest {
    /// The Instance folder.
    pub instance: String,
    /// What it should run on.
    pub target: RuntimeTarget,
}

impl TargetRequest {
    /// The words after `raul target`. The command has no `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = vec![self.target.word().to_owned()];
        push(&mut rest, "instance", &self.instance);
        rest
    }
}

/// What `raul tools list` and `raul tools verify` need.
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
    /// The words after `raul tools <verb>`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push_opt(&mut rest, "lock", self.lock.as_ref());
        push_opt(&mut rest, "store", self.store.as_ref());
        push_opt(&mut rest, "instance", self.instance.as_ref());
        rest
    }
}

/// What `raul rollback` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RollbackRequest {
    /// The Instance folder.
    pub instance: String,
}

/// What `raul profile create` needs: the default profile of an Instance that has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile's name; [`DEFAULT_PROFILE`] for the one setup makes.
    pub name: String,
}

impl ProfileRequest {
    /// The default profile of `instance`.
    pub fn default_for(instance: &str) -> Self {
        Self {
            instance: instance.to_owned(),
            name: DEFAULT_PROFILE.to_owned(),
        }
    }

    /// The words after `raul profile`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = vec!["create".to_owned()];
        push(&mut rest, "name", &self.name);
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
    /// `raul import-mo2`.
    ///
    /// # Errors
    /// [`ServiceError::Disagree`] when the profile files disagree, [`ServiceError::Failed`] otherwise.
    fn import_mo2(&self, request: &ImportRequest) -> Result<ImportReport, ServiceError>;

    /// `raul install`, also as a dry run.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed.
    fn install_mod(&self, request: &InstallRequest) -> Result<InstallResult, ServiceError>;

    /// `raul ttw`. Every run that printed its `raul-ttw-result` is a report, whatever its status (refused, failed,
    /// cancelled); its `error.code` says why, and `resumable` whether `--resume` continues it.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command could not run or printed no readable result.
    fn ttw(&self, request: &TtwRequest) -> Result<TtwReport, ServiceError>;

    /// `raul doctor --instance`. Findings of error severity are a report, not an error.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the Doctor could not run.
    fn doctor(&self, request: &DoctorRequest) -> Result<DoctorReport, ServiceError>;

    /// `raul sort`, a proposal or with `apply` the written order.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed (a failed apply is rolled back).
    fn sort(&self, request: &SortRequest) -> Result<SortReport, ServiceError>;

    /// `raul verify`. A verification that fails is a report with `verified` false, not an error.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the verification could not run.
    fn verify(&self, request: &VerifyRequest) -> Result<VerifyReport, ServiceError>;

    /// `raul target`.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed.
    fn set_target(&self, request: &TargetRequest) -> Result<TargetReport, ServiceError>;

    /// `raul tools list`.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the lock or the Instance could not be read.
    fn tools_list(&self, request: &ToolsRequest) -> Result<ToolsReport, ServiceError>;

    /// `raul tools verify`. A corrupt stored tool is a report with `verified` false, not an error.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the lock or the Instance could not be read.
    fn tools_verify(&self, request: &ToolsRequest) -> Result<ToolsReport, ServiceError>;

    /// `raul rollback`: undoes the last install or sort.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when there is nothing to undo or the undo failed.
    fn rollback(&self, request: &RollbackRequest) -> Result<RollbackReport, ServiceError>;

    /// One `raul mods enable|disable|reorder` call, also as a dry run.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed (a failed change is rolled back).
    fn mods_change(&self, request: &ModsRequest) -> Result<ModsReport, ServiceError>;

    /// `raul conflicts`: the file-level conflicts of a profile (read only).
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused.
    fn conflicts(&self, request: &ConflictsRequest) -> Result<ConflictsReport, ServiceError>;

    /// `raul tree`: the virtual Data folder of a profile (read only).
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused.
    fn tree(&self, request: &TreeRequest) -> Result<TreeReport, ServiceError>;

    /// `raul plugins list`: the plugins of a profile in load order (read only).
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused.
    fn plugins_list(&self, request: &PluginsRequest) -> Result<PluginsReport, ServiceError>;

    /// `raul setup`, run to its end and read from its event stream. A run that fails is an error carrying the
    /// card of the setup wizard; a dry run is a report. To follow a run while it goes, use
    /// [`CliService::run_job`] with [`SetupRequest::job_spec`].
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the run was refused or failed.
    fn setup(&self, request: &SetupRequest) -> Result<SetupReport, ServiceError>;

    /// `raul update check`: the release list of the Toolbox, read and nothing downloaded (ADR-0005).
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the check could not run, or its answer could not be read.
    fn update_check(&self) -> Result<UpdateReport, ServiceError>;

    /// `raul profile create`: the default profile of an Instance that has none. Setup makes it with its own
    /// steps, so a screen only asks for it when an Instance made another way has none.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the command refused or failed.
    fn profile_create(&self, request: &ProfileRequest) -> Result<(), ServiceError>;
}

/// What `raul update check --json` answered.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateReport {
    /// The version that runs.
    pub current: String,
    /// The newer release, when there is one.
    pub newer: Option<UpdateRelease>,
}

/// A release newer than the one that runs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateRelease {
    /// The version that is out, without its leading `v`.
    pub version: String,
    /// The release page.
    pub url: String,
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
    let mut s = String::from("raul");
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

/// The cause from standard error: the first non-empty line without the `raul <command>:` and `refused:`
/// prefixes the command line puts in front.
pub(crate) fn cause_of(stderr: &str, command: &str) -> String {
    let line = stderr.lines().map(str::trim).find(|l| !l.is_empty());
    let Some(line) = line else {
        return String::from("The command failed without saying why");
    };
    let line = line
        .strip_prefix(&format!("raul {command}:"))
        .map_or(line, str::trim);
    line.strip_prefix("refused:")
        .map_or(line, str::trim)
        .to_owned()
}

fn failure(out: &Output, command: &str, args: &[String]) -> ServiceError {
    let mut e = failure_error(out, command);
    e.repro = Some(repro(args));
    ServiceError::Failed(e)
}

/// The error of a command that failed, from its exit code and output, without its repro.
fn failure_error(out: &Output, command: &str) -> ViewError {
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
    let mut e = ViewError::new(class, cause.clone(), effect_of(class));
    e.retry = match class {
        ExitClass::StepFailedRolledBack => Retry::Yes,
        ExitClass::Refused | ExitClass::Usage | ExitClass::ProblemsFound => {
            Retry::AfterFix(String::from("fix what the message names, then try again"))
        }
        _ => Retry::No,
    };
    if names_a_missing_profile(&cause) {
        // A profile is an internal name: the headline and the guidance stay plain.
        let mut plain = no_profile_error(&cause);
        plain.repro = e.repro.take();
        return plain;
    }
    e
}

fn unreadable(what: &str, why: &serde_json::Error, args: &[String]) -> ServiceError {
    let mut e = ViewError::new(
        ExitClass::Internal,
        format!("The output of `raul {what}` could not be read: {why}"),
        effect_of(ExitClass::Internal),
    );
    e.repro = Some(repro(args));
    ServiceError::Failed(e)
}

/// Whether a command refused because the Instance has no profile: the one refusal the Toolbox answers by making
/// the default profile itself.
fn is_missing_profile(out: &Output) -> bool {
    out.code != Some(0)
        && (names_a_missing_profile(&out.stderr) || names_a_missing_profile(&out.stdout))
}

/// A scratch file holding the FOMOD answers, removed when dropped.
struct AnswersFile(PathBuf);

impl AnswersFile {
    fn write(answers: &serde_json::Value) -> Result<Self, ServiceError> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path =
            std::env::temp_dir().join(format!("raul-answers-{}-{nanos}.json", std::process::id()));
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

/// Runs the `raul` command line and reads its `--json` output.
#[derive(Debug, Clone)]
pub struct CliService<E: Exec> {
    exec: E,
}

impl<E: Exec> CliService<E> {
    /// A service over `exec`.
    pub fn new(exec: E) -> Self {
        Self { exec }
    }

    /// The `raul` words of a call: the command, `rest` and `--json` (commands without a JSON output do not get it).
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

impl TtwReport {
    /// Reads the `raul-ttw-result` from the `raul-events` stream `raul ttw --json` printed, as a finished job's
    /// [`Output`] holds it. The stream must end with `finished`, and its status must be the result's: a stream cut
    /// short (the command was killed after its result) or one that disagrees with itself is not a report.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the output holds no readable result, the stream is newer than this build, or
    /// the stream did not end as its result says. The error has no repro; the caller knows the command line.
    pub fn from_output(out: &Output) -> Result<TtwReport, ServiceError> {
        ttw_report(out, None)
    }
}

/// [`TtwReport::from_output`], with the repro of `args` on its errors when they are given.
fn ttw_report(out: &Output, args: Option<&[String]>) -> Result<TtwReport, ServiceError> {
    use raul_ui_model::events::{Event, LineError, parse_line};
    let unreadable = |class: ExitClass, what: String| {
        let mut e = ViewError::new(class, what, effect_of(class));
        e.repro = args.map(repro);
        ServiceError::Failed(e)
    };
    let (mut document, mut finished) = (None, None);
    for line in out.stdout.lines().filter(|l| !l.trim().is_empty()) {
        match parse_line(line) {
            Ok(parsed) => match parsed.event {
                Event::Result { result } => document = Some(result),
                Event::Finished { status, .. } => finished = Some(status),
                _ => {}
            },
            Err(LineError::Newer(v)) => {
                return Err(unreadable(
                    ExitClass::Internal,
                    format!(
                        "`raul ttw` speaks raul-events version {v}; this build reads version 1"
                    ),
                ));
            }
            Err(_) => {}
        }
    }
    let Some(document) = document else {
        let mut e = failure_error(out, "ttw");
        e.repro = args.map(repro);
        return Err(ServiceError::Failed(e));
    };
    let report = TtwReport::from_value(document).map_err(|why| {
        unreadable(
            ExitClass::Internal,
            format!("The result of `raul ttw` could not be read: {why}"),
        )
    })?;
    match finished {
        Some(status) if status == report.status => Ok(report),
        Some(status) => Err(unreadable(
            ExitClass::Internal,
            format!(
                "`raul ttw` ended as {status:?} but its result says {:?}",
                report.status
            ),
        )),
        None => Err(unreadable(
            ExitClass::Internal,
            "`raul ttw` stopped before it said how it ended".to_owned(),
        )),
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
        let rest = || {
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
            rest
        };
        let (out, args) = self.with_profile(Action::InstallMod, &r.instance, rest)?;
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
        let (out, args) = self.with_profile(
            Action::Ttw,
            r.instance.as_deref().unwrap_or_default(),
            || r.args(),
        )?;
        ttw_report(&out, Some(&args))
    }

    fn doctor(&self, r: &DoctorRequest) -> Result<DoctorReport, ServiceError> {
        let request = nvmod_api::doctor::CheckRequest {
            target: nvmod_api::doctor::Target::Instance {
                folder: PathBuf::from(&r.instance),
                profile: r.profile.clone(),
                game: None,
                masterlist: r.masterlist.as_deref().map(PathBuf::from),
                catalog: None,
            },
            fail_on: nvmod_api::doctor::Severity::Error,
        };
        let out = self.direct_with_profile(Action::Doctor, &r.instance, || {
            crate::direct::doctor(&request)
        })?;
        let unreadable_output = |why: &dyn std::fmt::Display| {
            ServiceError::Failed(ViewError::new(
                ExitClass::Internal,
                format!("The output of `raul doctor` could not be read: {why}"),
                effect_of(ExitClass::Internal),
            ))
        };
        match out.code {
            // Exit 1 is "findings at or above the failing severity": the report is on standard output.
            Some(0 | 1) => match serde_json::from_str::<DoctorReport>(&out.stdout) {
                Ok(report) => Ok(report),
                Err(e) if out.code == Some(1) => {
                    let message = serde_json::from_str::<serde_json::Value>(&out.stdout)
                        .ok()
                        .and_then(|v| v["error"].as_str().map(str::to_owned));
                    match message {
                        Some(m) => Err(ServiceError::Failed(ViewError::new(
                            ExitClass::ProblemsFound,
                            m,
                            effect_of(ExitClass::ProblemsFound),
                        ))),
                        None => Err(unreadable_output(&e)),
                    }
                }
                Err(e) => Err(unreadable_output(&e)),
            },
            _ => Err(ServiceError::Failed(failure_error(&out, "doctor"))),
        }
    }

    fn sort(&self, r: &SortRequest) -> Result<SortReport, ServiceError> {
        let (out, args) = self.with_profile(Action::Sort, &r.instance, || r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Sort, &out, &args),
            _ => Err(failure(&out, "sort", &args)),
        }
    }

    fn verify(&self, r: &VerifyRequest) -> Result<VerifyReport, ServiceError> {
        let request = nvmod_api::verify::VerifyRequest {
            instance: PathBuf::from(&r.instance),
            profile: r.profile.clone(),
            masterlist: r.masterlist.as_deref().map(PathBuf::from),
            catalog: r.catalog.as_deref().map(PathBuf::from),
            proton: r.proton.clone(),
            fail_on: match r.fail_on.as_deref().map(str::parse) {
                None => nvmod_api::verify::Severity::Error,
                Some(Ok(severity)) => severity,
                Some(Err(why)) => {
                    return Err(ServiceError::Failed(ViewError::new(
                        ExitClass::Usage,
                        format!("--fail-on: {why}"),
                        effect_of(ExitClass::Usage),
                    )));
                }
            },
        };
        let out = self.direct_with_profile(Action::Verify, &r.instance, || {
            crate::direct::verify(&request)
        })?;
        match out.code {
            // Exit 3 is "not verified": the report is on standard output.
            Some(0 | 3) => serde_json::from_str::<VerifyReport>(&out.stdout).map_err(|why| {
                ServiceError::Failed(ViewError::new(
                    ExitClass::Internal,
                    format!("The output of `raul verify` could not be read: {why}"),
                    effect_of(ExitClass::Internal),
                ))
            }),
            _ => Err(ServiceError::Failed(failure_error(&out, "verify"))),
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
        // The job the Undo button starts, run to its end: in process, under the Instance lock, no command line.
        let spec = JobSpec::new(JobId(0), Action::Rollback, Vec::new()).writing();
        let out = run_direct(
            &spec,
            crate::direct::rollback(Path::new(&r.instance)),
            &mut |_| {},
            &CancelHandle::new(),
        );
        if out.code != Some(0) {
            return Err(ServiceError::Failed(failure_error(&out, "rollback")));
        }
        serde_json::from_str(&out.stdout).map_err(|why| {
            ServiceError::Failed(ViewError::new(
                ExitClass::Internal,
                format!("The result of the rollback could not be read: {why}"),
                effect_of(ExitClass::Internal),
            ))
        })
    }

    fn mods_change(&self, r: &ModsRequest) -> Result<ModsReport, ServiceError> {
        let (out, args) = self.with_profile(Action::Mods, &r.instance, || r.args())?;
        match out.code {
            Some(0) => Self::parse(Action::Mods, &out, &args),
            _ => Err(failure(&out, "mods", &args)),
        }
    }

    fn conflicts(&self, r: &ConflictsRequest) -> Result<ConflictsReport, ServiceError> {
        let request = nvmod_api::conflicts::ReportRequest {
            instance: PathBuf::from(&r.instance),
            profile: r.profile.clone(),
            mod_name: r.mod_name.clone(),
            archives: false,
        };
        let out = self.direct_with_profile(Action::Conflicts, &r.instance, || {
            crate::direct::conflicts(&request)
        })?;
        Self::direct_answer(&out, "conflicts")
    }

    fn tree(&self, r: &TreeRequest) -> Result<TreeReport, ServiceError> {
        let request = nvmod_api::tree::TreeRequest {
            instance: PathBuf::from(&r.instance),
            profile: r.profile.clone(),
            path: r.path.clone(),
        };
        let out =
            self.direct_with_profile(Action::Tree, &r.instance, || crate::direct::tree(&request))?;
        Self::direct_answer(&out, "tree")
    }

    fn plugins_list(&self, r: &PluginsRequest) -> Result<PluginsReport, ServiceError> {
        let request = nvmod_api::plugins::ListRequest {
            instance: PathBuf::from(&r.instance),
            profile: r.profile.clone(),
        };
        let out = self.direct_with_profile(Action::Plugins, &r.instance, || {
            crate::direct::plugins_list(&request)
        })?;
        Self::direct_answer(&out, "plugins")
    }

    fn setup(&self, r: &SetupRequest) -> Result<SetupReport, ServiceError> {
        self.setup_report(r)
    }

    fn update_check(&self) -> Result<UpdateReport, ServiceError> {
        let (out, args) = self.invoke(Action::Update, vec!["check".to_owned()])?;
        match out.code {
            Some(0) => Self::parse(Action::Update, &out, &args),
            _ => Err(failure(&out, "update check", &args)),
        }
    }

    fn profile_create(&self, r: &ProfileRequest) -> Result<(), ServiceError> {
        self.profile_create_checked(r)
    }
}

impl<E: Exec> CliService<E> {
    /// Runs `action` with `rest`; when it refused because the Instance has no profile, makes the default one and
    /// runs it once more. This is the one place a missing profile is handled, so no screen has to know about it.
    fn with_profile(
        &self,
        action: Action,
        instance: &str,
        rest: impl Fn() -> Vec<String>,
    ) -> Result<(Output, Vec<String>), ServiceError> {
        let (out, args) = self.invoke(action, rest())?;
        if !is_missing_profile(&out) {
            return Ok((out, args));
        }
        self.profile_create_checked(&ProfileRequest::default_for(instance))?;
        self.invoke(action, rest())
    }

    /// Runs the direct job `make` makes for `action`; when it refused because the Instance has no profile, makes the
    /// default one and runs it once more (the direct counterpart of [`Self::with_profile`]). Nothing here starts a
    /// process for the job itself.
    fn direct_with_profile(
        &self,
        action: Action,
        instance: &str,
        make: impl Fn() -> crate::direct::DirectJob,
    ) -> Result<Output, ServiceError> {
        let run = || {
            let spec = JobSpec::new(JobId(0), action, Vec::new());
            run_direct(&spec, make(), &mut |_| {}, &CancelHandle::new())
        };
        let out = run();
        if !is_missing_profile(&out) {
            return Ok(out);
        }
        self.profile_create_checked(&ProfileRequest::default_for(instance))?;
        Ok(run())
    }

    /// The report a finished direct job printed, or the error it ended with. Only exit 0 is an answer here.
    fn direct_answer<T: DeserializeOwned>(out: &Output, command: &str) -> Result<T, ServiceError> {
        if out.code != Some(0) {
            return Err(ServiceError::Failed(failure_error(out, command)));
        }
        serde_json::from_str(&out.stdout).map_err(|why| {
            ServiceError::Failed(ViewError::new(
                ExitClass::Internal,
                format!("The result of {command} could not be read: {why}"),
                effect_of(ExitClass::Internal),
            ))
        })
    }

    /// Makes the default profile of an Instance that has none, and says plainly when that did not work.
    fn profile_create_checked(&self, r: &ProfileRequest) -> Result<(), ServiceError> {
        let (out, args) = self.invoke(Action::Profile, r.args())?;
        if out.code == Some(0) {
            return Ok(());
        }
        // A profile that is already there is what was asked for: the command refuses nothing when it is.
        if is_missing_profile(&out) {
            return Ok(());
        }
        let mut e = no_profile_error(&cause_of(&out.stderr, "profile"));
        e.repro = Some(repro(&args));
        Err(ServiceError::Failed(e))
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(why: &str) -> Output {
        Output {
            code: Some(2),
            stdout: String::new(),
            stderr: format!("raul plugins: refused: {why}\n"),
        }
    }

    #[test]
    fn a_refusal_naming_a_profile_is_plain_and_keeps_its_detail() {
        let e = failure_error(&refused("the Instance has no profile"), "plugins");
        assert_eq!(e.what, NO_PROFILE_WHAT);
        assert_eq!(e.exit, ExitClass::Refused);
        assert!(
            !e.effect.to_lowercase().contains("profile"),
            "the detail that repeats the internal name is dropped: {}",
            e.effect
        );
        let other = failure_error(&refused("no such plugin"), "plugins");
        assert_eq!(other.what, "no such plugin");
        assert!(names_a_missing_profile("the Instance has no profile"));
        assert!(!names_a_missing_profile("no such plugin"));
        assert_eq!(ProfileRequest::default_for("/i").name, DEFAULT_PROFILE);
        assert_eq!(
            ProfileRequest::default_for("/i").args(),
            ["create", "--name", "Default", "--instance", "/i"]
        );
    }

    /// A direct job that refuses `times` times as an Instance with no profile does, then answers an empty report.
    fn needs_a_profile(
        runs: &std::sync::Arc<std::sync::atomic::AtomicUsize>,
        times: usize,
    ) -> crate::direct::DirectJob {
        let runs = std::sync::Arc::clone(runs);
        crate::direct::DirectJob::reading(move |_, _| {
            let n = runs.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n < times {
                return Err(nvmod_api::JobError::new(
                    nvmod_api::ErrorKind::Refused,
                    "test.read",
                    "the Instance has no profile",
                ));
            }
            Ok(nvmod_api::Outcome::Done(
                serde_json::json!({"profile": "Default"}),
            ))
        })
    }

    #[test]
    fn a_missing_profile_is_made_before_the_direct_job_runs_again() {
        let exec = crate::fake::ScriptedExec::new();
        exec.on_run(&[], 0, "");
        let svc = CliService::new(exec);
        let runs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let out = svc
            .direct_with_profile(Action::Plugins, "/i", || needs_a_profile(&runs, 1))
            .unwrap();
        assert_eq!(out.code, Some(0));
        assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 2);
        let calls = svc.exec().calls();
        assert_eq!(calls.len(), 1, "only the profile is made by a command");
        assert_eq!(calls[0][..2], ["profile", "create"]);
    }

    #[test]
    fn a_direct_job_that_works_first_time_never_makes_a_profile() {
        let svc = CliService::new(crate::fake::ScriptedExec::new());
        let runs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        svc.direct_with_profile(Action::Plugins, "/i", || needs_a_profile(&runs, 0))
            .unwrap();
        assert!(svc.exec().calls().is_empty());
    }

    #[test]
    fn a_profile_that_cannot_be_made_is_a_plain_error() {
        let exec = crate::fake::ScriptedExec::new();
        exec.on_run(
            &[],
            2,
            "raul profile: refused: the Instance folder is not there\n",
        );
        let svc = CliService::new(exec);
        let runs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let err = svc
            .direct_with_profile(Action::Plugins, "/i", || needs_a_profile(&runs, 9))
            .unwrap_err();
        let e = err.view_error();
        assert_eq!(e.what, NO_PROFILE_WHAT);
        assert!(e.effect.starts_with(NO_PROFILE_EFFECT));
        assert_eq!(
            runs.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "it is not tried again"
        );
    }
}
