// SPDX-License-Identifier: GPL-3.0-only
//! The TTW flow (UI-RULES P4, T5, K3, G6): Sources, Output, Review and Install, on the shell like the setup wizard.
//!
//! Every step is a `raul` command from the registry, shown on the page. The sources come from `raul config list`
//! and are changed in Settings, their home. Review runs `raul ttw --dry-run`: the package is read and both game
//! copies are checked, and nothing is written. Install runs `raul ttw` as a job: its progress shows with a
//! percentage, it keeps going when the page is left, and Cancel stops it. A failed or cancelled run leaves the
//! Instance as it was; Resume continues a stopped one with `--resume`.

use super::home::{Mark, icon, mark};
use super::settings::{Pending, Visits, error_block, read_config, start, waiting};
use super::{Fixture, Page, PageCx, ShellState};
use crate::catalogue::{self as msg, ErrorText};
use crate::shell;
use crate::theme::{self, Tokens};
use eframe::egui::{self, RichText};
use raul_ui_model::events::Status;
use raul_ui_model::job::{JobId, JobState, Outcome};
use raul_ui_model::pages::load_order::Problem;
use raul_ui_model::pages::settings::{Command, Fetch};
use raul_ui_model::pages::ttw::{
    Progress, Report, Sources, State, Step, check_command, install_command,
};
use raul_ui_model::pages::{Flow, PageId, View};
use raul_ui_model::wizard::{Step as FlowStep, TtwNeed};
use raul_ui_service::Output;
use raul_ui_service::dto::TtwReport;

crate::texts!(TEXTS {
    STATE_NO_INSTANCE = "No Instance yet";
    STATE_READING = "Reading the sources";
    STATE_STEP = "Step {n} of 4 · {name}";
    STATE_CHECKING = "Checking the sources";
    STATE_INSTALLING = "Installing";
    STATE_INSTALLING_AT = "Installing · {p}%";
    STATE_CANCELLING = "Cancelling";
    STATE_INSTALLED = "Installed";
    STATE_NOT_INSTALLED = "Not installed";
    STEP_SOURCES = "Sources";
    STEP_OUTPUT = "Output";
    STEP_REVIEW = "Review";
    STEP_INSTALL = "Install";
    NO_INSTANCE = "TTW goes into the Instance, which lives in the storage folder. Set the storage folder in Settings first.";
    OPEN_SETTINGS = "Settings";
    READING = "Reading the sources from the configuration";
    INTRO = "TTW is built from your own Fallout 3 and New Vegas copies. Both stay untouched.";
    SECTION_SOURCES = "Sources";
    LABEL_FALLOUT3 = "Fallout 3 folder";
    LABEL_NEW_VEGAS = "New Vegas folder";
    LABEL_PACKAGE = "TTW package";
    SET = "Set";
    NOT_SET = "Not set";
    FROM_INSTANCE = "From the Instance";
    NEW_VEGAS_DEFAULT = "The Instance's own copy";
    CHANGE = "Change in Settings";
    MISSING_BOTH = "To continue, set the Fallout 3 folder and the TTW package in Settings.";
    MISSING_FALLOUT3 = "To continue, set the Fallout 3 folder in Settings.";
    MISSING_PACKAGE = "To continue, set the TTW package in Settings.";
    SECTION_OUTPUT = "Output";
    FACT_INTO = "Installs into";
    FACT_AS = "Added as";
    ADDED_AS = "One Mod, with its plugins enabled in your load order";
    FACT_GAMES = "Game folders";
    GAMES_UNTOUCHED = "Not changed";
    FACT_SPACE = "Space";
    SPACE_NOTE = "Needs tens of GB free in the storage folder";
    CHECKING = "Checking: the package is read and both game copies are checked; nothing is written";
    SECTION_CHECKS = "Checks";
    CHECK_FALLOUT3 = "Fallout 3 checked";
    CHECK_NEW_VEGAS = "New Vegas checked";
    CHECK_PACKAGE = "Package read";
    SECTION_PLAN = "What happens";
    FACT_OUTPUT = "Output";
    FACT_TIME = "Time";
    TIME = "Long: often most of an hour";
    FACT_STOPS = "If it stops";
    STOPS = "The Instance stays as it was, and Resume continues the run";
    FACT_TOOL = "Installer";
    RESUMING = "Install continues the stopped run where it left off.";
    CHECK_AGAIN = "Check again";
    BACK = "Back";
    NEXT = "Next";
    INSTALL = "Install";
    RESUME = "Resume";
    TRY_AGAIN = "Try again";
    CANCEL = "Cancel";
    DONE = "Done";
    WAITS_FOR_CHECK = "Install waits for the checks.";
    PHASE_STARTING = "Starting";
    PHASE_WORKING = "Working";
    PHASE_PACKAGE = "Reading the package";
    PHASE_ASSETS = "Building the files";
    PHASE_ARCHIVES = "Packing the archives";
    PHASE_POST = "Finishing the files";
    PHASE_ADD_MOD = "Adding the Mod to the Instance";
    PROGRESS_OF = "{done} of {total}";
    LEAVE = "You can leave this page; the install keeps going.";
    INSTALLED = "TTW is installed as one Mod, and its plugins are enabled in your load order.";
    FACT_PLUGINS = "Plugins enabled";
    FACT_FILES = "Files";
    INSTALLED_STATUS = "TTW installed.";
    UNREADABLE = "The command finished, but what it printed could not be read.";
});

