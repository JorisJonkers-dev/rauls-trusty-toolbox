// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the TTW flow (UI-RULES P4): four steps, Sources, Output, Review and Install.
//!
//! The sources (the Fallout 3 folder, the New Vegas folder and the TTW package) come from `raul config list`; their
//! home is Settings (UI-RULES P3). The output is the Instance: `raul ttw` adds TTW to it as one Mod and enables its
//! plugins in the Profile. Review runs `raul ttw --dry-run`, which reads the package and checks both game copies
//! without writing anything. Install runs `raul ttw` as a job that keeps going when the page is left; a failed or
//! cancelled run leaves the Instance as it was, and `--resume` continues it. The plan, review and apply rule is
//! the one of [`TtwWizard`].

use super::load_order::Problem;
use super::settings::{Command, Config, Fetch, Key};
use crate::error::{ExitClass, ViewError};
use crate::events::Status;
use crate::job::Outcome;
use crate::wizard::{FlowError, Step as FlowStep, TtwNeed, TtwWizard};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Where the TTW package is downloaded from: the one place the page links to.
pub const DOWNLOAD_URL: &str = "https://taleoftwowastelands.com/";

/// `raul ttw --instance <dir> --dry-run`: reads the package and checks the game copies; nothing is written.
pub fn check_command(instance: &str) -> Command {
    Command::read("ttw", 1, &["ttw", "--instance", instance, "--dry-run"])
}

/// `raul ttw --instance <dir>`, with `--resume` to continue a stopped run: installs TTW into the Instance.
pub fn install_command(instance: &str, resume: bool) -> Command {
    let mut words = vec!["ttw", "--instance", instance];
    if resume {
        words.push("--resume");
    }
    Command::write("ttw", 1, &words)
}

/// A step of the flow, as its rail shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Step {
    /// The Fallout 3 folder, the New Vegas folder and the TTW package.
    #[default]
    Sources,
    /// Where TTW goes: the Instance.
    Output,
    /// The checks of the dry run, and what the install will do.
    Review,
    /// The install, its progress and how it ended.
    Install,
}

impl Step {
    /// Every step, in order.
    pub const ALL: [Step; 4] = [Step::Sources, Step::Output, Step::Review, Step::Install];

    /// Its number, 1 to 4.
    pub fn number(self) -> usize {
        match self {
            Step::Sources => 1,
            Step::Output => 2,
            Step::Review => 3,
            Step::Install => 4,
        }
    }
}

/// The sources of the install, as `raul config` has them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sources {
    /// `game.fallout3`.
    pub fallout3: Option<String>,
    /// `game.newvegas`; `raul ttw` uses the Instance's base copy when it is not set.
    pub newvegas: Option<String>,
    /// `ttw.package`.
    pub package: Option<String>,
}

impl Sources {
    /// The sources in a configuration.
    pub fn of(config: &Config) -> Sources {
        let get = |key| config.get(key).map(str::to_owned);
        Sources {
            fallout3: get(Key::Fallout3),
            newvegas: get(Key::NewVegas),
            package: get(Key::TtwPackage),
        }
    }
}

/// What a run of `raul ttw` reported, as the page shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// How it ended.
    pub status: Status,
    /// A dry run: nothing was written.
    pub dry_run: bool,
    /// The plugins the Profile enabled with the Mod, in load order.
    pub plugins: Vec<String>,
    /// Files in the Mod, when one was added.
    pub files: Option<u64>,
    /// `raul ttw --resume` continues this run.
    pub resumable: bool,
    /// The installer program, as it is called; its version is a Component version and is not shown.
    pub tool: Option<String>,
    /// The stable error code, unless it succeeded.
    pub code: Option<String>,
    /// The error message, unless it succeeded.
    pub message: Option<String>,
}

impl Report {
    /// A report of `status` with nothing else in it.
    pub fn of(status: Status, dry_run: bool) -> Report {
        Report {
            status,
            dry_run,
            plugins: Vec::new(),
            files: None,
            resumable: false,
            tool: None,
            code: None,
            message: None,
        }
    }

