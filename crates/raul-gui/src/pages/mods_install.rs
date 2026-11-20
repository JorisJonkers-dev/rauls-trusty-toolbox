// SPDX-License-Identifier: GPL-3.0-only
//! Installing a Mod from an archive, on the Mods page: choose an archive (one found in the Downloads folder, or a
//! path), install it as one job that can be stopped, answer the FOMOD's questions in plain words when it has any, and
//! read the mod list again after. A stop or a refusal leaves the mod list as it was.

use super::PageCx;
use super::settings::{Pending, error_block, failure_of, start};
use crate::catalogue::ErrorText;
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::job::JobId;
use raul_ui_model::pages::mods_install::{Installed, Phase, State, install_command, is_archive};
use raul_ui_model::wizard::{AnswerError, FomodQuestion, GroupRule, OptionKind};
use raul_ui_service::dto::{parse_install_report, parse_questions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

crate::texts!(TEXTS {
    SECTION = "Install a Mod";
    INSTALL = "Install a Mod…";
    HINT = "Path of the archive";
    FOUND = "In your Downloads folder";
    NONE_FOUND = "No archives in your Downloads folder.";
    START = "Install";
    CLOSE = "Close";
    CANCEL = "Cancel";
    PLANNING = "Reading the archive";
    INSTALLING = "Installing the Mod";
    STOPPING = "Stopping. The mod list stays as it was.";
    QUESTIONS = "This Mod asks you to choose how to install it.";
    CHOOSE_ONE = "Choose one.";
    CHOOSE_ANY = "Choose any.";
    CHOOSE_AT_MOST_ONE = "Choose none or one.";
    CHOOSE_AT_LEAST_ONE = "Choose at least one.";
    CHOOSE_ALL = "All of these are installed.";
    NEEDED = "Needed";
    RECOMMENDED = "Recommended";
    DONE = "{name} is installed: {files} files.";
    DONE_PLUGINS = "{n} plugins are on.";
    TOO_FEW = "Choose at least {n} for {group}.";
    TOO_MANY = "Choose at most {n} for {group}.";
    CANNOT_USE = "{option} cannot be installed here.";
    LEAVE_OUT = "{option} has to stay on.";
    UNKNOWN = "{group} has no choice called {option}.";
    STATUS_DONE = "Mod installed.";
});

crate::errors!(ERRORS {
    INSTALL_FAILED = (
        "The Mod was not installed",
        "Nothing changed in your mod list. The line below says why."
    );
    ANSWERS_NOT_SAVED = (
        "Your choices could not be saved for the install",
        "Nothing changed in your mod list. The line below says why."
    );
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Plan,
    Install,
}

/// The install panel of the Mods page.
#[derive(Debug, Default)]
pub struct Panel {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Step>,
    running: Option<JobId>,
    /// The choices on screen: question id to the options chosen.
    chosen: BTreeMap<String, Vec<String>>,
    /// Why the choices were refused.
    refused: Option<String>,
    /// The answers file the install reads.
    answers: Option<PathBuf>,
}

/// What the panel hands the page after a frame.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    /// A Mod was installed: read the list again.
    pub installed: bool,
}

/// The archives in the player's Downloads folder, newest names last, at most `limit`.
pub fn downloads(limit: usize) -> Vec<String> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    let Some(home) = home else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(Path::new(&home).join("Downloads")) else {
        return Vec::new();
    };
    let mut found: Vec<(std::time::SystemTime, String)> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_str().is_some_and(is_archive))
        .filter_map(|e| {
            let at = e.metadata().ok()?.modified().ok()?;
            Some((at, e.path().to_str()?.to_owned()))
        })
        .collect();
    found.sort_by(|a, b| b.cmp(a));
    found.into_iter().take(limit).map(|(_, p)| p).collect()
}

impl Panel {
    /// A panel in `state`, as a fixture shows it.
    pub fn with(state: State) -> Panel {
        let mut panel = Panel {
            state,
            ..Panel::default()
        };
        panel.reset_choices();
        panel
    }

    fn reset_choices(&mut self) {
        self.chosen.clear();
        self.refused = None;
        if let Some(w) = &self.state.questions {
            for q in w.questions() {
                self.chosen.insert(q.id.clone(), w.selection(&q.id));
            }
        }
    }

    fn run(&mut self, cx: &mut PageCx<'_>, instance: &str, step: Step) {
        let answers = if step == Step::Install {
            self.answers
                .as_ref()
                .and_then(|p| p.to_str().map(str::to_owned))
        } else {
            None
        };
        let command = install_command(
            instance,
            self.state.archive_path(),
            answers.as_deref(),
            step == Step::Plan,
        );
        match step {
            Step::Plan => self.state.planning(),
            Step::Install => self.state.installing(),
        }
        match start(cx, &command) {
            Ok(id) => {
                self.running = Some(id);
                self.pending.add(step, id);
            }
            Err(e) => self.state.failed(e.what.to_owned()),
        }
    }

    fn write_answers(&mut self) -> Result<(), String> {
        let text = self.state.answers_text().unwrap_or_default();
        let path = std::env::temp_dir().join(format!("raul-answers-{}.json", std::process::id()));
        std::fs::write(&path, text).map_err(|e| e.to_string())?;
        self.answers = Some(path);
        Ok(())
    }

