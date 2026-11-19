// SPDX-License-Identifier: GPL-3.0-only
//! The Mods page (UI-RULES P1, P3, A3, T4, T7): the Profile's Mods in priority order, with a search and a filter,
//! and for the selected Mod: enable or disable, move one place up or down, and remove (asked once, with its cost).
//!
//! A generated LOD Mod gets its own treatment, as the TTW bundle row does (P4): it is tagged Generated, it has
//! Regenerate (on the LOD page) instead of Remove and Update, it is never checked for updates and its file conflicts
//! are expected, and one line says so. Every change is one command; the list is read again after it.

use super::home::{Mark, icon};
use super::settings::{Pending, Visits, error_block, failure_of, start, waiting};
use super::{Fixture, Page, PageCx, ShellState};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::pages::mods::{
    Filter, GeneratedList, GeneratedRow, ModEntry, Phase, Problem, State, disable_command,
    enable_command, generated_command, list_command, move_command, parse_generated, parse_list,
    remove_command,
};
use raul_ui_model::pages::settings::Command;
use raul_ui_model::pages::{PageId, View};
use raul_ui_service::Output;

crate::texts!(TEXTS {
    STATE_NO_INSTANCE = "No Instance yet";
    STATE_LOADING = "Reading the mod list";
    STATE_EMPTY = "No Mods";
    STATE_LISTED = "{n} Mods, {e} enabled";
    STATE_LISTED_ONE = "1 Mod, {e} enabled";
    STATE_CHANGING = "Changing the mod list";
    NO_INSTANCE = "Mods belong to the Instance, which lives in the storage folder. Set the storage folder in Settings first.";
    OPEN_SETTINGS = "Settings";
    EMPTY = "Your mod list has no Mods yet.";
    READING = "Reading the mod list";
    SEARCH_HINT = "Search Mods";
    FILTER_ALL = "All";
    FILTER_ENABLED = "Enabled";
    FILTER_DISABLED = "Disabled";
    SHOWING = "Showing {n} of {m}";
    CLEAR = "Clear";
    NO_MATCH = "No Mod matches the search and the filter.";
    SECTION_MODS = "Mods";
    SECTION_SELECTED = "Selected Mod";
    SELECT_HINT = "Select a Mod to change it.";
    ENABLE_HOVER = "Turns the Mod on or off.";
    TAG_GENERATED = "Generated";
    TAG_SEPARATOR = "Separator";
    TAG_MISSING = "Folder missing";
    TAG_GAME = "Managed by the game";
    WORD_ENABLED = "Enabled";
    WORD_DISABLED = "Disabled";
    PRIORITY = "Priority {n} of {m}";
    ENABLE = "Enable";
    DISABLE = "Disable";
    MOVE_UP = "Move up";
    MOVE_DOWN = "Move down";
    MOVE_HOVER = "A higher place wins file conflicts with the Mods below it.";
    REMOVE = "Remove…";
    REMOVE_TITLE = "Remove this Mod?";
    REMOVE_COST = "{name} leaves your mod list: its files move to the Instance's trash and its plugins leave the Load order. Getting it back means installing it again.";
    REMOVE_ACTION = "Remove";
    GENERATED_NOTE = "Made by LOD generation. It is not checked for updates, and its file conflicts are expected: it is meant to win them. A new run replaces it and keeps its place and whether it is on.";
    GENERATED_LEGACY = "Made by an earlier LOD run. The next run puts a new LOD Mod in its place.";
    GENERATED_VERSION = "Generator {v}";
    REGENERATE = "Regenerate…";
    REGENERATE_HOVER = "Opens the LOD page, where a new run replaces this Mod.";
    OPEN_LOD = "Open the LOD page";
    ENABLED_STATUS = "Mod enabled.";
    DISABLED_STATUS = "Mod disabled.";
    MOVED_STATUS = "Mod moved.";
    REMOVED_STATUS = "Mod removed.";
    UNREADABLE = "The command finished, but what it printed could not be read.";
});

