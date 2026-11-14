// SPDX-License-Identifier: GPL-3.0-only
//! The real `nvmod setup --json` event stream and the setup wizard it drives.
//!
//! The stream is what `nvmod-cli`'s `setup.rs` prints, one JSON object per line, each with an increasing `seq`
//! that starts at 1 in every invocation:
//!
//! | `event` | when | fields used here |
//! | --- | --- | --- |
//! | `plan` | first, before any step | `dry_run`, `resume`, `instance`, `steps[{step, description}]` |
//! | `step` | per step | `step`, `status` (`planned`, `started`, `done`, `failed`), `index`, `total`, `description`, `code`, `detail` |
//! | `resumed` | a resume, after `plan` | `total`, `completed[]`, `next` |
//! | `skipped` | a resume, per finished step | `step`, `index`, `total`, `reason` |
//! | `finished` | last | `status` (`ok`, `dry-run`, `failed`, `refused`, `restarted`), `exit_code`, `rolled_back`, `resumable`, `failed_step`, `reason` |
//!
//! A refusal before the plan (and `--restart`) prints a `finished` event alone, with `seq` 1. One wizard follows
//! one invocation: a resume is a new invocation with its own `plan` and a new wizard.

use crate::error::{ExitClass, ViewError};
use crate::job::{JobEvent, JobId, Outcome};
use serde::{Deserialize, Serialize};

/// A step of the plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlannedStep {
    /// The step name (`init`, `nv-rs`, `sort`, ...).
    pub step: String,
    /// What it does, in words.
    pub description: String,
}

/// The `status` of a `step` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepEventStatus {
    /// Listed by a dry run.
    Planned,
    /// Started.
    Started,
    /// Finished and verified.
    Done,
    /// Failed.
    Failed,
    /// A status this build does not know.
    #[serde(other)]
    Unknown,
}

/// The `status` of a `finished` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishStatus {
    /// Everything ran.
    #[serde(rename = "ok")]
    Ok,
    /// A dry run.
    #[serde(rename = "dry-run")]
    DryRun,
    /// A step failed or a resume could not verify an earlier step.
    #[serde(rename = "failed")]
    Failed,
    /// Refused before anything was written.
    #[serde(rename = "refused")]
    Refused,
    /// `--restart` removed a partial Instance.
    #[serde(rename = "restarted")]
    Restarted,
    /// A status this build does not know.
    #[serde(other)]
    Unknown,
}

/// One event of the stream.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SetupEvent {
    /// The plan.
    Plan {
        /// Nothing is written.
        dry_run: bool,
        /// A resume of an earlier run.
        #[serde(default)]
        resume: bool,
        /// The Instance path.
        #[serde(default)]
        instance: Option<String>,
        /// The steps, in order.
        steps: Vec<PlannedStep>,
    },
    /// A step changed state.
    Step {
        /// The step name.
        step: String,
        /// The new state.
        status: StepEventStatus,
        /// 1-based position.
        index: u64,
        /// Steps in all.
        total: u64,
        /// What it does.
        #[serde(default)]
        description: String,
        /// The exit code of a finished or failed step.
        #[serde(default)]
        code: Option<i64>,
        /// The output of the step.
        #[serde(default)]
        detail: Option<String>,
    },
    /// A resume found earlier work.
    Resumed {
        /// Steps in all.
        total: u64,
        /// Steps that finished earlier.
        #[serde(default)]
        completed: Vec<String>,
        /// The step that runs next.
        #[serde(default)]
        next: Option<String>,
    },
    /// A resume skips a finished step.
    Skipped {
        /// The step name.
        step: String,
        /// 1-based position.
        index: u64,
        /// Steps in all.
        total: u64,
        /// Why it is skipped.
        #[serde(default)]
        reason: String,
    },
    /// The run is over.
    Finished {
        /// How.
        status: FinishStatus,
        /// The exit code.
        exit_code: i64,
        /// The run was undone.
        #[serde(default)]
        rolled_back: bool,
        /// A resume can continue it.
        #[serde(default)]
        resumable: bool,
        /// The step that failed.
        #[serde(default)]
        failed_step: Option<String>,
        /// Why a refusal or a stop happened.
        #[serde(default)]
        reason: Option<String>,
    },
    /// An event this build does not know: ignored, so a newer CLI does not break an older GUI.
    #[serde(other)]
    Unknown,
}