    /// The exit class its status stands for.
    pub fn exit(&self) -> ExitClass {
        match self.status {
            Status::Success => ExitClass::Ok,
            Status::Refused | Status::Unsupported => ExitClass::Refused,
            Status::Unverifiable => ExitClass::VerificationFailed,
            Status::Failed | Status::Cancelled => ExitClass::StepFailedRolledBack,
        }
    }
}

/// What the search for the package on this machine found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Found {
    /// The search did not run.
    #[default]
    Idle,
    /// The search runs: a listing of two levels of the folder.
    Searching,
    /// The newest package found, and the folder it was looked for in.
    Found {
        /// The package file.
        file: String,
        /// The folder the search looked in.
        folder: String,
    },
    /// No package was found, and the folder it was looked for in.
    NotFound {
        /// The folder the search looked in.
        folder: String,
    },
}

/// Where the package is looked for when the shell does not say: the Downloads folder of the home folder.
pub fn default_downloads() -> String {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    // Without a home folder the search is relative to where the window runs, in a folder of that name.
    home.map_or_else(
        || PathBuf::from("Downloads"),
        |h| PathBuf::from(h).join("Downloads"),
    )
    .to_string_lossy()
    .into_owned()
}

/// Whether `name` is a TTW installer package: `TTW_*.mpi`, in any case.
pub fn is_package(name: &str) -> bool {
    let lower = name.to_lowercase();
    let Some(rest) = lower.strip_prefix("ttw_") else {
        return false;
    };
    rest.strip_suffix(".mpi")
        .is_some_and(|stem| !stem.is_empty())
}

/// The newest TTW installer package in `folder` and in one level of folders under it, as its path; `None` when
/// there is none. A package that is not there is skipped, and of two of the same age the one that comes first.
pub fn find_package(folder: &Path) -> Option<PathBuf> {
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for dir in std::iter::once(folder.to_path_buf()).chain(subfolders(folder)) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !is_package(&name) {
                continue;
            }
            let Ok(modified) = entry.metadata().and_then(|m| m.modified()) else {
                continue;
            };
            if best.as_ref().is_none_or(|(age, _)| modified > *age) {
                best = Some((modified, entry.path()));
            }
        }
    }
    best.map(|(_, path)| path)
}

/// The smallest size a finished package has: a smaller file is a partial download.
pub const MIN_PACKAGE_BYTES: u64 = 1_000_000;

/// The four bytes a package starts with: it is a zip-like container.
const PACKAGE_HEADER: [u8; 4] = *b"PK\x03\x04";

/// A file that passed [`check_package`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    /// The package file.
    pub path: PathBuf,
    /// Its size in bytes.
    pub size: u64,
}

/// Why [`check_package`] refused a file, in the words the page says it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageProblem {
    /// The file cannot be read.
    NotThere,
    /// The name is not a TTW package name: a download in progress ends in `.part`, `.crdownload` or `.tmp`.
    WrongName,
    /// Smaller than [`MIN_PACKAGE_BYTES`]: a partial download.
    TooSmall,
    /// The first bytes are not the package header, so the file is not a package.
    NotAPackage,
}

impl PackageProblem {
    /// One short plain sentence saying why the file was refused: no tool names and no paths in it.
    pub fn why(&self) -> &'static str {
        match self {
            PackageProblem::NotThere => "The file cannot be read.",
            PackageProblem::WrongName => "The name is not a TTW package name.",
            PackageProblem::TooSmall => "The file is too small: the download did not finish.",
            PackageProblem::NotAPackage => {
                "The file does not start like a TTW package, so it is not one."
            }
        }
    }
}

/// Whether `path` can be the package: the name is a TTW package name, the file is at least
/// [`MIN_PACKAGE_BYTES`] bytes big and its first four bytes are the package header. Only the first four
/// bytes of the file are read.
///
/// # Errors
/// [`PackageProblem::NotThere`] when the file cannot be read, [`PackageProblem::WrongName`] when the name
/// is not a TTW package name (a download in progress ends in `.part`, `.crdownload` or `.tmp`),
/// [`PackageProblem::TooSmall`] when the file is a partial download and [`PackageProblem::NotAPackage`]
/// when the first bytes are not the package header.
pub fn check_package(path: &Path) -> Result<PackageInfo, PackageProblem> {
    let name = path.file_name().map_or_default(|n| n.to_string_lossy());
    if !is_package(&name) {
        return Err(PackageProblem::WrongName);
    }
    let mut file = std::fs::File::open(path).map_err(|_| PackageProblem::NotThere)?;
    let size = file.metadata().map_err(|_| PackageProblem::NotThere)?.len();
    if size < MIN_PACKAGE_BYTES {
        return Err(PackageProblem::TooSmall);
    }
    let mut head = [0_u8; 4];
    file.read_exact(&mut head)
        .map_err(|_| PackageProblem::NotThere)?;
    if head != PACKAGE_HEADER {
        return Err(PackageProblem::NotAPackage);
    }
    Ok(PackageInfo {
        path: path.to_path_buf(),
        size,
    })
}

