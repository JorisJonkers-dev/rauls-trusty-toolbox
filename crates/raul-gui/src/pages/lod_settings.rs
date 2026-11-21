// SPDX-License-Identifier: GPL-3.0-only
//! The LOD settings view: how distant land is built for the Instance's Profile, reached from the LOD page (own
//! route, no sidebar entry).
//!
//! An amber warning sits at the top at all times. The settings that only choose what is built show at once; the
//! per-level terrain settings and the git folder switch sit behind an "Advanced settings" disclosure that is closed
//! on every visit, repeats the warning, and unlocks only after a one-time "I understand" (kept in the app's own
//! preferences, not in the LOD settings). Each field saves when it is left (UI-RULES T6) through
//! `raul lod settings set` or `unset`; Restore defaults asks once (T4) and keeps a restore point that Undo restore
//! puts back until the next change.

use super::settings::{Pending, Visits, error_block, failure_of, settle, start, waiting};
use super::{Fixture, Page, PageCx};
use crate::catalogue::ErrorText;
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText};
use raul_drivers::lod::settings::{catalogue as opt, descriptor};
use raul_ui_model::pages::lod_settings::{
    Busy, Doc, Field, Kind, OptionRow, State, Warning, parse, reset_command, show_command, text_of,
    undo_command,
};
use raul_ui_model::pages::settings::{Command, Fetch, Save};
use raul_ui_model::pages::{Flow, PageId, Severity, View};
use raul_ui_service::Output;
use serde_json::{Value, json};

