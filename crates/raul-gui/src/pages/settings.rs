// SPDX-License-Identifier: GPL-3.0-only
//! Settings: the paths the Toolbox uses, and nothing else (UI-RULES P3, P5, T6).
//!
//! Each path saves when its field is left, through `raul config set` (or `unset` when it is emptied). A refused
//! path is not written and says why. The page also reopens the setup wizard and shows the About and notices.
//!
//! The helpers at the end of this file start a [`Command`] through the jobs and read its answer. Home, Versions
//! and the setup wizard use them too, since all four pages read and write the same configuration.

use super::{Fixture, Page, PageCx};
use crate::catalogue::{self as msg, ErrorText};
use crate::jobs::Jobs;
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText};
use raul_ui_model::job::{JobError, JobId};
use raul_ui_model::pages::settings::{Answer, Command, Config, Fetch, Field, Key, Save, State};
use raul_ui_model::pages::{Flow, PageId, View};
use raul_ui_service::Output;
use raul_ui_service::registry;

crate::texts!(TEXTS {
    STATE_SAVES = "Changes save when you leave a field";
    STATE_READING = "Reading the settings";
    STATE_UNREADABLE = "The settings could not be read";
    STATE_SAVING = "Saving";
    SECTION_GAMES = "Game installations";
    SECTION_STORAGE = "Toolbox storage";
    SECTION_SETUP = "Setup";
    SECTION_ABOUT = "About and notices";
    LABEL_NEW_VEGAS = "Fallout: New Vegas";
    LABEL_FALLOUT3 = "Fallout 3";
    LABEL_TTW_PACKAGE = "TTW installer";
    LABEL_STORAGE = "Storage folder";
    LABEL_INSTANCE = "Instance";
    HINT_REQUIRED = "Required";
    HINT_FALLOUT3 = "Only for TTW";
    HINT_TTW_PACKAGE = "The package you downloaded";
    HINT_STORAGE = "Holds everything the Toolbox makes";
    HINT_INSTANCE = "Inside the storage folder";
    SAVED_NEW_VEGAS = "New Vegas folder saved.";
    SAVED_FALLOUT3 = "Fallout 3 folder saved.";
    SAVED_TTW_PACKAGE = "TTW installer saved.";
    SAVED_STORAGE = "Storage folder saved.";
    CLEARED_NEW_VEGAS = "New Vegas folder cleared.";
    CLEARED_FALLOUT3 = "Fallout 3 folder cleared.";
    CLEARED_TTW_PACKAGE = "TTW installer cleared.";
    CLEARED_STORAGE = "Storage folder cleared.";
    EXITED_WITH = "raul exited with code {code}";
    STOPPED = "raul was stopped";
    MARK_FOUND = "Found";
    MARK_NOT_SET = "Not set";
    MARK_SAVING = "Saving";
    MARK_NOT_SAVED = "Not saved";
    RUN_SETUP = "Run setup again";
    RUN_SETUP_NOTE = "Walks through the game folder, TTW, the engine and a quick check. Nothing in your game folder changes.";
    OPEN_ABOUT = "About and notices";
    ABOUT_NOTE = "What the Toolbox is, and every bundled asset with its licence.";
    RETRY = "Try again";
    CONFIG_FILE = "Saved in";
});

crate::errors!(ERRORS {
    UNREADABLE = (
        "The settings could not be read",
        "Nothing changed. The line below says why."
    );
    NOT_SAVED = (
        "That path was not saved",
        "Nothing changed. The line below says why."
    );
    WAITING = (
        "This build cannot run that command from the window yet",
        "Nothing changed."
    );
});

/// The Settings page.
#[derive(Debug, Default)]
pub struct Settings {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    List,
    Save(Key),
}

impl Settings {
    /// Settings in `state`, as a fixture shows it.
    pub fn with(state: State) -> Settings {
        Settings {
            state,
            ..Settings::default()
        }
    }