/// One line of the stream: the `seq` and the event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamLine {
    /// Order within the invocation, from 1.
    pub seq: u64,
    /// The event.
    pub event: SetupEvent,
}

/// A line that is not a valid event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    /// Not JSON (a truncated or noisy line).
    NotJson(String),
    /// JSON without a numeric `seq`.
    NoSeq,
    /// JSON that does not fit the event.
    Shape(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotJson(e) => write!(f, "a line of the setup stream is not JSON: {e}"),
            Self::NoSeq => write!(f, "a line of the setup stream has no seq"),
            Self::Shape(e) => write!(f, "a line of the setup stream has an unexpected shape: {e}"),
        }
    }
}

impl std::error::Error for StreamError {}

/// Parses one line of `nvmod setup --json`.
///
/// # Errors
/// [`StreamError`] for a line that is not an event with a `seq`.
pub fn parse_line(line: &str) -> Result<StreamLine, StreamError> {
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|e| StreamError::NotJson(e.to_string()))?;
    let seq = value
        .get("seq")
        .and_then(serde_json::Value::as_u64)
        .ok_or(StreamError::NoSeq)?;
    let event = serde_json::from_value(value).map_err(|e| StreamError::Shape(e.to_string()))?;
    Ok(StreamLine { seq, event })
}

/// The state of one step in the wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepState {
    /// Not reached.
    Pending,
    /// Listed by a dry run.
    Planned,
    /// Running now.
    Running,
    /// Finished.
    Done,
    /// Finished in an earlier run and still holds.
    Skipped,
    /// Failed.
    Failed,
}

/// A step as the wizard shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepView {
    /// The step name.
    pub name: String,
    /// What it does.
    pub description: String,
    /// Where it is.
    pub state: StepState,
    /// The step's output, when it has one.
    pub detail: Option<String>,
}

/// How the wizard stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// No event yet.
    Waiting,
    /// The plan is known and nothing ran.
    Planned,
    /// A step ran or is running.
    Running,
    /// Over.
    Finished(Finish),
}

/// The end of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finish {
    /// How it ended.
    pub status: FinishStatus,
    /// The exit class.
    pub exit: ExitClass,
    /// The run was undone: nothing is left behind.
    pub rolled_back: bool,
    /// `nvmod setup --resume` can continue.
    pub resumable: bool,
    /// The failed step.
    pub failed_step: Option<String>,
    /// The reason of a refusal or stop.
    pub reason: Option<String>,
}

/// What the wizard offers after the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Next {
    /// Nothing yet.
    Nothing,
    /// The Instance is ready: go to Home.
    Done,
    /// Run again without `--dry-run`.
    RunForReal,
    /// Offer `--resume` and `--restart`.
    ResumeOrRestart,
    /// Show the reason and let the user change the inputs.
    ChangeInputs,
    /// Start a new setup.
    StartAgain,
}

/// Whether an event changed the wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// It did.
    Yes,
    /// It was stale, repeated or unknown, and was dropped.
    Dropped,
}

/// The setup wizard: the plan, the steps and how the run ended, fed from the event stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupWizard {
    phase: Phase,
    dry_run: bool,
    resume: bool,
    instance: Option<String>,
    steps: Vec<StepView>,
    last_seq: u64,
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self {
            phase: Phase::Waiting,
            dry_run: false,
            resume: false,
            instance: None,
            steps: Vec::new(),
            last_seq: 0,
        }
    }
}

impl SetupWizard {
    /// A wizard for one invocation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Where the wizard stands.
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// The steps, in plan order.
    pub fn steps(&self) -> &[StepView] {
        &self.steps
    }

    /// The run is a dry run.
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// The run is a resume.
    pub fn is_resume(&self) -> bool {
        self.resume
    }

    /// The Instance path from the plan.
    pub fn instance(&self) -> Option<&str> {
        self.instance.as_deref()
    }