crate::texts!(TEXTS {
    TITLE = "LOD settings";
    STATE_SAVES = "Changes save when you leave a field";
    STATE_READING = "Reading the LOD settings";
    STATE_UNREADABLE = "The LOD settings could not be read";
    STATE_SAVING = "Saving";
    NO_INSTANCE = "Choose a storage folder in Settings first; the LOD settings belong to its Instance.";
    OPEN_SETTINGS = "Open Settings";
    WARNING_WORD = "Warning";
    BANNER = "These settings change how distant land is built. The defaults match the game's own LOD. Change them only if you know what each one does. Wrong values can leave LOD missing, slow or broken in the game. Restore defaults undoes every change on this page.";
    RUN_NOTE = "A run is in progress. Changes apply to the next run.";
    SECTION_BUILT = "What is built";
    SECTION_OBJECTS = "Objects";
    SECTION_RESOURCES = "Resources";
    SECTION_ADVANCED = "Advanced settings";
    SHOW_ADVANCED = "Show advanced settings";
    HIDE_ADVANCED = "Hide advanced settings";
    UNDERSTAND = "I understand that wrong values can leave distant land missing, slow or broken.";
    LOCKED = "Tick the box above to change these settings.";
    LEVEL = "Terrain level {n}";
    LEVEL_NOTE = "Each block of this level covers {n} by {n} cells. Texture sizes cost disk space and memory.";
    DEFAULT_IS = "Default: {value}";
    DEFAULT_RUN_TIME = "Default: decided at run time";
    DEFAULT_EMPTY = "Default: none";
    LIST_HINT = "Separate names with commas";
    SCALES_HINT = "Record id and number, like 0012AB34=2, with commas between";
    MARK_SET = "Changed";
    MARK_SAVING = "Saving";
    MARK_NOT_SAVED = "Not saved";
    SAVED = "Setting saved.";
    RESTORE = "Restore defaults";
    RESTORE_NOTHING = "Every setting is at its default.";
    RESTORE_TITLE = "Restore the default LOD settings?";
    RESTORE_COST = "Every changed LOD setting goes back to its default, also one you set to the default value. Allow output in a git folder stays as it is. The next generation builds the LOD again. Undo restore brings the settings back.";
    RESTORE_COST_AGAIN = "Every changed LOD setting goes back to its default, also one you set to the default value. Allow output in a git folder stays as it is. The next generation builds the LOD again. The settings the last restore kept are replaced by these.";
    RESTORE_CHANGES = "These change:";
    RESTORED = "Default LOD settings restored.";
    UNDO = "Undo restore";
    UNDO_NOTE = "Puts back the settings as they were before the last restore, until you change one.";
    UNDONE = "LOD settings put back.";
    BACK_TO_LOD = "Back to LOD";
    RETRY = "Try again";
    WARN_UNKNOWN_WORLD = "No plugin defines this world, so nothing is built for it.";
    WARN_WORLDS_UNCHECKED = "The worlds could not be checked against the plugins.";
    WARN_PLUGIN_NOT_ACTIVE = "This plugin is not active, so building LOD stops with an error.";
    WARN_NO_OBJECT_LEFT = "Every plugin in Only these plugins is also left out, so no object enters object LOD.";
    LABEL_WORLDS = opt::WORLDS;
    HELP_WORLDS = opt::WORLDS_HELP;
    LABEL_EXCLUDE_PLUGINS = opt::EXCLUDE_PLUGINS;
    HELP_EXCLUDE_PLUGINS = opt::EXCLUDE_PLUGINS_HELP;
    LABEL_ONLY_PLUGINS = opt::ONLY_PLUGINS;
    HELP_ONLY_PLUGINS = opt::ONLY_PLUGINS_HELP;
    LABEL_EXCLUDE_RECORDS = opt::EXCLUDE_RECORDS;
    HELP_EXCLUDE_RECORDS = opt::EXCLUDE_RECORDS_HELP;
    LABEL_EXCLUDE_MODELS = opt::EXCLUDE_MODELS;
    HELP_EXCLUDE_MODELS = opt::EXCLUDE_MODELS_HELP;
    LABEL_TERRAIN = opt::TERRAIN;
    HELP_TERRAIN = opt::TERRAIN_HELP;
    LABEL_LAND_TEXTURES = opt::LAND_TEXTURES;
    HELP_LAND_TEXTURES = opt::LAND_TEXTURES_HELP;
    LABEL_WATER = opt::WATER;
    HELP_WATER = opt::WATER_HELP;
    LABEL_OBJECTS = opt::OBJECTS;
    HELP_OBJECTS = opt::OBJECTS_HELP;
    LABEL_TREES = opt::TREES;
    HELP_TREES = opt::TREES_HELP;
    LABEL_THREADS = opt::THREADS;
    HELP_THREADS = opt::THREADS_HELP;
    LABEL_MEMORY = opt::MEMORY;
    HELP_MEMORY = opt::MEMORY_HELP;
    LABEL_ALLOW_IN_REPO = opt::ALLOW_IN_REPO;
    HELP_ALLOW_IN_REPO = opt::ALLOW_IN_REPO_HELP;
    LABEL_QUALITY = opt::QUALITY;
    HELP_QUALITY = opt::QUALITY_HELP;
    LABEL_MAX_VERTICES = opt::MAX_VERTICES;
    HELP_MAX_VERTICES = opt::MAX_VERTICES_HELP;
    LABEL_DIFFUSE_SIZE = opt::DIFFUSE_SIZE;
    HELP_DIFFUSE_SIZE = opt::DIFFUSE_SIZE_HELP;
    LABEL_NORMAL_SIZE = opt::NORMAL_SIZE;
    HELP_NORMAL_SIZE = opt::NORMAL_SIZE_HELP;
    LABEL_DROP_UNDER_TERRAIN = opt::DROP_UNDER_TERRAIN;
    HELP_DROP_UNDER_TERRAIN = opt::DROP_UNDER_TERRAIN_HELP;
    LABEL_DROP_UNDER_WATER = opt::DROP_UNDER_WATER;
    HELP_DROP_UNDER_WATER = opt::DROP_UNDER_WATER_HELP;
});

crate::errors!(ERRORS {
    UNREADABLE = (
        "The LOD settings could not be read",
        "Nothing changed. The line below says why."
    );
    NOT_SAVED = (
        "That setting was not saved",
        "Nothing changed. The line below says why."
    );
    NOT_RESTORED = (
        "The settings were not changed",
        "Nothing changed. The line below says why."
    );
});

const RESTORE_DIALOG: &str = "lod-settings-restore";

/// The LOD settings view.
#[derive(Debug, Default)]
pub struct LodSettings {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
    /// The "I understand" preference was read (fixtures set it themselves).
    prefs_read: bool,
    /// Bring the advanced settings into view on the next frame: they were just opened.
    scroll_to_advanced: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Show,
    Save(usize),
    Restore,
    Undo,
}

impl LodSettings {
    /// The view in `state`, as a fixture shows it.
    pub fn with(state: State) -> LodSettings {
        LodSettings {
            scroll_to_advanced: state.advanced_open,
            state,
            prefs_read: true,
            ..LodSettings::default()
        }
    }

