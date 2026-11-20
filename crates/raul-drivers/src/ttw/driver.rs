// SPDX-License-Identifier: GPL-3.0-only
//! The TTW driver: checks the inputs, runs ttwright's install in raul's own process (ttwright starts no program), maps
//! its progress to [`Event`]s while it runs, reads the `ttwright-result` it returns, and adds the output to the
//! Instance as one Mod with nvmod's mod-install library. Every outcome but success leaves the Instance as it was.

use std::path::{Path, PathBuf};
use std::time::Duration;

use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::{InstallError, Limits, Request};

use super::package::{self, Package};
use super::{CONTRACT, MOD_NAME, SCHEMA_VERSION, TtwError, TtwErrorCode, TtwResult};
use crate::events::{Event, Status, ToolInfo};

/// The `ttwright-events` version this build reads.
pub const TTWRIGHT_EVENTS_VERSION: i64 = 1;

/// The ttwright driver's name, as the result's `tool` block reports it.
pub const TTWRIGHT: &str = "ttwright";

/// How long an install may run: the slowest machines reported take about 40 minutes; three hours is a stuck run.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_hours(3);

/// The folder inside the Instance where ttwright writes, kept between a cancelled run and `--resume`.
pub const WORK_DIR: &str = ".raul/ttw";

/// The folder inside the work folder where the package is unpacked to, when it was given as a download archive.
pub const PACKAGE_DIR: &str = "package";

/// What the install itself needs on the Instance's storage besides the unpacked package: its output and the files it
/// works with, rounded up (the same figure the TTW page's Output step checks).
pub const INSTALL_NEED_BYTES: u64 = 40 * 1024 * 1024 * 1024;

/// Reads the free bytes on the storage of a folder (one that does not exist yet counts as its nearest existing parent);
/// `None` when they are not known.
pub type FreeSpace = fn(&Path) -> Option<u64>;

/// What `raul ttw` was asked to do. The CLI fills the paths from flags or from `raul config`.
#[derive(Debug, Clone)]
pub struct TtwRequest {
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile; the Instance's only or selected one when `None`.
    pub profile: Option<String>,
    /// The Fallout 3 folder.
    pub fallout3: Option<PathBuf>,
    /// The Fallout: New Vegas folder; the Instance's base game when `None`.
    pub newvegas: Option<PathBuf>,
    /// The TTW package: the `.mpi` file, or the archive the player downloaded, which holds it (see [`package`]).
    pub package: Option<PathBuf>,
    /// Read and check everything, write nothing.
    pub dry_run: bool,
    /// Continue a cancelled or interrupted install.
    pub resume: bool,
    /// How long ttwright may run.
    pub timeout: Duration,
    /// Reads the free space where the Instance is, for the check before a package is unpacked; no check when `None`.
    pub free_space: Option<FreeSpace>,
}

/// The finished run: the result document and the exit code of `raul`.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// The result.
    pub result: TtwResult,
    /// The exit code of `raul ttw`.
    pub exit_code: u8,
    /// Whether the Instance was changed.
    pub changed: bool,
}

impl Outcome {
    /// The `finished` event of this outcome.
    #[must_use]
    pub fn finished(&self) -> Event {
        Event::Finished {
            status: self.result.status,
            exit_code: self.exit_code,
            code: self.result.error.as_ref().map(|e| {
                serde_json::to_value(e.code)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default()
            }),
            message: self.result.error.as_ref().map(|e| e.message.clone()),
            resumable: self.result.resumable,
            changed: self.changed,
        }
    }
}

/// The exit code `raul ttw` ends with for `status`.
#[must_use]
pub fn exit_code(status: Status) -> u8 {
    match status {
        Status::Success => 0,
        Status::Refused | Status::Unsupported => 2,
        Status::Unverifiable => 3,
        Status::Failed | Status::Cancelled => 4,
    }
}

/// The result of a run that stopped before a Mod was added.
struct Draft {
    result: TtwResult,
}

impl Draft {
    fn new(req: &TtwRequest) -> Draft {
        Draft {
            result: TtwResult {
                schema_version: SCHEMA_VERSION,
                contract: CONTRACT.to_owned(),
                status: Status::Success,
                dry_run: req.dry_run,
                resumed: req.resume,
                instance: req.instance.clone(),
                profile: req.profile.clone().unwrap_or_default(),
                mod_name: None,
                plugins_enabled: Vec::new(),
                files: None,
                staging: None,
                resumable: false,
                tool: None,
                ttwright: None,
                error: None,
            },
        }
    }

