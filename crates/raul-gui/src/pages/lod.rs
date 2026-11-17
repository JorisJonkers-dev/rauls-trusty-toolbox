// SPDX-License-Identifier: GPL-3.0-only
//! The LOD page (UI-RULES P3, T5, K3, G6): the plan, the blocks with their status and progress, Generate, Stop, and
//! the output folder.
//!
//! The plan comes from `raul lod plan` and says what a run reads and writes; nothing is run for it. Generate starts
//! `raul lod generate`, which runs lodwright and keeps its output as one Mod of the Instance, enabled in the
//! Profile; a new run replaces that Mod. The run keeps going when the page is left, and Stop asks it to end, which
//! leaves the Instance as it was. The output folder is shown read only; its home is Settings.

use super::home::{Mark, icon, mark};
use super::settings::{Pending, Visits, error_block, failure_of, start, waiting};
use super::{Fixture, Page, PageCx, ShellState};
use crate::catalogue::{self as msg, ErrorText};
use crate::dialog::{Answer, Dialog};
use crate::shell;
use crate::theme::Tokens;
use eframe::egui::{self, RichText};
use raul_ui_model::events::Status;
use raul_ui_model::job::{JobId, JobState};
use raul_ui_model::pages::lod::{
    BlockStatus, MOD_NAME, Outcome, Phase, Plan, Problem, Progress, State, Written,
    generate_command, parse_plan, plan_command, read_result,
};
use raul_ui_model::pages::settings::Command;
use raul_ui_model::pages::{Attention, PageId, Severity, View};
use raul_ui_service::Output;

crate::texts!(TEXTS {
    STATE_NO_INSTANCE = "No Instance yet";
    STATE_PLANNING = "Reading the plan";
    STATE_NOT_GENERATED = "{n} of {m} blocks not generated this session";
    STATE_FAILED = "The last run did not finish";
    STATE_CURRENT = "All blocks up to date";
    STATE_GENERATING = "Generating · {p}% · you can leave this page";
    STATE_GENERATING_START = "Generating · you can leave this page";
    STATE_STOPPING = "Stopping";
    GENERATE = "Generate";
    CONFIRM_TITLE = "Generate LOD?";
    CONFIRM_COST = "lodwright reads {n} active plugins and writes LOD for them. This can take a long time; you can leave the page while it runs. The result replaces the Mod LOD (lodwright). Stop leaves the Instance as it was.";
    CONFIRM_COST_ONE = "lodwright reads 1 active plugin and writes LOD for it. This can take a long time; you can leave the page while it runs. The result replaces the Mod LOD (lodwright). Stop leaves the Instance as it was.";
    STOP = "Stop";
    NO_INSTANCE = "LOD is generated for the Instance, which lives in the storage folder. Set the storage folder in Settings first.";
    OPEN_SETTINGS = "Settings";
    SECTION_PLAN = "What a run does";
    SECTION_BLOCKS = "Blocks";
    READS = "Reads";
    READS_VALUE = "{n} active plugins from the Profile";
    READS_ONE = "1 active plugin from the Profile";
    WRITES = "Writes";
    KIND_TERRAIN = "Terrain";
    KIND_OBJECTS = "Objects";
    KIND_TREES = "Trees";
    KIND_NONE = "Nothing: every kind is switched off in the Profile's settings";
    KEEPS = "Keeps it as";
    KEEPS_VALUE = "One Mod, enabled in the Profile. A new run replaces it.";
    WORK_FOLDER = "Work folder";
    COL_BLOCK = "Block";
    COL_STATUS = "Status";
    COL_PROGRESS = "Progress";
    COL_FILES = "Files";
    EVERY_WORLD = "Every world with terrain";
    STATUS_NOT_GENERATED = "Not set";
    STATUS_BUILDING = "Building";
    STATUS_BUILT = "Generated";
    STATUS_CURRENT = "Up to date";
    STATUS_FAILED = "Problem";
    PROGRESS_TEXT = "{p}%";
    PROGRESS_WAITING = "Starting";
    FILES_VALUE = "{n} files";
    DONE_BUILT = "LOD generated: {n} files in the Mod.";
    DONE_CURRENT = "Nothing changed: the LOD output is current.";
    DONE_STATUS = "LOD generated.";
    CURRENT_STATUS = "LOD is up to date.";
    STOPPED_STATUS = "LOD generation stopped. Nothing changed.";
    OUTPUT_LABEL = "Output";
    PHASE = "Phase: {phase}";
    UNREADABLE = "The command finished, but what it printed could not be read.";
});

