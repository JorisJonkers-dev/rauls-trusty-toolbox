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
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the TTW package is downloaded from: the one place the page links to.
pub const DOWNLOAD_URL: &str = "https://taleoftwowastelands.com/";

/// The space the TTW install needs on the Instance's storage: the output and the files it works with, rounded up.
pub const NEEDED_BYTES: u64 = 40 * 1024 * 1024 * 1024;

/// `raul space --folder <dir>`: the free space where the Instance is; read only.
pub fn space_command(folder: &str) -> Command {
    Command::read("space", 1, &["space", "--folder", folder])
}

/// The free bytes in the answer of `raul space --json`.
///
/// # Errors
/// The answer is not that document.
pub fn free_bytes_of(stdout: &str) -> Result<u64, String> {
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not a raul space answer: {e}"))?;
    v.get("free_bytes")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "the answer has no free space".to_owned())
}

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
    /// The folders Settings points at that a package may be in: the folder of the package set there, and the folder
    /// the Fallout 3 folder is in (a download often lands next to the games). Folders that are not named are left
    /// out, and none comes twice.
    pub fn search_folders(&self) -> Vec<PathBuf> {
        let mut folders: Vec<PathBuf> = Vec::new();
        for path in [self.package.as_deref(), self.fallout3.as_deref()]
            .into_iter()
            .flatten()
        {
            if let Some(parent) = Path::new(path)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                && !folders.iter().any(|f| f == parent)
            {
                folders.push(parent.to_path_buf());
            }
        }
        folders
    }

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
    /// How many files went to `_ttwright_game_root/` in the Mod instead of a place outside it, also those past the
    /// ones listed.
    pub redirected_total: u64,
    /// The files redirected that the installer listed (at most 100).
    pub redirected: Vec<Redirected>,
}

/// A file the installer put under `_ttwright_game_root/` because the package meant it for a place outside the Mod.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirected {
    /// The package's name of the place it was meant for, such as `FNV Root`.
    pub location: String,
    /// The path the package meant.
    pub target: String,
    /// Where it went, relative to the Mod.
    pub written_to: String,
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
            redirected_total: 0,
            redirected: Vec::new(),
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
        /// The size of the file in bytes; 0 when it is not known.
        size: u64,
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

/// The folders the package is looked in for: the Downloads folder of `home`, then its Desktop folder,
/// then the folders set in Settings, without duplicates and in that order. Without a home folder only
/// the folders set in Settings come.
pub fn usual_folders(home: Option<&Path>, extra: &[PathBuf]) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    if let Some(home) = home {
        folders.push(home.join("Downloads"));
        folders.push(home.join("Desktop"));
    }
    for folder in extra {
        if !folders.contains(folder) {
            folders.push(folder.clone());
        }
    }
    folders
}

/// How long the size of a watched file of the Downloads folder must stay the same before it counts as complete.
pub const SETTLE: Duration = Duration::from_secs(3);

/// Tells whether one file is complete from the sizes a caller reads for it; the type reads nothing of the file
/// system itself. A name that ends in `.part`, `.crdownload` or `.tmp` is never complete; any other file is once
/// it is there, is not empty and its size has stayed the same for [`SETTLE`] in total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settling {
    /// The file watched, as it was given.
    path: PathBuf,
    /// Its size on the last poll; `None` before the first and while it is not there.
    size: Option<u64>,
    /// For how long its size has stayed the same; a change or a missing file puts it back to zero.
    stable_for: Duration,
}

impl Settling {
    /// Watches `path` as it is given: the name of a file that is still being written is kept, and never settles.
    pub fn new(path: PathBuf) -> Settling {
        Settling {
            path,
            size: None,
            stable_for: Duration::ZERO,
        }
    }

    /// The file watched.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// One poll: `now_elapsed` is the time passed since the last poll, and `size_now` the file's current size,
    /// `None` when it is not there. True only when the file is there, is not empty and its size is the one of
    /// the last poll and has stayed the same for [`SETTLE`] in total; a change of size or a missing file puts
    /// the stable time back to zero.
    pub fn poll(&mut self, now_elapsed: Duration, size_now: Option<u64>) -> bool {
        if is_partial(&self.path) {
            return false;
        }
        let settled = self.size.is_some_and(|s| Some(s) == size_now && s > 0);
        if settled {
            self.stable_for += now_elapsed;
        } else {
            self.stable_for = Duration::ZERO;
        }
        self.size = size_now;
        settled && self.stable_for >= SETTLE
    }
}

/// Whether `path` is named as a file that is still being written: one that ends in `.part`, `.crdownload` or
/// `.tmp`, in any case. Such a file never settles.
fn is_partial(path: &Path) -> bool {
    path.extension().is_some_and(|ext| {
        ext.eq_ignore_ascii_case("part")
            || ext.eq_ignore_ascii_case("crdownload")
            || ext.eq_ignore_ascii_case("tmp")
    })
}

