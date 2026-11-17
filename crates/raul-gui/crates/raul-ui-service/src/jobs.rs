// SPDX-License-Identifier: GPL-3.0-only
//! Running a command line as a job: its events go to a sink, or straight into the job table.
//!
//! A command that streams (`raul setup --json` prints one event per line) feeds the table while it runs. A
//! command that prints one report at the end gets a final record instead: `Started`, a log line, `Finished`. In
//! both cases the job is in the table from its start to its end, and a late or missing event cannot leave it
//! running.

use crate::exec::{CancelHandle, Exec, Output};
use crate::registry::Action;
use crate::service::{CliService, ServiceError, cause_of, repro};
use crate::wiring::outcome_of;
use raul_ui_model::error::{ExitClass, ViewError};
use raul_ui_model::job::{JobError, JobEvent, JobId, JobTable, Outcome};
use raul_ui_model::setup::{parse_line, to_job_event};

/// What to run as a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    /// The job's id, chosen by the caller.
    pub id: JobId,
    /// The command.
    pub action: Action,
    /// The words after the command, before `--json` (a typed request's `args()`).
    pub args: Vec<String>,
    /// Whether the job writes to the Instance. Only a writing job is refused while another writes.
    pub writes: bool,
    /// Whether the command only plans (`--dry-run`): a successful run ends as a dry run.
    pub dry_run: bool,
    /// Whether Cancel is offered ([`Action::cancellable`] for the actions that can be stopped).
    pub cancellable: bool,
}

impl JobSpec {
    /// A job of `action` with its arguments.
    pub fn new(id: JobId, action: Action, args: Vec<String>) -> Self {
        Self {
            id,
            action,
            args,
            writes: false,
            dry_run: false,
            cancellable: action.cancellable(),
        }
    }

    /// Marks the job as one that writes to the Instance.
    #[must_use]
    pub fn writing(mut self) -> Self {
        self.writes = true;
        self
    }

    /// Marks the job as a dry run. A dry run is not offered Cancel.
    #[must_use]
    pub fn dry(mut self) -> Self {
        self.dry_run = true;
        self.cancellable = false;
        self
    }
}

/// Keeps the events of one job well formed: `Started` first, and a `Finished` at the end whatever the command
/// printed.
struct Feed<'a> {
    spec: &'a JobSpec,
    sink: &'a mut dyn FnMut(JobEvent),
    started: bool,
    finished: bool,
    streamed: bool,
    last_seq: u64,
}

impl Feed<'_> {
    fn start(&mut self) {
        if !self.started {
            self.started = true;
            (self.sink)(JobEvent::Started {
                id: self.spec.id,
                op: self.spec.action.command().to_owned(),
                writes: self.spec.writes,
                cancellable: self.spec.cancellable,
            });
        }
    }

    fn emit(&mut self, event: JobEvent) {
        match &event {
            JobEvent::Started { .. } => {
                if self.started {
                    return;
                }
                self.started = true;
            }
            JobEvent::Progress { seq, .. } | JobEvent::Log { seq, .. } => {
                self.start();
                self.last_seq = self.last_seq.max(*seq);
            }
            JobEvent::Finished { seq, .. } => {
                self.start();
                self.last_seq = self.last_seq.max(*seq);
                self.finished = true;
            }
        }
        (self.sink)(event);
    }

    fn next_seq(&mut self) -> u64 {
        self.last_seq += 1;
        self.last_seq
    }
}

/// What the log says of a command that was stopped before it could report.
fn cancelled_line(spec: &JobSpec) -> String {
    if spec.writes {
        format!(
            "cancelled: raul {} was stopped before it could say what it left behind; run the Doctor{}",
            spec.action.command(),
            if spec.action == Action::Setup {
                " (the Instance can be continued with a resume or removed with a restart)"
            } else {
                ""
            }
        )
    } else {
        format!("cancelled: raul {} was stopped", spec.action.command())
    }
}