crate::errors!(ERRORS {
    PLAN_FAILED = (
        "The LOD plan could not be read",
        "Nothing changed. The line below says why."
    );
    GENERATE_FAILED = (
        "LOD was not generated",
        "The Instance is as it was. The line below says why."
    );
    GENERATE_REFUSED = (
        "LOD generation was refused",
        "Nothing changed. The line below says why."
    );
    GENERATE_STOPPED = (
        "LOD generation was stopped",
        "The Instance is as it was."
    );
    STOP_FAILED = (
        "The run could not be stopped",
        "It keeps going. Try again, or wait for it to end."
    );
});

const GENERATE_DIALOG: &str = "lod-generate";

/// The LOD page.
#[derive(Debug, Default)]
pub struct Lod {
    /// Its toolkit-free state.
    pub state: State,
    pending: Pending<Job>,
    visits: Visits,
    generate: Option<JobId>,
    /// The Instance the open Generate dialog names.
    asked: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Job {
    Plan,
    Generate,
}

impl Lod {
    /// The page in `state`, as a fixture shows it; it reads nothing by itself until its Instance changes.
    pub fn with(state: State) -> Lod {
        Lod {
            state,
            ..Lod::default()
        }
    }

    fn command(&self, job: Job) -> Option<Command> {
        let i = self.state.instance.as_deref();
        match job {
            Job::Plan => i.map(plan_command),
            Job::Generate => i.map(generate_command),
        }
    }

    fn run(&mut self, cx: &mut PageCx<'_>, job: Job) {
        // One Generate at a time: a second would lose the first one's Stop.
        if job == Job::Generate && self.generate.is_some() {
            return;
        }
        let Some(command) = self.command(job) else {
            return;
        };
        match job {
            Job::Plan => self.state.planning(),
            Job::Generate => self.state.generating(),
        }
        match start(cx, &command) {
            Ok(id) => {
                self.pending.add(job, id);
                if job == Job::Generate {
                    self.generate = Some(id);
                }
            }
            Err(e) => self.state.failed(Problem {
                id: e.id,
                cause: e.what.to_owned(),
                command: Some(command.shown()),
            }),
        }
    }

    /// Generate is long and replaces the Mod, so it asks once and states the cost (UI-RULES T4).
    fn ask_generate(&mut self, cx: &mut PageCx<'_>, instance: &str) {
        self.asked = Some(instance.to_owned());
        let plugins = self.state.plan.as_ref().map_or(0, |p| p.plugins);
        let cost = if plugins == 1 {
            CONFIRM_COST_ONE.to_owned()
        } else {
            CONFIRM_COST.replace("{n}", &plugins.to_string())
        };
        cx.dialogs.open(
            Dialog::confirm(GENERATE_DIALOG, CONFIRM_TITLE, cost, GENERATE)
                .with_command(generate_command(instance).shown()),
        );
    }

    fn stop(&mut self, cx: &mut PageCx<'_>) {
        let Some(id) = self.generate else {
            return;
        };
        match cx.jobs.cancel(id) {
            Ok(()) => self.state.stopping(),
            Err(_) => {
                self.state.problem = Some(Problem {
                    id: STOP_FAILED.id,
                    cause: String::new(),
                    command: None,
                });
            }
        }
    }