crate::errors!(ERRORS {
    LIST_FAILED = (
        "The mod list could not be read",
        "Nothing changed. The line below says why."
    );
    GENERATED_FAILED = (
        "Generated LOD Mods could not be told apart",
        "Nothing changed. They show as plain Mods until the list is read again. The line below says why."
    );
    CHANGE_FAILED = (
        "The mod list was not changed",
        "Nothing changed. The line below says why."
    );
    REMOVE_FAILED = (
        "The Mod was not removed",
        "Nothing changed. The line below says why."
    );
});

const REMOVE_DIALOG: &str = "mods-remove";

/// The width from which the selected Mod shows beside the list instead of under it (UI-RULES A3).
const TWO_PANES: f32 = 860.0;

/// The Mods page.
#[derive(Debug, Default)]
pub struct Mods {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
    /// The Mod the open Remove dialog names.
    asked: Option<String>,
    /// The Mod the running change is for, and the place a move takes it to.
    target: Option<(String, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    List,
    Generated,
    Enable,
    Disable,
    Move,
    Remove,
}

impl Mods {
    /// The page in `state`, as a fixture shows it; it reads nothing by itself until its Instance changes.
    pub fn with(state: State) -> Mods {
        Mods {
            state,
            ..Mods::default()
        }
    }

    fn command(&self, job: Job) -> Option<Command> {
        let i = self.state.instance.as_deref()?;
        let (name, to) = match (&self.target, job) {
            (_, Job::List | Job::Generated) => ("", 0),
            (Some((name, to)), _) => (name.as_str(), *to),
            (None, _) => return None,
        };
        Some(match job {
            Job::List => list_command(i),
            Job::Generated => generated_command(i),
            Job::Enable => enable_command(i, name),
            Job::Disable => disable_command(i, name),
            Job::Move => move_command(i, name, to),
            Job::Remove => remove_command(i, name),
        })
    }

    /// Starts `job` for the Mod `name` (and, for a move, the place `to`).
    fn change(&mut self, cx: &mut PageCx<'_>, job: Job, name: String, to: usize) {
        if self.state.busy() {
            return;
        }
        self.target = Some((name, to));
        self.run(cx, job);
    }

    /// Starts the command of `job`, after the state moved to the step it runs.
    fn run(&mut self, cx: &mut PageCx<'_>, job: Job) {
        let Some(command) = self.command(job) else {
            return;
        };
        match job {
            Job::List => self.state.loading(),
            Job::Generated => {}
            _ => self.state.changing(),
        }
        match start(cx, &command) {
            Ok(id) => self.pending.add(job, id),
            Err(e) => self.state.failed(Problem {
                id: e.id,
                cause: e.what.to_owned(),
                command: Some(command.shown()),
            }),
        }
    }

    /// Reads the list and which of its Mods are generated LOD.
    fn read(&mut self, cx: &mut PageCx<'_>) {
        self.run(cx, Job::List);
        self.run(cx, Job::Generated);
    }

    /// Follows the Instance the shell knows: a new one is read, none clears the page. A command that runs for the
    /// old one ends first.
    fn follow_instance(&mut self, cx: &mut PageCx<'_>) {
        if cx.shell.instance == self.state.instance || self.state.busy() {
            return;
        }
        self.state = State {
            instance: cx.shell.instance.clone(),
            ..State::default()
        };
        self.read(cx);
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            let command = self.command(job).map(|c| c.shown());
            let problem = |e: &ErrorText, cause: String| Problem {
                id: e.id,
                cause,
                command: command.clone(),
            };
            match job {
                Job::List => match answer(&result).and_then(|out| parse_list(&out.stdout)) {
                    Ok((profile, mods)) => self.state.listed(profile, mods),
                    Err(why) => self.state.failed(problem(&LIST_FAILED, why)),
                },
                Job::Generated => {
                    match answer(&result).and_then(|out| parse_generated(&out.stdout)) {
                        Ok(g) => self.state.mark_generated(g),
                        Err(why) => {
                            self.state.mark_generated(GeneratedList::default());
                            self.state.problem = Some(problem(&GENERATED_FAILED, why));
                        }
                    }
                }
                Job::Enable | Job::Disable | Job::Move | Job::Remove => {
                    let failed = if job == Job::Remove {
                        &REMOVE_FAILED
                    } else {
                        &CHANGE_FAILED
                    };
                    match answer(&result) {
                        Ok(_) => {
                            cx.shell.status = Some(
                                match job {
                                    Job::Enable => ENABLED_STATUS,
                                    Job::Disable => DISABLED_STATUS,
                                    Job::Move => MOVED_STATUS,
                                    _ => REMOVED_STATUS,
                                }
                                .to_owned(),
                            );
                            self.read(cx);
                        }
                        Err(why) => self.state.failed(problem(failed, why)),
                    }
                }
            }
        }
    }