crate::errors!(ERRORS {
    SOURCES_UNREADABLE = (
        "The sources could not be read",
        "Nothing changed. The line below says why."
    );
    CHECK_FAILED = (
        "The checks did not pass",
        "Nothing changed. Fix what the line below names, then check again."
    );
    INSTALL_FAILED = (
        "TTW was not installed",
        "The Instance is as it was. The line below says why."
    );
    INSTALL_CANCELLED = (
        "The install was cancelled",
        "The Instance is as it was. Resume continues where it stopped."
    );
    INSTALL_UNKNOWN = (
        "The install ended without a result",
        "The state of the Instance is not known. Run the Doctor before you try again."
    );
    CANCEL_REFUSED = (
        "Cancel was not taken",
        "The install keeps going. Try again in a moment."
    );
});

/// The TTW flow.
#[derive(Debug, Default)]
pub struct Ttw {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
    /// The install's job, for its progress and Cancel.
    install: Option<JobId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Sources,
    /// The sources read again when the page is shown again; the flow stays unless they changed.
    Refresh,
    Check,
    Install,
}

impl Ttw {
    /// The flow in `state`, as a fixture shows it; it reads nothing by itself until its Instance changes.
    pub fn with(state: State) -> Ttw {
        let mut page = Ttw {
            state,
            ..Ttw::default()
        };
        page.visits.loaded = true;
        page
    }

    fn command(&self, job: Job) -> Option<Command> {
        let i = self.state.instance.as_deref()?;
        Some(match job {
            Job::Sources | Job::Refresh => Command::config_list(),
            Job::Check => check_command(i),
            Job::Install => install_command(i, self.state.resuming),
        })
    }

