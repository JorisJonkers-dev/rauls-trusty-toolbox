// SPDX-License-Identifier: GPL-3.0-only
//! The JSON the `nvmod` command line prints, as typed values. The shapes are the ones of `nvmod-cli`: a change
//! there that breaks one of these fails the end-to-end tests of this crate.

use nvmod_ui_model::wizard::{FomodOption, FomodQuestion, GroupRule, OptionKind};
use serde::Deserialize;

/// One thing a check found (`nvmod_core::Finding`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Finding {
    /// Stable identifier, `<area>.<what>`.
    pub code: String,
    /// `info`, `warning` or `error`.
    pub severity: String,
    /// What it is about.
    pub subject: String,
    /// What was found.
    pub message: String,
}

/// `nvmod import-mo2 --json` on success.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ImportReport {
    /// The new Instance's folder.
    pub instance: String,
    /// The profile imported.
    pub profile: String,
    /// Enabled Mods, highest priority first.
    pub enabled_mods: Vec<String>,
    /// Active plugins in order.
    pub plugins: Vec<String>,
    /// All plugins in load order.
    pub load_order: Vec<String>,
    /// Mod folders mirrored.
    pub mods: usize,
    /// Files hard-linked.
    pub linked: usize,
    /// Files copied.
    pub copied: usize,
    /// What could not be mapped or was left out.
    pub unmapped: Vec<Finding>,
}

/// A FOMOD choice made without asking.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Inferred {
    /// The question id it would have had.
    pub id: String,
    /// The options chosen.
    pub selected: Vec<String>,
    /// Why this was unambiguous.
    pub reason: String,
}

/// `nvmod install --json` on success (also a dry run).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InstallReport {
    /// `installed` or `dry-run`.
    pub status: String,
    /// True for a dry run.
    pub dry_run: bool,
    /// The Mod's folder name.
    #[serde(rename = "mod")]
    pub mod_name: String,
    /// The profile.
    pub profile: String,
    /// Files of the Mod.
    pub files: Vec<String>,
    /// Plugins newly activated.
    pub plugins: Vec<String>,
    /// FOMOD choices made without asking.
    pub inferred: Vec<Inferred>,
    /// Files in the Staging tree afterwards, none for a dry run.
    pub staged_files: Option<usize>,
    /// The journal entry `nvmod rollback` uses, none for a dry run.
    pub journal: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuestionOptionDto {
    name: String,
    description: String,
    #[serde(rename = "type")]
    kind: Option<OptionKind>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuestionDto {
    id: String,
    step: String,
    group: String,
    kind: GroupRule,
    min: usize,
    max: usize,
    options: Vec<QuestionOptionDto>,
}

impl From<QuestionDto> for FomodQuestion {
    fn from(q: QuestionDto) -> Self {
        FomodQuestion {
            id: q.id,
            step: q.step,
            group: q.group,
            rule: q.kind,
            min: q.min,
            max: q.max,
            options: q
                .options
                .into_iter()
                .map(|o| FomodOption {
                    name: o.name,
                    description: o.description,
                    kind: o.kind,
                })
                .collect(),
        }
    }
}

/// `nvmod install --json` when the FOMOD needs answers (exit 1).
#[derive(Debug, Deserialize)]
pub(crate) struct QuestionsDoc {
    pub(crate) questions: Vec<QuestionDto>,
}

/// `nvmod import-mo2 --json` when the profile files disagree (exit 1).
#[derive(Debug, Deserialize)]
pub(crate) struct DisagreeDoc {
    pub(crate) disagreements: Vec<Finding>,
}

/// The installer's own result record inside the TTW report (`ttw-installer-result.v1`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TtwRecord {
    /// `success` or `failed`.
    pub outcome: String,
    /// The installer's exit code.
    pub exit_code: i64,
    /// Whether the installer verified Fallout 3.
    pub fallout3_verified: bool,
    /// Files the installer wrote.
    pub files_written: u64,
    /// What the installer said.
    pub messages: Vec<String>,
}

