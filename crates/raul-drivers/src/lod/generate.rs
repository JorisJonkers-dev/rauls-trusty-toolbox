// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate`: lodwright's generate, called in process through `lodwright-api` with the request the Profile
//! gives (its progress becomes `raul-events` lines), and its output kept as the Mod `LOD (lodwright)`. A second run
//! over an unchanged Load order is `up_to_date` and writes nothing. Every outcome but success leaves the Instance as
//! it was.

use std::path::{Path, PathBuf};
use std::time::Duration;

use lodwright_api::{CancelToken, GenerateOutcome, GenerateRequest};
use lodwright_contract::{Request as LodwrightRequest, Status as LodwrightStatus};
use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::lifecycle::{self, Action};
use nvmod_modinstall::{InstallError, Limits, Request};
use serde_json::Value;

use super::inproc;
use super::{
    CONTRACT, LodError, LodErrorCode, LodResult, MANIFEST_FILE, MOD_NAME, SCHEMA_VERSION, WORK_DIR,
    plan, read_manifest,
};
use crate::events::{Event, Status, ToolInfo};
use crate::ttw::driver::{clear, count_files};
use crate::ttw::exit_code;

/// The lodwright name, as the `tool` of the events and the result.
pub const LODWRIGHT: &str = "lodwright";

/// What `raul lod generate` was asked to do.
#[derive(Debug, Clone)]
pub struct LodRequest {
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile; the Instance's only or selected one when `None`.
    pub profile: Option<String>,
    /// Worlds that replace the Profile's settings; empty keeps them.
    pub worlds: Vec<String>,
    /// Build and check the request, run nothing, write nothing.
    pub dry_run: bool,
    /// Let the output folder lie inside a git working tree for this run, whatever the Profile's settings say; it
    /// writes nothing to them.
    pub allow_in_repo: bool,
    /// How long the generation may run. It is checked between terrain blocks, worlds and steps: a step that is already
    /// running when the time is up finishes first.
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

/// The request document of a Profile as the library's typed request.
fn typed(request: &Value) -> Result<LodwrightRequest, String> {
    serde_json::from_value(request.clone()).map_err(|e| e.to_string())
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
    let mods = instance.root.join("mods");
    let lifecycle_error = |e: nvmod_modinstall::mods::ModsError| match e {
        nvmod_modinstall::mods::ModsError::Refused(w)
        | nvmod_modinstall::mods::ModsError::Failed(w) => w,
    };
    let remove = Action::Remove {
        force: true,
        dependants: Vec::new(),
    };
    // A backup an interrupted run left behind is this command's own. Beside a LOD Mod it is the older copy and
    // goes first; alone it is the only copy, stopped between the rename and the install, and is renamed back.
    if find_ci(&mods, &previous).is_some() {
        if find_ci(&mods, MOD_NAME).is_some() {
            lifecycle::apply(instance, profile, &previous, &remove, false)
                .map_err(lifecycle_error)?;
        } else {
            let back = Action::Rename {
                to: MOD_NAME.to_owned(),
            };
            lifecycle::apply(instance, profile, &previous, &back, false)
                .map_err(lifecycle_error)?;
        }
    }
    let had = find_ci(&mods, MOD_NAME).is_some();
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
        limits: Limits::default(),
    };
    match nvmod_modinstall::install(&request) {
        Ok(report) => {
            if had {
                let removed = lifecycle::apply(instance, profile, &previous, &remove, false);
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
                let back = lifecycle::apply(
                    instance,
                    profile,
                    &previous,
                    &Action::Rename {
                        to: MOD_NAME.to_owned(),
                    },
                    false,
                );
                if let Err(e) = back {
                    return Err(format!(
                        "{why}; the old LOD Mod stays as `{previous}`, because renaming it back failed too: {}",
                        lifecycle_error(e)
                    ));
                }
            }
            Err(why)
        }
    }
}

/// Clears the output folder after a run that failed while writing it and left no manifest. Without a manifest the next
/// run cannot tell these files from a foreign folder and refuses with `output_not_empty`, so the folder is cleared.
/// A folder with a manifest is kept: the next run resumes from it.
fn discard_partial(out: &Path) {
    if !out.join(MANIFEST_FILE).exists() {
        let _ = clear(out);
    }
}

/// Generate LOD as `req` says, emitting [`Event`]s while it runs (not `finished`). It runs until it is done, failed or
/// stopped by a stop request or the timeout.
pub fn generate(req: &LodRequest, emit: &mut dyn FnMut(Event)) -> LodOutcome {
    generate_with_cancel(req, &CancelToken::new(), emit)
}

