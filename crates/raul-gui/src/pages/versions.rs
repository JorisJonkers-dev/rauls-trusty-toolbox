// SPDX-License-Identifier: GPL-3.0-only
//! About and Update (the Versions page): raul's own version, the update check and a block of details to copy
//! (UI-RULES P3, P5, T4).
//!
//! It shows the one version a person sees, the Toolbox's own, as plain text, with the one update action beside it as
//! a real button: Check for updates, or Update once a newer release is known. It runs `raul update check`, which
//! reads the release list and downloads nothing. Every packaged Component comes with raul and changes only when raul
//! does, so no Component version and no per-Component action is shown. Nothing reaches the network before that
//! click.

use super::home::Mark;
use super::settings::{Pending, WAITING, error_block, failure_of, start, waiting};
use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::Dialog;
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::pages::settings::Fetch;
use raul_ui_model::pages::versions::{
    Apply, Release, Shipped, State, Step, Update, UpdateAction, UpdateStatus, download_command,
    install_update_command, parse_applied, parse_status, parse_update, parse_version,
    rollback_command, status_command, update_command, version_command,
};
use raul_ui_model::pages::{Attention, PageId, View};
use raul_ui_service::Output;

crate::texts!(TEXTS {
    STATE_CHECKING = "Checking for updates";
    STATE_NEWER = "A newer release is out";
    STATE_NEWEST = "This is the newest release";
    STATE_IDLE = "Update check not run";
    SECTION_ABOUT = "About";
    SECTION_DETAILS = "Details";
    TOOLBOX_LABEL = "Toolbox";
    CHECK_UPDATES = "Check for updates";
    UPDATE = "Update";
    UPDATE_READING = "Reading the version";
    UPDATE_CHECKING = "Checking for updates";
    UPDATE_NEWEST = "This is the newest release.";
    UPDATE_FOUND = "Version {version} is out; you run {current}.";
    UPDATE_OPEN = "Release page";
    UPDATE_NOTE = "Every step is yours to run: nothing is looked up or downloaded until you press its button. An update replaces the Toolbox and everything that comes with it, and the version before is kept so you can go back.";
    DOWNLOAD = "Download update";
    INSTALL = "Install update";
    RESTART = "Restart";
    GO_BACK = "Go back to version {version}";
    DOWNLOADING = "Downloading and checking version {version}. This can take a minute.";
    DOWNLOADED = "Version {version} is downloaded and checked. Installing it replaces the version you run.";
    INSTALLING = "Installing version {version} and checking that it works.";
    INSTALLED = "Version {version} is installed. Restart to use it.";
    ROLLING_BACK = "Going back to the version before.";
    WENT_BACK = "Version {version} is back. Restart to use it.";
    KEPT = "Version {version} from before the last update is kept.";
    NOT_HERE = "{reason}";
    COPY_DETAILS = "Copy details";
    WAITING_NOTE = "This build cannot read it from the window yet.";
});

crate::errors!(ERRORS {
    UPDATE_FAILED = (
        "The update check could not finish",
        "Nothing changed. The line below says why."
    );
    APPLY_FAILED = (
        "The update did not finish",
        "Nothing is half done: the version you run is still in place. The line below says why."
    );
    VERSION_UNREADABLE = (
        "The version could not be read",
        "Nothing changed. The line below says why."
    );
});

const UPDATE_DIALOG: &str = "versions-update";

/// The About and Update page.
#[derive(Debug, Default)]
pub struct Versions {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    loaded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Version,
    Update,
    Status,
    Download,
    Install,
    Rollback,
}

impl Versions {
    /// The page in `state`, as a fixture shows it.
    pub fn with(state: State) -> Versions {
        Versions {
            state,
            ..Versions::default()
        }
    }

    fn read_version(&mut self, cx: &mut PageCx<'_>) {
        match start(cx, &version_command()) {
            Ok(id) => {
                self.pending.add(Job::Version, id);
                self.state.shipped = Fetch::Loading;
            }
            Err(e) => self.state.shipped = Fetch::Failed(e.what.to_owned()),
        }
    }

    /// Check for updates: `raul update check` reads the release list and downloads nothing.
    fn check_updates(&mut self, cx: &mut PageCx<'_>) {
        match start(cx, &update_command()) {
            Ok(id) => {
                self.pending.add(Job::Update, id);
                self.state.update = Update::Checking;
            }
            Err(e) => self.state.update = Update::Failed(e.what.to_owned()),
        }
    }