/// The folders directly under `folder`, in the order the file system lists them.
fn subfolders(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect()
}

/// How far the install is, from the job's progress events.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Progress {
    /// The phase, as the stream names it (`package`, `assets`, `archives`, `post_install`, `add_mod`).
    pub phase: String,
    /// Units done in the phase.
    pub done: u64,
    /// Units in the phase; `None` while there is no estimate.
    pub total: Option<u64>,
}

impl Progress {
    /// The share of the phase that is done, 0 to 1; `None` while there is no estimate.
    pub fn fraction(&self) -> Option<f32> {
        let total = self.total.filter(|t| *t > 0)?;
        #[allow(clippy::cast_precision_loss)]
        let share = self.done.min(total) as f32 / total as f32;
        Some(share)
    }
}

/// The state of the TTW flow.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The step shown.
    pub step: Step,
    /// The Instance TTW goes into, as the shell knows it.
    pub instance: Option<String>,
    /// The sources, read from `raul config list`.
    pub sources: Fetch<Sources>,
    /// The plan, review and apply machine.
    pub wizard: TtwWizard,
    /// What the dry run of the review reported.
    pub check: Option<Report>,
    /// How far the install is.
    pub progress: Option<Progress>,
    /// How the install ended.
    pub result: Option<Report>,
    /// The last problem, shown until the next step starts.
    pub problem: Option<Problem>,
    /// The install runs with `--resume`, continuing a stopped run.
    pub resuming: bool,
    /// The search for the package on this machine.
    pub found: Found,
}

impl State {
    /// The flow for `instance`, before its sources are read.
    pub fn new(instance: Option<String>) -> State {
        State {
            instance,
            ..State::default()
        }
    }

    /// A wizard with the sources chosen.
    fn wizard_for(sources: &Sources) -> TtwWizard {
        let mut w = TtwWizard::new();
        if let Some(f) = &sources.fallout3 {
            w.set_fallout3(f.clone());
        }
        if let Some(p) = &sources.package {
            w.set_package(p.clone());
        }
        w
    }

    /// The sources are being read.
    pub fn loading(&mut self) {
        self.sources = Fetch::Loading;
    }

    /// The sources arrived.
    pub fn sources_read(&mut self, sources: Sources) {
        self.wizard = State::wizard_for(&sources);
        self.sources = Fetch::Ready(sources);
    }

    /// The sources could not be read; the command's message.
    pub fn sources_failed(&mut self, why: String) {
        self.sources = Fetch::Failed(why);
    }

    /// The search for the package started.
    pub fn searching(&mut self) {
        self.found = Found::Searching;
    }

    /// The search ended: with the package it found, or with nothing and the folder it looked in.
    pub fn searched(&mut self, folder: &Path, found: Option<&Path>) {
        let folder = folder.to_string_lossy().into_owned();
        self.found = match found {
            Some(file) => Found::Found {
                file: file.to_string_lossy().into_owned(),
                folder,
            },
            None => Found::NotFound { folder },
        };
    }

    /// The package found was chosen: it is the package the flow uses, until the sources are read again.
    pub fn use_package(&mut self, file: &str) {
        self.wizard.set_package(file);
        if let Fetch::Ready(sources) = &mut self.sources {
            sources.package = Some(file.to_owned());
        }
    }

    /// Puts back the package the flow had before the one found was chosen, as when its save was refused or failed:
    /// `wizard` is the package the wizard had and `sources` the one the sources had, `None` when there was none.
    pub fn restore_package(&mut self, wizard: Option<String>, sources: Option<String>) {
        match wizard {
            Some(file) => self.wizard.set_package(file),
            None => self.wizard.clear_package(),
        }
        if let Fetch::Ready(s) = &mut self.sources {
            s.package = sources;
        }
    }

