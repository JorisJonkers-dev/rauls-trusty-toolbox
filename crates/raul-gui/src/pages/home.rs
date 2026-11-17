// SPDX-License-Identifier: GPL-3.0-only
//! Home: what is ready, what needs a look, and the launch panel (UI-RULES G2, G3, G5).
//!
//! The readiness rows show the paths of Settings, the engine of Versions, TTW and the two pages of the Play group,
//! each one click from the page that fixes it. The chips above them list what needs attention. Play runs
//! `raul run nv-rs`; when something blocks it, the reason is shown under it with a Fix button, and pressing it
//! explains the reason too. On a first run, with no New Vegas folder set, Home opens the setup wizard once.
//!
//! The status mark at the end of this file (colour, icon and word, UI-RULES C3) is shared with Settings, Versions
//! and the wizard.

use super::settings::{
    Pending, Visits, error_block, failure_of, read_config, settle, share_config, start,
};
use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText, Stroke};
use raul_ui_model::pages::home::{Block, Launch, Level, Row, RowId, State, Why};
use raul_ui_model::pages::settings::{Command, Config, Fetch, Key};
use raul_ui_model::pages::versions::{self as engine_model, Engine, EngineStatus};
use raul_ui_model::pages::{Attention, PageId, Severity, View};

crate::texts!(TEXTS {
    STATE_CHECKING = "Checking";
    STATE_READY = "Ready to play";
    STATE_ONE = "One thing to look at";
    STATE_MANY = "{n} things to look at";
    STATE_RUNNING = "The game is running";
    CHECK_AGAIN = "Check again";
    SECTION_READINESS = "Readiness";
    SECTION_LAUNCH = "Launch";
    ROW_NEW_VEGAS = "Game folder";
    ROW_STORAGE = "Storage";
    ROW_ENGINE = "Engine";
    ROW_TTW = "TTW";
    ROW_LOAD_ORDER = "Load order";
    ROW_LOD = "LOD";
    DETAIL_GAME_NOT_SET = "Not set. Point the Toolbox at your game.";
    DETAIL_STORAGE_NOT_SET = "Not set. Pick where the Toolbox keeps its files.";
    DETAIL_CHECKING = "Checking";
    DETAIL_ENGINE_ACTIVE = "nv-rs, checked against its install record";
    DETAIL_ENGINE_MISSING = "No engine installed yet";
    DETAIL_TTW_READY = "Sources set. Ready when you are.";
    DETAIL_TTW_NEEDS = "Optional. Needs Fallout 3 and the TTW installer.";
    DETAIL_LOAD_ORDER = "Sorted and reviewed on its own page";
    DETAIL_LOD = "Generated and checked on its own page";
    MARK_OK = "OK";
    MARK_WARNING = "Warning";
    MARK_PROBLEM = "Problem";
    MARK_NOT_SET = "Not set";
    MARK_NOT_CHECKED = "Not checked";
    MARK_CHECKING = "Checking";
    FIX = "Fix";
    OPEN = "Open";
    SET_UP = "Set up…";
    PLAY_BIG = "Play now";
    STARTING = "Starting";
    FACT_ENGINE = "Engine";
    FACT_INSTANCE = "Instance";
    FACT_ENGINE_ACTIVE = "nv-rs";
    FACT_NONE = "None yet";
    RUN_SHAPE = "raul run nv-rs --instance <dir>";
    STARTED_STATUS = "The game was started.";
    RAUL_READY = "All set. Go on, the Mojave's waiting.";
    RAUL_WARNINGS = "Ready. A thing or two could be better.";
    RAUL_STARTED = "Game's running. I'll be here.";
});

crate::errors!(ERRORS {
    PLAY_UNREADABLE = (
        "The settings could not be read",
        "Open Settings to see why, then press Play again."
    );
    PLAY_NO_STORAGE = (
        "No storage folder is set",
        "Set it in Settings; it holds the Instance the game runs from."
    );
    PLAY_NO_ENGINE = (
        "The engine is not installed",
        "Install it on Versions, then press Play again."
    );
    PLAY_ENGINE_CHANGED = (
        "The engine differs from its install record",
        "Nothing was started. Install it again on Versions."
    );
    RUN_FAILED = (
        "The game did not start",
        "Nothing else changed. The line below says why."
    );
});