    /// Read what this install can do about updates; local, nothing is fetched.
    fn read_status(&mut self, cx: &mut PageCx<'_>) {
        match start(cx, &status_command()) {
            Ok(id) => {
                self.pending.add(Job::Status, id);
                self.state.status = Fetch::Loading;
            }
            Err(e) => self.state.status = Fetch::Failed(e.what.to_owned()),
        }
    }

    /// Run one step of applying an update and note it as under way.
    fn run_step(&mut self, cx: &mut PageCx<'_>, step: &Step) {
        let (command, job, under_way) = match step {
            Step::Download(v) => (download_command(v), Job::Download, Apply::Downloading),
            Step::Install => (install_update_command(), Job::Install, Apply::Installing),
            Step::Restart => {
                restart(cx.repaint.as_ref());
                return;
            }
        };
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(job, id);
                self.state.apply = under_way;
            }
            Err(e) => self.state.apply = Apply::Failed(e.what.to_owned()),
        }
    }

    fn go_back(&mut self, cx: &mut PageCx<'_>) {
        match start(cx, &rollback_command()) {
            Ok(id) => {
                self.pending.add(Job::Rollback, id);
                self.state.apply = Apply::RollingBack;
            }
            Err(e) => self.state.apply = Apply::Failed(e.what.to_owned()),
        }
    }

    /// Reads what a finished step answered: its version, or the one line that says why it did not work.
    fn take_step(result: &Result<Output, String>, key: &str) -> Result<String, String> {
        match result {
            Ok(out) if out.code == Some(0) => {
                parse_applied(&out.stdout, key).map_err(|_| APPLY_FAILED.detail.to_owned())
            }
            Ok(out) => Err(failure_of(out)),
            Err(why) => Err(why.clone()),
        }
    }

    /// Reads what the update check answered, or why there is none.
    fn take_update(&mut self, result: &Result<Output, String>) {
        self.state.update = match result {
            Ok(out) => {
                update_answer(out).map_or_else(Update::Failed, |(running, newer)| Update::Checked {
                    raul: running.raul,
                    newer,
                })
            }
            Err(why) => Update::Failed(why.clone()),
        };
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            match job {
                Job::Version => {
                    self.state.shipped = match &result {
                        Ok(out) => parse_version(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
                Job::Update => self.take_update(&result),
                Job::Status => {
                    self.state.status = match &result {
                        Ok(out) if out.code == Some(0) => parse_status(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Ok(out) => Fetch::Failed(failure_of(out)),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                    self.state.settle_from_status();
                }
                Job::Download => {
                    self.state.apply = match Self::take_step(&result, "staged") {
                        Ok(v) => Apply::Ready(v),
                        Err(why) => Apply::Failed(why),
                    };
                }
                Job::Install => {
                    self.state.apply = match Self::take_step(&result, "installed") {
                        Ok(v) => Apply::Installed(v),
                        Err(why) => Apply::Failed(why),
                    };
                    self.read_status(cx);
                }
                Job::Rollback => {
                    self.state.apply = match Self::take_step(&result, "back_to") {
                        Ok(v) => Apply::WentBack(v),
                        Err(why) => Apply::Failed(why),
                    };
                    self.read_status(cx);
                }
            }
        }
    }
}

impl Page for Versions {
    fn view(&self) -> View {
        View::Page(PageId::Versions)
    }

    fn title(&self) -> &'static str {
        msg::PAGE_VERSIONS
    }

    fn state_line(&self) -> String {
        match &self.state.update {
            Update::Checking => STATE_CHECKING,
            Update::Checked { newer: Some(_), .. } => STATE_NEWER,
            Update::Checked { newer: None, .. } => STATE_NEWEST,
            Update::Idle | Update::Failed(_) => STATE_IDLE,
        }
        .to_owned()
    }

    fn attention(&self) -> Option<Attention> {
        None
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        // The version never changes while the window is open: it is read when the page first opens.
        if !self.loaded {
            self.loaded = true;
            if matches!(self.state.shipped, Fetch::Idle) {
                self.read_version(cx);
            }
            if matches!(self.state.status, Fetch::Idle) {
                self.read_status(cx);
            }
        }
        self.take_answers(cx);
        let _ = cx.dialogs.take_answer(UPDATE_DIALOG);
        if self.state.update == Update::Failed(WAITING.what.to_owned()) {
            // The check cannot run from this window yet; it is not tried again on every frame.
            self.state.update = Update::Idle;
        }
        let t = *cx.tokens;
        shell::section_label(ui, &t, SECTION_ABOUT, true);
        self.about_section(ui, cx);
        ui.add_space(16.0);
        shell::section_label(ui, &t, SECTION_DETAILS, true);
        self.details_section(ui, cx);
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl Versions {
    /// The one version a person sees, as plain text, with the one update action beside it as a real button.
    fn about_section(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let version = self.state.raul().map(str::to_owned);
        let action = self.state.update_action();
        let step = self.state.next_step();
        let can = self.state.can_check_updates() && !self.state.busy();
        let mut clicked = false;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(TOOLBOX_LABEL).color(t.muted.color()));
            if let Some(v) = &version {
                ui.label(RichText::new(v).monospace().color(t.text.color()));
            }
            let label = match (&step, action) {
                (Some(Step::Download(_)), _) => DOWNLOAD.to_owned(),
                (Some(Step::Install), _) => INSTALL.to_owned(),
                (Some(Step::Restart), _) => RESTART.to_owned(),
                (None, UpdateAction::Update) if self.state.not_updatable().is_none() => {
                    UPDATE.to_owned()
                }
                (None, _) => CHECK_UPDATES.to_owned(),
            };
            let primary = step.is_some() || action == UpdateAction::Update;
            let button = if primary {
                shell::primary_button(&t, &label)
            } else {
                egui::Button::new(&label)
            };
            clicked = ui.add_enabled(can, button).clicked();
        });
        if clicked {
            match &step {
                Some(step) => self.run_step(cx, step),
                None => self.check_updates(cx),
            }
        }
        match &self.state.update {
            Update::Idle => {}
            Update::Checking => {
                ui.horizontal_wrapped(|ui| {
                    super::home::icon(ui, &t, Mark::Info);
                    let line = RichText::new(UPDATE_CHECKING).color(t.muted.color());
                    ui.add(egui::Label::new(line).wrap());
                });
            }
            Update::Checked { raul, newer } => match newer {
                Some(r) => {
                    let line = UPDATE_FOUND
                        .replace("{version}", &r.version)
                        .replace("{current}", raul);
                    ui.horizontal_wrapped(|ui| {
                        wrapped_mark(ui, &t, Mark::Info, &line);
                        if ui.small_button(UPDATE_OPEN).clicked() {
                            cx.dialogs.open(Dialog::info(
                                UPDATE_DIALOG,
                                UPDATE,
                                vec![r.url.clone()],
                            ));
                        }
                    });
                }
                None => {
                    note(ui, &t, UPDATE_NEWEST);
                }
            },
            Update::Failed(why) if why == WAITING.what => {
                note(ui, &t, WAITING_NOTE);
            }
            Update::Failed(why) => error_block(ui, &t, &UPDATE_FAILED, Some(why)),
        }
        self.apply_lines(ui, cx);
        if version.is_none() {
            match &self.state.shipped {
                Fetch::Failed(why) if why == WAITING.what => {
                    note(ui, &t, WAITING_NOTE);
                }
                Fetch::Failed(why) => {
                    error_block(ui, &t, &VERSION_UNREADABLE, Some(why));
                }
                Fetch::Idle | Fetch::Loading => {
                    waiting(ui, &t, UPDATE_READING, &version_command());
                }
                Fetch::Ready(_) => {}
            }
        }
    }

    /// The lines of the update steps: what is under way or done, why this install cannot update itself, and the way back.
    fn apply_lines(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let version_of = |v: &str, text: &str| text.replace("{version}", v);
        match self.state.apply.clone() {
            Apply::Idle => {}
            Apply::Downloading => {
                let v = self
                    .state
                    .newer()
                    .map_or(String::new(), |r| r.version.clone());
                wrapped_mark(ui, &t, Mark::Info, &version_of(&v, DOWNLOADING));
            }
            Apply::Ready(v) => wrapped_mark(ui, &t, Mark::Ok, &version_of(&v, DOWNLOADED)),
            Apply::Installing => {
                let v = self
                    .state
                    .newer()
                    .map_or(String::new(), |r| r.version.clone());
                wrapped_mark(ui, &t, Mark::Info, &version_of(&v, INSTALLING));
            }
            Apply::Installed(v) => wrapped_mark(ui, &t, Mark::Ok, &version_of(&v, INSTALLED)),
            Apply::RollingBack => wrapped_mark(ui, &t, Mark::Info, ROLLING_BACK),
            Apply::WentBack(v) => wrapped_mark(ui, &t, Mark::Ok, &version_of(&v, WENT_BACK)),
            Apply::Failed(why) => error_block(ui, &t, &APPLY_FAILED, Some(&why)),
        }
        if let Some(reason) = self.state.not_updatable() {
            note(ui, &t, reason);
        }
        if let Some(previous) = self.state.go_back_to().map(str::to_owned) {
            ui.horizontal_wrapped(|ui| {
                note(ui, &t, &version_of(&previous, KEPT));
                if ui.small_button(version_of(&previous, GO_BACK)).clicked() {
                    self.go_back(cx);
                }
            });
        }
    }

    /// The details block, with one button that copies it.
    fn details_section(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let text = self
            .state
            .details(std::env::consts::OS, std::env::consts::ARCH);
        ui.add(egui::Label::new(RichText::new(&text).monospace().color(t.text.color())).wrap());
        ui.add_space(4.0);
        if ui.button(COPY_DETAILS).clicked() {
            ui.ctx().copy_text(text);
        }
        ui.add_space(8.0);
        note(ui, &t, UPDATE_NOTE);
    }
}

/// Start raul again and close this window: the new version runs in a fresh process. When it cannot be started the
/// window stays, so nothing is lost.
fn restart(repaint: Option<&egui::Context>) {
    let program = std::env::var_os("APPIMAGE")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_exe().ok());
    let (Some(program), Ok(cwd)) = (program, std::env::current_dir()) else {
        return;
    };
    let env: std::collections::BTreeMap<String, String> = std::env::vars().collect();
    let started = raul_runner::start_program(
        &program.to_string_lossy(),
        &[] as &[&str],
        &env,
        &cwd,
        false,
        None,
    );
    if started.is_ok()
        && let Some(ctx) = repaint
    {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

/// A line of secondary text that wraps inside the page instead of reaching past its right edge (UI-RULES A5, A6).
fn note(ui: &mut egui::Ui, t: &Tokens, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).color(t.muted.color())).wrap());
}