    /// Follows the Instance the shell knows: a new one is planned, none clears the page. A running command ends
    /// first.
    fn follow_instance(&mut self, cx: &mut PageCx<'_>) {
        if cx.shell.instance == self.state.instance || self.state.busy() {
            return;
        }
        self.state = State {
            instance: cx.shell.instance.clone(),
            ..State::default()
        };
        self.run(cx, Job::Plan);
    }

    fn track_progress(&mut self, cx: &PageCx<'_>) {
        let Some(job) = self.generate.and_then(|id| cx.jobs.table().get(id)) else {
            return;
        };
        if job.state == JobState::Running || job.state == JobState::Cancelling {
            self.state.progress = Progress {
                phase: job.phase.clone(),
                done: job.done,
                total: job.total,
            };
        }
    }

    fn take_answers(&mut self, cx: &mut PageCx<'_>) {
        for (job, result) in self.pending.finished(cx.jobs) {
            let command = self.command(job).map(|c| c.shown());
            match job {
                Job::Plan => match plan_answer(&result) {
                    Ok(plan) => self.state.planned(plan),
                    Err(cause) => self.state.failed(Problem {
                        id: PLAN_FAILED.id,
                        cause,
                        command,
                    }),
                },
                Job::Generate => {
                    self.generate = None;
                    let stopped = self.state.phase == Phase::Stopping;
                    let outcome = match &result {
                        Ok(out) => read_result(&out.stdout).map_err(|why| {
                            if out.code == Some(0) {
                                why
                            } else {
                                failure_of(out)
                            }
                        }),
                        Err(why) => Err(why.clone()),
                    };
                    match outcome {
                        Ok(o) => self.finished(cx, o, command),
                        Err(cause) => {
                            if stopped {
                                cx.shell.status = Some(STOPPED_STATUS.to_owned());
                            }
                            self.state.failed(Problem {
                                id: if stopped {
                                    GENERATE_STOPPED.id
                                } else {
                                    GENERATE_FAILED.id
                                },
                                cause,
                                command,
                            });
                        }
                    }
                }
            }
        }
    }

    fn finished(&mut self, cx: &mut PageCx<'_>, o: Outcome, command: Option<String>) {
        let problem = match o.status {
            Status::Success => {
                cx.shell.status = Some(
                    if o.up_to_date {
                        CURRENT_STATUS
                    } else {
                        DONE_STATUS
                    }
                    .to_owned(),
                );
                None
            }
            Status::Cancelled => {
                cx.shell.status = Some(STOPPED_STATUS.to_owned());
                Some(GENERATE_STOPPED)
            }
            Status::Refused | Status::Unsupported | Status::Unverifiable => Some(GENERATE_REFUSED),
            Status::Failed => Some(GENERATE_FAILED),
        };
        let cause = o
            .error
            .as_ref()
            .map_or_else(String::new, |(_, message)| message.clone());
        self.state.generated(o);
        if let Some(e) = problem {
            self.state.problem = Some(Problem {
                id: e.id,
                cause,
                command,
            });
        }
    }