/// The Home page.
#[derive(Debug, Default)]
pub struct Home {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Config,
    Verify,
    Run,
}

const BLOCKED_DIALOG: &str = "home-play-blocked";

impl Home {
    /// Home in `state`, as a fixture shows it.
    pub fn with(state: State) -> Home {
        Home {
            state,
            ..Home::default()
        }
    }

    fn load(&mut self, cx: &mut PageCx<'_>, first: bool) {
        self.state.engine = Fetch::Idle;
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

    /// Reads the engine once the Instance is known: an Instance that does not exist yet holds no engine.
    fn load_engine(&mut self, cx: &mut PageCx<'_>, config: &Config) {
        let Some(instance) = config.get(Key::Instance) else {
            self.state.engine = Fetch::Ready(Engine::missing());
            return;
        };
        if !std::path::Path::new(instance).exists() {
            self.state.engine = Fetch::Ready(Engine::missing());
            return;
        }
        match start(cx, &engine_model::verify_command(instance)) {
            Ok(id) => {
                self.pending.add(Job::Verify, id);
                self.state.engine = Fetch::Loading;
            }
            Err(e) => self.state.engine = Fetch::Failed(e.what.to_owned()),
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            match job {
                Job::Config => match read_config(&result) {
                    Ok(config) => {
                        self.load_engine(cx, &config);
                        self.state.config = Fetch::Ready(config);
                    }
                    Err(why) => self.state.config = Fetch::Failed(why),
                },
                Job::Verify => {
                    self.state.engine = match &result {
                        Ok(out) => engine_model::parse_verify(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
                Job::Run => {
                    self.state.launch = match &result {
                        Ok(out) if out.code == Some(0) => {
                            cx.shell.status = Some(STARTED_STATUS.to_owned());
                            Launch::Started
                        }
                        Ok(out) => Launch::Failed(failure_of(out)),
                        Err(why) => Launch::Failed(why.clone()),
                    };
                }
            }
        }
    }

    /// Tells the shell what the top bar shows and whether its Play is blocked.
    fn share(&self, cx: &mut PageCx<'_>) {
        if let Some(config) = self.state.config.ready() {
            share_config(cx, config);
        }
        if let Some(engine) = self.state.engine.ready() {
            cx.shell.engine =
                (engine.status() == EngineStatus::Active).then(|| FACT_ENGINE_ACTIVE.to_owned());
        }
        if !self.state.config.is_pending() && !self.state.engine.is_pending() {
            cx.shell.play_blocked = self.state.block().map(block_text);
        }
    }

    fn play(&mut self, cx: &mut PageCx<'_>) {
        if let Some(block) = self.state.block() {
            let e = block_text(block);
            cx.dialogs.open(Dialog::confirm(
                BLOCKED_DIALOG,
                msg::PLAY_BLOCKED_TITLE,
                e.detail,
                FIX,
            ));
            return;
        }
        let Some(command) = self.state.run_command().filter(|_| self.state.can_play()) else {
            return;
        };
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(Job::Run, id);
                self.state.launch = Launch::Starting;
            }
            Err(e) => self.state.launch = Launch::Failed(e.what.to_owned()),
        }
    }
}

/// The error a block shows.
pub(crate) fn block_text(block: Block) -> ErrorText {
    match block {
        Block::Unreadable => PLAY_UNREADABLE,
        Block::NoGame => msg::PLAY_NO_GAME,
        Block::NoStorage => PLAY_NO_STORAGE,
        Block::NoEngine => PLAY_NO_ENGINE,
        Block::EngineDiffers => PLAY_ENGINE_CHANGED,
    }
}

impl Page for Home {
    fn view(&self) -> View {
        View::Page(PageId::Home)
    }

    fn title(&self) -> &'static str {
        msg::PAGE_HOME
    }

    fn state_line(&self) -> String {
        if self.state.launch == Launch::Started {
            return STATE_RUNNING.to_owned();
        }
        if self.state.config.is_pending() || self.state.engine.is_pending() {
            return STATE_CHECKING.to_owned();
        }
        match self.state.attention().len() {
            0 => STATE_READY.to_owned(),
            1 => STATE_ONE.to_owned(),
            n => STATE_MANY.replace("{n}", &n.to_string()),
        }
    }

    fn attention(&self) -> Option<Attention> {
        self.state
            .chip()
            .map(|(severity, count)| Attention { severity, count })
    }

    fn tools(&mut self, ui: &mut egui::Ui, _cx: &mut PageCx<'_>) {
        let busy = self.state.config.is_pending() || self.state.engine.is_pending();
        if ui
            .add_enabled(!busy, egui::Button::new(CHECK_AGAIN))
            .clicked()
        {
            self.state.config = Fetch::Idle;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        if self.visits.returned(ui.ctx()) && self.state.launch != Launch::Starting {
            self.state.config = Fetch::Idle;
        }
        if self.state.config == Fetch::Idle {
            let first = !self.visits.loaded;
            self.visits.loaded = true;
            self.load(cx, first);
        }
        self.take_answers(cx);
        self.share(cx);
        if self.state.wants_setup() {
            self.state.offered_setup = true;
            cx.shell.go_to = Some(View::Flow(raul_ui_model::pages::Flow::Setup));
        }
        if cx.dialogs.take_answer(BLOCKED_DIALOG) == Some(Answer::Confirmed)
            && let Some(block) = self.state.block()
        {
            cx.shell.go_to = Some(block.fix());
        }
        // The shell's Play: start once the reads are in; a block found by then is shown, not started.
        if cx.shell.play_requested
            && !self.state.config.is_pending()
            && !self.state.engine.is_pending()
        {
            cx.shell.play_requested = false;
            if self.state.block().is_none() {
                self.play(cx);
            }
        }
        let t = *cx.tokens;
        let rows = self.state.rows();
        chips(ui, &t, cx, &self.state.attention());
        let wide = ui.available_width() >= 760.0;
        let readiness = |ui: &mut egui::Ui, cx: &mut PageCx<'_>| {
            shell::section_label(ui, &t, SECTION_READINESS, true);
            for row in &rows {
                readiness_row(ui, &t, cx, row);
            }
        };
        if wide {
            ui.horizontal_top(|ui| {
                let left = (ui.available_width() * 0.58).floor();
                ui.allocate_ui_with_layout(
                    egui::vec2(left, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(left);
                        readiness(ui, cx);
                    },
                );
                ui.add_space(24.0);
                ui.vertical(|ui| self.launch_panel(ui, cx));
            });
        } else {
            readiness(ui, cx);
            ui.add_space(16.0);
            self.launch_panel(ui, cx);
        }
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl Home {
    fn launch_panel(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        shell::section_label(ui, &t, SECTION_LAUNCH, true);
        let block = self.state.block();
        let checking = self.state.config.is_pending() || self.state.engine.is_pending();
        let line = match (&self.state.launch, block) {
            (Launch::Started, _) => Some(RAUL_STARTED),
            (Launch::Failed(_), _) | (_, Some(_)) => None,
            _ if checking => None,
            _ if self.state.attention().is_empty() => Some(RAUL_READY),
            _ => Some(RAUL_WARNINGS),
        };
        if let Some(line) = line {
            shell::raul_line(ui, &t, line);
            ui.add_space(4.0);
        }
        let starting = self.state.launch == Launch::Starting;
        let label = if starting { STARTING } else { PLAY_BIG };
        let button = shell::primary_button(&t, label).min_size(egui::vec2(180.0, 44.0));
        let pressed = ui.add_enabled(!starting && !checking, button);
        if let Some(b) = block {
            pressed.clone().on_hover_text(block_text(b).what);
        }
        if pressed.clicked() {
            self.play(cx);
        }
        if starting {
            icon(ui, &t, Mark::Info);
        }
        if let Some(b) = block {
            let e = block_text(b);
            ui.horizontal_wrapped(|ui| {
                icon(ui, &t, Mark::Problem);
                ui.label(RichText::new(e.what).color(t.err.color()));
                if ui.button(FIX).clicked() {
                    cx.shell.go_to = Some(b.fix());
                }
            });
            ui.label(RichText::new(e.detail).color(t.muted.color()));
        }
        if let Launch::Failed(why) = &self.state.launch {
            error_block(ui, &t, &RUN_FAILED, Some(why));
        }
        let command = self
            .state
            .run_command()
            .map_or_else(|| RUN_SHAPE.to_owned(), |c| c.shown());
        shell::command_line(ui, &t, &command);
        ui.add_space(8.0);
        let engine = match self.state.engine.ready().map(Engine::status) {
            Some(EngineStatus::Active) => FACT_ENGINE_ACTIVE,
            Some(_) => FACT_NONE,
            None => DETAIL_CHECKING,
        };
        let instance = self
            .state
            .config
            .ready()
            .and_then(|c| c.get(Key::Instance))
            .unwrap_or(FACT_NONE)
            .to_owned();
        egui::Grid::new("home-facts")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new(FACT_ENGINE).color(t.muted.color()));
                ui.label(RichText::new(engine).monospace());
                ui.end_row();
                ui.label(RichText::new(FACT_INSTANCE).color(t.muted.color()));
                ui.add(egui::Label::new(RichText::new(&instance).monospace()).truncate())
                    .on_hover_text(&instance);
                ui.end_row();
            });
    }
}

