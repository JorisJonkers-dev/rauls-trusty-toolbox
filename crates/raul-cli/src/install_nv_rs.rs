// SPDX-License-Identifier: GPL-3.0-only
//! `raul install-nv-rs --instance <dir> [--source <dir>] [--cargo <path>]`: put the engine in an Instance. It is the
//! build that ships next to `raul`, checked against its checksums; nothing is fetched (#179). `--source` builds a
//! developer's nv-rs workspace instead. The options that used to fetch a release (`--release`, `--release-url`,
//! `--asset`, `--repo`, `--gh`, `--curl`, `--retry-delay-ms`) are gone and are a usage error. It replaces nvmod's
//! command of the same name, which fetches.

use std::io::Write;
use std::path::PathBuf;

use nvmod_api::{CancelToken, ErrorKind, JobError, NoProgress, Outcome};
use raul_core::instance_lock::InstanceGuard;
use raul_core::setup::packaged;

use crate::{FAILED, OK, REFUSED, USAGE};

const USAGE_LINE: &str =
    "usage: raul install-nv-rs --instance <dir> [--source <dir>] [--cargo <path>]";
/// Options that take a value.
const VALUED: &[&str] = &["instance", "source", "cargo"];
/// The options that fetched a release: gone, with the reason said.
const FETCH_OPTIONS: &[&str] = &[
    "release",
    "release-url",
    "asset",
    "repo",
    "gh",
    "curl",
    "retry-delay-ms",
];
/// The exit code of a verification failure, as nvmod's command gave it.
const VERIFICATION: u8 = nvmod_cli::VERIFICATION;

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (mut instance, mut source, mut cargo) = (None, None, None);
    let mut words = args.iter();
    while let Some(word) = words.next() {
        let Some(name) = word.strip_prefix("--") else {
            let _ = writeln!(err, "{USAGE_LINE}");
            return USAGE;
        };
        if FETCH_OPTIONS.contains(&name) {
            let _ = writeln!(
                err,
                "raul install-nv-rs: {word} is gone: the engine comes with raul and is never downloaded"
            );
            return USAGE;
        }
        if !VALUED.contains(&name) {
            let _ = writeln!(err, "raul install-nv-rs: unknown option {word}");
            return USAGE;
        }
        let Some(value) = words.next() else {
            let _ = writeln!(err, "raul install-nv-rs: {word} needs a value");
            return USAGE;
        };
        match name {
            "instance" => instance = Some(PathBuf::from(value)),
            "source" => source = Some(PathBuf::from(value)),
            "cargo" => cargo = Some(PathBuf::from(value)),
            _ => {}
        }
    }
    let Some(instance) = instance else {
        let _ = writeln!(err, "{USAGE_LINE}");
        return USAGE;
    };
    if cargo.is_some() && source.is_none() {
        let _ = writeln!(
            err,
            "raul install-nv-rs: --cargo only goes with --source (it builds the engine from source)"
        );
        return USAGE;
    }
    let _guard = match InstanceGuard::acquire(&instance, "install-nv-rs") {
        Ok(guard) => guard,
        Err(e) => {
            let _ = writeln!(err, "raul install-nv-rs: refused: {e}");
            return REFUSED;
        }
    };
    let result = packaged::job(
        &instance,
        source.as_deref(),
        cargo.as_deref(),
        None,
        &mut NoProgress,
        &CancelToken::new(),
    );
    match result {
        Ok(Outcome::Done(done)) => {
            let how = match done.method {
                "source" => "built from source",
                _ => "installed from the build that ships with raul and checked",
            };
            let _ = writeln!(
                out,
                "raul install-nv-rs: the engine {how}: {}",
                done.binary.display()
            );
            OK
        }
        Ok(Outcome::Cancelled) => REFUSED,
        Err(e) => failure(&e, err),
    }
}

/// Print `e` the way this command does: what a failed check printed first, then one headline line last, so the last
/// line is what a reader (the setup wizard) shows.
pub(crate) fn failure(e: &JobError, err: &mut dyn Write) -> u8 {
    let (head, detail) = e
        .summary
        .split_once('\n')
        .unwrap_or((e.summary.as_str(), ""));
    if !detail.trim().is_empty() {
        let _ = writeln!(err, "{}", detail.trim_end());
    }
    let e = &JobError::new(e.kind, "install_nvrs.install", head);
    match e.kind {
        ErrorKind::Refused => {
            let _ = writeln!(err, "raul install-nv-rs: refused: {}", e.summary);
            REFUSED
        }
        ErrorKind::Corrupt => {
            let _ = writeln!(
                err,
                "raul install-nv-rs: verification failed: {}\nnothing was installed",
                e.summary
            );
            VERIFICATION
        }
        _ => {
            let _ = writeln!(
                err,
                "raul install-nv-rs: failed and was rolled back: {}",
                e.summary
            );
            FAILED
        }
    }
}