    fn plan_section(ui: &mut egui::Ui, t: &Tokens, plan: &Plan) {
        shell::section_label(ui, t, SECTION_PLAN, true);
        let kinds: Vec<&str> = [
            (plan.terrain, KIND_TERRAIN),
            (plan.objects, KIND_OBJECTS),
            (plan.trees, KIND_TREES),
        ]
        .into_iter()
        .filter_map(|(on, k)| on.then_some(k))
        .collect();
        let writes = if kinds.is_empty() {
            KIND_NONE.to_owned()
        } else {
            kinds.join(", ")
        };
        let reads = if plan.plugins == 1 {
            READS_ONE.to_owned()
        } else {
            READS_VALUE.replace("{n}", &plan.plugins.to_string())
        };
        egui::Grid::new("lod-plan")
            .num_columns(2)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                let label = |ui: &mut egui::Ui, text: &str| {
                    ui.label(RichText::new(text).color(t.muted.color()));
                };
                label(ui, READS);
                ui.vertical(|ui| {
                    ui.label(reads);
                    path(ui, t, &plan.data_folder);
                });
                ui.end_row();
                label(ui, WRITES);
                ui.label(writes);
                ui.end_row();
                label(ui, KEEPS);
                ui.vertical(|ui| {
                    ui.label(RichText::new(MOD_NAME).strong());
                    ui.label(RichText::new(KEEPS_VALUE).color(t.muted.color()));
                });
                ui.end_row();
                label(ui, WORK_FOLDER);
                path(ui, t, &plan.output_dir);
                ui.end_row();
            });
    }

    fn blocks(&self, ui: &mut egui::Ui, t: &Tokens) {
        shell::section_label(ui, t, SECTION_BLOCKS, true);
        let progress = self.state.progress.percent();
        let width = ui.available_width();
        let name_width = (width * 0.3).clamp(120.0, 320.0);
        egui::Grid::new("lod-blocks")
            .num_columns(4)
            .spacing([18.0, 10.0])
            .min_row_height(28.0)
            .show(ui, |ui| {
                for h in [COL_BLOCK, COL_STATUS, COL_PROGRESS, COL_FILES] {
                    ui.label(RichText::new(h).color(t.muted.color()).small());
                }
                ui.end_row();
                for b in self.state.blocks() {
                    let name = b.world.as_deref().unwrap_or(EVERY_WORLD);
                    // A truncating label in a grid takes no width of its own: give the column its share.
                    ui.allocate_ui(egui::vec2(name_width, 28.0), |ui| {
                        ui.set_min_width(name_width);
                        ui.add(egui::Label::new(name).truncate())
                            .on_hover_text(name);
                    });
                    let (m, word) = match b.status {
                        BlockStatus::NotGenerated => (Mark::Unset, STATUS_NOT_GENERATED),
                        BlockStatus::Building => (Mark::Info, STATUS_BUILDING),
                        BlockStatus::Built => (Mark::Ok, STATUS_BUILT),
                        BlockStatus::UpToDate => (Mark::Ok, STATUS_CURRENT),
                        BlockStatus::Failed => (Mark::Problem, STATUS_FAILED),
                    };
                    mark(ui, t, m, word);
                    let (share, text) = match b.status {
                        BlockStatus::Building => match progress {
                            Some(p) => (
                                f32::from(p) / 100.0,
                                PROGRESS_TEXT.replace("{p}", &p.to_string()),
                            ),
                            None => (0.0, PROGRESS_WAITING.to_owned()),
                        },
                        BlockStatus::Built | BlockStatus::UpToDate => {
                            (1.0, PROGRESS_TEXT.replace("{p}", "100"))
                        }
                        BlockStatus::NotGenerated | BlockStatus::Failed => {
                            (0.0, PROGRESS_TEXT.replace("{p}", "0"))
                        }
                    };
                    ui.add(
                        egui::ProgressBar::new(share)
                            .desired_width((width * 0.32).clamp(120.0, 360.0))
                            .text(text),
                    );
                    let files = if b.files > 0 {
                        FILES_VALUE.replace("{n}", &b.files.to_string())
                    } else {
                        "—".to_owned()
                    };
                    ui.label(RichText::new(files).color(t.muted.color()));
                    ui.end_row();
                }
            });
    }
}

impl Page for Lod {
    fn view(&self) -> View {
        View::Page(PageId::Lod)
    }