    fn load(&mut self, cx: &mut PageCx<'_>, first: bool) {
        match start(cx, &Command::config_list()) {
            Ok(id) => {
                self.pending.add(Job::List, id);
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
            Err(e) => field.save = Save::Failed(e.what.to_owned()),
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            match job {
                Job::List => {
                    self.state = match read_config(&result) {
                        Ok(config) => State::loaded(config),
                        Err(why) => State {
                            config: Fetch::Failed(why),
                            fields: Vec::new(),
                        },
                    };
                }
                Job::Save(key) => match config_answer(&result) {
                    Ok(answer) => {
                        self.state.saved(key, &answer);
                        if let Answer::Entries(c) = &answer {
                            cx.shell.status =
                                Some(saved_text(key, c.get(key).is_some()).to_owned());
                        }
                    }
                    Err(why) => {
                        if let Some(f) = self.state.field_mut(key) {
                            f.save = Save::Failed(why);
                        }
                    }
                },
            }
        }
        if let Some(config) = self.state.config.ready() {
            share_config(cx, config);
        }
    }
}

impl Page for Settings {
    fn view(&self) -> View {
        View::Page(PageId::Settings)
    }

    fn title(&self) -> &'static str {
        msg::PAGE_SETTINGS
    }

    fn state_line(&self) -> String {
        match &self.state.config {
            Fetch::Idle | Fetch::Loading => STATE_READING,
            Fetch::Failed(_) => STATE_UNREADABLE,
            Fetch::Ready(_) if self.state.saving() => STATE_SAVING,
            Fetch::Ready(_) => STATE_SAVES,
        }
        .to_owned()
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        if self.visits.returned(ui.ctx()) && !self.state.saving() && !self.state.config.is_pending()
        {
            self.state.config = Fetch::Idle;
        }
        if self.state.config == Fetch::Idle {
            let first = !self.visits.loaded;
            self.visits.loaded = true;
            self.load(cx, first);
        }
        self.take_answers(cx);
        let t = *cx.tokens;
        // What the footer's status line says is a failure too, and can be copied there.
        if let Fetch::Failed(_) = &self.state.config {
            cx.shell.status_error = Some(UNREADABLE);
        } else {
            cx.shell.status_error = None;
        }
        match &self.state.config {
            Fetch::Idle | Fetch::Loading => {
                waiting(ui, &t, STATE_READING, &Command::config_list());
            }
            Fetch::Failed(why) => {
                error_block(ui, &t, &UNREADABLE, Some(why));
                if ui.button(RETRY).clicked() {
                    self.state.config = Fetch::Idle;
                }
            }
            Fetch::Ready(config) => {
                let file = config.file.clone();
                let instance = config.get(Key::Instance).map(str::to_owned);
                self.form(ui, cx, &file, instance.as_deref());
            }
        }
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl Settings {
    fn form(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, file: &str, instance: Option<&str>) {
        let t = *cx.tokens;
        let mut leave = None;
        shell::section_label(ui, &t, SECTION_GAMES, true);
        for key in [Key::NewVegas, Key::Fallout3, Key::TtwPackage] {
            if let Some(f) = self.state.field_mut(key)
                && path_field(ui, &t, f, label_of(key), hint_of(key), required(key))
            {
                leave = Some(key);
            }
        }
        ui.add_space(12.0);
        shell::section_label(ui, &t, SECTION_STORAGE, true);
        if let Some(f) = self.state.field_mut(Key::StorageRoot)
            && path_field(ui, &t, f, LABEL_STORAGE, HINT_STORAGE, true)
        {
            leave = Some(Key::StorageRoot);
        }
        read_only_row(ui, &t, LABEL_INSTANCE, HINT_INSTANCE, instance);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(CONFIG_FILE).color(t.muted.color()));
            ui.add(
                egui::Label::new(RichText::new(file).monospace().color(t.muted.color())).truncate(),
            )
            .on_hover_text(file);
        });
        if let Some(key) = leave {
            self.save(cx, key);
        }
        ui.add_space(12.0);
        shell::section_label(ui, &t, SECTION_SETUP, true);
        ui.label(RichText::new(RUN_SETUP_NOTE).color(t.muted.color()));
        if ui.button(RUN_SETUP).clicked() {
            cx.shell.go_to = Some(View::Flow(Flow::Setup));
        }
        ui.add_space(12.0);
        shell::section_label(ui, &t, SECTION_ABOUT, true);
        ui.label(RichText::new(ABOUT_NOTE).color(t.muted.color()));
        if ui.button(OPEN_ABOUT).clicked() {
            cx.dialogs.open(shell::about());
        }
    }
}

