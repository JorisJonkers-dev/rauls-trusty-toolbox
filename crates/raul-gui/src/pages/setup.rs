// SPDX-License-Identifier: GPL-3.0-only
//! The setup wizard on the shell (UI-RULES P2, G3, G5): a step rail, the step's page with one Raul line, and a
//! footer with Back and Next. Every action shows its `raul` command; nothing changes in a game folder.

use super::home::{Mark, icon, mark};
use super::settings::{
    LABEL_WIDTH, Pending, Visits, WAITING, available, config_answer, error_block, failure_of,
    path_field, read_config, settle, share_config, start, waiting,
};
use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText, Stroke};
use raul_ui_model::pages::settings::{Answer, Command, Config, Fetch, Field, Key, Save};
use raul_ui_model::pages::setup::{
    Checks, EngineStep, Finding, Missing, State, Step, detect_command, doctor_command,
    init_command, parse_detect, parse_doctor, suggest_storage,
};
use raul_ui_model::pages::versions::install_command;
use raul_ui_model::pages::{Flow, PageId, View};

crate::texts!(TEXTS {
    STATE_STEP = "Step {n} of 5";
    SKIP = "Skip setup";
    BACK = "Back";
    NEXT = "Next";
    FINISH = "Open the Toolbox";
    RAIL_HINT = "Nothing in your game folder changes during setup.";
    RAIL_NEW_VEGAS = "New Vegas";
    RAIL_NEW_VEGAS_SUB = "Find your game";
    RAIL_TTW = "TTW";
    RAIL_TTW_SUB = "Optional";
    RAIL_ENGINE = "Engine";
    RAIL_ENGINE_SUB = "Install nv-rs";
    RAIL_CHECK = "Check";
    RAIL_CHECK_SUB = "Quick health check";
    RAIL_READY = "Ready";
    RAIL_READY_SUB = "What was set up";
    DONE = "Done";
    TITLE_NEW_VEGAS = "Where's New Vegas?";
    SUB_NEW_VEGAS = "Point the Toolbox at your game folder, and pick where it keeps its own files.";
    RAUL_NEW_VEGAS = "Let's find your game. I won't touch anything yet.";
    TITLE_TTW = "Tale of Two Wastelands";
    SUB_TTW = "Optional. Plays both worlds as one game. You can set it up later from Home.";
    RAUL_TTW = "Both worlds as one? Needs Fallout 3 too. Skip it if unsure.";
    TITLE_ENGINE = "Engine";
    SUB_ENGINE = "nv-rs runs the game natively. Your game folder stays as it is.";
    RAUL_ENGINE = "This is the bit that makes it run on Linux. No Proton.";
    TITLE_CHECK = "Quick check";
    SUB_CHECK = "Read-only checks of the game and the Instance.";
    RAUL_CHECK = "Kicking the tyres.";
    TITLE_READY = "Ready";
    SUB_READY = "Setup is done. Home shows anything that still needs a look, one click from its fix.";
    RAUL_READY = "Set up. Have fun out there.";
    RAUL_READY_LOOK = "Set up. A couple of things to look at later.";
    LABEL_GAME = "Game folder";
    HINT_GAME = "Holds the game's executable";
    LABEL_STORAGE = "Storage folder";
    HINT_STORAGE = "Where the Toolbox keeps its files";
    LABEL_FALLOUT3 = "Fallout 3 folder";
    HINT_FALLOUT3 = "Needed for TTW";
    FIND = "Find";
    FIND_NOTE = "Find looks in the usual Steam, GOG and Epic libraries. You can also type the folder.";
    FIND_NONE = "No copy found in the usual libraries. Type the folder instead.";
    FIND_WAITING = "Find is not in this build yet. Type the folder instead.";
    GAME_FOUND = "Game found. Nothing in it changed.";
    TTW_NO = "Not now";
    TTW_NO_SUB = "Plain New Vegas. Add TTW any time.";
    TTW_YES = "I want TTW";
    TTW_YES_SUB = "Needs your Fallout 3 install. The install runs later and takes a while.";
    TTW_LATER = "You can also set this later in Settings.";
    INSTALL = "Install";
    INSTALL_AGAIN = "Try again";
    INITIALISING = "Making the Instance in the storage folder";
    INSTALLING = "Downloading and checking the nv-rs release. This can take a few minutes.";
    INSTALLED = "Installed and checked.";
    ENGINE_NOTE = "You can install it again later on Versions.";
    MARK_INSTALLED = "Installed";
    MARK_WORKING = "Working";
    CHECKING = "Running the checks";
    NOTHING_FOUND = "Nothing to fix.";
    MORE_FINDINGS = "Doctor lists the rest.";
    MARK_OK = "OK";
    MARK_WARNING = "Warning";
    MARK_PROBLEM = "Problem";
    MARK_INFO = "Note";
    FACT_GAME = "Game";
    FACT_STORAGE = "Storage";
    FACT_TTW = "TTW";
    FACT_ENGINE = "Engine";
    FACT_CHECKS = "Checks";
    TTW_WANTED = "Wanted. Set it up from Home or now.";
    TTW_NOT_NOW = "Not now";
    ENGINE_INSTALLED = "nv-rs, installed and checked";
    CHECKS_CLEAN = "Nothing to look at";
    CHECKS_SOME = "{n} to look at";
    CHECKS_UNKNOWN = "Not run";
    SET_UP_TTW = "Set up TTW…";
    MISSING_GAME = "Next needs the game folder.";
    MISSING_STORAGE = "Next needs a storage folder.";
    MISSING_SAVING = "Saving the folders.";
    MISSING_ENGINE = "Next needs the engine installed.";
    MISSING_CHECKS = "The checks are still running.";
    MISSING_REFUSED = "Fix the folder above, then press Next.";
    READING = "Reading the settings";
});

