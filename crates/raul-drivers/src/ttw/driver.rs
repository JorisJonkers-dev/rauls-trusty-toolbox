// SPDX-License-Identifier: GPL-3.0-only
//! The TTW driver: checks the inputs, runs `ttwright install --json` through `raul-runner`, streams ttwright's event
//! lines as [`Event`]s while it runs, reads its `ttwright-result`, and adds the output to the Instance as one Mod
//! with nvmod's mod-install library. Every outcome but success leaves the Instance as it was.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::{InstallError, Limits, Request};
use raul_runner::{Call, Output, Runner, ToolError, sha256_hex};
use serde_json::Value;

use super::{CONTRACT, MOD_NAME, SCHEMA_VERSION, TtwError, TtwErrorCode, TtwResult};
use crate::events::{Event, Status, ToolInfo};

/// The `ttwright-result` versions this build reads (the newest it knows, and the one before it when there is one).
pub const TTWRIGHT_RESULT_VERSIONS: &[i64] = &[1];

/// The `ttwright-events` version this build reads.
pub const TTWRIGHT_EVENTS_VERSION: i64 = 1;

/// The ttwright program's name, without the `.exe` Windows adds.
pub const TTWRIGHT: &str = "ttwright";

/// How long an install may run: the slowest machines reported take about 40 minutes; three hours is a stuck run.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_hours(3);

/// The folder inside the Instance where ttwright writes, kept between a cancelled run and `--resume`.
pub const WORK_DIR: &str = ".raul/ttw";

/// How often the event file is read while ttwright runs.
const POLL: Duration = Duration::from_millis(100);

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
    /// The TTW package (the `.mpi` file).
    pub package: Option<PathBuf>,
    /// The ttwright program; `None` when none was found.
    pub ttwright: Option<PathBuf>,
    /// Read and check everything, write nothing.
    pub dry_run: bool,
    /// Continue a cancelled or interrupted install.
    pub resume: bool,
    /// How long ttwright may run.
    pub timeout: Duration,
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
    package: PathBuf,
    /// The ttwright program and its SHA-256 when one was given; `None` runs ttwright in this process.
    program: Option<(PathBuf, String)>,
}

fn missing(what: &str, flag: &str, key: &str) -> String {
    format!("no {what}: pass {flag} or run `raul config set {key} <path>`")
}

/// Check the inputs; a refusal is its error code and reason. `profile` receives the Profile once it is known.
fn prepare(req: &TtwRequest, profile: &mut String) -> Result<Ready, (TtwErrorCode, String)> {
    use TtwErrorCode::{InstanceInvalid, MissingInput, ModExists, ProfileMissing, ToolMissing};
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
    if !package.is_file() {
        return Err((
            MissingInput,
            format!("the TTW package {} is not a file", package.display()),
        ));
    }
    if !req.dry_run && find_ci(&instance.root.join("mods"), MOD_NAME).is_some() {
        return Err((
            ModExists,
            format!("the Instance already has a Mod `{MOD_NAME}`; raul does not overwrite it"),
        ));
    }
    let program = match &req.ttwright {
        Some(p) if p.is_file() => {
            let bytes =
                std::fs::read(p).map_err(|e| (ToolMissing, format!("{}: {e}", p.display())))?;
            Some((p.clone(), sha256_hex(&bytes)))
        }
        Some(p) => {
            return Err((
                ToolMissing,
                format!("the ttwright program {} is not a file", p.display()),
            ));
        }
        None => None,
    };
    Ok(Ready {
        instance,
        profile: profile.clone(),
        fallout3,
        newvegas,
        package,
        program,
    })
}

/// The arguments of `ttwright install`.
fn arguments(req: &TtwRequest, ready: &Ready, dest: &Path, events: &Path) -> Vec<String> {
    let mut words: Vec<String> = ["install", "--json", "--no-detect"]
        .map(str::to_owned)
        .to_vec();
    for (flag, path) in [
        ("--mpi", ready.package.as_path()),
        ("--fo3", ready.fallout3.as_path()),
        ("--fnv", ready.newvegas.as_path()),
        ("--dest", dest),
        ("--events", events),
    ] {
        words.push(flag.to_owned());
        words.push(path.display().to_string());
    }
    if req.resume {
        words.push("--resume".to_owned());
    }
    if req.dry_run {
        words.push("--dry-run".to_owned());
    }
    words
}

