// SPDX-License-Identifier: GPL-3.0-only
//! The TTW flow (UI-RULES P4, T5, K3, G6): Sources, Output, Review and Install, on the shell like the setup wizard.
//!
//! Every step is a `raul` command from the registry, shown on the page. The sources come from `raul config list`
//! and are changed in Settings, their home. Review runs `raul ttw --dry-run`: the package is read and both game
//! copies are checked, and nothing is written. Install runs `raul ttw` as a job: its progress shows with a
//! percentage, it keeps going when the page is left, and Cancel stops it. A failed or cancelled run leaves the
//! Instance as it was; Resume continues a stopped one with `--resume`.

use super::home::{Mark, icon, mark};
use super::settings::{Pending, Visits, error_block, read_config, start, waiting};
use super::{Fixture, Page, PageCx, ShellState};
use crate::catalogue::{self as msg, ErrorText};
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText};
use raul_ui_model::events::Status;
use raul_ui_model::job::{JobId, JobState, Outcome};
use raul_ui_model::pages::load_order::Problem;
use raul_ui_model::pages::settings::{Answer, Command, Fetch, Key};
use raul_ui_model::pages::ttw::{
    DOWNLOAD_URL, Found, PackageInfo, Progress, Report, Settling, Sources, State, Step,
    check_command, check_package, default_downloads, find_in, install_command, newest_candidate,
};
use raul_ui_model::pages::{Flow, PageId, View};
use raul_ui_model::wizard::{Step as FlowStep, TtwNeed};
use raul_ui_service::Output;
use raul_ui_service::dto::TtwReport;
use std::path::{Path, PathBuf};

crate::texts!(TEXTS {
    STATE_NO_INSTANCE = "No Instance yet";
    STATE_READING = "Reading the sources";
    STATE_STEP = "Step {n} of 4 · {name}";
    STATE_CHECKING = "Checking the sources";
    STATE_INSTALLING = "Installing";
    STATE_INSTALLING_AT = "Installing · {p}%";
    STATE_CANCELLING = "Cancelling";
    STATE_INSTALLED = "Installed";
    STATE_NOT_INSTALLED = "Not installed";
    STEP_SOURCES = "Sources";
    STEP_OUTPUT = "Output";
    STEP_REVIEW = "Review";
    STEP_INSTALL = "Install";
    NO_INSTANCE = "TTW goes into the Instance, which lives in the storage folder. Set the storage folder in Settings first.";
    OPEN_SETTINGS = "Settings";
    READING = "Reading the sources from the configuration";
    INTRO = "TTW is built from your own Fallout 3 and New Vegas copies. Both stay untouched.";
    SECTION_SOURCES = "Sources";
    LABEL_FALLOUT3 = "Fallout 3 folder";
    LABEL_NEW_VEGAS = "New Vegas folder";
    LABEL_PACKAGE = "TTW package";
    SET = "Set";
    NOT_SET = "Not set";
    FROM_INSTANCE = "From the Instance";
    NEW_VEGAS_DEFAULT = "The Instance's own copy";
    CHANGE = "Change in Settings";
    MISSING_BOTH = "To continue, set the Fallout 3 folder in Settings, and add the TTW package below.";
    MISSING_FALLOUT3 = "To continue, set the Fallout 3 folder in Settings.";
    MISSING_PACKAGE = "To continue, add the TTW package below.";
    SECTION_PACKAGE = "The TTW package";
    PACKAGE_WHAT = "Download the Tale of Two Wastelands installer package from the TTW website. Its file name starts with TTW_ and ends in .mpi. If what you downloaded ends in .7z, unpack it first.";
    OPEN_DOWNLOAD = "Open the download page";
    FIND_IT = "Find it for me";
    WATCH = "Watch my Downloads folder and use the package when it arrives";
    FINDING = "Looking for the TTW package in your Downloads folder";
    FOUND = "Found the newest TTW package in your Downloads folder:";
    USE_THIS = "Use this file";
    NOT_FOUND = "No TTW package in your Downloads folder or in the folders inside it. Download it, unpack it if it came in an archive, then find it again.";
    LOOKED_IN = "Looked in";
    PACKAGE_SAVED = "TTW package saved.";
    SECTION_OUTPUT = "Output";
    FACT_INTO = "Installs into";
    FACT_AS = "Added as";
    ADDED_AS = "One Mod, with its plugins enabled in your load order";
    FACT_GAMES = "Game folders";
    GAMES_UNTOUCHED = "Not changed";
    FACT_SPACE = "Space";
    SPACE_NOTE = "Needs tens of GB free in the storage folder";
    CHECKING = "Checking: the package is read and both game copies are checked; nothing is written";
    SECTION_CHECKS = "Checks";
    CHECK_FALLOUT3 = "Fallout 3 checked";
    CHECK_NEW_VEGAS = "New Vegas checked";
    CHECK_PACKAGE = "Package read";
    SECTION_PLAN = "What happens";
    FACT_OUTPUT = "Output";
    FACT_TIME = "Time";
    TIME = "Long: often most of an hour";
    FACT_STOPS = "If it stops";
    STOPS = "The Instance stays as it was, and Resume continues the run";
    FACT_TOOL = "Installer";
    FACT_TOOL_CHECKED = "Checked and ready";
    RESUMING = "Install continues the stopped run where it left off.";
    CHECK_AGAIN = "Check again";
    BACK = "Back";
    NEXT = "Next";
    INSTALL = "Install";
    RESUME = "Resume";
    TRY_AGAIN = "Try again";
    CANCEL = "Cancel";
    DONE = "Done";
    WAITS_FOR_CHECK = "Install waits for the checks.";
    PHASE_STARTING = "Starting";
    PHASE_WORKING = "Working";
    PHASE_PACKAGE = "Reading the package";
    PHASE_ASSETS = "Building the files";
    PHASE_ARCHIVES = "Packing the archives";
    PHASE_POST = "Finishing the files";
    PHASE_ADD_MOD = "Adding the Mod to the Instance";
    PROGRESS_OF = "{done} of {total}";
    LEAVE = "You can leave this page; the install keeps going.";
    INSTALLED = "TTW is installed as one Mod, and its plugins are enabled in your load order.";
    FACT_PLUGINS = "Plugins enabled";
    FACT_FILES = "Files";
    INSTALLED_STATUS = "TTW installed.";
    UNREADABLE = "The run finished, but what it printed could not be read.";
});

crate::errors!(ERRORS {
    SOURCES_UNREADABLE = (
        "The sources could not be read",
        "Nothing changed. The line below says why."
    );
    CHECK_FAILED = (
        "The checks did not pass",
        "Nothing changed. Fix what the line below names, then check again."
    );
    INSTALL_FAILED = (
        "TTW was not installed",
        "The Instance is as it was. The line below says why."
    );
    INSTALL_CANCELLED = (
        "The install was cancelled",
        "The Instance is as it was. Resume continues where it stopped."
    );
    INSTALL_UNKNOWN = (
        "The install ended without a result",
        "The state of the Instance is not known. Run the Doctor before you try again."
    );
    CANCEL_REFUSED = (
        "Cancel was not taken",
        "The install keeps going. Try again in a moment."
    );
    PACKAGE_NOT_SAVED = (
        "The TTW package was not saved",
        "Nothing changed. The line below says why."
    );
});