    /// Steps done or skipped, and steps in all.
    pub fn progress(&self) -> (usize, usize) {
        let done = self
            .steps
            .iter()
            .filter(|s| matches!(s.state, StepState::Done | StepState::Skipped))
            .count();
        (done, self.steps.len())
    }

    /// The text form of the progress, for the live region.
    pub fn progress_text(&self) -> String {
        let (d, t) = self.progress();
        if t == 0 {
            "no estimate yet".to_owned()
        } else {
            format!("{d} of {t}")
        }
    }

    /// The step running now.
    pub fn current(&self) -> Option<&StepView> {
        self.steps.iter().find(|s| s.state == StepState::Running)
    }

    /// Parses and applies one line.
    ///
    /// # Errors
    /// [`StreamError`] when the line is not an event; the wizard is unchanged.
    pub fn feed_line(&mut self, line: &str) -> Result<Applied, StreamError> {
        Ok(self.apply(&parse_line(line)?))
    }

    /// Applies one event. A repeated, stale or unknown event is dropped, and so is anything after `finished`.
    pub fn apply(&mut self, line: &StreamLine) -> Applied {
        if line.seq <= self.last_seq
            || matches!(self.phase, Phase::Finished(_))
            || line.event == SetupEvent::Unknown
        {
            return Applied::Dropped;
        }
        self.last_seq = line.seq;
        match &line.event {
            SetupEvent::Plan {
                dry_run,
                resume,
                instance,
                steps,
            } => {
                self.dry_run = *dry_run;
                self.resume = *resume;
                self.instance.clone_from(instance);
                self.steps = steps
                    .iter()
                    .map(|s| StepView {
                        name: s.step.clone(),
                        description: s.description.clone(),
                        state: StepState::Pending,
                        detail: None,
                    })
                    .collect();
                self.phase = Phase::Planned;
            }
            SetupEvent::Step {
                step,
                status,
                description,
                detail,
                ..
            } => {
                let state = match status {
                    StepEventStatus::Planned => StepState::Planned,
                    StepEventStatus::Started => StepState::Running,
                    StepEventStatus::Done => StepState::Done,
                    StepEventStatus::Failed => StepState::Failed,
                    StepEventStatus::Unknown => return Applied::Dropped,
                };
                let view = self.step_mut(step, description);
                view.state = state;
                if detail.as_deref().is_some_and(|d| !d.is_empty()) {
                    view.detail.clone_from(detail);
                }
                if state != StepState::Planned {
                    self.phase = Phase::Running;
                }
            }
            SetupEvent::Resumed { .. } => self.phase = Phase::Running,
            SetupEvent::Skipped { step, reason, .. } => {
                let view = self.step_mut(step, "");
                view.state = StepState::Skipped;
                view.detail = Some(reason.clone()).filter(|r| !r.is_empty());
                self.phase = Phase::Running;
            }
            SetupEvent::Finished {
                status,
                exit_code,
                rolled_back,
                resumable,
                failed_step,
                reason,
            } => {
                self.phase = Phase::Finished(Finish {
                    status: *status,
                    exit: ExitClass::from_code(*exit_code),
                    rolled_back: *rolled_back,
                    resumable: *resumable,
                    failed_step: failed_step.clone(),
                    reason: reason.clone(),
                });
            }
            SetupEvent::Unknown => return Applied::Dropped,
        }
        Applied::Yes
    }

    fn step_mut(&mut self, name: &str, description: &str) -> &mut StepView {
        if let Some(i) = self.steps.iter().position(|s| s.name == name) {
            return &mut self.steps[i];
        }
        self.steps.push(StepView {
            name: name.to_owned(),
            description: description.to_owned(),
            state: StepState::Pending,
            detail: None,
        });
        self.steps.last_mut().expect("just pushed")
    }

    /// What to offer once the run is over.
    pub fn next(&self) -> Next {
        let Phase::Finished(f) = &self.phase else {
            return Next::Nothing;
        };
        match f.status {
            FinishStatus::Ok => Next::Done,
            FinishStatus::DryRun => Next::RunForReal,
            FinishStatus::Refused => Next::ChangeInputs,
            FinishStatus::Failed | FinishStatus::Unknown if f.resumable => Next::ResumeOrRestart,
            FinishStatus::Restarted | FinishStatus::Failed | FinishStatus::Unknown => {
                Next::StartAgain
            }
        }
    }

