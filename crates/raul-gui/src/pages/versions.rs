// SPDX-License-Identifier: GPL-3.0-only
//! Versions: the engine build and the Components (UI-RULES P3, P5, T4).
//!
//! The engine section shows the active build, as `raul verify` sees it against its install record, and installs
//! the nv-rs release with `raul install-nv-rs --release` after one confirm that states the cost. Nothing reaches the
//! network before that click. The Components section lists the Known-good set this `raul` ships. Versions holds no
//! path; without a storage folder it points to Settings.

use super::home::{Mark, mark};
use super::settings::{
    Pending, Visits, WAITING, available, error_block, failure_of, read_config, settle,
    share_config, start, waiting,
};
use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::pages::settings::{Command, Config, Fetch, Key};
use raul_ui_model::pages::versions::{
    Engine, EngineStatus, Install, Shipped, State, parse_verify, parse_version, verify_command,
    version_command,
};
use raul_ui_model::pages::{Attention, PageId, Severity, View};

crate::texts!(TEXTS {
    STATE_CHECKING = "Checking";
    STATE_ACTIVE = "nv-rs is installed";
    STATE_MISSING = "No engine installed";
    STATE_DIFFERS = "The engine needs a look";
    STATE_INSTALLING = "Installing nv-rs";
    STATE_UNKNOWN = "Engine state not known";
    CHECK_AGAIN = "Check again";
    SECTION_ENGINE = "Engine";
    SECTION_COMPONENTS = "Components";
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
    COMPONENT = "Component";
    VERSION = "Version";
    TOOLBOX = "Toolbox";
    KNOWN_GOOD = "Known-good set";
    COMPONENTS_NOTE = "The versions this Toolbox was built and tested with.";
    READING = "Reading";
    WAITING_NOTE = "This build cannot read it from the window yet. The command below shows it in a terminal.";
    RETRY = "Try again";
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
    COMPONENTS_UNREADABLE = (
        "The Components could not be listed",
        "Nothing changed. The line below says why."
    );
});

const INSTALL_DIALOG: &str = "versions-install";

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
        // The shipped set never changes while the window is open: read it once, or again after a failure.
        if matches!(self.state.shipped, Fetch::Idle | Fetch::Failed(_)) {
            match start(cx, &version_command()) {
                Ok(id) => {
                    self.pending.add(Job::Version, id);
                    self.state.shipped = Fetch::Loading;
                }
                Err(e) => self.state.shipped = Fetch::Failed(e.what.to_owned()),
            }
        }
        if first {
            settle(cx.jobs);
        }
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
            || self.state.install == Install::Running;
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
        let t = *cx.tokens;
        shell::section_label(ui, &t, SECTION_ENGINE, true);
        self.engine_section(ui, cx);
        ui.add_space(16.0);
        shell::section_label(ui, &t, SECTION_COMPONENTS, true);
        components(ui, &t, &self.state.shipped);
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl Versions {
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
        let button = if installed {
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

fn components(ui: &mut egui::Ui, t: &Tokens, shipped: &Fetch<Shipped>) {
    match shipped {
        Fetch::Idle | Fetch::Loading => waiting(ui, t, READING, &version_command()),
        Fetch::Failed(why) if why == WAITING.what => {
            ui.label(RichText::new(WAITING_NOTE).color(t.muted.color()));
            shell::command_line(ui, t, &version_command().shown());
        }
        Fetch::Failed(why) => {
            error_block(ui, t, &COMPONENTS_UNREADABLE, Some(why));
            shell::command_line(ui, t, &version_command().shown());
        }
        Fetch::Ready(s) => {
            ui.label(RichText::new(COMPONENTS_NOTE).color(t.muted.color()));
            egui::Grid::new("versions-components")
                .num_columns(2)
                .spacing([32.0, 8.0])
                .striped(false)
                .show(ui, |ui| {
                    ui.label(RichText::new(COMPONENT).color(t.muted.color()));
                    ui.label(RichText::new(VERSION).color(t.muted.color()));
                    ui.end_row();
                    ui.label(TOOLBOX);
                    ui.label(RichText::new(&s.raul).monospace());
                    ui.end_row();
                    ui.label(KNOWN_GOOD);
                    ui.label(RichText::new(&s.set).monospace());
                    ui.end_row();
                    for (name, version) in &s.components {
                        ui.label(RichText::new(name).monospace());
                        ui.label(RichText::new(version).monospace());
                        ui.end_row();
                    }
                });
            shell::command_line(ui, t, &version_command().shown());
        }
    }
}

/// The states the snapshot tests render.
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
        set: "2026-11".into(),
        components: [
            ("nv-rs", "0.5.0"),
            ("nvmod-rs", "0.6.0"),
            ("ttwright-rs", "0.3.0"),
            ("lodwright-rs", "0.3.0"),
            ("loot-rs", "0.4.1"),
        ]
        .into_iter()
        .map(|(c, v)| (c.to_owned(), v.to_owned()))
        .collect(),
    };
    let ready = |engine: Engine, install: Install| {
        Versions::with(State {
            config: Fetch::Ready(config.clone()),
            engine: Fetch::Ready(engine),
            shipped: Fetch::Ready(shipped.clone()),
            install,
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
            }),
        ),
        Fixture::new(
            "waiting",
            Versions::with(State {
                config: Fetch::Ready(config.clone()),
                engine: Fetch::Failed(WAITING.what.to_owned()),
                shipped: Fetch::Failed(WAITING.what.to_owned()),
                install: Install::Idle,
            }),
        ),
        Fixture::new(
            "error",
            Versions::with(State {
                config: Fetch::Ready(config.clone()),
                engine: Fetch::Failed("raul verify: refused: the Instance has no profile".into()),
                shipped: Fetch::Ready(shipped),
                install: Install::Idle,
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
    use egui_kittest::kittest::Queryable;
    use raul_ui_service::{Exec, Output};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Recording {
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            Ok(Output {
                code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn install_asks_once_then_runs_the_release_install() {
        let exec = Arc::new(Recording::default());
        let fixture = fixtures()
            .into_iter()
            .find(|f| f.name == "not-installed")
            .unwrap();
        let app = App::fixture(fixture, Jobs::new(exec.clone()));
        let mut h = Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            });
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
        assert!(exec.calls.lock().unwrap().is_empty());
        h.get_by_label(INSTALL_ACTION).click();
        for _ in 0..200 {
            h.run_steps(2);
            if !exec.calls.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            exec.calls.lock().unwrap()[0],
            [
                "install-nv-rs",
                "--instance",
                "/users/player/raul-storage/instance",
                "--release"
            ]
        );
    }
}