fn label_of(key: Key) -> &'static str {
    match key {
        Key::NewVegas => LABEL_NEW_VEGAS,
        Key::Fallout3 => LABEL_FALLOUT3,
        Key::TtwPackage => LABEL_TTW_PACKAGE,
        Key::StorageRoot => LABEL_STORAGE,
        Key::Instance => LABEL_INSTANCE,
    }
}

fn hint_of(key: Key) -> &'static str {
    match key {
        Key::NewVegas => HINT_REQUIRED,
        Key::Fallout3 => HINT_FALLOUT3,
        Key::TtwPackage => HINT_TTW_PACKAGE,
        Key::StorageRoot => HINT_STORAGE,
        Key::Instance => HINT_INSTANCE,
    }
}

fn required(key: Key) -> bool {
    matches!(key, Key::NewVegas | Key::StorageRoot)
}

/// The footer line of a save (UI-RULES T6): saved, or cleared when the path was emptied.
fn saved_text(key: Key, set: bool) -> &'static str {
    match (key, set) {
        (Key::NewVegas, true) => SAVED_NEW_VEGAS,
        (Key::NewVegas, false) => CLEARED_NEW_VEGAS,
        (Key::Fallout3, true) => SAVED_FALLOUT3,
        (Key::Fallout3, false) => CLEARED_FALLOUT3,
        (Key::TtwPackage, true) => SAVED_TTW_PACKAGE,
        (Key::TtwPackage, false) => CLEARED_TTW_PACKAGE,
        (Key::StorageRoot | Key::Instance, true) => SAVED_STORAGE,
        (Key::StorageRoot | Key::Instance, false) => CLEARED_STORAGE,
    }
}

/// The width of the label column of a form.
pub(crate) const LABEL_WIDTH: f32 = 190.0;

/// The label column of a form in the space `ui` has: [`LABEL_WIDTH`], narrower when the page is (UI-RULES A6).
pub(crate) fn label_width(ui: &egui::Ui) -> f32 {
    let room = ui.available_width();
    if room < 520.0 {
        (room * 0.3).clamp(84.0, LABEL_WIDTH)
    } else {
        LABEL_WIDTH
    }
}

/// One path field (UI-RULES T6): label and hint, the path in monospace, and its mark.
/// Returns true when the field was left with a change to save.
pub(crate) fn path_field(
    ui: &mut egui::Ui,
    t: &Tokens,
    f: &mut Field,
    label: &str,
    hint: &str,
    required: bool,
) -> bool {
    let mut leave = false;
    let label_width = label_width(ui);
    ui.horizontal(|ui| {
        let label_id = ui
            .vertical(|ui| {
                ui.set_width(label_width);
                let id = ui.label(label).id;
                ui.label(
                    RichText::new(hint)
                        .text_style(theme::secondary_style())
                        .color(t.muted.color()),
                );
                id
            })
            .inner;
        let saving = f.save == Save::Saving;
        let edit = egui::TextEdit::singleline(&mut f.draft)
            .font(egui::TextStyle::Monospace)
            .hint_text(MARK_NOT_SET)
            .desired_width((ui.available_width() - 130.0).max(80.0));
        let response = ui.add_enabled(!saving, edit).labelled_by(label_id);
        if !f.draft.is_empty() {
            response.clone().on_hover_text(f.draft.clone());
        }
        // Enter also leaves a single-line field, so both save.
        if response.lost_focus() && f.changed() && !saving {
            leave = true;
        }
        match &f.save {
            Save::Saving => super::home::mark(ui, t, super::home::Mark::Info, MARK_SAVING),
            Save::Refused(_) | Save::Failed(_) => {
                super::home::mark(ui, t, super::home::Mark::Problem, MARK_NOT_SAVED);
            }
            Save::Idle if f.saved.is_some() => {
                super::home::mark(ui, t, super::home::Mark::Ok, MARK_FOUND);
            }
            Save::Idle if required => {
                super::home::mark(ui, t, super::home::Mark::Warning, MARK_NOT_SET);
            }
            Save::Idle => super::home::mark(ui, t, super::home::Mark::Unset, MARK_NOT_SET),
        }
    });
    ui.horizontal(|ui| {
        ui.add_space(label_width + ui.spacing().item_spacing.x);
        ui.vertical(|ui| {
            if let Save::Refused(why) | Save::Failed(why) = &f.save {
                error_block(ui, t, &NOT_SAVED, Some(why));
            }
        });
    });
    ui.add_space(6.0);
    leave
}