crate::errors!(ERRORS {
    INSTALL_FAILED = (
        "The engine was not installed",
        "Your game folder did not change. The line below says why."
    );
    CHECKS_FAILED = (
        "The checks could not run",
        "Nothing changed. You can run them later from Home."
    );
    UNREADABLE = (
        "The settings could not be read",
        "Nothing changed. The line below says why."
    );
});

/// The setup wizard.
#[derive(Debug, Default)]
pub struct Setup {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Config,
    Save(Key),
    Find(Key),
    Init,
    Install,
    Doctor,
}

impl Setup {
    /// The wizard in `state`, as a fixture shows it.
    pub fn with(state: State) -> Setup {
        Setup {
            state,
            ..Setup::default()
        }
    }

    fn load(&mut self, cx: &mut PageCx<'_>, first: bool) {
        match start(cx, &Command::config_list()) {
            Ok(id) => {
                self.pending.add(Job::Config, id);
                self.state.config = Fetch::Loading;
                if first {
                    settle(cx.jobs);
                }
            }
            Err(e) => self.state.config = Fetch::Failed(e.what.to_owned()),
        }
    }

    fn save(&mut self, cx: &mut PageCx<'_>, key: Key) {
        let Some(field) = self.state.field_mut(key) else {
            return;
        };
        match start(cx, &field.command()) {
            Ok(id) => {
                field.save = Save::Saving;
                self.pending.add(Job::Save(key), id);
            }
            Err(e) => {
                field.save = Save::Failed(e.what.to_owned());
                self.state.advancing = false;
            }
        }
    }

    fn find(&mut self, cx: &mut PageCx<'_>, key: Key) {
        match start(cx, &detect_command()) {
            Ok(id) => self.pending.add(Job::Find(key), id),
            Err(e) => self.state.find_failed = Some(e.what.to_owned()),
        }
    }

