// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate`: run `lodwright generate --request <file> --json --events <file>` through `raul-runner`,
//! stream its block progress, read its `lodwright-result` and manifest, and keep the output as the Mod
//! `LOD (lodwright)`. A second run over an unchanged Load order is `up_to_date` and writes nothing. Every outcome but
//! success leaves the Instance as it was.

use std::path::{Path, PathBuf};
use std::time::Duration;

use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::lifecycle::{self, Action};
use nvmod_modinstall::{InstallError, Limits, Request};
use raul_runner::{Runner, ToolError, sha256_hex};
use serde_json::Value;

use super::{
    CONTRACT, LodError, LodErrorCode, LodResult, MANIFEST_FILE, MOD_NAME, SCHEMA_VERSION, WORK_DIR,
    plan, read_manifest,
};
use crate::events::{Event, Status, ToolInfo};
use crate::ttw::driver::{EventTail, clear, count_files, run_streaming};
use crate::ttw::exit_code;

/// The lodwright program's name, without the `.exe` Windows adds.
pub const LODWRIGHT: &str = "lodwright";

/// The `lodwright-result` versions this build reads.
pub const LODWRIGHT_RESULT_VERSIONS: &[i64] = &[1];

/// What `raul lod generate` was asked to do.
#[derive(Debug, Clone)]
pub struct LodRequest {
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile; the Instance's only or selected one when `None`.
    pub profile: Option<String>,
    /// Worlds that replace the Profile's settings; empty keeps them.
    pub worlds: Vec<String>,
    /// The lodwright program; `None` when none was found.
    pub lodwright: Option<PathBuf>,
    /// Build and check the request, run nothing, write nothing.
    pub dry_run: bool,
    /// How long lodwright may run.
    pub timeout: Duration,
}

/// The finished run.
#[derive(Debug, Clone, PartialEq)]
pub struct LodOutcome {
    /// The result.
    pub result: LodResult,
    /// The exit code of `raul lod generate`.
    pub exit_code: u8,
    /// Whether the Instance was changed.
    pub changed: bool,
}

impl LodOutcome {
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
            resumable: false,
            changed: self.changed,
        }
    }
}

fn result(req: &LodRequest) -> LodResult {
    LodResult {
        schema_version: SCHEMA_VERSION,
        contract: CONTRACT.to_owned(),
        status: Status::Success,
        up_to_date: false,
        dry_run: req.dry_run,
        instance: req.instance.clone(),
        profile: req.profile.clone().unwrap_or_default(),
        mod_name: None,
        files: None,
        worlds: Vec::new(),
        request: None,
        tool: None,
        lodwright: None,
        error: None,
    }
}

fn done(result: LodResult, changed: bool) -> LodOutcome {
    LodOutcome {
        exit_code: exit_code(result.status),
        result,
        changed,
    }
}

fn fail(
    mut result: LodResult,
    status: Status,
    code: LodErrorCode,
    lodwright_code: Option<String>,
    message: impl Into<String>,
) -> LodOutcome {
    result.status = status;
    result.error = Some(LodError {
        code,
        lodwright_code,
        message: message.into(),
    });
    done(result, false)
}

/// One lodwright event line (`{"event":"block","world":..,"level":..,"done":..,"of":..}`) as a `progress` event.
fn lodwright_line(v: &Value, _warned: &mut bool, emit: &mut dyn FnMut(Event)) {
    if v["event"] == "block" {
        emit(Event::Progress {
            phase: "terrain".to_owned(),
            done: v["done"].as_u64().unwrap_or(0),
            of: v["of"].as_u64().unwrap_or(0),
            message: Some(format!(
                "{} level {}",
                v["world"].as_str().unwrap_or("?"),
                v["level"]
            )),
        });
    }
}

/// The `lodwright-result` on standard output.
fn read_result(stdout: &[u8]) -> Result<Value, (Status, LodErrorCode, String)> {
    let failed = |why: String| (Status::Failed, LodErrorCode::ToolFailed, why);
    let doc: Value = serde_json::from_slice(stdout).map_err(|e| {
        failed(format!(
            "lodwright printed no lodwright-result document ({e})"
        ))
    })?;
    if doc["contract"] != "lodwright-result" {
        return Err(failed(format!(
            "lodwright printed a {} document, not a lodwright-result",
            doc["contract"]
        )));
    }
    let newest = LODWRIGHT_RESULT_VERSIONS.iter().max().copied().unwrap_or(1);
    match doc["schema_version"].as_i64() {
        Some(v) if LODWRIGHT_RESULT_VERSIONS.contains(&v) => Ok(doc),
        Some(v) if v > newest => Err((
            Status::Refused,
            LodErrorCode::ContractRefused,
            format!(
                "lodwright printed lodwright-result version {v}; this raul reads version {newest}. Use the lodwright bundled with this raul, or update raul"
            ),
        )),
        _ => Err(failed(
            "the lodwright-result has no schema_version this raul reads".to_owned(),
        )),
    }
}

