// SPDX-License-Identifier: GPL-3.0-only
//! The Load order page (UI-RULES P3, T4, T5, G6): the Profile's plugins, a sort preview with the moves it makes and
//! each plugin's group and messages, Apply, and Undo.
//!
//! Every step is a `raul` command from the registry, shown on the page: `raul plugins list`, `raul sort
//! --fetch-masterlist` for the preview (nothing is written), the same with `--apply`, and `raul rollback` for Undo.
//! A sort is reversible, so Apply does not ask; Undo is offered until the page reads another order.

use super::home::{Mark, icon};
use super::settings::{Pending, Visits, error_block, failure_of, start, waiting};
use super::{Fixture, Page, PageCx, ShellState};
use crate::catalogue::{self as msg, ErrorText};
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::pages::load_order::{
    Message, MessageKind, Phase, Plugin, Problem, State, apply_command, list_command,
    preview_command, undo_command,
};
use raul_ui_model::pages::settings::Command;
use raul_ui_model::pages::{Attention, PageId, Severity, View};
use raul_ui_service::Output;
use raul_ui_service::conflicts::{PluginRow, PluginsReport};
use raul_ui_service::dto::{RollbackReport, SortMessage, SortReport};

crate::texts!(TEXTS {
    STATE_NO_INSTANCE = "No Instance yet";
    STATE_LOADING = "Reading the plugins";
    STATE_LISTED = "{n} plugins, {a} active";
    STATE_EMPTY = "No plugins";
    STATE_SORTING = "Sorting with the masterlist";
    STATE_PREVIEW = "Preview · {n} moves · nothing changes until Apply";
    STATE_PREVIEW_ONE = "Preview · 1 move · nothing changes until Apply";
    STATE_SORTED = "Already sorted";
    STATE_APPLYING = "Applying the sorted order";
    STATE_APPLIED = "Applied · Undo restores the old order";
    STATE_UNDOING = "Restoring the old order";
    SORT = "Sort";
    APPLY = "Apply {n} moves";
    APPLY_ONE = "Apply 1 move";
    CANCEL = "Cancel";
    UNDO = "Undo";
    NO_INSTANCE = "The Load order belongs to the Instance, which lives in the storage folder. Set the storage folder in Settings first.";
    OPEN_SETTINGS = "Settings";
    EMPTY = "The Profile has no plugins yet.";
    PREVIEW_NOTE = "Preview: {n} moves pending";
    PREVIEW_NOTE_ONE = "Preview: 1 move pending";
    PREVIEW_DETAIL = "Nothing changes until Apply. Undo stays available after.";
    SORTED_NOTE = "The plugins are already in the masterlist's order.";
    APPLIED_NOTE = "The sorted order is written. Undo puts the old one back.";
    COL_POSITION = "#";
    COL_PLUGIN = "Plugin";
    COL_MOVE = "Move";
    COL_GROUP = "Group";
    COL_MESSAGES = "Messages";
    INACTIVE = "{name} · inactive";
    NONE = "—";
    MOVE_FROM_TO = "{from} → {to}";
    MORE_MESSAGES = "{n} more";
    SECTION_PLUGINS = "Plugins";
    SECTION_LIST = "About the list";
    APPLIED_STATUS = "Load order applied. Undo restores the old one.";
    NOTHING_STATUS = "Nothing to change: the order was already sorted.";
    UNDONE_STATUS = "Previous order restored.";
    READING = "Reading the plugin list";
    UNREADABLE = "The command finished, but what it printed could not be read.";
    SORTING = "Sorting: the pinned masterlist is fetched and checked first";
});

crate::errors!(ERRORS {
    LIST_FAILED = (
        "The plugins could not be read",
        "Nothing changed. The line below says why."
    );
    SORT_FAILED = (
        "The sort could not be computed",
        "Nothing changed. The line below says why."
    );
    APPLY_FAILED = (
        "The sorted order was not written",
        "What it changed was rolled back. The line below says why."
    );
    UNDO_FAILED = (
        "The old order could not be restored",
        "The sorted order is still in place. The line below says why."
    );
    UNDO_OTHER = (
        "Undo took back another change",
        "The last change of the Instance was not this sort. Run the Doctor and check the Load order."
    );
});

