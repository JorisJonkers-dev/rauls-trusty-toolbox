// SPDX-License-Identifier: GPL-3.0-only
//! The `raul-events` v1 stream (`contracts/raul-events.v1.json`): what a driver command such as `raul ttw --json`
//! prints, one JSON object per line, and how each line becomes an event of the job table.
//!
//! Every line carries `schema_version`, `contract`, `seq` (from 1), `command` and `event`. A command that is refused
//! before it starts prints only `result` and `finished`; `finished` is always the last line and its `exit_code` is
//! the process's.

use crate::error::ExitClass;
use crate::job::{JobEvent, JobId, Outcome};
use serde::Deserialize;

/// The contract name.
pub const CONTRACT: &str = "raul-events";
/// The version this build reads.
pub const VERSION: u32 = 1;

/// How a driver command ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Done (exit code 0). For a dry run: everything checked, nothing written.
    Success,
    /// Refused before anything was written (exit code 2).
    Refused,
    /// The Component refused the input as one it does not support (exit code 2); nothing was written.
    Unsupported,
    /// The program could not be verified against its checksum (exit code 3); it was not started.
    Unverifiable,
    /// A step failed (exit code 4); the Instance is as it was.
    Failed,
    /// Cancelled (exit code 4); the Instance is as it was, and `--resume` continues.
    Cancelled,
}

impl Status {
    /// The job outcome it is; `dry_run` turns a success into [`Outcome::DryRun`].
    pub fn outcome(self, dry_run: bool) -> Outcome {
        match self {
            Status::Success if dry_run => Outcome::DryRun,
            Status::Success => Outcome::Completed,
            Status::Refused | Status::Unsupported | Status::Unverifiable => Outcome::Refused,
            // The driver leaves the Instance as it was on both, so nothing is left to clean up.
            Status::Failed => Outcome::RolledBack,
            Status::Cancelled => Outcome::Cancelled,
        }
    }
}

/// The Component program a driver runs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ToolInfo {
    /// Its name, such as `ttwright`.
    pub name: String,
    /// The file that ran.
    pub path: String,
    /// Its SHA-256.
    pub sha256: String,
    /// Its version as it reported it; `None` before it ran.
    pub version: Option<String>,
}

/// One event of the stream.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// The checks passed and the Component program is about to start.
    Started {
        /// Nothing is written.
        dry_run: bool,
        /// The run continues an interrupted one.
        resume: bool,
        /// The Instance.
        instance: String,
        /// The Profile that gets the result.
        profile: String,
        /// The program that runs.
        tool: ToolInfo,
    },
    /// A phase of the work began.
    Phase {
        /// The phase.
        phase: String,
    },
    /// Progress within a phase.
    Progress {
        /// The phase.
        phase: String,
        /// Items done.
        done: u64,
        /// Items in all; 0 when not known yet.
        of: u64,
        /// What is being worked on.
        message: Option<String>,
    },
    /// Something did not go as planned, but the run goes on.
    Warning {
        /// For people.
        message: String,
    },
    /// The command's result document, such as a `raul-ttw-result`.
    Result {
        /// The document.
        result: serde_json::Value,
    },
    /// The run ended; always the last line.
    Finished {
        /// How it ended.
        status: Status,
        /// The exit code of `raul`.
        exit_code: u8,
        /// The stable error code of the result, unless the status is `success`.
        code: Option<String>,
        /// For people.
        message: Option<String>,
        /// `--resume` continues the run.
        resumable: bool,
        /// Whether the Instance was changed.
        changed: bool,
    },
}

/// One line of the stream.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Line {
    /// The version of the stream.
    pub schema_version: u32,
    /// Always [`CONTRACT`].
    pub contract: String,
    /// The order of the line, from 1.
    pub seq: u64,
    /// The command, such as `ttw`.
    pub command: String,
    /// The event.
    #[serde(flatten)]
    pub event: Event,
}

/// Why a line is not one this build reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineError {
    /// Not JSON, or not a `raul-events` line.
    NotAnEvent,
    /// A `raul-events` line of a newer version than this build reads.
    Newer(u32),
    /// A `raul-events` line whose event this build does not know or cannot read.
    Unreadable(String),
}

/// Parses one line of the stream.
///
/// # Errors
/// [`LineError::NotAnEvent`] for anything that is not a `raul-events` line (setup's own stream, a log line),
/// [`LineError::Newer`] for a newer version, and [`LineError::Unreadable`] for a line of this version that does
/// not match it.
pub fn parse_line(text: &str) -> Result<Line, LineError> {
    let value: serde_json::Value =
        serde_json::from_str(text.trim()).map_err(|_| LineError::NotAnEvent)?;
    if value.get("contract").and_then(serde_json::Value::as_str) != Some(CONTRACT) {
        return Err(LineError::NotAnEvent);
    }
    let version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| LineError::Unreadable("no schema_version".into()))?;
    if version > VERSION {
        return Err(LineError::Newer(version));
    }
    serde_json::from_value(value).map_err(|e| LineError::Unreadable(e.to_string()))
}