    fn fail(mut self, status: Status, code: TtwErrorCode, message: impl Into<String>) -> Outcome {
        self.result.status = status;
        self.result.error = Some(TtwError {
            code,
            ttwright_code: None,
            message: message.into(),
        });
        Outcome {
            exit_code: exit_code(status),
            result: self.result,
            changed: false,
        }
    }

    fn fail_ttwright(
        mut self,
        status: Status,
        code: TtwErrorCode,
        ttwright_code: Option<String>,
        message: impl Into<String>,
    ) -> Outcome {
        self.result.status = status;
        self.result.error = Some(TtwError {
            code,
            ttwright_code,
            message: message.into(),
        });
        Outcome {
            exit_code: exit_code(status),
            result: self.result,
            changed: false,
        }
    }
}

/// The outcome of a run refused before anything started, for a caller that refuses on its own (the CLI, when no
/// Instance is named anywhere).
#[must_use]
pub fn refused(req: &TtwRequest, code: TtwErrorCode, message: &str) -> Outcome {
    Draft::new(req).fail(Status::Refused, code, message)
}

/// The inputs that passed their checks.
struct Ready {
    instance: NvmodInstance,
    profile: String,
    fallout3: PathBuf,
    newvegas: PathBuf,
    package: Package,
}

fn missing(what: &str, flag: &str, key: &str) -> String {
    format!("no {what}: pass {flag} or run `raul config set {key} <path>`")
}

/// Check the inputs; a refusal is its error code and reason. `profile` receives the Profile once it is known.
fn prepare(req: &TtwRequest, profile: &mut String) -> Result<Ready, (TtwErrorCode, String)> {
    use TtwErrorCode::{InstanceInvalid, MissingInput, ModExists, ProfileMissing};
    let instance = NvmodInstance::open(&req.instance)
        .map_err(|e| (InstanceInvalid, format!("{}: {e}", req.instance.display())))?;
    *profile = instance
        .resolve_profile(req.profile.as_deref())
        .map_err(|e| (ProfileMissing, e.to_string()))?;
    let fallout3 = req.fallout3.clone().ok_or_else(|| {
        (
            MissingInput,
            missing("Fallout 3 folder", "--fallout3 <dir>", "game.fallout3"),
        )
    })?;
    let newvegas = req
        .newvegas
        .clone()
        .unwrap_or_else(|| instance.config.base.clone());
    for (dir, game) in [(&fallout3, "Fallout 3"), (&newvegas, "Fallout: New Vegas")] {
        if !dir.is_dir() {
            return Err((
                MissingInput,
                format!("the {game} folder {} is not a folder", dir.display()),
            ));
        }
    }
    let package = req.package.clone().ok_or_else(|| {
        (
            MissingInput,
            missing("TTW package", "--package <file>", "ttw.package"),
        )
    })?;
    // By content: the `.mpi` itself, or an archive that holds exactly one; anything else is refused here, before a
    // work folder exists.
    let package = package::inspect(&package).map_err(|why| (MissingInput, why.to_string()))?;
    if !req.dry_run && find_ci(&instance.root.join("mods"), MOD_NAME).is_some() {
        return Err((
            ModExists,
            format!("the Instance already has a Mod `{MOD_NAME}`; raul does not overwrite it"),
        ));
    }
    Ok(Ready {
        instance,
        profile: profile.clone(),
        fallout3,
        newvegas,
        package,
    })
}

/// Files under `dir`, not following links (a link counts as one file, so a loop of links cannot recurse).
pub(crate) fn count_files(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => count_files(&e.path()),
            _ => 1,
        })
        .sum()
}

