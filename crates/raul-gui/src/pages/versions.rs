// SPDX-License-Identifier: GPL-3.0-only
//! Versions: raul's own version with the update check, and the engine build (UI-RULES P3, P5, T4).
//!
//! The Toolbox section shows the one version a person sees, the Toolbox's own, as plain text, with the one update
//! action beside it as a real button: Check for updates, or Update once a newer release is known. It runs `raul
//! update check`, which reads the release list and downloads nothing. The engine section shows the active build, as
//! `raul verify` sees it against its install record, and installs the nv-rs release with `raul install-nv-rs
//! --release` after one confirm that states the cost. No Component version is shown, and the release check's set is
//! a build-time idea that no screen names. Nothing reaches the network before that click. Versions holds no path;
//! without a storage folder it points to Settings.

use super::home::{Mark, mark};
use super::settings::{
    Pending, Visits, WAITING, available, error_block, failure_of, read_config, settle,
    share_config, start, waiting,
};
use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use eframe::egui::{self, RichText};
use raul_ui_model::pages::settings::{Command, Config, Fetch, Key};
use raul_ui_model::pages::versions::{
    Engine, EngineStatus, Install, Release, Shipped, State, Update, UpdateAction, parse_update,
    parse_verify, parse_version, update_command, verify_command, version_command,
};
use raul_ui_model::pages::{Attention, PageId, Severity, View};
use raul_ui_service::Output;

crate::texts!(TEXTS {
    STATE_CHECKING = "Checking";
    STATE_ACTIVE = "nv-rs is installed";
    STATE_MISSING = "No engine installed";
    STATE_DIFFERS = "The engine needs a look";
    STATE_INSTALLING = "Installing nv-rs";
    STATE_UNKNOWN = "Engine state not known";
    CHECK_AGAIN = "Check again";
    SECTION_TOOLBOX = "Toolbox";
    SECTION_ENGINE = "Engine";
    ACTIVE_BUILD = "Active build";
    MARK_ACTIVE = "Active";
    MARK_MISSING = "Not set";
    MARK_DIFFERS = "Problem";
    MARK_INSTALLING = "Installing";
    MARK_UNKNOWN = "Warning";
    DETAIL_ACTIVE = "nv-rs, checked against its install record.";
    DETAIL_MISSING = "No engine in the Instance yet. Install puts the nv-rs release there.";
    INSTALL = "Install…";
    REINSTALL = "Install again…";
    INSTALLING = "Downloading and checking the release. This can take a few minutes.";
    INSTALLED_STATUS = "nv-rs installed and checked.";
    NO_STORAGE = "Set the storage folder in Settings first; the engine goes into the Instance there.";
    OPEN_SETTINGS = "Settings";
    INSTALL_TITLE = "Install the nv-rs release?";
    INSTALL_COST = "Downloads the latest nv-rs release, checks its checksum and runs its check. Your game folder is not changed; if anything fails, it is rolled back.";
    REINSTALL_COST = "Downloads the latest nv-rs release and puts it in place of the installed one after it passes its check. Your game folder is not changed; if anything fails, it is rolled back.";
    INSTALL_ACTION = "Install";
    INSTALL_SHAPE = "raul install-nv-rs --instance <dir> --release";
    TOOLBOX_LABEL = "Toolbox";
    CHECK_UPDATES = "Check for updates";
    UPDATE = "Update";
    UPDATE_READING = "Reading the version";
    UPDATE_CHECKING = "Checking for updates";
    UPDATE_NEWEST = "This is the newest release.";
    UPDATE_FOUND = "Version {version} is out; you run {current}.";
    UPDATE_OPEN = "Release page";
    READING = "Reading";
    WAITING_NOTE = "This build cannot read it from the window yet. The command below shows it in a terminal.";
});

crate::errors!(ERRORS {
    INSTALL_FAILED = (
        "nv-rs was not installed",
        "What it changed was rolled back. The line below says why."
    );
    ENGINE_UNREADABLE = (
        "The engine could not be checked",
        "Nothing changed. The line below says why."
    );
    UPDATE_FAILED = (
        "The update check could not finish",
        "Nothing changed. The line below says why."
    );
    VERSION_UNREADABLE = (
        "The version could not be read",
        "Nothing changed. The line below says why."
    );
});