    /// The sources were read again, as when the page is shown again after Settings. When they changed, the flow
    /// starts over from Sources with them; when they did not, or while a command runs or after the install ended,
    /// the flow stays where it is.
    pub fn refreshed(&mut self, sources: Sources) {
        if self.busy() || self.done() || self.sources.ready() == Some(&sources) {
            return;
        }
        *self = State {
            step: Step::Sources,
            ..State::new(self.instance.take())
        };
        self.sources_read(sources);
    }

    /// What is still missing before the flow can go past Sources; nothing is known missing before the read.
    pub fn missing(&self) -> Vec<TtwNeed> {
        if self.sources.ready().is_some() {
            self.wizard.needs()
        } else {
            Vec::new()
        }
    }

    /// Whether a command of the flow runs: the sources are read, the review checks or the install runs.
    pub fn busy(&self) -> bool {
        self.sources == Fetch::Loading
            || matches!(
                self.wizard.flow.step(),
                FlowStep::Planning | FlowStep::Applying | FlowStep::Cancelling
            )
    }

    /// Whether Next is enabled on the step shown.
    pub fn can_next(&self) -> bool {
        match self.step {
            Step::Sources => self.sources.ready().is_some() && self.missing().is_empty(),
            Step::Output => self.instance.is_some(),
            Step::Review => self.wizard.flow.can_apply(),
            Step::Install => false,
        }
    }

    /// Next on Sources or Output. On Output it moves to the review, where the page starts the dry run with
    /// [`State::checking`].
    pub fn next(&mut self) {
        if !self.can_next() {
            return;
        }
        self.step = match self.step {
            Step::Sources => Step::Output,
            Step::Output => Step::Review,
            other => other,
        };
    }

    /// Back on Output or Review. It waits while a command runs, so nothing runs against a step left behind.
    pub fn back(&mut self) {
        if self.busy() {
            return;
        }
        match self.step {
            Step::Output => self.step = Step::Sources,
            Step::Review => {
                self.step = Step::Output;
                self.reset_review();
            }
            Step::Sources | Step::Install => {}
        }
    }

    /// Forgets the dry run: the wizard is back at its input.
    fn reset_review(&mut self) {
        if let Some(sources) = self.sources.ready() {
            self.wizard = State::wizard_for(sources);
        }
        self.check = None;
        self.problem = None;
    }

    /// Whether the review waits for its dry run to be started: it is shown, and no check ran or runs.
    pub fn needs_check(&self) -> bool {
        self.step == Step::Review
            && self.wizard.flow.step() == FlowStep::Input
            && self.check.is_none()
            && self.problem.is_none()
    }

    /// The dry run of the review was started.
    ///
    /// # Errors
    /// [`FlowError::Blocked`] while a source is missing, or [`FlowError::WrongStep`].
    pub fn checking(&mut self) -> Result<(), FlowError> {
        self.reset_review();
        self.wizard.plan()
    }

    /// The dry run reported. A success is the review, shown at once; anything else is `failed`, and the flow
    /// stays on the review so the check can run again.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when no check runs.
    pub fn checked(&mut self, report: Report, failed: Problem) -> Result<(), FlowError> {
        if report.status == Status::Success {
            self.wizard.flow.plan_ready(0)?;
            self.wizard.flow.mark_reviewed()?;
        } else {
            let what = report.message.clone().unwrap_or_default();
            self.wizard.flow.plan_failed(ViewError::new(
                report.exit(),
                what,
                Outcome::Refused.words(),
            ))?;
            self.problem = Some(failed);
        }
        self.check = Some(report);
        Ok(())
    }

    /// The dry run did not report: it could not start or its output could not be read.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when no check runs.
    pub fn check_failed(&mut self, problem: Problem) -> Result<(), FlowError> {
        self.wizard.flow.plan_failed(ViewError::new(
            ExitClass::Internal,
            problem.cause.clone(),
            Outcome::Refused.words(),
        ))?;
        self.problem = Some(problem);
        Ok(())
    }

