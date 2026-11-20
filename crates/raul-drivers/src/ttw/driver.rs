// SPDX-License-Identifier: GPL-3.0-only
//! The TTW driver: checks the inputs, runs ttwright's install in raul's own process (ttwright starts no program), maps
//! its progress to [`Event`]s while it runs, reads the `ttwright-result` it returns, and adds the output to the
//! Instance as one Mod with nvmod's mod-install library. Every outcome but success leaves the Instance as it was.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::{InstallError, Limits, OnConflict, Request};

use super::package::{self, Package};
use super::{CONTRACT, MOD_NAME, SCHEMA_VERSION, TtwError, TtwErrorCode, TtwResult};
use crate::Stop;
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
                    format!(
                        ", and the install about {} more",
                        package::size_text(install)
                    )
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

/// How many of ttwright's messages (one per failed asset) the text of a failure shows; the file holds all of them.
const SHOWN_MESSAGES: usize = 10;

/// The file in raul's state folder, next to the run logs, that holds the details of the last TTW failure.
pub const FAILURE_LOG: &str = "ttw-last-failure.txt";

/// `headline` followed by the reason of each failed asset ttwright named (the first few, and how many more there are),
/// and the file that holds them all. The reasons are only in ttwright's `result`, which a plain run does not print.
fn with_details(headline: &str, messages: &serde_json::Value, logs: &Path) -> String {
    let messages: Vec<&str> = messages
        .as_array()
        .map_or_default(|all| all.iter().filter_map(|m| m.as_str()).collect());
    if messages.is_empty() {
        return headline.to_owned();
    }
    let mut text = headline.to_owned();
    for message in messages.iter().take(SHOWN_MESSAGES) {
        text.push_str("\n  ");
        text.push_str(message);
    }
    if messages.len() > SHOWN_MESSAGES {
        let _ = write!(text, "\n  and {} more", messages.len() - SHOWN_MESSAGES);
    }
    if let Some(file) = write_failure_log(logs, headline, &messages) {
        let _ = write!(text, "\nAll of them are in {}", file.display());
    }
    text
}

/// Write the failure to [`FAILURE_LOG`] in the logs folder of the state folder; `None` when it cannot be written (a
/// log that cannot be written is not a reason to fail the command).
fn write_failure_log(folder: &Path, headline: &str, messages: &[&str]) -> Option<PathBuf> {
    std::fs::create_dir_all(folder).ok()?;
    let file = folder.join(FAILURE_LOG);
    let mut text = format!("{headline}\n");
    for message in messages {
        text.push_str(message);
        text.push('\n');
    }
    std::fs::write(&file, text).ok()?;
    Some(file)
}

/// Install TTW as `req` says, emitting [`Event`]s while it runs (not `finished`: the caller emits
/// [`Outcome::finished`] after the `result`). A dry run leaves nothing in the Instance, not even its work folder
/// (unless it continues a run that was stopped, whose work folder is that run's).
pub fn install(req: &TtwRequest, emit: &mut dyn FnMut(Event)) -> Outcome {
    install_with_stop(req, Stop::Process, emit)
}