/// A watch of the Downloads folder: a thread that polls it once a second and sends the first package that is
/// complete (its size stayed the same for [`SETTLE`]); it ends when `stop` is set or the page drops it.
#[derive(Debug)]
struct Watch {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    found: std::sync::mpsc::Receiver<PackageInfo>,
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// How often the watch looks at the folder.
const WATCH_EVERY: std::time::Duration = if cfg!(test) {
    std::time::Duration::from_millis(10)
} else {
    std::time::Duration::from_secs(1)
};

/// The TTW flow.
#[derive(Debug, Default)]
pub struct Ttw {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
    /// The install's job, for its progress and Cancel.
    install: Option<JobId>,
    /// Where the search for the package looks; the shell's, so a test can point it at a fake Downloads folder.
    downloads: Option<PathBuf>,
    /// The search that runs, with the folder it looks in and what it found.
    search: Option<std::sync::mpsc::Receiver<(PathBuf, Option<PackageInfo>)>>,
    /// The watch of the Downloads folder the user agreed to, if one runs.
    watch: Option<Watch>,
    /// Why the package found was not saved.
    save_failed: Option<String>,
    /// The package the flow had before the one found was chosen: the wizard's and the sources', put back when the
    /// save is refused or fails.
    unsaved_from: Option<(Option<String>, Option<String>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Sources,
    /// The sources read again when the page is shown again; the flow stays unless they changed.
    Refresh,
    Check,
    Install,
    /// `raul config set ttw.package`, as Settings saves it.
    Save,
}

impl Ttw {
    /// The flow in `state`, as a fixture shows it; it reads nothing by itself until its Instance changes.
    pub fn with(state: State) -> Ttw {
        let mut page = Ttw {
            state,
            ..Ttw::default()
        };
        page.visits.loaded = true;
        page
    }

    /// Looks for the package in `folder` rather than in the Downloads folder of the home folder.
    #[must_use]
    pub fn looking_in(mut self, folder: impl Into<PathBuf>) -> Ttw {
        self.downloads = Some(folder.into());
        self
    }

    /// The folder the search looks in.
    fn downloads(&self) -> PathBuf {
        self.downloads
            .clone()
            .unwrap_or_else(|| PathBuf::from(default_downloads()))
    }

    fn command(&self, job: Job) -> Option<Command> {
        let i = self.state.instance.as_deref()?;
        Some(match job {
            Job::Sources | Job::Refresh => Command::config_list(),
            Job::Check => check_command(i),
            Job::Install => install_command(i, self.state.resuming),
            Job::Save => {
                let file = self.state.wizard.package()?;
                Command::config_save(Key::TtwPackage, file)
            }
        })
    }

    /// Starts the command of `job`, after the state moved to the step it runs.
    fn run(&mut self, cx: &mut PageCx<'_>, job: Job) {
        let Some(command) = self.command(job) else {
            return;
        };
        let moved = match job {
            Job::Sources => {
                self.state.loading();
                Ok(())
            }
            Job::Refresh => Ok(()),
            Job::Check => self.state.checking(),
            Job::Install => self.state.installing(),
            Job::Save => {
                self.save_failed = None;
                Ok(())
            }
        };
        if moved.is_err() {
            return;
        }
        let problem = |e: ErrorText| Problem {
            id: e.id,
            cause: e.what.to_owned(),
            command: Some(command.shown()),
        };
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(job, id);
                if job == Job::Install {
                    self.install = Some(id);
                }
            }
            Err(e) => {
                let _ = match job {
                    Job::Sources => {
                        self.state.sources_failed(e.what.to_owned());
                        Ok(())
                    }
                    // The flow keeps the sources it has.
                    Job::Refresh => Ok(()),
                    Job::Check => self.state.check_failed(problem(e)),
                    Job::Install => self.state.install_failed(problem(e)),
                    Job::Save => {
                        self.save_failed = Some(e.what.to_owned());
                        Ok(())
                    }
                };
            }
        }
    }

    /// Looks for the package in the Downloads folder, off the UI thread: a listing of two levels is quick, but a
    /// network share is not.
    fn find(&mut self, cx: &mut PageCx<'_>) {
        if self.state.found == Found::Searching {
            return;
        }
        self.state.searching();
        let folder = self.downloads();
        // Downloads first (the one the page was given), then the Desktop of the home folder.
        let mut folders = vec![folder.clone()];
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
        if let Some(home) = home {
            folders.push(PathBuf::from(home).join("Desktop"));
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let repaint = cx.repaint.clone();
        std::thread::spawn(move || {
            let found = find_in(&folders);
            let _ = tx.send((folder, found));
            if let Some(repaint) = repaint {
                repaint.request_repaint();
            }
        });
        self.search = Some(rx);
    }

    /// Starts watching the folders the search looks in. Nothing starts until the user asked for it.
    fn start_watch(&mut self, cx: &PageCx<'_>) {
        let mut folders = vec![self.downloads()];
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
        if let Some(home) = home {
            folders.push(PathBuf::from(home).join("Desktop"));
        }
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel();
        let (flag, repaint) = (stop.clone(), cx.repaint.clone());
        std::thread::spawn(move || {
            let mut settling: Option<Settling> = None;
            while !flag.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(WATCH_EVERY);
                // The newest candidate by name and shape, however small it is still: a file that is growing is
                // settled first, and only then checked as a whole.
                let Some(path) = newest_candidate(&folders) else {
                    settling = None;
                    continue;
                };
                let size = std::fs::metadata(&path).ok().map(|m| m.len());
                let watched = settling.get_or_insert_with(|| Settling::new(path.clone()));
                if watched.path() != path {
                    *watched = Settling::new(path.clone());
                }
                if watched.poll(WATCH_EVERY, size)
                    && let Ok(info) = check_package(&path)
                {
                    let _ = tx.send(info);
                    if let Some(repaint) = repaint {
                        repaint.request_repaint();
                    }
                    return;
                }
            }
        });
        self.watch = Some(Watch { stop, found: rx });
    }

    /// Takes in a package the watch found: it is offered as the one found and saved at once, as if the user had
    /// pressed "Use this file".
    fn take_watch(&mut self, cx: &mut PageCx<'_>) {
        let Some(watch) = &self.watch else {
            return;
        };
        if let Ok(info) = watch.found.try_recv() {
            self.watch = None;
            let folder = info.path.parent().unwrap_or(Path::new("")).to_path_buf();
            self.state.searched_package(&folder, Some(&info));
            self.use_found(cx);
        }
    }

    /// Takes in the answer of the search, once it is there.
    fn take_search(&mut self) {
        let Some(rx) = &self.search else {
            return;
        };
        if let Ok((folder, found)) = rx.try_recv() {
            self.state.searched_package(&folder, found.as_ref());
            self.search = None;
        }
    }