    /// Install was started.
    ///
    /// # Errors
    /// See [`TtwWizard::apply`].
    pub fn installing(&mut self) -> Result<(), FlowError> {
        self.wizard.apply()?;
        self.step = Step::Install;
        self.progress = None;
        self.result = None;
        self.problem = None;
        Ok(())
    }

    /// The install's job reported progress; progress after the end is dropped.
    pub fn progressed(&mut self, progress: Progress) {
        if matches!(
            self.wizard.flow.step(),
            FlowStep::Applying | FlowStep::Cancelling
        ) {
            self.progress = Some(progress);
        }
    }

    /// Cancel was asked of the install.
    ///
    /// # Errors
    /// See [`crate::wizard::Flow::cancel`].
    pub fn cancelling(&mut self) -> Result<(), FlowError> {
        self.wizard.flow.cancel()
    }

    /// The install reported how it ended; `failed` is the problem shown when it did not succeed.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when no install runs.
    pub fn installed(&mut self, report: Report, failed: Problem) -> Result<(), FlowError> {
        let outcome = report.status.outcome(report.dry_run);
        self.wizard.flow.finish(report.exit(), outcome)?;
        if report.status != Status::Success {
            self.problem = Some(failed);
        }
        self.result = Some(report);
        Ok(())
    }

    /// The install did not report: it could not start, or it was cut off before it said how it ended. The state
    /// of the Instance is not known then.
    ///
    /// # Errors
    /// [`FlowError::WrongStep`] when no install runs.
    pub fn install_failed(&mut self, problem: Problem) -> Result<(), FlowError> {
        self.wizard
            .flow
            .finish(ExitClass::Internal, Outcome::Failed)?;
        self.problem = Some(problem);
        Ok(())
    }

    /// Whether the install has ended.
    pub fn done(&self) -> bool {
        self.wizard.flow.step() == FlowStep::Done
    }

    /// How the install ended, once it has.
    pub fn outcome(&self) -> Option<Outcome> {
        self.wizard.flow.result().map(|(_, outcome)| outcome)
    }

    /// Whether the ended install can be continued with `--resume`.
    pub fn resumable(&self) -> bool {
        self.done() && self.result.as_ref().is_some_and(|r| r.resumable)
    }

