// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate`: lodwright's generate, called in process through `lodwright-api` with the request the Profile
//! gives (its progress becomes `raul-events` lines), and its output kept as the Profile's LOD Mod (`Generated LOD (<Profile>)`). A second run
//! over an unchanged Load order is `up_to_date` and writes nothing. Every outcome but success leaves the Instance as
//! it was.

use std::path::{Path, PathBuf};
use std::time::Duration;

use lodwright_api::{CancelToken, GenerateOutcome, GenerateRequest};
use lodwright_contract::{Request as LodwrightRequest, Status as LodwrightStatus, WARNINGS_FILE};
use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::lifecycle::{self, Action};
use nvmod_modinstall::mods::{Change, ModsError, Place};
use nvmod_modinstall::{InstallError, Limits, OnConflict, Request};
use nvmod_plugins::{Edit, Place as PluginPlace, PluginsError};
use serde_json::Value;

use super::inproc;
use super::marker::{self, LEGACY_MOD_NAME, META_FILE, Marker, mod_name};
use super::run_lock::RunLock;
use super::stale::{self, Fingerprint};
use super::{
    CONTRACT, LodError, LodErrorCode, LodResult, MANIFEST_FILE, SCHEMA_VERSION, SHARED_OUTPUT_DIR,
    WORK_DIR, plan, read_manifest,
};
use crate::Stop;
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

/// What a run that stopped before the swap says about the LOD Mod: lodwright builds in a staging folder next to the
/// output and replaces the output only when it is finished, and the Mod is replaced from the output after that.
const MOD_UNTOUCHED: &str = "the LOD Mod in the Instance was not touched";

/// The request document of a Profile as the library's typed request.
fn typed(request: &Value) -> Result<LodwrightRequest, String> {
    serde_json::from_value(request.clone()).map_err(|e| e.to_string())
}