    /// Saves the package found as the TTW package, with the same `raul config set` Settings runs for that key. The
    /// flow uses the file only once it is saved; when it is not, the state says why and the file stays as it was.
    fn use_found(&mut self, cx: &mut PageCx<'_>) {
        let Found::Found { file, .. } = &self.state.found else {
            return;
        };
        let file = file.clone();
        self.save_failed = None;
        let command = Command::config_save(Key::TtwPackage, &file);
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(Job::Save, id);
                self.unsaved_from = Some((
                    self.state.wizard.package().map(str::to_owned),
                    self.state.sources.ready().and_then(|s| s.package.clone()),
                ));
                self.state.use_package(&file);
            }
            Err(e) => self.save_failed = Some(e.what.to_owned()),
        }
    }

    /// Puts back the package the flow had before the one found was chosen, when its save did not go through.
    fn restore_unsaved(&mut self) {
        if let Some((wizard, sources)) = self.unsaved_from.take() {
            self.state.restore_package(wizard, sources);
        }
    }

    /// Follows the Instance the shell knows: a new one starts the flow over. A command that runs for the old one
    /// ends first.
    fn follow_instance(&mut self, cx: &mut PageCx<'_>) {
        if cx.shell.instance == self.state.instance || self.state.busy() {
            return;
        }
        self.state = State::new(cx.shell.instance.clone());
        self.install = None;
        self.search = None;
        self.watch = None;
        self.save_failed = None;
        self.unsaved_from = None;
        if self.state.instance.is_some() {
            self.run(cx, Job::Sources);
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            let command = self.command(job).map(|c| c.shown());
            let problem = |e: &ErrorText, cause: String| Problem {
                id: e.id,
                cause,
                command: command.clone(),
            };
            let _ = match job {
                Job::Sources => {
                    match read_config(&result) {
                        Ok(c) => self.state.sources_read(Sources::of(&c)),
                        Err(why) => self.state.sources_failed(why),
                    }
                    Ok(())
                }
                Job::Refresh => {
                    if let Ok(c) = read_config(&result) {
                        self.state.refreshed(Sources::of(&c));
                    }
                    Ok(())
                }
                Job::Check => match report_of(&result) {
                    Ok(r) => {
                        let cause = r.message.clone().unwrap_or_default();
                        self.state.checked(r, problem(&CHECK_FAILED, cause))
                    }
                    Err(why) => self.state.check_failed(problem(&CHECK_FAILED, why)),
                },
                Job::Install => {
                    self.install = None;
                    match report_of(&result) {
                        Ok(r) => {
                            let e = match r.status {
                                Status::Cancelled => &INSTALL_CANCELLED,
                                _ => &INSTALL_FAILED,
                            };
                            let cause = r.message.clone().unwrap_or_default();
                            if r.status == Status::Success {
                                cx.shell.status = Some(INSTALLED_STATUS.to_owned());
                            }
                            self.state.installed(r, problem(e, cause))
                        }
                        Err(why) => self.state.install_failed(problem(&INSTALL_UNKNOWN, why)),
                    }
                }
                Job::Save => match super::settings::config_answer(&result) {
                    Ok(Answer::Refused(r)) => {
                        self.save_failed = Some(r.message.clone());
                        self.restore_unsaved();
                        Ok(())
                    }
                    Ok(Answer::Entries(_)) => {
                        self.unsaved_from = None;
                        cx.shell.status = Some(PACKAGE_SAVED.to_owned());
                        Ok(())
                    }
                    Err(why) => {
                        self.save_failed = Some(why);
                        self.restore_unsaved();
                        Ok(())
                    }
                },
            };
        }
    }

    /// Takes the install job's progress from the job table.
    fn follow_install(&mut self, cx: &PageCx<'_>) {
        let Some(view) = self.install.and_then(|id| cx.jobs.table().get(id)) else {
            return;
        };
        if matches!(view.state, JobState::Running | JobState::Cancelling) {
            self.state.progressed(Progress {
                phase: view.phase.clone(),
                done: view.done,
                total: view.total,
            });
        }
    }

    fn cancel(&mut self, cx: &mut PageCx<'_>) {
        let Some(id) = self.install else {
            return;
        };
        if cx.jobs.cancel(id).is_ok() {
            let _ = self.state.cancelling();
        } else {
            self.state.problem = Some(Problem {
                id: CANCEL_REFUSED.id,
                cause: String::new(),
                command: None,
            });
        }
    }

    fn problem_block(&self, ui: &mut egui::Ui, t: &Tokens) {
        let Some(p) = &self.state.problem else {
            return;
        };
        let e = ERRORS
            .iter()
            .chain(msg::SHELL_ERRORS)
            .chain(super::settings::ERRORS)
            .find(|e| e.id == p.id)
            .copied()
            .unwrap_or(INSTALL_FAILED);
        let said = (!p.cause.is_empty()).then_some(p.cause.as_str());
        error_block(ui, t, &e, said);
        ui.add_space(12.0);
    }

    fn sources(&mut self, ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>, s: &Sources) {
        ui.label(RichText::new(INTRO).color(t.muted.color()));
        ui.add_space(12.0);
        shell::section_label(ui, t, SECTION_SOURCES, true);
        let width = ui.available_width();
        let rows = [
            (LABEL_FALLOUT3, s.fallout3.as_deref(), false),
            (LABEL_NEW_VEGAS, s.newvegas.as_deref(), true),
            (LABEL_PACKAGE, s.package.as_deref(), false),
        ];
        if width < 480.0 {
            // Too narrow for three columns: each source is its own wrapped line (UI-RULES A6).
            for (label, value, defaults) in rows {
                source_line(ui, t, label, value, defaults);
            }
        } else {
            egui::Grid::new("ttw-sources")
                .num_columns(3)
                .spacing([18.0, 10.0])
                .min_row_height(28.0)
                .show(ui, |ui| {
                    for (label, value, defaults) in rows {
                        source_row(ui, t, width, label, value, defaults);
                    }
                });
        }
        ui.add_space(12.0);
        if ui.button(CHANGE).clicked() {
            cx.shell.go_to = Some(View::Page(PageId::Settings));
        }
        ui.add_space(16.0);
        self.package(ui, t, cx);
    }

    /// Where the package comes from (the download page) and how to find it on this machine once it is there.
    fn package(&mut self, ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>) {
        shell::section_label(ui, t, SECTION_PACKAGE, true);
        ui.scope(|ui| {
            ui.set_max_width(ui.available_width().min(560.0));
            ui.add(egui::Label::new(RichText::new(PACKAGE_WHAT).color(t.muted.color())).wrap());
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button(OPEN_DOWNLOAD).clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(DOWNLOAD_URL));
            }
            let finding = self.state.found == Found::Searching;
            if ui
                .add_enabled(!finding, egui::Button::new(FIND_IT))
                .clicked()
            {
                self.find(cx);
            }
        });
        // Off until the user agrees; it stops when the box is cleared, a package arrives or the flow starts over.
        let mut watching = self.watch.is_some();
        if ui.checkbox(&mut watching, WATCH).changed() {
            if watching {
                self.start_watch(cx);
            } else {
                self.watch = None;
            }
        }
        match self.state.found.clone() {
            Found::Idle => {}
            Found::Searching => {
                ui.add_space(8.0);
                note(ui, t, Mark::Info, FINDING, None);
            }
            Found::Found { file, size, .. } => {
                ui.add_space(8.0);
                let detail = if size > 0 {
                    format!("{file} ({})", size_text(size))
                } else {
                    file
                };
                note(ui, t, Mark::Ok, FOUND, Some(&detail));
                ui.add_space(4.0);
                if ui.button(USE_THIS).clicked() {
                    self.use_found(cx);
                }
            }
            Found::NotFound { folder } => {
                ui.add_space(8.0);
                ui.scope(|ui| {
                    ui.set_max_width(ui.available_width().min(560.0));
                    ui.add(egui::Label::new(RichText::new(NOT_FOUND).color(t.text.color())).wrap());
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new(LOOKED_IN).color(t.muted.color()));
                    ui.add(
                        egui::Label::new(RichText::new(&folder).monospace().color(t.muted.color()))
                            .truncate(),
                    )
                    .on_hover_text(&folder);
                });
            }
        }
        if let Some(why) = &self.save_failed {
            error_block(ui, t, &PACKAGE_NOT_SAVED, Some(why));
        }
        ui.add_space(12.0);
    }

    fn output(ui: &mut egui::Ui, t: &Tokens, instance: &str) {
        shell::section_label(ui, t, SECTION_OUTPUT, true);
        facts(
            ui,
            t,
            "ttw-output",
            &[
                (FACT_INTO, instance, true),
                (FACT_AS, ADDED_AS, false),
                (FACT_GAMES, GAMES_UNTOUCHED, false),
                (FACT_SPACE, SPACE_NOTE, false),
            ],
        );
    }

    fn review(&self, ui: &mut egui::Ui, t: &Tokens, instance: &str) {
        if self.state.wizard.flow.step() == FlowStep::Planning {
            waiting(ui, t, CHECKING, &check_command(instance));
            return;
        }
        if self.state.problem.is_some() {
            return;
        }
        let Some(check) = &self.state.check else {
            return;
        };
        shell::section_label(ui, t, SECTION_CHECKS, true);
        let sources = self.state.sources.ready().cloned().unwrap_or_default();
        for (line, detail) in [
            (CHECK_FALLOUT3, sources.fallout3.as_deref()),
            (
                CHECK_NEW_VEGAS,
                Some(sources.newvegas.as_deref().unwrap_or(NEW_VEGAS_DEFAULT)),
            ),
            (CHECK_PACKAGE, sources.package.as_deref()),
        ] {
            note(ui, t, Mark::Ok, line, detail);
        }
        ui.add_space(12.0);
        shell::section_label(ui, t, SECTION_PLAN, true);
        let mut rows = vec![
            (FACT_OUTPUT, instance, true),
            (FACT_TIME, TIME, false),
            (FACT_STOPS, STOPS, false),
        ];
        if check.tool.is_some() {
            rows.push((FACT_TOOL, FACT_TOOL_CHECKED, true));
        }
        facts(ui, t, "ttw-plan", &rows);
        ui.add_space(8.0);
        if self.state.resuming {
            ui.label(RichText::new(RESUMING).color(t.muted.color()));
        }
    }

    fn install(&self, ui: &mut egui::Ui, t: &Tokens, instance: &str) {
        if !self.state.done() {
            let p = self.state.progress.clone().unwrap_or_default();
            let bar = egui::ProgressBar::new(p.fraction().unwrap_or(0.0))
                .desired_width(ui.available_width().min(560.0))
                .fill(t.info.color());
            // Without an estimate the bar stays empty and the phase says what runs; it does not animate, so the
            // window only repaints when the job reports.
            let bar = if p.fraction().is_some() {
                bar.show_percentage()
            } else {
                bar
            };
            ui.add(bar);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(phase_words(&p.phase)).color(t.text.color()));
                if let Some(total) = p.total {
                    ui.label(
                        RichText::new(
                            PROGRESS_OF
                                .replace("{done}", &p.done.to_string())
                                .replace("{total}", &total.to_string()),
                        )
                        .color(t.muted.color()),
                    );
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new(LEAVE).color(t.muted.color()));
            return;
        }
        if self.state.outcome() != Some(Outcome::Completed) {
            return;
        }
        note(ui, t, Mark::Ok, INSTALLED, None);
        ui.add_space(12.0);
        let Some(r) = &self.state.result else {
            return;
        };
        let files = r.files.map_or_else(String::new, |f| f.to_string());
        let mut rows = vec![(FACT_OUTPUT, instance, true)];
        if r.files.is_some() {
            rows.push((FACT_FILES, files.as_str(), false));
        }
        facts(ui, t, "ttw-installed", &rows);
        if !r.plugins.is_empty() {
            ui.add_space(12.0);
            shell::section_label(ui, t, FACT_PLUGINS, true);
            let width = ui.available_width();
            for p in &r.plugins {
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(width);
                        ui.add(
                            egui::Label::new(RichText::new(p).monospace().color(t.text.color()))
                                .truncate(),
                        )
                        .on_hover_text(p);
                    },
                );
            }
        }
    }

    /// The footer of the flow: the line that says what Next waits for, then Back and the step's one primary
    /// button (UI-RULES A1, G3).
    fn footer(&mut self, ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>) {
        ui.add_space(16.0);
        shell::hairline(ui, t);
        ui.add_space(10.0);
        let why = self.waits_for();
        // What Next waits for gets a line of its own, so the buttons below keep their room (UI-RULES A6).
        if let Some(why) = why {
            ui.add(egui::Label::new(RichText::new(why).color(t.muted.color())).wrap());
        }
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // The layout runs right to left: the primary button comes first.
                match self.state.step {
                    Step::Install if !self.state.done() => {
                        let cancelling = self.state.wizard.flow.step() == FlowStep::Cancelling;
                        if ui
                            .add_enabled(!cancelling, egui::Button::new(CANCEL))
                            .clicked()
                        {
                            self.cancel(cx);
                        }
                    }
                    Step::Install => {
                        if self.state.resumable() {
                            if ui.add(shell::primary_button(t, RESUME)).clicked() {
                                self.state.again(true);
                            }
                        } else if self.state.outcome() == Some(Outcome::Completed) {
                            if ui.add(shell::primary_button(t, DONE)).clicked() {
                                cx.shell.go_to = Some(View::Page(PageId::Home));
                            }
                        } else if ui.add(shell::primary_button(t, TRY_AGAIN)).clicked() {
                            self.state.again(false);
                        }
                    }
                    step => {
                        let label = match step {
                            Step::Review if self.state.resuming => RESUME,
                            Step::Review => INSTALL,
                            _ => NEXT,
                        };
                        let next =
                            ui.add_enabled(self.state.can_next(), shell::primary_button(t, label));
                        if next.clicked() {
                            if step == Step::Review {
                                self.run(cx, Job::Install);
                            } else {
                                self.state.next();
                            }
                        }
                        if step == Step::Review
                            && self.state.problem.is_some()
                            && ui.button(CHECK_AGAIN).clicked()
                        {
                            self.run(cx, Job::Check);
                        }
                        if step != Step::Sources
                            && ui
                                .add_enabled(!self.state.busy(), egui::Button::new(BACK))
                                .clicked()
                        {
                            self.state.back();
                        }
                    }
                }
            });
        });
    }

    /// What a disabled Next waits for, in one line (UI-RULES G3).
    fn waits_for(&self) -> Option<&'static str> {
        match self.state.step {
            Step::Sources => {
                let missing = self.state.missing();
                match (
                    missing.contains(&TtwNeed::Fallout3),
                    missing.contains(&TtwNeed::Package),
                ) {
                    (true, true) => Some(MISSING_BOTH),
                    (true, false) => Some(MISSING_FALLOUT3),
                    (false, true) => Some(MISSING_PACKAGE),
                    (false, false) => None,
                }
            }
            Step::Review if !self.state.can_next() && self.state.problem.is_none() => {
                Some(WAITS_FOR_CHECK)
            }
            _ => None,
        }
    }
}