/// The Load order page.
#[derive(Debug, Default)]
pub struct LoadOrder {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    List,
    Preview,
    Apply,
    Undo,
}

impl LoadOrder {
    /// The page in `state`, as a fixture shows it; it reads nothing by itself until its Instance changes.
    pub fn with(state: State) -> LoadOrder {
        LoadOrder {
            state,
            ..LoadOrder::default()
        }
    }

    fn command(&self, job: Job) -> Option<Command> {
        let i = self.state.instance.as_deref()?;
        Some(match job {
            Job::List => list_command(i),
            Job::Preview => preview_command(i),
            Job::Apply => apply_command(i),
            Job::Undo => undo_command(i),
        })
    }

    /// Starts the command of `job`, after the state moved to the step it runs.
    fn run(&mut self, cx: &mut PageCx<'_>, job: Job) {
        let Some(command) = self.command(job) else {
            return;
        };
        match job {
            Job::List => self.state.loading(),
            Job::Preview => self.state.sorting(),
            Job::Apply => self.state.applying(),
            Job::Undo => self.state.undoing(),
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
        self.run(cx, Job::List);
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
                Job::List => match parse(&result, serde_json::from_value::<PluginsReport>) {
                    Ok(r) => self
                        .state
                        .listed(r.profile, r.plugins.iter().map(plugin).collect()),
                    Err(why) => self.state.failed(problem(&LIST_FAILED, why)),
                },
                Job::Preview => match parse(&result, serde_json::from_value::<SortReport>) {
                    Ok(r) => {
                        let (proposed, notes) = preview_of(&r);
                        self.state.previewed(proposed, notes);
                    }
                    Err(why) => self.state.failed(problem(&SORT_FAILED, why)),
                },
                Job::Apply => match parse(&result, serde_json::from_value::<SortReport>) {
                    Ok(r) => {
                        let journal = r.applied.and_then(|a| a.journal);
                        cx.shell.status = Some(
                            if journal.is_some() {
                                APPLIED_STATUS
                            } else {
                                NOTHING_STATUS
                            }
                            .to_owned(),
                        );
                        self.state.applied(journal);
                        // Read back what was written.
                        self.run(cx, Job::List);
                    }
                    Err(why) => self.state.failed(problem(&APPLY_FAILED, why)),
                },
                Job::Undo => match parse(&result, serde_json::from_value::<RollbackReport>) {
                    Ok(r) => {
                        let ours = r.kind == "sort";
                        self.state.undone();
                        self.run(cx, Job::List);
                        if ours {
                            cx.shell.status = Some(UNDONE_STATUS.to_owned());
                        } else {
                            self.state.problem = Some(problem(&UNDO_OTHER, r.kind));
                        }
                    }
                    Err(why) => self.state.failed(problem(&UNDO_FAILED, why)),
                },
            }
        }
    }

    fn table(&self, ui: &mut egui::Ui, t: &Tokens) {
        let preview = matches!(self.state.phase, Phase::Preview | Phase::Applying);
        let rows: &[Plugin] = if preview {
            &self.state.proposed
        } else {
            &self.state.current
        };
        let pending = self.state.moves();
        let header = |ui: &mut egui::Ui, text: &str| {
            ui.label(RichText::new(text).color(t.muted.color()).small());
        };
        // The plugin column takes what the others leave, so a long name truncates instead of scrolling sideways.
        let plugin_width = (ui.available_width() * 0.36).clamp(120.0, 420.0);
        egui::Grid::new("load-order-plugins")
            .num_columns(if preview { 5 } else { 4 })
            .spacing([18.0, 10.0])
            .min_row_height(28.0)
            .show(ui, |ui| {
                header(ui, COL_POSITION);
                header(ui, COL_PLUGIN);
                if preview {
                    header(ui, COL_MOVE);
                }
                header(ui, COL_GROUP);
                header(ui, COL_MESSAGES);
                ui.end_row();
                for (i, p) in rows.iter().enumerate() {
                    let moved = pending.iter().find(|m| m.name == p.name);
                    ui.label(RichText::new((i + 1).to_string()).color(t.muted.color()));
                    let name = RichText::new(&p.name).monospace();
                    let name = if moved.is_some() {
                        name.strong().color(t.text.color())
                    } else if p.active {
                        name.color(t.text.color())
                    } else {
                        name.color(t.muted.color())
                    };
                    let hover = if p.active {
                        p.name.clone()
                    } else {
                        INACTIVE.replace("{name}", &p.name)
                    };
                    ui.allocate_ui_with_layout(
                        egui::vec2(plugin_width, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_width(plugin_width);
                            ui.add(egui::Label::new(name).truncate())
                                .on_hover_text(hover);
                        },
                    );
                    if preview {
                        match moved {
                            Some(m) => {
                                let colour = if m.up() { t.info } else { t.warn };
                                let arrow = MOVE_FROM_TO
                                    .replace("{from}", &m.from.to_string())
                                    .replace("{to}", &m.to.to_string());
                                ui.label(RichText::new(arrow).color(colour.color()));
                            }
                            None => {
                                ui.label(RichText::new(NONE).color(t.muted.color()));
                            }
                        }
                    }
                    ui.label(
                        RichText::new(p.group.as_deref().unwrap_or(NONE)).color(t.muted.color()),
                    );
                    messages(ui, t, &p.messages);
                    ui.end_row();
                }
            });
    }

    /// The note under the header and the command the next step runs (UI-RULES G6).
    fn step_note(&self, ui: &mut egui::Ui, t: &Tokens, instance: &str) -> bool {
        match self.state.phase {
            Phase::Loading if self.state.current.is_empty() => {
                waiting(ui, t, READING, &list_command(instance));
                return false;
            }
            Phase::Sorting => waiting(ui, t, SORTING, &preview_command(instance)),
            Phase::Applying => waiting(ui, t, STATE_APPLYING, &apply_command(instance)),
            Phase::Undoing => waiting(ui, t, STATE_UNDOING, &undo_command(instance)),
            Phase::Preview => match self.state.moves().len() {
                0 => note(ui, t, Mark::Ok, SORTED_NOTE, None),
                n => {
                    let head = if n == 1 {
                        PREVIEW_NOTE_ONE.to_owned()
                    } else {
                        PREVIEW_NOTE.replace("{n}", &n.to_string())
                    };
                    note(ui, t, Mark::Warning, &head, Some(PREVIEW_DETAIL));
                    shell::command_line(ui, t, &apply_command(instance).shown());
                }
            },
            Phase::Applied => {
                note(ui, t, Mark::Ok, APPLIED_NOTE, None);
                shell::command_line(ui, t, &undo_command(instance).shown());
            }
            // A problem already shows the command that failed.
            _ if self.state.problem.is_some() => return true,
            _ => shell::command_line(ui, t, &preview_command(instance).shown()),
        }
        ui.add_space(12.0);
        true
    }
}