pub(crate) fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// Remove the work folder: a link is removed as a link, never followed; a missing folder is fine.
pub(crate) fn clear(path: &Path) -> std::io::Result<()> {
    let result = if is_link(path) {
        std::fs::remove_file(path)
    } else {
        std::fs::remove_dir_all(path)
    };
    match result {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// The `.mpi` ttwright reads: the package itself, or the one unpacked from the download archive into the work folder
/// (a resumed run keeps the one it unpacked). An archive is unpacked in process after a check of the free space where
/// the Instance is; the reason it cannot be is the error, in plain words.
fn mpi_for(
    req: &TtwRequest,
    package: &Package,
    work: &Path,
    emit: &mut dyn FnMut(Event),
) -> Result<PathBuf, String> {
    let Package::Archive { unpacked_bytes, .. } = package else {
        return match package {
            Package::Mpi { path } => Ok(path.clone()),
            Package::Archive { .. } => unreachable!("matched above"),
        };
    };
    let into = work.join(PACKAGE_DIR);
    if req.resume
        && let Some(done) = package::unpacked_in(&into)
    {
        return Ok(done);
    }
    if let Some(free) = req.free_space.and_then(|read| read(work)) {
        let install = if req.dry_run { 0 } else { INSTALL_NEED_BYTES };
        let need = unpacked_bytes.saturating_add(install);
        if free < need {
            return Err(format!(
                "there is not enough free space to unpack the TTW package: it needs {}{} and {} is free where the Instance is. Free {} or use an Instance on a drive with more room",
                package::size_text(*unpacked_bytes),
                if install == 0 {
                    String::new()
                } else {
                    format!(", and the install about {} more", package::size_text(install))
                },
                package::size_text(free),
                package::size_text(need - free),
            ));
        }
    }
    emit(Event::Phase {
        phase: "unpack".to_owned(),
    });
    package::unpack(package, &into).map_err(|why| why.to_string())
}

/// Install TTW as `req` says, emitting [`Event`]s while it runs (not `finished`: the caller emits
/// [`Outcome::finished`] after the `result`). A dry run leaves nothing in the Instance, not even its work folder
/// (unless it continues a run that was stopped, whose work folder is that run's).
pub fn install(req: &TtwRequest, emit: &mut dyn FnMut(Event)) -> Outcome {
    let mut outcome = run(req, emit);
    // `tool` is set once the work folder was made.
    if req.dry_run && !req.resume && outcome.result.tool.is_some() {
        let work = req.instance.join(WORK_DIR);
        if !is_link(&work) {
            let _ = clear(&work);
        }
        outcome.result.staging = None;
        outcome.result.resumable = false;
    }
    outcome
}

#[allow(clippy::too_many_lines)] // one flow: check, run, read, add
fn run(req: &TtwRequest, emit: &mut dyn FnMut(Event)) -> Outcome {
    let mut draft = Draft::new(req);
    let ready = match prepare(req, &mut draft.result.profile) {
        Ok(r) => r,
        Err((code, message)) => return draft.fail(Status::Refused, code, message),
    };
    let work = ready.instance.root.join(WORK_DIR);
    let dest = work.join("out");
    let events = work.join("events.ndjson");
    let prepared = (|| {
        if !req.resume {
            clear(&work)?;
        } else if is_link(&work) {
            return Err(std::io::Error::other(
                "it is a link, not a folder; run without --resume to start again",
            ));
        }
        match std::fs::remove_file(&events) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
        std::fs::create_dir_all(&dest)
    })();
    if let Err(e) = prepared {
        return draft.fail(
            Status::Failed,
            TtwErrorCode::ToolFailed,
            format!("{}: {e}", work.display()),
        );
    }
    // ttwright runs in process; the running executable stands for it, with no file checksum.
    let tool = ToolInfo {
        name: TTWRIGHT.to_owned(),
        version: None,
        path: std::env::current_exe().unwrap_or_default(),
        sha256: String::new(),
    };
    draft.result.tool = Some(tool.clone());
    draft.result.staging = Some(dest.clone());
    emit(Event::Started {
        dry_run: req.dry_run,
        resume: req.resume,
        instance: req.instance.clone(),
        profile: ready.profile.clone(),
        tool,
    });

    let mpi = match mpi_for(req, &ready.package, &work, emit) {
        Ok(mpi) => mpi,
        Err(why) => {
            if !req.resume {
                let _ = clear(&work);
                draft.result.staging = None;
            }
            return draft.fail(Status::Refused, TtwErrorCode::MissingInput, why);
        }
    };
    let job = ttwright_core::api::InstallJob {
        mpi_path: mpi,
        fallout3_path: Some(ready.fallout3.clone()),
        falloutnv_path: Some(ready.newvegas.clone()),
        destination_path: dest.clone(),
        dry_run: req.dry_run,
        resume: req.resume,
        ..ttwright_core::api::InstallJob::default()
    };
    let done = super::inproc::install_with_events(&job, req.timeout, emit);
    let journal = dest.join(".ttwright-journal.json").is_file();
    let (exit_code, doc) = (done.exit_code, done.doc);
    if let Some(tool) = &mut draft.result.tool {
        tool.version = doc["tool"]["version"].as_str().map(str::to_owned);
    }
    draft.result.ttwright = Some(doc.clone());
    let ttwright_code = doc["error"]["code"].as_str().map(str::to_owned);
    let ttwright_message = doc["error"]["message"]
        .as_str()
        .unwrap_or("no reason given")
        .to_owned();
    let because = |what: &str| match &ttwright_code {
        Some(code) => format!("{what} ({code}): {ttwright_message}"),
        None => format!("{what}: {ttwright_message}"),
    };
    match exit_code {
        0 => {}
        1 | 2 => {
            draft.result.resumable = journal;
            return draft.fail_ttwright(
                Status::Failed,
                TtwErrorCode::TtwrightFailed,
                ttwright_code.clone(),
                because("ttwright failed"),
            );
        }
        3 => {
            let _ = clear(&work);
            draft.result.staging = None;
            return draft.fail_ttwright(
                Status::Unsupported,
                TtwErrorCode::Unsupported,
                ttwright_code.clone(),
                because("ttwright does not support this TTW package; nothing was written"),
            );
        }
        4 => {
            draft.result.resumable = true;
            return draft.fail_ttwright(
                Status::Cancelled,
                TtwErrorCode::Cancelled,
                ttwright_code.clone(),
                "cancelled; `raul ttw --resume` continues",
            );
        }
        other => {
            draft.result.resumable = journal;
            return draft.fail(
                Status::Failed,
                TtwErrorCode::ToolFailed,
                format!("ttwright ended with exit code {other}, which it does not document"),
            );
        }
    }
    if doc["status"] != "success" {
        draft.result.resumable = journal;
        return draft.fail(
            Status::Failed,
            TtwErrorCode::ToolFailed,
            format!("ttwright exited 0 but its result says {}", doc["status"]),
        );
    }
    // A stop asked for while ttwright was finishing: its work is kept and nothing is added.
    if nvmod_core::cancel::requested() {
        draft.result.resumable = true;
        return draft.fail(
            Status::Cancelled,
            TtwErrorCode::Cancelled,
            "cancelled before the Mod was added; `raul ttw --resume` continues",
        );
    }
    if req.dry_run {
        let _ = clear(&work);
        draft.result.staging = None;
        return Outcome {
            exit_code: 0,
            result: draft.result,
            changed: false,
        };
    }

    emit(Event::Phase {
        phase: "add_mod".to_owned(),
    });
    let files = count_files(&dest);
    if files == 0 {
        let _ = clear(&work);
        draft.result.staging = None;
        return draft.fail(
            Status::Failed,
            TtwErrorCode::ToolFailed,
            "ttwright reported success but wrote no files",
        );
    }
    let request = Request {
        instance: &ready.instance,
        profile: &ready.profile,
        archive: &dest,
        name: Some(MOD_NAME),
        dry_run: false,
        answers: None,
        limits: Limits::default(),
    };
    let why = match nvmod_modinstall::install(&request) {
        Ok(report) => {
            let _ = clear(&work);
            draft.result.staging = None;
            draft.result.mod_name = Some(report.r#mod);
            draft.result.plugins_enabled = report.plugins;
            draft.result.files = Some(report.files.len() as u64);
            return Outcome {
                exit_code: 0,
                result: draft.result,
                changed: true,
            };
        }
        Err(InstallError::Refused(why) | InstallError::Failed(why)) => why,
        Err(InstallError::Questions(_)) => "the output holds a FOMOD that needs answers".to_owned(),
    };
    if nvmod_core::cancel::requested() {
        return draft.fail(
            Status::Cancelled,
            TtwErrorCode::Cancelled,
            format!("cancelled while adding the Mod; nothing was added ({why})"),
        );
    }
    draft.fail(
        Status::Failed,
        TtwErrorCode::AddModFailed,
        format!("adding the Mod failed and was rolled back: {why}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_its_exit_code() {
        for (status, code) in [
            (Status::Success, 0),
            (Status::Refused, 2),
            (Status::Unsupported, 2),
            (Status::Unverifiable, 3),
            (Status::Failed, 4),
            (Status::Cancelled, 4),
        ] {
            assert_eq!(exit_code(status), code, "{status:?}");
        }
    }
}

#[cfg(all(test, unix))]
mod unix_tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    #[test]
    fn clearing_a_linked_work_folder_removes_the_link_and_keeps_its_target() {
        let dir = TempDir::new("raul-clear");
        write_file(dir.path(), "target/keep", b"x");
        let link = dir.path().join("work");
        std::os::unix::fs::symlink(dir.path().join("target"), &link).unwrap();
        assert!(is_link(&link));
        clear(&link).unwrap();
        assert!(!link.exists() && dir.path().join("target/keep").is_file());
        clear(&link).unwrap();
    }

    #[test]
    fn a_loop_of_links_is_counted_without_following_it() {
        let dir = TempDir::new("raul-count");
        write_file(dir.path(), "a/file", b"x");
        std::os::unix::fs::symlink(dir.path().join("a"), dir.path().join("a/loop")).unwrap();
        assert_eq!(count_files(dir.path()), 2);
    }
}
