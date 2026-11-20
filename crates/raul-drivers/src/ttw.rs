// SPDX-License-Identifier: GPL-3.0-only
//! `raul ttw`: Tale of Two Wastelands through ttwright-rs. This module holds the `raul-ttw-result` v1 Contract, the
//! document the `result` event of `raul ttw --json` carries, and the driver that produces it.

pub(crate) mod driver;
pub(crate) mod inproc;
pub mod package;

use std::path::PathBuf;

pub use driver::{
    DEFAULT_TIMEOUT, Outcome, TTWRIGHT, TTWRIGHT_EVENTS_VERSION, TtwRequest, WORK_DIR, exit_code,
    install, refused,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::events::{Status, ToolInfo};

/// The Contract name of the result document.
pub const CONTRACT: &str = "raul-ttw-result";

/// The version of the result document this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// The Mod folder the TTW output becomes.
pub const MOD_NAME: &str = "Tale of Two Wastelands";

/// The result of `raul ttw` (Contract `raul-ttw-result`, version 1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TtwResult {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-ttw-result`.
    #[schemars(extend("const" = "raul-ttw-result"))]
    pub contract: String,
    /// How the run ended; the same as the `finished` event's.
    pub status: Status,
    /// A dry run: ttwright read the package and checked the game copies, and nothing was written.
    pub dry_run: bool,
    /// The run continued an interrupted one.
    pub resumed: bool,
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile that got the Mod.
    pub profile: String,
    /// The Mod added to the Instance (`Tale of Two Wastelands`), or `null` when none was.
    #[serde(rename = "mod")]
    pub mod_name: Option<String>,
    /// The plugins the Profile enabled with the Mod, in load order.
    pub plugins_enabled: Vec<String>,
    /// Files in the Mod, or `null` when no Mod was added.
    pub files: Option<u64>,
    /// The folder ttwright wrote to. It is kept after a cancelled or failed run so `--resume` can continue there,
    /// and removed after a successful one.
    pub staging: Option<PathBuf>,
    /// `raul ttw --resume` with the same arguments continues this run.
    pub resumable: bool,
    /// The ttwright driver that ran in process: the `raul` executable, with its name and version and no SHA-256.
    /// `null` when no install started.
    pub tool: Option<ToolInfo>,
    /// The `ttwright-result` v1 document ttwright returned, as it is, or `null` when it returned none.
    pub ttwright: Option<serde_json::Value>,
    /// Set unless the status is `success`.
    pub error: Option<TtwError>,
}

/// Why `raul ttw` did not succeed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TtwError {
    /// The stable code callers match on.
    pub code: TtwErrorCode,
    /// For `ttwright_failed`, `unsupported` and `cancelled`: ttwright's own `error.code`, for example
    /// `check_failed` or `unsupported_version`.
    pub ttwright_code: Option<String>,
    /// Prose for people; never matched on.
    pub message: String,
}

/// The stable error codes of `raul ttw`. The exit code of each is in `contracts/README.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TtwErrorCode {
    /// The Instance does not exist or cannot be opened (exit code 2).
    InstanceInvalid,
    /// The Profile does not exist (exit code 2).
    ProfileMissing,
    /// A game folder or the TTW package was given neither as a flag nor in `raul config`, or is not what it
    /// should be (exit code 2).
    MissingInput,
    /// The Instance already has a `Tale of Two Wastelands` Mod (exit code 2).
    ModExists,
    /// Kept for older result documents; raul no longer produces it (exit code 2).
    ToolMissing,
    /// Kept for older result documents; raul no longer produces it (exit code 3).
    ToolUnverifiable,
    /// The install could not be prepared (its work folder), ttwright ended with an exit code the Toolbox does not
    /// document, or its result was not a success (exit code 4).
    ToolFailed,
    /// Kept for older result documents; raul no longer produces it (exit code 2).
    ContractRefused,
    /// ttwright ran and failed (its exit code 1 or 2; exit code 4); `ttwright_code` says why.
    TtwrightFailed,
    /// ttwright does not support the package or one of its features (its exit code 3; exit code 2). Nothing was
    /// written.
    Unsupported,
    /// The run was cancelled (exit code 4); `--resume` continues it. This also covers the install timeout
    /// (`--timeout-secs`): a timed-out install is cancelled, not failed.
    Cancelled,
    /// ttwright succeeded but adding its output to the Instance failed and was rolled back (exit code 4).
    AddModFailed,
}

/// The JSON Schema of [`TtwResult`], as `contracts/raul-ttw-result.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::schema_text(&schemars::schema_for!(TtwResult))
}