/// Copy `from` into `to`, leaving out lodwright's manifest (it describes lodwright's folder, not the Mod).
fn copy_output(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let (source, name) = (entry.path(), entry.file_name());
        if from.join(MANIFEST_FILE) == source {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_output(&source, &to.join(&name))?;
        } else if kind.is_file() {
            std::fs::copy(&source, to.join(&name))?;
        }
    }
    Ok(())
}

/// Put the output into the Instance as [`MOD_NAME`]. An existing LOD Mod is renamed out of the way first, removed
/// once the new one is in, and renamed back when the new one cannot be added. Returns the Mod's file count, and a
/// warning when the old Mod could not be removed (the new one is in and active either way).
fn replace_mod(
    instance: &NvmodInstance,
    profile: &str,
    source: &Path,
) -> Result<(u64, Option<String>), String> {
    let previous = format!("{MOD_NAME} previous");
    let had = find_ci(&instance.root.join("mods"), MOD_NAME).is_some();
    let lifecycle_error = |e: nvmod_modinstall::mods::ModsError| match e {
        nvmod_modinstall::mods::ModsError::Refused(w)
        | nvmod_modinstall::mods::ModsError::Failed(w) => w,
    };
    if had {
        lifecycle::apply(
            instance,
            profile,
            MOD_NAME,
            &Action::Rename {
                to: previous.clone(),
            },
            false,
        )
        .map_err(lifecycle_error)?;
    }
    let request = Request {
        instance,
        profile,
        archive: source,
        name: Some(MOD_NAME),
        dry_run: false,
        answers: None,
        extract_tool: None,
        limits: Limits::default(),
    };
    match nvmod_modinstall::install(&request) {
        Ok(report) => {
            if had {
                let removed = lifecycle::apply(
                    instance,
                    profile,
                    &previous,
                    &Action::Remove {
                        force: true,
                        dependants: Vec::new(),
                    },
                    false,
                );
                if let Err(e) = removed {
                    let warning = format!(
                        "the new LOD Mod is in, but the old one, `{previous}`, could not be removed: {}; remove it with `raul mods remove`",
                        lifecycle_error(e)
                    );
                    return Ok((report.files.len() as u64, Some(warning)));
                }
            }
            Ok((report.files.len() as u64, None))
        }
        Err(e) => {
            let why = match e {
                InstallError::Refused(w) | InstallError::Failed(w) => w,
                InstallError::Questions(_) => {
                    "the output holds a FOMOD that needs answers".to_owned()
                }
            };
            if had {
                let _ = lifecycle::apply(
                    instance,
                    profile,
                    &previous,
                    &Action::Rename {
                        to: MOD_NAME.to_owned(),
                    },
                    false,
                );
            }
            Err(why)
        }
    }
}