impl Page for LoadOrder {
    fn view(&self) -> View {
        View::Page(PageId::LoadOrder)
    }

    fn title(&self) -> &'static str {
        msg::PAGE_LOAD_ORDER
    }

    fn state_line(&self) -> String {
        let s = &self.state;
        match s.phase {
            Phase::NoInstance => STATE_NO_INSTANCE.to_owned(),
            Phase::Loading => STATE_LOADING.to_owned(),
            Phase::Listed if s.current.is_empty() => STATE_EMPTY.to_owned(),
            Phase::Listed => STATE_LISTED
                .replace("{n}", &s.current.len().to_string())
                .replace("{a}", &s.active().to_string()),
            Phase::Sorting => STATE_SORTING.to_owned(),
            Phase::Preview => match s.moves().len() {
                0 => STATE_SORTED.to_owned(),
                1 => STATE_PREVIEW_ONE.to_owned(),
                n => STATE_PREVIEW.replace("{n}", &n.to_string()),
            },
            Phase::Applying => STATE_APPLYING.to_owned(),
            Phase::Applied => STATE_APPLIED.to_owned(),
            Phase::Undoing => STATE_UNDOING.to_owned(),
        }
    }

    fn attention(&self) -> Option<Attention> {
        let errors = self
            .state
            .current
            .iter()
            .filter(|p| p.worst() == Some(MessageKind::Error))
            .count();
        (errors > 0).then(|| Attention {
            severity: Severity::Warning,
            count: u32::try_from(errors).unwrap_or(u32::MAX),
        })
    }

    fn tools(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let Some(instance) = self.state.instance.clone() else {
            return;
        };
        // The layout runs right to left: the primary button comes first.
        match self.state.phase {
            Phase::Preview => {
                let n = self.state.moves().len();
                if n > 0 {
                    let label = if n == 1 {
                        APPLY_ONE.to_owned()
                    } else {
                        APPLY.replace("{n}", &n.to_string())
                    };
                    if ui
                        .add(shell::primary_button(&t, &label))
                        .on_hover_text(apply_command(&instance).shown())
                        .clicked()
                    {
                        self.run(cx, Job::Apply);
                    }
                }
                if ui.button(CANCEL).clicked() {
                    self.state.cancel_preview();
                }
            }
            Phase::Listed | Phase::Applied => {
                let sort = if self.state.current.is_empty() {
                    ui.add_enabled(false, egui::Button::new(SORT))
                } else {
                    ui.add(shell::primary_button(&t, SORT))
                };
                if sort
                    .on_hover_text(preview_command(&instance).shown())
                    .clicked()
                {
                    self.run(cx, Job::Preview);
                }
                if self.state.journal.is_some()
                    && ui
                        .button(UNDO)
                        .on_hover_text(undo_command(&instance).shown())
                        .clicked()
                {
                    self.run(cx, Job::Undo);
                }
            }
            _ => {}
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let back = self.visits.returned(ui.ctx());
        self.follow_instance(cx);
        if back && matches!(self.state.phase, Phase::Listed | Phase::Applied) {
            self.run(cx, Job::List);
        }
        self.take_answers(cx);
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
            if let Some(c) = &p.command {
                shell::command_line(ui, &t, c);
            }
            ui.add_space(12.0);
        }
        if !self.step_note(ui, &t, &instance) {
            return;
        }
        if !self.state.notes.is_empty() && self.state.phase == Phase::Preview {
            shell::section_label(ui, &t, SECTION_LIST, true);
            for m in &self.state.notes {
                ui.horizontal(|ui| {
                    icon(ui, &t, mark_of(m.kind));
                    ui.add(egui::Label::new(&m.text).wrap());
                });
            }
            ui.add_space(12.0);
        }
        shell::section_label(ui, &t, SECTION_PLUGINS, true);
        if self.state.current.is_empty() && self.state.proposed.is_empty() {
            ui.label(RichText::new(EMPTY).color(t.muted.color()));
        } else {
            self.table(ui, &t);
        }
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

/// The report a finished command printed, read by `read`, or why there is none: a command that could not start,
/// that did not exit 0, or whose output does not read.
fn parse<T, E>(
    result: &Result<Output, String>,
    read: impl Fn(serde_json::Value) -> Result<T, E>,
) -> Result<T, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    if out.code != Some(0) {
        return Err(failure_of(out));
    }
    serde_json::from_str(out.stdout.trim())
        .ok()
        .and_then(|v| read(v).ok())
        .ok_or_else(|| UNREADABLE.to_owned())
}