    /// Resume or Try again after an install that did not succeed: back to the review. With `resume` the review
    /// of the stopped run still holds and Install continues it; without, the dry run runs again first.
    pub fn again(&mut self, resume: bool) {
        if !self.done() {
            return;
        }
        let check = self.check.take();
        self.reset_review();
        self.progress = None;
        self.result = None;
        self.resuming = false;
        self.step = Step::Review;
        if resume
            && self.wizard.plan().is_ok()
            && self.wizard.flow.plan_ready(0).is_ok()
            && self.wizard.flow.mark_reviewed().is_ok()
        {
            self.check = check;
            self.resuming = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem() -> Problem {
        Problem {
            id: "x",
            cause: "it broke".into(),
            command: None,
        }
    }

    fn ready() -> State {
        let mut s = State::new(Some("/i".into()));
        s.loading();
        assert!(s.busy());
        s.sources_read(Sources {
            fallout3: Some("/fo3".into()),
            newvegas: None,
            package: Some("/p.mpi".into()),
        });
        s
    }

    fn success(dry_run: bool) -> Report {
        Report {
            plugins: vec!["A.esm".into()],
            ..Report::of(Status::Success, dry_run)
        }
    }

    #[test]
    fn the_sources_come_from_the_configuration() {
        let c = Config::with(
            "/c",
            &[(Key::Fallout3, "/fo3"), (Key::TtwPackage, "/p.mpi")],
        );
        let s = Sources::of(&c);
        assert_eq!(s.fallout3.as_deref(), Some("/fo3"));
        assert_eq!(s.package.as_deref(), Some("/p.mpi"));
        assert!(s.newvegas.is_none());
    }

    #[test]
    fn a_missing_source_holds_the_flow_on_sources() {
        let mut s = State::new(Some("/i".into()));
        assert!(!s.can_next(), "nothing read yet");
        assert!(
            s.missing().is_empty(),
            "nothing is known missing before the read"
        );
        s.sources_read(Sources::default());
        assert_eq!(s.missing(), vec![TtwNeed::Fallout3, TtwNeed::Package]);
        s.next();
        assert_eq!(s.step, Step::Sources);
        s.sources_failed("unreadable".into());
        assert!(!s.can_next());
    }

    #[test]
    fn sources_output_review_install_walk_in_order() {
        let mut s = ready();
        assert!(!s.busy());
        s.next();
        assert_eq!(s.step, Step::Output);
        s.next();
        assert_eq!(s.step, Step::Review);
        assert!(s.needs_check() && !s.can_next(), "the check has not run");
        s.checking().unwrap();
        assert!(s.busy() && !s.needs_check());
        s.checked(success(true), problem()).unwrap();
        assert!(s.can_next() && s.problem.is_none());
        s.installing().unwrap();
        assert_eq!(s.step, Step::Install);
        s.progressed(Progress {
            phase: "assets".into(),
            done: 1,
            total: Some(4),
        });
        assert_eq!(s.progress.as_ref().and_then(Progress::fraction), Some(0.25));
        s.installed(success(false), problem()).unwrap();
        assert!(s.done() && !s.busy());
        assert_eq!(s.outcome(), Some(Outcome::Completed));
        assert!(s.problem.is_none() && !s.resumable());
        s.progressed(Progress::default());
        assert!(
            s.progress.as_ref().is_some_and(|p| p.done == 1),
            "late progress is dropped"
        );
    }

    #[test]
    fn sources_read_again_start_over_only_when_they_changed() {
        let mut s = ready();
        s.next();
        let same = s.sources.ready().cloned().unwrap();
        s.refreshed(same.clone());
        assert_eq!(s.step, Step::Output, "unchanged sources keep the step");
        let changed = Sources {
            package: Some("/other.mpi".into()),
            ..same.clone()
        };
        s.step = Step::Review;
        s.checking().unwrap();
        s.refreshed(changed.clone());
        assert_eq!(s.step, Step::Review, "a running check is not cut off");
        s.checked(success(true), problem()).unwrap();
        s.refreshed(changed.clone());
        assert_eq!(s.step, Step::Sources);
        assert!(s.check.is_none() && s.instance.as_deref() == Some("/i"));
        assert_eq!(s.wizard.package(), Some("/other.mpi"));
        assert!(s.can_next());
    }

    #[test]
    fn a_failed_check_stays_on_the_review_and_runs_again() {
        let mut s = ready();
        s.step = Step::Review;
        s.checking().unwrap();
        let refused = Report {
            code: Some("missing_input".into()),
            message: Some("the package is not a file".into()),
            ..Report::of(Status::Refused, true)
        };
        s.checked(refused, problem()).unwrap();
        assert!(s.problem.is_some() && !s.can_next() && !s.needs_check());
        assert_eq!(s.wizard.flow.step(), FlowStep::Input);
        s.checking().unwrap();
        assert!(s.problem.is_none(), "a new check clears the problem");
        s.check_failed(problem()).unwrap();
        assert!(s.problem.is_some());
        assert!(
            s.checked(success(true), problem()).is_err(),
            "no check runs"
        );
    }

    #[test]
    fn back_forgets_the_review_and_waits_while_a_command_runs() {
        let mut s = ready();
        s.next();
        s.next();
        s.checking().unwrap();
        s.back();
        assert_eq!(s.step, Step::Review, "the check runs");
        s.checked(success(true), problem()).unwrap();
        s.back();
        assert_eq!(s.step, Step::Output);
        assert!(s.check.is_none() && s.wizard.flow.step() == FlowStep::Input);
        s.back();
        assert_eq!(s.step, Step::Sources);
        s.back();
        assert_eq!(s.step, Step::Sources);
    }

    #[test]
    fn a_cancelled_install_can_resume_and_a_failed_one_checks_again() {
        let mut s = ready();
        s.step = Step::Review;
        s.checking().unwrap();
        s.checked(success(true), problem()).unwrap();
        s.installing().unwrap();
        s.again(true);
        assert_eq!(s.step, Step::Install, "nothing to resume while it runs");
        s.cancelling().unwrap();
        assert!(s.busy());
        let cancelled = Report {
            resumable: true,
            ..Report::of(Status::Cancelled, false)
        };
        s.installed(cancelled, problem()).unwrap();
        assert_eq!(s.outcome(), Some(Outcome::Cancelled));
        assert!(s.resumable() && s.problem.is_some());
        s.again(true);
        assert!(s.resuming && s.can_next(), "a resume needs no new check");
        assert!(s.check.is_some() && s.problem.is_none());
        assert_eq!(s.step, Step::Review);
        s.installing().unwrap();
        s.install_failed(problem()).unwrap();
        assert_eq!(s.outcome(), Some(Outcome::Failed));
        assert!(!s.resumable());
        s.again(false);
        assert!(!s.resuming && !s.can_next(), "the review runs again first");
        assert!(s.check.is_none() && s.needs_check());
    }

    #[test]
    fn the_commands_name_the_instance_and_only_the_install_writes() {
        let c = check_command("/a b");
        assert_eq!(c.shown(), "raul ttw --instance '/a b' --dry-run");
        assert_eq!(c.rest(), ["--instance", "/a b", "--dry-run"]);
        assert!(!c.writes && install_command("/i", false).writes);
        assert_eq!(
            install_command("/i", true).shown(),
            "raul ttw --instance /i --resume"
        );
        assert_eq!(Step::ALL.map(Step::number), [1, 2, 3, 4]);
    }

    fn touching(dir: &Path, name: &str, age: u64) {
        let path = dir.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"synthetic package").unwrap();
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000 + age);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }

    /// A folder of its own for one test, removed when it ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Scratch {
            let root =
                std::env::temp_dir().join(format!("raul-ttw-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).unwrap();
            Scratch(root)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_package_is_a_ttw_mpi_file_in_any_case() {
        assert!(is_package("TTW_3.4.0.mpi"));
        assert!(is_package("ttw_something.MPI"));
        assert!(!is_package("TTW_3.4.0.7z"));
        assert!(!is_package("other.mpi"));
        assert!(!is_package("TTW_.mpi"));
        assert!(default_downloads().ends_with("Downloads"));
    }

    #[test]
    fn the_search_takes_the_newest_package_of_two_levels() {
        let scratch = Scratch::new("search");
        let root = &scratch.0;
        assert_eq!(find_package(root), None, "an empty folder");
        touching(root, "notes.txt", 500);
        touching(root, "TTW_old.mpi", 100);
        touching(root, "ttw_other.MPI", 300);
        assert_eq!(
            find_package(root).unwrap().file_name().unwrap(),
            "ttw_other.MPI"
        );
        // One level under the folder counts; two do not.
        touching(root, "unpacked/TTW_new.mpi", 900);
        touching(root, "unpacked/deeper/TTW_deepest.mpi", 9_999);
        assert_eq!(
            find_package(root).unwrap().file_name().unwrap(),
            "TTW_new.mpi"
        );
        assert_eq!(find_package(&root.join("missing")), None);
    }

    #[test]
    fn a_package_found_is_used_until_the_sources_are_read_again() {
        let mut s = ready();
        assert_eq!(s.found, Found::Idle);
        s.searching();
        assert_eq!(s.found, Found::Searching);
        s.searched(Path::new("/d"), None);
        assert_eq!(
            s.found,
            Found::NotFound {
                folder: "/d".into()
            }
        );
        s.searched(Path::new("/d"), Some(Path::new("/d/TTW_1.mpi")));
        assert_eq!(
            s.found,
            Found::Found {
                file: "/d/TTW_1.mpi".into(),
                folder: "/d".into()
            }
        );
        s.use_package("/d/TTW_1.mpi");
        assert_eq!(s.wizard.package(), Some("/d/TTW_1.mpi"));
        assert_eq!(
            s.sources.ready().and_then(|f| f.package.as_deref()),
            Some("/d/TTW_1.mpi")
        );
        assert!(s.missing().is_empty());
        assert_eq!(DOWNLOAD_URL, "https://taleoftwowastelands.com/");
    }

    #[test]
    fn a_package_that_was_not_saved_is_put_back() {
        let mut s = ready();
        s.use_package("/d/TTW_1.mpi");
        assert_eq!(s.wizard.package(), Some("/d/TTW_1.mpi"));
        s.restore_package(Some("/p.mpi".into()), Some("/p.mpi".into()));
        assert_eq!(s.wizard.package(), Some("/p.mpi"));
        assert_eq!(
            s.sources.ready().and_then(|f| f.package.as_deref()),
            Some("/p.mpi")
        );
        // The flow had no package at all: the one found is not used.
        s.use_package("/d/TTW_1.mpi");
        s.restore_package(None, None);
        assert_eq!(s.wizard.package(), None);
        assert_eq!(s.sources.ready().and_then(|f| f.package.as_deref()), None);
        assert_eq!(s.missing(), vec![TtwNeed::Package]);
    }

    #[test]
    fn progress_without_an_estimate_has_no_share_and_never_passes_the_whole() {
        let p = Progress {
            phase: "package".into(),
            done: 3,
            total: None,
        };
        assert_eq!(p.fraction(), None);
        let over = Progress {
            total: Some(2),
            ..p.clone()
        };
        assert_eq!(over.fraction(), Some(1.0));
        let zero = Progress {
            total: Some(0),
            ..p
        };
        assert_eq!(zero.fraction(), None);
        assert_eq!(
            Report::of(Status::Unverifiable, false).exit(),
            ExitClass::VerificationFailed
        );
    }

    /// A package-shaped body of `len` bytes: the header, then whatever fills it out.
    fn package_body(len: usize) -> Vec<u8> {
        let mut body = vec![0; len];
        body[..PACKAGE_HEADER.len()].copy_from_slice(&PACKAGE_HEADER);
        body
    }

    fn big() -> usize {
        usize::try_from(MIN_PACKAGE_BYTES).unwrap()
    }

    #[test]
    fn a_package_that_is_there_and_big_enough_passes_with_its_size() {
        let scratch = Scratch::new("check-good");
        let path = scratch.0.join("TTW_3.4.0.mpi");
        std::fs::write(&path, package_body(big())).unwrap();
        let info = check_package(&path).unwrap();
        assert_eq!(info.path, path);
        assert_eq!(info.size, MIN_PACKAGE_BYTES);
    }

    #[test]
    fn a_package_that_is_still_downloading_is_too_small() {
        let scratch = Scratch::new("check-short");
        let path = scratch.0.join("TTW_3.4.0.mpi");
        std::fs::write(&path, package_body(16)).unwrap();
        assert_eq!(check_package(&path), Err(PackageProblem::TooSmall));
    }

    #[test]
    fn a_file_that_does_not_start_like_a_package_is_refused() {
        let scratch = Scratch::new("check-head");
        let path = scratch.0.join("TTW_3.4.0.mpi");
        let mut body = package_body(big());
        body[..PACKAGE_HEADER.len()].copy_from_slice(b"notz");
        std::fs::write(&path, body).unwrap();
        assert_eq!(check_package(&path), Err(PackageProblem::NotAPackage));
    }

    #[test]
    fn a_download_in_progress_is_not_a_package_name() {
        let scratch = Scratch::new("check-part");
        for name in [
            "TTW_3.4.0.mpi.part",
            "TTW_3.4.0.mpi.crdownload",
            "TTW_3.4.0.mpi.tmp",
        ] {
            let path = scratch.0.join(name);
            std::fs::write(&path, package_body(big())).unwrap();
            assert_eq!(check_package(&path), Err(PackageProblem::WrongName));
        }
    }

    #[test]
    fn a_package_that_is_not_there_cannot_be_read() {
        let scratch = Scratch::new("check-missing");
        assert_eq!(
            check_package(&scratch.0.join("TTW_missing.mpi")),
            Err(PackageProblem::NotThere)
        );
    }

    #[test]
    fn every_reason_is_one_short_plain_sentence() {
        let problems = [
            PackageProblem::NotThere,
            PackageProblem::WrongName,
            PackageProblem::TooSmall,
            PackageProblem::NotAPackage,
        ];
        let mut whys: Vec<&str> = problems.iter().map(PackageProblem::why).collect();
        for why in &whys {
            assert!(why.ends_with('.'), "{why}");
            assert!(!why.contains('/') && !why.contains('\\'), "{why}");
            assert!(!why.to_lowercase().contains("raul"), "{why}");
        }
        whys.sort_unstable();
        whys.dedup();
        assert_eq!(whys.len(), 4, "each problem says something of its own");
    }
}