/// How one event line of a Component becomes [`Event`]s: the parsed line, whether a warning about it was given
/// already, and where the events go.
pub(crate) type MapLine = fn(&Value, &mut bool, &mut dyn FnMut(Event));

/// Reads the lines a Component appends to its event file and turns each into [`Event`]s.
pub(crate) struct EventTail {
    file: PathBuf,
    offset: u64,
    partial: String,
    warned: bool,
    map: MapLine,
}

impl EventTail {
    pub(crate) fn new(file: PathBuf, map: MapLine) -> EventTail {
        EventTail {
            file,
            offset: 0,
            partial: String::new(),
            warned: false,
            map,
        }
    }

    /// Emit every complete line written since the last call; with `last`, also a final line without a newline.
    pub(crate) fn drain(&mut self, emit: &mut dyn FnMut(Event), last: bool) {
        if let Ok(mut f) = std::fs::File::open(&self.file)
            && f.seek(SeekFrom::Start(self.offset)).is_ok()
        {
            let mut bytes = Vec::new();
            if let Ok(n) = f.read_to_end(&mut bytes) {
                self.offset += n as u64;
                self.partial.push_str(&String::from_utf8_lossy(&bytes));
            }
        }
        while let Some(end) = self.partial.find('\n') {
            let line: String = self.partial.drain(..=end).collect();
            self.line(line.trim(), emit);
        }
        if last && !self.partial.trim().is_empty() {
            let line = std::mem::take(&mut self.partial);
            self.line(line.trim(), emit);
        }
    }

    fn line(&mut self, line: &str, emit: &mut dyn FnMut(Event)) {
        if line.is_empty() {
            return;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            (self.map)(&v, &mut self.warned, emit);
        }
    }
}

/// One `ttwright-events` line as [`Event`]s.
fn ttwright_line(v: &Value, warned: &mut bool, emit: &mut dyn FnMut(Event)) {
    if v["schema_version"].as_i64() != Some(TTWRIGHT_EVENTS_VERSION) {
        if !*warned {
            *warned = true;
            emit(Event::Warning {
                message: format!(
                    "ttwright writes events of version {}; this raul reads {TTWRIGHT_EVENTS_VERSION}, so progress is not shown",
                    v["schema_version"]
                ),
            });
        }
        return;
    }
    let text = |k: &str| v[k].as_str().map(str::to_owned);
    match v["event"].as_str() {
        Some("phase") => {
            if let Some(phase) = text("phase") {
                emit(Event::Phase { phase });
            }
        }
        Some("progress") => emit(Event::Progress {
            phase: text("phase").unwrap_or_default(),
            done: v["done"].as_u64().unwrap_or(0),
            of: v["of"].as_u64().unwrap_or(0),
            message: text("message"),
        }),
        Some("warning") => emit(Event::Warning {
            message: text("message").unwrap_or_default(),
        }),
        // `started` and `finished` are the driver's own to report.
        _ => {}
    }
}

/// Run the Component `tool` through `runner`, turning the lines of its event file into events while it runs.
pub(crate) fn run_streaming(
    runner: &Runner,
    tool: &str,
    call_program: &Path,
    args: &[String],
    sha256: &str,
    mut tail: EventTail,
    emit: &mut dyn FnMut(Event),
) -> Result<Output, ToolError> {
    let (tx, rx) = mpsc::channel();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let mut call = Call::new(tool, call_program, args);
            call.expected_sha256 = Some(sha256);
            let _ = tx.send(runner.run_any(&call));
        });
        loop {
            match rx.recv_timeout(POLL) {
                Ok(result) => {
                    tail.drain(emit, true);
                    return result;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => tail.drain(emit, false),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(ToolError::Failed {
                        tool: tool.to_owned(),
                        reason: "the run ended without an answer".to_owned(),
                    });
                }
            }
        }
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