/// Asks a job to stop: the table marks it as cancelling (so it keeps holding the Instance until it ends) and
/// `handle`, the one the job was started with, tells the runner.
///
/// # Errors
/// [`JobError::Unknown`] or [`JobError::NotCancellable`] (not offered, not running, or already cancelling); the
/// handle is not touched then.
pub fn cancel_job(table: &mut JobTable, id: JobId, handle: &CancelHandle) -> Result<(), JobError> {
    table.request_cancel(id)?;
    handle.request();
    Ok(())
}

impl<E: Exec> CliService<E> {
    /// Runs `spec` as a job, handing each [`JobEvent`] to `sink` as it happens. The command line is run with
    /// `--json`.
    ///
    /// Lines of standard output that are events (the setup stream) become job events as they arrive. When the
    /// command prints no events the job gets a final record; when it printed events but no `Finished`, one is
    /// added from the exit code. A command that fails is a job that finished: the returned [`Output`] has its
    /// exit code.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the process could not be started; no event is sent then.
    pub fn run_job(
        &self,
        spec: &JobSpec,
        sink: &mut dyn FnMut(JobEvent),
    ) -> Result<Output, ServiceError> {
        self.run_job_cancellable(spec, sink, &CancelHandle::new())
    }

    /// [`CliService::run_job`] that can be stopped: when `cancel` is requested the command is asked to end and
    /// forced to after a grace period (see [`Exec::run_cancellable`]), and the job ends as follows.
    ///
    /// * The command reported its own end (a `finished` event, or an exit code of its own): that is the outcome,
    ///   so a command that rolled itself back ends as [`Outcome::RolledBack`] and one that kept its partial
    ///   result as [`Outcome::KeptPartial`].
    /// * The command was cut off by the signal before it could report: a job that writes ends as
    ///   [`Outcome::Failed`] (the state is not known, the log says to run the Doctor), one that does not write as
    ///   [`Outcome::Cancelled`].
    /// * Cancel was requested before the command started: it is not started, and the job is
    ///   [`Outcome::Cancelled`].
    ///
    /// # Errors
    /// [`ServiceError::Failed`] when the process could not be started; no event is sent then.
    pub fn run_job_cancellable(
        &self,
        spec: &JobSpec,
        sink: &mut dyn FnMut(JobEvent),
        cancel: &CancelHandle,
    ) -> Result<Output, ServiceError> {
        let args = Self::words(spec.action, spec.args.clone(), true);
        let mut feed = Feed {
            spec,
            sink,
            started: false,
            finished: false,
            streamed: false,
            last_seq: 0,
        };
        if cancel.is_requested() {
            feed.start();
            let line = String::from("cancelled before it started: nothing was run");
            feed.emit(JobEvent::Log {
                id: spec.id,
                seq: 1,
                line,
            });
            feed.emit(JobEvent::Finished {
                id: spec.id,
                seq: 2,
                outcome: Outcome::Cancelled,
                exit: ExitClass::Ok,
            });
            return Ok(Output {
                code: None,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        let out = self
            .exec()
            .run_cancellable(
                &args,
                &mut |line| {
                    if let Ok(parsed) = parse_line(line)
                        && let Some(event) = to_job_event(spec.id, &parsed)
                    {
                        feed.streamed = true;
                        feed.emit(event);
                    }
                },
                cancel,
            )
            .map_err(|why| {
                let mut e = ViewError::new(ExitClass::Internal, why, "Nothing was started");
                e.repro = Some(repro(&args));
                ServiceError::Failed(e)
            })?;
        if !feed.finished {
            let cut_off = out.code.is_none() && cancel.is_requested();
            let class = out.code.map_or(ExitClass::Internal, ExitClass::from_code);
            let outcome = if class == ExitClass::Ok && spec.dry_run {
                Outcome::DryRun
            } else if cut_off && !spec.writes {
                Outcome::Cancelled
            } else {
                outcome_of(class)
            };
            let class = if outcome == Outcome::Cancelled {
                ExitClass::Ok
            } else {
                class
            };
            feed.start();
            if !feed.streamed {
                let line = if cut_off {
                    cancelled_line(spec)
                } else if class == ExitClass::Ok {
                    format!("raul {} finished", spec.action.command())
                } else {
                    cause_of(&out.stderr, spec.action.command())
                };
                let seq = feed.next_seq();
                feed.emit(JobEvent::Log {
                    id: spec.id,
                    seq,
                    line,
                });
            }
            if cut_off && feed.streamed {
                let seq = feed.next_seq();
                feed.emit(JobEvent::Log {
                    id: spec.id,
                    seq,
                    line: cancelled_line(spec),
                });
            }
            let seq = feed.next_seq();
            feed.emit(JobEvent::Finished {
                id: spec.id,
                seq,
                outcome,
                exit: class,
            });
        }
        Ok(out)
    }

    /// [`CliService::run_job`] into the job table. A writing job is refused while another job writes.
    ///
    /// # Errors
    /// [`ServiceError::Failed`] with a refusal when another job holds the Instance, or when the process could
    /// not be started.
    pub fn run_job_in(&self, spec: &JobSpec, table: &mut JobTable) -> Result<Output, ServiceError> {
        self.run_job_in_cancellable(spec, table, &CancelHandle::new())
    }

    /// [`CliService::run_job_in`] that [`cancel_job`] can stop, given the same `cancel` handle.
    ///
    /// # Errors
    /// As [`CliService::run_job_in`].
    pub fn run_job_in_cancellable(
        &self,
        spec: &JobSpec,
        table: &mut JobTable,
        cancel: &CancelHandle,
    ) -> Result<Output, ServiceError> {
        if spec.writes
            && let Err(busy) = table.check_can_start_writer()
        {
            let what = match busy {
                raul_ui_model::job::JobError::Busy(id) => {
                    format!("another job (number {}) is writing to the Instance", id.0)
                }
                other => format!("{other:?}"),
            };
            return Err(ServiceError::Failed(ViewError::new(
                ExitClass::Refused,
                what,
                "Nothing was started",
            )));
        }
        self.run_job_cancellable(
            spec,
            &mut |event| {
                table.apply(event);
            },
            cancel,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::ScriptedExec;
    use raul_ui_model::job::JobState;

    const PLAN: &str = r#"{"seq":1,"event":"plan","dry_run":false,"resume":false,"instance":"/i","steps":[{"step":"init","description":"Make the Instance"}]}"#;
    const STEP_STARTED: &str = r#"{"seq":2,"event":"step","step":"init","status":"started","index":1,"total":1,"description":"Make the Instance"}"#;
    const STEP_DONE: &str = r#"{"seq":3,"event":"step","step":"init","status":"done","index":1,"total":1,"detail":"made"}"#;
    const FINISHED: &str = r#"{"seq":4,"event":"finished","status":"ok","exit_code":0,"rolled_back":false,"resumable":false}"#;

    fn setup_spec() -> JobSpec {
        JobSpec::new(
            JobId(1),
            Action::Setup,
            vec!["--instance".into(), "/i".into()],
        )
        .writing()
    }

    fn collect(svc: &CliService<ScriptedExec>, spec: &JobSpec) -> (Vec<JobEvent>, Output) {
        let mut events = Vec::new();
        let out = svc.run_job(spec, &mut |e| events.push(e)).unwrap();
        (events, out)
    }

    #[test]
    fn a_streaming_command_feeds_the_table_line_by_line() {
        let exec = ScriptedExec::new();
        exec.on_run(&[PLAN, STEP_STARTED, STEP_DONE, FINISHED], 0, "");
        let svc = CliService::new(exec);
        let mut table = JobTable::default();
        svc.run_job_in(&setup_spec(), &mut table).unwrap();
        let job = table.get(JobId(1)).unwrap();
        assert_eq!(job.state, JobState::Finished(Outcome::Completed));
        assert_eq!(job.progress_text(), "1 of 1");
        assert!(job.writes && job.log_tail.contains(&"made".to_owned()));
        let args = &svc.exec().calls()[0];
        assert_eq!(args.first().map(String::as_str), Some("setup"));
        assert_eq!(args.last().map(String::as_str), Some("--json"));
    }

    #[test]
    fn a_stream_cut_short_is_finished_from_the_exit_code() {
        let exec = ScriptedExec::new();
        exec.on_run(&[PLAN, STEP_STARTED], 4, "raul setup: it broke");
        let svc = CliService::new(exec);
        let (events, out) = collect(&svc, &setup_spec());
        assert_eq!(out.code, Some(4));
        let Some(JobEvent::Finished {
            seq, outcome, exit, ..
        }) = events.last()
        else {
            panic!("no Finished");
        };
        assert_eq!(
            (*seq, *outcome, *exit),
            (3, Outcome::RolledBack, ExitClass::StepFailedRolledBack)
        );
        let mut table = JobTable::default();
        for e in events {
            table.apply(e);
        }
        assert!(table.writing().is_none());
    }

    #[test]
    fn a_refusal_that_prints_only_finished_still_gets_a_started() {
        let exec = ScriptedExec::new();
        exec.on_run(
            &[r#"{"seq":1,"event":"finished","status":"refused","exit_code":2,"rolled_back":false,"reason":"no"}"#],
            2,
            "",
        );
        let svc = CliService::new(exec);
        let mut table = JobTable::default();
        svc.run_job_in(&setup_spec(), &mut table).unwrap();
        assert_eq!(
            table.get(JobId(1)).unwrap().state,
            JobState::Finished(Outcome::Refused)
        );
    }

    #[test]
    fn a_command_that_prints_one_report_gets_a_final_record() {
        let exec = ScriptedExec::new();
        exec.on_run(&["{", "  \"profile\": \"Default\"", "}"], 0, "");
        exec.on_run(&[], 4, "raul sort: the write failed\nmore");
        exec.on_run(&["{}"], 0, "");
        let svc = CliService::new(exec);
        let spec = JobSpec::new(JobId(7), Action::Sort, vec!["--apply".into()]).writing();
        let mut table = JobTable::default();
        svc.run_job_in(&spec, &mut table).unwrap();
        let j = table.get(JobId(7)).unwrap();
        assert_eq!(j.state, JobState::Finished(Outcome::Completed));
        assert_eq!(j.log_tail, ["raul sort finished"]);

        let spec = JobSpec::new(JobId(8), Action::Sort, vec![]).writing();
        svc.run_job_in(&spec, &mut table).unwrap();
        let j = table.get(JobId(8)).unwrap();
        assert_eq!(j.state, JobState::Finished(Outcome::RolledBack));
        assert_eq!(j.log_tail, ["the write failed"]);
        assert_eq!(j.exit, Some(ExitClass::StepFailedRolledBack));

        let spec = JobSpec::new(JobId(9), Action::InstallMod, vec![]).dry();
        svc.run_job_in(&spec, &mut table).unwrap();
        assert_eq!(
            table.get(JobId(9)).unwrap().state,
            JobState::Finished(Outcome::DryRun)
        );
    }

    /// Starts the run, asks to stop once the plan arrived (through the job table, as a screen does).
    fn cancel_after_the_plan(svc: &CliService<ScriptedExec>, spec: &JobSpec) -> (JobTable, Output) {
        let cancel = CancelHandle::new();
        let mut table = JobTable::default();
        let mut asked = false;
        let out = svc
            .run_job_cancellable(
                spec,
                &mut |e| {
                    table.apply(e);
                    if !asked {
                        asked = true;
                        cancel_job(&mut table, spec.id, &cancel).unwrap();
                        assert_eq!(table.get(spec.id).unwrap().state, JobState::Cancelling);
                    }
                },
                &cancel,
            )
            .unwrap();
        (table, out)
    }

    #[test]
    fn a_stopped_setup_ends_as_the_command_reported_it() {
        let rolled_back = r#"{"seq":3,"event":"finished","status":"failed","exit_code":4,"rolled_back":true,"resumable":false}"#;
        let kept = r#"{"seq":3,"event":"finished","status":"failed","exit_code":4,"rolled_back":false,"resumable":true}"#;
        for (stop, code, want) in [
            (vec![rolled_back], Some(4), Outcome::RolledBack),
            (vec![kept], Some(4), Outcome::KeptPartial),
            (vec![], None, Outcome::Failed),
        ] {
            let exec = ScriptedExec::new();
            exec.on_run(&[PLAN, STEP_STARTED, STEP_DONE, FINISHED], 0, "")
                .on_cancel(&stop, code, "");
            let svc = CliService::new(exec);
            let (table, out) = cancel_after_the_plan(&svc, &setup_spec());
            assert_eq!(out.code, code);
            let job = table.get(JobId(1)).unwrap();
            assert!(job.cancellable, "the plan of a real run offers Cancel");
            assert_eq!(job.state, JobState::Finished(want), "{stop:?}");
            if want == Outcome::Failed {
                assert!(
                    job.log_tail.iter().any(|l| l.contains("resume")),
                    "{:?}",
                    job.log_tail
                );
            }
            assert!(table.writing().is_none(), "the Instance is free again");
        }
    }

    #[test]
    fn a_stopped_job_that_writes_nothing_is_cancelled_and_one_that_cannot_be_stopped_is_refused() {
        let exec = ScriptedExec::new();
        exec.on_run(&[PLAN, STEP_STARTED, FINISHED], 0, "")
            .on_cancel(&[], None, "");
        let svc = CliService::new(exec);
        let spec = JobSpec::new(JobId(4), Action::InstallMod, vec![]);
        assert!(spec.cancellable && !spec.writes);
        let (table, _) = cancel_after_the_plan(&svc, &spec);
        let job = table.get(JobId(4)).unwrap();
        assert_eq!(job.state, JobState::Finished(Outcome::Cancelled));
        assert_eq!(job.exit, Some(ExitClass::Ok));

        // Sort cannot be stopped, a dry run is not offered Cancel, and a finished job is over.
        let sort = JobSpec::new(JobId(5), Action::Sort, vec![]);
        assert!(!sort.cancellable);
        assert!(
            !JobSpec::new(JobId(6), Action::Ttw, vec![])
                .dry()
                .cancellable
        );
        let mut table = JobTable::default();
        svc.exec().on_run(&["{}"], 0, "");
        svc.run_job_in(&sort, &mut table).unwrap();
        let handle = CancelHandle::new();
        assert!(cancel_job(&mut table, JobId(5), &handle).is_err());
        assert!(cancel_job(&mut table, JobId(99), &handle).is_err());
        assert!(
            !handle.is_requested(),
            "a refused cancel does not touch the runner"
        );
    }

    #[test]
    fn a_second_writer_is_refused_before_anything_runs() {
        let exec = ScriptedExec::new();
        let svc = CliService::new(exec);
        let mut table = JobTable::default();
        table.apply(JobEvent::Started {
            id: JobId(1),
            op: "setup".into(),
            writes: true,
            cancellable: false,
        });
        let spec = JobSpec::new(JobId(2), Action::Rollback, vec![]).writing();
        let ServiceError::Failed(e) = svc.run_job_in(&spec, &mut table).unwrap_err() else {
            panic!("expected a refusal");
        };
        assert_eq!(e.exit, ExitClass::Refused);
        assert!(e.what.contains("number 1"));
        assert_eq!(svc.exec().calls().len(), 0);
        // A read is allowed alongside.
        let read = JobSpec::new(JobId(3), Action::Doctor, vec![]);
        svc.exec().on_run(&["{}"], 0, "");
        svc.run_job_in(&read, &mut table).unwrap();
        assert!(table.get(JobId(3)).is_some());
    }

    #[test]
    fn a_process_that_cannot_start_sends_no_events() {
        let exec = ScriptedExec::new();
        exec.on_start_failure("could not start raul");
        let svc = CliService::new(exec);
        let mut sent = 0;
        let err = svc.run_job(&setup_spec(), &mut |_| sent += 1).unwrap_err();
        assert_eq!(sent, 0);
        assert_eq!(err.view_error().effect, "Nothing was started");
    }
}