    fn title(&self) -> &'static str {
        msg::PAGE_LOD
    }

    fn state_line(&self) -> String {
        let s = &self.state;
        match s.phase {
            Phase::NoInstance => STATE_NO_INSTANCE.to_owned(),
            Phase::Planning => STATE_PLANNING.to_owned(),
            Phase::Generating => match s.progress.percent() {
                Some(p) => STATE_GENERATING.replace("{p}", &p.to_string()),
                None => STATE_GENERATING_START.to_owned(),
            },
            Phase::Stopping => STATE_STOPPING.to_owned(),
            Phase::Planned => {
                let blocks = s.blocks();
                if blocks.iter().any(|b| b.status == BlockStatus::Failed) {
                    STATE_FAILED.to_owned()
                } else if s.not_current() == 0 {
                    STATE_CURRENT.to_owned()
                } else {
                    STATE_NOT_GENERATED
                        .replace("{n}", &s.not_current().to_string())
                        .replace("{m}", &blocks.len().to_string())
                }
            }
        }
    }

    fn attention(&self) -> Option<Attention> {
        if matches!(self.state.phase, Phase::Generating | Phase::Stopping) {
            return Some(Attention {
                severity: Severity::Info,
                count: 1,
            });
        }
        self.state
            .blocks()
            .iter()
            .any(|b| b.status == BlockStatus::Failed)
            .then_some(Attention {
                severity: Severity::Warning,
                count: 1,
            })
    }

    fn tools(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let Some(instance) = self.state.instance.clone() else {
            return;
        };
        match self.state.phase {
            Phase::Generating => {
                if ui
                    .button(STOP)
                    .on_hover_text(generate_command(&instance).shown())
                    .clicked()
                {
                    self.stop(cx);
                }
            }
            Phase::Planned
                if self.state.plan.is_some()
                    && ui
                        .add(shell::primary_button(&t, GENERATE))
                        .on_hover_text(generate_command(&instance).shown())
                        .clicked() =>
            {
                self.ask_generate(cx, &instance);
            }
            _ => {}
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        let back = self.visits.returned(ui.ctx());
        self.follow_instance(cx);
        if back && self.state.phase == Phase::Planned {
            self.run(cx, Job::Plan);
        }
        self.take_answers(cx);
        // The answer counts only for the Instance the dialog named, and only while nothing runs.
        if let Some(answer) = cx.dialogs.take_answer(GENERATE_DIALOG) {
            let asked = self.asked.take();
            if answer == Answer::Confirmed
                && self.state.phase == Phase::Planned
                && asked.is_some()
                && asked == self.state.instance
            {
                self.run(cx, Job::Generate);
            }
        }
        self.track_progress(cx);
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
                .unwrap_or(GENERATE_FAILED);
            error_block(
                ui,
                &t,
                &e,
                (!p.cause.is_empty()).then_some(p.cause.as_str()),
            );
            if let Some(c) = &p.command {
                shell::command_line(ui, &t, c);
            }
            ui.add_space(12.0);
        }
        match self.state.phase {
            Phase::Planning if self.state.plan.is_none() => {
                waiting(ui, &t, STATE_PLANNING, &plan_command(&instance));
                return;
            }
            Phase::Generating | Phase::Stopping => {
                let what = if self.state.progress.phase.is_empty() {
                    STATE_GENERATING_START.to_owned()
                } else {
                    PHASE.replace("{phase}", &self.state.progress.phase)
                };
                waiting(ui, &t, &what, &generate_command(&instance));
            }
            _ if self.state.problem.is_some() => {}
            _ => {
                if let Some(o) = self
                    .state
                    .outcome
                    .as_ref()
                    .filter(|o| o.status == Status::Success)
                {
                    let line = if o.up_to_date {
                        DONE_CURRENT.to_owned()
                    } else {
                        DONE_BUILT.replace("{n}", &o.files.unwrap_or_default().to_string())
                    };
                    ui.horizontal(|ui| {
                        icon(ui, &t, Mark::Ok);
                        ui.label(line);
                    });
                }
                shell::command_line(ui, &t, &generate_command(&instance).shown());
            }
        }
        ui.add_space(12.0);
        if let Some(plan) = self.state.plan.clone() {
            Self::plan_section(ui, &t, &plan);
            ui.add_space(16.0);
            self.blocks(ui, &t);
            ui.add_space(14.0);
        }
        self.footer(ui, cx);
    }

    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        TEXTS
    }

    fn errors(&self) -> &'static [ErrorText] {
        ERRORS
    }
}

