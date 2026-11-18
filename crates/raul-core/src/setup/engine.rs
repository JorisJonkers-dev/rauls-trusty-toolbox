// SPDX-License-Identifier: GPL-3.0-only
//! The engine step of the setup: puts nv-rs in the Instance and checks it, by calling the engine install library
//! code (`nvmod-install`, linked as a crate) in this process. It is not nvmod's `install-nv-rs` job or command.
//!
//! **This whole file goes away with the play mode (#260)**, which removes the separate engine program: delete this
//! module, the `engine` mod line, the `EngineOptions` re-export and the `Request::engine_options` field in
//! `setup.rs`, the `Step::Engine` arm in `run.rs`, and the `nvmod-install` dependency.
//!
//! The library reports no progress and cannot be interrupted, so the step reports what it is about to do, honours a
//! stop asked for before the install starts, and reports the passed check afterwards. The library stages the
//! install and rolls it back on failure, so a refused or failed install leaves the Instance as it was.

use std::path::PathBuf;
use std::time::Duration;

use nvmod_install::{DEFAULT_ASSET, DEFAULT_RELEASE_URL, DEFAULT_REPO, InstallError, Options};
use serde::{Deserialize, Serialize};

use super::{CancelToken, Error, ErrorKind, Event, Progress, Request, Step};

/// The detail label while nv-rs is built from a source workspace (the build, then the check).
pub const BUILDING: &str = "setup.engine.building";
/// The detail label while the release is fetched and verified (checksum and attestation), then unpacked.
pub const FETCHING: &str = "setup.engine.fetching";
/// The detail label once the installed nv-rs passed its check.
pub const CHECKED: &str = "setup.engine.checked";

const JOB: &str = "engine.install";

/// How the engine step gets nv-rs: the options of nvmod's `setup` engine step (`--source`, `--release`,
/// `--cargo`, `--release-url`, `--asset`, `--repo`, `--gh`, `--curl`, `--retry-delay-ms`), with its defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineOptions {
    /// The nv-rs workspace to build. With a Rust toolchain and no source the step is refused.
    pub source: Option<PathBuf>,
    /// Fetch the release even when a Rust toolchain exists.
    pub release: bool,
    /// The `cargo` program; `None` looks for one on `PATH`.
    pub cargo: Option<PathBuf>,
    /// Where the release is fetched from.
    pub release_url: String,
    /// The release asset.
    pub asset: String,
    /// The repository the attestation must name.
    pub repo: String,
    /// The `gh` program that verifies the attestation.
    pub gh: PathBuf,
    /// The `curl` program that fetches the release.
    pub curl: PathBuf,
    /// How long to wait between fetch retries, in milliseconds.
    pub retry_delay_ms: u64,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            source: None,
            release: false,
            cargo: None,
            release_url: DEFAULT_RELEASE_URL.to_owned(),
            asset: DEFAULT_ASSET.to_owned(),
            repo: DEFAULT_REPO.to_owned(),
            gh: PathBuf::from("gh"),
            curl: PathBuf::from("curl"),
            retry_delay_ms: 1000,
        }
    }
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
    // The library's own choice of method: the release when asked for or when no toolchain is found.
    let release = opts.release || nvmod_install::resolve_cargo(opts.cargo.as_deref()).is_none();
    detail(progress, if release { FETCHING } else { BUILDING });
    if cancel.is_cancelled() {
        return Ok(false);
    }
    let options = Options {
        instance: req.instance.clone(),
        source: opts.source.clone(),
        release: opts.release,
        cargo: opts.cargo.clone(),
        release_url: opts.release_url.clone(),
        asset: opts.asset.clone(),
        repo: opts.repo.clone(),
        gh: opts.gh.clone(),
        curl: opts.curl.clone(),
        retry_delay: Duration::from_millis(opts.retry_delay_ms),
    };
    match nvmod_install::install(&options) {
        Ok(_) => {
            detail(progress, CHECKED);
            Ok(true)
        }
        Err(e) => Err(map(step, e)),
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
    fn the_defaults_are_those_of_the_nvmod_setup() {
        let o = EngineOptions::default();
        assert_eq!(o.release_url, DEFAULT_RELEASE_URL);
        assert_eq!(o.asset, DEFAULT_ASSET);
        assert_eq!(o.repo, DEFAULT_REPO);
        assert!(!o.release && o.source.is_none());
        let from_empty: EngineOptions = serde_json::from_str("{}").expect("defaults");
        assert_eq!(from_empty, o);
    }
}