fn wrapped_mark(ui: &mut egui::Ui, t: &Tokens, m: Mark, word: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        super::home::icon(ui, t, m);
        let text = RichText::new(word).color(m.colour(t));
        ui.add(egui::Label::new(text).wrap());
    });
}

/// What a finished `raul update check` answered, or why there is none: the command failed, or its answer is not the
/// document. The release list is the network's, so its answer may be anything.
fn update_answer(out: &Output) -> Result<(Shipped, Option<Release>), String> {
    if out.code != Some(0) {
        return Err(failure_of(out));
    }
    parse_update(&out.stdout).map_err(|_| UPDATE_FAILED.detail.to_owned())
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    let shipped = Shipped {
        raul: "0.2.0".into(),
    };
    let newer = Release {
        version: "0.3.0".into(),
        url: "https://example.invalid/rauls-trusty-toolbox/releases/tag/v0.3.0".into(),
    };
    let checked = |newer: Option<Release>| {
        Versions::with(State {
            shipped: Fetch::Ready(shipped.clone()),
            update: Update::Checked {
                raul: "0.2.0".into(),
                newer,
            },
            ..State::default()
        })
    };
    vec![
        Fixture::new(
            "checking",
            Versions::with(State {
                shipped: Fetch::Loading,
                ..State::default()
            }),
        ),
        Fixture::new(
            "active",
            Versions::with(State {
                shipped: Fetch::Ready(shipped.clone()),
                ..State::default()
            }),
        ),
        Fixture::new(
            "waiting",
            Versions::with(State {
                shipped: Fetch::Failed(WAITING.what.to_owned()),
                ..State::default()
            }),
        ),
        Fixture::new(
            "error",
            Versions::with(State {
                shipped: Fetch::Failed("raul version: the version could not be read".into()),
                ..State::default()
            }),
        ),
        Fixture::new("update-checking", {
            let mut page = checked(None);
            page.state.update = Update::Checking;
            page
        }),
        Fixture::new("update-available", checked(Some(newer))),
        Fixture::new("up-to-date", checked(None)),
        Fixture::new(
            "update-failed",
            Versions::with(State {
                shipped: Fetch::Ready(shipped.clone()),
                update: Update::Failed(
                    "raul update check: the release list could not be read".into(),
                ),
                ..State::default()
            }),
        ),
    ]
    .into_iter()
    .chain(step_fixtures(&shipped))
    .collect()
}