/// Copy `from` into `to`, leaving out lodwright's manifest and warnings file (they describe lodwright's run and stay in
/// its folder; they are not game files).
fn copy_output(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let (source, name) = (entry.path(), entry.file_name());
        if from.join(MANIFEST_FILE) == source || from.join(WARNINGS_FILE) == source {
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

/// Whether `profile` lists the Mod `name`, and if so whether it enables it.
fn listed(instance: &NvmodInstance, profile: &str, name: &str) -> Option<bool> {
    nvmod_modinstall::mods::list(instance, profile)
        .into_iter()
        .find(|r| r.kind == "mod" && r.name.eq_ignore_ascii_case(name))
        .map(|r| r.state == "enabled")
}

fn mods_error(e: ModsError) -> String {
    match e {
        ModsError::Refused(w) | ModsError::Failed(w) => w,
    }
}

const REMOVE: Action = Action::Remove {
    force: true,
    dependants: Vec::new(),
};

fn rename(instance: &NvmodInstance, profile: &str, from: &str, to: &str) -> Result<(), String> {
    let action = Action::Rename { to: to.to_owned() };
    lifecycle::apply(instance, profile, from, &action, false)
        .map(|_| ())
        .map_err(mods_error)
}

fn remove(instance: &NvmodInstance, profile: &str, name: &str) -> Result<(), String> {
    lifecycle::apply(instance, profile, name, &REMOVE, false)
        .map(|_| ())
        .map_err(mods_error)
}

fn change(instance: &NvmodInstance, profile: &str, change: &Change) -> Result<(), String> {
    nvmod_modinstall::mods::apply(instance, profile, change, false)
        .map(|_| ())
        .map_err(mods_error)
}

/// A backup an interrupted run left behind is this command's own. Beside the Mod `name` it is the older copy and
/// goes; alone it is the only copy, stopped between the rename and the install, and is renamed back.
fn recover(instance: &NvmodInstance, profile: &str, name: &str) -> Result<(), String> {
    let mods = instance.root.join("mods");
    let previous = format!("{name} previous");
    if find_ci(&mods, &previous).is_none() {
        return Ok(());
    }
    if find_ci(&mods, name).is_some() {
        remove(instance, profile, &previous)
    } else {
        rename(instance, profile, &previous, name)
    }
}

/// The Mod the new LOD replaces, as `profile` has it.
struct Replaced {
    /// Its line in the Profile while the new Mod goes in: `<name> previous`, or the old shared name.
    line: String,
    /// Whether the Profile lists it, and if so whether it enables it.
    enabled: Option<bool>,
    /// It is the Mod of the old shared name, which other Profiles may list too.
    legacy: bool,
}

/// The `meta.ini` of the Mod the next run of `profile` replaces, when it has one: the Profile's own LOD Mod (or the
/// copy an interrupted run renamed aside), or the old shared one when the Profile lists that instead.
fn replaced_meta(instance: &NvmodInstance, profile: &str, name: &str) -> Option<Vec<u8>> {
    let mods = instance.root.join("mods");
    let legacy_previous = format!("{LEGACY_MOD_NAME} previous");
    let dir = find_ci(&mods, name)
        .or_else(|| find_ci(&mods, &format!("{name} previous")))
        .or_else(|| listed(instance, profile, LEGACY_MOD_NAME).and(find_ci(&mods, LEGACY_MOD_NAME)))
        // An interrupted run of the old shared Mod left only its renamed-aside copy.
        .or_else(|| {
            listed(instance, profile, &legacy_previous).and(find_ci(&mods, &legacy_previous))
        })?;
    std::fs::read(find_ci(&dir, META_FILE)?).ok()
}

/// Put the output into the Instance as the Mod `name` of `profile`. The Mod it replaces (the Profile's own LOD Mod,
/// or the old shared one the Profile lists) gives the new one its place and its enabled state; a new Mod goes on
/// top, enabled. The Profile's own Mod is renamed out of the way first, removed once the new one is in, and renamed
/// back when the new one cannot be added. The old shared Mod is removed, or only disabled in this Profile while
/// another Profile lists it. Returns the Mod's file count (its `meta.ini` not counted), and warnings for steps after
/// the new Mod was in place that did not work.
fn replace_mod(
    instance: &NvmodInstance,
    profile: &str,
    name: &str,
    source: &Path,
) -> Result<(u64, Vec<String>), String> {
    let mods = instance.root.join("mods");
    let previous = format!("{name} previous");
    recover(instance, profile, name)?;
    recover(instance, profile, LEGACY_MOD_NAME)?;
    let replaced = if find_ci(&mods, name).is_some() {
        rename(instance, profile, name, &previous)?;
        Some(Replaced {
            enabled: listed(instance, profile, &previous),
            line: previous.clone(),
            legacy: false,
        })
    } else if find_ci(&mods, LEGACY_MOD_NAME).is_some()
        && let Some(enabled) = listed(instance, profile, LEGACY_MOD_NAME)
    {
        Some(Replaced {
            line: LEGACY_MOD_NAME.to_owned(),
            enabled: Some(enabled),
            legacy: true,
        })
    } else {
        None
    };
    // Puts the Profile back as it was when the new Mod cannot be added or placed.
    let undo = |why: String| -> String {
        match &replaced {
            Some(r) if !r.legacy => match rename(instance, profile, &previous, name) {
                Ok(()) => why,
                Err(e) => format!(
                    "{why}; the old LOD Mod stays as `{previous}`, because renaming it back failed too: {e}"
                ),
            },
            _ => why,
        }
    };
    let request = Request {
        instance,
        profile,
        archive: source,
        name: Some(name),
        dry_run: false,
        answers: None,
        answers_inline: None,
        limits: Limits::default(),
        on_conflict: OnConflict::default(),
    };
    let report = match nvmod_modinstall::install(&request) {
        Ok(report) => report,
        Err(e) => {
            let why = match e {
                InstallError::Refused(w) | InstallError::Failed(w) => w,
                InstallError::Questions(_) => {
                    "the output holds a FOMOD that needs answers".to_owned()
                }
            };
            return Err(undo(why));
        }
    };
    let files = report
        .files
        .iter()
        .filter(|f| !f.eq_ignore_ascii_case(META_FILE))
        .count() as u64;
    // The new Mod is on top and enabled; it takes the replaced Mod's place and state.
    if let Some(r) = &replaced
        && let Some(enabled) = r.enabled
    {
        let placed = change(
            instance,
            profile,
            &Change::Move {
                name: name.to_owned(),
                to: Place::Before(r.line.clone()),
            },
        )
        .and_then(|()| {
            if enabled {
                Ok(())
            } else {
                change(instance, profile, &Change::Disable(vec![name.to_owned()]))
            }
        });
        if let Err(why) = placed {
            let why = match remove(instance, profile, name) {
                Ok(()) => why,
                Err(e) => format!("{why}; removing the new LOD Mod failed too: {e}"),
            };
            return Err(undo(why));
        }
    }
    Ok((
        files,
        drop_replaced(instance, profile, replaced.as_ref(), &previous),
    ))
}

/// The plugins at the top of the Mod folder `name` (a packed LOD's stub), by file name.
fn mod_plugins(instance: &NvmodInstance, name: &str) -> Vec<String> {
    let Some(dir) = find_ci(&instance.root.join("mods"), name) else {
        return Vec::new();
    };
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            Path::new(n).extension().is_some_and(|x| {
                ["esp", "esm", "esl"]
                    .iter()
                    .any(|w| x.eq_ignore_ascii_case(w))
            })
        })
        .collect();
    names.sort();
    names
}