    /// Starts the command of `job`, after the state moved to the step it runs.
    fn run(&mut self, cx: &mut PageCx<'_>, job: Job) {
        let Some(command) = self.command(job) else {
            return;
        };
        let moved = match job {
            Job::Sources => {
                self.state.loading();
                Ok(())
            }
            Job::Refresh => Ok(()),
            Job::Check => self.state.checking(),
            Job::Install => self.state.installing(),
        };
        if moved.is_err() {
            return;
        }
        let problem = |e: ErrorText| Problem {
            id: e.id,
            cause: e.what.to_owned(),
            command: Some(command.shown()),
        };
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(job, id);
                if job == Job::Install {
                    self.install = Some(id);
                }
            }
            Err(e) => {
                let _ = match job {
                    Job::Sources => {
                        self.state.sources_failed(e.what.to_owned());
                        Ok(())
                    }
                    // The flow keeps the sources it has.
                    Job::Refresh => Ok(()),
                    Job::Check => self.state.check_failed(problem(e)),
                    Job::Install => self.state.install_failed(problem(e)),
                };
            }
        }
    }

    /// Follows the Instance the shell knows: a new one starts the flow over. A command that runs for the old one
    /// ends first.
    fn follow_instance(&mut self, cx: &mut PageCx<'_>) {
        if cx.shell.instance == self.state.instance || self.state.busy() {
            return;
        }
        self.state = State::new(cx.shell.instance.clone());
        self.install = None;
        if self.state.instance.is_some() {
            self.run(cx, Job::Sources);
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            let command = self.command(job).map(|c| c.shown());
            let problem = |e: &ErrorText, cause: String| Problem {
                id: e.id,
                cause,
                command: command.clone(),
            };
            let _ = match job {
                Job::Sources => {
                    match read_config(&result) {
                        Ok(c) => self.state.sources_read(Sources::of(&c)),
                        Err(why) => self.state.sources_failed(why),
                    }
                    Ok(())
                }
                Job::Refresh => {
                    if let Ok(c) = read_config(&result) {
                        self.state.refreshed(Sources::of(&c));
                    }
                    Ok(())
                }
                Job::Check => match report_of(&result) {
                    Ok(r) => {
                        let cause = r.message.clone().unwrap_or_default();
                        self.state.checked(r, problem(&CHECK_FAILED, cause))
                    }
                    Err(why) => self.state.check_failed(problem(&CHECK_FAILED, why)),
                },
                Job::Install => {
                    self.install = None;
                    match report_of(&result) {
                        Ok(r) => {
                            let e = match r.status {
                                Status::Cancelled => &INSTALL_CANCELLED,
                                _ => &INSTALL_FAILED,
                            };
                            let cause = r.message.clone().unwrap_or_default();
                            if r.status == Status::Success {
                                cx.shell.status = Some(INSTALLED_STATUS.to_owned());
                            }
                            self.state.installed(r, problem(e, cause))
                        }
                        Err(why) => self.state.install_failed(problem(&INSTALL_UNKNOWN, why)),
                    }
                }
            };
        }
    }

    /// Takes the install job's progress from the job table.
    fn follow_install(&mut self, cx: &PageCx<'_>) {
        let Some(view) = self.install.and_then(|id| cx.jobs.table().get(id)) else {
            return;
        };
        if matches!(view.state, JobState::Running | JobState::Cancelling) {
            self.state.progressed(Progress {
                phase: view.phase.clone(),
                done: view.done,
                total: view.total,
            });
        }
    }

    fn cancel(&mut self, cx: &mut PageCx<'_>) {
        let Some(id) = self.install else {
            return;
        };
        if cx.jobs.cancel(id).is_ok() {
            let _ = self.state.cancelling();
        } else {
            self.state.problem = Some(Problem {
                id: CANCEL_REFUSED.id,
                cause: String::new(),
                command: None,
            });
        }
    }

    fn problem_block(&self, ui: &mut egui::Ui, t: &Tokens) {
        let Some(p) = &self.state.problem else {
            return;
        };
        let e = ERRORS
            .iter()
            .chain(msg::SHELL_ERRORS)
            .chain(super::settings::ERRORS)
            .find(|e| e.id == p.id)
            .copied()
            .unwrap_or(INSTALL_FAILED);
        let said = (!p.cause.is_empty()).then_some(p.cause.as_str());
        error_block(ui, t, &e, said);
        if let Some(c) = &p.command {
            shell::command_line(ui, t, c);
        }
        ui.add_space(12.0);
    }

    fn sources(ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>, s: &Sources) {
        ui.label(RichText::new(INTRO).color(t.muted.color()));
        ui.add_space(12.0);
        shell::section_label(ui, t, SECTION_SOURCES, true);
        let width = ui.available_width();
        egui::Grid::new("ttw-sources")
            .num_columns(3)
            .spacing([18.0, 10.0])
            .min_row_height(28.0)
            .show(ui, |ui| {
                source_row(ui, t, width, LABEL_FALLOUT3, s.fallout3.as_deref(), false);
                source_row(ui, t, width, LABEL_NEW_VEGAS, s.newvegas.as_deref(), true);
                source_row(ui, t, width, LABEL_PACKAGE, s.package.as_deref(), false);
            });
        ui.add_space(12.0);
        if ui.button(CHANGE).clicked() {
            cx.shell.go_to = Some(View::Page(PageId::Settings));
        }
    }

    fn output(ui: &mut egui::Ui, t: &Tokens, instance: &str) {
        shell::section_label(ui, t, SECTION_OUTPUT, true);
        facts(
            ui,
            t,
            "ttw-output",
            &[
                (FACT_INTO, instance, true),
                (FACT_AS, ADDED_AS, false),
                (FACT_GAMES, GAMES_UNTOUCHED, false),
                (FACT_SPACE, SPACE_NOTE, false),
            ],
        );
    }

    fn review(&self, ui: &mut egui::Ui, t: &Tokens, instance: &str) {
        if self.state.wizard.flow.step() == FlowStep::Planning {
            waiting(ui, t, CHECKING, &check_command(instance));
            return;
        }
        if self.state.problem.is_some() {
            return;
        }
        let Some(check) = &self.state.check else {
            return;
        };
        shell::section_label(ui, t, SECTION_CHECKS, true);
        let sources = self.state.sources.ready().cloned().unwrap_or_default();
        for (line, detail) in [
            (CHECK_FALLOUT3, sources.fallout3.as_deref()),
            (
                CHECK_NEW_VEGAS,
                Some(sources.newvegas.as_deref().unwrap_or(NEW_VEGAS_DEFAULT)),
            ),
            (CHECK_PACKAGE, sources.package.as_deref()),
        ] {
            note(ui, t, Mark::Ok, line, detail);
        }
        ui.add_space(12.0);
        shell::section_label(ui, t, SECTION_PLAN, true);
        let mut rows = vec![
            (FACT_OUTPUT, instance, true),
            (FACT_TIME, TIME, false),
            (FACT_STOPS, STOPS, false),
        ];
        if let Some(tool) = &check.tool {
            rows.push((FACT_TOOL, tool.as_str(), true));
        }
        facts(ui, t, "ttw-plan", &rows);
        ui.add_space(8.0);
        if self.state.resuming {
            ui.label(RichText::new(RESUMING).color(t.muted.color()));
        }
        shell::command_line(
            ui,
            t,
            &install_command(instance, self.state.resuming).shown(),
        );
    }

    fn install(&self, ui: &mut egui::Ui, t: &Tokens, instance: &str) {
        if !self.state.done() {
            let p = self.state.progress.clone().unwrap_or_default();
            let bar = egui::ProgressBar::new(p.fraction().unwrap_or(0.0))
                .desired_width(ui.available_width().min(560.0))
                .fill(t.info.color());
            // Without an estimate the bar stays empty and the phase says what runs; it does not animate, so the
            // window only repaints when the job reports.
            let bar = if p.fraction().is_some() {
                bar.show_percentage()
            } else {
                bar
            };
            ui.add(bar);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(phase_words(&p.phase)).color(t.text.color()));
                if let Some(total) = p.total {
                    ui.label(
                        RichText::new(
                            PROGRESS_OF
                                .replace("{done}", &p.done.to_string())
                                .replace("{total}", &total.to_string()),
                        )
                        .color(t.muted.color()),
                    );
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new(LEAVE).color(t.muted.color()));
            shell::command_line(
                ui,
                t,
                &install_command(instance, self.state.resuming).shown(),
            );
            return;
        }
        if self.state.outcome() != Some(Outcome::Completed) {
            return;
        }
        note(ui, t, Mark::Ok, INSTALLED, None);
        ui.add_space(12.0);
        let Some(r) = &self.state.result else {
            return;
        };
        let files = r.files.map_or_else(String::new, |f| f.to_string());
        let mut rows = vec![(FACT_OUTPUT, instance, true)];
        if r.files.is_some() {
            rows.push((FACT_FILES, files.as_str(), false));
        }
        facts(ui, t, "ttw-installed", &rows);
        if !r.plugins.is_empty() {
            ui.add_space(12.0);
            shell::section_label(ui, t, FACT_PLUGINS, true);
            let width = ui.available_width();
            for p in &r.plugins {
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(width);
                        ui.add(
                            egui::Label::new(RichText::new(p).monospace().color(t.text.color()))
                                .truncate(),
                        )
                        .on_hover_text(p);
                    },
                );
            }
        }
    }

    /// The footer of the flow: the line that says what Next waits for, then Back and the step's one primary
    /// button (UI-RULES A1, G3).
    fn footer(&mut self, ui: &mut egui::Ui, t: &Tokens, cx: &mut PageCx<'_>) {
        ui.add_space(16.0);
        shell::hairline(ui, t);
        ui.add_space(10.0);
        let why = self.waits_for();
        ui.horizontal(|ui| {
            if let Some(why) = why {
                ui.add(egui::Label::new(RichText::new(why).color(t.muted.color())).wrap());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // The layout runs right to left: the primary button comes first.
                match self.state.step {
                    Step::Install if !self.state.done() => {
                        let cancelling = self.state.wizard.flow.step() == FlowStep::Cancelling;
                        if ui
                            .add_enabled(!cancelling, egui::Button::new(CANCEL))
                            .clicked()
                        {
                            self.cancel(cx);
                        }
                    }
                    Step::Install => {
                        if self.state.resumable() {
                            if ui.add(shell::primary_button(t, RESUME)).clicked() {
                                self.state.again(true);
                            }
                        } else if self.state.outcome() == Some(Outcome::Completed) {
                            if ui.add(shell::primary_button(t, DONE)).clicked() {
                                cx.shell.go_to = Some(View::Page(PageId::Home));
                            }
                        } else if ui.add(shell::primary_button(t, TRY_AGAIN)).clicked() {
                            self.state.again(false);
                        }
                    }
                    step => {
                        let label = match step {
                            Step::Review if self.state.resuming => RESUME,
                            Step::Review => INSTALL,
                            _ => NEXT,
                        };
                        let next =
                            ui.add_enabled(self.state.can_next(), shell::primary_button(t, label));
                        let next = match (step, self.state.instance.as_deref()) {
                            (Step::Review, Some(i)) => {
                                next.on_hover_text(install_command(i, self.state.resuming).shown())
                            }
                            _ => next,
                        };
                        if next.clicked() {
                            if step == Step::Review {
                                self.run(cx, Job::Install);
                            } else {
                                self.state.next();
                            }
                        }
                        if step == Step::Review
                            && self.state.problem.is_some()
                            && ui.button(CHECK_AGAIN).clicked()
                        {
                            self.run(cx, Job::Check);
                        }
                        if step != Step::Sources
                            && ui
                                .add_enabled(!self.state.busy(), egui::Button::new(BACK))
                                .clicked()
                        {
                            self.state.back();
                        }
                    }
                }
            });
        });
    }

    /// What a disabled Next waits for, in one line (UI-RULES G3).
    fn waits_for(&self) -> Option<&'static str> {
        match self.state.step {
            Step::Sources => {
                let missing = self.state.missing();
                match (
                    missing.contains(&TtwNeed::Fallout3),
                    missing.contains(&TtwNeed::Package),
                ) {
                    (true, true) => Some(MISSING_BOTH),
                    (true, false) => Some(MISSING_FALLOUT3),
                    (false, true) => Some(MISSING_PACKAGE),
                    (false, false) => None,
                }
            }
            Step::Review if !self.state.can_next() && self.state.problem.is_none() => {
                Some(WAITS_FOR_CHECK)
            }
            _ => None,
        }
    }
}