    fn drop_answers(&mut self) {
        if let Some(path) = self.answers.take() {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Takes in what the jobs of the panel printed.
    fn take_answers(&mut self, cx: &mut PageCx<'_>, instance: &str) -> Outcome {
        let mut outcome = Outcome::default();
        for (step, result) in self.pending.finished(cx.jobs) {
            self.running = None;
            let out = match result {
                Ok(out) => out,
                Err(why) => {
                    self.state.failed(why);
                    continue;
                }
            };
            match out.code {
                Some(0) if step == Step::Plan => {
                    // No questions: the archive installs as it is.
                    self.drop_answers();
                    self.run(cx, instance, Step::Install);
                }
                Some(0) => match parse_install_report(&out.stdout) {
                    Ok(report) => {
                        self.drop_answers();
                        self.state.done(Installed {
                            name: report.mod_name,
                            files: report.files.len(),
                            plugins: report.plugins,
                        });
                        cx.shell.status = Some(STATUS_DONE.to_owned());
                        outcome.installed = true;
                    }
                    Err(why) => self.state.failed(why),
                },
                Some(1) => match parse_questions(&out.stdout) {
                    Some(questions) => {
                        self.drop_answers();
                        self.state.asking(questions);
                        self.reset_choices();
                    }
                    None => self.state.failed(failure_of(&out)),
                },
                _ => {
                    self.drop_answers();
                    self.state.failed(failure_of(&out));
                }
            }
        }
        outcome
    }

    fn cancel(&mut self, cx: &mut PageCx<'_>) {
        if let Some(id) = self.running
            && cx.jobs.cancel(id).is_ok()
        {
            self.state.stopping();
        }
    }

    /// Checks the choices against each question's rule and starts the install when they hold.
    fn submit(&mut self, cx: &mut PageCx<'_>, instance: &str) {
        let Some(wizard) = self.state.questions.as_mut() else {
            return;
        };
        let ids: Vec<String> = wizard.questions().iter().map(|q| q.id.clone()).collect();
        for id in ids {
            let picked = self.chosen.get(&id).cloned().unwrap_or_default();
            let names: Vec<&str> = picked.iter().map(String::as_str).collect();
            if let Err(e) = wizard.answer(&id, &names) {
                let group = wizard
                    .questions()
                    .iter()
                    .find(|q| q.id == id)
                    .map_or(id.as_str(), |q| q.group.as_str())
                    .to_owned();
                self.refused = Some(refusal(&e, &group));
                return;
            }
        }
        self.refused = None;
        if let Err(why) = self.write_answers() {
            self.state.failed(why);
            self.state.phase = Phase::Failed;
            return;
        }
        self.run(cx, instance, Step::Install);
    }

    /// Draws the panel, and reads what its jobs finished.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        cx: &mut PageCx<'_>,
        t: &Tokens,
        instance: &str,
    ) -> Outcome {
        let outcome = self.take_answers(cx, instance);
        match self.state.phase {
            Phase::Closed => {
                if ui.button(INSTALL).clicked() {
                    self.state.open(downloads(8));
                }
            }
            Phase::Choosing | Phase::Failed => self.choosing(ui, cx, t, instance),
            Phase::Planning | Phase::Installing | Phase::Stopping => self.running_ui(ui, cx, t),
            Phase::Asking => self.asking(ui, cx, t, instance),
            Phase::Done => {
                shell::section_label(ui, t, SECTION, true);
                if let Some(i) = &self.state.installed {
                    let mut line = DONE
                        .replace("{name}", &i.name)
                        .replace("{files}", &i.files.to_string());
                    if !i.plugins.is_empty() {
                        line.push(' ');
                        line.push_str(&DONE_PLUGINS.replace("{n}", &i.plugins.len().to_string()));
                    }
                    ui.label(line);
                }
                ui.horizontal_wrapped(|ui| {
                    if ui.button(INSTALL).clicked() {
                        self.state.open(downloads(8));
                    }
                    if ui.button(CLOSE).clicked() {
                        self.state.close();
                    }
                });
            }
        }
        outcome
    }

    fn choosing(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, t: &Tokens, instance: &str) {
        shell::section_label(ui, t, SECTION, true);
        if self.state.phase == Phase::Failed
            && let Some(why) = &self.state.problem
        {
            error_block(ui, t, &INSTALL_FAILED, Some(why));
            ui.add_space(8.0);
        }
        ui.add(
            egui::TextEdit::singleline(&mut self.state.archive)
                .hint_text(HINT)
                .desired_width(f32::INFINITY),
        );
        ui.add_space(6.0);
        ui.label(RichText::new(FOUND).color(t.muted.color()));
        if self.state.found.is_empty() {
            ui.label(RichText::new(NONE_FOUND).color(t.muted.color()));
        }
        let mut picked = None;
        for path in &self.state.found {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);
            let selected = self.state.archive_path() == path;
            if ui
                .add(egui::Button::selectable(selected, name).truncate())
                .on_hover_text(path)
                .clicked()
            {
                picked = Some(path.clone());
            }
        }
        if let Some(p) = picked {
            self.state.pick(&p);
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(self.state.can_start(), shell::primary_button(t, START))
                .clicked()
            {
                self.run(cx, instance, Step::Plan);
            }
            if ui.button(CLOSE).clicked() {
                self.state.close();
            }
        });
    }

    fn running_ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, t: &Tokens) {
        shell::section_label(ui, t, SECTION, true);
        let line = match self.state.phase {
            Phase::Planning => PLANNING,
            Phase::Stopping => STOPPING,
            _ => INSTALLING,
        };
        ui.horizontal_wrapped(|ui| {
            super::home::icon(ui, t, super::home::Mark::Info);
            ui.label(RichText::new(line).color(t.muted.color()));
        });
        ui.add_space(6.0);
        let stopping = self.state.phase == Phase::Stopping;
        if ui
            .add_enabled(!stopping, egui::Button::new(CANCEL))
            .clicked()
        {
            self.cancel(cx);
        }
    }

    fn asking(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>, t: &Tokens, instance: &str) {
        shell::section_label(ui, t, SECTION, true);
        ui.label(QUESTIONS);
        ui.add_space(6.0);
        let questions: Vec<FomodQuestion> = self
            .state
            .questions
            .as_ref()
            .map_or_else(Vec::new, |w| w.questions().to_vec());
        for q in &questions {
            self.question(ui, t, q);
            ui.add_space(8.0);
        }
        if let Some(why) = &self.refused {
            ui.label(RichText::new(why).color(t.warn.color()));
            ui.add_space(6.0);
        }
        ui.horizontal_wrapped(|ui| {
            if ui.add(shell::primary_button(t, START)).clicked() {
                self.submit(cx, instance);
            }
            if ui.button(CANCEL).clicked() {
                self.state.close();
            }
        });
    }

    fn question(&mut self, ui: &mut egui::Ui, t: &Tokens, q: &FomodQuestion) {
        let title = if q.step.is_empty() || q.step == q.group {
            q.group.clone()
        } else {
            format!("{}: {}", q.step, q.group)
        };
        ui.label(RichText::new(title).strong());
        ui.label(
            RichText::new(rule_text(q.rule))
                .small()
                .color(t.muted.color()),
        );
        let one = matches!(
            q.rule,
            GroupRule::SelectExactlyOne | GroupRule::SelectAtMostOne
        );
        let picked = self.chosen.entry(q.id.clone()).or_default();
        for o in &q.options {
            let usable = o.kind != Some(OptionKind::NotUsable) && q.rule != GroupRule::SelectAll;
            let fixed = o.kind == Some(OptionKind::Required);
            let mut on = picked.contains(&o.name);
            let mut label = o.name.clone();
            match o.kind {
                Some(OptionKind::Required) => {
                    label.push_str(" (");
                    label.push_str(NEEDED);
                    label.push(')');
                }
                Some(OptionKind::Recommended) => {
                    label.push_str(" (");
                    label.push_str(RECOMMENDED);
                    label.push(')');
                }
                _ => {}
            }
            let before = on;
            let response = ui
                .add_enabled(usable && !fixed, egui::Checkbox::new(&mut on, label))
                .on_hover_text(&o.description);
            if !o.description.is_empty() {
                ui.label(RichText::new(&o.description).small().color(t.muted.color()));
            }
            if response.changed() && on != before {
                if on {
                    if one {
                        picked.clear();
                    }
                    picked.push(o.name.clone());
                } else {
                    picked.retain(|n| n != &o.name);
                }
            }
        }
    }
}

