// SPDX-License-Identifier: GPL-3.0-only
//! Home: what is ready, what needs a look, and the launch panel (UI-RULES G2, G3, G5).
//!
//! The readiness rows show the paths of Settings, TTW and the two pages of the Play group,
//! each one click from the page that fixes it. The chips above them list what needs attention. Play runs
//! `raul run nv-rs`; when the setup misses a step, Play says which and its Fix opens the page that sets it up
//! (UI-RULES G3). The panel and the header say "Ready to play" and "All set" only when readiness holds.
//! On a first run, with no New Vegas folder set, Home opens the setup wizard once.
//!
//! The status mark at the end of this file (colour, icon and word, UI-RULES C3) is shared with Settings, Versions
//! and the wizard.

use super::settings::{
    Pending, UNREADABLE, Visits, error_block, failure_of, read_config, settle, share_config, start,
};
use super::{Fixture, Page, PageCx, ShellState};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText, Stroke};
use raul_ui_model::pages::home::{
    Block, Launch, Level, Readiness, Row, RowId, State, Step, Why, doctor_command, load_order_command,
    parse_order_errors,
};
use raul_ui_model::pages::settings::{Command, Config, Fetch, Key};
use raul_ui_model::pages::{Attention, PageId, Severity, View};

crate::texts!(TEXTS {
    STATE_CHECKING = "Checking";
    STATE_READY = "Ready to play";
    STATE_ONE = "One thing to look at";
    STATE_MANY = "{n} things to look at";
    STATE_RUNNING = "The game is running";
    STATE_NOT_READY = "Not ready yet";
    STATE_NEXT = "Not ready yet: {step}";
    STEP_GAME = "set the game folder";
    STEP_STORAGE = "set the storage folder";
    STEP_INSTANCE = "set up the Instance";
    STEP_LOAD_ORDER = "fix the load order";
    WHY_NO_SETTINGS = "the settings could not be read";
    WHY_NO_INSTANCE = "the Instance folder is not there yet";
    CHECK_AGAIN = "Check again";
    SECTION_READINESS = "Readiness";
    SECTION_LAUNCH = "Launch";
    ROW_NEW_VEGAS = "Game folder";
    ROW_STORAGE = "Storage";
    ROW_TTW = "TTW";
    ROW_LOAD_ORDER = "Load order";
    ROW_LOD = "LOD";
    DETAIL_GAME_NOT_SET = "Not set. Point the Toolbox at your game.";
    DETAIL_STORAGE_NOT_SET = "Not set. Pick where the Toolbox keeps its files.";
    DETAIL_CHECKING = "Checking";
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
    STOP_GAME = "Stop the game";
    STOP_ASKED_STATUS = "Asked the game to stop";
    STARTING = "Starting";
    FACT_INSTANCE = "Instance";
    FACT_NONE = "None yet";
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
    /// The shell's Play arrived while the reads were still on their way.
    play_waiting: bool,
    /// What the dialog under Play offered to fix, so its Fix opens that page even when the state moved on.
    blocked_fix: Option<View>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Config,
    LoadOrder,
    OrderErrors,
    Run,
    Stop,
}

const BLOCKED_DIALOG: &str = "home-play-blocked";

/// The room the Fix button beside a blocking problem needs, so the text next to it wraps instead of pushing the
/// button past the right edge at the smallest window (UI-RULES A5, A6).
const FIX_ROOM: f32 = 72.0;

impl Home {
    /// Home in `state`, as a fixture shows it.
    pub fn with(state: State) -> Home {
        Home {
            state,
            ..Home::default()
        }
    }

    fn load(&mut self, cx: &mut PageCx<'_>, first: bool) {
        self.state.instance_exists = None;
        self.state.load_order = Fetch::Idle;
        self.state.order_errors = Fetch::Idle;
        match start(cx, &Command::config_list()) {
            Ok(id) => {
                self.pending.add(Job::Config, id);
                self.state.config = Fetch::Loading;
                if first {
                    settle(cx.jobs);
                }
            }
            Err(e) => {
                // Without the settings there are no paths, so no other read could even be asked for. Home settles
                // on this answer instead of waiting for reads that were never started.
                self.state.config = Fetch::Failed(e.what.to_owned());
                self.settle_without_paths(WHY_NO_SETTINGS.to_owned());
            }
        }
    }

    /// Settles the reads that run against the settings' paths, with the reason they were not asked for.
    fn settle_without_paths(&mut self, why: String) {
        self.state.instance_exists = Some(false);
        self.state.load_order = Fetch::Failed(why.clone());
        self.state.order_errors = Fetch::Failed(why);
    }

    /// Settles the reads that run against the Instance folder: an Instance that is not there has no load order to
    /// read.
    fn settle_without_instance(&mut self) {
        self.state.instance_exists = Some(false);
        self.state.load_order = Fetch::Failed(WHY_NO_INSTANCE.to_owned());
        self.state.order_errors = Fetch::Failed(WHY_NO_INSTANCE.to_owned());
    }

    /// Reads the load order once the Instance is known: an Instance that does not exist yet has none.
    fn load_instance(&mut self, cx: &mut PageCx<'_>, config: &Config) {
        let Some(instance) = config.get(Key::Instance) else {
            self.settle_without_instance();
            return;
        };
        if !std::path::Path::new(instance).exists() {
            self.settle_without_instance();
            return;
        }
        self.state.instance_exists = Some(true);
        match start(cx, &load_order_command(instance)) {
            Ok(id) => {
                self.pending.add(Job::LoadOrder, id);
                self.state.load_order = Fetch::Loading;
            }
            Err(e) => self.state.load_order = Fetch::Failed(e.what.to_owned()),
        }
        match start(cx, &doctor_command(instance)) {
            Ok(id) => {
                self.pending.add(Job::OrderErrors, id);
                self.state.order_errors = Fetch::Loading;
            }
            Err(e) => self.state.order_errors = Fetch::Failed(e.what.to_owned()),
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            match job {
                Job::Config => match read_config(&result) {
                    Ok(config) => {
                        self.load_instance(cx, &config);
                        self.state.config = Fetch::Ready(config);
                    }
                    Err(why) => {
                        // The answer said no: the paths are not known, so nothing else was read.
                        self.state.config = Fetch::Failed(why);
                        self.settle_without_paths(WHY_NO_SETTINGS.to_owned());
                    }
                },
                Job::LoadOrder => {
                    self.state.load_order = match &result {
                        Ok(out) if out.code == Some(0) => Fetch::Ready(()),
                        Ok(out) => Fetch::Failed(failure_of(out)),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
                Job::OrderErrors => {
                    self.state.order_errors = match &result {
                        Ok(out) => parse_order_errors(&out.stdout)
                            .map_or_else(|_| Fetch::Failed(failure_of(out)), Fetch::Ready),
                        Err(why) => Fetch::Failed(why.clone()),
                    };
                }
                Job::Stop => {
                    // The game ends between two frames; Home reads its state again.
                    if let Err(why) = &result {
                        self.state.launch = Launch::Failed(why.clone());
                    } else if let Ok(out) = &result
                        && out.code != Some(0)
                    {
                        self.state.launch = Launch::Failed(failure_of(out));
                    } else {
                        cx.shell.status = Some(STOP_ASKED_STATUS.to_owned());
                    }
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
        if !self.state.reading() {
            cx.shell.play_blocked = self.state.block().map(block_text);
        }
    }

    /// Pressing Play: start the game, or say what stands in the way and offer the page that fixes it. A press is
    /// never a silent no-op (UI-RULES G3), and a setup that misses a step never starts a game.
    fn play(&mut self, cx: &mut PageCx<'_>) {
        if self.state.launch == Launch::Starting {
            return; // A launch is on its way; the panel says Starting.
        }
        if self.state.reading() {
            // A Play from the top bar that arrived while the reads run starts as soon as they settle.
            self.play_waiting = true;
            return;
        }
        if let Some((why, fix)) = self.stop() {
            self.blocked_fix = Some(fix);
            let dialog = Dialog::confirm(BLOCKED_DIALOG, msg::PLAY_BLOCKED_TITLE, why, FIX)
                .with_error(self.blocked_error());
            cx.dialogs.open(dialog);
            return;
        }
        if !self.state.can_play() {
            return; // Readiness has the last word on starting; `stop` has already explained the rest.
        }
        let Some(command) = self.state.run_command() else {
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

    /// Stop: asks the running game to end between two frames, so its save flush and shader cache are kept.
    fn stop_game(&mut self, cx: &mut PageCx<'_>) {
        let Some(command) = self.state.stop_command() else {
            return;
        };
        match start(cx, &command) {
            Ok(id) => self.pending.add(Job::Stop, id),
            Err(e) => self.state.launch = Launch::Failed(e.what.to_owned()),
        }
    }

    /// The error the blocked dialog shows and copies: the blocking problem, or the one the reads hit.
    fn blocked_error(&self) -> ErrorText {
        self.state
            .block()
            .map_or(super::settings::UNREADABLE, block_text)
    }

    /// What stands between this press and a game that starts, and the view whose control fixes it: `None` when
    /// nothing does. A known blocking problem says more than the step does; otherwise the step is what readiness
    /// missed, and the sentence under Play says the same thing (rule 2: named once).
    fn stop(&self) -> Option<(String, View)> {
        if let Some(block) = self.state.block() {
            return Some((block_text(block).detail.to_owned(), block.fix()));
        }
        match self.state.readiness() {
            // Checking is not reached: a press during the reads waits in `play` instead.
            Readiness::Ready | Readiness::Checking => None,
            Readiness::Next(step) => {
                Some((STATE_NEXT.replace("{step}", step_text(step)), step.fix()))
            }
        }
    }
}

fn step_text(step: Step) -> &'static str {
    match step {
        Step::Game => STEP_GAME,
        Step::Storage => STEP_STORAGE,
        Step::Instance => STEP_INSTANCE,
        Step::LoadOrder => STEP_LOAD_ORDER,
    }
}

/// The error a block shows.
pub(crate) fn block_text(block: Block) -> ErrorText {
    match block {
        Block::Unreadable => PLAY_UNREADABLE,
        Block::NoGame => msg::PLAY_NO_GAME,
        Block::NoStorage => PLAY_NO_STORAGE,
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
        // The header counts what the rows say and leaves the step that no row marks to the line under Play, so a
        // missing step is named once (UI-RULES G5).
        if matches!(self.state.readiness(), Readiness::Next(_)) && self.state.attention().is_empty()
        {
            return STATE_NOT_READY.to_owned();
        }
        if self.state.reading() {
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
        let busy = self.state.reading();
        if ui
            .add_enabled(!busy, egui::Button::new(CHECK_AGAIN))
            .clicked()
        {
            self.state.config = Fetch::Idle;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let returned = self.visits.returned(ui.ctx());
        if returned && self.play_waiting {
            // A Play that waited for the reads was left behind when the person opened another page: drop it, so
            // the game never starts later without a click.
            self.play_waiting = false;
            cx.shell.play_requested = false;
        }
        if returned && self.state.launch != Launch::Starting && !self.state.reading() {
            if self.state.launch == Launch::Started {
                // `raul run` returned once the game started; coming back, Home shows readiness again.
                self.state.launch = Launch::Idle;
            }
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
        match cx.dialogs.take_answer(BLOCKED_DIALOG) {
            // Fix opens the page the dialog named, even when the reads moved the state on since it opened.
            // Either answer answers the press that opened the dialog: it must not reopen the dialog or start the game
            // later, even when a read started meanwhile.
            Some(Answer::Confirmed) => {
                cx.shell.play_requested = false;
                self.play_waiting = false;
                if let Some(fix) = self.blocked_fix.take() {
                    cx.shell.go_to = Some(fix);
                }
            }
            Some(Answer::Closed) => {
                self.blocked_fix = None;
                cx.shell.play_requested = false;
                self.play_waiting = false;
            }
            None => {}
        }
        // The shell's Play: start once the reads are in; a block found by then is explained, not started.
        if cx.shell.play_requested {
            if self.state.reading() {
                self.play_waiting = true;
            } else {
                cx.shell.play_requested = false;
                self.play_waiting = false;
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
    #[allow(clippy::too_many_lines)]
    fn launch_panel(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        shell::section_label(ui, &t, SECTION_LAUNCH, true);
        let block = self.state.block();
        let checking = self.state.reading();
        let readiness = self.state.readiness();
        let warned = !self.state.attention().is_empty();
        // Raul's line is the panel's headline, so it does not say twice what the error line or the step line under
        // Play already says (UI-RULES G5). "All set" shows only when readiness holds in full (rule 1).
        let line = match &self.state.launch {
            Launch::Started => Some(RAUL_STARTED),
            Launch::Failed(_) => None,
            _ if checking || block.is_some() => None,
            _ if readiness == Readiness::Ready => {
                Some(if warned { RAUL_WARNINGS } else { RAUL_READY })
            }
            _ => None,
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
        if self.state.launch == Launch::Started && ui.button(STOP_GAME).clicked() {
            self.stop_game(cx);
        }
        if let Some(b) = block {
            let e = block_text(b);
            let fix = ui
                .horizontal_top(|ui| {
                    icon(ui, &t, Mark::Problem);
                    ui.vertical(|ui| {
                        // The text keeps room for the Fix button beside it, so the row never reaches past the
                        // page at the smallest window (UI-RULES A5, A6).
                        ui.set_max_width((ui.available_width() - FIX_ROOM).max(80.0));
                        ui.add(
                            egui::Label::new(RichText::new(e.what).color(t.err.color()))
                                .wrap()
                                .selectable(true),
                        );
                        ui.add(
                            egui::Label::new(RichText::new(e.detail).color(t.muted.color()))
                                .wrap()
                                .selectable(true),
                        );
                    });
                    ui.button(FIX).clicked()
                })
                .inner;
            if ui.button(msg::COPY_DETAILS).clicked() {
                let text = shell::copy_text(e.detail, e.what, e.detail);
                ui.ctx().copy_text(text);
            }
            if fix {
                cx.shell.go_to = Some(b.fix());
            }
        }
        // The step that no row marks, said once under Play with the page that sets it up (UI-RULES G2, G3). A step
        // that is known to be missing is named even while other reads run, so the header's "Not ready yet" always
        // has its reason on the page.
        if let Readiness::Next(step) = readiness
            && block.is_none()
            && !warned
        {
            ui.horizontal_wrapped(|ui| {
                icon(ui, &t, Mark::Warning);
                ui.label(
                    RichText::new(STATE_NEXT.replace("{step}", step_text(step)))
                        .color(t.muted.color()),
                );
                if ui.button(FIX).clicked() {
                    cx.shell.go_to = Some(step.fix());
                }
            });
        }
        if let Launch::Failed(why) = &self.state.launch {
            error_block(ui, &t, &RUN_FAILED, Some(why));
        }
        ui.add_space(8.0);
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
        RowId::Ttw => ROW_TTW,
        RowId::LoadOrder => ROW_LOAD_ORDER,
        RowId::Lod => ROW_LOD,
    }
}

fn row_detail(row: &Row) -> String {
    match &row.why {
        Why::Path(p) => p.clone(),
        Why::Unreadable(m) => m.clone(),
        Why::NotSet if row.id == RowId::Storage => DETAIL_STORAGE_NOT_SET.to_owned(),
        Why::NotSet => DETAIL_GAME_NOT_SET.to_owned(),
        Why::Checking => DETAIL_CHECKING.to_owned(),
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
    vec![
        Fixture::new(
            "checking",
            seen(State {
                config: Fetch::Loading,
                ..State::default()
            }),
        ),
        Fixture::new("first-run", seen(State::ready(Config::with(file, &[])))),
        Fixture::new("ready", seen(State::ready(all.clone()))),
        Fixture::new(
            "no-instance",
            seen(State {
                instance_exists: Some(false),
                ..State::ready(all.clone())
            }),
        ),
        Fixture::new(
            "bad-load-order",
            seen(State {
                load_order: Fetch::Failed(
                    "the load order could not be read: there is no mod list yet".into(),
                ),
                ..State::ready(all.clone())
            }),
        ),
        Fixture::new("ttw-ready", seen(State::ready(ttw))),
        Fixture {
            name: "error",
            page: Box::new(seen(State {
                config: Fetch::Failed(
                    "the configuration file has schema_version 2; this raul reads 1".into(),
                ),
                ..State::default()
            })),
            shell: ShellState {
                status: Some(UNREADABLE.detail.to_owned()),
                status_error: Some(UNREADABLE),
                ..ShellState::default()
            },
        },
        Fixture::new(
            "starting",
            seen(State {
                launch: Launch::Starting,
                ..State::ready(all.clone())
            }),
        ),
        Fixture::new(
            "started",
            seen(State {
                launch: Launch::Started,
                ..State::ready(all.clone())
            }),
        ),
        Fixture::new(
            "run-failed",
            seen(State {
                launch: Launch::Failed("nvmod run: refused: the game is already running".into()),
                ..State::ready(all)
            }),
        ),
        Fixture::new("long-text", seen(State::ready(long))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::Jobs;
    use crate::shell::App;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;
    use raul_ui_service::{Exec, Output, registry};
    use std::sync::{Arc, Mutex};

    const RUN_FAILED_PROBE: &str = "the game is not running";

    #[derive(Default)]
    struct Recording {
        calls: Mutex<Vec<Vec<String>>>,
        /// What every call answers; `None` is a success.
        answer: Mutex<Option<Result<Output, String>>>,
    }

    impl Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            if let Some(answer) = self.answer.lock().unwrap().clone() {
                return answer;
            }
            Ok(Output {
                code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

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

    #[test]
    fn play_runs_the_game_through_raul_run_or_says_it_cannot_yet() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("ready", exec.clone());
        h.run();
        h.get_by_label(PLAY_BIG).click();
        for _ in 0..100 {
            h.run_steps(2);
            if !exec.calls.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let waiting = registry::command("run").is_some_and(|c| c.awaits.is_some());
        let calls = exec.calls.lock().unwrap().clone();
        if waiting {
            assert!(calls.is_empty(), "a waiting command is never started");
            assert!(
                h.query_by_label(super::super::settings::WAITING.what)
                    .is_some()
            );
        } else {
            assert_eq!(
                calls[0],
                [
                    "run",
                    "nv-rs",
                    "--instance",
                    "/users/player/raul-storage/instance",
                    "--json"
                ]
            );
        }
    }

    fn click_stop(name_answer: Option<Result<Output, String>>) -> (Vec<Vec<String>>, bool, bool) {
        let exec = Arc::new(Recording::default());
        *exec.answer.lock().unwrap() = name_answer;
        let mut h = harness("started", exec.clone());
        h.run();
        h.get_by_label(STOP_GAME).click();
        for _ in 0..100 {
            h.run_steps(2);
            if !exec.calls.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // The answer arrives on a worker thread: wait for the page to show it rather than
        // assuming a fixed number of frames is enough on a slow machine.
        for _ in 0..200 {
            h.run_steps(2);
            if h.query_by_label(STOP_ASKED_STATUS).is_some()
                || h.query_by_label(RUN_FAILED_PROBE).is_some()
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let calls = exec.calls.lock().unwrap().clone();
        let asked = h.query_by_label(STOP_ASKED_STATUS).is_some();
        let failed = h.query_by_label(RUN_FAILED_PROBE).is_some();
        (calls, asked, failed)
    }

    #[test]
    fn stop_asks_the_running_game_to_stop_through_raul_run_stop() {
        let (calls, asked, _) = click_stop(None);
        let waiting = registry::command("run-stop").is_some_and(|c| c.awaits.is_some());
        if !waiting {
            assert_eq!(
                calls[0],
                [
                    "run",
                    "--stop",
                    "--instance",
                    "/users/player/raul-storage/instance",
                    "--json"
                ]
            );
            assert!(asked, "the status says the game was asked to stop");
        }
    }

    #[test]
    fn a_refused_or_failed_stop_is_shown_as_a_failed_launch() {
        let refused = Output {
            code: Some(1),
            stdout: String::new(),
            stderr: RUN_FAILED_PROBE.to_owned(),
        };
        let (_, asked, _) = click_stop(Some(Ok(refused)));
        assert!(!asked);
        let (_, asked, _) = click_stop(Some(Err(RUN_FAILED_PROBE.to_owned())));
        assert!(!asked);
    }

    #[test]
    fn a_blocked_play_explains_and_its_fix_opens_the_owning_page() {
        let exec = Arc::new(Recording::default());
        let mut h = harness("first-run", exec.clone());
        h.run();
        h.get_by_label(PLAY_BIG).click();
        h.run();
        let dialog = h.state().dialogs().current().cloned().unwrap();
        assert_eq!(dialog.error, Some(msg::PLAY_NO_GAME));
        assert_eq!(dialog.body[0], msg::PLAY_NO_GAME.detail);
        // The error is an error surface of its own: it can be copied from the dialog.
        assert!(h.query_all_by_label(msg::COPY_DETAILS).next().is_some());
        h.get_all_by_label(FIX).last().unwrap().click();
        h.run_steps(4);
        assert_eq!(h.state().view(), View::Page(PageId::Settings));
        assert!(
            exec.calls.lock().unwrap().iter().all(|c| c[0] != "run"),
            "the game was not started"
        );
    }
}