impl Page for Ttw {
    fn view(&self) -> View {
        View::Flow(Flow::Ttw)
    }

    fn title(&self) -> &'static str {
        msg::FLOW_TTW
    }

    fn state_line(&self) -> String {
        let s = &self.state;
        if s.instance.is_none() {
            return STATE_NO_INSTANCE.to_owned();
        }
        if s.sources.is_pending() {
            return STATE_READING.to_owned();
        }
        match s.wizard.flow.step() {
            FlowStep::Planning => return STATE_CHECKING.to_owned(),
            FlowStep::Cancelling => return STATE_CANCELLING.to_owned(),
            FlowStep::Applying => {
                return match s.progress.as_ref().and_then(Progress::fraction) {
                    Some(f) => STATE_INSTALLING_AT.replace("{p}", &percent(f).to_string()),
                    None => STATE_INSTALLING.to_owned(),
                };
            }
            FlowStep::Done if s.outcome() == Some(Outcome::Completed) => {
                return STATE_INSTALLED.to_owned();
            }
            FlowStep::Done => return STATE_NOT_INSTALLED.to_owned(),
            FlowStep::Input | FlowStep::Review => {}
        }
        STATE_STEP
            .replace("{n}", &s.step.number().to_string())
            .replace("{name}", step_name(s.step))
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let back = self.visits.returned(ui.ctx());
        self.follow_instance(cx);
        if !self.visits.loaded {
            self.visits.loaded = true;
            if self.state.sources == Fetch::Idle {
                self.run(cx, Job::Sources);
            }
        } else if back && self.state.sources.ready().is_some() && !self.state.busy() {
            // Shown again: the sources may have changed in Settings; the flow starts over only if they did.
            self.run(cx, Job::Refresh);
        }
        self.take_answers(cx);
        self.follow_install(cx);
        if self.state.needs_check() {
            self.run(cx, Job::Check);
        }
        let Some(instance) = self.state.instance.clone() else {
            ui.label(NO_INSTANCE);
            ui.add_space(8.0);
            if ui.button(OPEN_SETTINGS).clicked() {
                cx.shell.go_to = Some(View::Page(PageId::Settings));
            }
            return;
        };
        let sources = match &self.state.sources {
            Fetch::Idle | Fetch::Loading => {
                waiting(ui, &t, READING, &Command::config_list());
                return;
            }
            Fetch::Failed(why) => {
                error_block(ui, &t, &SOURCES_UNREADABLE, Some(why));
                shell::command_line(ui, &t, &Command::config_list().shown());
                ui.add_space(8.0);
                if ui.button(CHECK_AGAIN).clicked() {
                    self.run(cx, Job::Sources);
                }
                return;
            }
            Fetch::Ready(s) => s.clone(),
        };
        let installed = self.state.outcome() == Some(Outcome::Completed);
        steps(ui, &t, self.state.step, installed);
        ui.add_space(14.0);
        self.problem_block(ui, &t);
        match self.state.step {
            Step::Sources => Ttw::sources(ui, &t, cx, &sources),
            Step::Output => Ttw::output(ui, &t, &instance),
            Step::Review => self.review(ui, &t, &instance),
            Step::Install => self.install(ui, &t, &instance),
        }
        self.footer(ui, &t, cx);
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

/// The report a finished `raul ttw --json` printed, whatever its status, or why there is none.
fn report_of(result: &Result<Output, String>) -> Result<Report, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    let r = TtwReport::from_output(out).map_err(|e| {
        let what = e.view_error().what;
        if what.is_empty() {
            UNREADABLE.to_owned()
        } else {
            what
        }
    })?;
    Ok(Report {
        status: r.status,
        dry_run: r.dry_run,
        plugins: r.plugins_enabled,
        files: r.files,
        resumable: r.resumable,
        tool: r.tool.map(|t| match t.version {
            Some(v) => format!("{} {v}", t.name),
            None => t.name,
        }),
        code: r.error.as_ref().map(|e| e.code.clone()),
        message: r.error.map(|e| e.message),
    })
}