/// The chips of what needs attention; each opens the page that fixes it (UI-RULES G2).
fn chips(ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>, rows: &[Row]) {
    if rows.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for row in rows {
            let m = mark_of(row.level);
            let colour = m.colour(t);
            let chip = egui::Button::new(RichText::new(row_title(row.id)).color(t.text.color()))
                .fill(colour.gamma_multiply(0.16))
                .stroke(Stroke::new(1.0, colour))
                .corner_radius(14);
            if ui.add(chip).clicked() {
                cx.shell.go_to = Some(row.id.home());
            }
        }
    });
    ui.add_space(12.0);
}

fn readiness_row(ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>, row: &Row) {
    let width = ui.available_width();
    let mark_width = 120.0;
    let action_width = 96.0;
    let text_width = (width - mark_width - action_width - 16.0).max(120.0);
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.set_width(text_width);
            ui.label(
                RichText::new(row_title(row.id))
                    .family(egui::FontFamily::Name(theme::SEMIBOLD.into())),
            );
            let detail = row_detail(row);
            let rich = if matches!(row.why, Why::Path(_) | Why::Unreadable(_)) {
                RichText::new(&detail).monospace().color(t.muted.color())
            } else {
                RichText::new(&detail).color(t.muted.color())
            };
            ui.add(egui::Label::new(rich).truncate())
                .on_hover_text(&detail);
        });
        ui.allocate_ui_with_layout(
            egui::vec2(mark_width, 20.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| mark(ui, t, mark_of(row.level), mark_word(row.level)),
        );
        let action = match (row.id, row.level) {
            (RowId::Ttw, _) => Some(SET_UP),
            (RowId::LoadOrder | RowId::Lod, _) => Some(OPEN),
            (_, Level::Problem | Level::Warning) => Some(FIX),
            _ => None,
        };
        if let Some(label) = action
            && ui.button(label).clicked()
        {
            cx.shell.go_to = Some(row.id.home());
        }
    });
    shell::hairline(ui, t);
}

