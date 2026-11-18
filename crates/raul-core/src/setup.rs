// SPDX-License-Identifier: GPL-3.0-only
//! The setup job (ADR-0019): `raul` plans the steps from a typed request and runs each as a call in its own
//! process, from a base copy to a modded, checked Instance. The steps are the ones nvmod's `setup` runs today: init
//! or the MO2 import, the first Profile, the engine, the target, TTW, the Mod installs, Staging, the Load order and
//! the check. The mod-management steps call the typed jobs of `nvmod-api`; the TTW step calls the in-process
//! ttwright driver of `raul-drivers`. No argument vector is built and no command is re-entered.
//!
//! The job has the shape of ADR-0016: [`run`]`(request, &mut dyn Progress, &CancelToken) -> Result<Outcome,
//! Error>`. A cancelled run is an [`Outcome::Cancelled`], a result and not an error; the first failing step stops
//! the run with its typed [`Error`]. The resume journal and the wizard are not part of this module.

mod plan;
mod run;

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub use nvmod_api::CancelToken;
pub use plan::plan;
pub use run::run;

/// What the person chose for a setup. Every path is absolute; the caller makes them so.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// The base copy of the game.
    pub base: PathBuf,
    /// The Instance folder to make.
    pub instance: PathBuf,
    /// The masterlist file for the Load order sort and the check (the caller fetches and verifies it).
    pub masterlist: PathBuf,
    /// The Profile: the name of the first one, or the MO2 profile to import. `None` takes the default name.
    #[serde(default)]
    pub profile: Option<String>,
    /// An MO2 folder to import instead of making an empty Instance.
    #[serde(default)]
    pub import_mo2: Option<PathBuf>,
    /// Copy files into the Staging tree and the import instead of linking them.
    #[serde(default)]
    pub copy: bool,
    /// Put the nv-rs engine in the Instance and make it the target. `false` keeps the vanilla target.
    #[serde(default)]
    pub engine: bool,
    /// Install Tale of Two Wastelands; `None` skips the step.
    #[serde(default)]
    pub ttw: Option<Ttw>,
    /// The Mod archives to install, in order.
    #[serde(default)]
    pub mods: Vec<PathBuf>,
    /// The answers file for the Mod installers' questions.
    #[serde(default)]
    pub answers: Option<PathBuf>,
    /// A tool that extracts the Mod archives nvmod cannot read itself.
    #[serde(default)]
    pub extract_tool: Option<PathBuf>,
    /// Sort an imported Instance too. An imported Instance keeps the owner's MO2 order unless this is set.
    #[serde(default)]
    pub sort: bool,
    /// A catalog file for the check, instead of the built-in one.
    #[serde(default)]
    pub catalog: Option<PathBuf>,
    /// Steps that are already done (a resume): they are planned as skipped.
    #[serde(default)]
    pub done: Vec<Step>,
}

/// The TTW step's choices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ttw {
    /// The Fallout 3 folder.
    pub fallout3: PathBuf,
    /// The Fallout: New Vegas folder; the Instance's base game when `None`.
    #[serde(default)]
    pub newvegas: Option<PathBuf>,
    /// The TTW package (the `.mpi` file).
    #[serde(default)]
    pub package: Option<PathBuf>,
    /// How long the install may run; ttwright's default when `None`.
    #[serde(default)]
    pub timeout: Option<Duration>,
}

/// One step of the setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Step {
    /// Make the Instance skeleton for the base copy.
    Init,
    /// Import an MO2 profile into a new Instance.
    ImportMo2,
    /// Make the first Profile.
    Profile,
    /// Put the nv-rs engine in the Instance (`#275`; today it only reports that it is not done yet).
    Engine,
    /// Make nv-rs the target of the Instance.
    Target,
    /// Install Tale of Two Wastelands.
    Ttw,
    /// Install one Mod archive.
    Mod(PathBuf),
    /// Build the Staging tree.
    Stage,
    /// Compute the Load order with loot-rs, write it and stamp the plugin file times.
    Sort,
    /// Keep the imported Load order as it is and stamp the plugin file times.
    Stamp,
    /// Run the Doctor on the finished Instance and plan the launch.
    Verify,
}

impl Step {
    /// The stable id of the step, as nvmod's `setup` names it: `init`, `import-mo2`, `profile`, `engine`,
    /// `target`, `ttw`, `mod:<file name>`, `stage`, `sort`, `stamp`, `verify`.
    #[must_use]
    pub fn id(&self) -> String {
        match self {
            Self::Init => "init".to_owned(),
            Self::ImportMo2 => "import-mo2".to_owned(),
            Self::Profile => "profile".to_owned(),
            Self::Engine => "engine".to_owned(),
            Self::Target => "target".to_owned(),
            Self::Ttw => "ttw".to_owned(),
            Self::Mod(archive) => format!(
                "mod:{}",
                archive.file_name().map_or_else(
                    || archive.to_string_lossy().into_owned(),
                    |n| n.to_string_lossy().into_owned()
                )
            ),
            Self::Stage => "stage".to_owned(),
            Self::Sort => "sort".to_owned(),
            Self::Stamp => "stamp".to_owned(),
            Self::Verify => "verify".to_owned(),
        }
    }

    /// The label key the interface words in Raul's voice, for example `setup.stage`.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Init => "setup.init",
            Self::ImportMo2 => "setup.import-mo2",
            Self::Profile => "setup.profile",
            Self::Engine => "setup.engine",
            Self::Target => "setup.target",
            Self::Ttw => "setup.ttw",
            Self::Mod(_) => "setup.mod",
            Self::Stage => "setup.stage",
            Self::Sort => "setup.sort",
            Self::Stamp => "setup.stamp",
            Self::Verify => "setup.verify",
        }
    }
}