impl Lod {
    /// The output folder, read only, with the way to its home page.
    fn footer(&self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let t = *cx.tokens;
        ui.horizontal_wrapped(|ui| {
            if let Some(plan) = &self.state.plan {
                ui.label(RichText::new(OUTPUT_LABEL).color(t.muted.color()));
                path(ui, &t, &plan.output_dir);
            }
            if ui.small_button(OPEN_SETTINGS).clicked() {
                cx.shell.go_to = Some(View::Page(PageId::Settings));
            }
        });
    }
}

/// The plan a finished `raul lod plan` printed, or why there is none: the command failed, or its answer is not a
/// plan.
fn plan_answer(result: &Result<Output, String>) -> Result<Plan, String> {
    let out = result.as_ref().map_err(Clone::clone)?;
    if out.code != Some(0) {
        return Err(failure_of(out));
    }
    parse_plan(&out.stdout).map_err(|_| UNREADABLE.to_owned())
}

/// A path in monospace that truncates, in full in its tooltip (UI-RULES A5).
fn path(ui: &mut egui::Ui, t: &Tokens, p: &str) {
    ui.add(egui::Label::new(RichText::new(p).monospace().color(t.text.color())).truncate())
        .on_hover_text(p);
}

const INSTANCE: &str = "/users/player/raul-storage/instance";

fn plan(worlds: &[&str]) -> Plan {
    Plan {
        plugins: 14,
        worlds: worlds.iter().map(|w| (*w).to_owned()).collect(),
        data_folder: format!("{INSTANCE}/.raul/staging/Default/Data"),
        output_dir: format!("{INSTANCE}/.raul/lod/out"),
        terrain: true,
        objects: true,
        trees: true,
    }
}