fn row_title(id: RowId) -> &'static str {
    match id {
        RowId::NewVegas => ROW_NEW_VEGAS,
        RowId::Storage => ROW_STORAGE,
        RowId::Engine => ROW_ENGINE,
        RowId::Ttw => ROW_TTW,
        RowId::LoadOrder => ROW_LOAD_ORDER,
        RowId::Lod => ROW_LOD,
    }
}

fn row_detail(row: &Row) -> String {
    match &row.why {
        Why::Path(p) => p.clone(),
        Why::Unreadable(m) | Why::EngineDiffers(m) => m.clone(),
        Why::NotSet if row.id == RowId::Storage => DETAIL_STORAGE_NOT_SET.to_owned(),
        Why::NotSet => DETAIL_GAME_NOT_SET.to_owned(),
        Why::Checking => DETAIL_CHECKING.to_owned(),
        Why::EngineActive => DETAIL_ENGINE_ACTIVE.to_owned(),
        Why::EngineMissing => DETAIL_ENGINE_MISSING.to_owned(),
        Why::TtwReady => DETAIL_TTW_READY.to_owned(),
        Why::TtwNeedsSources => DETAIL_TTW_NEEDS.to_owned(),
        Why::OnItsPage if row.id == RowId::Lod => DETAIL_LOD.to_owned(),
        Why::OnItsPage => DETAIL_LOAD_ORDER.to_owned(),
    }
}