impl Page for Ttw {
    fn view(&self) -> View {
        View::Flow(Flow::Ttw)
    }

    fn title(&self) -> &'static str {
        msg::FLOW_TTW
    }

    fn state_line(&self) -> String {
        let s = &self.state;
        if s.instance.is_none() {
            return STATE_NO_INSTANCE.to_owned();
        }
        if s.sources.is_pending() {
            return STATE_READING.to_owned();
        }
        match s.wizard.flow.step() {
            FlowStep::Planning => return STATE_CHECKING.to_owned(),
            FlowStep::Cancelling => return STATE_CANCELLING.to_owned(),
            FlowStep::Applying => {
                return match s.progress.as_ref().and_then(Progress::fraction) {
                    Some(f) => STATE_INSTALLING_AT.replace("{p}", &percent(f).to_string()),
                    None => STATE_INSTALLING.to_owned(),
                };
            }
            FlowStep::Done if s.outcome() == Some(Outcome::Completed) => {
                return STATE_INSTALLED.to_owned();
            }
            FlowStep::Done => return STATE_NOT_INSTALLED.to_owned(),
            FlowStep::Input | FlowStep::Review => {}
        }
        STATE_STEP
            .replace("{n}", &s.step.number().to_string())
            .replace("{name}", step_name(s.step))
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let back = self.visits.returned(ui.ctx());
        self.take_search();
        self.take_watch(cx);
        self.follow_instance(cx);
        if !self.visits.loaded {
            self.visits.loaded = true;
            if self.state.sources == Fetch::Idle {
                self.run(cx, Job::Sources);
            }
        } else if back && self.state.sources.ready().is_some() && !self.state.busy() {
            // Shown again: the sources may have changed in Settings; the flow starts over only if they did.
            self.run(cx, Job::Refresh);
        }
        self.take_answers(cx);
        self.follow_install(cx);
        if self.state.needs_check() {
            self.run(cx, Job::Check);
        }
        let Some(instance) = self.state.instance.clone() else {
            ui.label(NO_INSTANCE);
            ui.add_space(8.0);
            if ui.button(OPEN_SETTINGS).clicked() {
                cx.shell.go_to = Some(View::Page(PageId::Settings));
            }
            return;
        };
        let sources = match &self.state.sources {
            Fetch::Idle | Fetch::Loading => {
                waiting(ui, &t, READING, &Command::config_list());
                return;
            }
            Fetch::Failed(why) => {
                error_block(ui, &t, &SOURCES_UNREADABLE, Some(why));
                ui.add_space(8.0);
                if ui.button(CHECK_AGAIN).clicked() {
                    self.run(cx, Job::Sources);
                }
                return;
            }
            Fetch::Ready(s) => s.clone(),
        };
        let installed = self.state.outcome() == Some(Outcome::Completed);
        steps(ui, &t, self.state.step, installed);
        ui.add_space(14.0);
        self.problem_block(ui, &t);
        match self.state.step {
            Step::Sources => self.sources(ui, &t, cx, &sources),
            Step::Output => Ttw::output(ui, &t, &instance),
            Step::Review => self.review(ui, &t, &instance),
            Step::Install => self.install(ui, &t, &instance),
        }
        self.footer(ui, &t, cx);
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

/// The report a finished `raul ttw --json` printed, whatever its status, or why there is none.
fn report_of(result: &Result<Output, String>) -> Result<Report, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    let r = TtwReport::from_output(out).map_err(|e| {
        let what = e.view_error().what;
        if what.is_empty() {
            UNREADABLE.to_owned()
        } else {
            what
        }
    })?;
    Ok(Report {
        status: r.status,
        dry_run: r.dry_run,
        plugins: r.plugins_enabled,
        files: r.files,
        resumable: r.resumable,
        tool: r.tool.map(|t| t.name),
        code: r.error.as_ref().map(|e| e.code.clone()),
        message: r.error.map(|e| e.message),
    })
}