/// `nvmod ttw --json` on success.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TtwReport {
    /// `installed`.
    pub status: String,
    /// The Mod's folder name.
    #[serde(rename = "mod")]
    pub mod_name: String,
    /// The profile.
    pub profile: String,
    /// How many files the Mod has.
    pub files: usize,
    /// Plugins newly activated.
    pub plugins: Vec<String>,
    /// The journal entry `nvmod rollback` uses.
    pub journal: Option<String>,
    /// The installer's result record.
    pub result: TtwRecord,
}

/// Counts of a Doctor run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct DoctorSummary {
    /// Findings of severity info.
    pub info: usize,
    /// Findings of severity warning.
    pub warnings: usize,
    /// Findings of severity error.
    pub errors: usize,
}

/// `nvmod doctor --instance <dir> --json` (the report of `nvmod_doctor::BaseReport`; the fields the Doctor
/// screen does not show yet are not read).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DoctorReport {
    /// The game folder the Instance runs on.
    pub base: String,
    /// What was found.
    pub findings: Vec<Finding>,
    /// Counts.
    pub summary: DoctorSummary,
}

impl From<&DoctorReport> for nvmod_ui_model::doctor::DoctorView {
    fn from(report: &DoctorReport) -> Self {
        Self {
            base: report.base.clone(),
            findings: report
                .findings
                .iter()
                .map(|f| nvmod_ui_model::doctor::DoctorFinding {
                    code: f.code.clone(),
                    severity: nvmod_ui_model::doctor::Severity::parse(&f.severity),
                    subject: f.subject.clone(),
                    message: f.message.clone(),
                })
                .collect(),
        }
    }
}

/// A message of the masterlist or a finding of the sort (`kind` and `text`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SortMessage {
    /// What kind of message it is (`say`, `warn`, `error`, lowercase).
    pub kind: String,
    /// The text.
    pub text: String,
}

/// A finding about one plugin in the sort.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SortFinding {
    /// What kind of finding it is.
    pub kind: String,
    /// The text.
    pub text: String,
    /// True when it is a problem rather than a note.
    pub hard: bool,
}

/// One plugin in the proposed Load order.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SortRow {
    /// 1-based position.
    pub position: usize,
    /// The plugin file name.
    pub name: String,
    /// Whether the profile has it active.
    pub active: bool,
    /// Whether the sorter found the file.
    pub installed: bool,
    /// The masterlist group.
    pub group: Option<String>,
    /// Masterlist messages.
    #[serde(default)]
    pub messages: Vec<SortMessage>,
    /// Messages the sorter generated.
    #[serde(default)]
    pub generated_messages: Vec<SortMessage>,
    /// Findings about the plugin.
    #[serde(default)]
    pub findings: Vec<SortFinding>,
}

/// What `--apply` wrote.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SortApplied {
    /// `plugins.txt` or `loadorder.txt` was rewritten.
    pub lists_written: bool,
    /// Plugin files that got a new modification time.
    pub stamped: usize,
    /// Of those, files whose hard link was replaced by a copy.
    pub detached: usize,
    /// The journal `nvmod rollback` uses, `None` when nothing needed to change.
    pub journal: Option<String>,
}

/// `nvmod sort --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SortReport {
    /// The profile.
    pub profile: String,
    /// Whether the proposed order differs from the current one.
    pub reordered: bool,
    /// The proposed order.
    pub order: Vec<SortRow>,
    /// Messages about the list as a whole.
    #[serde(default)]
    pub general_messages: Vec<SortMessage>,
    /// Plugins the profile names that are not installed.
    #[serde(default)]
    pub not_installed: Vec<String>,
    /// Warnings of the sorter.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// What was written; `None` for a proposal that was not applied.
    pub applied: Option<SortApplied>,
}

/// The state of the installed nv-rs in `nvmod verify`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NvRsCheck {
    /// Whether an nv-rs is installed.
    pub installed: bool,
    /// Whether it matches its install record.
    pub verified: bool,
    /// In words.
    pub message: String,
}