fn mark_of(level: Level) -> Mark {
    match level {
        Level::Ok => Mark::Ok,
        Level::Warning => Mark::Warning,
        Level::Problem => Mark::Problem,
        Level::NotSet | Level::Elsewhere => Mark::Unset,
        Level::Checking => Mark::Info,
    }
}

fn mark_word(level: Level) -> &'static str {
    match level {
        Level::Ok => MARK_OK,
        Level::Warning => MARK_WARNING,
        Level::Problem => MARK_PROBLEM,
        Level::NotSet => MARK_NOT_SET,
        Level::Elsewhere => MARK_NOT_CHECKED,
        Level::Checking => MARK_CHECKING,
    }
}

// ---- The status mark, shared by the setup path pages (UI-RULES C3: colour, icon and word). ----

/// The kind of a status mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    /// OK, found, active.
    Ok,
    /// Worth a look.
    Warning,
    /// Blocks, or failed.
    Problem,
    /// Running work, an update, or being read.
    Info,
    /// Not set, and that is allowed.
    Unset,
}

impl Mark {
    /// Its colour.
    pub(crate) fn colour(self, t: &Tokens) -> egui::Color32 {
        match self {
            Mark::Ok => t.ok.color(),
            Mark::Warning => shell::severity_colour(t, Severity::Warning),
            Mark::Problem => shell::severity_colour(t, Severity::Problem),
            Mark::Info => shell::severity_colour(t, Severity::Info),
            Mark::Unset => t.muted.color(),
        }
    }
}

/// A mark's icon, 16 px, drawn in its colour; hidden from screen readers since the word follows it.
pub(crate) fn icon(ui: &mut egui::Ui, t: &Tokens, m: Mark) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    let c = rect.center();
    let colour = m.colour(t);
    let stroke = Stroke::new(1.6, colour);
    let p = ui.painter();
    match m {
        Mark::Ok => {
            p.circle_stroke(c, 6.5, stroke);
            p.line_segment(
                [c + egui::vec2(-3.0, 0.3), c + egui::vec2(-0.8, 2.6)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(-0.8, 2.6), c + egui::vec2(3.2, -2.4)],
                stroke,
            );
        }
        Mark::Warning => {
            let top = c + egui::vec2(0.0, -6.5);
            let left = c + egui::vec2(-7.0, 5.5);
            let right = c + egui::vec2(7.0, 5.5);
            p.line_segment([top, left], stroke);
            p.line_segment([left, right], stroke);
            p.line_segment([right, top], stroke);
            p.line_segment(
                [c + egui::vec2(0.0, -2.0), c + egui::vec2(0.0, 1.6)],
                stroke,
            );
            p.circle_filled(c + egui::vec2(0.0, 3.6), 0.9, colour);
        }
        Mark::Problem => {
            p.circle_stroke(c, 6.5, stroke);
            p.line_segment(
                [c + egui::vec2(0.0, -3.4), c + egui::vec2(0.0, 0.8)],
                stroke,
            );
            p.circle_filled(c + egui::vec2(0.0, 3.2), 0.9, colour);
        }
        Mark::Info => {
            p.line_segment(
                [c + egui::vec2(0.0, -6.0), c + egui::vec2(0.0, 3.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(-3.5, -0.5), c + egui::vec2(0.0, 3.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(3.5, -0.5), c + egui::vec2(0.0, 3.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(-5.5, 6.0), c + egui::vec2(5.5, 6.0)],
                stroke,
            );
        }
        Mark::Unset => {
            p.circle_stroke(c, 6.5, stroke);
            p.line_segment(
                [c + egui::vec2(-3.0, 0.0), c + egui::vec2(3.0, 0.0)],
                stroke,
            );
        }
    }
}

/// A status mark: icon and word in the mark's colour.
pub(crate) fn mark(ui: &mut egui::Ui, t: &Tokens, m: Mark, word: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        icon(ui, t, m);
        ui.label(RichText::new(word).color(m.colour(t)));
    });
}