    fn run(&mut self, cx: &mut PageCx<'_>, job: Job, command: &Command) -> bool {
        match start(cx, command) {
            Ok(id) => {
                self.pending.add(job, id);
                true
            }
            Err(e) => {
                let why = e.what.to_owned();
                match job {
                    Job::Show => self.state.doc = Fetch::Failed(why),
                    Job::Save(i) => {
                        if let Some(f) = self.state.fields.get_mut(i) {
                            f.save = Save::Failed(why);
                        }
                    }
                    Job::Restore | Job::Undo => self.state.problem = Some(why),
                }
                false
            }
        }
    }

    fn load(&mut self, cx: &mut PageCx<'_>, instance: &str) {
        let first = !self.visits.loaded;
        self.visits.loaded = true;
        if self.run(cx, Job::Show, &show_command(instance)) {
            self.state.doc = Fetch::Loading;
            if first {
                settle(cx.jobs);
            }
        }
    }

    fn save(&mut self, cx: &mut PageCx<'_>, i: usize, instance: &str) {
        let Some(command) = self.state.fields.get(i).map(|f| f.command(instance)) else {
            return;
        };
        if self.run(cx, Job::Save(i), &command)
            && let Some(f) = self.state.fields.get_mut(i)
        {
            f.save = Save::Saving;
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            let doc = answer(&result);
            match job {
                Job::Show => match doc {
                    Ok(d) if d.error.is_none() => self.state.take(d),
                    Ok(d) => {
                        self.state.doc =
                            Fetch::Failed(d.error.map_or_else(String::new, |e| e.said()));
                    }
                    Err(why) => self.state.doc = Fetch::Failed(why),
                },
                Job::Save(i) => match doc {
                    Ok(d) if d.error.is_none() => {
                        self.state.take(d);
                        cx.shell.status = Some(SAVED.to_owned());
                    }
                    Ok(d) => {
                        if let Some(f) = self.state.fields.get_mut(i) {
                            f.save = Save::Refused(d.error.map_or_else(String::new, |e| e.said()));
                        }
                    }
                    Err(why) => {
                        if let Some(f) = self.state.fields.get_mut(i) {
                            f.save = Save::Failed(why);
                        }
                    }
                },
                Job::Restore | Job::Undo => {
                    self.state.busy = Busy::Idle;
                    match doc {
                        Ok(d) if d.error.is_none() => {
                            self.state.take(d);
                            self.state.problem = None;
                            cx.shell.status = Some(
                                if job == Job::Restore {
                                    RESTORED
                                } else {
                                    UNDONE
                                }
                                .to_owned(),
                            );
                        }
                        Ok(d) => {
                            self.state.problem =
                                Some(d.error.map_or_else(String::new, |e| e.said()));
                        }
                        Err(why) => self.state.problem = Some(why),
                    }
                }
            }
        }
    }

    /// Restore defaults asks once, listing what changes (UI-RULES T4).
    fn ask_restore(&mut self, cx: &mut PageCx<'_>) {
        let names: Vec<String> = self
            .state
            .restorable()
            .iter()
            .map(|k| shown_name(k))
            .collect();
        let cost = if self.state.can_undo() {
            RESTORE_COST_AGAIN
        } else {
            RESTORE_COST
        };
        let mut dialog = Dialog::confirm(RESTORE_DIALOG, RESTORE_TITLE, cost, RESTORE);
        dialog.body.push(RESTORE_CHANGES.to_owned());
        dialog.body.push(names.join(", "));
        if let Some(instance) = &self.state.instance {
            dialog = dialog.with_command(reset_command(instance).shown());
        }
        cx.dialogs.open(dialog);
    }

    fn read_prefs(&mut self, cx: &PageCx<'_>) {
        if self.prefs_read {
            return;
        }
        self.prefs_read = true;
        if let Some(path) = cx.jobs.context().settings_file() {
            self.state.understood |=
                raul_core::update::Settings::load(&path).lod_advanced_understood;
        }
    }

    fn understood_now(&mut self, cx: &PageCx<'_>) {
        self.state.understood = true;
        if let Some(path) = cx.jobs.context().settings_file() {
            let mut prefs = raul_core::update::Settings::load(&path);
            prefs.lod_advanced_understood = true;
            // The box stays ticked for this visit even when the preference cannot be written.
            let _ = prefs.save(&path);
        }
    }
}

/// The document a finished `raul lod settings` command printed; a refusal is a document too.
fn answer(result: &Result<Output, String>) -> Result<Doc, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    parse(&out.stdout).map_err(|_| failure_of(out))
}