/// [`generate`] with a token the caller can cancel from another thread. A cancelled run stops at its next safe point,
/// leaves the output it finished in the work folder, and adds nothing; the next run continues from it.
#[allow(clippy::too_many_lines)] // one flow: plan, run, read, keep
pub fn generate_with_cancel(
    req: &LodRequest,
    cancel: &CancelToken,
    emit: &mut dyn FnMut(Event),
) -> LodOutcome {
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
    let plan = match plan(&instance, &profile, &req.worlds, req.allow_in_repo) {
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
    // The folder above the output holds the output and, after a run, the Mod's copy; it is lodwright's work folder.
    let work = plan
        .output_dir
        .parent()
        .map_or_else(|| instance.root.join(WORK_DIR), Path::to_path_buf);
    if let Err(e) = std::fs::create_dir_all(&work) {
        let why = format!("{}: {e}", work.display());
        return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
    }
    let job = match typed(&plan.request) {
        Ok(request) => GenerateRequest {
            allow_in_repo: plan.allow_in_repo,
            ..GenerateRequest::new(request)
        },
        Err(why) => {
            let why = format!("the request does not read as a lodwright request: {why}");
            return fail(r, Status::Refused, LodErrorCode::PlanRefused, None, why);
        }
    };
    // Linked into raul: there is no file to name, so the running executable stands for it.
    let tool = ToolInfo {
        name: LODWRIGHT.to_owned(),
        version: None,
        path: std::env::current_exe().unwrap_or_default(),
        sha256: String::new(),
    };
    r.tool = Some(tool.clone());
    emit(Event::Started {
        dry_run: false,
        resume: false,
        instance: req.instance.clone(),
        profile: profile.clone(),
        tool,
    });

    let run = inproc::generate_with_events(&job, cancel, req.timeout, emit);
    if run.panicked {
        discard_partial(&plan.output_dir);
        let why = "LOD generation stopped unexpectedly; nothing was added";
        return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
    }
    let doc = match run.answer {
        Ok(GenerateOutcome::Done { result }) => *result,
        Ok(GenerateOutcome::Cancelled) if run.timed_out => {
            let why = format!(
                "LOD generation ran longer than {} seconds and was stopped; nothing was added",
                req.timeout.as_secs()
            );
            return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
        }
        Ok(GenerateOutcome::Cancelled) => {
            let why = "cancelled; nothing was added to the Instance, and the next run continues the LOD already written";
            return fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why);
        }
        Err(e) => {
            if matches!(e.code, "generate_failed" | "output_failed") {
                discard_partial(&plan.output_dir);
            }
            let why = match &e.detail {
                Some(detail) => format!("{}: {detail}", e.summary),
                None => e.summary.clone(),
            };
            return fail(
                r,
                Status::Failed,
                LodErrorCode::LodwrightFailed,
                Some(e.code.to_owned()),
                why,
            );
        }
    };
    if let Some(t) = &mut r.tool {
        t.version = Some(doc.tool.version.clone());
    }
    r.lodwright = serde_json::to_value(&doc).ok();
    if doc.status != LodwrightStatus::Ok {
        let why = format!(
            "LOD generation reported {:?}; nothing was added",
            doc.status
        );
        return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
    }
    if nvmod_core::cancel::requested() {
        let why = "cancelled before the Mod was replaced; nothing was added";
        return fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why);
    }
    match read_manifest(&plan.output_dir) {
        Ok(Some(_)) => {}
        Ok(None) => {
            let why = "LOD generation reported success but left no manifest";
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
    if doc.code == "up_to_date"
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
        Err(why) if nvmod_core::cancel::requested() => {
            let why = format!("cancelled while the Mod was replaced; it was rolled back ({why})");
            fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why)
        }
        Err(why) => {
            let why =
                format!("putting the LOD into the Instance failed and was rolled back: {why}");
            fail(r, Status::Failed, LodErrorCode::AddModFailed, None, why)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    #[test]
    fn a_failed_write_without_a_manifest_is_cleared_and_one_with_a_manifest_is_kept() {
        let dir = TempDir::new("raul-lod-partial");
        let out = dir.path().join("out");
        write_file(&out, "meshes/partial.nif", b"half");
        discard_partial(&out);
        assert!(!out.exists(), "the partial output is cleared");

        write_file(&out, "meshes/done.nif", b"whole");
        write_file(&out, MANIFEST_FILE, b"{}");
        discard_partial(&out);
        assert!(
            out.join("meshes/done.nif").is_file(),
            "a resumable folder is kept"
        );
    }
}