fn plugin(row: &PluginRow) -> Plugin {
    Plugin {
        name: row.name.clone(),
        active: row.active,
        ..Plugin::default()
    }
}

fn message(m: &SortMessage) -> Message {
    Message {
        kind: MessageKind::parse(&m.kind),
        text: m.text.clone(),
    }
}

/// The proposed order of a sort, with each plugin's group, messages and findings, and the notes about the whole
/// list: its general messages, the sorter's warnings and the plugins the Profile names that are not installed.
fn preview_of(r: &SortReport) -> (Vec<Plugin>, Vec<Message>) {
    let proposed = r
        .order
        .iter()
        .map(|row| {
            let mut messages: Vec<Message> = row
                .messages
                .iter()
                .chain(&row.generated_messages)
                .map(message)
                .collect();
            messages.extend(row.findings.iter().map(|f| Message {
                kind: if f.hard {
                    MessageKind::Error
                } else {
                    MessageKind::Warning
                },
                text: f.text.clone(),
            }));
            Plugin {
                name: row.name.clone(),
                active: row.active,
                group: row.group.clone(),
                messages,
            }
        })
        .collect();
    let mut notes: Vec<Message> = r.general_messages.iter().map(message).collect();
    let warn = |text: &String| Message {
        kind: MessageKind::Warning,
        text: text.clone(),
    };
    notes.extend(r.warnings.iter().map(warn));
    notes.extend(r.not_installed.iter().map(warn));
    (proposed, notes)
}