/// The name a view shows for `key`: its label, with the level for a per-level option.
fn shown_name(key: &str) -> String {
    let label = descriptor(key).map_or(key, |d| d.label);
    match level_of(key) {
        Some(n) => format!("{} ({})", LEVEL.replace("{n}", &n.to_string()), label),
        None => label.to_owned(),
    }
}

fn level_of(key: &str) -> Option<u32> {
    key.strip_prefix("levels.")?.split('.').next()?.parse().ok()
}

fn default_text(row: &OptionRow) -> String {
    match &row.default {
        // A number that is empty by default (the water depth) is not set, which keeps everything: "none". The worker
        // count and the memory budget are whole numbers the program decides when unset.
        Value::Null if row.kind == Kind::Number => DEFAULT_EMPTY.to_owned(),
        Value::Null => DEFAULT_RUN_TIME.to_owned(),
        Value::Array(a) if a.is_empty() => DEFAULT_EMPTY.to_owned(),
        Value::Bool(_) => String::new(),
        other => DEFAULT_IS.replace("{value}", &text_of(other)),
    }
}

fn warning_text(w: &Warning) -> &'static str {
    match w.code.as_str() {
        "unknown_world" => WARN_UNKNOWN_WORLD,
        "worlds_unchecked" => WARN_WORLDS_UNCHECKED,
        "plugin_not_active" => WARN_PLUGIN_NOT_ACTIVE,
        _ => WARN_NO_OBJECT_LEFT,
    }
}

impl Page for LodSettings {
    fn view(&self) -> View {
        View::Flow(Flow::LodSettings)
    }

