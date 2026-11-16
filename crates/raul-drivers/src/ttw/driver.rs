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
    program: PathBuf,
    sha256: String,
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
        Some(p) if p.is_file() => p.clone(),
        Some(p) => {
            return Err((
                ToolMissing,
                format!("the ttwright program {} is not a file", p.display()),
            ));
        }
        None => {
            return Err((
                ToolMissing,
                "no ttwright program: none is bundled next to raul; pass --ttwright <path>".into(),
            ));
        }
    };
    let bytes = std::fs::read(&program)
        .map_err(|e| (ToolMissing, format!("{}: {e}", program.display())))?;
    Ok(Ready {
        instance,
        profile: profile.clone(),
        fallout3,
        newvegas,
        package,
        program,
        sha256: sha256_hex(&bytes),
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

/// Reads the lines ttwright appends to its event file and turns each into an [`Event`].
struct EventTail {
    file: PathBuf,
    offset: u64,
    partial: String,
    warned: bool,
}

impl EventTail {
    fn new(file: PathBuf) -> EventTail {
        EventTail {
            file,
            offset: 0,
            partial: String::new(),
            warned: false,
        }
    }

    /// Emit every complete line written since the last call; with `last`, also a final line without a newline.
    fn drain(&mut self, emit: &mut dyn FnMut(Event), last: bool) {
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
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return;
        };
        if v["schema_version"].as_i64() != Some(TTWRIGHT_EVENTS_VERSION) {
            if !self.warned {
                self.warned = true;
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
}

/// Run ttwright through `runner`, emitting its events while it runs.
fn run_streaming(
    runner: &Runner,
    call_program: &Path,
    args: &[String],
    sha256: &str,
    events: &Path,
    emit: &mut dyn FnMut(Event),
) -> Result<Output, ToolError> {
    let (tx, rx) = mpsc::channel();
    let mut tail = EventTail::new(events.to_path_buf());
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let mut call = Call::new(TTWRIGHT, call_program, args);
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
                        tool: TTWRIGHT.to_owned(),
                        reason: "the run ended without an answer".to_owned(),
                    });
                }
            }
        }
    })
}

fn count_files(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| {
            let path = e.path();
            if path.is_dir() { count_files(&path) } else { 1 }
        })
        .sum()
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
    if !req.resume {
        let _ = std::fs::remove_dir_all(&work);
    }
    let _ = std::fs::remove_file(&events);
    if let Err(e) = std::fs::create_dir_all(&dest) {
        return draft.fail(
            Status::Failed,
            TtwErrorCode::ToolFailed,
            format!("{}: {e}", dest.display()),
        );
    }
    let tool = ToolInfo {
        name: TTWRIGHT.to_owned(),
        version: None,
        path: ready.program.clone(),
        sha256: ready.sha256.clone(),
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

    let runner = Runner {
        timeout: req.timeout,
        max_output: 16 * 1024 * 1024,
        ..Runner::default()
    };
    let args = arguments(req, &ready, &dest, &events);
    let output = run_streaming(&runner, &ready.program, &args, &ready.sha256, &events, emit);
    let journal = dest.join(".ttwright-journal.json").is_file();
    let output = match output {
        Ok(o) => o,
        Err(ToolError::Unverifiable { reason, .. }) => {
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
    match output.exit_code {
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
            let _ = std::fs::remove_dir_all(&work);
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
    if req.dry_run {
        let _ = std::fs::remove_dir_all(&work);
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
            let _ = std::fs::remove_dir_all(&work);
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