fn mark_of(kind: MessageKind) -> Mark {
    match kind {
        MessageKind::Note => Mark::Info,
        MessageKind::Warning => Mark::Warning,
        MessageKind::Error => Mark::Problem,
    }
}

/// The messages of one plugin: the most serious first, the rest counted, all of them in the tooltip.
fn messages(ui: &mut egui::Ui, t: &Tokens, list: &[Message]) {
    let Some(first) = list.iter().max_by_key(|m| m.kind) else {
        ui.label("");
        return;
    };
    let all = list
        .iter()
        .map(|m| m.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        icon(ui, t, mark_of(first.kind));
        ui.add(egui::Label::new(RichText::new(&first.text).color(t.text.color())).truncate())
            .on_hover_text(all);
        if list.len() > 1 {
            ui.label(
                RichText::new(MORE_MESSAGES.replace("{n}", &(list.len() - 1).to_string()))
                    .color(t.muted.color()),
            );
        }
    });
}

/// A note under the header: a mark, a line and an optional plain detail.
fn note(ui: &mut egui::Ui, t: &Tokens, m: Mark, line: &str, detail: Option<&str>) {
    ui.horizontal_top(|ui| {
        icon(ui, t, m);
        ui.vertical(|ui| {
            ui.label(RichText::new(line).color(t.text.color()));
            if let Some(d) = detail {
                ui.label(RichText::new(d).color(t.muted.color()));
            }
        });
    });
}

const INSTANCE: &str = "/users/player/raul-storage/instance";

const CURRENT: &[&str] = &[
    "BaseGame.esm",
    "BaseDLC_A.esm",
    "SampleQuestPack.esp",
    "SampleLighting.esp",
    "SampleInterior.esp",
    "SampleWeaponsA.esp",
    "SampleRadio.esp",
];

const SORTED: &[&str] = &[
    "BaseGame.esm",
    "BaseDLC_A.esm",
    "SampleWeaponsA.esp",
    "SampleQuestPack.esp",
    "SampleInterior.esp",
    "SampleLighting.esp",
    "SampleRadio.esp",
];

fn sample(names: &[&str]) -> Vec<Plugin> {
    names.iter().map(|n| Plugin::named(*n)).collect()
}