/// The `ttwright-result` on standard output: refused when its version is newer than this build reads.
fn read_result(stdout: &[u8]) -> Result<Value, (Status, TtwErrorCode, String)> {
    let failed = |why: String| (Status::Failed, TtwErrorCode::ToolFailed, why);
    let text = String::from_utf8_lossy(stdout);
    let doc: Value = serde_json::from_str(text.trim()).map_err(|e| {
        failed(format!(
            "ttwright printed no ttwright-result document ({e})"
        ))
    })?;
    if doc["contract"] != "ttwright-result" {
        return Err(failed(format!(
            "ttwright printed a {} document, not a ttwright-result",
            doc["contract"]
        )));
    }
    let Some(version) = doc["schema_version"].as_i64() else {
        return Err(failed(
            "the ttwright-result has no schema_version".to_owned(),
        ));
    };
    let newest = TTWRIGHT_RESULT_VERSIONS.iter().max().copied().unwrap_or(1);
    if version > newest {
        return Err((
            Status::Refused,
            TtwErrorCode::ContractRefused,
            format!(
                "ttwright printed ttwright-result version {version}; this raul reads version {newest}. Use the ttwright bundled with this raul, or update raul"
            ),
        ));
    }
    if !TTWRIGHT_RESULT_VERSIONS.contains(&version) {
        return Err(failed(format!(
            "ttwright printed ttwright-result version {version}, which this raul does not read"
        )));
    }
    Ok(doc)
}