/// The job event a line becomes, if any (a `result` is read by the caller, not shown).
pub fn to_job_event(id: JobId, line: &Line, dry_run: bool) -> Option<JobEvent> {
    let seq = line.seq;
    match &line.event {
        Event::Started { dry_run, .. } => Some(JobEvent::Started {
            id,
            op: line.command.clone(),
            writes: !dry_run,
            cancellable: !dry_run,
        }),
        Event::Phase { phase } => Some(JobEvent::Progress {
            id,
            seq,
            phase: phase.clone(),
            done: 0,
            total: None,
            message: None,
        }),
        Event::Progress {
            phase,
            done,
            of,
            message,
        } => Some(JobEvent::Progress {
            id,
            seq,
            phase: phase.clone(),
            done: *done,
            total: (*of > 0).then_some(*of),
            message: message.clone(),
        }),
        Event::Warning { message } => Some(JobEvent::Log {
            id,
            seq,
            line: format!("warning: {message}"),
        }),
        Event::Result { .. } => None,
        Event::Finished {
            status, exit_code, ..
        } => Some(JobEvent::Finished {
            id,
            seq,
            outcome: status.outcome(dry_run),
            exit: ExitClass::from_code(i64::from(*exit_code)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"{"schema_version":1,"contract":"raul-events","seq":6,"command":"ttw","event":"progress","phase":"assets","done":1200,"of":2400,"message":"copying"}"#;

    #[test]
    fn a_progress_line_becomes_job_progress() {
        let line = parse_line(LINE).unwrap();
        assert_eq!((line.seq, line.command.as_str()), (6, "ttw"));
        assert_eq!(
            to_job_event(JobId(3), &line, false),
            Some(JobEvent::Progress {
                id: JobId(3),
                seq: 6,
                phase: "assets".into(),
                done: 1200,
                total: Some(2400),
                message: Some("copying".into()),
            })
        );
    }

    #[test]
    fn other_lines_are_not_events_and_newer_ones_are_refused() {
        assert_eq!(parse_line("plain text"), Err(LineError::NotAnEvent));
        assert_eq!(
            parse_line(r#"{"contract":"setup","seq":1}"#),
            Err(LineError::NotAnEvent)
        );
        assert_eq!(
            parse_line(&LINE.replace("\"schema_version\":1", "\"schema_version\":2")),
            Err(LineError::Newer(2))
        );
        assert!(matches!(
            parse_line(&LINE.replace("\"progress\"", "\"teleport\"")),
            Err(LineError::Unreadable(_))
        ));
        assert!(matches!(
            parse_line(r#"{"contract":"raul-events"}"#),
            Err(LineError::Unreadable(_))
        ));
    }

    #[test]
    fn statuses_map_onto_outcomes() {
        assert_eq!(Status::Success.outcome(false), Outcome::Completed);
        assert_eq!(Status::Success.outcome(true), Outcome::DryRun);
        for s in [Status::Refused, Status::Unsupported, Status::Unverifiable] {
            assert_eq!(s.outcome(false), Outcome::Refused);
        }
        assert_eq!(Status::Failed.outcome(false), Outcome::RolledBack);
        assert_eq!(Status::Cancelled.outcome(false), Outcome::Cancelled);
    }

    /// The published fixtures of the Contract: every line parses, the last one is `finished`, and the outcome
    /// follows its status.
    #[test]
    fn every_published_fixture_reads() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../contracts/fixtures/raul-events");
        let mut read = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            let lines: Vec<Line> = text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| parse_line(l).unwrap_or_else(|e| panic!("{}: {e:?}: {l}", path.display())))
                .collect();
            let seqs: Vec<u64> = lines.iter().map(|l| l.seq).collect();
            assert!(seqs.windows(2).all(|w| w[0] < w[1]), "{}", path.display());
            let Some(Event::Finished { status, .. }) = lines.last().map(|l| &l.event) else {
                panic!("{} does not end with finished", path.display());
            };
            let event = to_job_event(JobId(1), lines.last().unwrap(), false).unwrap();
            assert!(
                matches!(event, JobEvent::Finished { outcome, .. } if outcome == status.outcome(false)),
                "{}",
                path.display()
            );
            read += 1;
        }
        assert!(read >= 6, "{read}");
    }
}