/// An enabled LOD Mod's plugins are active and last in the Load order, so the game registers the archive they name
/// and its files win over every other far view. Installing a Mod does not activate a plugin the Profile already knew
/// (an earlier LOD Mod's stub of the same name, kept inactive after that Mod was turned off or replaced), so this is
/// done here. Returns a warning for each plugin it could not place; the Mod is in either way.
fn activate_plugins(instance: &NvmodInstance, profile: &str, name: &str) -> Vec<String> {
    if listed(instance, profile, name) != Some(true) {
        return Vec::new();
    }
    let say = |plugin: &str, e: PluginsError| {
        let why = match e {
            PluginsError::Refused(w) | PluginsError::Failed(w) => w,
        };
        format!(
            "the LOD Mod's plugin `{plugin}` could not be made active and last in the Load order, so the game may not \
             load the new far view: {why}"
        )
    };
    let mut warnings = Vec::new();
    for plugin in mod_plugins(instance, name) {
        let edits = [
            Edit::Enable {
                names: vec![plugin.clone()],
                with_masters: false,
            },
            Edit::Move {
                name: plugin.clone(),
                to: PluginPlace::Bottom,
            },
        ];
        for edit in &edits {
            if let Err(e) = nvmod_plugins::apply(instance, profile, edit, false) {
                warnings.push(say(&plugin, e));
                break;
            }
        }
    }
    warnings
}

/// Takes the replaced Mod out once the new one is in place: the Profile's renamed-aside copy is removed; the old
/// shared Mod is removed, or only disabled in this Profile while another Profile lists it. Returns a warning for each
/// step that did not work (the new Mod is in either way).
fn drop_replaced(
    instance: &NvmodInstance,
    profile: &str,
    replaced: Option<&Replaced>,
    previous: &str,
) -> Vec<String> {
    let mut warnings = Vec::new();
    match replaced {
        Some(r) if r.legacy => {
            let elsewhere = instance
                .profiles()
                .iter()
                .any(|p| p != profile && listed(instance, p, LEGACY_MOD_NAME).is_some());
            let done = if elsewhere {
                change(
                    instance,
                    profile,
                    &Change::Disable(vec![LEGACY_MOD_NAME.to_owned()]),
                )
            } else {
                remove(instance, profile, LEGACY_MOD_NAME)
            };
            if let Err(e) = done {
                warnings.push(format!(
                    "the new LOD Mod is in, but the old one, `{LEGACY_MOD_NAME}`, could not be turned off or removed: {e}; disable or remove it with `raul mods`"
                ));
            }
        }
        Some(_) => {
            if let Err(e) = remove(instance, profile, previous) {
                warnings.push(format!(
                    "the new LOD Mod is in, but the old one, `{previous}`, could not be removed: {e}; remove it with `raul mods remove`"
                ));
            }
        }
        None => {}
    }
    warnings
}

/// Records what the run was built from, so a later change can show the LOD as out of date. A failure is a warning:
/// the LOD is in place either way.
fn keep_fingerprint(
    instance: &NvmodInstance,
    profile: &str,
    plan: &super::Plan,
    manifest: &Value,
    emit: &mut dyn FnMut(Event),
) {
    let digest = manifest["request_digest"].as_str().unwrap_or_default();
    let kept = Fingerprint::capture(plan, digest)
        .and_then(|f| stale::record(&instance.root, profile, &f).map_err(|e| e.to_string()));
    if let Err(why) = kept {
        emit(Event::Warning {
            message: format!(
                "the LOD is in place, but what it was built from could not be recorded, so it will not show as out of date: {why}"
            ),
        });
    }
}

/// Generate LOD as `req` says, emitting [`Event`]s while it runs (not `finished`). It runs until it is done, failed or
/// stopped by a stop request or the timeout.
pub fn generate(req: &LodRequest, emit: &mut dyn FnMut(Event)) -> LodOutcome {
    generate_with_cancel(req, &CancelToken::new(), emit)
}

/// [`generate`] that stops when `stop` says so and on the timeout, and not on the process-wide request: the run a
/// window starts, whose stop belongs to that run alone.
pub fn generate_with_stop(
    req: &LodRequest,
    stop: Stop<'_>,
    emit: &mut dyn FnMut(Event),
) -> LodOutcome {
    generate_stopped(req, &CancelToken::new(), stop, emit)
}

/// [`generate`] with a token the caller can cancel from another thread. A cancelled run stops at its next safe point,
/// leaves the output it finished in the work folder, and adds nothing; the next run continues from it.
#[allow(clippy::too_many_lines)] // one flow: plan, run, read, keep
pub fn generate_with_cancel(
    req: &LodRequest,
    cancel: &CancelToken,
    emit: &mut dyn FnMut(Event),
) -> LodOutcome {
    generate_stopped(req, cancel, Stop::Process, emit)
}