/// Why a planned step is not run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkipReason {
    /// The request lists the step as done.
    Done,
    /// An imported Instance keeps its MO2 Load order; the sort is opt-in.
    KeepsImportedOrder,
}

/// A step of the plan and whether it runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Planned {
    /// The step.
    pub step: Step,
    /// Why it is not run, `None` when it is.
    pub skip: Option<SkipReason>,
}

/// A progress event of the setup, in the order the job made them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum Event {
    /// The plan, before anything runs.
    Planned {
        /// Every step in order, with the ones that will be skipped.
        steps: Vec<Planned>,
    },
    /// A step began. `index` counts the plan from 1.
    Started {
        /// The step.
        step: Step,
        /// Its place in the plan, from 1.
        index: usize,
        /// The number of steps in the plan.
        total: usize,
    },
    /// Progress inside a step.
    Detail {
        /// The step.
        step: Step,
        /// A stable label key of the job that reports it, for example `stage.stage.building-tree`.
        label: String,
        /// Items done, when the job counts.
        done: Option<u64>,
        /// Items in total, when known.
        total: Option<u64>,
    },
    /// A step was not run.
    Skipped {
        /// The step.
        step: Step,
        /// Its place in the plan, from 1.
        index: usize,
        /// The number of steps in the plan.
        total: usize,
        /// Why.
        reason: SkipReason,
    },
    /// A step ended well.
    Finished {
        /// The step.
        step: Step,
        /// Its place in the plan, from 1.
        index: usize,
        /// The number of steps in the plan.
        total: usize,
    },
}

/// Where the setup's progress goes. A closure `FnMut(&Event)` is one.
pub trait Progress {
    /// One event.
    fn report(&mut self, event: &Event);
}

impl<F: FnMut(&Event)> Progress for F {
    fn report(&mut self, event: &Event) {
        self(event);
    }
}

/// How a setup that did not fail ended.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Every step that was planned ran or was skipped.
    Done(Output),
    /// A stop was asked for; the run ended before the next step. The Instance holds the steps in
    /// [`Output::completed`] and nothing of the one that was about to start.
    Cancelled(Output),
}

/// What a setup did.
#[derive(Debug, Clone)]
pub struct Output {
    /// The steps that ran to their end, in order.
    pub completed: Vec<Step>,
    /// The steps that were planned and not run, with the reason.
    pub skipped: Vec<(Step, SkipReason)>,
    /// The result of the check, when its step ran.
    pub verify: Option<nvmod_api::verify::VerifyOutput>,
}

/// The class of an [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    /// The step refused its input before it wrote anything.
    Refused,
    /// The step failed and put back what it had started.
    Failed,
    /// Every step ran and the check does not pass.
    Unverified,
}

impl ErrorKind {
    /// A stable code: `refused`, `failed` or `unverified`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Refused => "refused",
            Self::Failed => "failed",
            Self::Unverified => "unverified",
        }
    }
}

/// Why a setup stopped at a step. It never holds a command line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Error {
    /// The class.
    pub kind: ErrorKind,
    /// The step that stopped the run.
    pub step: Step,
    /// The job that failed, as `<area>.<action>`, for example `stage.stage`.
    pub job: &'static str,
    /// What went wrong, in plain words.
    pub summary: String,
}

impl Error {
    /// The error as one block of plain text to copy into a bug report: `raul <version>`, the step, the job, the
    /// code and the summary, with the home directory written as `~`.
    #[must_use]
    pub fn copy_text(&self) -> String {
        let summary = std::env::var("HOME")
            .ok()
            .map_or_else(|| self.summary.clone(), |home| tilde(&self.summary, &home));
        format!(
            "raul {}\nstep: {}\njob: {}\nerror: {}\n{summary}",
            env!("CARGO_PKG_VERSION"),
            self.step.id(),
            self.job,
            self.kind.code()
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary)
    }
}

impl std::error::Error for Error {}

/// `text` with the home directory written as `~` where a path starts with it.
fn tilde(text: &str, home: &str) -> String {
    let home = home.trim_end_matches('/');
    if home.len() < 2 {
        return text.to_owned();
    }
    let path_char = |c: char| c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | '~');
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(home) {
        let after = &rest[at + home.len()..];
        out.push_str(&rest[..at]);
        let starts = !out.chars().next_back().is_some_and(path_char);
        let ends = after
            .chars()
            .next()
            .is_none_or(|c| c == '/' || !path_char(c));
        out.push_str(if starts && ends { "~" } else { home });
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_home_directory_is_written_as_a_tilde_only_where_a_path_starts_with_it() {
        let home = "/home/someone";
        assert_eq!(tilde("/home/someone/Data", home), "~/Data");
        assert_eq!(tilde("at /home/someone", home), "at ~");
        assert_eq!(
            tilde("/mnt/home/someone/Data", home),
            "/mnt/home/someone/Data"
        );
        assert_eq!(tilde("/home/someone2/Data", home), "/home/someone2/Data");
        assert_eq!(tilde("/a", "/"), "/a");
    }

    #[test]
    fn steps_have_stable_ids_and_labels() {
        assert_eq!(Step::ImportMo2.id(), "import-mo2");
        assert_eq!(Step::Mod(PathBuf::from("/m/A B.zip")).id(), "mod:A B.zip");
        assert_eq!(Step::Mod(PathBuf::from("/m/A.zip")).label(), "setup.mod");
        assert_eq!(Step::Verify.label(), "setup.verify");
    }

    #[test]
    fn an_error_kind_has_a_stable_code() {
        assert_eq!(ErrorKind::Refused.code(), "refused");
        assert_eq!(ErrorKind::Failed.code(), "failed");
        assert_eq!(ErrorKind::Unverified.code(), "unverified");
    }
}