    /// Install: `raul init` first when there is no Instance yet, then `raul install-nv-rs`. `made` says the init
    /// of this install already succeeded, so it is not run again whatever the folder looks like.
    fn install(&mut self, cx: &mut PageCx<'_>, made: bool) {
        let (Some(instance), Some(game)) = (self.state.instance(), self.state.game.saved.clone())
        else {
            return;
        };
        let has_instance = made || std::path::Path::new(&instance).exists();
        let (job, command, step) = if has_instance {
            (
                Job::Install,
                install_command(&instance),
                EngineStep::Installing,
            )
        } else {
            (
                Job::Init,
                init_command(&game, &instance),
                EngineStep::Initialising,
            )
        };
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(job, id);
                self.state.engine = step;
            }
            Err(e) => self.state.engine = EngineStep::Failed(e.what.to_owned()),
        }
    }

    fn run_checks(&mut self, cx: &mut PageCx<'_>) {
        let Some(instance) = self.state.instance() else {
            self.state.checks = Fetch::Failed(MISSING_STORAGE.to_owned());
            return;
        };
        match start(cx, &doctor_command(&instance)) {
            Ok(id) => {
                self.pending.add(Job::Doctor, id);
                self.state.checks = Fetch::Loading;
            }
            Err(e) => self.state.checks = Fetch::Failed(e.what.to_owned()),
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            match job {
                Job::Config => match read_config(&result) {
                    Ok(config) => {
                        let env = |k: &str| std::env::var(k).ok();
                        let suggestion = suggest_storage(
                            env("XDG_DATA_HOME").as_deref(),
                            env("HOME").as_deref(),
                            env("LOCALAPPDATA").as_deref(),
                        );
                        share_config(cx, &config);
                        self.state = State::opened(config, suggestion.as_deref());
                    }
                    Err(why) => self.state.config = Fetch::Failed(why),
                },
                Job::Save(key) => {
                    let answer = config_answer(&result);
                    if let Some(f) = self.state.field_mut(key) {
                        match &answer {
                            Ok(a) => f.saved_as(a),
                            Err(why) => f.save = Save::Failed(why.clone()),
                        }
                    }
                    if !matches!(answer, Ok(Answer::Entries(_))) {
                        self.state.advancing = false;
                    }
                    if key == Key::NewVegas
                        && let Ok(Answer::Entries(c)) = &answer
                    {
                        cx.shell.new_vegas = c.get(Key::NewVegas).map(str::to_owned);
                    }
                }
                Job::Find(key) => {
                    let game = if key == Key::Fallout3 {
                        "fallout3"
                    } else {
                        "new-vegas"
                    };
                    match &result {
                        Ok(out) => match parse_detect(&out.stdout, game) {
                            Ok(found) => {
                                if let (Some(path), Some(f)) = (&found, self.state.field_mut(key)) {
                                    f.draft.clone_from(path);
                                    f.save = Save::Idle;
                                }
                                self.state.found = Some(found);
                                if let Some(f) = self.state.field(key)
                                    && f.changed()
                                {
                                    self.save(cx, key);
                                }
                            }
                            Err(_) => self.state.find_failed = Some(failure_of(out)),
                        },
                        Err(why) => self.state.find_failed = Some(why.clone()),
                    }
                }
                Job::Init => match &result {
                    Ok(out) if out.code == Some(0) => self.install(cx, true),
                    Ok(out) => self.state.engine = EngineStep::Failed(failure_of(out)),
                    Err(why) => self.state.engine = EngineStep::Failed(why.clone()),
                },
                Job::Install => {
                    self.state.engine = match &result {
                        Ok(out) if out.code == Some(0) => {
                            cx.shell.engine = Some(super::home::FACT_ENGINE_ACTIVE.to_owned());
                            EngineStep::Installed
                        }
                        Ok(out) => EngineStep::Failed(failure_of(out)),
                        Err(why) => EngineStep::Failed(why.clone()),
                    };
                }
                Job::Doctor => {
                    self.state.checks = match &result {
                        Ok(out) => parse_doctor(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
            }
        }
    }

    /// Next: saves what the step holds, one path at a time, then moves on.
    fn drive_next(&mut self, cx: &mut PageCx<'_>) {
        if !self.state.advancing || self.state.saving() {
            return;
        }
        if let Some(key) = self.state.next_save() {
            self.save(cx, key);
            return;
        }
        if !self.state.advance() {
            self.state.advancing = false;
        }
        self.entered_step(cx);
    }

    /// What a step does when it is shown: the engine step notices an engine already in place, and the check step
    /// runs the checks.
    fn entered_step(&mut self, cx: &mut PageCx<'_>) {
        match self.state.step {
            Step::Engine if self.state.engine == EngineStep::Idle => {
                let record = self
                    .state
                    .instance()
                    .map(|i| std::path::Path::new(&i).join("nv-rs").join("install.json"));
                if record.is_some_and(|r| r.exists()) {
                    self.state.engine = EngineStep::Installed;
                }
            }
            Step::Check if self.state.checks == Fetch::Idle => self.run_checks(cx),
            _ => {}
        }
    }
}

impl Page for Setup {
    fn view(&self) -> View {
        View::Flow(Flow::Setup)
    }

    fn title(&self) -> &'static str {
        msg::FLOW_SETUP
    }

    fn state_line(&self) -> String {
        STATE_STEP.replace("{n}", &self.state.step.number().to_string())
    }

    fn tools(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let skip = egui::Button::new(RichText::new(SKIP).color(t.accent.color())).frame(false);
        if ui.add(skip).clicked() {
            cx.shell.go_to = Some(View::Page(PageId::Home));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        if self.visits.returned(ui.ctx()) && self.pending_is_empty() {
            // Opened again (Run setup again): start over from what is saved.
            self.state = State::default();
        }
        if self.state.config == Fetch::Idle {
            let first = !self.visits.loaded;
            self.visits.loaded = true;
            self.load(cx, first);
        }
        self.take_answers(cx);
        self.drive_next(cx);
        let t = *cx.tokens;
        match &self.state.config {
            Fetch::Idle | Fetch::Loading => {
                waiting(ui, &t, READING, &Command::config_list());
                return;
            }
            Fetch::Failed(why) => {
                error_block(ui, &t, &UNREADABLE, Some(why));
                shell::command_line(ui, &t, &Command::config_list().shown());
                return;
            }
            Fetch::Ready(_) => {}
        }
        let wide = ui.available_width() >= 760.0;
        if wide {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(200.0, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(200.0);
                        rail(ui, &t, self.state.step);
                    },
                );
                ui.add(egui::Separator::default().vertical());
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.set_max_width(760.0);
                    self.step_page(ui, cx);
                });
            });
        } else {
            self.step_page(ui, cx);
        }
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl Setup {
    fn pending_is_empty(&mut self) -> bool {
        !self.state.saving()
            && !matches!(
                self.state.engine,
                EngineStep::Initialising | EngineStep::Installing
            )
            && self.state.checks != Fetch::Loading
    }

    fn step_page(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let (title, sub, line) = match self.state.step {
            Step::NewVegas => (TITLE_NEW_VEGAS, SUB_NEW_VEGAS, RAUL_NEW_VEGAS),
            Step::Ttw => (TITLE_TTW, SUB_TTW, RAUL_TTW),
            Step::Engine => (TITLE_ENGINE, SUB_ENGINE, RAUL_ENGINE),
            Step::Check => (TITLE_CHECK, SUB_CHECK, RAUL_CHECK),
            Step::Ready if self.state.to_look_at() > 0 || self.state.checks.ready().is_none() => {
                (TITLE_READY, SUB_READY, RAUL_READY_LOOK)
            }
            Step::Ready => (TITLE_READY, SUB_READY, RAUL_READY),
        };
        ui.label(
            RichText::new(title)
                .text_style(theme::title_style())
                .size(theme::BASE * 1.25),
        );
        ui.label(RichText::new(sub).color(t.muted.color()));
        ui.add_space(4.0);
        shell::raul_line(ui, &t, line);
        ui.add_space(8.0);
        shell::hairline(ui, &t);
        ui.add_space(12.0);
        match self.state.step {
            Step::NewVegas => self.new_vegas(ui, cx),
            Step::Ttw => self.ttw(ui, cx),
            Step::Engine => self.engine(ui, cx),
            Step::Check => checks(ui, &t, &self.state),
            Step::Ready => self.ready(ui, cx),
        }
        ui.add_space(16.0);
        shell::hairline(ui, &t);
        ui.add_space(10.0);
        self.footer(ui, cx);
    }

    fn find_row(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, key: Key) {
        let t = *cx.tokens;
        ui.horizontal(|ui| {
            ui.add_space(LABEL_WIDTH + ui.spacing().item_spacing.x);
            let can = available(&detect_command());
            if ui.add_enabled(can, egui::Button::new(FIND)).clicked() {
                self.find(cx, key);
            }
            let note = match (&self.state.found, &self.state.find_failed) {
                _ if !can => FIND_WAITING,
                (Some(None), _) => FIND_NONE,
                _ => FIND_NOTE,
            };
            ui.label(RichText::new(note).color(t.muted.color()));
        });
        if let Some(why) = &self.state.find_failed
            && why != WAITING.what
        {
            ui.label(RichText::new(why).monospace().color(t.muted.color()));
        }
        ui.horizontal(|ui| {
            ui.add_space(LABEL_WIDTH + ui.spacing().item_spacing.x);
            ui.label(
                RichText::new(detect_command().shown())
                    .monospace()
                    .size(theme::BASE * 0.84)
                    .color(t.muted.color()),
            );
        });
        ui.add_space(8.0);
    }

    fn field(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, key: Key) {
        let t = *cx.tokens;
        let (label, hint) = match key {
            Key::NewVegas => (LABEL_GAME, HINT_GAME),
            Key::Fallout3 => (LABEL_FALLOUT3, HINT_FALLOUT3),
            _ => (LABEL_STORAGE, HINT_STORAGE),
        };
        let busy = self.state.saving();
        let left = self
            .state
            .field_mut(key)
            .is_some_and(|f| path_field(ui, &t, f, label, hint, key != Key::Fallout3));
        if left && !busy {
            self.save(cx, key);
        }
    }

    fn new_vegas(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        self.field(ui, cx, Key::NewVegas);
        self.find_row(ui, cx, Key::NewVegas);
        if self.state.game.saved.is_some() && !self.state.game.changed() {
            ui.horizontal(|ui| {
                ui.add_space(LABEL_WIDTH + ui.spacing().item_spacing.x);
                mark(ui, &t, Mark::Ok, GAME_FOUND);
            });
            ui.add_space(8.0);
        }
        self.field(ui, cx, Key::StorageRoot);
    }

    fn ttw(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        ui.horizontal_wrapped(|ui| {
            for (want, title, sub) in [(false, TTW_NO, TTW_NO_SUB), (true, TTW_YES, TTW_YES_SUB)] {
                if option_card(ui, &t, self.state.want_ttw == want, title, sub) {
                    self.state.want_ttw = want;
                }
            }
        });
        ui.add_space(12.0);
        if self.state.want_ttw {
            self.field(ui, cx, Key::Fallout3);
            self.find_row(ui, cx, Key::Fallout3);
            ui.label(RichText::new(TTW_LATER).color(t.muted.color()));
        }
    }

    fn engine(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let instance = self.state.instance().unwrap_or_default();
        let game = self.state.game.saved.clone().unwrap_or_default();
        let has_instance = std::path::Path::new(&instance).exists();
        let install = install_command(&instance);
        let init = init_command(&game, &instance);
        match &self.state.engine {
            EngineStep::Idle | EngineStep::Failed(_) => {
                let label = if matches!(self.state.engine, EngineStep::Failed(_)) {
                    INSTALL_AGAIN
                } else {
                    INSTALL
                };
                let can = available(&install) && (has_instance || available(&init));
                if ui
                    .add_enabled(can, shell::primary_button(&t, label))
                    .clicked()
                {
                    self.install(cx, false);
                }
            }
            EngineStep::Initialising => {
                mark(ui, &t, Mark::Info, MARK_WORKING);
                ui.label(RichText::new(INITIALISING).color(t.muted.color()));
            }
            EngineStep::Installing => {
                mark(ui, &t, Mark::Info, MARK_WORKING);
                ui.label(RichText::new(INSTALLING).color(t.muted.color()));
            }
            EngineStep::Installed => {
                mark(ui, &t, Mark::Ok, MARK_INSTALLED);
                ui.label(RichText::new(INSTALLED).color(t.muted.color()));
            }
        }
        if let EngineStep::Failed(why) = &self.state.engine {
            error_block(ui, &t, &INSTALL_FAILED, Some(why));
        }
        if !has_instance && self.state.engine != EngineStep::Installed {
            shell::command_line(ui, &t, &init.shown());
        }
        shell::command_line(ui, &t, &install.shown());
        ui.label(RichText::new(ENGINE_NOTE).color(t.muted.color()));
    }

    fn ready(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let s = &self.state;
        let checks = match &s.checks {
            Fetch::Ready(_) if s.to_look_at() == 0 => CHECKS_CLEAN.to_owned(),
            Fetch::Ready(_) => CHECKS_SOME.replace("{n}", &s.to_look_at().to_string()),
            _ => CHECKS_UNKNOWN.to_owned(),
        };
        let game = s.game.saved.clone().unwrap_or_default();
        let storage = s.storage.saved.clone().unwrap_or_default();
        egui::Grid::new("setup-ready")
            .num_columns(2)
            .spacing([24.0, 8.0])
            .show(ui, |ui| {
                let fact = |ui: &mut egui::Ui, name: &str, value: &str, mono: bool| {
                    ui.label(RichText::new(name).color(t.muted.color()));
                    let rich = if mono {
                        RichText::new(value).monospace()
                    } else {
                        RichText::new(value)
                    };
                    ui.add(egui::Label::new(rich).truncate())
                        .on_hover_text(value);
                    ui.end_row();
                };
                fact(ui, FACT_GAME, &game, true);
                fact(ui, FACT_STORAGE, &storage, true);
                fact(
                    ui,
                    FACT_TTW,
                    if s.want_ttw { TTW_WANTED } else { TTW_NOT_NOW },
                    false,
                );
                fact(ui, FACT_ENGINE, ENGINE_INSTALLED, false);
                fact(ui, FACT_CHECKS, &checks, false);
            });
        if s.want_ttw {
            ui.add_space(8.0);
            if ui.button(SET_UP_TTW).clicked() {
                cx.shell.go_to = Some(View::Flow(Flow::Ttw));
            }
        }
    }

    fn footer(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let step = self.state.step;
        let refused = self.state.step_fields().iter().any(|k| {
            self.state
                .field(*k)
                .is_some_and(|f| matches!(f.save, Save::Refused(_) | Save::Failed(_)))
        });
        let why = match self.state.missing() {
            Some(Missing::Game) => Some(MISSING_GAME),
            Some(Missing::Storage) => Some(MISSING_STORAGE),
            Some(Missing::Saving) => Some(MISSING_SAVING),
            Some(Missing::Engine) => Some(MISSING_ENGINE),
            Some(Missing::Checks) => Some(MISSING_CHECKS),
            None if refused => Some(MISSING_REFUSED),
            None => None,
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.state_line()).color(t.muted.color()));
            progress(ui, &t, step);
            if let Some(why) = why {
                ui.label(RichText::new(why).color(t.muted.color()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // The engine step's Install is its primary button until the engine is in.
                let primary = !(step == Step::Engine && self.state.engine != EngineStep::Installed);
                let label = if step == Step::Ready { FINISH } else { NEXT };
                let button = if primary {
                    shell::primary_button(&t, label)
                } else {
                    egui::Button::new(label)
                };
                let next = ui.add_enabled(why.is_none() && !self.state.advancing, button);
                if next.clicked() {
                    if step == Step::Ready {
                        cx.shell.go_to = Some(View::Page(PageId::Home));
                    } else {
                        self.state.advancing = true;
                    }
                }
                if step != Step::NewVegas && step != Step::Ready && ui.button(BACK).clicked() {
                    self.state.back();
                }
            });
        });
    }
}