fn read_only_row(ui: &mut egui::Ui, t: &Tokens, label: &str, hint: &str, value: Option<&str>) {
    let label_width = label_width(ui);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(label_width);
            ui.label(label);
            ui.label(
                RichText::new(hint)
                    .text_style(theme::secondary_style())
                    .color(t.muted.color()),
            );
        });
        let text = value.unwrap_or(MARK_NOT_SET);
        let rich = if value.is_some() {
            RichText::new(text).monospace().color(t.text.color())
        } else {
            RichText::new(text).color(t.muted.color())
        };
        ui.add(egui::Label::new(rich).truncate())
            .on_hover_text(text);
    });
    ui.add_space(6.0);
}

/// Tells the shell the New Vegas folder the top bar shows, and lifts a Play block a new path may have fixed; Home
/// checks again when Play is pressed.
pub(crate) fn share_config(cx: &mut PageCx<'_>, config: &Config) {
    cx.shell.new_vegas = config.get(Key::NewVegas).map(str::to_owned);
    if config.get(Key::NewVegas).is_some() && config.get(Key::Instance).is_some() {
        let lifted = [
            msg::PLAY_NO_GAME.id,
            super::home::PLAY_NO_STORAGE.id,
            super::home::PLAY_UNREADABLE.id,
        ];
        if cx
            .shell
            .play_blocked
            .is_some_and(|b| lifted.contains(&b.id))
        {
            cx.shell.play_blocked = None;
        }
    }
}

// ---- Shared by the setup path pages: starting a command and reading its answer. ----

/// Starts `command` through the jobs, by its entry in the command registry (UI-RULES G6: the same `raul` command
/// the page shows).
///
/// # Errors
/// [`WAITING`] when the registry has no entry for it yet or marks it as waiting for its pull request,
/// [`msg::JOB_BUSY`] while another writing command runs, and [`msg::JOB_NOT_STARTED`] otherwise.
pub(crate) fn start(cx: &mut PageCx<'_>, command: &Command) -> Result<JobId, ErrorText> {
    start_then(cx, command, None)
}

/// [`start`] with a second command that runs as part of the same job once the first one went through.
pub(crate) fn start_then(
    cx: &mut PageCx<'_>,
    command: &Command,
    then: Option<&Command>,
) -> Result<JobId, ErrorText> {
    let Some(entry) = registry::command(command.id).filter(|c| c.awaits.is_none()) else {
        return Err(WAITING);
    };
    let then = then.map(|c| {
        registry::command(c.id).map_or_else(Vec::new, |e| {
            let mut args = vec![e.action.command().to_owned()];
            args.extend(e.args(c.rest()));
            if e.action.takes_json() {
                args.push("--json".to_owned());
            }
            args
        })
    });
    // An action that is converted runs as a typed job in process (`raul_ui_service::direct`); the page reads its
    // answer the same way, so nothing above this call knows the difference.
    let started = match cx.jobs.direct_job(command.id, &command.rest()) {
        Some(job) if then.is_none() => cx.jobs.start_direct(entry.action, job, entry.writes),
        _ => cx
            .jobs
            .start_then(entry.action, entry.args(command.rest()), then, entry.writes),
    };
    started.map_err(|e| match e {
        JobError::Busy(_) => msg::JOB_BUSY,
        JobError::Unknown(_) | JobError::NotCancellable(_) => msg::JOB_NOT_STARTED,
    })
}

/// Whether this build can start `command` from the window.
pub(crate) fn available(command: &Command) -> bool {
    registry::command(command.id).is_some_and(|c| c.awaits.is_none())
}

