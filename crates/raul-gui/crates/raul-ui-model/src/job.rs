// SPDX-License-Identifier: GPL-3.0-only
//! The job events of document 04, section 4, and the table that keeps them in order.

use crate::error::ExitClass;
use serde::{Deserialize, Serialize};

/// Identifies a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JobId(pub u64);

/// How a job ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// It did what was asked.
    Completed,
    /// A dry run: nothing was written.
    DryRun,
    /// A failed step was undone; nothing is left behind.
    RolledBack,
    /// A step failed and the partial result was kept so the run can continue.
    KeptPartial,
    /// It was refused before anything changed.
    Refused,
    /// It failed and the state is not known to be clean (needs a decision).
    Failed,
}

/// What a job reports. Events of one job carry a `seq` that grows; a stale or repeated one is dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobEvent {
    /// The job exists.
    Started {
        /// The job.
        id: JobId,
        /// The operation, as its command words (`setup`).
        op: String,
        /// Whether the job writes to the Instance (a dry run and a read do not).
        writes: bool,
        /// Whether Cancel is offered.
        cancellable: bool,
    },
    /// Progress in a phase.
    Progress {
        /// The job.
        id: JobId,
        /// Order of the event.
        seq: u64,
        /// The phase in words.
        phase: String,
        /// Units done.
        done: u64,
        /// Units in all; `None` is "no estimate yet".
        total: Option<u64>,
        /// A line to show.
        message: Option<String>,
    },
    /// A log line (never game data).
    Log {
        /// The job.
        id: JobId,
        /// Order of the event.
        seq: u64,
        /// The line.
        line: String,
    },
    /// The job is over.
    Finished {
        /// The job.
        id: JobId,
        /// Order of the event.
        seq: u64,
        /// How it ended.
        outcome: Outcome,
        /// The exit class.
        exit: ExitClass,
    },
}

/// The state of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    /// Going.
    Running,
    /// Cancel was requested; waiting for the job to finish.
    Cancelling,
    /// Over.
    Finished(Outcome),
}

/// What a screen shows of one job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobView {
    /// The job.
    pub id: JobId,
    /// The operation.
    pub op: String,
    /// Whether the job writes to the Instance.
    pub writes: bool,
    /// Whether Cancel is offered.
    pub cancellable: bool,
    /// The state.
    pub state: JobState,
    /// The phase.
    pub phase: String,
    /// Units done.
    pub done: u64,
    /// Units in all.
    pub total: Option<u64>,
    /// The last lines.
    pub log_tail: Vec<String>,
    /// The exit class once finished.
    pub exit: Option<ExitClass>,
    last_seq: u64,
}

impl JobView {
    /// "12 of 63", or "no estimate yet": the text form of the progress.
    pub fn progress_text(&self) -> String {
        match self.total {
            Some(t) => format!("{} of {t}", self.done),
            None => "no estimate yet".to_owned(),
        }
    }
}

const LOG_TAIL: usize = 50;

/// The refusals of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// Another job that writes holds the Instance.
    Busy(JobId),
    /// No job with this id.
    Unknown(JobId),
    /// The job is already over or already cancelling.
    NotCancellable(JobId),
}

/// What happened to an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// It changed the table.
    Yes,
    /// It was stale, repeated or for a job that is over, and was dropped.
    Dropped,
}

/// The job table: one writing job at a time, ordered events.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobTable {
    jobs: Vec<JobView>,
}

impl JobTable {
    /// The jobs, oldest first.
    pub fn jobs(&self) -> &[JobView] {
        &self.jobs
    }

    /// One job.
    pub fn get(&self, id: JobId) -> Option<&JobView> {
        self.jobs.iter().find(|j| j.id == id)
    }

    /// The job that is running and writes, if any.
    pub fn writing(&self) -> Option<JobId> {
        self.jobs
            .iter()
            .find(|j| matches!(j.state, JobState::Running | JobState::Cancelling) && j.writes)
            .map(|j| j.id)
    }