fn rail(ui: &mut egui::Ui, t: &Tokens, current: Step) {
    for step in Step::ALL {
        let (name, sub) = match step {
            Step::NewVegas => (RAIL_NEW_VEGAS, RAIL_NEW_VEGAS_SUB),
            Step::Ttw => (RAIL_TTW, RAIL_TTW_SUB),
            Step::Engine => (RAIL_ENGINE, RAIL_ENGINE_SUB),
            Step::Check => (RAIL_CHECK, RAIL_CHECK_SUB),
            Step::Ready => (RAIL_READY, RAIL_READY_SUB),
        };
        let here = step == current;
        let done = step.number() < current.number();
        let frame = egui::Frame::new()
            .fill(if here {
                t.sel.color()
            } else {
                egui::Color32::TRANSPARENT
            })
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(10, 7));
        let response = frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let colour = if here || done { t.text } else { t.muted };
                        let mut name = RichText::new(name).color(colour.color());
                        if here {
                            name = name.family(egui::FontFamily::Name(theme::SEMIBOLD.into()));
                        }
                        ui.label(name);
                        ui.label(
                            RichText::new(sub)
                                .text_style(theme::secondary_style())
                                .color(t.muted.color()),
                        );
                    });
                    if done {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            icon(ui, t, Mark::Ok);
                        })
                        .response
                        .on_hover_text(DONE);
                    }
                });
            })
            .response;
        if here {
            let bar = egui::Rect::from_min_max(
                egui::pos2(response.rect.left() - 4.0, response.rect.top() + 8.0),
                egui::pos2(response.rect.left() - 1.0, response.rect.bottom() - 8.0),
            );
            ui.painter().rect_filled(bar, 2.0, t.accent.color());
        }
    }
    ui.add_space(12.0);
    ui.label(
        RichText::new(RAIL_HINT)
            .text_style(theme::secondary_style())
            .color(t.muted.color()),
    );
}