/// Generate LOD as `req` says, emitting [`Event`]s while it runs (not `finished`).
#[allow(clippy::too_many_lines)] // one flow: plan, run, read, keep
pub fn generate(req: &LodRequest, emit: &mut dyn FnMut(Event)) -> LodOutcome {
    let mut r = result(req);
    let instance = match NvmodInstance::open(&req.instance) {
        Ok(i) => i,
        Err(e) => {
            let why = format!("{}: {e}", req.instance.display());
            return fail(r, Status::Refused, LodErrorCode::InstanceInvalid, None, why);
        }
    };
    let profile = match instance.resolve_profile(req.profile.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            return fail(
                r,
                Status::Refused,
                LodErrorCode::ProfileMissing,
                None,
                e.to_string(),
            );
        }
    };
    r.profile.clone_from(&profile);
    let plan = match plan(&instance, &profile, &req.worlds) {
        Ok(p) => p,
        Err(why) => return fail(r, Status::Refused, LodErrorCode::PlanRefused, None, why),
    };
    r.worlds = plan.request["worlds"]
        .as_array()
        .map_or_else(Vec::new, |w| {
            w.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        });
    if req.dry_run {
        return done(r, false);
    }
    let program = match &req.lodwright {
        Some(p) if p.is_file() => p.clone(),
        other => {
            let why = other.as_ref().map_or_else(
                || {
                    "no lodwright program: none is bundled next to raul; pass --lodwright <path>"
                        .to_owned()
                },
                |p| format!("the lodwright program {} is not a file", p.display()),
            );
            return fail(r, Status::Refused, LodErrorCode::ToolMissing, None, why);
        }
    };
    let sha256 = match std::fs::read(&program) {
        Ok(b) => sha256_hex(&b),
        Err(e) => {
            let why = format!("{}: {e}", program.display());
            return fail(r, Status::Refused, LodErrorCode::ToolMissing, None, why);
        }
    };
    let work = plan
        .output_dir
        .parent()
        .map_or_else(|| instance.root.join(WORK_DIR), Path::to_path_buf);
    let (request_file, events) = (work.join("request.json"), work.join("events.ndjson"));
    let written = std::fs::create_dir_all(&work).and_then(|()| {
        let _ = std::fs::remove_file(&events);
        std::fs::write(
            &request_file,
            serde_json::to_vec_pretty(&plan.request).unwrap_or_default(),
        )
    });
    if let Err(e) = written {
        let why = format!("{}: {e}", work.display());
        return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
    }
    r.request = Some(request_file.clone());
    let tool = ToolInfo {
        name: LODWRIGHT.to_owned(),
        version: None,
        path: program.clone(),
        sha256: sha256.clone(),
    };
    r.tool = Some(tool.clone());
    emit(Event::Started {
        dry_run: false,
        resume: false,
        instance: req.instance.clone(),
        profile: profile.clone(),
        tool,
    });
    emit(Event::Phase {
        phase: "terrain".to_owned(),
    });
    let runner = Runner {
        timeout: req.timeout,
        max_output: 16 * 1024 * 1024,
        ..Runner::default()
    };
    let args: Vec<String> = vec![
        "generate".into(),
        "--request".into(),
        request_file.display().to_string(),
        "--json".into(),
        "--events".into(),
        events.display().to_string(),
    ];
    let output = run_streaming(
        &runner,
        LODWRIGHT,
        &program,
        &args,
        &sha256,
        EventTail::new(events.clone(), lodwright_line),
        emit,
    );
    let output = match output {
        Ok(o) => o,
        Err(ToolError::Unverifiable { reason, .. }) => {
            return fail(
                r,
                Status::Unverifiable,
                LodErrorCode::ToolUnverifiable,
                None,
                reason,
            );
        }
        Err(e) if nvmod_core::cancel::requested() => {
            let why = format!("cancelled ({e}); nothing was added");
            return fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why);
        }
        Err(e) => {
            return fail(
                r,
                Status::Failed,
                LodErrorCode::ToolFailed,
                None,
                e.to_string(),
            );
        }
    };
    let doc = match read_result(&output.stdout) {
        Ok(d) => d,
        Err((status, code, why)) => return fail(r, status, code, None, why),
    };
    if let Some(t) = &mut r.tool {
        t.version = doc["tool"]["version"].as_str().map(str::to_owned);
    }
    r.lodwright = Some(doc.clone());
    let lodwright_code = doc["code"].as_str().map(str::to_owned);
    let reason = doc["reason"]
        .as_str()
        .unwrap_or("no reason given")
        .to_owned();
    let because = |what: &str| match &lodwright_code {
        Some(c) => format!("{what} ({c}): {reason}"),
        None => format!("{what}: {reason}"),
    };
    match output.exit_code {
        0 if doc["status"] == "ok" => {}
        0 => {
            let why = format!("lodwright exited 0 but its result says {}", doc["status"]);
            return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
        }
        1 | 2 => {
            let why = because("lodwright failed");
            return fail(
                r,
                Status::Failed,
                LodErrorCode::LodwrightFailed,
                lodwright_code,
                why,
            );
        }
        3 => {
            let why = because("lodwright cannot do this yet; nothing was written");
            return fail(
                r,
                Status::Unsupported,
                LodErrorCode::Unsupported,
                lodwright_code,
                why,
            );
        }
        other => {
            let why = format!("lodwright ended with exit code {other}, which it does not document");
            return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
        }
    }
    if nvmod_core::cancel::requested() {
        let why = "cancelled before the Mod was replaced; nothing was added";
        return fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why);
    }
    match read_manifest(&plan.output_dir) {
        Ok(Some(_)) => {}
        Ok(None) => {
            let why = "lodwright reported success but left no manifest";
            return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
        }
        Err((code, why)) => {
            let status = if code == LodErrorCode::ContractRefused {
                Status::Refused
            } else {
                Status::Failed
            };
            return fail(r, status, code, None, why);
        }
    }
    let current = find_ci(&instance.root.join("mods"), MOD_NAME);
    if lodwright_code.as_deref() == Some("up_to_date")
        && let Some(dir) = current
    {
        r.up_to_date = true;
        r.mod_name = Some(MOD_NAME.to_owned());
        r.files = Some(count_files(&dir));
        return done(r, false);
    }
    emit(Event::Phase {
        phase: "add_mod".to_owned(),
    });
    let copy = work.join("mod");
    let _ = clear(&copy);
    if let Err(e) = copy_output(&plan.output_dir, &copy) {
        let _ = clear(&copy);
        let why = format!("{}: {e}", copy.display());
        return fail(r, Status::Failed, LodErrorCode::AddModFailed, None, why);
    }
    let replaced = replace_mod(&instance, &profile, &copy);
    let _ = clear(&copy);
    match replaced {
        Ok((files, warning)) => {
            if let Some(message) = warning {
                emit(Event::Warning { message });
            }
            r.mod_name = Some(MOD_NAME.to_owned());
            r.files = Some(files);
            done(r, true)
        }
        Err(why) => {
            let why =
                format!("putting the LOD into the Instance failed and was rolled back: {why}");
            fail(r, Status::Failed, LodErrorCode::AddModFailed, None, why)
        }
    }
}