/// [`install`] that stops when `stop` says so and on the timeout, and not on the process-wide request: the run a
/// window starts, whose stop belongs to that run alone.
pub fn install_with_stop(req: &TtwRequest, stop: Stop<'_>, emit: &mut dyn FnMut(Event)) -> Outcome {
    let mut outcome = run(req, stop, emit);
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
fn run(req: &TtwRequest, stop: Stop<'_>, emit: &mut dyn FnMut(Event)) -> Outcome {
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
    let done = super::inproc::install_with_events(&job, stop, req.timeout, emit);
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
            let headline = because("ttwright failed");
            let message = with_details(
                &headline,
                &doc["messages"],
                &nvmod_runlog::logs_dir(&nvmod_runlog::state_dir()),
            );
            return draft.fail_ttwright(
                Status::Failed,
                TtwErrorCode::TtwrightFailed,
                ttwright_code.clone(),
                message,
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
    if stop.requested() {
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
        answers_inline: None,
        limits: Limits::default(),
        on_conflict: OnConflict::default(),
    };
    let why = match nvmod_modinstall::install(&request) {
        Ok(report) => {
            let _ = clear(&work);
            draft.result.staging = None;
            draft.result.mod_name = Some(report.r#mod);
            // The install enables the new plugins by name, so a master can follow the plugin that needs it: sort
            // right away so the load order is valid when the player first looks. A sort that cannot run leaves the
            // Mod in place and says so.
            draft.result.plugins_enabled = match sort_new_plugins(
                &ready.instance,
                &ready.profile,
                &report.plugins,
            ) {
                Ok(order) => order,
                Err(why) => {
                    emit(Event::Warning {
                        message: format!(
                            "the load order was not sorted after the install ({why}); run `raul sort`"
                        ),
                    });
                    report.plugins
                }
            };
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
    if stop.requested() {
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

/// Sort the load order of `profile` with loot-rs and no masterlist (masters before the plugins that need them,
/// which is all the sort a fresh install needs and works offline), and write it. Returns `enabled` in the order the
/// profile now has.
///
/// # Errors
/// The reason, when the sort or the write failed; the lists are then as the install left them.
fn sort_new_plugins(
    instance: &NvmodInstance,
    profile: &str,
    enabled: &[String],
) -> Result<Vec<String>, String> {
    let describe = |e: nvmod_loadorder::Error| match e {
        nvmod_loadorder::Error::Refused(why) | nvmod_loadorder::Error::Failed(why) => why,
    };
    let proposal =
        nvmod_loadorder::propose(instance, profile, &nvmod_loadorder::empty_masterlist())
            .map_err(describe)?;
    if proposal.reordered() {
        nvmod_loadorder::apply(instance, &proposal).map_err(describe)?;
    }
    let mut ordered: Vec<String> = proposal
        .proposed
        .iter()
        .filter_map(|slot| {
            enabled
                .iter()
                .find(|p| p.eq_ignore_ascii_case(&slot.name))
                .cloned()
        })
        .collect();
    // Whatever the proposal did not list keeps its place at the end.
    let rest: Vec<String> = enabled
        .iter()
        .filter(|p| !ordered.iter().any(|o| o.eq_ignore_ascii_case(p)))
        .cloned()
        .collect();
    ordered.extend(rest);
    Ok(ordered)
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

#[cfg(test)]
mod details_tests {
    use super::*;
    use nvmod_testkit::TempDir;

    #[test]
    fn a_failure_without_messages_is_its_headline_and_many_messages_are_counted_and_kept() {
        let dir = TempDir::new("raul-details");
        let logs = dir.path().join("logs");
        assert_eq!(
            with_details("ttwright failed", &serde_json::json!([]), &logs),
            "ttwright failed"
        );
        assert_eq!(
            with_details("ttwright failed", &serde_json::Value::Null, &logs),
            "ttwright failed"
        );
        assert!(!logs.exists(), "nothing to keep, nothing written");
        let many: Vec<String> = (0..14)
            .map(|i| format!("asset{i}.dll (op=1): denied"))
            .collect();
        let text = with_details("ttwright failed", &serde_json::json!(many), &logs);
        assert!(
            text.starts_with("ttwright failed\n  asset0.dll (op=1): denied\n"),
            "{text}"
        );
        assert!(
            text.contains("asset9.dll") && !text.contains("asset10.dll"),
            "{text}"
        );
        assert!(text.contains("\n  and 4 more"), "{text}");
        let file = logs.join(FAILURE_LOG);
        assert!(text.contains(&file.display().to_string()), "{text}");
        let kept = std::fs::read_to_string(&file).unwrap();
        assert!(kept.contains("asset13.dll") && kept.starts_with("ttwright failed\n"));
    }

    #[test]
    fn a_log_that_cannot_be_written_does_not_change_the_text() {
        let dir = TempDir::new("raul-details-blocked");
        let blocker = dir.path().join("logs");
        std::fs::write(&blocker, b"a file where the folder should be").unwrap();
        let text = with_details("head", &serde_json::json!(["a.dll: denied"]), &blocker);
        assert_eq!(text, "head\n  a.dll: denied");
    }
}

#[cfg(test)]
#[allow(clippy::unnecessary_wraps)] // the stand-ins for `FreeSpace`, whose type returns an Option
mod unpack_tests {
    use super::*;
    use nvmod_testkit::TempDir;

    fn request(free_space: Option<FreeSpace>, dry_run: bool, resume: bool) -> TtwRequest {
        TtwRequest {
            instance: PathBuf::new(),
            profile: None,
            fallout3: None,
            newvegas: None,
            package: None,
            dry_run,
            resume,
            timeout: DEFAULT_TIMEOUT,
            free_space,
        }
    }

    /// A download made small: the package (`BSA\0` and made-up bytes) next to a made-up installer program.
    fn download(dir: &TempDir) -> Package {
        let path = dir.path().join("Tale of Two Wastelands 3.4.7z");
        let mut writer = sevenz_rust2::ArchiveWriter::create(&path).unwrap();
        for (name, data) in [
            (
                "Tale of Two Wastelands 3.4.mpi",
                &b"BSA\0 made-up package"[..],
            ),
            ("installer.exe", &b"MZ made-up"[..]),
        ] {
            writer
                .push_archive_entry(sevenz_rust2::ArchiveEntry::new_file(name), Some(data))
                .unwrap();
        }
        writer.finish().unwrap();
        package::inspect(&path).unwrap()
    }

    fn phases(events: &[Event]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Phase { phase } => Some(phase.as_str()),
                _ => None,
            })
            .collect()
    }

    fn little(_: &Path) -> Option<u64> {
        Some(1_000)
    }
    fn some(_: &Path) -> Option<u64> {
        Some(10 * 1024 * 1024 * 1024)
    }
    fn plenty(_: &Path) -> Option<u64> {
        Some(u64::MAX / 2)
    }
    fn unknown(_: &Path) -> Option<u64> {
        None
    }

    #[test]
    fn too_little_space_refuses_before_anything_is_unpacked_and_says_how_much_is_missing() {
        let dir = TempDir::new("raul-unpack-space");
        let package = download(&dir);
        let work = dir.path().join("work");
        let mut events = Vec::new();
        let why = mpi_for(
            &request(Some(little), false, false),
            &package,
            &work,
            &mut |e| {
                events.push(e);
            },
        )
        .unwrap_err();
        assert!(why.contains("not enough free space"), "{why}");
        assert!(why.contains("1 MB is free"), "{why}");
        assert!(why.contains("43.0 GB") || why.contains("42."), "{why}");
        assert!(events.is_empty(), "nothing was unpacked: {events:?}");
        assert!(!work.exists());
    }

    #[test]
    fn the_install_need_counts_on_top_of_the_package_but_a_dry_run_only_needs_the_package() {
        let dir = TempDir::new("raul-unpack-dry");
        let package = download(&dir);
        let work = dir.path().join("work");
        let real = mpi_for(
            &request(Some(some), false, false),
            &package,
            &work,
            &mut |_| {},
        );
        assert!(real.is_err(), "10 GiB is less than the install needs");
        let mut events = Vec::new();
        let dry = mpi_for(
            &request(Some(some), true, false),
            &package,
            &work,
            &mut |e| {
                events.push(e);
            },
        )
        .unwrap();
        assert_eq!(dry, work.join("package/Tale of Two Wastelands 3.4.mpi"));
        assert_eq!(phases(&events), ["unpack"]);
        assert!(package::is_mpi(&dry));
    }

    #[test]
    fn an_unknown_amount_of_space_does_not_block_and_a_resume_keeps_what_was_unpacked() {
        let dir = TempDir::new("raul-unpack-resume");
        let package = download(&dir);
        let work = dir.path().join("work");
        let first = mpi_for(
            &request(Some(unknown), false, false),
            &package,
            &work,
            &mut |_| {},
        )
        .unwrap();
        let mut events = Vec::new();
        // Even with no room left, a resumed run uses the package it already has.
        let again = mpi_for(
            &request(Some(little), false, true),
            &package,
            &work,
            &mut |e| {
                events.push(e);
            },
        )
        .unwrap();
        assert_eq!(first, again);
        assert!(events.is_empty(), "no second unpack: {events:?}");
        // Without --resume it is unpacked again, and a plain package is passed through as it is.
        let fresh = mpi_for(
            &request(Some(plenty), false, false),
            &package,
            &work,
            &mut |e| {
                events.push(e);
            },
        )
        .unwrap();
        assert_eq!(phases(&events), ["unpack"]);
        assert_eq!(fresh, first);
        let mpi = Package::Mpi {
            path: dir.path().join("x.mpi"),
        };
        assert_eq!(
            mpi_for(
                &request(Some(little), false, false),
                &mpi,
                &work,
                &mut |_| {}
            ),
            Ok(dir.path().join("x.mpi"))
        );
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

#[cfg(test)]
mod sort_tests {
    use super::*;
    use nvmod_testkit::{BaseBuilder, PluginBuilder, TempDir, ZipBuilder};

    /// A profile with the base game's master, and a TTW-like archive whose plugins sort wrongly by name.
    fn installed() -> (TempDir, NvmodInstance, Vec<String>) {
        let dir = TempDir::new("raul-ttw-sort");
        let game = dir.path().join("game");
        BaseBuilder::steam().write(&game);
        std::fs::write(
            game.join("Data/FalloutNV.esm"),
            PluginBuilder::master().build(),
        )
        .unwrap();
        let instance = NvmodInstance::create(&dir.path().join("inst"), &game).expect("instance");
        instance.create_profile("Default").expect("profile");
        let archive = dir.path().join("ttw.zip");
        ZipBuilder::new()
            .file(
                "Aaa.esm",
                &PluginBuilder::master().requires("Zzz.esm").build(),
            )
            .file("Zzz.esm", &PluginBuilder::master().build())
            .file("Mid.esp", &PluginBuilder::new().requires("Aaa.esm").build())
            .write(&archive);
        let report = nvmod_modinstall::install(&Request {
            instance: &instance,
            profile: "Default",
            archive: &archive,
            name: Some("TTW"),
            dry_run: false,
            answers: None,
            answers_inline: None,
            limits: Limits::default(),
            on_conflict: OnConflict::default(),
        })
        .expect("install");
        (dir, instance, report.plugins)
    }

    #[test]
    fn the_new_plugins_are_in_a_valid_order_after_the_sort() {
        let (_dir, instance, enabled) = installed();
        let before = nvmod_loadorder::read_slots(&instance.profile_dir("Default")).unwrap();
        let pos = |slots: &[nvmod_loadorder::Slot], n: &str| {
            slots.iter().position(|s| s.name == n).unwrap()
        };
        assert!(
            pos(&before, "Aaa.esm") < pos(&before, "Zzz.esm"),
            "the install alone enables the master after its dependant"
        );
        let order = sort_new_plugins(&instance, "Default", &enabled).unwrap();
        let after = nvmod_loadorder::read_slots(&instance.profile_dir("Default")).unwrap();
        assert!(pos(&after, "Zzz.esm") < pos(&after, "Aaa.esm"));
        assert!(pos(&after, "Aaa.esm") < pos(&after, "Mid.esp"));
        assert_eq!(order, ["Zzz.esm", "Aaa.esm", "Mid.esp"]);
    }

    #[test]
    fn a_sort_that_cannot_run_is_reported_and_leaves_the_lists_alone() {
        let (_dir, instance, enabled) = installed();
        let err = sort_new_plugins(&instance, "Missing", &enabled).unwrap_err();
        assert!(err.contains("Missing"), "{err}");
    }
}