/// The wizard's progress as a short bar (decorative; the step count is the text).
fn progress(ui: &mut egui::Ui, t: &Tokens, step: Step) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 6.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 3.0, t.line.color());
    #[allow(clippy::cast_precision_loss)]
    let share = (step.number() - 1) as f32 / (Step::ALL.len() - 1) as f32;
    let filled =
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * share, rect.height()));
    p.rect_filled(filled, 3.0, t.accent.color());
}

/// One of two choices drawn as a card with a radio; clicking anywhere on it chooses it.
fn option_card(ui: &mut egui::Ui, t: &Tokens, chosen: bool, title: &str, sub: &str) -> bool {
    let stroke = if chosen { t.accent } else { t.line };
    let frame = egui::Frame::new()
        .fill(if chosen { t.sel.color() } else { t.bg.color() })
        .stroke(Stroke::new(1.0, stroke.color()))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(14, 12));
    // Each card adds its margins (2 x 14) and stroke to this width.
    let width = ((ui.available_width() - 80.0) / 2.0).clamp(200.0, 340.0);
    let inner = frame.show(ui, |ui| {
        ui.set_width(width);
        ui.vertical(|ui| {
            let radio = ui.radio(
                chosen,
                RichText::new(title).family(egui::FontFamily::Name(theme::SEMIBOLD.into())),
            );
            ui.add(egui::Label::new(RichText::new(sub).color(t.muted.color())).wrap());
            radio.clicked()
        })
        .inner
    });
    inner.inner || inner.response.interact(egui::Sense::click()).clicked()
}