#[allow(clippy::too_many_lines)] // one flow: plan, run, read, keep
fn generate_stopped(
    req: &LodRequest,
    cancel: &CancelToken,
    stop: Stop<'_>,
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
    let name = mod_name(&profile);
    let mods = instance.root.join("mods");
    // The Mod name, and the name an interrupted run renames it aside to, must be ours before anything replaces,
    // renames or removes them. The old shared name's backup is ours by name: that Mod had no marker.
    let previous = format!("{name} previous");
    if let Some(taken) = [&name, &previous]
        .into_iter()
        .find(|n| find_ci(&mods, n).is_some() && !marker::is_generated(&mods, n))
    {
        let why = format!(
            "a Mod named `{taken}` is in the Instance and is not generated LOD; rename or remove it, then generate again; nothing changed"
        );
        return fail(r, Status::Refused, LodErrorCode::PlanRefused, None, why);
    }
    // One run per Profile: held until this run ends, released by the system if the process dies.
    let _run = if req.dry_run {
        None
    } else {
        match RunLock::try_acquire(&instance.root, &profile) {
            Ok(Some(lock)) => Some(lock),
            Ok(None) => {
                let why = "a LOD run of this Profile is already going; this one did not start; nothing changed";
                return fail(r, Status::Refused, LodErrorCode::RunInProgress, None, why);
            }
            Err(e) => {
                let why = format!("{}: {e}", RunLock::path(&instance.root, &profile).display());
                return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
            }
        }
    };
    // The settings are read once, here: a change made while the run goes applies to the next run.
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
    // The output folder every Profile shared before becomes the first generating Profile's own, so its manifest
    // keeps the blocks that are still current.
    let shared = instance.root.join(SHARED_OUTPUT_DIR);
    if shared.is_dir() && !plan.output_dir.exists() {
        let moved = plan
            .output_dir
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::rename(&shared, &plan.output_dir));
        if let Err(e) = moved {
            let why = format!("{}: {e}", shared.display());
            return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
        }
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

    let run = inproc::generate_with_events(&job, cancel, stop, req.timeout, emit);
    if run.panicked {
        let why = "LOD generation stopped unexpectedly; nothing was added";
        return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
    }
    let doc = match run.answer {
        Ok(GenerateOutcome::Done { result }) => *result,
        Ok(GenerateOutcome::Cancelled) if run.timed_out => {
            let why = format!(
                "LOD generation ran longer than {} seconds and was stopped; {MOD_UNTOUCHED}, and the next run continues from its checkpoint",
                req.timeout.as_secs()
            );
            return fail(r, Status::Failed, LodErrorCode::ToolFailed, None, why);
        }
        Ok(GenerateOutcome::Cancelled) => {
            let why = format!(
                "cancelled; {MOD_UNTOUCHED}, and the next run with the same settings continues from the checkpoint it kept"
            );
            return fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why);
        }
        Err(e) => {
            let why = match &e.detail {
                Some(detail) => format!("{}: {detail}", e.summary),
                None => e.summary.clone(),
            };
            let why = format!("{why}; {MOD_UNTOUCHED}");
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
    if stop.requested() {
        let why = "cancelled before the Mod was replaced; nothing was added";
        return fail(r, Status::Cancelled, LodErrorCode::Cancelled, None, why);
    }
    let manifest = match read_manifest(&plan.output_dir) {
        Ok(Some(m)) => m,
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
    };
    if doc.code == "up_to_date"
        && let Some(dir) = find_ci(&mods, &name)
    {
        r.up_to_date = true;
        for message in activate_plugins(&instance, &profile, &name) {
            emit(Event::Warning { message });
        }
        keep_fingerprint(&instance, &profile, &plan, &manifest, emit);
        r.mod_name = Some(name);
        r.files = Some(count_files(&dir) - u64::from(find_ci(&dir, META_FILE).is_some()));
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
    let marker = Marker {
        profile: profile.clone(),
        request_digest: manifest["request_digest"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        lodwright_version: doc.tool.version.clone(),
    };
    let previous = replaced_meta(&instance, &profile, &name);
    let meta = marker::meta_ini(previous.as_deref(), &marker);
    if let Err(e) = std::fs::write(copy.join(META_FILE), meta) {
        let _ = clear(&copy);
        let why = format!("{}: {e}", copy.join(META_FILE).display());
        return fail(r, Status::Failed, LodErrorCode::AddModFailed, None, why);
    }
    let replaced = replace_mod(&instance, &profile, &name, &copy);
    let _ = clear(&copy);
    match replaced {
        Ok((files, mut warnings)) => {
            warnings.extend(activate_plugins(&instance, &profile, &name));
            for message in warnings {
                emit(Event::Warning { message });
            }
            keep_fingerprint(&instance, &profile, &plan, &manifest, emit);
            r.mod_name = Some(name);
            r.files = Some(files);
            done(r, true)
        }
        Err(why) if stop.requested() => {
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
