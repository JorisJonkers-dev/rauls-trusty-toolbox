// SPDX-License-Identifier: GPL-3.0-only
//! The engine step of the setup: puts nv-rs in the Instance and checks it. It never fetches anything (#179, ADR-0005):
//! the engine is the build that ships next to `raul` ([`super::packaged`]), checked against the checksum list the
//! release wrote beside it. A developer may still build it from a source workspace (`source`, with a Rust toolchain).
//! A copy of raul that ships no engine fails the step with plain instructions.
//!
//! **This whole file goes away with the play mode (#260)**, which removes the separate engine program: delete this
//! module, `packaged.rs`, the `engine` and `packaged` mod lines, the `EngineOptions` re-export and the
//! `Request::engine_options` field in `setup.rs`, the `Step::Engine` arm in `run.rs`, and the `nvmod-install`
//! dependency.
//!
//! The library reports no progress, so the step reports what it is about to do, honours a stop asked for before the
//! install starts, bridges a stop asked for during a source build to nvmod's process-wide stop request, and reports
//! the passed check afterwards. Both ways stage the install and roll it back on failure, so a refused, stopped or
//! failed install leaves the Instance as it was. A run that dies after the install committed and before the journal
//! recorded the step is refused on resume with "already installed"; the caller then marks the step done or starts
//! over. `cargo` comes from the request, else `NVMOD_CARGO`, else `PATH`, as in nvmod's setup.

use nvmod_install::{InstallError, Options};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use super::packaged;
use super::run::with_cancel_bridge;
use super::{CancelToken, Error, ErrorKind, Event, Progress, Request, Step};

/// The detail label while nv-rs is built from a source workspace (the build, then the check).
pub const BUILDING: &str = "setup.engine.building";
/// The detail label while the packaged engine is checked against its checksums and put in the Instance.
pub const INSTALLING: &str = "setup.engine.installing";
/// The detail label once the installed nv-rs passed its check.
pub const CHECKED: &str = "setup.engine.checked";

const JOB: &str = "engine.install";

/// How the engine step gets nv-rs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineOptions {
    /// An nv-rs workspace to build instead of using the packaged engine (for developers; needs a Rust toolchain).
    pub source: Option<PathBuf>,
    /// The `cargo` program for a source build; `None` looks for one on `PATH`.
    pub cargo: Option<PathBuf>,
    /// The packaged engine's folder; `None` looks for the one that ships next to the running `raul`. Not kept in the journal.
    pub packaged: Option<PathBuf>,
}

/// Run the engine step. `Ok(false)` means a stop was asked for before the install started; nothing was written.
///
/// # Errors
/// [`ErrorKind::Refused`] when the library refused before it wrote anything (an engine is already installed, or
/// there is no source to build); [`ErrorKind::Unverified`] when a checksum, an attestation or a response did not
/// check out (nothing is installed); [`ErrorKind::Failed`] when the build, the unpacking or the check failed (what
/// was written is removed).
pub(super) fn run(
    req: &Request,
    step: &Step,
    progress: &mut dyn Progress,
    cancel: &CancelToken,
) -> Result<bool, Error> {
    let opts = &req.engine_options;
    let detail = |progress: &mut dyn Progress, label: &str| {
        progress.report(&Event::Detail {
            step: step.clone(),
            label: label.to_owned(),
            done: None,
            total: None,
        });
    };
    let Some(source) = opts.source.clone() else {
        // The packaged engine: found beside the running program unless the request names it.
        let Some(dir) = opts.packaged.clone().or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| packaged::locate(&exe))
        }) else {
            return Err(map_problem(step, packaged::missing()));
        };
        if !dir.join(packaged::MANIFEST).is_file() {
            return Err(map_problem(step, packaged::missing()));
        }
        detail(progress, INSTALLING);
        if cancel.is_cancelled() {
            return Ok(false);
        }
        return match packaged::install(&req.instance, &dir, &|| cancel.is_cancelled()) {
            Ok(()) => {
                detail(progress, CHECKED);
                Ok(true)
            }
            Err(_) if cancel.is_cancelled() => Ok(false),
            Err(problem) => Err(map_problem(step, problem)),
        };
    };
    // A source build. `cargo` comes from the request, else `NVMOD_CARGO`, else `PATH`; without one nothing is built,
    // because the library would otherwise fetch a release.
    let cargo = opts
        .cargo
        .clone()
        .or_else(|| std::env::var_os("NVMOD_CARGO").map(PathBuf::from));
    if nvmod_install::resolve_cargo(cargo.as_deref()).is_none() {
        return Err(Error {
            kind: ErrorKind::Refused,
            step: step.clone(),
            job: JOB,
            summary: "Building the engine from source needs a Rust toolchain, and none was found."
                .to_owned(),
        });
    }
    detail(progress, BUILDING);
    if cancel.is_cancelled() {
        return Ok(false);
    }
    let options = Options {
        instance: req.instance.clone(),
        source: Some(source),
        release: false,
        cargo,
        // Never read: a source build fetches nothing.
        release_url: String::new(),
        asset: String::new(),
        repo: String::new(),
        gh: PathBuf::new(),
        curl: PathBuf::new(),
        retry_delay: Duration::ZERO,
    };
    // The library runs the build as a child process that stops on nvmod's process-wide stop request, so a stop asked
    // for through `cancel` is bridged to it for the length of the install.
    match with_cancel_bridge(cancel, || nvmod_install::install(&options)) {
        Ok(_) => {
            detail(progress, CHECKED);
            Ok(true)
        }
        // A stop that ended the install ends the step as cancelled; the library rolled back what it had written.
        Err(_) if cancel.is_cancelled() => Ok(false),
        Err(e) => Err(map(step, e)),
    }
}

fn map_problem(step: &Step, problem: packaged::Problem) -> Error {
    Error {
        kind: problem.kind,
        step: step.clone(),
        job: JOB,
        summary: problem.summary,
    }
}

/// The library's error as the step's typed error. The summary is the library's text: the build log, the failed
/// check's output or the reason, never a command line of its own.
fn map(step: &Step, error: InstallError) -> Error {
    let (kind, summary) = match error {
        InstallError::Refused(why) => (ErrorKind::Refused, why),
        InstallError::Verification(why) => (ErrorKind::Unverified, why),
        InstallError::Failed(why) => (ErrorKind::Failed, why),
    };
    Error {
        kind,
        step: step.clone(),
        job: JOB,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_library_error_maps_to_a_kind_and_keeps_its_text() {
        for (error, kind) in [
            (InstallError::Refused("r".into()), ErrorKind::Refused),
            (
                InstallError::Verification("v".into()),
                ErrorKind::Unverified,
            ),
            (InstallError::Failed("f".into()), ErrorKind::Failed),
        ] {
            let text = error.to_string();
            let mapped = map(&Step::Engine, error);
            assert_eq!(mapped.kind, kind);
            assert_eq!(mapped.summary, text);
            assert_eq!(mapped.job, "engine.install");
        }
    }

    #[test]
    fn the_defaults_build_nothing_and_name_no_folder() {
        let o = EngineOptions::default();
        assert!(o.source.is_none() && o.cargo.is_none() && o.packaged.is_none());
        let from_empty: EngineOptions = serde_json::from_str("{}").expect("defaults");
        assert_eq!(from_empty, o);
    }

    #[test]
    fn an_old_journal_that_names_a_release_still_reads() {
        let old = r#"{"release":true,"release_url":"https://x","gh":"gh","curl":"curl"}"#;
        let o: EngineOptions = serde_json::from_str(old).expect("old options");
        assert_eq!(o, EngineOptions::default());
    }
}