    /// Refuses a second writing job (document 04: `ViewError::Busy`).
    ///
    /// # Errors
    /// [`JobError::Busy`] with the job that holds the Instance.
    pub fn check_can_start_writer(&self) -> Result<(), JobError> {
        self.writing().map_or(Ok(()), |id| Err(JobError::Busy(id)))
    }

    /// Asks a job to stop. The state is `Cancelling` until its `Finished` arrives.
    ///
    /// # Errors
    /// [`JobError::Unknown`] or [`JobError::NotCancellable`].
    pub fn request_cancel(&mut self, id: JobId) -> Result<(), JobError> {
        let job = self
            .jobs
            .iter_mut()
            .find(|j| j.id == id)
            .ok_or(JobError::Unknown(id))?;
        if job.state != JobState::Running || !job.cancellable {
            return Err(JobError::NotCancellable(id));
        }
        job.state = JobState::Cancelling;
        Ok(())
    }

    /// Applies one event. Events for a finished job, repeats and events that arrive out of order are dropped, so
    /// a late event after cancel cannot revive a job.
    pub fn apply(&mut self, event: JobEvent) -> Applied {
        match event {
            JobEvent::Started {
                id,
                op,
                writes,
                cancellable,
            } => {
                if self.get(id).is_some() {
                    return Applied::Dropped;
                }
                self.jobs.push(JobView {
                    id,
                    op,
                    writes,
                    cancellable,
                    state: JobState::Running,
                    phase: String::new(),
                    done: 0,
                    total: None,
                    log_tail: Vec::new(),
                    exit: None,
                    last_seq: 0,
                });
                Applied::Yes
            }
            JobEvent::Progress {
                id,
                seq,
                phase,
                done,
                total,
                message,
            } => self.with_live(id, seq, |j| {
                j.phase = phase;
                j.done = done;
                j.total = total;
                if let Some(m) = message {
                    push_log(j, m);
                }
            }),
            JobEvent::Log { id, seq, line } => self.with_live(id, seq, |j| push_log(j, line)),
            JobEvent::Finished {
                id,
                seq,
                outcome,
                exit,
            } => self.with_live(id, seq, |j| {
                j.state = JobState::Finished(outcome);
                j.exit = Some(exit);
            }),
        }
    }

    fn with_live(&mut self, id: JobId, seq: u64, f: impl FnOnce(&mut JobView)) -> Applied {
        match self.jobs.iter_mut().find(|j| j.id == id) {
            Some(j) if !matches!(j.state, JobState::Finished(_)) && seq > j.last_seq => {
                j.last_seq = seq;
                f(j);
                Applied::Yes
            }
            _ => Applied::Dropped,
        }
    }
}

/// The events of a command that streams nothing: `Started`, one `Log` per line and `Finished`, in order. This is
/// the fallback for a long command that prints one report at the end, so its job still shows in the table.
pub fn final_record(
    id: JobId,
    op: &str,
    writes: bool,
    outcome: Outcome,
    exit: ExitClass,
    lines: &[String],
) -> Vec<JobEvent> {
    let mut events = vec![JobEvent::Started {
        id,
        op: op.to_owned(),
        writes,
        cancellable: false,
    }];
    let mut seq = 0;
    for line in lines {
        seq += 1;
        events.push(JobEvent::Log {
            id,
            seq,
            line: line.clone(),
        });
    }
    events.push(JobEvent::Finished {
        id,
        seq: seq + 1,
        outcome,
        exit,
    });
    events
}