    fn ask_remove(&mut self, cx: &mut PageCx<'_>, instance: &str, name: &str) {
        self.asked = Some(name.to_owned());
        cx.dialogs.open(
            Dialog::confirm(
                REMOVE_DIALOG,
                REMOVE_TITLE,
                REMOVE_COST.replace("{name}", name),
                REMOVE_ACTION,
            )
            .with_command(remove_command(instance, name).shown())
            .danger(),
        );
    }

    /// The search, the filter, and what they hide (UI-RULES T7).
    fn search(&mut self, ui: &mut egui::Ui, t: &Tokens) {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.state.query)
                    .hint_text(SEARCH_HINT)
                    .desired_width(180.0),
            );
            for f in Filter::ALL {
                let label = match f {
                    Filter::All => FILTER_ALL,
                    Filter::Enabled => FILTER_ENABLED,
                    Filter::Disabled => FILTER_DISABLED,
                };
                if ui.selectable_label(self.state.filter == f, label).clicked() {
                    self.state.filter = f;
                }
            }
            if self.state.is_filtered() {
                let shown = SHOWING
                    .replace("{n}", &self.state.visible_mods().to_string())
                    .replace("{m}", &self.state.mod_count().to_string());
                ui.label(RichText::new(shown).color(t.muted.color()));
                if ui.button(CLEAR).clicked() {
                    self.state.clear_filter();
                }
            }
        });
    }

    /// The list: whether each Mod is on, its place and its name, tagged when it is generated, a separator, missing
    /// or the game's own. A click selects; the box turns a Mod on or off at once (reversible, so it does not ask).
    fn list(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, t: &Tokens) {
        let rows: Vec<ModEntry> = self.state.visible().into_iter().cloned().collect();
        if rows.is_empty() {
            ui.label(RichText::new(NO_MATCH).color(t.muted.color()));
            return;
        }
        let busy = self.state.busy();
        let name_width = (ui.available_width() - 180.0).clamp(64.0, 420.0);
        let mut toggled = None;
        egui::Grid::new("mods-list")
            .num_columns(4)
            .spacing([10.0, 8.0])
            .min_row_height(28.0)
            .show(ui, |ui| {
                for m in &rows {
                    ui.label(RichText::new(m.position.to_string()).color(t.muted.color()));
                    if m.changeable() {
                        let mut on = m.enabled;
                        let r = ui
                            .add_enabled(!busy, egui::Checkbox::without_text(&mut on))
                            .on_hover_text(ENABLE_HOVER);
                        if r.changed() {
                            toggled = Some((m.name.clone(), on));
                        }
                    } else {
                        ui.label("");
                    }
                    let selected = self.state.selected.as_deref() == Some(m.name.as_str());
                    let text = if m.separator {
                        RichText::new(&m.name).color(t.muted.color()).strong()
                    } else if m.enabled {
                        RichText::new(&m.name).color(t.text.color())
                    } else {
                        RichText::new(&m.name).color(t.muted.color())
                    };
                    ui.allocate_ui_with_layout(
                        egui::vec2(name_width, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_width(name_width);
                            let r = ui
                                .add(egui::Button::selectable(selected, text).truncate())
                                .on_hover_text(&m.name);
                            if r.clicked() {
                                self.state.select(&m.name);
                            }
                        },
                    );
                    tag(ui, t, m);
                    ui.end_row();
                }
            });
        if let Some((name, on)) = toggled {
            let job = if on { Job::Enable } else { Job::Disable };
            self.change(cx, job, name, 0);
        }
    }

    /// What can be done with the selected Mod.
    fn detail(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, t: &Tokens, instance: &str) {
        shell::section_label(ui, t, SECTION_SELECTED, true);
        let Some(m) = self.state.selected().cloned() else {
            ui.label(RichText::new(SELECT_HINT).color(t.muted.color()));
            return;
        };
        ui.add(egui::Label::new(RichText::new(&m.name).strong()).truncate())
            .on_hover_text(&m.name);
        ui.horizontal_wrapped(|ui| {
            if m.enabled {
                super::home::mark(ui, t, Mark::Ok, WORD_ENABLED);
            } else {
                super::home::mark(ui, t, Mark::Unset, WORD_DISABLED);
            }
            let priority = PRIORITY
                .replace("{n}", &m.position.to_string())
                .replace("{m}", &self.state.mods.len().to_string());
            ui.label(RichText::new(priority).color(t.muted.color()));
        });
        ui.add_space(8.0);
        if !m.changeable() {
            return;
        }
        let busy = self.state.busy();
        let mut next = None;
        ui.horizontal_wrapped(|ui| {
            let toggle = if m.enabled { DISABLE } else { ENABLE };
            if ui.add_enabled(!busy, egui::Button::new(toggle)).clicked() {
                next = Some((if m.enabled { Job::Disable } else { Job::Enable }, 0));
            }
            for (label, up) in [(MOVE_UP, true), (MOVE_DOWN, false)] {
                let to = self.state.step(&m.name, up);
                if ui
                    .add_enabled(!busy && to.is_some(), egui::Button::new(label))
                    .on_hover_text(MOVE_HOVER)
                    .clicked()
                    && let Some(to) = to
                {
                    next = Some((Job::Move, to));
                }
            }
        });
        if let Some((job, to)) = next {
            self.change(cx, job, m.name.clone(), to);
        }
        ui.add_space(8.0);
        match &m.generated {
            Some(g) => {
                // The staleness chip of raul#66 goes here, beside Regenerate, once `g.stale` is filled.
                ui.horizontal_top(|ui| {
                    icon(ui, t, Mark::Info);
                    ui.add(
                        egui::Label::new(
                            RichText::new(if g.legacy {
                                GENERATED_LEGACY
                            } else {
                                GENERATED_NOTE
                            })
                            .color(t.muted.color()),
                        )
                        .wrap(),
                    );
                });
                if let Some(v) = &g.version {
                    ui.label(
                        RichText::new(GENERATED_VERSION.replace("{v}", v)).color(t.muted.color()),
                    );
                }
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let regenerate = ui
                        .button(REGENERATE)
                        .on_hover_text(REGENERATE_HOVER)
                        .clicked();
                    let link = ui.link(OPEN_LOD).clicked();
                    if regenerate || link {
                        cx.shell.go_to = Some(View::Page(PageId::Lod));
                    }
                });
            }
            None => {
                if ui
                    .add_enabled(!busy, shell::danger_button(t, REMOVE))
                    .clicked()
                {
                    self.ask_remove(cx, instance, &m.name);
                }
            }
        }
    }
}

