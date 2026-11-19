// SPDX-License-Identifier: GPL-3.0-only
//! About and Update (the Versions page): raul's own version, the update check and a block of details to copy
//! (UI-RULES P3, P5, T4).
//!
//! It shows the one version a person sees, the Toolbox's own, as plain text, with the one update action beside it as
//! a real button: Check for updates, or Update once a newer release is known. It runs `raul update check`, which
//! reads the release list and downloads nothing. Every packaged Component comes with raul and changes only when raul
//! does, so no Component version and no per-Component action is shown. Nothing reaches the network before that
//! click.

use super::home::{Mark, mark};
use super::settings::{Pending, WAITING, error_block, failure_of, start, waiting};
use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::pages::settings::{Command, Fetch};
use raul_ui_model::pages::versions::{
    Release, Shipped, State, Update, UpdateAction, parse_update, parse_version, update_command,
    version_command,
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
    UPDATE_NOTE = "The check is yours to run: nothing looks for updates until you press the button. An update replaces raul and everything that comes with it.";
    COPY_DETAILS = "Copy details";
    WAITING_NOTE = "This build cannot read it from the window yet.";
});

crate::errors!(ERRORS {
    UPDATE_FAILED = (
        "The update check could not finish",
        "Nothing changed. The line below says why."
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
        let can = self.state.can_check_updates();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(TOOLBOX_LABEL).color(t.muted.color()));
            if let Some(v) = &version {
                ui.label(RichText::new(v).monospace().color(t.text.color()));
            }
            let label = match action {
                UpdateAction::Check => CHECK_UPDATES,
                UpdateAction::Update => UPDATE,
            };
            let button = if action == UpdateAction::Update {
                shell::primary_button(&t, label)
            } else {
                egui::Button::new(label)
            };
            if ui.add_enabled(can, button).clicked() {
                self.check_updates(cx);
            }
        });
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
                .filter(|c| c[0] == command)
                .count()
        }
    }

    impl Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            if self.fail_update && args.first().map(String::as_str) == Some("update") {
                return Ok(Output {
                    code: Some(1),
                    stdout: String::new(),
                    stderr: "raul update check: the release list could not be read".to_owned(),
                });
            }
            let stdout = match args.first().map(String::as_str) {
                Some("version") => r#"{"raul":"0.2.0"}"#.to_owned(),
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
                .rfind(|c| c[0] == command)
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
        assert_eq!(exec.count("update"), 0, "nothing checks before a click");
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
        let call = wait_for_call(&mut h, &exec, "update");
        assert_eq!(call, ["update", "check", "--json"], "the button checks");
        wait_for_label(&mut h, UPDATE);
        assert!(
            h.query_by_label(UPDATE).is_some(),
            "a newer release turns the action into Update"
        );
        assert!(
            h.query_by_label_contains("0.3.0").is_some(),
            "the release that is out is named"
        );
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
            h.query_by_label_contains(&["known", "-good"].concat()).is_none(),
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
        let _ = wait_for_call(&mut h, &exec, "update");
        wait_for_label(&mut h, UPDATE);
        assert!(
            h.query_by_label(UPDATE_FAILED.what).is_none(),
            "the check answered, so there is no failure to show"
        );
        assert!(
            h.query_by_label(UPDATE).is_some(),
            "the button becomes Update once the release is known"
        );
    }

    #[test]
    fn a_failed_check_is_shown_where_it_happened() {
        let failing = Arc::new(Recording::failing());
        let mut h = harness("update-failed", failing.clone());
        h.run_steps(4);
        assert_eq!(failing.count("update"), 0, "nothing checks before a click");
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
        assert_eq!(exec.count("update"), 0, "nothing checks before a click");
        let button = h.get_by_label(CHECK_UPDATES);
        assert_eq!(
            button.accesskit_node().role(),
            eframe::egui::accesskit::Role::Button,
            "the update action is a real button beside the version"
        );
        button.click();
        let call = wait_for_call(&mut h, &exec, "update");
        assert_eq!(call, ["update", "check", "--json"], "the button checks");
    }

    #[test]
    fn nothing_on_the_page_installs_or_changes_a_component() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("update-available", exec.clone());
        h.run_steps(4);
        for gone in ["Install\u{2026}", "Install again\u{2026}", "Engine"] {
            assert!(
                h.query_by_label(gone).is_none(),
                "{gone} is a per-Component action and is not offered"
            );
        }
        assert!(
            exec.calls
                .lock()
                .unwrap()
                .iter()
                .all(|c| c[0] != "install-nv-rs"),
            "the page never starts an install"
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
        assert!(h.query_by_label_contains("update check: not checked").is_some());
    }
}