fn fixture(name: &'static str, state: State) -> Fixture {
    Fixture {
        name,
        page: Box::new(LoadOrder::with(State {
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

fn listed(names: &[&str]) -> State {
    let mut s = State::default();
    s.listed("Default".to_owned(), sample(names));
    if let Some(last) = s.current.last_mut() {
        last.active = false;
    }
    s
}

fn preview() -> State {
    let mut proposed = sample(SORTED);
    for p in &mut proposed {
        let group = if p.name.starts_with("Base") {
            "Base"
        } else {
            "Default"
        };
        p.group = Some(group.to_owned());
    }
    proposed[2].messages.push(Message {
        kind: MessageKind::Note,
        text: "Loads after the quest pack's master records.".to_owned(),
    });
    proposed[5].messages.push(Message {
        kind: MessageKind::Warning,
        text: "Needs a patch to work with the interior pack.".to_owned(),
    });
    let mut s = listed(CURRENT);
    s.sorting();
    s.previewed(
        proposed,
        vec![Message {
            kind: MessageKind::Note,
            text: "Masterlist for this synthetic list.".to_owned(),
        }],
    );
    s
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    let large: Vec<String> = (1..=60)
        .map(|i| format!("SamplePlugin{i:02}.esp"))
        .collect();
    let large: Vec<&str> = large.iter().map(String::as_str).collect();
    let long: Vec<String> = CURRENT
        .iter()
        .map(|n| format!("A Very Long Synthetic Plugin Name That Keeps Going On - {n}"))
        .collect();
    let long: Vec<&str> = long.iter().map(String::as_str).collect();
    let mut applying = preview();
    applying.applying();
    let mut applied = preview();
    applied.applying();
    applied.applied(Some("journal-1".to_owned()));
    let mut sorted = listed(CURRENT);
    sorted.sorting();
    sorted.previewed(sample(CURRENT), Vec::new());
    let mut failed = listed(CURRENT);
    failed.sorting();
    failed.failed(Problem {
        id: SORT_FAILED.id,
        cause: "the masterlist could not be fetched: the network is not reachable".to_owned(),
        command: Some(preview_command(INSTANCE).shown()),
    });
    let mut sorting = listed(CURRENT);
    sorting.sorting();
    let mut empty = State::default();
    empty.listed("Default".to_owned(), Vec::new());
    vec![
        Fixture {
            name: "no-instance",
            page: Box::new(LoadOrder::with(State::default())),
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
        fixture("listed", listed(CURRENT)),
        fixture("sorting", sorting),
        fixture("preview", preview()),
        fixture("sorted", sorted),
        fixture("applying", applying),
        fixture("applied", applied),
        fixture("error", failed),
        fixture("long-text", listed(&long)),
        fixture("large-list", listed(&large)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_ui_service::dto::{SortApplied, SortFinding, SortRow};

    #[test]
    fn a_sort_report_becomes_the_preview_with_its_messages_and_notes() {
        let row = |name: &str, hard: Option<bool>| SortRow {
            position: 1,
            name: name.to_owned(),
            active: true,
            installed: true,
            group: Some("default".to_owned()),
            messages: vec![SortMessage {
                kind: "say".into(),
                text: "hello".into(),
            }],
            generated_messages: Vec::new(),
            findings: hard
                .map(|hard| SortFinding {
                    kind: "missing_master".into(),
                    text: "needs a master".into(),
                    hard,
                })
                .into_iter()
                .collect(),
        };
        let report = SortReport {
            profile: "Default".into(),
            reordered: true,
            order: vec![row("A.esm", None), row("B.esp", Some(true))],
            general_messages: Vec::new(),
            not_installed: vec!["Gone.esp".into()],
            warnings: Vec::new(),
            applied: Some(SortApplied {
                lists_written: true,
                stamped: 0,
                detached: 0,
                journal: None,
            }),
        };
        let (proposed, notes) = preview_of(&report);
        assert_eq!(proposed[1].worst(), Some(MessageKind::Error));
        assert_eq!(proposed[0].worst(), Some(MessageKind::Note));
        assert_eq!(proposed[0].group.as_deref(), Some("default"));
        assert_eq!(notes[0].text, "Gone.esp");
    }

    #[test]
    fn only_a_successful_command_is_a_report() {
        let read = serde_json::from_value::<RollbackReport>;
        let ok = Ok(Output {
            code: Some(0),
            stdout: r#"{"journal":"j","kind":"sort","mod":"","profile":"Default"}"#.into(),
            stderr: String::new(),
        });
        assert_eq!(parse(&ok, read).unwrap().kind, "sort");
        let refused = Ok(Output {
            code: Some(2),
            stdout: String::new(),
            stderr: "raul rollback: refused: nothing to undo\n".into(),
        });
        assert!(
            parse(&refused, read)
                .unwrap_err()
                .contains("nothing to undo")
        );
        assert_eq!(parse(&Err("no raul".into()), read).unwrap_err(), "no raul");
    }

    #[test]
    fn the_header_and_the_chip_follow_the_state() {
        let page = LoadOrder::with(preview());
        assert_eq!(
            page.state_line(),
            "Preview · 2 moves · nothing changes until Apply"
        );
        let page = LoadOrder::with(listed(CURRENT));
        assert_eq!(page.state_line(), "7 plugins, 6 active");
        assert!(page.attention().is_none());
        let mut s = listed(CURRENT);
        s.current[0].messages.push(Message {
            kind: MessageKind::Error,
            text: "x".into(),
        });
        assert_eq!(LoadOrder::with(s).attention().map(|a| a.count), Some(1));
    }
}