    /// The error card for a run that did not succeed, `None` for success and dry runs.
    pub fn error(&self) -> Option<ViewError> {
        let Phase::Finished(f) = &self.phase else {
            return None;
        };
        if matches!(
            f.status,
            FinishStatus::Ok | FinishStatus::DryRun | FinishStatus::Restarted
        ) {
            return None;
        }
        let step = f.failed_step.as_deref();
        let what = match (step, &f.reason) {
            (Some(s), _) => {
                let detail = self
                    .steps
                    .iter()
                    .find(|v| v.name == s)
                    .and_then(|v| v.detail.as_deref())
                    .unwrap_or("");
                format!("step `{s}` failed. {detail}").trim_end().to_owned()
            }
            (None, Some(r)) => r.clone(),
            (None, None) => f.exit.words().to_owned(),
        };
        let effect = if f.rolled_back {
            "Rolled back: the Instance was removed and nothing is left behind.".to_owned()
        } else if f.resumable {
            "The Instance is kept; the steps that finished stay finished.".to_owned()
        } else if f.exit == ExitClass::Refused {
            "Nothing was changed.".to_owned()
        } else {
            "The Instance may be left partly made; check it before starting again.".to_owned()
        };
        let mut e = ViewError::new(f.exit, what, effect);
        e.retry = if f.resumable {
            crate::error::Retry::Yes
        } else if f.exit == ExitClass::Refused {
            crate::error::Retry::AfterFix("change the inputs the reason names".to_owned())
        } else {
            crate::error::Retry::No
        };
        Some(e)
    }

    /// The command lines the footer offers for the current state (`--json` is the GUI's own).
    pub fn equivalent_command(&self) -> String {
        match (self.resume, self.dry_run, self.instance.as_deref()) {
            (true, _, Some(i)) => format!("nvmod setup --resume --instance {i}"),
            (_, true, Some(i)) => format!("nvmod setup --instance {i} --dry-run"),
            (_, _, Some(i)) => format!("nvmod setup --instance {i}"),
            _ => "nvmod setup".to_owned(),
        }
    }
}