/// Waits a moment for the reads a page just started, so its first frame shows what they found rather than a
/// flash of "Checking". Only a page's first read waits; a slow read just keeps loading.
pub(crate) fn settle(jobs: &mut Jobs) {
    jobs.wait_all(std::time::Duration::from_millis(200));
}

/// The jobs a page waits for, each with what it was for.
#[derive(Debug)]
pub(crate) struct Pending<P> {
    jobs: Vec<(P, JobId)>,
}

impl<P> Default for Pending<P> {
    fn default() -> Self {
        Pending { jobs: Vec::new() }
    }
}

impl<P: Copy + PartialEq> Pending<P> {
    /// Whether no job is awaited.
    pub(crate) fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Waits for job `id`, started for `purpose`.
    pub(crate) fn add(&mut self, purpose: P, id: JobId) {
        self.jobs.push((purpose, id));
    }

    /// The jobs that ended since the last call: what each printed, or why it could not be started.
    pub(crate) fn finished(&mut self, jobs: &Jobs) -> Vec<(P, Result<Output, String>)> {
        let mut done = Vec::new();
        self.jobs.retain(|(purpose, id)| {
            if let Some(out) = jobs.output(*id) {
                done.push((*purpose, Ok(out.clone())));
                false
            } else if let Some(why) = jobs.failure(*id) {
                done.push((*purpose, Err(why.what.clone())));
                false
            } else {
                true
            }
        });
        done
    }
}

/// Why a command that ran did not succeed: the last line it wrote to standard error, or its exit code.
pub(crate) fn failure_of(out: &Output) -> String {
    out.stderr
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map_or_else(
            || match out.code {
                Some(code) => EXITED_WITH.replace("{code}", &code.to_string()),
                None => STOPPED.to_owned(),
            },
            str::to_owned,
        )
}

/// What a finished `raul config` command answered.
pub(crate) fn config_answer(result: &Result<Output, String>) -> Result<Answer, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    raul_ui_model::pages::settings::parse(&out.stdout).map_err(|_| failure_of(out))
}

/// The configuration a finished `raul config list` printed.
pub(crate) fn read_config(result: &Result<Output, String>) -> Result<Config, String> {
    match config_answer(result)? {
        Answer::Entries(c) => Ok(c),
        Answer::Refused(r) => Err(r.message),
    }
}

/// Notices when a page is shown again after another one was, so it can read its data again.
#[derive(Debug, Default)]
pub(crate) struct Visits {
    last: Option<u64>,
    /// Whether the page has started its first read.
    pub(crate) loaded: bool,
}

impl Visits {
    /// Whether the page was away for at least one frame before this one.
    pub(crate) fn returned(&mut self, ctx: &egui::Context) -> bool {
        let now = ctx.cumulative_frame_nr();
        let back = self.last.is_some_and(|last| now > last + 1);
        self.last = Some(now);
        back
    }
}

/// A read on its way: a spinner and what is read. The command that reads it is kept as a parameter so callers
/// still compile, but is no longer drawn on screen.
pub(crate) fn waiting(ui: &mut egui::Ui, t: &Tokens, what: &str, _command: &Command) {
    ui.horizontal_wrapped(|ui| {
        super::home::icon(ui, t, super::home::Mark::Info);
        ui.label(RichText::new(what).color(t.muted.color()));
    });
}