/// A share as a whole percentage, 0 to 100.
fn percent(fraction: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let p = (fraction.clamp(0.0, 1.0) * 100.0).round() as u8;
    p
}

fn step_name(step: Step) -> &'static str {
    match step {
        Step::Sources => STEP_SOURCES,
        Step::Output => STEP_OUTPUT,
        Step::Review => STEP_REVIEW,
        Step::Install => STEP_INSTALL,
    }
}

/// The words of a phase of the install's stream.
fn phase_words(phase: &str) -> &'static str {
    match phase {
        "package" => PHASE_PACKAGE,
        "assets" => PHASE_ASSETS,
        "archives" => PHASE_ARCHIVES,
        "post_install" => PHASE_POST,
        "add_mod" => PHASE_ADD_MOD,
        "" => PHASE_STARTING,
        _ => PHASE_WORKING,
    }
}

/// The four steps in a row that wraps: the one shown is marked, the ones before it are checked, and all four once
/// TTW is `installed`.
fn steps(ui: &mut egui::Ui, t: &Tokens, current: Step, installed: bool) {
    ui.horizontal_wrapped(|ui| {
        for step in Step::ALL {
            let here = step == current && !installed;
            let past = step.number() < current.number() || installed;
            let frame = egui::Frame::new()
                .fill(if here {
                    t.sel.color()
                } else {
                    egui::Color32::TRANSPARENT
                })
                .corner_radius(6)
                .inner_margin(egui::Margin::symmetric(10, 6));
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if past {
                        icon(ui, t, Mark::Ok);
                    }
                    let colour = if here || past { t.text } else { t.muted };
                    let mut name = RichText::new(step_name(step)).color(colour.color());
                    if here {
                        name = name.family(egui::FontFamily::Name(theme::SEMIBOLD.into()));
                    }
                    ui.label(name);
                });
            });
        }
    });
}