fn fixture(name: &'static str, state: State) -> Fixture {
    Fixture {
        name,
        page: Box::new(Lod::with(State {
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

fn planned(worlds: &[&str]) -> State {
    let mut s = State::default();
    s.planned(plan(worlds));
    s
}

fn outcome(status: Status, up_to_date: bool, worlds: &[&str]) -> Outcome {
    Outcome {
        status,
        up_to_date,
        dry_run: false,
        mod_name: (status == Status::Success).then(|| MOD_NAME.to_owned()),
        files: (status == Status::Success).then_some(26),
        worlds: worlds.iter().map(|w| (*w).to_owned()).collect(),
        written: if up_to_date || status != Status::Success {
            Vec::new()
        } else {
            worlds
                .iter()
                .map(|w| Written {
                    world: (*w).to_owned(),
                    kind: "terrain_mesh".to_owned(),
                    files: 13,
                    bytes: 40960,
                })
                .collect()
        },
        error: (status != Status::Success).then(|| {
            (
                "lodwright_failed".to_owned(),
                "lodwright could not read a terrain record of a synthetic plugin".to_owned(),
            )
        }),
    }
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    const WORLDS: &[&str] = &["NorthRidge", "OldHighway"];
    let mut generating = planned(WORLDS);
    generating.generating();
    generating.progress = Progress {
        phase: "terrain".to_owned(),
        done: 3,
        total: Some(8),
    };
    let mut built = planned(WORLDS);
    built.generated(outcome(Status::Success, false, WORLDS));
    let mut current = planned(WORLDS);
    current.generated(outcome(Status::Success, true, WORLDS));
    let mut failed = planned(WORLDS);
    let o = outcome(Status::Failed, false, WORLDS);
    let command = Some(generate_command(INSTANCE).shown());
    failed.generated(o.clone());
    failed.problem = Some(Problem {
        id: GENERATE_FAILED.id,
        cause: o.error.map_or_else(String::new, |(_, m)| m),
        command,
    });
    let long: Vec<String> = (1..=3)
        .map(|i| format!("ASyntheticWorldWithAVeryLongEditorIdThatKeepsGoing{i:02}"))
        .collect();
    let long: Vec<&str> = long.iter().map(String::as_str).collect();
    let mut long_state = planned(&long);
    if let Some(p) = &mut long_state.plan {
        p.output_dir = format!(
            "{INSTANCE}/a/very/long/folder/name/that/keeps/going/and/going/until/it/cannot/fit/.raul/lod/out"
        );
    }
    let many: Vec<String> = (1..=24).map(|i| format!("SyntheticWorld{i:02}")).collect();
    let many: Vec<&str> = many.iter().map(String::as_str).collect();
    let mut plan_failed = State::default();
    plan_failed.planning();
    plan_failed.failed(Problem {
        id: PLAN_FAILED.id,
        cause: "the Profile has no Staging tree; run `raul stage` first".to_owned(),
        command: Some(plan_command(INSTANCE).shown()),
    });
    vec![
        Fixture {
            name: "no-instance",
            page: Box::new(Lod::with(State::default())),
            shell: ShellState::default(),
        },
        fixture(
            "loading",
            State {
                phase: Phase::Planning,
                ..State::default()
            },
        ),
        fixture("planned", planned(&[])),
        fixture("generating", generating),
        fixture("generated", built),
        fixture("up-to-date", current),
        fixture("error", failed),
        fixture("plan-error", plan_failed),
        fixture("long-text", long_state),
        fixture("large-list", planned(&many)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_follows_the_blocks_and_the_run() {
        let mut s = planned(&["A", "B"]);
        assert_eq!(
            Lod::with(s.clone()).state_line(),
            "2 of 2 blocks not generated this session"
        );
        s.generating();
        assert_eq!(
            Lod::with(s.clone()).state_line(),
            "Generating · you can leave this page"
        );
        s.progress.total = Some(4);
        s.progress.done = 2;
        assert_eq!(
            Lod::with(s.clone()).state_line(),
            "Generating · 50% · you can leave this page"
        );
        assert_eq!(
            Lod::with(s.clone()).attention().map(|a| a.severity),
            Some(Severity::Info)
        );
        s.generated(outcome(Status::Success, false, &["A", "B"]));
        assert_eq!(Lod::with(s.clone()).state_line(), "All blocks up to date");
        assert!(Lod::with(s.clone()).attention().is_none());
        s.generated(outcome(Status::Failed, false, &["A", "B"]));
        assert_eq!(Lod::with(s).state_line(), "The last run did not finish");
    }

    #[test]
    fn a_plan_counts_only_when_it_exits_0_and_reads() {
        let p = plan(&["A"]);
        let document = serde_json::json!({
            "contract": "lodwright-request",
            "schema_version": 1,
            "data_folder": p.data_folder,
            "plugins": ["A.esp"],
            "worlds": p.worlds,
            "output_dir": p.output_dir,
            "terrain": { "enabled": true },
            "objects": { "enabled": true },
            "trees": { "enabled": true },
        });
        let ok = Ok(Output {
            code: Some(0),
            stdout: document.to_string(),
            stderr: String::new(),
        });
        assert_eq!(plan_answer(&ok).unwrap().plugins, 1);
        let unreadable = Ok(Output {
            code: Some(0),
            stdout: "not a plan".into(),
            stderr: String::new(),
        });
        assert_eq!(plan_answer(&unreadable).unwrap_err(), UNREADABLE);
        let refused = Ok(Output {
            code: Some(2),
            stdout: String::new(),
            stderr: "raul lod plan: refused: no Instance\n".into(),
        });
        assert!(plan_answer(&refused).unwrap_err().contains("no Instance"));
    }
}