fn push_log(job: &mut JobView, line: String) {
    job.log_tail.push(line);
    if job.log_tail.len() > LOG_TAIL {
        job.log_tail.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started(table: &mut JobTable, id: u64) {
        let applied = table.apply(JobEvent::Started {
            id: JobId(id),
            op: "setup".into(),
            writes: true,
            cancellable: true,
        });
        assert_eq!(applied, Applied::Yes);
    }

    fn progress(id: u64, seq: u64, done: u64) -> JobEvent {
        JobEvent::Progress {
            id: JobId(id),
            seq,
            phase: "init".into(),
            done,
            total: Some(5),
            message: Some(format!("step {done}")),
        }
    }

    #[test]
    fn events_are_ordered_and_deduplicated() {
        let mut t = JobTable::default();
        started(&mut t, 1);
        assert_eq!(
            t.apply(JobEvent::Started {
                id: JobId(1),
                op: "x".into(),
                writes: false,
                cancellable: false
            }),
            Applied::Dropped
        );
        assert_eq!(t.apply(progress(1, 2, 2)), Applied::Yes);
        assert_eq!(t.apply(progress(1, 1, 1)), Applied::Dropped);
        assert_eq!(t.apply(progress(1, 2, 2)), Applied::Dropped);
        let j = t.get(JobId(1)).unwrap();
        assert_eq!(j.done, 2);
        assert_eq!(j.progress_text(), "2 of 5");
    }

    #[test]
    fn late_event_after_finish_is_ignored() {
        let mut t = JobTable::default();
        started(&mut t, 1);
        t.apply(JobEvent::Finished {
            id: JobId(1),
            seq: 3,
            outcome: Outcome::RolledBack,
            exit: ExitClass::StepFailedRolledBack,
        });
        assert_eq!(t.apply(progress(1, 4, 4)), Applied::Dropped);
        assert_eq!(
            t.apply(JobEvent::Log {
                id: JobId(1),
                seq: 9,
                line: "x".into()
            }),
            Applied::Dropped
        );
        let j = t.get(JobId(1)).unwrap();
        assert_eq!(j.state, JobState::Finished(Outcome::RolledBack));
        assert_eq!(j.exit, Some(ExitClass::StepFailedRolledBack));
        assert!(t.writing().is_none());
    }

    #[test]
    fn one_writer_at_a_time_and_cancel_is_a_request() {
        let mut t = JobTable::default();
        assert!(t.check_can_start_writer().is_ok());
        started(&mut t, 1);
        assert_eq!(t.check_can_start_writer(), Err(JobError::Busy(JobId(1))));
        assert_eq!(t.request_cancel(JobId(2)), Err(JobError::Unknown(JobId(2))));
        t.request_cancel(JobId(1)).unwrap();
        assert_eq!(t.get(JobId(1)).unwrap().state, JobState::Cancelling);
        assert_eq!(
            t.request_cancel(JobId(1)),
            Err(JobError::NotCancellable(JobId(1)))
        );
        assert_eq!(t.writing(), Some(JobId(1)));
        t.apply(JobEvent::Finished {
            id: JobId(1),
            seq: 1,
            outcome: Outcome::Completed,
            exit: ExitClass::Ok,
        });
        assert!(t.check_can_start_writer().is_ok());
    }

    #[test]
    fn a_final_record_runs_a_job_from_start_to_finish() {
        let mut t = JobTable::default();
        for e in final_record(
            JobId(3),
            "sort",
            true,
            Outcome::RolledBack,
            ExitClass::StepFailedRolledBack,
            &["a".to_owned(), "b".to_owned()],
        ) {
            assert_eq!(t.apply(e), Applied::Yes);
        }
        let j = t.get(JobId(3)).unwrap();
        assert_eq!(j.log_tail, ["a", "b"]);
        assert_eq!(j.state, JobState::Finished(Outcome::RolledBack));
        assert!(t.writing().is_none());
    }

    #[test]
    fn log_tail_is_bounded_and_progress_without_total_has_text() {
        let mut t = JobTable::default();
        started(&mut t, 1);
        for i in 1..=60 {
            t.apply(JobEvent::Log {
                id: JobId(1),
                seq: i,
                line: format!("l{i}"),
            });
        }
        let j = t.get(JobId(1)).unwrap();
        assert_eq!(j.log_tail.len(), 50);
        assert_eq!(j.log_tail[0], "l11");
        assert_eq!(j.progress_text(), "no estimate yet");
        assert_eq!(t.jobs().len(), 1);
    }
}