/// One source on a narrow page: its label, then its value and mark, wrapping onto the next line when they do not fit.
/// A size in bytes as megabytes or gigabytes, for the package found.
fn size_text(bytes: u64) -> String {
    const MB: u64 = 1_000_000;
    if bytes >= 1_000 * MB {
        format!(
            "{}.{} GB",
            bytes / (1_000 * MB),
            bytes % (1_000 * MB) / (100 * MB)
        )
    } else {
        format!("{} MB", bytes / MB)
    }
}

fn source_line(ui: &mut egui::Ui, t: &Tokens, label: &str, value: Option<&str>, defaults: bool) {
    ui.label(RichText::new(label).color(t.text.color()));
    // A path takes the width the page has and truncates; the mark goes on the line below it.
    match value {
        Some(v) => {
            ui.add(egui::Label::new(RichText::new(v).monospace().color(t.text.color())).truncate())
                .on_hover_text(v);
        }
        None if defaults => {
            ui.label(RichText::new(NEW_VEGAS_DEFAULT).color(t.muted.color()));
        }
        None => {
            ui.label(RichText::new(NOT_SET).color(t.muted.color()));
        }
    }
    match (value, defaults) {
        (Some(_), _) => mark(ui, t, Mark::Ok, SET),
        (None, true) => mark(ui, t, Mark::Unset, FROM_INSTANCE),
        (None, false) => mark(ui, t, Mark::Warning, NOT_SET),
    }
    ui.add_space(6.0);
}

/// One source: its label, its value (a path, truncated with the whole in a tooltip) and its mark.
fn source_row(
    ui: &mut egui::Ui,
    t: &Tokens,
    width: f32,
    label: &str,
    value: Option<&str>,
    defaults: bool,
) {
    ui.label(RichText::new(label).color(t.text.color()));
    let value_width = (width * 0.5).clamp(120.0, 520.0);
    ui.allocate_ui_with_layout(
        egui::vec2(value_width, 20.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(value_width);
            match value {
                Some(v) => {
                    ui.add(
                        egui::Label::new(RichText::new(v).monospace().color(t.text.color()))
                            .truncate(),
                    )
                    .on_hover_text(v);
                }
                None if defaults => {
                    ui.label(RichText::new(NEW_VEGAS_DEFAULT).color(t.muted.color()));
                }
                None => {
                    ui.label(RichText::new(NOT_SET).color(t.muted.color()));
                }
            }
        },
    );
    match (value, defaults) {
        (Some(_), _) => mark(ui, t, Mark::Ok, SET),
        (None, true) => mark(ui, t, Mark::Unset, FROM_INSTANCE),
        (None, false) => mark(ui, t, Mark::Warning, NOT_SET),
    }
    ui.end_row();
}

/// Label and value rows; a path value is monospace and truncates with the whole in a tooltip.
fn facts(ui: &mut egui::Ui, t: &Tokens, id: &str, rows: &[(&str, &str, bool)]) {
    let width = (ui.available_width() * 0.6).clamp(120.0, 620.0);
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 8.0])
        .show(ui, |ui| {
            for (label, value, path) in rows {
                ui.label(RichText::new(*label).color(t.muted.color()));
                let text = if *path {
                    RichText::new(*value).monospace()
                } else {
                    RichText::new(*value)
                };
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(width);
                        ui.add(egui::Label::new(text.color(t.text.color())).truncate())
                            .on_hover_text(*value);
                    },
                );
                ui.end_row();
            }
        });
}

/// A note: a mark, a line and an optional detail (a path, truncated).
fn note(ui: &mut egui::Ui, t: &Tokens, m: Mark, line: &str, detail: Option<&str>) {
    ui.horizontal_top(|ui| {
        icon(ui, t, m);
        ui.vertical(|ui| {
            ui.label(RichText::new(line).color(t.text.color()));
            if let Some(d) = detail {
                ui.add(
                    egui::Label::new(RichText::new(d).monospace().color(t.muted.color()))
                        .truncate(),
                )
                .on_hover_text(d);
            }
        });
    });
}

const INSTANCE: &str = "/users/player/raul-storage/instance";
const FALLOUT3: &str = "/games/steamapps/common/Fallout 3 goty";
const PACKAGE: &str = "/users/player/Downloads/TTW_3.4.0.mpi";
const DOWNLOADS: &str = "/users/player/Downloads";

fn sources(fallout3: Option<&str>, package: Option<&str>) -> Sources {
    Sources {
        fallout3: fallout3.map(str::to_owned),
        newvegas: None,
        package: package.map(str::to_owned),
    }
}

fn at(step: Step, sources: Sources) -> State {
    let mut s = State::new(Some(INSTANCE.to_owned()));
    s.sources_read(sources);
    s.step = step;
    s
}

fn reviewed() -> State {
    let mut s = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
    let _ = s.checking();
    let _ = s.checked(
        Report {
            tool: Some("ttwright".to_owned()),
            ..Report::of(Status::Success, true)
        },
        problem(&CHECK_FAILED, ""),
    );
    s
}

fn installing(done: u64, total: Option<u64>) -> State {
    let mut s = reviewed();
    let _ = s.installing();
    s.progressed(Progress {
        phase: "assets".to_owned(),
        done,
        total,
    });
    s
}

fn problem(e: &ErrorText, cause: &str) -> Problem {
    Problem {
        id: e.id,
        cause: cause.to_owned(),
        command: Some(check_command(INSTANCE).shown()),
    }
}

fn fixture(name: &'static str, state: State) -> Fixture {
    Fixture {
        name,
        page: Box::new(Ttw::with(state)),
        shell: ShellState {
            instance: Some(INSTANCE.to_owned()),
            new_vegas: Some("/games/steamapps/common/Fallout New Vegas".to_owned()),
            ..ShellState::default()
        },
    }
}