/// One way to start the game.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LaunchEntry {
    /// `linux` or `windows`.
    pub platform: String,
    /// The program.
    pub program: String,
    /// Its arguments.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: std::collections::BTreeMap<String, String>,
    /// The folder to run in.
    pub working_dir: String,
}

/// How the game is started from the Instance.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LaunchPlan {
    /// The name of the path.
    pub path: String,
    /// Whether xNVSE starts.
    pub extender: bool,
    /// What the plan does and cannot do.
    pub note: String,
    /// One per platform.
    pub entries: Vec<LaunchEntry>,
    /// Where native nv-rs launch stands.
    pub nv_rs: String,
}

/// `nvmod verify --json` (exit 0 verified, exit 3 not verified: both carry this report).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VerifyReport {
    /// The overall answer.
    pub verified: bool,
    /// The Doctor's report.
    pub doctor: DoctorReport,
    /// The nv-rs check.
    pub nv_rs: NvRsCheck,
    /// The launch plan.
    pub launch_plan: LaunchPlan,
}

/// `nvmod rollback --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RollbackReport {
    /// The journal undone.
    pub journal: String,
    /// What was undone: `install`, `sort` or `migrate`.
    pub kind: String,
    /// The Mod removed, empty for a sort.
    #[serde(rename = "mod")]
    pub mod_name: String,
    /// The profile restored.
    pub profile: String,
}

/// One pinned tool in `nvmod tools list` and `verify`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ToolRow {
    /// The tool.
    pub name: String,
    /// The pinned version.
    pub version: String,
    /// `verified`, `not fetched`, `corrupt`, `needs-trust`, `no-asset` or `linked`.
    pub state: String,
    /// More words about the state.
    pub detail: String,
}

/// The rows of `nvmod tools list` or `verify`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolsReport {
    /// One row per pinned tool.
    pub rows: Vec<ToolRow>,
    /// False when the command exited 3 because a stored tool is corrupt (`verify` only).
    pub verified: bool,
}

/// What `nvmod target` did. The command prints a sentence, so this is built from the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetReport {
    /// The Instance.
    pub instance: String,
    /// What it runs on now.
    pub target: crate::service::RuntimeTarget,
}

/// One entry of the list `nvmod mods ... --json` prints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModEntry {
    /// 1 is the highest priority.
    pub position: usize,
    /// The folder name.
    pub name: String,
    /// `enabled`, `disabled` or `foreign`.
    pub state: String,
    /// `mod` or `separator`.
    pub kind: String,
    /// Whether the Mod folder exists.
    pub installed: bool,
}

impl From<&ModEntry> for nvmod_ui_model::mods::ModRow {
    fn from(e: &ModEntry) -> Self {
        Self {
            name: e.name.clone(),
            enabled: e.state == "enabled",
        }
    }
}

/// `nvmod mods enable|disable|reorder --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModsReport {
    /// `changed`, `unchanged` or `dry-run`.
    pub status: String,
    /// True for a dry run.
    pub dry_run: bool,
    /// `enable`, `disable` or `reorder`.
    pub action: String,
    /// The profile.
    pub profile: String,
    /// The entries that changed (or would).
    pub changed: Vec<String>,
    /// The list after the change (or as it would be).
    pub mods: Vec<ModEntry>,
    /// Files in the Staging tree after the change.
    pub staged_files: Option<usize>,
    /// The journal `nvmod rollback` undoes; absent for a dry run or when nothing changed.
    pub journal: Option<String>,
}

/// What a run of `nvmod setup --json` came to, read from its event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupReport {
    /// The steps and how each ended.
    pub steps: Vec<nvmod_ui_model::setup::StepView>,
    /// How the run ended.
    pub finish: nvmod_ui_model::setup::Finish,
    /// Whether it was a dry run.
    pub dry_run: bool,
    /// Whether it was a resume.
    pub resume: bool,
}