/// An error where it occurred (UI-RULES W3, G4): what went wrong, the plain detail, and what the command said.
/// The shared error widget draws it, so its text is selectable and it has a "Copy details" button.
pub(crate) fn error_block(ui: &mut egui::Ui, t: &Tokens, e: &ErrorText, said: Option<&str>) {
    // `e.detail` is drawn once, above the headline; the lines below it are only what the command said.
    shell::error_widget(ui, t, e.detail, e.what, said.unwrap_or_default());
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    let full = Config::with(
        "/users/player/.config/raul/config.toml",
        &[
            (Key::NewVegas, "/games/steamapps/common/Fallout New Vegas"),
            (Key::Fallout3, "/games/steamapps/common/Fallout 3 goty"),
            (Key::TtwPackage, "/users/player/Downloads/ttw-installer.mpi"),
            (Key::StorageRoot, "/users/player/raul-storage"),
            (Key::Instance, "/users/player/raul-storage/instance"),
        ],
    );
    let partial = Config::with(
        "/users/player/.config/raul/config.toml",
        &[
            (Key::NewVegas, "/games/steamapps/common/Fallout New Vegas"),
            (Key::StorageRoot, "/users/player/raul-storage"),
            (Key::Instance, "/users/player/raul-storage/instance"),
        ],
    );
    let mut refused = State::loaded(partial.clone());
    if let Some(f) = refused.field_mut(Key::NewVegas) {
        *f = Field::typed(
            Key::NewVegas,
            Some("/games/steamapps/common/Fallout New Vegas"),
            "/users/player/Downloads",
            Save::Refused(
                "/users/player/Downloads is not a Fallout: New Vegas folder: no FalloutNV.exe in it".into(),
            ),
        );
    }
    let mut saving = State::loaded(partial.clone());
    if let Some(f) = saving.field_mut(Key::Fallout3) {
        *f = Field::typed(
            Key::Fallout3,
            None,
            "/games/steamapps/common/Fallout 3 goty",
            Save::Saving,
        );
    }
    let deep = "/mnt/second-drive/a/very/deeply/nested/library/folder/that/keeps/going/and/going/steamapps/common/Fallout New Vegas";
    let long = Config::with(
        "/users/player-with-a-long-name/.config/raul/config.toml",
        &[
            (Key::NewVegas, deep),
            (Key::Fallout3, deep),
            (Key::TtwPackage, deep),
            (Key::StorageRoot, deep),
            (Key::Instance, deep),
        ],
    );
    vec![
        Fixture::new(
            "loading",
            Settings::with(State {
                config: Fetch::Loading,
                fields: Vec::new(),
            }),
        ),
        Fixture::new(
            "empty",
            Settings::with(State::loaded(Config::with(
                "/users/player/.config/raul/config.toml",
                &[],
            ))),
        ),
        Fixture::new("partial", Settings::with(State::loaded(partial))),
        Fixture::new("full", Settings::with(State::loaded(full))),
        Fixture::new("saving", Settings::with(saving)),
        Fixture::new("refused", Settings::with(refused)),
        Fixture::new(
            "error",
            Settings::with(State {
                config: Fetch::Failed(
                    "the configuration file has schema_version 2; this raul reads 1".into(),
                ),
                fields: Vec::new(),
            }),
        ),
        Fixture::new("long-text", Settings::with(State::loaded(long))),
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

    /// Answers `raul config set` as the contract does and records every command.
    #[derive(Default)]
    struct Recording {
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl Exec for Recording {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            let value = args.get(3).cloned().unwrap_or_default();
            Ok(Output {
                code: Some(0),
                stdout: format!(
                    r#"{{"schema_version":1,"contract":"raul-config","file":"/c","entries":[{{"key":"game.fallout3","value":"{value}","read_only":false}}],"error":null}}"#
                ),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn leaving_a_changed_field_saves_it_through_raul_config_set() {
        let exec = Arc::new(Recording::default());
        let fixture = fixtures()
            .into_iter()
            .find(|f| f.name == "partial")
            .unwrap();
        let app = App::fixture(
            fixture,
            Jobs::new(exec.clone()).scripted(&[
                "config-list",
                "config-get",
                "config-set",
                "config-unset",
            ]),
        );
        let mut h = Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                crate::set_up(&cc.egui_ctx);
                app
            });
        h.run();
        h.get_by_label(LABEL_FALLOUT3).click();
        h.run();
        h.get_by_label(LABEL_FALLOUT3).type_text("/g/fo3");
        h.run();
        h.key_press(egui::Key::Enter);
        h.run();
        for _ in 0..50 {
            if !exec.calls.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        // The result arrives from a thread, so the frames that follow depend on timing: step a fixed number of frames
        // instead of until the window settles within the default four.
        h.run_steps(10);
        assert_eq!(
            exec.calls.lock().unwrap().first().cloned(),
            Some(
                ["config", "set", "game.fallout3", "/g/fo3", "--json"]
                    .map(String::from)
                    .to_vec()
            )
        );
        assert!(
            h.query_by_label(SAVED_FALLOUT3).is_some(),
            "the footer says it was saved"
        );
    }
}