const INSTALL_DIALOG: &str = "versions-install";
const UPDATE_DIALOG: &str = "versions-update";

/// The Versions page.
#[derive(Debug, Default)]
pub struct Versions {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Config,
    Verify,
    Version,
    Install,
    Update,
}

impl Versions {
    /// Versions in `state`, as a fixture shows it.
    pub fn with(state: State) -> Versions {
        Versions {
            state,
            ..Versions::default()
        }
    }

    fn load(&mut self, cx: &mut PageCx<'_>, first: bool) {
        self.state.engine = Fetch::Idle;
        match start(cx, &Command::config_list()) {
            Ok(id) => {
                self.pending.add(Job::Config, id);
                self.state.config = Fetch::Loading;
            }
            Err(e) => self.state.config = Fetch::Failed(e.what.to_owned()),
        }
        // The version never changes while the window is open: read it once, or again after a failure.
        if matches!(self.state.shipped, Fetch::Idle | Fetch::Failed(_)) {
            self.read_version(cx);
        }
        if first {
            settle(cx.jobs);
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

    /// Check for updates: `raul update check` reads the release list and downloads nothing (ADR-0005).
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

    fn load_engine(&mut self, cx: &mut PageCx<'_>, config: &Config) {
        let Some(instance) = config.get(Key::Instance) else {
            self.state.engine = Fetch::Ready(Engine::missing());
            return;
        };
        if !std::path::Path::new(instance).exists() {
            self.state.engine = Fetch::Ready(Engine::missing());
            return;
        }
        match start(cx, &verify_command(instance)) {
            Ok(id) => {
                self.pending.add(Job::Verify, id);
                self.state.engine = Fetch::Loading;
            }
            Err(e) => self.state.engine = Fetch::Failed(e.what.to_owned()),
        }
    }

    fn install(&mut self, cx: &mut PageCx<'_>) {
        let Some(command) = self.state.install_command() else {
            return;
        };
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(Job::Install, id);
                self.state.install = Install::Running;
            }
            Err(e) => self.state.install = Install::Failed(e.what.to_owned()),
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            match job {
                Job::Config => match read_config(&result) {
                    Ok(config) => {
                        self.load_engine(cx, &config);
                        share_config(cx, &config);
                        self.state.config = Fetch::Ready(config);
                    }
                    Err(why) => self.state.config = Fetch::Failed(why),
                },
                Job::Verify => {
                    self.state.engine = match &result {
                        Ok(out) => parse_verify(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
                Job::Version => {
                    self.state.shipped = match &result {
                        Ok(out) => parse_version(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
                Job::Update => self.take_update(&result),
                Job::Install => {
                    self.state.install = match &result {
                        Ok(out) if out.code == Some(0) => {
                            cx.shell.status = Some(INSTALLED_STATUS.to_owned());
                            let lifted = [
                                super::home::PLAY_NO_ENGINE.id,
                                super::home::PLAY_ENGINE_CHANGED.id,
                            ];
                            if cx
                                .shell
                                .play_blocked
                                .is_some_and(|b| lifted.contains(&b.id))
                            {
                                cx.shell.play_blocked = None;
                            }
                            Install::Done
                        }
                        Ok(out) => Install::Failed(failure_of(out)),
                        Err(why) => Install::Failed(why.clone()),
                    };
                    // Read the engine again, whatever the outcome.
                    if let Some(config) = self.state.config.ready().cloned() {
                        self.load_engine(cx, &config);
                    }
                }
            }
        }
        if let Some(engine) = self.state.engine.ready() {
            cx.shell.engine = (engine.status() == EngineStatus::Active)
                .then(|| super::home::FACT_ENGINE_ACTIVE.to_owned());
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
        if self.state.install == Install::Running {
            return STATE_INSTALLING.to_owned();
        }
        match &self.state.engine {
            Fetch::Idle | Fetch::Loading => STATE_CHECKING,
            Fetch::Failed(_) => STATE_UNKNOWN,
            Fetch::Ready(e) => match e.status() {
                EngineStatus::Active => STATE_ACTIVE,
                EngineStatus::Missing => STATE_MISSING,
                EngineStatus::Differs => STATE_DIFFERS,
            },
        }
        .to_owned()
    }

    fn attention(&self) -> Option<Attention> {
        self.state.needs_attention().then_some(Attention {
            severity: Severity::Problem,
            count: 1,
        })
    }

    fn tools(&mut self, ui: &mut egui::Ui, _cx: &mut PageCx<'_>) {
        let busy = self.state.config.is_pending()
            || self.state.engine.is_pending()
            || self.state.install == Install::Running
            || self.state.update == Update::Checking;
        if ui
            .add_enabled(!busy, egui::Button::new(CHECK_AGAIN))
            .clicked()
        {
            self.state.config = Fetch::Idle;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        if self.visits.returned(ui.ctx())
            && self.state.install != Install::Running
            && !self.state.config.is_pending()
            && !self.state.engine.is_pending()
        {
            self.state.config = Fetch::Idle;
        }
        if self.state.config == Fetch::Idle {
            let first = !self.visits.loaded;
            self.visits.loaded = true;
            self.load(cx, first);
        }
        self.take_answers(cx);
        if let Some(config) = self.state.config.ready().cloned() {
            share_config(cx, &config);
        }
        match cx.dialogs.take_answer(INSTALL_DIALOG) {
            Some(Answer::Confirmed) => self.install(cx),
            Some(Answer::Closed) => self.state.install = Install::Idle,
            None => {}
        }
        let _ = cx.dialogs.take_answer(UPDATE_DIALOG);
        let t = *cx.tokens;
        // The version is the only one a person sees. It is read when the page opens; a read that failed is tried again
        // when the page is shown again, not on every frame.
        if self.state.raul().is_none() && matches!(self.state.shipped, Fetch::Idle) {
            self.read_version(cx);
        }
        if self.state.update == Update::Failed(WAITING.what.to_owned()) {
            // The check cannot run from this window yet; it is not tried again on every frame.
            self.state.update = Update::Idle;
        }
        shell::section_label(ui, &t, SECTION_TOOLBOX, true);
        self.toolbox_section(ui, cx);
        ui.add_space(16.0);
        shell::section_label(ui, &t, SECTION_ENGINE, true);
        self.engine_section(ui, cx);
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
    fn toolbox_section(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
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
            if ui
                .add_enabled(can, button)
                .on_hover_text(update_command().shown())
                .clicked()
            {
                self.check_updates(cx);
            }
        });
        match &self.state.update {
            Update::Idle => {}
            Update::Checking => {
                ui.horizontal(|ui| {
                    super::home::icon(ui, &t, Mark::Info);
                    ui.label(RichText::new(UPDATE_CHECKING).color(t.muted.color()));
                });
            }
            Update::Checked { raul, newer } => match newer {
                Some(r) => {
                    let line = UPDATE_FOUND
                        .replace("{version}", &r.version)
                        .replace("{current}", raul);
                    ui.horizontal_wrapped(|ui| {
                        mark(ui, &t, Mark::Info, &line);
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
                    ui.label(RichText::new(UPDATE_NEWEST).color(t.muted.color()));
                }
            },
            Update::Failed(why) if why == WAITING.what => {
                ui.label(RichText::new(WAITING_NOTE).color(t.muted.color()));
            }
            Update::Failed(why) => error_block(ui, &t, &UPDATE_FAILED, Some(why)),
        }
        if version.is_none() {
            match &self.state.shipped {
                Fetch::Failed(why) if why == WAITING.what => {
                    ui.label(RichText::new(WAITING_NOTE).color(t.muted.color()));
                    shell::command_line(ui, &t, &version_command().shown());
                }
                Fetch::Failed(why) => {
                    error_block(ui, &t, &VERSION_UNREADABLE, Some(why));
                    shell::command_line(ui, &t, &version_command().shown());
                }
                Fetch::Idle | Fetch::Loading => {
                    waiting(ui, &t, UPDATE_READING, &version_command());
                }
                Fetch::Ready(_) => {}
            }
        }
        shell::command_line(ui, &t, &update_command().shown());
    }

    fn engine_section(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let running = self.state.install == Install::Running;
        ui.horizontal(|ui| {
            ui.label(RichText::new(ACTIVE_BUILD).color(t.muted.color()));
            match (&self.state.engine, running) {
                (_, true) => mark(ui, &t, Mark::Info, MARK_INSTALLING),
                (Fetch::Idle | Fetch::Loading, _) => mark(ui, &t, Mark::Info, READING),
                (Fetch::Failed(_), _) => mark(ui, &t, Mark::Warning, MARK_UNKNOWN),
                (Fetch::Ready(e), _) => match e.status() {
                    EngineStatus::Active => mark(ui, &t, Mark::Ok, MARK_ACTIVE),
                    EngineStatus::Missing => mark(ui, &t, Mark::Unset, MARK_MISSING),
                    EngineStatus::Differs => mark(ui, &t, Mark::Problem, MARK_DIFFERS),
                },
            }
        });
        match &self.state.engine {
            Fetch::Ready(e) => {
                let detail = match e.status() {
                    EngineStatus::Active => DETAIL_ACTIVE.to_owned(),
                    EngineStatus::Missing => DETAIL_MISSING.to_owned(),
                    EngineStatus::Differs => e.message.clone(),
                };
                ui.label(RichText::new(detail).color(t.muted.color()));
            }
            Fetch::Failed(why) if why == WAITING.what => {
                ui.label(RichText::new(WAITING_NOTE).color(t.muted.color()));
            }
            Fetch::Failed(why) => error_block(ui, &t, &ENGINE_UNREADABLE, Some(why)),
            Fetch::Idle | Fetch::Loading => {}
        }
        if let Some(instance) = self.state.instance() {
            shell::command_line(ui, &t, &verify_command(instance).shown());
        }
        ui.add_space(8.0);
        if self.state.config.ready().is_some() && self.state.instance().is_none() {
            ui.label(RichText::new(NO_STORAGE).color(t.muted.color()));
            if ui.button(OPEN_SETTINGS).clicked() {
                cx.shell.go_to = Some(View::Page(PageId::Settings));
            }
            shell::command_line(ui, &t, INSTALL_SHAPE);
            return;
        }
        if running {
            ui.horizontal(|ui| {
                super::home::icon(ui, &t, super::home::Mark::Info);
                ui.label(RichText::new(INSTALLING).color(t.muted.color()));
            });
        }
        if let Install::Failed(why) = &self.state.install {
            error_block(ui, &t, &INSTALL_FAILED, Some(why));
        }
        let installed = self
            .state
            .engine
            .ready()
            .is_some_and(|e| e.status() != EngineStatus::Missing);
        let label = if installed { REINSTALL } else { INSTALL };
        // One primary button per page: Check for updates is it while a release is out.
        let button = if installed || self.state.newer().is_some() {
            egui::Button::new(label)
        } else {
            shell::primary_button(&t, label)
        };
        let command = self.state.install_command();
        let can = self.state.can_install() && command.as_ref().is_some_and(available);
        if ui.add_enabled(can, button).clicked()
            && let Some(command) = &command
        {
            self.state.install = Install::Asking;
            cx.dialogs.open(
                Dialog::confirm(
                    INSTALL_DIALOG,
                    INSTALL_TITLE,
                    if installed {
                        REINSTALL_COST
                    } else {
                        INSTALL_COST
                    },
                    INSTALL_ACTION,
                )
                .with_command(command.shown()),
            );
        }
        shell::command_line(
            ui,
            &t,
            &command.map_or_else(|| INSTALL_SHAPE.to_owned(), |c| c.shown()),
        );
    }
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
#[allow(clippy::too_many_lines)]
pub fn fixtures() -> Vec<Fixture> {
    let config = Config::with(
        "/users/player/.config/raul/config.toml",
        &[
            (Key::NewVegas, "/games/steamapps/common/Fallout New Vegas"),
            (Key::StorageRoot, "/users/player/raul-storage"),
            (Key::Instance, "/users/player/raul-storage/instance"),
        ],
    );
    let shipped = Shipped {
        raul: "0.2.0".into(),
    };
    let newer = Release {
        version: "0.3.0".into(),
        url: "https://example.invalid/rauls-trusty-toolbox/releases/tag/v0.3.0".into(),
    };
    let ready = |engine: Engine, install: Install| {
        Versions::with(State {
            config: Fetch::Ready(config.clone()),
            engine: Fetch::Ready(engine),
            shipped: Fetch::Ready(shipped.clone()),
            install,
            ..State::default()
        })
    };
    let checked = |newer: Option<Release>| {
        Versions::with(State {
            config: Fetch::Ready(config.clone()),
            engine: Fetch::Ready(Engine::active()),
            shipped: Fetch::Ready(shipped.clone()),
            install: Install::Idle,
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
                config: Fetch::Loading,
                engine: Fetch::Loading,
                shipped: Fetch::Loading,
                install: Install::Idle,
                ..State::default()
            }),
        ),
        Fixture::new("not-installed", ready(Engine::missing(), Install::Idle)),
        Fixture::new("active", ready(Engine::active(), Install::Idle)),
        Fixture::new(
            "differs",
            ready(
                Engine {
                    installed: true,
                    verified: false,
                    message: "nv-rs differs from the checksum recorded when it was installed".into(),
                },
                Install::Idle,
            ),
        ),
        Fixture::new("installing", ready(Engine::missing(), Install::Running)),
        Fixture::new(
            "install-failed",
            ready(
                Engine::missing(),
                Install::Failed(
                    "nvmod install-nv-rs: verification failed: the asset's SHA-256 does not match its release manifest"
                        .into(),
                ),
            ),
        ),
        Fixture::new(
            "no-storage",
            Versions::with(State {
                config: Fetch::Ready(Config::with(
                    "/users/player/.config/raul/config.toml",
                    &[(Key::NewVegas, "/games/steamapps/common/Fallout New Vegas")],
                )),
                engine: Fetch::Ready(Engine::missing()),
                shipped: Fetch::Ready(shipped.clone()),
                install: Install::Idle,
                ..State::default()
            }),
        ),
        Fixture::new(
            "waiting",
            Versions::with(State {
                config: Fetch::Ready(config.clone()),
                engine: Fetch::Failed(WAITING.what.to_owned()),
                shipped: Fetch::Failed(WAITING.what.to_owned()),
                install: Install::Idle,
                ..State::default()
            }),
        ),
        Fixture::new(
            "error",
            Versions::with(State {
                config: Fetch::Ready(config.clone()),
                engine: Fetch::Failed("raul verify: refused: the Instance has no profile".into()),
                shipped: Fetch::Ready(shipped.clone()),
                install: Install::Idle,
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
                config: Fetch::Ready(config.clone()),
                engine: Fetch::Ready(Engine::active()),
                shipped: Fetch::Ready(shipped.clone()),
                install: Install::Idle,
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
        h.run_steps(2);
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
            h.query_by_label_contains("known").is_none(),
            "the release check's set is a build-time idea"
        );
    }

    #[test]
    fn a_check_that_answers_says_nothing_went_wrong() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("active", exec.clone());
        h.run_steps(4);
        h.get_by_label(CHECK_UPDATES).click();
        let _ = wait_for_call(&mut h, &exec, "update");
        h.run_steps(4);
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
        h.run_steps(4);
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
    fn install_asks_once_then_runs_the_release_install() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("not-installed", exec.clone());
        h.run();
        h.get_by_label(INSTALL).click();
        h.run();
        assert_eq!(
            h.state()
                .dialogs()
                .current()
                .map(|d| d.title.clone())
                .as_deref(),
            Some(INSTALL_TITLE),
            "nothing reaches the network before the confirm"
        );
        assert!(
            exec.calls
                .lock()
                .unwrap()
                .iter()
                .all(|c| c[0] != "install-nv-rs"),
            "the install waits for the confirm"
        );
        h.get_by_label(INSTALL_ACTION).click();
        let call = wait_for_call(&mut h, &exec, "install-nv-rs");
        assert_eq!(
            call,
            [
                "install-nv-rs",
                "--instance",
                "/users/player/raul-storage/instance",
                "--release"
            ]
        );
    }
}