fn checks(ui: &mut egui::Ui, t: &Tokens, state: &State) {
    let command = doctor_command(&state.instance().unwrap_or_default());
    match &state.checks {
        Fetch::Idle | Fetch::Loading => waiting(ui, t, CHECKING, &command),
        Fetch::Failed(why) => {
            error_block(ui, t, &CHECKS_FAILED, Some(why));
            shell::command_line(ui, t, &command.shown());
        }
        Fetch::Ready(c) => {
            findings(ui, t, c);
            shell::command_line(ui, t, &command.shown());
        }
    }
}

/// How many findings the check step lists.
const SHOWN: usize = 8;

fn findings(ui: &mut egui::Ui, t: &Tokens, c: &Checks) {
    if c.findings.is_empty() {
        mark(ui, t, Mark::Ok, NOTHING_FOUND);
        return;
    }
    for f in c.findings.iter().take(SHOWN) {
        finding_row(ui, t, f);
    }
    if c.findings.len() > SHOWN {
        ui.label(RichText::new(MORE_FINDINGS).color(t.muted.color()));
    }
}

fn finding_row(ui: &mut egui::Ui, t: &Tokens, f: &Finding) {
    let (m, word) = match f.severity.as_str() {
        "error" => (Mark::Problem, MARK_PROBLEM),
        "warning" => (Mark::Warning, MARK_WARNING),
        "info" => (Mark::Unset, MARK_INFO),
        _ => (Mark::Ok, MARK_OK),
    };
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(110.0, 20.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| mark(ui, t, m, word),
        );
        ui.vertical(|ui| {
            ui.label(RichText::new(&f.subject).monospace());
            ui.add(egui::Label::new(RichText::new(&f.message).color(t.muted.color())).wrap());
        });
    });
    shell::hairline(ui, t);
}