    fn title(&self) -> &'static str {
        TITLE
    }

    fn state_line(&self) -> String {
        match &self.state.doc {
            Fetch::Idle | Fetch::Loading => STATE_READING,
            Fetch::Failed(_) => STATE_UNREADABLE,
            Fetch::Ready(_) if self.state.saving() => STATE_SAVING,
            Fetch::Ready(_) => STATE_SAVES,
        }
        .to_owned()
    }

    fn tools(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        if ui.button(BACK_TO_LOD).clicked() {
            cx.shell.go_to = Some(View::Page(PageId::Lod));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        self.read_prefs(cx);
        // The disclosure is closed on every visit, and the options are read again.
        if self.visits.returned(ui.ctx()) {
            self.state.advanced_open = false;
            if !self.state.saving() && !self.state.doc.is_pending() {
                self.state.doc = Fetch::Idle;
            }
        }
        if cx.shell.instance != self.state.instance && !self.state.saving() {
            self.state = State {
                instance: cx.shell.instance.clone(),
                understood: self.state.understood,
                ..State::default()
            };
        }
        let Some(instance) = self.state.instance.clone() else {
            ui.label(NO_INSTANCE);
            ui.add_space(8.0);
            if ui.button(OPEN_SETTINGS).clicked() {
                cx.shell.go_to = Some(View::Page(PageId::Settings));
            }
            return;
        };
        if self.state.doc == Fetch::Idle {
            self.load(cx, &instance);
        }
        self.take_answers(cx);
        if cx.dialogs.take_answer(RESTORE_DIALOG) == Some(Answer::Confirmed)
            && self.state.busy == Busy::Idle
            && self.run(cx, Job::Restore, &reset_command(&instance))
        {
            self.state.busy = Busy::Restoring;
        }
        banner(ui, &t, BANNER);
        if self.state.run_in_progress() {
            ui.horizontal_wrapped(|ui| {
                super::home::icon(ui, &t, super::home::Mark::Info);
                ui.label(RUN_NOTE);
            });
        }
        ui.add_space(8.0);
        match self.state.doc.clone() {
            Fetch::Idle | Fetch::Loading => {
                waiting(ui, &t, STATE_READING, &show_command(&instance));
            }
            Fetch::Failed(why) => {
                error_block(ui, &t, &UNREADABLE, Some(&why));
                if ui.button(RETRY).clicked() {
                    self.state.doc = Fetch::Idle;
                }
            }
            Fetch::Ready(doc) => self.form(ui, cx, &t, &instance, &doc),
        }
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl LodSettings {
    #[allow(clippy::too_many_lines)] // one page of sections, in the order they are drawn
    fn form(
        &mut self,
        ui: &mut egui::Ui,
        cx: &mut PageCx<'_>,
        t: &Tokens,
        instance: &str,
        doc: &Doc,
    ) {
        let mut leave = None;
        let busy = self.state.saving();
        // Restore defaults and Undo restore sit at the top, where they are seen.
        self.restore_row(ui, cx, t);
        ui.add_space(10.0);
        let tier1 = |k: &str| descriptor(k).is_some_and(|d| d.tier == 1);
        for (section, keys) in [
            (
                SECTION_BUILT,
                &[
                    "worlds",
                    "output.terrain",
                    "output.land_textures",
                    "output.water",
                    "output.objects",
                    "output.trees",
                ][..],
            ),
            (
                SECTION_OBJECTS,
                &[
                    "inclusion.exclude_plugins",
                    "inclusion.only_plugins",
                    "inclusion.exclude_records",
                    "inclusion.exclude_models",
                ],
            ),
            (SECTION_RESOURCES, &["threads", "memory_mib"]),
        ] {
            shell::section_label(ui, t, section, true);
            for key in keys.iter().filter(|k| tier1(k)) {
                if let Some(i) = self.index(key)
                    && field(ui, t, &mut self.state.fields[i], true, busy)
                {
                    leave = Some(i);
                }
                for w in doc.warnings.iter().filter(|w| w.key == *key) {
                    warning_line(ui, t, w);
                }
            }
            ui.add_space(10.0);
        }
        shell::section_label(ui, t, SECTION_ADVANCED, true);
        let toggle = if self.state.advanced_open {
            HIDE_ADVANCED
        } else {
            SHOW_ADVANCED
        };
        let toggled = ui.button(toggle);
        if self.scroll_to_advanced {
            toggled.scroll_to_me(Some(egui::Align::TOP));
            self.scroll_to_advanced = false;
        }
        if toggled.clicked() {
            self.state.advanced_open = !self.state.advanced_open;
            self.scroll_to_advanced = self.state.advanced_open;
        }
        if self.state.advanced_open {
            ui.add_space(6.0);
            banner(ui, t, BANNER);
            let mut understood = self.state.understood;
            if ui
                .add_enabled(
                    !understood,
                    egui::Checkbox::new(&mut understood, UNDERSTAND),
                )
                .changed()
                && understood
            {
                self.understood_now(cx);
            }
            let open = self.state.understood;
            if !open {
                ui.label(RichText::new(LOCKED).color(t.muted.color()));
            }
            for key in [
                "allow_in_repo",
                "objects.drop_under_terrain",
                "objects.drop_under_water",
                "terrain.drop_under_water",
                "terrain.texture_scales",
            ] {
                if let Some(i) = self.index(key)
                    && field(ui, t, &mut self.state.fields[i], open, busy)
                {
                    leave = Some(i);
                }
            }
            for n in raul_drivers::lod::settings::LEVELS {
                ui.add_space(6.0);
                ui.label(RichText::new(LEVEL.replace("{n}", &n.to_string())).strong());
                ui.label(
                    RichText::new(LEVEL_NOTE.replace("{n}", &n.to_string()))
                        .text_style(theme::secondary_style())
                        .color(t.muted.color()),
                );
                for field_name in [
                    "quality",
                    "max_vertices",
                    "diffuse_size",
                    "normal_size",
                    "specular_alpha",
                    "skirts",
                ] {
                    let key = format!("levels.{n}.{field_name}");
                    if let Some(i) = self.index(&key)
                        && field(ui, t, &mut self.state.fields[i], open, busy)
                    {
                        leave = Some(i);
                    }
                }
            }
        }
        if let Some(i) = leave {
            self.save(cx, i, instance);
        }
    }

    fn index(&self, key: &str) -> Option<usize> {
        self.state.fields.iter().position(|f| f.row.key == key)
    }

    fn restore_row(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, t: &Tokens) {
        let restorable = !self.state.restorable().is_empty();
        let idle = !self.state.saving();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(restorable && idle, egui::Button::new(RESTORE))
                .clicked()
            {
                self.ask_restore(cx);
            }
            if self.state.can_undo()
                && ui.add_enabled(idle, egui::Button::new(UNDO)).clicked()
                && let Some(instance) = self.state.instance.clone()
                && self.run(cx, Job::Undo, &undo_command(&instance))
            {
                self.state.busy = Busy::Undoing;
            }
        });
        if !restorable {
            ui.label(RichText::new(RESTORE_NOTHING).color(t.muted.color()));
        }
        if self.state.can_undo() {
            ui.label(RichText::new(UNDO_NOTE).color(t.muted.color()));
        }
        if let Some(why) = &self.state.problem {
            error_block(ui, t, &NOT_RESTORED, Some(why));
        }
    }
}