/// The states of the update steps the snapshot tests render.
fn step_fixtures(shipped: &Shipped) -> Vec<Fixture> {
    let steps = |apply: Apply, previous: Option<&str>| {
        Versions::with(State {
            shipped: Fetch::Ready(shipped.clone()),
            update: Update::Checked {
                raul: "0.2.0".into(),
                newer: Some(Release {
                    version: "0.3.0".into(),
                    url: "https://example.invalid/rauls-trusty-toolbox/releases/tag/v0.3.0".into(),
                }),
            },
            status: Fetch::Ready(UpdateStatus {
                updatable: true,
                reason: None,
                staged: None,
                previous: previous.map(str::to_owned),
            }),
            apply,
        })
    };
    vec![
        Fixture::new(
            "update-downloaded",
            steps(Apply::Ready("0.3.0".into()), None),
        ),
        Fixture::new("update-installing", steps(Apply::Installing, None)),
        Fixture::new(
            "update-installed",
            steps(Apply::Installed("0.3.0".into()), Some("0.2.0")),
        ),
        Fixture::new(
            "update-apply-failed",
            steps(Apply::Failed("The new version did not start.".into()), None),
        ),
        Fixture::new("update-previous-kept", {
            let mut page = steps(Apply::Idle, Some("0.1.0"));
            page.state.update = Update::Checked {
                raul: "0.2.0".into(),
                newer: None,
            };
            page
        }),
        Fixture::new(
            "update-not-here",
            Versions::with(State {
                shipped: Fetch::Ready(shipped.clone()),
                update: Update::Checked {
                    raul: "0.2.0".into(),
                    newer: Some(Release {
                        version: "0.3.0".into(),
                        url: "https://example.invalid/rauls-trusty-toolbox/releases/tag/v0.3.0"
                            .into(),
                    }),
                },
                status: Fetch::Ready(UpdateStatus {
                    updatable: false,
                    reason: Some("This copy was installed with Nix, so Nix updates it.".into()),
                    ..UpdateStatus::default()
                }),
                ..State::default()
            }),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::Jobs;
    use crate::shell::App;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};
    use raul_ui_service::Exec;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Recording {
        calls: Mutex<Vec<Vec<String>>>,
        /// `update check` fails as it does on a machine without the network.
        fail_update: bool,
    }

    impl Recording {
        /// A recorder whose `update check` fails.
        fn failing() -> Recording {
            Recording {
                fail_update: true,
                ..Recording::default()
            }
        }

        /// How many times `command` ran.
        fn count(&self, command: &str) -> usize {
            self.calls
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.join(" ").starts_with(command))
                .count()
        }
    }

    impl Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            if self.fail_update && args.get(..2) == Some(&["update".to_owned(), "check".to_owned()])
            {
                return Ok(Output {
                    code: Some(1),
                    stdout: String::new(),
                    stderr: "raul update check: the release list could not be read".to_owned(),
                });
            }
            let stdout = match args.first().map(String::as_str) {
                Some("version") => r#"{"raul":"0.2.0"}"#.to_owned(),
                Some("update") if args.get(1).map(String::as_str) == Some("status") => {
                    r#"{"current":"0.2.0","status":{"updatable":true,"reason":null,"staged":null,"previous":null}}"#
                        .to_owned()
                }
                Some("update") if args.get(1).map(String::as_str) == Some("download") => {
                    r#"{"ok":true,"staged":{"from":"0.2.0","to":"0.3.0","files":["raul"]}}"#.to_owned()
                }
                Some("update") if args.get(1).map(String::as_str) == Some("install") => {
                    r#"{"ok":true,"installed":{"from":"0.2.0","to":"0.3.0","files":["raul"]}}"#.to_owned()
                }
                Some("update") if args.get(1).map(String::as_str) == Some("rollback") => {
                    r#"{"ok":true,"back_to":"0.2.0"}"#.to_owned()
                }
                Some("update") => {
                    r#"{"current":"0.2.0","newer":{"version":"0.3.0","url":"https://example.invalid/r"}}"#
                        .to_owned()
                }
                _ => String::new(),
            };
            Ok(Output {
                code: Some(0),
                stdout,
                stderr: String::new(),
            })
        }
    }

    /// The page in fixture `name`, with every command answered and recorded.
    fn harness(name: &str, exec: Arc<Recording>) -> Harness<'static, App> {
        let fixture = fixtures().into_iter().find(|f| f.name == name).unwrap();
        let app = App::fixture(fixture, Jobs::new(exec));
        Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            })
    }

    /// Runs frames until `command` was started, and answers with what it was started with. The last call of it
    /// counts, since a fixture may already have run it.
    fn wait_for_call(
        h: &mut Harness<'static, App>,
        exec: &Recording,
        command: &str,
    ) -> Vec<String> {
        for _ in 0..200 {
            h.run_steps(2);
            let found = exec
                .calls
                .lock()
                .unwrap()
                .iter()
                .rfind(|c| c.join(" ").starts_with(command))
                .cloned();
            if let Some(call) = found {
                return call;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("`raul {command}` never ran");
    }

    #[test]
    fn the_update_button_sits_beside_the_version_and_checks_for_updates() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("active", exec.clone());
        h.run();
        h.run_steps(2);
        assert_eq!(
            exec.count("update check"),
            0,
            "nothing checks before a click"
        );
        assert!(
            h.query_by_label(super::TOOLBOX_LABEL).is_some(),
            "the version is plain text beside its label"
        );
        assert!(
            h.query_by_label("0.2.0").is_some(),
            "the Toolbox's own version is shown"
        );
        let button = h.get_by_label(CHECK_UPDATES);
        assert_eq!(
            button.accesskit_node().role(),
            eframe::egui::accesskit::Role::Button,
            "the update action is a real button beside the version"
        );
        button.click();
        let call = wait_for_call(&mut h, &exec, "update check");
        assert_eq!(call, ["update", "check", "--json"], "the button checks");
        wait_for_label(&mut h, DOWNLOAD);
        assert!(
            h.query_by_label(DOWNLOAD).is_some(),
            "a newer release turns the action into Download update"
        );
        assert!(
            h.query_all_by_label_contains("0.3.0").next().is_some(),
            "the release that is out is named"
        );
    }

    #[test]
    fn the_update_goes_through_download_install_and_restart_one_press_at_a_time() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("update-available", exec.clone());
        h.run_steps(4);
        wait_for_label(&mut h, DOWNLOAD);
        assert_eq!(
            exec.count("update download"),
            0,
            "nothing downloads before a press"
        );
        h.get_by_label(DOWNLOAD).click();
        let call = wait_for_call(&mut h, &exec, "update download");
        assert_eq!(call, ["update", "download", "--tag", "v0.3.0", "--json"]);
        wait_for_label(&mut h, INSTALL);
        assert_eq!(
            exec.count("update install"),
            0,
            "installing is a press of its own"
        );
        h.get_by_label(INSTALL).click();
        wait_for_call(&mut h, &exec, "update install");
        wait_for_label(&mut h, RESTART);
        assert!(
            h.query_by_label(RESTART).is_some(),
            "after an install the next step is Restart"
        );
        assert!(
            h.query_by_label_contains("Version 0.3.0 is installed")
                .is_some(),
            "the page says what happened"
        );
    }

    #[test]
    fn going_back_is_offered_while_a_previous_version_is_kept_and_runs_on_a_press() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("update-previous-kept", exec.clone());
        h.run_steps(4);
        let label = GO_BACK.replace("{version}", "0.1.0");
        assert!(h.query_by_label(&label).is_some(), "the way back is named");
        assert_eq!(exec.count("update rollback"), 0);
        h.get_by_label(&label).click();
        let call = wait_for_call(&mut h, &exec, "update rollback");
        assert_eq!(call, ["update", "rollback", "--json"]);
        wait_for_label(&mut h, RESTART);
    }

    #[test]
    fn an_install_that_cannot_update_itself_says_why_and_offers_no_download() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("update-not-here", exec.clone());
        h.run_steps(4);
        assert!(h.query_by_label_contains("Nix updates it").is_some());
        for label in [DOWNLOAD, INSTALL, RESTART] {
            assert!(h.query_by_label(label).is_none(), "{label}");
        }
        assert_eq!(exec.count("update download"), 0);
    }

    #[test]
    fn no_component_version_is_shown_on_the_page() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("update-available", exec);
        h.run_steps(4);
        for tool in ["nvmod-rs", "loot-rs", "lodwright-rs", "ttwright-rs"] {
            assert!(
                h.query_by_label_contains(tool).is_none(),
                "{tool} is a Component version and is not shown"
            );
        }
        assert!(
            h.query_by_label_contains(&["known", "-good"].concat())
                .is_none(),
            "the release check's set is a build-time idea"
        );
    }

    /// Runs frames until `label` is on screen: a command's answer lands some frames after it was started.
    fn wait_for_label(h: &mut Harness<'static, App>, label: &str) {
        for _ in 0..200 {
            h.run_steps(2);
            if h.query_by_label(label).is_some() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn a_check_that_answers_says_nothing_went_wrong() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("active", exec.clone());
        h.run_steps(4);
        h.get_by_label(CHECK_UPDATES).click();
        let _ = wait_for_call(&mut h, &exec, "update check");
        wait_for_label(&mut h, DOWNLOAD);
        assert!(
            h.query_by_label(UPDATE_FAILED.what).is_none(),
            "the check answered, so there is no failure to show"
        );
        assert!(
            h.query_by_label(DOWNLOAD).is_some(),
            "the button becomes Download update once the release is known"
        );
    }

    #[test]
    fn a_failed_check_is_shown_where_it_happened() {
        let failing = Arc::new(Recording::failing());
        let mut h = harness("update-failed", failing.clone());
        h.run_steps(4);
        assert_eq!(
            failing.count("update check"),
            0,
            "nothing checks before a click"
        );
        assert!(
            h.query_by_label(UPDATE_FAILED.what).is_some(),
            "a failed check is shown where it happened"
        );
    }

    #[test]
    fn the_button_offers_the_check_again_after_a_failed_one() {
        let failing = Arc::new(Recording::failing());
        let mut h = harness("active", failing.clone());
        h.run_steps(4);
        h.get_by_label(CHECK_UPDATES).click();
        let _ = wait_for_call(&mut h, &failing, "update");
        wait_for_label(&mut h, UPDATE_FAILED.what);
        assert!(
            h.query_by_label(UPDATE_FAILED.what).is_some(),
            "the check failed where it was clicked"
        );
        assert!(
            h.query_by_label(CHECK_UPDATES).is_some(),
            "the button offers the check again"
        );
    }

    #[test]
    fn the_update_action_is_a_real_button_that_starts_the_check() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("active", exec.clone());
        h.run_steps(4);
        assert_eq!(
            exec.count("update check"),
            0,
            "nothing checks before a click"
        );
        let button = h.get_by_label(CHECK_UPDATES);
        assert_eq!(
            button.accesskit_node().role(),
            eframe::egui::accesskit::Role::Button,
            "the update action is a real button beside the version"
        );
        button.click();
        let call = wait_for_call(&mut h, &exec, "update check");
        assert_eq!(call, ["update", "check", "--json"], "the button checks");
    }

    #[test]
    fn nothing_on_the_page_installs_or_changes_a_component() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("active", exec.clone());
        h.run_steps(4);
        h.get_by_label(CHECK_UPDATES).click();
        wait_for_label(&mut h, DOWNLOAD);
        // Press every action the page offers, then see what ran.
        h.get_by_label(DOWNLOAD).click();
        h.run_steps(8);
        let calls = exec.calls.lock().unwrap();
        assert!(
            calls.iter().all(|c| c[0] == "version" || c[0] == "update"),
            "the page runs only the version read and the update steps, not {calls:?}"
        );
    }

    #[test]
    fn the_details_block_names_the_version() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("active", exec);
        h.run_steps(4);
        assert!(
            h.query_by_label(COPY_DETAILS).is_some(),
            "the details can be copied"
        );
        assert!(
            h.query_by_label_contains("Update check: not checked")
                .is_some()
        );
        assert!(
            h.query_by_label_contains("Toolbox version: 0.2.0")
                .is_some(),
            "the details carry the version the page reads"
        );
    }
}