/// The states the snapshot tests render.
#[allow(clippy::too_many_lines)]
pub fn fixtures() -> Vec<Fixture> {
    let nv = "/games/steamapps/common/Fallout New Vegas";
    let storage = "/users/player/raul-storage";
    let config = Config::with(
        "/users/player/.config/raul/config.toml",
        &[
            (Key::NewVegas, nv),
            (Key::StorageRoot, storage),
            (Key::Instance, "/users/player/raul-storage/instance"),
        ],
    );
    let base = State::opened(config.clone(), None);
    let at = |step: Step, s: State| Setup::with(State { step, ..s });
    let mut refused = State::opened(
        Config::with("/users/player/.config/raul/config.toml", &[]),
        Some("/users/player/.local/share/raul"),
    );
    refused.game = Field::typed(
        Key::NewVegas,
        None,
        "/users/player/Downloads",
        Save::Refused(
            "/users/player/Downloads is not a Fallout: New Vegas folder: no FalloutNV.exe in it"
                .into(),
        ),
    );
    let checks = Checks {
        errors: 1,
        warnings: 1,
        findings: vec![
            Finding {
                severity: "error".into(),
                subject: "SampleQuestPack.esp".into(),
                message: "needs SampleLighting.esp, which is not active".into(),
            },
            Finding {
                severity: "warning".into(),
                subject: "loadorder.txt".into(),
                message: "two plugins are out of the masterlist's order".into(),
            },
        ],
    };
    vec![
        Fixture::new(
            "reading",
            Setup::with(State {
                config: Fetch::Loading,
                ..State::default()
            }),
        ),
        Fixture::new(
            "new-vegas-empty",
            Setup::with(State::opened(
                Config::with("/users/player/.config/raul/config.toml", &[]),
                Some("/users/player/.local/share/raul"),
            )),
        ),
        Fixture::new("new-vegas-refused", Setup::with(refused)),
        Fixture::new("new-vegas-found", Setup::with(base.clone())),
        Fixture::new("ttw-not-now", at(Step::Ttw, base.clone())),
        Fixture::new(
            "ttw-wanted",
            at(
                Step::Ttw,
                State {
                    want_ttw: true,
                    ..base.clone()
                },
            ),
        ),
        Fixture::new("engine", at(Step::Engine, base.clone())),
        Fixture::new(
            "engine-installing",
            at(
                Step::Engine,
                State {
                    engine: EngineStep::Installing,
                    ..base.clone()
                },
            ),
        ),
        Fixture::new(
            "engine-failed",
            at(
                Step::Engine,
                State {
                    engine: EngineStep::Failed(
                        "nvmod install-nv-rs: failed and was rolled back: the download stopped halfway".into(),
                    ),
                    ..base.clone()
                },
            ),
        ),
        Fixture::new(
            "engine-installed",
            at(
                Step::Engine,
                State {
                    engine: EngineStep::Installed,
                    ..base.clone()
                },
            ),
        ),
        Fixture::new(
            "check-running",
            at(
                Step::Check,
                State {
                    engine: EngineStep::Installed,
                    checks: Fetch::Loading,
                    ..base.clone()
                },
            ),
        ),
        Fixture::new(
            "check-done",
            at(
                Step::Check,
                State {
                    engine: EngineStep::Installed,
                    checks: Fetch::Ready(checks.clone()),
                    ..base.clone()
                },
            ),
        ),
        Fixture::new(
            "ready",
            at(
                Step::Ready,
                State {
                    want_ttw: true,
                    engine: EngineStep::Installed,
                    checks: Fetch::Ready(checks),
                    ..base.clone()
                },
            ),
        ),
        Fixture::new(
            "ready-clean",
            at(
                Step::Ready,
                State {
                    engine: EngineStep::Installed,
                    checks: Fetch::Ready(Checks::default()),
                    ..base
                },
            ),
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

    /// Answers each command as `raul` does on success and records them.
    #[derive(Default)]
    struct Script {
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl Exec for Script {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            let stdout = match args.first().map(String::as_str) {
                Some("doctor") => {
                    r#"{"base":"/g","findings":[{"code":"a.b","severity":"warning","subject":"loadorder.txt","message":"out of order"}],"summary":{"info":0,"warnings":1,"errors":0}}"#
                }
                _ => "",
            };
            Ok(Output {
                code: Some(0),
                stdout: stdout.into(),
                stderr: String::new(),
            })
        }
    }

    fn wait_for(h: &mut Harness<'_, App>, done: impl Fn(&App) -> bool) {
        for _ in 0..200 {
            // Job threads ask for repaints as they report, so step rather than wait for the UI to settle.
            h.run_steps(2);
            if done(h.state()) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the wizard did not get there");
    }

    #[test]
    fn the_wizard_installs_the_engine_runs_the_checks_and_opens_home() {
        let storage = std::env::temp_dir().join("raul-setup-test-does-not-exist");
        let storage = storage.to_string_lossy().into_owned();
        let state = State::opened(
            Config::with(
                "/c",
                &[(Key::NewVegas, "/games/nv"), (Key::StorageRoot, &storage)],
            ),
            None,
        );
        let exec = Arc::new(Script::default());
        let app = App::fixture(
            Fixture::new("flow", Setup::with(state)),
            Jobs::new(exec.clone()),
        );
        let mut h = Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            });
        h.run();
        h.get_by_label(NEXT).click();
        wait_for(&mut h, |_| true);
        h.get_by_label(NEXT).click();
        h.run_steps(2);
        h.get_by_label(INSTALL).click();
        let calls = exec.clone();
        wait_for(&mut h, move |_| calls.calls.lock().unwrap().len() >= 2);
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(4);
        h.get_by_label(NEXT).click();
        let calls = exec.clone();
        wait_for(&mut h, move |_| calls.calls.lock().unwrap().len() >= 3);
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(4);
        h.get_by_label(NEXT).click();
        h.run_steps(2);
        assert!(h.query_by_label(SUB_READY).is_some());
        h.get_by_label(FINISH).click();
        h.run_steps(2);
        assert_eq!(
            h.state().view(),
            View::Page(PageId::Home),
            "Open the Toolbox goes Home"
        );
        let instance = format!("{storage}/instance");
        let calls = exec.calls.lock().unwrap();
        assert_eq!(
            calls[0],
            [
                "init",
                "--base",
                "/games/nv",
                "--instance",
                instance.as_str()
            ]
        );
        assert_eq!(
            calls[1],
            [
                "install-nv-rs",
                "--instance",
                instance.as_str(),
                "--release"
            ]
        );
        assert_eq!(
            calls[2],
            ["doctor", "--instance", instance.as_str(), "--json"]
        );
    }
}