/// The smallest size a finished package has: a smaller file is a partial download.
pub const MIN_PACKAGE_BYTES: u64 = 1_000_000;

/// A file the search accepted as the TTW package, or as the archive that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    /// The package file.
    pub path: PathBuf,
    /// Its size in bytes.
    pub size: u64,
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
    /// The free bytes where the Instance is, read from `raul space` when Output is shown.
    pub space: Fetch<u64>,
    /// Sources read again while a command ran, applied by [`State::settle`] when it ends.
    pub deferred: Option<Sources>,
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

    /// The free space is being read.
    pub fn space_loading(&mut self) {
        self.space = Fetch::Loading;
    }

    /// The free space arrived.
    pub fn space_read(&mut self, free_bytes: u64) {
        self.space = Fetch::Ready(free_bytes);
    }

    /// The free space could not be read; the command's message.
    pub fn space_failed(&mut self, why: String) {
        self.space = Fetch::Failed(why);
    }

    /// How many bytes are missing when the free space is known and smaller than the install needs.
    pub fn space_missing(&self) -> Option<u64> {
        self.space
            .ready()
            .and_then(|free| NEEDED_BYTES.checked_sub(*free))
            .filter(|missing| *missing > 0)
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
                size: 0,
            },
            None => Found::NotFound { folder },
        };
    }

    /// The search over the usual folders is done: `package` is the newest valid one with its size, or `None`.
    pub fn searched_package(&mut self, folder: &Path, package: Option<&PackageInfo>) {
        self.searched(folder, package.map(|p| p.path.as_path()));
        if let (Found::Found { size, .. }, Some(p)) = (&mut self.found, package) {
            *size = p.size;
        }
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
        if self.busy() {
            self.deferred = Some(sources);
            return;
        }
        self.deferred = None;
        if self.done() || self.sources.ready() == Some(&sources) {
            return;
        }
        *self = State {
            step: Step::Sources,
            ..State::new(self.instance.take())
        };
        self.sources_read(sources);
    }

    /// Applies the sources that were read again while a command ran, once no command runs. A running command is
    /// never restarted or cut off by them.
    pub fn settle(&mut self) {
        if !self.busy()
            && let Some(sources) = self.deferred.take()
        {
            self.refreshed(sources);
        }
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
            Step::Output => self.instance.is_some() && self.space_missing().is_none(),
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
            Step::Output => {
                self.step = Step::Sources;
                self.space = Fetch::Idle;
            }
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

    /// Whether an install ended in success: the page then shows a Mod that the Instance must still hold.
    pub fn finished_install(&self) -> bool {
        self.done()
            && self
                .result
                .as_ref()
                .is_some_and(|r| r.status == Status::Success && !r.dry_run)
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
    fn output_waits_only_for_a_known_shortage_of_space() {
        let mut s = ready();
        s.next();
        assert_eq!(s.step, Step::Output);
        assert!(s.can_next(), "space not read yet: not blocked");
        s.space_loading();
        assert!(s.can_next());
        s.space_failed("unreadable".into());
        assert!(s.can_next(), "an unreadable folder does not block");
        s.space_read(NEEDED_BYTES - 1);
        assert_eq!(s.space_missing(), Some(1));
        assert!(!s.can_next());
        s.space_read(NEEDED_BYTES);
        assert_eq!(s.space_missing(), None);
        assert!(s.can_next());
    }

    #[test]
    fn the_space_answer_is_read_from_its_document() {
        assert_eq!(
            free_bytes_of(r#"{"folder":"/i","free_bytes":5,"total_bytes":9}"#),
            Ok(5)
        );
        assert!(free_bytes_of("nope").is_err());
        assert!(free_bytes_of("{}").is_err());
        assert_eq!(space_command("/i").shown(), "raul space --folder /i");
    }

    #[test]
    fn a_re_read_that_finishes_during_a_job_is_applied_when_the_job_ends() {
        let mut s = ready();
        s.next();
        s.step = Step::Review;
        s.checking().unwrap();
        let changed = Sources {
            package: Some("/new.mpi".into()),
            ..s.sources.ready().cloned().unwrap()
        };
        s.refreshed(changed);
        s.settle();
        assert!(s.busy(), "the running job is not cancelled");
        s.checked(success(true), problem()).unwrap();
        assert!(s.deferred.is_some());
        s.settle();
        assert_eq!(s.wizard.package(), Some("/new.mpi"));
        assert!(s.deferred.is_none() && s.step == Step::Sources);
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
        s.settle();
        assert_eq!(
            s.step,
            Step::Sources,
            "the answer that came during the check is applied when it ends"
        );
        let mut s = ready();
        s.next();
        s.step = Step::Review;
        s.checking().unwrap();
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

    #[test]
    fn the_usual_folders_are_downloads_then_desktop_then_the_settings_ones() {
        let home = Path::new("/home/person");
        assert_eq!(
            usual_folders(Some(home), &[]),
            vec![home.join("Downloads"), home.join("Desktop")]
        );
        let extra = vec![
            PathBuf::from("/media/usb"),
            home.join("Downloads"),
            PathBuf::from("/media/usb"),
        ];
        assert_eq!(
            usual_folders(Some(home), &extra),
            vec![
                home.join("Downloads"),
                home.join("Desktop"),
                PathBuf::from("/media/usb"),
            ],
            "the folders of Settings come last, each one once"
        );
        assert_eq!(
            usual_folders(None, &extra),
            vec![PathBuf::from("/media/usb"), home.join("Downloads")],
            "without a home folder only the folders of Settings come"
        );
        assert_eq!(usual_folders(None, &[]), Vec::<PathBuf>::new());
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
                folder: "/d".into(),
                size: 0
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

    #[test]
    fn settings_point_at_the_folders_of_the_package_and_of_the_game() {
        let s = Sources {
            fallout3: Some("/games/Fallout 3".into()),
            newvegas: None,
            package: Some("/games/TTW_3.4.mpi".into()),
        };
        assert_eq!(s.search_folders(), [PathBuf::from("/games")]);
        let s = Sources {
            fallout3: Some("/a/FO3".into()),
            newvegas: None,
            package: Some("/b/TTW.mpi".into()),
        };
        assert_eq!(
            s.search_folders(),
            [PathBuf::from("/b"), PathBuf::from("/a")]
        );
        assert!(Sources::default().search_folders().is_empty());
    }

    #[test]
    fn a_download_settles_only_when_its_size_stays_the_same() {
        let mut s = Settling::new(PathBuf::from("/d/TTW_3.4.0.mpi"));
        let sec = Duration::from_secs(1);
        assert!(!s.poll(sec, Some(0)), "an empty file is nothing yet");
        assert!(
            !s.poll(sec, Some(64)),
            "nothing to compare with on the first poll"
        );
        assert!(!s.poll(sec, Some(192)), "the download grows");
        assert!(
            !s.poll(sec, Some(192)),
            "its size stayed for less than SETTLE"
        );
        assert!(
            !s.poll(sec, Some(192)),
            "its size stayed for less than SETTLE"
        );
        assert!(
            s.poll(sec, Some(192)),
            "its size stayed for SETTLE: complete"
        );
        assert!(s.poll(sec, Some(192)), "a complete file stays complete");
        assert!(!s.poll(sec, Some(320)), "it grows again");
        assert!(
            !s.poll(sec, Some(320)),
            "the wait starts over after a change"
        );
        assert!(!s.poll(sec, Some(320)), "less than SETTLE has passed since");
        assert!(
            s.poll(sec, Some(320)),
            "after a new wait of SETTLE it is complete again"
        );
    }

    #[test]
    fn a_missing_file_starts_the_wait_over() {
        let mut s = Settling::new(PathBuf::from("/d/TTW_3.4.0.mpi"));
        let sec = Duration::from_secs(1);
        s.poll(sec, Some(64));
        s.poll(sec, Some(64));
        s.poll(sec, Some(64));
        assert!(
            !s.poll(sec, None),
            "a file that is not there is not complete"
        );
        assert_eq!(s.size, None, "its last size is forgotten");
        assert!(
            !s.poll(sec, Some(64)),
            "nothing to compare with after it came back"
        );
        assert!(
            !s.poll(SETTLE.saturating_sub(sec), Some(64)),
            "less than SETTLE passed since it came back"
        );
        assert!(s.poll(sec, Some(64)), "the whole wait ran again");
    }

    #[test]
    fn an_empty_file_is_never_complete() {
        let mut s = Settling::new(PathBuf::from("/d/empty.mpi"));
        assert!(!s.poll(Duration::from_secs(1), Some(0)));
        assert!(
            !s.poll(SETTLE, Some(0)),
            "a file with nothing in it stays nothing"
        );
        assert_eq!(s.stable_for, Duration::ZERO, "an empty file never waits");
    }

    #[test]
    fn a_name_that_is_still_being_written_is_never_complete() {
        for name in [
            "TTW_3.4.0.mpi.part",
            "ttw_3.4.0.mpi.crdownload",
            "ttw_3.4.0.mpi.tmp",
            "TTW_3.4.0.mpi.PART",
        ] {
            let mut s = Settling::new(PathBuf::from(name));
            assert_eq!(s.path, PathBuf::from(name), "the name is kept as it is");
            s.poll(Duration::from_secs(1), Some(64));
            assert!(
                !s.poll(Duration::from_secs(10 * SETTLE.as_secs()), Some(64)),
                "{name} is never complete"
            );
        }
    }
}