/// Maps one line of the stream onto the job event of document 04 (at most one, so the line's `seq` orders it).
pub fn to_job_event(id: JobId, line: &StreamLine) -> Option<JobEvent> {
    let seq = line.seq;
    match &line.event {
        SetupEvent::Plan { dry_run, .. } => Some(JobEvent::Started {
            id,
            op: "setup".to_owned(),
            writes: !dry_run,
            // A dry run is over in an instant; a real run can be stopped.
            cancellable: !dry_run,
        }),
        SetupEvent::Step {
            step,
            status,
            index,
            total,
            description,
            detail,
            ..
        } => {
            let (done, message) = match status {
                StepEventStatus::Started => (index.saturating_sub(1), description.clone()),
                StepEventStatus::Done => (*index, detail.clone().unwrap_or_default()),
                StepEventStatus::Failed => (
                    index.saturating_sub(1),
                    format!("failed: {}", detail.clone().unwrap_or_default()),
                ),
                StepEventStatus::Planned => {
                    return Some(JobEvent::Log {
                        id,
                        seq,
                        line: format!("{step}: {description}"),
                    });
                }
                StepEventStatus::Unknown => return None,
            };
            Some(JobEvent::Progress {
                id,
                seq,
                phase: step.clone(),
                done,
                total: Some(*total),
                message: Some(message).filter(|m| !m.is_empty()),
            })
        }
        SetupEvent::Resumed {
            total, completed, ..
        } => Some(JobEvent::Progress {
            id,
            seq,
            phase: "resumed".to_owned(),
            done: completed.len() as u64,
            total: Some(*total),
            message: None,
        }),
        SetupEvent::Skipped {
            step,
            index,
            total,
            reason,
        } => Some(JobEvent::Progress {
            id,
            seq,
            phase: step.clone(),
            done: *index,
            total: Some(*total),
            message: Some(format!("skipped: {reason}")),
        }),
        SetupEvent::Finished {
            status,
            exit_code,
            rolled_back,
            resumable,
            ..
        } => {
            let outcome = match status {
                FinishStatus::Ok | FinishStatus::Restarted => Outcome::Completed,
                FinishStatus::DryRun => Outcome::DryRun,
                FinishStatus::Refused => Outcome::Refused,
                FinishStatus::Failed | FinishStatus::Unknown if *rolled_back => Outcome::RolledBack,
                FinishStatus::Failed | FinishStatus::Unknown if *resumable => Outcome::KeptPartial,
                FinishStatus::Failed | FinishStatus::Unknown => Outcome::Failed,
            };
            Some(JobEvent::Finished {
                id,
                seq,
                outcome,
                exit: ExitClass::from_code(*exit_code),
            })
        }
        SetupEvent::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::{JobState, JobTable};

    const PLAN: &str = r#"{"seq":1,"event":"plan","dry_run":false,"resume":false,"instance":"/i","steps":[{"step":"init","description":"make it"},{"step":"nv-rs","description":"install nv-rs"},{"step":"verify","description":"doctor"}]}"#;

    fn step(seq: u64, name: &str, status: &str, index: u64) -> String {
        format!(
            r#"{{"seq":{seq},"event":"step","step":"{name}","status":"{status}","index":{index},"total":3,"description":"d","code":0,"detail":""}}"#
        )
    }

    fn feed(w: &mut SetupWizard, lines: &[String]) {
        for l in lines {
            w.feed_line(l).unwrap();
        }
    }

    #[test]
    fn a_full_successful_run() {
        let mut w = SetupWizard::new();
        assert_eq!(w.next(), Next::Nothing);
        assert_eq!(w.progress_text(), "no estimate yet");
        w.feed_line(PLAN).unwrap();
        assert_eq!(*w.phase(), Phase::Planned);
        assert_eq!(w.instance(), Some("/i"));
        assert_eq!(w.progress_text(), "0 of 3");
        feed(&mut w, &[step(2, "init", "started", 1)]);
        assert_eq!(w.current().unwrap().name, "init");
        assert_eq!(*w.phase(), Phase::Running);
        feed(
            &mut w,
            &[
                step(3, "init", "done", 1),
                step(4, "nv-rs", "started", 2),
                step(5, "nv-rs", "done", 2),
                step(6, "verify", "started", 3),
                step(7, "verify", "done", 3),
                r#"{"seq":8,"event":"finished","status":"ok","exit_code":0,"rolled_back":false,"resumable":false,"instance":"/i"}"#.to_owned(),
            ],
        );
        assert_eq!(w.progress(), (3, 3));
        assert!(w.current().is_none());
        assert_eq!(w.next(), Next::Done);
        assert!(w.error().is_none());
        assert_eq!(w.equivalent_command(), "nvmod setup --instance /i");
        // Nothing revives a finished wizard.
        assert_eq!(
            w.feed_line(&step(9, "init", "started", 1)).unwrap(),
            Applied::Dropped
        );
    }

    #[test]
    fn a_failed_step_rolls_back() {
        let mut w = SetupWizard::new();
        w.feed_line(PLAN).unwrap();
        w.feed_line(&step(2, "init", "started", 1)).unwrap();
        w.feed_line(
            r#"{"seq":3,"event":"step","step":"init","status":"failed","index":1,"total":3,"description":"d","code":4,"detail":"disk full"}"#,
        )
        .unwrap();
        w.feed_line(
            r#"{"seq":4,"event":"finished","status":"failed","exit_code":4,"rolled_back":true,"resumable":false,"failed_step":"init"}"#,
        )
        .unwrap();
        let e = w.error().unwrap();
        assert_eq!(e.exit, ExitClass::StepFailedRolledBack);
        assert!(e.what.contains("init") && e.what.contains("disk full"));
        assert!(e.effect.starts_with("Rolled back"));
        assert_eq!(w.next(), Next::StartAgain);
        assert_eq!(w.steps()[0].state, StepState::Failed);
    }

    #[test]
    fn a_kept_partial_run_can_resume() {
        let mut w = SetupWizard::new();
        w.feed_line(PLAN).unwrap();
        w.feed_line(
            r#"{"seq":2,"event":"finished","status":"failed","exit_code":3,"rolled_back":false,"resumable":true,"failed_step":"nv-rs"}"#,
        )
        .unwrap();
        assert_eq!(w.next(), Next::ResumeOrRestart);
        let e = w.error().unwrap();
        assert_eq!(e.exit, ExitClass::VerificationFailed);
        assert_eq!(e.retry, crate::error::Retry::Yes);
        assert!(e.effect.contains("kept"));
    }

    #[test]
    fn a_refusal_is_one_finished_event() {
        let mut w = SetupWizard::new();
        w.feed_line(r#"{"seq":1,"event":"finished","status":"refused","exit_code":2,"rolled_back":false,"reason":"already exists"}"#)
            .unwrap();
        assert_eq!(w.next(), Next::ChangeInputs);
        let e = w.error().unwrap();
        assert_eq!(e.what, "already exists");
        assert_eq!(e.effect, "Nothing was changed.");
        assert!(e.exit.is_result_card());
        assert!(matches!(e.retry, crate::error::Retry::AfterFix(_)));
    }

    #[test]
    fn a_dry_run_lists_planned_steps_and_offers_the_real_run() {
        let mut w = SetupWizard::new();
        w.feed_line(&PLAN.replace("\"dry_run\":false", "\"dry_run\":true"))
            .unwrap();
        w.feed_line(&step(2, "init", "planned", 1)).unwrap();
        assert_eq!(*w.phase(), Phase::Planned);
        assert_eq!(w.steps()[0].state, StepState::Planned);
        w.feed_line(
            r#"{"seq":3,"event":"finished","status":"dry-run","exit_code":0,"rolled_back":false}"#,
        )
        .unwrap();
        assert!(w.is_dry_run());
        assert_eq!(w.next(), Next::RunForReal);
        assert!(w.error().is_none());
        assert_eq!(
            w.equivalent_command(),
            "nvmod setup --instance /i --dry-run"
        );
    }

    #[test]
    fn a_resume_skips_finished_steps() {
        let mut w = SetupWizard::new();
        w.feed_line(&PLAN.replace("\"resume\":false", "\"resume\":true"))
            .unwrap();
        w.feed_line(r#"{"seq":2,"event":"resumed","instance":"/i","total":3,"completed":["init"],"next":"nv-rs"}"#).unwrap();
        w.feed_line(r#"{"seq":3,"event":"skipped","step":"init","index":1,"total":3,"description":"d","reason":"finished earlier and still holds"}"#).unwrap();
        assert!(w.is_resume());
        assert_eq!(w.steps()[0].state, StepState::Skipped);
        assert_eq!(w.progress(), (1, 3));
        assert_eq!(w.equivalent_command(), "nvmod setup --resume --instance /i");
    }

    #[test]
    fn a_restart_is_a_finished_event_alone() {
        let mut w = SetupWizard::new();
        w.feed_line(r#"{"seq":1,"event":"finished","status":"restarted","exit_code":0,"rolled_back":false,"removed":"/i"}"#).unwrap();
        assert_eq!(w.next(), Next::StartAgain);
        assert!(w.error().is_none());
    }

    #[test]
    fn stale_unknown_and_malformed_lines() {
        let mut w = SetupWizard::new();
        w.feed_line(PLAN).unwrap();
        assert_eq!(w.feed_line(PLAN).unwrap(), Applied::Dropped);
        assert_eq!(
            w.feed_line(r#"{"seq":2,"event":"telemetry","x":1}"#)
                .unwrap(),
            Applied::Dropped
        );
        assert_eq!(
            w.feed_line(
                r#"{"seq":3,"event":"step","step":"init","status":"paused","index":1,"total":3}"#
            )
            .unwrap(),
            Applied::Dropped
        );
        assert!(matches!(
            w.feed_line("not json"),
            Err(StreamError::NotJson(_))
        ));
        assert_eq!(w.feed_line(r#"{"event":"plan"}"#), Err(StreamError::NoSeq));
        assert!(matches!(
            w.feed_line(r#"{"seq":9,"event":"step","step":1}"#),
            Err(StreamError::Shape(_))
        ));
        for e in [
            StreamError::NoSeq,
            StreamError::NotJson("x".into()),
            StreamError::Shape("y".into()),
        ] {
            assert!(!e.to_string().is_empty());
        }
        assert_eq!(w.progress(), (0, 3));
    }

    #[test]
    fn a_step_outside_the_plan_is_added_and_a_finish_without_status_is_unknown() {
        let mut w = SetupWizard::new();
        w.feed_line(&step(1, "extra", "started", 1)).unwrap();
        assert_eq!(w.steps()[0].name, "extra");
        w.feed_line(
            r#"{"seq":2,"event":"finished","status":"weird","exit_code":9,"resumable":false}"#,
        )
        .unwrap();
        assert_eq!(w.next(), Next::StartAgain);
        assert_eq!(w.error().unwrap().exit, ExitClass::Internal);
    }

    #[test]
    fn job_events_follow_the_stream_into_a_job_table() {
        let lines = [
            PLAN.to_owned(),
            step(2, "init", "started", 1),
            step(3, "init", "done", 1),
            step(4, "nv-rs", "started", 2),
            r#"{"seq":5,"event":"step","step":"nv-rs","status":"failed","index":2,"total":3,"description":"d","code":4,"detail":"boom"}"#.to_owned(),
            r#"{"seq":6,"event":"finished","status":"failed","exit_code":4,"rolled_back":true,"resumable":false,"failed_step":"nv-rs"}"#.to_owned(),
        ];
        let mut table = JobTable::default();
        let id = JobId(7);
        for l in &lines {
            let ev = to_job_event(id, &parse_line(l).unwrap()).unwrap();
            table.apply(ev);
            if l == &lines[1] {
                assert_eq!(table.writing(), Some(id));
            }
        }
        let j = table.get(id).unwrap();
        assert_eq!(j.state, JobState::Finished(Outcome::RolledBack));
        assert_eq!(j.exit, Some(ExitClass::StepFailedRolledBack));
        assert_eq!(j.phase, "nv-rs");
        assert_eq!(j.done, 1);
        assert!(j.log_tail.last().unwrap().starts_with("failed: boom"));
        assert!(table.writing().is_none());
    }

    #[test]
    fn job_event_mapping_covers_every_event() {
        let id = JobId(1);
        let map = |l: &str| to_job_event(id, &parse_line(l).unwrap());
        let started = map(&PLAN.replace("\"dry_run\":false", "\"dry_run\":true")).unwrap();
        assert!(matches!(
            started,
            JobEvent::Started {
                writes: false,
                cancellable: false,
                ..
            }
        ));
        assert!(matches!(
            map(PLAN).unwrap(),
            JobEvent::Started {
                writes: true,
                cancellable: true,
                ..
            }
        ));
        assert!(matches!(
            map(&step(2, "init", "planned", 1)),
            Some(JobEvent::Log { .. })
        ));
        assert!(
            map(r#"{"seq":2,"event":"step","step":"i","status":"zzz","index":1,"total":2}"#)
                .is_none()
        );
        assert!(map(r#"{"seq":2,"event":"nope"}"#).is_none());
        assert!(matches!(
            map(r#"{"seq":2,"event":"resumed","total":3,"completed":["a","b"],"next":"c"}"#),
            Some(JobEvent::Progress { done: 2, .. })
        ));
        assert!(matches!(
            map(r#"{"seq":3,"event":"skipped","step":"a","index":1,"total":3,"reason":"ok"}"#),
            Some(JobEvent::Progress { done: 1, .. })
        ));
        for (status, extra, want) in [
            ("ok", "", Outcome::Completed),
            ("restarted", "", Outcome::Completed),
            ("dry-run", "", Outcome::DryRun),
            ("refused", "", Outcome::Refused),
            ("failed", ",\"rolled_back\":true", Outcome::RolledBack),
            ("failed", ",\"resumable\":true", Outcome::KeptPartial),
            ("failed", "", Outcome::Failed),
        ] {
            let l = format!(
                r#"{{"seq":9,"event":"finished","status":"{status}","exit_code":0{extra}}}"#
            );
            let Some(JobEvent::Finished { outcome, .. }) = map(&l) else {
                panic!("no finish")
            };
            assert_eq!(outcome, want, "{status}{extra}");
        }
    }
}