/// The amber warning: colour, icon and a word (UI-RULES C2, C3).
fn banner(ui: &mut egui::Ui, t: &Tokens, text: &str) {
    let colour = shell::severity_colour(t, Severity::Warning);
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, colour))
        .inner_margin(egui::Margin::same(10))
        .corner_radius(6)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                super::home::mark(ui, t, super::home::Mark::Warning, WARNING_WORD);
                ui.label(text);
            });
        });
}

fn warning_line(ui: &mut egui::Ui, t: &Tokens, w: &Warning) {
    ui.horizontal_wrapped(|ui| {
        super::home::icon(ui, t, super::home::Mark::Warning);
        let text = match &w.value {
            Some(v) => format!("{v}: {}", warning_text(w)),
            None => warning_text(w).to_owned(),
        };
        ui.label(RichText::new(text).color(shell::severity_colour(t, Severity::Warning)));
    });
}

/// One option: its name and help, then a check box (switches save when clicked) or a text field (saves when left),
/// its default, and its mark. Returns true when it has a change to save.
fn field(ui: &mut egui::Ui, t: &Tokens, f: &mut Field, enabled: bool, busy: bool) -> bool {
    let mut leave = false;
    let d = descriptor(&f.row.key);
    let label = d.map_or(f.row.key.as_str(), |d| d.label);
    let help = d.map_or("", |d| d.help);
    let saving = f.save == Save::Saving;
    let editable = enabled && !busy;
    let mut label_id = None;
    ui.horizontal_wrapped(|ui| {
        if f.row.kind == Kind::Switch {
            let mut on = f.draft == "true";
            if ui
                .add_enabled(editable, egui::Checkbox::new(&mut on, label))
                .changed()
            {
                f.draft = on.to_string();
                leave = true;
            }
        } else {
            label_id = Some(ui.label(label).id);
        }
        match &f.save {
            Save::Saving => super::home::mark(ui, t, super::home::Mark::Info, MARK_SAVING),
            Save::Refused(_) | Save::Failed(_) => {
                super::home::mark(ui, t, super::home::Mark::Problem, MARK_NOT_SAVED);
            }
            Save::Idle if f.row.overridden => {
                super::home::mark(ui, t, super::home::Mark::Info, MARK_SET);
            }
            Save::Idle => {}
        }
    });
    ui.label(
        RichText::new(help)
            .text_style(theme::secondary_style())
            .color(t.muted.color()),
    );
    if f.row.kind != Kind::Switch {
        let hint = if f.row.kind == Kind::Scales {
            SCALES_HINT
        } else if matches!(f.row.kind, Kind::Names | Kind::Records) {
            LIST_HINT
        } else {
            ""
        };
        let edit = egui::TextEdit::singleline(&mut f.draft)
            .hint_text(hint)
            .desired_width(ui.available_width().min(420.0));
        let mut response = ui.add_enabled(editable && !saving, edit);
        if let Some(id) = label_id {
            response = response.labelled_by(id);
        }
        if response.lost_focus() && f.changed() && !saving {
            leave = true;
        }
        let default = default_text(&f.row);
        if !default.is_empty() {
            ui.label(
                RichText::new(default)
                    .text_style(theme::secondary_style())
                    .color(t.muted.color()),
            );
        }
    }
    if let Save::Refused(why) | Save::Failed(why) = &f.save {
        error_block(ui, t, &NOT_SAVED, Some(why));
    }
    ui.add_space(6.0);
    leave
}

const INSTANCE: &str = "/users/player/raul-storage/instance";

fn row(key: &str, value: Value, overridden: bool) -> OptionRow {
    let d = descriptor(key).expect("a known option");
    let kind = match d.kind {
        raul_drivers::lod::settings::Kind::Switch => Kind::Switch,
        raul_drivers::lod::settings::Kind::Count => Kind::Count,
        raul_drivers::lod::settings::Kind::Number => Kind::Number,
        raul_drivers::lod::settings::Kind::TextureSize => Kind::TextureSize,
        raul_drivers::lod::settings::Kind::Names => Kind::Names,
        raul_drivers::lod::settings::Kind::Records => Kind::Records,
        raul_drivers::lod::settings::Kind::Scales => Kind::Scales,
    };
    let default = raul_drivers::lod::settings::default_value(key).unwrap_or(Value::Null);
    OptionRow {
        key: key.to_owned(),
        tier: d.tier,
        kind,
        value: if overridden { value } else { default.clone() },
        default,
        overridden,
        restores: d.restores,
    }
}