/// Install TTW as `req` says, emitting [`Event`]s while it runs (not `finished`: the caller emits
/// [`Outcome::finished`] after the `result`).
#[allow(clippy::too_many_lines)] // one flow: check, run, read, add
pub fn install(req: &TtwRequest, emit: &mut dyn FnMut(Event)) -> Outcome {
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
    let tool = match &ready.program {
        Some((path, sha256)) => ToolInfo {
            name: TTWRIGHT.to_owned(),
            version: None,
            path: path.clone(),
            sha256: sha256.clone(),
        },
        // Linked into raul: there is no file to name, so the running executable stands for it.
        None => ToolInfo {
            name: TTWRIGHT.to_owned(),
            version: None,
            path: std::env::current_exe().unwrap_or_default(),
            sha256: String::new(),
        },
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

    let (exit_code, doc, journal) = if let Some((program, sha256)) = &ready.program {
        let runner = Runner {
            timeout: req.timeout,
            max_output: 16 * 1024 * 1024,
            ..Runner::default()
        };
        let args = arguments(req, &ready, &dest, &events);
        let output = run_streaming(
            &runner,
            TTWRIGHT,
            program,
            &args,
            sha256,
            EventTail::new(events.clone(), ttwright_line),
            emit,
        );
        let journal = dest.join(".ttwright-journal.json").is_file();
        let output = match output {
            Ok(o) => o,
            Err(ToolError::Unverifiable { reason, .. }) => {
                let _ = clear(&work);
                draft.result.staging = None;
                return draft.fail(Status::Unverifiable, TtwErrorCode::ToolUnverifiable, reason);
            }
            Err(e) if nvmod_core::cancel::requested() => {
                draft.result.resumable = true;
                return draft.fail(
                    Status::Cancelled,
                    TtwErrorCode::Cancelled,
                    format!("cancelled ({e}); `raul ttw --resume` continues"),
                );
            }
            Err(e) => {
                draft.result.resumable = journal;
                return draft.fail(Status::Failed, TtwErrorCode::ToolFailed, e.to_string());
            }
        };
        let doc = match read_result(&output.stdout) {
            Ok(doc) => doc,
            Err((status, code, message)) => {
                draft.result.resumable = journal && status == Status::Failed;
                return draft.fail(status, code, message);
            }
        };
        (output.exit_code, doc, journal)
    } else {
        let job = ttwright_core::api::InstallJob {
            mpi_path: ready.package.clone(),
            fallout3_path: Some(ready.fallout3.clone()),
            falloutnv_path: Some(ready.newvegas.clone()),
            destination_path: dest.clone(),
            dry_run: req.dry_run,
            resume: req.resume,
            ..ttwright_core::api::InstallJob::default()
        };
        let done = super::inproc::install_with_events(&job, req.timeout, emit);
        let journal = dest.join(".ttwright-journal.json").is_file();
        (done.exit_code, done.doc, journal)
    };
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
        extract_tool: None,
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

/// Where a bundled Component program is, given the folder `raul` runs from: `components/` next to it (the
/// portable archive and the Windows installer), `../Resources/components/` (the macOS app), or
/// `../lib/<package>/components/` (the Linux packages).
#[must_use]
pub fn bundled(exe_dir: &Path, name: &str) -> Option<PathBuf> {
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let mut candidates = vec![
        exe_dir.join("components").join(&file),
        exe_dir.join("../Resources/components").join(&file),
    ];
    if let Ok(entries) = std::fs::read_dir(exe_dir.join("../lib")) {
        let mut libs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        libs.sort();
        candidates.extend(libs.into_iter().map(|l| l.join("components").join(&file)));
    }
    candidates.into_iter().find(|c| c.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    #[test]
    fn a_bundled_program_is_found_in_each_package_layout() {
        let file = format!("{TTWRIGHT}{}", std::env::consts::EXE_SUFFIX);
        for layout in [
            "app/components",
            "Contents/Resources/components",
            "usr/lib/raul/components",
        ] {
            let root = TempDir::new("raul-bundled");
            write_file(root.path(), &format!("{layout}/{file}"), b"x");
            let exe_dir = match layout {
                "app/components" => root.path().join("app"),
                "Contents/Resources/components" => root.path().join("Contents/MacOS"),
                _ => root.path().join("usr/bin"),
            };
            std::fs::create_dir_all(&exe_dir).unwrap();
            let found = bundled(&exe_dir, TTWRIGHT).expect(layout);
            assert!(found.ends_with(format!("components/{file}")), "{layout}");
        }
        let empty = TempDir::new("raul-bundled-none");
        assert_eq!(bundled(empty.path(), TTWRIGHT), None);
    }

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

    #[test]
    fn the_event_tail_reads_whole_lines_and_warns_once_about_a_newer_version() {
        let dir = TempDir::new("raul-tail");
        let file = dir.path().join("events.ndjson");
        let mut tail = EventTail::new(file.clone(), ttwright_line);
        let mut seen = Vec::new();
        tail.drain(&mut |e| seen.push(e), false);
        assert!(seen.is_empty(), "no file yet");
        std::fs::write(
            &file,
            concat!(
                "{\"schema_version\":1,\"event\":\"phase\",\"phase\":\"assets\"}\n",
                "not json\n",
                "{\"schema_version\":2,\"event\":\"phase\",\"phase\":\"x\"}\n",
                "{\"schema_version\":3,\"event\":\"phase\",\"phase\":\"y\"}\n",
                "{\"schema_version\":1,\"event\":\"progress\",\"phase\":\"assets\",\"done\":1,\"of\":2}\n",
                "{\"schema_version\":1,\"event\":\"warning\",\"message\":\"w\"}\n",
                "{\"schema_version\":1,\"event\":\"phase\",\"pha",
            ),
        )
        .unwrap();
        tail.drain(&mut |e| seen.push(e), false);
        assert_eq!(seen.len(), 4, "{seen:?}");
        assert!(matches!(&seen[0], Event::Phase { phase } if phase == "assets"));
        assert!(matches!(&seen[1], Event::Warning { message } if message.contains("version 2")));
        assert!(matches!(&seen[2], Event::Progress { done: 1, of: 2, .. }));
        assert!(matches!(&seen[3], Event::Warning { message } if message == "w"));
        // The unfinished line is completed by a later write.
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&file)
            .unwrap();
        std::io::Write::write_all(&mut f, b"se\":\"post_install\"}").unwrap();
        tail.drain(&mut |e| seen.push(e), true);
        assert!(matches!(&seen[4], Event::Phase { phase } if phase == "post_install"));
    }

    #[test]
    fn a_result_of_another_contract_or_without_a_version_is_a_tool_failure() {
        for doc in [
            "{\"contract\":\"lodwright-result\",\"schema_version\":1}",
            "{\"contract\":\"ttwright-result\"}",
            "{\"contract\":\"ttwright-result\",\"schema_version\":0}",
        ] {
            let (status, code, _) = read_result(doc.as_bytes()).unwrap_err();
            assert_eq!(
                (status, code),
                (Status::Failed, TtwErrorCode::ToolFailed),
                "{doc}"
            );
        }
        let (status, code, _) =
            read_result(b"{\"contract\":\"ttwright-result\",\"schema_version\":2}").unwrap_err();
        assert_eq!(
            (status, code),
            (Status::Refused, TtwErrorCode::ContractRefused)
        );
        assert!(read_result(b"{\"contract\":\"ttwright-result\",\"schema_version\":1}").is_ok());
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