/// A share as a whole percentage, 0 to 100.
fn percent(fraction: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let p = (fraction.clamp(0.0, 1.0) * 100.0).round() as u8;
    p
}

fn step_name(step: Step) -> &'static str {
    match step {
        Step::Sources => STEP_SOURCES,
        Step::Output => STEP_OUTPUT,
        Step::Review => STEP_REVIEW,
        Step::Install => STEP_INSTALL,
    }
}

/// The words of a phase of the install's stream.
fn phase_words(phase: &str) -> &'static str {
    match phase {
        "package" => PHASE_PACKAGE,
        "assets" => PHASE_ASSETS,
        "archives" => PHASE_ARCHIVES,
        "post_install" => PHASE_POST,
        "add_mod" => PHASE_ADD_MOD,
        "" => PHASE_STARTING,
        _ => PHASE_WORKING,
    }
}

/// The four steps in a row that wraps: the one shown is marked, the ones before it are checked, and all four once
/// TTW is `installed`.
fn steps(ui: &mut egui::Ui, t: &Tokens, current: Step, installed: bool) {
    ui.horizontal_wrapped(|ui| {
        for step in Step::ALL {
            let here = step == current && !installed;
            let past = step.number() < current.number() || installed;
            let frame = egui::Frame::new()
                .fill(if here {
                    t.sel.color()
                } else {
                    egui::Color32::TRANSPARENT
                })
                .corner_radius(6)
                .inner_margin(egui::Margin::symmetric(10, 6));
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if past {
                        icon(ui, t, Mark::Ok);
                    }
                    let colour = if here || past { t.text } else { t.muted };
                    let mut name = RichText::new(step_name(step)).color(colour.color());
                    if here {
                        name = name.family(egui::FontFamily::Name(theme::SEMIBOLD.into()));
                    }
                    ui.label(name);
                });
            });
        }
    });
}