/// The states the snapshot tests render.
#[allow(clippy::too_many_lines)]
pub fn fixtures() -> Vec<Fixture> {
    let file = "/users/player/.config/raul/config.toml";
    let all = Config::with(
        file,
        &[
            (Key::NewVegas, "/games/steamapps/common/Fallout New Vegas"),
            (Key::StorageRoot, "/users/player/raul-storage"),
            (Key::Instance, "/users/player/raul-storage/instance"),
        ],
    );
    let mut ttw = all.clone();
    ttw.put(
        Key::Fallout3,
        Some("/games/steamapps/common/Fallout 3 goty".into()),
    );
    ttw.put(
        Key::TtwPackage,
        Some("/users/player/Downloads/ttw-installer.mpi".into()),
    );
    let seen = |s: State| {
        Home::with(State {
            offered_setup: true,
            ..s
        })
    };
    let deep = "/mnt/second-drive/a/very/deeply/nested/library/folder/that/keeps/going/and/going/steamapps/common/Fallout New Vegas";
    let long = Config::with(
        file,
        &[
            (Key::NewVegas, deep),
            (Key::StorageRoot, deep),
            (Key::Instance, deep),
        ],
    );
    let differs = Engine {
        installed: true,
        verified: false,
        message: "nv-rs differs from the checksum recorded when it was installed".into(),
    };
    vec![
        Fixture::new(
            "checking",
            seen(State {
                config: Fetch::Loading,
                engine: Fetch::Loading,
                ..State::default()
            }),
        ),
        Fixture::new(
            "first-run",
            seen(State::ready(Config::with(file, &[]), Engine::missing())),
        ),
        Fixture::new(
            "no-engine",
            seen(State::ready(all.clone(), Engine::missing())),
        ),
        Fixture::new("engine-differs", seen(State::ready(all.clone(), differs))),
        Fixture::new("ready", seen(State::ready(all.clone(), Engine::active()))),
        Fixture::new("ttw-ready", seen(State::ready(ttw, Engine::active()))),
        Fixture::new(
            "warning",
            seen(State {
                config: Fetch::Ready(all.clone()),
                engine: Fetch::Failed("raul verify: refused: the Instance has no profile".into()),
                ..State::default()
            }),
        ),
        Fixture::new(
            "error",
            seen(State {
                config: Fetch::Failed(
                    "the configuration file has schema_version 2; this raul reads 1".into(),
                ),
                engine: Fetch::Ready(Engine::missing()),
                ..State::default()
            }),
        ),
        Fixture::new(
            "starting",
            seen(State {
                launch: Launch::Starting,
                ..State::ready(all.clone(), Engine::active())
            }),
        ),
        Fixture::new(
            "started",
            seen(State {
                launch: Launch::Started,
                ..State::ready(all.clone(), Engine::active())
            }),
        ),
        Fixture::new(
            "run-failed",
            seen(State {
                launch: Launch::Failed(
                    "raul run: refused: no memory-capped run guard is available".into(),
                ),
                ..State::ready(all, Engine::active())
            }),
        ),
        Fixture::new("long-text", seen(State::ready(long, Engine::active()))),
    ]
}