/// The tag after a name: Generated, Separator, Folder missing, or Managed by the game.
fn tag(ui: &mut egui::Ui, t: &Tokens, m: &ModEntry) {
    let (text, colour) = if m.generated.is_some() {
        (TAG_GENERATED, t.info.color())
    } else if m.separator {
        (TAG_SEPARATOR, t.muted.color())
    } else if !m.installed {
        (TAG_MISSING, t.warn.color())
    } else if m.foreign {
        (TAG_GAME, t.muted.color())
    } else {
        ui.label("");
        return;
    };
    ui.label(RichText::new(text).small().color(colour));
}

/// The output of a command that ran and exited 0, or why there is none.
fn answer(result: &Result<Output, String>) -> Result<&Output, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    if out.code == Some(0) {
        Ok(out)
    } else {
        Err(failure_of(out))
    }
}

impl Page for Mods {
    fn view(&self) -> View {
        View::Page(PageId::Mods)
    }

    fn title(&self) -> &'static str {
        msg::PAGE_MODS
    }

    fn state_line(&self) -> String {
        let s = &self.state;
        match s.phase {
            Phase::NoInstance => STATE_NO_INSTANCE.to_owned(),
            Phase::Loading => STATE_LOADING.to_owned(),
            Phase::Changing => STATE_CHANGING.to_owned(),
            Phase::Listed => match s.mod_count() {
                0 => STATE_EMPTY.to_owned(),
                1 => STATE_LISTED_ONE.replace("{e}", &s.enabled_count().to_string()),
                n => STATE_LISTED
                    .replace("{n}", &n.to_string())
                    .replace("{e}", &s.enabled_count().to_string()),
            },
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let back = self.visits.returned(ui.ctx());
        self.follow_instance(cx);
        if back && self.state.phase == Phase::Listed {
            self.read(cx);
        }
        self.take_answers(cx);
        // The answer counts only for the Mod the dialog named, and only while nothing runs.
        if let Some(answer) = cx.dialogs.take_answer(REMOVE_DIALOG) {
            let asked = self.asked.take();
            if answer == Answer::Confirmed
                && self.state.phase == Phase::Listed
                && let Some(name) = asked
                && self
                    .state
                    .mods
                    .iter()
                    .any(|m| m.name == name && m.generated.is_none())
            {
                self.change(cx, Job::Remove, name, 0);
            }
        }
        let Some(instance) = self.state.instance.clone() else {
            ui.label(NO_INSTANCE);
            ui.add_space(8.0);
            if ui.button(OPEN_SETTINGS).clicked() {
                cx.shell.go_to = Some(View::Page(PageId::Settings));
            }
            return;
        };
        if let Some(p) = &self.state.problem {
            let e = ERRORS
                .iter()
                .chain(msg::SHELL_ERRORS)
                .chain(super::settings::ERRORS)
                .find(|e| e.id == p.id)
                .copied()
                .unwrap_or(LIST_FAILED);
            error_block(ui, &t, &e, Some(&p.cause));
            ui.add_space(12.0);
        }
        if self.state.phase == Phase::Loading && self.state.mods.is_empty() {
            waiting(ui, &t, READING, &list_command(&instance));
            return;
        }
        if self.state.mods.is_empty() {
            ui.label(RichText::new(EMPTY).color(t.muted.color()));
            return;
        }
        let wide = ui.available_width() >= TWO_PANES;
        if wide {
            ui.columns(2, |cols| {
                shell::section_label(&mut cols[0], &t, SECTION_MODS, true);
                self.search(&mut cols[0], &t);
                cols[0].add_space(8.0);
                self.list(&mut cols[0], cx, &t);
                self.detail(&mut cols[1], cx, &t, &instance);
            });
        } else {
            shell::section_label(ui, &t, SECTION_MODS, true);
            self.search(ui, &t);
            ui.add_space(8.0);
            self.list(ui, cx, &t);
            ui.add_space(16.0);
            self.detail(ui, cx, &t, &instance);
        }
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

const INSTANCE: &str = "/users/player/raul-storage/instance";

const NAMES: &[&str] = &[
    "Sample Texture Pack",
    "Sample Weapons",
    "Sample Interface",
    "Sample Lighting",
    "Sample Radio",
];

fn fixture(name: &'static str, state: State) -> Fixture {
    Fixture {
        name,
        page: Box::new(Mods::with(State {
            instance: Some(INSTANCE.to_owned()),
            ..state
        })),
        shell: ShellState {
            instance: Some(INSTANCE.to_owned()),
            new_vegas: Some("/games/steamapps/common/Fallout New Vegas".to_owned()),
            ..ShellState::default()
        },
    }
}

fn entries(names: &[&str]) -> Vec<ModEntry> {
    names
        .iter()
        .enumerate()
        .map(|(i, n)| ModEntry::named(*n, i + 1))
        .collect()
}

/// A Profile with a generated LOD Mod at place 2, one disabled Mod and one whose folder is missing.
fn listed() -> State {
    let mut mods = entries(NAMES);
    mods.insert(1, ModEntry::named("Generated LOD (Default)", 2));
    for (i, m) in mods.iter_mut().enumerate() {
        m.position = i + 1;
    }
    mods[3].enabled = false;
    mods[5].installed = false;
    let mut s = State::default();
    s.listed("Default".to_owned(), mods);
    s.mark_generated(GeneratedList {
        own_name: "Generated LOD (Default)".to_owned(),
        mods: vec![GeneratedRow {
            name: "Generated LOD (Default)".to_owned(),
            own: true,
            legacy: false,
            lodwright_version: Some("0.3.0".to_owned()),
        }],
    });
    s
}

/// The states the snapshot tests render (UI-RULES S1).
pub fn fixtures() -> Vec<Fixture> {
    let mut empty = State::default();
    empty.listed("Default".to_owned(), Vec::new());
    let mut selected = listed();
    selected.select("Sample Weapons");
    let mut generated = listed();
    generated.select("Generated LOD (Default)");
    let mut filtered = listed();
    filtered.filter = Filter::Enabled;
    "sample".clone_into(&mut filtered.query);
    let mut changing = selected.clone();
    changing.changing();
    let mut failed = selected.clone();
    failed.failed(Problem {
        id: REMOVE_FAILED.id,
        cause: "active plugins of other Mods need its plugins".to_owned(),
        command: Some(remove_command(INSTANCE, "Sample Weapons").shown()),
    });
    let long: Vec<String> = NAMES
        .iter()
        .map(|n| format!("A Very Long Synthetic Mod Name That Keeps Going On - {n}"))
        .collect();
    let long: Vec<&str> = long.iter().map(String::as_str).collect();
    let mut long_text = State::default();
    long_text.listed("Default".to_owned(), entries(&long));
    long_text.select(long[0]);
    let large: Vec<String> = (1..=60).map(|i| format!("Sample Mod {i:02}")).collect();
    let large: Vec<&str> = large.iter().map(String::as_str).collect();
    let mut large_list = State::default();
    large_list.listed("Default".to_owned(), entries(&large));
    vec![
        Fixture {
            name: "no-instance",
            page: Box::new(Mods::with(State::default())),
            shell: ShellState::default(),
        },
        fixture(
            "loading",
            State {
                phase: Phase::Loading,
                ..State::default()
            },
        ),
        fixture("empty", empty),
        fixture("listed", selected),
        fixture("generated", generated),
        fixture("filtered", filtered),
        fixture("changing", changing),
        fixture("error", failed),
        fixture("long-text", long_text),
        fixture("large-list", large_list),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_counts_the_mods_and_the_enabled_ones() {
        let page = Mods::with(listed());
        assert_eq!(page.state_line(), "6 Mods, 5 enabled");
        let mut one = State::default();
        one.listed("Default".into(), entries(&["Only"]));
        assert_eq!(Mods::with(one).state_line(), "1 Mod, 1 enabled");
        assert_eq!(Mods::default().state_line(), STATE_NO_INSTANCE);
    }

    #[test]
    fn only_a_command_that_exited_0_is_an_answer() {
        let ok: Result<Output, String> = Ok(Output {
            code: Some(0),
            stdout: "{}".into(),
            stderr: String::new(),
        });
        assert!(answer(&ok).is_ok());
        let refused: Result<Output, String> = Ok(Output {
            code: Some(2),
            stdout: String::new(),
            stderr: "raul mods remove: refused: needed by others\n".into(),
        });
        assert!(answer(&refused).unwrap_err().contains("needed by others"));
        assert_eq!(answer(&Err("no raul".into())).unwrap_err(), "no raul");
    }
}