/// One source: its label, its value (a path, truncated with the whole in a tooltip) and its mark.
fn source_row(
    ui: &mut egui::Ui,
    t: &Tokens,
    width: f32,
    label: &str,
    value: Option<&str>,
    defaults: bool,
) {
    ui.label(RichText::new(label).color(t.text.color()));
    let value_width = (width * 0.5).clamp(120.0, 520.0);
    ui.allocate_ui_with_layout(
        egui::vec2(value_width, 20.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(value_width);
            match value {
                Some(v) => {
                    ui.add(
                        egui::Label::new(RichText::new(v).monospace().color(t.text.color()))
                            .truncate(),
                    )
                    .on_hover_text(v);
                }
                None if defaults => {
                    ui.label(RichText::new(NEW_VEGAS_DEFAULT).color(t.muted.color()));
                }
                None => {
                    ui.label(RichText::new(NOT_SET).color(t.muted.color()));
                }
            }
        },
    );
    match (value, defaults) {
        (Some(_), _) => mark(ui, t, Mark::Ok, SET),
        (None, true) => mark(ui, t, Mark::Unset, FROM_INSTANCE),
        (None, false) => mark(ui, t, Mark::Warning, NOT_SET),
    }
    ui.end_row();
}

/// Label and value rows; a path value is monospace and truncates with the whole in a tooltip.
fn facts(ui: &mut egui::Ui, t: &Tokens, id: &str, rows: &[(&str, &str, bool)]) {
    let width = (ui.available_width() * 0.6).clamp(120.0, 620.0);
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 8.0])
        .show(ui, |ui| {
            for (label, value, path) in rows {
                ui.label(RichText::new(*label).color(t.muted.color()));
                let text = if *path {
                    RichText::new(*value).monospace()
                } else {
                    RichText::new(*value)
                };
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(width);
                        ui.add(egui::Label::new(text.color(t.text.color())).truncate())
                            .on_hover_text(*value);
                    },
                );
                ui.end_row();
            }
        });
}

