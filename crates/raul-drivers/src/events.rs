// SPDX-License-Identifier: GPL-3.0-only
//! The `raul-events` v1 Contract: the JSON lines a driver command (`raul ttw --json`, later `raul lod generate
//! --json`) prints on standard output while it runs. Every line is one [`EventLine`]; the last one is always
//! `finished`. A `result` line comes right before it whenever the command got far enough to have one.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The Contract name of an event line.
pub const CONTRACT: &str = "raul-events";

/// The version of the event lines this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// One line of the stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EventLine {
    /// The line's version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-events`.
    #[schemars(extend("const" = "raul-events"))]
    pub contract: String,
    /// The line's number in the stream, from 1.
    pub seq: u64,
    /// The `raul` command that prints the stream, for example `ttw`.
    pub command: String,
    /// What happened.
    #[serde(flatten)]
    pub event: Event,
}

/// What happened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// The checks passed and the Component program is about to start.
    Started {
        /// Nothing is written.
        dry_run: bool,
        /// The run continues an interrupted one (`--resume`).
        resume: bool,
        /// The Instance.
        instance: PathBuf,
        /// The Profile that gets the result.
        profile: String,
        /// The program that runs.
        tool: ToolInfo,
    },
    /// A phase of the Component's work began. The phases are the Component's own (for `ttw`: `package`,
    /// `preflight`, `checks`, `assets`, `post_install`), then the Toolbox's: `add_mod`.
    Phase {
        /// The phase.
        phase: String,
    },
    /// Progress within the current phase.
    Progress {
        /// The phase.
        phase: String,
        /// Items done so far.
        done: u64,
        /// Items in total; 0 when the total is not known yet.
        of: u64,
        /// What is being worked on, for people.
        message: Option<String>,
    },
    /// Something did not go as planned, but the run goes on.
    Warning {
        /// For people.
        message: String,
    },
    /// The command's result document: a `raul-ttw-result` for `ttw`.
    Result {
        /// The document.
        result: serde_json::Value,
    },
    /// The run ended. Always the last line; the exit code of the process equals `exit_code`.
    Finished {
        /// How it ended.
        status: Status,
        /// The exit code of `raul`.
        exit_code: u8,
        /// The stable error code of the command's result, unless the status is `success`.
        code: Option<String>,
        /// For people; never matched on.
        message: Option<String>,
        /// `--resume` continues the run.
        resumable: bool,
        /// Whether the Instance was changed.
        changed: bool,
    },
}

/// How a driver command ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

/// The Component program a driver runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolInfo {
    /// The program's name, for example `ttwright`.
    pub name: String,
    /// Its version as the program reported it in its result; `null` in the `started` event, before it ran.
    pub version: Option<String>,
    /// The file that ran.
    pub path: PathBuf,
    /// The SHA-256 of that file, checked again right before it was started.
    pub sha256: String,
}

/// The JSON Schema of [`EventLine`], as `contracts/raul-events.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::schema_text(&schemars::schema_for!(EventLine))
}