/// A document with every option at its default, then `set` applied.
fn doc(set: &[(&str, Value)]) -> Doc {
    Doc {
        options: raul_drivers::lod::settings::OPTIONS
            .iter()
            .map(|d| {
                let value = set
                    .iter()
                    .find(|(k, _)| *k == d.key)
                    .map(|(_, v)| v.clone());
                row(d.key, value.clone().unwrap_or(Value::Null), value.is_some())
            })
            .collect(),
        restore_point: None,
        warnings: Vec::new(),
        run_in_progress: false,
        change: None,
        error: None,
    }
}

fn fixture(name: &'static str, state: State) -> Fixture {
    let mut f = Fixture::new(name, LodSettings::with(state));
    f.shell.instance = Some(INSTANCE.to_owned());
    f.shell.new_vegas = Some("/games/steamapps/common/Fallout New Vegas".to_owned());
    f
}

/// The states the snapshot tests render.
#[allow(clippy::too_many_lines)]
pub fn fixtures() -> Vec<Fixture> {
    let changed_set = [
        ("worlds", json!(["SampleWorld", "Nowhere"])),
        ("output.trees", json!(false)),
        ("threads", json!(2)),
        ("allow_in_repo", json!(true)),
    ];
    let mut changed = doc(&changed_set);
    changed.warnings.push(Warning {
        code: "unknown_world".into(),
        key: "worlds".into(),
        value: Some("Nowhere".into()),
        what: String::new(),
    });
    let mut locked = State::loaded(INSTANCE, doc(&[]));
    locked.advanced_open = true;
    let mut open = State::loaded(
        INSTANCE,
        doc(&[
            ("levels.16.quality", json!(18.0)),
            ("levels.4.diffuse_size", json!(512)),
        ]),
    );
    open.advanced_open = true;
    open.understood = true;
    let mut saving = State::loaded(INSTANCE, doc(&changed_set));
    if let Some(f) = saving.field_mut("threads") {
        "3".clone_into(&mut f.draft);
        f.save = Save::Saving;
    }
    let mut refused = State::loaded(INSTANCE, doc(&changed_set));
    if let Some(f) = refused.field_mut("inclusion.exclude_records") {
        "0001".clone_into(&mut f.draft);
        f.save = Save::Refused(
            "A record to leave out must be eight hex digits. Nothing changed.".into(),
        );
    }
    let mut restored_doc = doc(&[("allow_in_repo", json!(true))]);
    restored_doc.restore_point = Some(format!(
        "{INSTANCE}/profiles/Default/raul-lod.toml.previous"
    ));
    let mut running = doc(&changed_set);
    running.run_in_progress = true;
    let long_worlds: Vec<String> = (0..24)
        .map(|i| format!("SampleWorldWithALongEditorId{i:02}"))
        .collect();
    let mut failed = State::loaded(INSTANCE, doc(&changed_set));
    failed.problem = Some(
        "The LOD settings could not be saved. Permission denied (os error 13) Nothing changed."
            .into(),
    );
    vec![
        fixture(
            "loading",
            State {
                instance: Some(INSTANCE.to_owned()),
                doc: Fetch::Loading,
                ..State::default()
            },
        ),
        fixture("defaults", State::loaded(INSTANCE, doc(&[]))),
        fixture("changed", State::loaded(INSTANCE, changed)),
        fixture("advanced-locked", locked),
        fixture("advanced-open", open),
        fixture("saving", saving),
        fixture("refused", refused),
        fixture("restored", State::loaded(INSTANCE, restored_doc)),
        fixture("run-in-progress", State::loaded(INSTANCE, running)),
        fixture("restore-failed", failed),
        fixture(
            "error",
            State {
                instance: Some(INSTANCE.to_owned()),
                doc: Fetch::Failed(
                    "The LOD settings hold a setting this version does not know. (colour) Nothing changed.".into(),
                ),
                ..State::default()
            },
        ),
        fixture(
            "long-text",
            State::loaded(INSTANCE, doc(&[("worlds", json!(long_worlds))])),
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
    use raul_ui_service::Exec;
    use std::sync::{Arc, Mutex};

    /// Answers `raul lod settings` with a document where `threads` is 4 and records every command.
    #[derive(Default)]
    struct Recording {
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            let d = doc(&[("threads", json!(4))]);
            let options: Vec<Value> = d
                .options
                .iter()
                .map(|o| {
                    json!({"key": o.key, "tier": o.tier, "kind": kind_name(o.kind), "value": o.value,
                           "default": o.default, "overridden": o.overridden, "restores": o.restores})
                })
                .collect();
            Ok(Output {
                code: Some(0),
                stdout: json!({"contract": "raul-lod-settings", "options": options, "warnings": [],
                               "restore_point": null, "change": {"keys": ["threads"], "changed": true}, "error": null})
                .to_string(),
                stderr: String::new(),
            })
        }
    }

    fn kind_name(k: Kind) -> &'static str {
        match k {
            Kind::Switch => "switch",
            Kind::Count => "count",
            Kind::Number => "number",
            Kind::TextureSize => "texture_size",
            Kind::Names => "names",
            Kind::Records => "records",
            Kind::Scales => "scales",
        }
    }

    #[test]
    fn leaving_a_changed_field_saves_it_through_raul_lod_settings_set() {
        let exec = Arc::new(Recording::default());
        let fixture = fixtures()
            .into_iter()
            .find(|f| f.name == "defaults")
            .unwrap();
        let app = App::fixture(fixture, Jobs::new(exec.clone()));
        let mut h = Harness::builder()
            .with_size([1280.0, 3200.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            });
        h.run_steps(4);
        h.get_by_label(LABEL_THREADS).click();
        h.run_steps(2);
        h.get_by_label(LABEL_THREADS).type_text("4");
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        for _ in 0..50 {
            h.run_steps(2);
            let calls = exec.calls.lock().unwrap();
            if calls.iter().any(|c| c.get(2).is_some_and(|w| w == "set")) {
                break;
            }
            drop(calls);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let calls = exec.calls.lock().unwrap().clone();
        let set = calls
            .iter()
            .find(|c| c.get(2).is_some_and(|w| w == "set"))
            .unwrap_or_else(|| panic!("no set in {calls:?}"));
        assert_eq!(set[3], "threads");
        assert_eq!(set[4], "4");
    }

    #[test]
    fn restore_defaults_lists_every_changed_generation_setting_and_keeps_the_git_folder_switch() {
        let s = State::loaded(
            INSTANCE,
            doc(&[
                ("threads", json!(2)),
                ("allow_in_repo", json!(true)),
                ("levels.8.quality", json!(15.0)),
            ]),
        );
        assert_eq!(s.restorable(), ["threads", "levels.8.quality"]);
        assert_eq!(
            shown_name("levels.8.quality"),
            format!("{} ({})", LEVEL.replace("{n}", "8"), opt::QUALITY)
        );
    }

    #[test]
    fn every_terrain_level_has_the_shine_row_in_the_advanced_settings() {
        let fixture = fixtures()
            .into_iter()
            .find(|f| f.name == "advanced-open")
            .unwrap();
        let app = App::fixture(fixture, Jobs::new(Arc::new(Recording::default())));
        let mut h = Harness::builder()
            .with_size([1280.0, 3600.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            });
        h.run_steps(4);
        let shown = h
            .query_all_by_label(raul_drivers::lod::settings::catalogue::SPECULAR_ALPHA)
            .count();
        assert_eq!(shown, 4, "one row for each of the four levels");
        assert_eq!(
            shown_name("levels.16.specular_alpha"),
            format!(
                "{} ({})",
                LEVEL.replace("{n}", "16"),
                raul_drivers::lod::settings::catalogue::SPECULAR_ALPHA
            )
        );
    }

    #[test]
    fn the_skirts_and_water_depth_rows_are_in_the_advanced_settings() {
        use raul_drivers::lod::settings::catalogue as c;
        let fixture = fixtures()
            .into_iter()
            .find(|f| f.name == "advanced-open")
            .unwrap();
        let app = App::fixture(fixture, Jobs::new(Arc::new(Recording::default())));
        let mut h = Harness::builder()
            .with_size([1280.0, 4200.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            });
        h.run_steps(4);
        assert_eq!(
            h.query_all_by_label(c::SKIRTS).count(),
            4,
            "one for each level"
        );
        assert_eq!(h.query_all_by_label(c::TERRAIN_DROP_UNDER_WATER).count(), 1);
        assert_eq!(h.query_all_by_label(c::TEXTURE_SCALES).count(), 1);
    }
}