/// A note: a mark, a line and an optional detail (a path, truncated).
fn note(ui: &mut egui::Ui, t: &Tokens, m: Mark, line: &str, detail: Option<&str>) {
    ui.horizontal_top(|ui| {
        icon(ui, t, m);
        ui.vertical(|ui| {
            ui.label(RichText::new(line).color(t.text.color()));
            if let Some(d) = detail {
                ui.add(
                    egui::Label::new(RichText::new(d).monospace().color(t.muted.color()))
                        .truncate(),
                )
                .on_hover_text(d);
            }
        });
    });
}

const INSTANCE: &str = "/users/player/raul-storage/instance";
const FALLOUT3: &str = "/games/steamapps/common/Fallout 3 goty";
const PACKAGE: &str = "/users/player/Downloads/ttw-installer.mpi";

fn sources(fallout3: Option<&str>, package: Option<&str>) -> Sources {
    Sources {
        fallout3: fallout3.map(str::to_owned),
        newvegas: None,
        package: package.map(str::to_owned),
    }
}

fn at(step: Step, sources: Sources) -> State {
    let mut s = State::new(Some(INSTANCE.to_owned()));
    s.sources_read(sources);
    s.step = step;
    s
}

fn reviewed() -> State {
    let mut s = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
    let _ = s.checking();
    let _ = s.checked(
        Report {
            tool: Some("ttwright 0.3.0".to_owned()),
            ..Report::of(Status::Success, true)
        },
        problem(&CHECK_FAILED, ""),
    );
    s
}

fn installing(done: u64, total: Option<u64>) -> State {
    let mut s = reviewed();
    let _ = s.installing();
    s.progressed(Progress {
        phase: "assets".to_owned(),
        done,
        total,
    });
    s
}

fn problem(e: &ErrorText, cause: &str) -> Problem {
    Problem {
        id: e.id,
        cause: cause.to_owned(),
        command: Some(check_command(INSTANCE).shown()),
    }
}