fn rule_text(rule: GroupRule) -> &'static str {
    match rule {
        GroupRule::SelectAny => CHOOSE_ANY,
        GroupRule::SelectAll => CHOOSE_ALL,
        GroupRule::SelectExactlyOne => CHOOSE_ONE,
        GroupRule::SelectAtMostOne => CHOOSE_AT_MOST_ONE,
        GroupRule::SelectAtLeastOne => CHOOSE_AT_LEAST_ONE,
    }
}

/// What a refused choice says, in plain words.
fn refusal(e: &AnswerError, group: &str) -> String {
    match e {
        AnswerError::TooFew { min } => TOO_FEW
            .replace("{n}", &min.to_string())
            .replace("{group}", group),
        AnswerError::TooMany { max } => TOO_MANY
            .replace("{n}", &max.to_string())
            .replace("{group}", group),
        AnswerError::NotUsable(o) => CANNOT_USE.replace("{option}", o),
        AnswerError::MissingRequired(o) => LEAVE_OUT.replace("{option}", o),
        AnswerError::UnknownOption(o) => UNKNOWN.replace("{option}", o).replace("{group}", group),
        AnswerError::UnknownQuestion(id) => {
            UNKNOWN.replace("{option}", id).replace("{group}", group)
        }
    }
}

/// The errors of the panel, for the page's catalogue.
pub const PANEL_ERRORS: &[ErrorText] = ERRORS;

/// The strings of the panel, for the page's catalogue.
pub const PANEL_TEXTS: &[(&str, &str)] = TEXTS;