/// A fixture of the flow in `state`, looking for the package in the Downloads folder of the example player.
fn looking(name: &'static str, state: State) -> Fixture {
    Fixture {
        page: Box::new(Ttw::with(state).looking_in(DOWNLOADS)),
        ..fixture(name, State::default())
    }
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    let mut loading = State::new(Some(INSTANCE.to_owned()));
    loading.loading();
    let mut unreadable = State::new(Some(INSTANCE.to_owned()));
    unreadable.sources_failed("the configuration file is not valid TOML".to_owned());
    let mut checking = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
    let _ = checking.checking();
    let mut check_failed = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
    let _ = check_failed.checking();
    let _ = check_failed.checked(
        Report {
            code: Some("unsupported".to_owned()),
            message: Some("the package version is not supported by this installer".to_owned()),
            ..Report::of(Status::Unsupported, true)
        },
        problem(
            &CHECK_FAILED,
            "the package version is not supported by this installer",
        ),
    );
    let mut cancelling = installing(12, Some(48));
    let _ = cancelling.cancelling();
    let mut installed = installing(48, Some(48));
    let plugins: Vec<String> = (1..=40)
        .map(|i| format!("SampleWorldPart{i:02}.esm"))
        .collect();
    let _ = installed.installed(
        Report {
            plugins,
            files: Some(41_872),
            ..Report::of(Status::Success, false)
        },
        problem(&INSTALL_FAILED, ""),
    );
    let mut cancelled = installing(30, Some(48));
    let _ = cancelled.cancelling();
    let _ = cancelled.installed(
        Report {
            resumable: true,
            ..Report::of(Status::Cancelled, false)
        },
        Problem {
            command: Some(install_command(INSTANCE, false).shown()),
            ..problem(&INSTALL_CANCELLED, "cancelled on request")
        },
    );
    let mut unknown = installing(30, Some(48));
    let _ = unknown.install_failed(Problem {
        command: Some(install_command(INSTANCE, false).shown()),
        ..problem(
            &INSTALL_UNKNOWN,
            "The run stopped before it said how it ended.",
        )
    });
    let long_path = "/users/player/a folder with a very long name that keeps going on and on/and another one/Fallout 3 goty";
    // The search runs: its answer is not in yet, so the page shows it looking.
    let mut searching = at(Step::Sources, sources(Some(FALLOUT3), None));
    searching.searching();
    let mut found = at(Step::Sources, sources(Some(FALLOUT3), None));
    found.searched(
        std::path::Path::new(DOWNLOADS),
        Some(std::path::Path::new(PACKAGE)),
    );
    let mut not_found = at(Step::Sources, sources(Some(FALLOUT3), None));
    not_found.searched(std::path::Path::new(DOWNLOADS), None);
    vec![
        Fixture {
            name: "no-instance",
            page: Box::new(Ttw::with(State::default())),
            shell: ShellState::default(),
        },
        fixture("loading", loading),
        fixture("unreadable", unreadable),
        fixture("sources-missing", at(Step::Sources, sources(None, None))),
        fixture(
            "sources-ready",
            at(Step::Sources, sources(Some(FALLOUT3), Some(PACKAGE))),
        ),
        looking("sources-search", searching),
        looking("sources-found", found),
        looking("sources-not-found", not_found),
        fixture(
            "long-text",
            at(Step::Sources, sources(Some(long_path), Some(PACKAGE))),
        ),
        fixture(
            "output",
            at(Step::Output, sources(Some(FALLOUT3), Some(PACKAGE))),
        ),
        fixture("checking", checking),
        fixture("check-failed", check_failed),
        fixture("review", reviewed()),
        fixture("installing", installing(12, Some(48))),
        fixture("installing-no-estimate", installing(0, None)),
        fixture("cancelling", cancelling),
        fixture("installed", installed),
        fixture("cancelled", cancelled),
        fixture("unknown", unknown),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(code: i64, stdout: &str) -> Output {
        Output {
            code: Some(code),
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    // The result document of the command; its `profile` field is internal and never shown.
    const RESULT: &str = concat!(
        r#"{"schema_version":1,"contract":"raul-events","seq":1,"command":"ttw","event":"result","result":{"schema_version":1,"contract":"raul-ttw-result","status":"cancelled","dry_run":false,"resumed":false,"instance":"/i","profile":"Default","mod":null,"plugins_enabled":[],"files":null,"staging":"/i/s","resumable":true,"tool":{"name":"ttwright","path":"/t","sha256":"00","version":"0.3.0"},"ttwright":null,"error":{"code":"cancelled","message":"stopped","ttwright_code":"cancelled"}}}"#,
        "\n{\"schema_version\":1,\"contract\":\"raul-events\",\"seq\":2,\"command\":\"ttw\",\"event\":\"finished\",\"status\":\"cancelled\",\"exit_code\":4,\"code\":\"cancelled\",\"message\":\"stopped\",\"resumable\":true,\"changed\":false}"
    );

    #[test]
    fn a_ttw_stream_becomes_the_report_the_page_shows() {
        let r = report_of(&Ok(output(4, RESULT))).unwrap();
        assert_eq!(r.status, Status::Cancelled);
        assert!(r.resumable);
        assert_eq!(r.tool.as_deref(), Some("ttwright"), "no Component version");
        assert_eq!(r.code.as_deref(), Some("cancelled"));
        assert_eq!(r.message.as_deref(), Some("stopped"));
        assert!(report_of(&Ok(output(0, "not a stream"))).is_err());
        assert_eq!(report_of(&Err("no raul".into())).unwrap_err(), "no raul");
    }

    #[test]
    fn the_header_follows_the_flow() {
        assert_eq!(Ttw::with(State::default()).state_line(), STATE_NO_INSTANCE);
        let s = at(Step::Sources, sources(None, None));
        assert_eq!(Ttw::with(s).state_line(), "Step 1 of 4 · Sources");
        assert_eq!(
            Ttw::with(installing(12, Some(48))).state_line(),
            "Installing · 25%"
        );
        assert_eq!(
            Ttw::with(installing(0, None)).state_line(),
            STATE_INSTALLING
        );
        let mut done = installing(1, Some(1));
        let _ = done.installed(
            Report::of(Status::Success, false),
            problem(&INSTALL_FAILED, ""),
        );
        assert_eq!(Ttw::with(done).state_line(), STATE_INSTALLED);
        assert_eq!(percent(2.0), 100);
        assert_eq!(phase_words("something new"), PHASE_WORKING);
        assert_eq!(phase_words(""), PHASE_STARTING);
    }

    #[test]
    fn next_says_what_it_waits_for() {
        let page = Ttw::with(at(Step::Sources, sources(None, Some(PACKAGE))));
        assert_eq!(page.waits_for(), Some(MISSING_FALLOUT3));
        let page = Ttw::with(at(Step::Sources, sources(Some(FALLOUT3), None)));
        assert_eq!(page.waits_for(), Some(MISSING_PACKAGE));
        let page = Ttw::with(at(Step::Sources, sources(None, None)));
        assert_eq!(page.waits_for(), Some(MISSING_BOTH));
        let page = Ttw::with(reviewed());
        assert_eq!(page.waits_for(), None);
        let mut checking = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
        let _ = checking.checking();
        assert_eq!(Ttw::with(checking).waits_for(), Some(WAITS_FOR_CHECK));
    }

    /// Answers `raul config set` as the contract does and records every command.
    #[derive(Default)]
    struct Recording {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
    }

    impl raul_ui_service::Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            let value = args.get(3).cloned().unwrap_or_default();
            Ok(Output {
                code: Some(0),
                stdout: format!(
                    r#"{{"schema_version":1,"contract":"raul-config","file":"/c","entries":[{{"key":"ttw.package","value":"{value}","read_only":false}}],"error":null}}"#
                ),
                stderr: String::new(),
            })
        }
    }

    /// Answers `raul config set` with a refusal, as the contract does, and records every command.
    #[derive(Default)]
    struct Refusing {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
    }

    impl raul_ui_service::Exec for Refusing {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            Ok(Output {
                code: Some(1),
                stdout: concat!(
                    r#"{"schema_version":1,"contract":"raul-config","file":"/c","entries":[],"#,
                    r#""error":{"code":"invalid_value","key":"ttw.package","message":"the package is not a TTW installer"}}"#,
                )
                .to_owned(),
                stderr: String::new(),
            })
        }
    }

    /// A fake Downloads folder with one package in it, and the recorder of the commands the page runs.
    fn fake_downloads(label: &str) -> (std::path::PathBuf, std::sync::Arc<Recording>) {
        let root =
            std::env::temp_dir().join(format!("raul-ttw-gui-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("unpacked")).unwrap();
        std::fs::write(root.join("notes.txt"), b"not a package").unwrap();
        // A package-shaped file: the search skips a file that is too small or does not start like a package.
        let mut body = vec![0u8; 1_000_000];
        body[..4].copy_from_slice(b"PK\x03\x04");
        std::fs::write(root.join("unpacked/TTW_3.4.0.mpi"), body).unwrap();
        (root, std::sync::Arc::new(Recording::default()))
    }

    fn harness<E>(
        fixture: Fixture,
        exec: std::sync::Arc<E>,
    ) -> egui_kittest::Harness<'static, crate::shell::App>
    where
        E: raul_ui_service::Exec + Send + Sync + 'static,
    {
        let app = crate::shell::App::fixture(fixture, crate::jobs::Jobs::new(exec));
        egui_kittest::Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            })
    }

    #[test]
    fn a_watched_download_is_used_only_once_it_has_stopped_growing() {
        use egui_kittest::kittest::Queryable;
        let (downloads, exec) = fake_downloads("watch");
        let _ = std::fs::remove_dir_all(downloads.join("unpacked"));
        let file = downloads.join("TTW_9.9.9.mpi");
        let mut h = harness(
            Fixture {
                page: Box::new(
                    Ttw::with(at(Step::Sources, sources(Some(FALLOUT3), None)))
                        .looking_in(downloads.clone()),
                ),
                ..fixture("sources-missing", State::default())
            },
            exec.clone(),
        );
        h.run();
        assert!(exec.calls.lock().unwrap().is_empty());
        h.get_by_label(WATCH).click();
        h.run_steps(2);
        // A download that grows: package-shaped from the first bytes, a megabyte only at the end.
        let writer = {
            let file = file.clone();
            std::thread::spawn(move || {
                let mut body = vec![0u8; 400_000];
                body[..4].copy_from_slice(b"PK\x03\x04");
                for extra in [0usize, 300_000, 300_000] {
                    body.extend(std::iter::repeat_n(1u8, extra));
                    std::fs::write(&file, &body).unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(800));
                }
            })
        };
        std::thread::sleep(std::time::Duration::from_millis(1500));
        h.run_steps(2);
        assert!(
            exec.calls.lock().unwrap().is_empty(),
            "a file that is still growing is not used"
        );
        let want = file.display().to_string();
        let mut used = false;
        for _ in 0..1000 {
            h.run_steps(2);
            used =
                exec.calls.lock().unwrap().iter().any(|c| {
                    c.get(2).map(String::as_str) == Some("ttw.package") && c.contains(&want)
                });
            if used {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        writer.join().unwrap();
        assert!(used, "the package that arrived is saved and used");
        let _ = std::fs::remove_dir_all(&downloads);
    }

    #[test]
    fn find_it_for_me_shows_the_package_it_found_and_use_this_file_saves_it() {
        use egui_kittest::kittest::Queryable;
        let (downloads, exec) = fake_downloads("found");
        let file = format!("{}/unpacked/TTW_3.4.0.mpi", downloads.display());
        let mut h = harness(
            Fixture {
                page: Box::new(
                    Ttw::with(at(Step::Sources, sources(Some(FALLOUT3), None)))
                        .looking_in(downloads.clone()),
                ),
                ..fixture("sources-missing", State::default())
            },
            exec.clone(),
        );
        h.run();
        h.get_by_label(FIND_IT).click();
        for _ in 0..100 {
            h.run_steps(2);
            if h.query_by_label(FOUND).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(h.query_by_label(FOUND).is_some(), "the found state");
        assert!(
            h.query_by_label_contains(&file).is_some(),
            "the file it found is named"
        );
        assert!(
            h.query_by_label_contains("1 MB").is_some(),
            "and so is its size"
        );
        h.get_by_label(USE_THIS).click();
        for _ in 0..100 {
            if !exec.calls.lock().unwrap().is_empty() {
                break;
            }
            h.run_steps(2);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            exec.calls.lock().unwrap().first().cloned(),
            Some(
                ["config", "set", "ttw.package", file.as_str(), "--json"]
                    .map(String::from)
                    .to_vec()
            )
        );
        let _ = std::fs::remove_dir_all(&downloads);
    }

    #[test]
    fn a_refused_save_says_so_and_does_not_use_the_file() {
        use egui_kittest::kittest::Queryable;
        let (downloads, _) = fake_downloads("refused");
        let file = format!("{}/unpacked/TTW_3.4.0.mpi", downloads.display());
        let exec = std::sync::Arc::new(Refusing::default());
        let mut h = harness(
            Fixture {
                page: Box::new(
                    Ttw::with(at(Step::Sources, sources(Some(FALLOUT3), None)))
                        .looking_in(downloads.clone()),
                ),
                ..fixture("sources-missing", State::default())
            },
            exec.clone(),
        );
        h.run();
        h.get_by_label(FIND_IT).click();
        for _ in 0..100 {
            h.run_steps(2);
            if h.query_by_label(FOUND).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(h.query_by_label(FOUND).is_some(), "the found state");
        h.get_by_label(USE_THIS).click();
        for _ in 0..100 {
            if !exec.calls.lock().unwrap().is_empty() {
                break;
            }
            h.run_steps(2);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            exec.calls.lock().unwrap().first().cloned(),
            Some(
                ["config", "set", "ttw.package", file.as_str(), "--json"]
                    .map(String::from)
                    .to_vec()
            )
        );
        for _ in 0..100 {
            h.run_steps(2);
            if h.query_by_label(PACKAGE_NOT_SAVED.what).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            h.query_by_label(PACKAGE_NOT_SAVED.what).is_some(),
            "the save failure is shown"
        );
        assert!(
            h.query_by_label(MISSING_PACKAGE).is_some(),
            "the file is not used"
        );
        let _ = std::fs::remove_dir_all(&downloads);
    }

    #[test]
    fn find_it_for_me_says_where_it_looked_when_there_is_no_package() {
        use egui_kittest::kittest::Queryable;
        let empty = std::env::temp_dir().join(format!("raul-ttw-gui-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).unwrap();
        let exec = std::sync::Arc::new(Recording::default());
        let mut h = harness(
            Fixture {
                page: Box::new(
                    Ttw::with(at(Step::Sources, sources(Some(FALLOUT3), None)))
                        .looking_in(empty.clone()),
                ),
                ..fixture("sources-missing", State::default())
            },
            exec,
        );
        h.run();
        h.get_by_label(FIND_IT).click();
        for _ in 0..100 {
            h.run_steps(2);
            if h.query_by_label(NOT_FOUND).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(h.query_by_label(NOT_FOUND).is_some(), "the not found state");
        assert!(
            h.query_by_label(&empty.display().to_string()).is_some(),
            "it says where it looked"
        );
        let _ = std::fs::remove_dir_all(&empty);
    }
}