fn fixture(name: &'static str, state: State) -> Fixture {
    Fixture {
        name,
        page: Box::new(Ttw::with(state)),
        shell: ShellState {
            instance: Some(INSTANCE.to_owned()),
            new_vegas: Some("/games/steamapps/common/Fallout New Vegas".to_owned()),
            ..ShellState::default()
        },
    }
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    let mut loading = State::new(Some(INSTANCE.to_owned()));
    loading.loading();
    let mut unreadable = State::new(Some(INSTANCE.to_owned()));
    unreadable.sources_failed("raul config: the configuration file is not valid TOML".to_owned());
    let mut checking = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
    let _ = checking.checking();
    let mut check_failed = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
    let _ = check_failed.checking();
    let _ = check_failed.checked(
        Report {
            code: Some("unsupported".to_owned()),
            message: Some("the package version is not supported by this installer".to_owned()),
            ..Report::of(Status::Unsupported, true)
        },
        problem(
            &CHECK_FAILED,
            "the package version is not supported by this installer",
        ),
    );
    let mut cancelling = installing(12, Some(48));
    let _ = cancelling.cancelling();
    let mut installed = installing(48, Some(48));
    let plugins: Vec<String> = (1..=40)
        .map(|i| format!("SampleWorldPart{i:02}.esm"))
        .collect();
    let _ = installed.installed(
        Report {
            plugins,
            files: Some(41_872),
            ..Report::of(Status::Success, false)
        },
        problem(&INSTALL_FAILED, ""),
    );
    let mut cancelled = installing(30, Some(48));
    let _ = cancelled.cancelling();
    let _ = cancelled.installed(
        Report {
            resumable: true,
            ..Report::of(Status::Cancelled, false)
        },
        Problem {
            command: Some(install_command(INSTANCE, false).shown()),
            ..problem(&INSTALL_CANCELLED, "cancelled on request")
        },
    );
    let mut unknown = installing(30, Some(48));
    let _ = unknown.install_failed(Problem {
        command: Some(install_command(INSTANCE, false).shown()),
        ..problem(
            &INSTALL_UNKNOWN,
            "`raul ttw` stopped before it said how it ended",
        )
    });
    let long_path = "/users/player/a folder with a very long name that keeps going on and on/and another one/Fallout 3 goty";
    vec![
        Fixture {
            name: "no-instance",
            page: Box::new(Ttw::with(State::default())),
            shell: ShellState::default(),
        },
        fixture("loading", loading),
        fixture("unreadable", unreadable),
        fixture("sources-missing", at(Step::Sources, sources(None, None))),
        fixture(
            "sources-ready",
            at(Step::Sources, sources(Some(FALLOUT3), Some(PACKAGE))),
        ),
        fixture(
            "long-text",
            at(Step::Sources, sources(Some(long_path), Some(PACKAGE))),
        ),
        fixture(
            "output",
            at(Step::Output, sources(Some(FALLOUT3), Some(PACKAGE))),
        ),
        fixture("checking", checking),
        fixture("check-failed", check_failed),
        fixture("review", reviewed()),
        fixture("installing", installing(12, Some(48))),
        fixture("installing-no-estimate", installing(0, None)),
        fixture("cancelling", cancelling),
        fixture("installed", installed),
        fixture("cancelled", cancelled),
        fixture("unknown", unknown),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(code: i64, stdout: &str) -> Output {
        Output {
            code: Some(code),
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    // The result document of the command; its `profile` field is internal and never shown.
    const RESULT: &str = concat!(
        r#"{"schema_version":1,"contract":"raul-events","seq":1,"command":"ttw","event":"result","result":{"schema_version":1,"contract":"raul-ttw-result","status":"cancelled","dry_run":false,"resumed":false,"instance":"/i","profile":"Default","mod":null,"plugins_enabled":[],"files":null,"staging":"/i/s","resumable":true,"tool":{"name":"ttwright","path":"/t","sha256":"00","version":"0.3.0"},"ttwright":null,"error":{"code":"cancelled","message":"stopped","ttwright_code":"cancelled"}}}"#,
        "\n{\"schema_version\":1,\"contract\":\"raul-events\",\"seq\":2,\"command\":\"ttw\",\"event\":\"finished\",\"status\":\"cancelled\",\"exit_code\":4,\"code\":\"cancelled\",\"message\":\"stopped\",\"resumable\":true,\"changed\":false}"
    );

    #[test]
    fn a_ttw_stream_becomes_the_report_the_page_shows() {
        let r = report_of(&Ok(output(4, RESULT))).unwrap();
        assert_eq!(r.status, Status::Cancelled);
        assert!(r.resumable);
        assert_eq!(r.tool.as_deref(), Some("ttwright 0.3.0"));
        assert_eq!(r.code.as_deref(), Some("cancelled"));
        assert_eq!(r.message.as_deref(), Some("stopped"));
        assert!(report_of(&Ok(output(0, "not a stream"))).is_err());
        assert_eq!(report_of(&Err("no raul".into())).unwrap_err(), "no raul");
    }

    #[test]
    fn the_header_follows_the_flow() {
        assert_eq!(Ttw::with(State::default()).state_line(), STATE_NO_INSTANCE);
        let s = at(Step::Sources, sources(None, None));
        assert_eq!(Ttw::with(s).state_line(), "Step 1 of 4 · Sources");
        assert_eq!(
            Ttw::with(installing(12, Some(48))).state_line(),
            "Installing · 25%"
        );
        assert_eq!(
            Ttw::with(installing(0, None)).state_line(),
            STATE_INSTALLING
        );
        let mut done = installing(1, Some(1));
        let _ = done.installed(
            Report::of(Status::Success, false),
            problem(&INSTALL_FAILED, ""),
        );
        assert_eq!(Ttw::with(done).state_line(), STATE_INSTALLED);
        assert_eq!(percent(2.0), 100);
        assert_eq!(phase_words("something new"), PHASE_WORKING);
        assert_eq!(phase_words(""), PHASE_STARTING);
    }

    #[test]
    fn next_says_what_it_waits_for() {
        let page = Ttw::with(at(Step::Sources, sources(None, Some(PACKAGE))));
        assert_eq!(page.waits_for(), Some(MISSING_FALLOUT3));
        let page = Ttw::with(at(Step::Sources, sources(Some(FALLOUT3), None)));
        assert_eq!(page.waits_for(), Some(MISSING_PACKAGE));
        let page = Ttw::with(at(Step::Sources, sources(None, None)));
        assert_eq!(page.waits_for(), Some(MISSING_BOTH));
        let page = Ttw::with(reviewed());
        assert_eq!(page.waits_for(), None);
        let mut checking = at(Step::Review, sources(Some(FALLOUT3), Some(PACKAGE)));
        let _ = checking.checking();
        assert_eq!(Ttw::with(checking).waits_for(), Some(WAITS_FOR_CHECK));
    }
}
