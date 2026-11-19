// SPDX-License-Identifier: GPL-3.0-only
//! LOD through lodwright-rs, linked into `raul`: the request document the Toolbox builds from a Profile, checked
//! against the `lodwright-request` v1 schema lodwright publishes (a snapshot is in `contracts/snapshots/lodwright-rs/`),
//! and the manifest lodwright leaves in its output folder. [`generate`] calls lodwright's library API in process with
//! that request and keeps its output as one Mod per Profile, marked as generated LOD in its `meta.ini`.

mod generate;
mod inproc;
mod marker;
mod mods;
pub mod settings;

pub use generate::{LODWRIGHT, LodOutcome, LodRequest, generate, generate_with_cancel};
pub use marker::{LEGACY_MOD_NAME, Marker, is_generated, meta_ini, mod_name, read_marker};
pub use mods::{
    GeneratedMod, LodMods, MODS_CONTRACT, MODS_SCHEMA_VERSION, generated_mods, mods_schema,
};

use std::path::{Path, PathBuf};

use nvmod_instance::NvmodInstance;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The Contract name of the request.
pub const REQUEST_CONTRACT: &str = "lodwright-request";

/// The request version this build writes.
pub const REQUEST_VERSION: u32 = 1;

/// The `lodwright-request` v1 schema as lodwright publishes it.
pub const REQUEST_SCHEMA: &str =
    include_str!("../../../contracts/snapshots/lodwright-rs/lodwright-request.v1.json");

/// The Contract name of the manifest lodwright leaves in its output folder.
pub const MANIFEST_CONTRACT: &str = "lodwright-manifest";

/// The manifest versions this build reads.
pub const MANIFEST_VERSIONS: &[i64] = &[1];

/// The manifest's file name in the output folder.
pub const MANIFEST_FILE: &str = "manifest.json";

/// The LOD settings of a Profile: `raul-lod.toml` in the Profile's folder. Every field is optional; a Profile
/// without the file gets lodwright's defaults (every world, terrain, objects and trees, no exclusions).
pub const SETTINGS_FILE: &str = "raul-lod.toml";

/// The folder inside the Instance that holds lodwright's work folder of each Profile.
pub const WORK_DIR: &str = ".raul/lod";

/// The output folder of `profile` in the Instance at `root`: `<root>/.raul/lod/<Profile>/out`, where lodwright writes
/// and keeps its manifest between runs. Each Profile has its own, so switching Profiles does not make the next run of
/// the other one start over (ADR-0021).
#[must_use]
pub fn output_dir(root: &Path, profile: &str) -> PathBuf {
    root.join(WORK_DIR).join(profile).join("out")
}

/// The output folder every Profile shared before each had its own.
pub const SHARED_OUTPUT_DIR: &str = ".raul/lod/out";

/// The LOD settings of a Profile. [`settings`] reads and changes them option by option.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Worlds to generate (editor ids); empty means every world with terrain.
    pub worlds: Vec<String>,
    /// Which objects enter object LOD.
    pub inclusion: Inclusion,
    /// What is written.
    pub output: Output,
    /// Worker threads; lodwright decides when unset.
    pub threads: Option<u32>,
    /// An advisory memory budget in MiB.
    pub memory_mib: Option<u64>,
    /// Let the output folder lie inside a git working tree, which is refused otherwise (raul and lodwright both
    /// check). Not a LOD generation setting: it changes where output may go, not what is built.
    pub allow_in_repo: bool,
    /// Per terrain level: what differs from the defaults. When any level sets anything, the request lists all four
    /// levels, the rest at their defaults.
    pub levels: Levels,
}

/// The terrain levels the settings change, by their cell count (`[levels.4]` to `[levels.32]`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Levels {
    /// Level 4.
    #[serde(rename = "4", skip_serializing_if = "Option::is_none")]
    pub l4: Option<Level>,
    /// Level 8.
    #[serde(rename = "8", skip_serializing_if = "Option::is_none")]
    pub l8: Option<Level>,
    /// Level 16.
    #[serde(rename = "16", skip_serializing_if = "Option::is_none")]
    pub l16: Option<Level>,
    /// Level 32.
    #[serde(rename = "32", skip_serializing_if = "Option::is_none")]
    pub l32: Option<Level>,
}

impl Levels {
    /// The settings of `level` (4, 8, 16 or 32).
    #[must_use]
    pub fn of(&self, level: u32) -> Option<&Level> {
        match level {
            4 => self.l4.as_ref(),
            8 => self.l8.as_ref(),
            16 => self.l16.as_ref(),
            32 => self.l32.as_ref(),
            _ => None,
        }
    }
}

/// One terrain level's settings; each one the file leaves out takes its default.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Level {
    /// The allowed vertical error is twice this many units.
    pub quality: Option<f32>,
    /// Vertex cap per block, 3 to 32767.
    pub max_vertices: Option<u32>,
    /// Diffuse texture edge in pixels, a power of two from 4 to 4096.
    pub diffuse_size: Option<u32>,
    /// Normal texture edge in pixels, a power of two from 4 to 4096.
    pub normal_size: Option<u32>,
}

/// The terrain levels of the request: none (the generator's defaults) when the settings change no level and not
/// `always`, else all four with what the settings change and the defaults for the rest.
#[must_use]
pub fn levels_of(settings: &Settings, always: bool) -> Vec<lodwright_contract::LevelRequest> {
    let any = LEVELS_ALL.iter().any(|l| settings.levels.of(*l).is_some());
    if !any && !always {
        return Vec::new();
    }
    let default = |level: u32, field: &str| {
        settings::default_value(&format!("levels.{level}.{field}"))
            .and_then(|v| v.as_f64())
            .unwrap_or_default()
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // the defaults are small and whole
    LEVELS_ALL
        .iter()
        .map(|&level| {
            let set = settings.levels.of(level).cloned().unwrap_or_default();
            lodwright_contract::LevelRequest {
                level,
                quality: set.quality.unwrap_or(default(level, "quality") as f32),
                max_vertices: set
                    .max_vertices
                    .unwrap_or(default(level, "max_vertices") as u32),
                diffuse_size: set
                    .diffuse_size
                    .unwrap_or(default(level, "diffuse_size") as u32),
                normal_size: set
                    .normal_size
                    .unwrap_or(default(level, "normal_size") as u32),
            }
        })
        .collect()
}

const LEVELS_ALL: [u32; 4] = settings::LEVELS;

/// The parts of the request the settings decide: `terrain`, `objects`, `trees`, `inclusion` and `limits`.
#[must_use]
pub fn request_parts(settings: &Settings) -> Value {
    let mut limits = serde_json::Map::new();
    if let Some(t) = settings.threads {
        limits.insert("threads".into(), t.into());
    }
    if let Some(m) = settings.memory_mib {
        limits.insert("memory_mib".into(), m.into());
    }
    let o = &settings.output;
    let i = &settings.inclusion;
    json!({
        "terrain": {
            "enabled": o.terrain,
            "levels": levels_of(settings, false),
            "land_textures": o.land_textures,
            "water": o.water,
        },
        "objects": { "enabled": o.objects },
        "trees": { "enabled": o.trees },
        "inclusion": {
            "exclude_plugins": i.exclude_plugins,
            "only_plugins": i.only_plugins,
            "exclude_records": i.exclude_records,
            "exclude_models": i.exclude_models,
        },
        "limits": limits,
    })
}

/// The `request_digest` lodwright gives `request`: equal digests mean an output folder stays up to date.
///
/// # Errors
///
/// The document does not read as a lodwright request.
pub fn digest(request: &Value) -> Result<String, String> {
    serde_json::from_value::<lodwright_contract::Request>(request.clone())
        .map(|r| r.digest())
        .map_err(|e| e.to_string())
}

/// Which objects enter object LOD (lodwright's inclusion rules).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Inclusion {
    /// Leave out objects from these plugins.
    pub exclude_plugins: Vec<String>,
    /// When not empty, keep only objects from these plugins.
    pub only_plugins: Vec<String>,
    /// Leave out these records (eight hex digits).
    pub exclude_records: Vec<String>,
    /// Leave out models matching these patterns.
    pub exclude_models: Vec<String>,
}

/// What lodwright writes.
#[allow(clippy::struct_excessive_bools)] // each one is a switch of the request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Output {
    /// Terrain LOD.
    pub terrain: bool,
    /// Blend land textures into the terrain.
    pub land_textures: bool,
    /// Water planes in the terrain.
    pub water: bool,
    /// Object LOD.
    pub objects: bool,
    /// Tree LOD.
    pub trees: bool,
}

impl Default for Output {
    fn default() -> Output {
        Output {
            terrain: true,
            land_textures: true,
            water: true,
            objects: true,
            trees: true,
        }
    }
}

impl Settings {
    /// The settings of `profile`: its `raul-lod.toml`, or the defaults when it has none.
    ///
    /// # Errors
    ///
    /// The file exists but cannot be read or holds something these settings do not have.
    pub fn load(instance: &NvmodInstance, profile: &str) -> Result<Settings, String> {
        Settings::load_file(&instance.profile_dir(profile).join(SETTINGS_FILE))
    }

    /// The settings in `file`, checked as [`settings::check_table`] does, or the defaults when it does not exist.
    ///
    /// # Errors
    ///
    /// The file exists but cannot be read, or holds a key or value that is refused; the reason is in plain words.
    pub fn load_file(file: &Path) -> Result<Settings, String> {
        let text = match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()),
            Err(e) => return Err(format!("{}: {e}", file.display())),
        };
        let table: toml::Table = text
            .parse()
            .map_err(|e: toml::de::Error| format!("{}: {e}", file.display()))?;
        settings::check_table(&table).map_err(|r| format!("{}: {}", file.display(), r.line()))
    }
}

/// What a request is built from.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The Profile's active plugins, in Load order.
    pub plugins: Vec<String>,
    /// The Profile's Staging tree `Data` folder, which lodwright reads.
    pub data_folder: PathBuf,
    /// Where lodwright writes.
    pub output_dir: PathBuf,
    /// The request document.
    pub request: Value,
    /// The output folder may lie inside a git working tree: the Profile's settings or the run allow it. lodwright is
    /// told so, since it checks too; it is not part of the request.
    pub allow_in_repo: bool,
}

/// Build the request for `profile` of `instance`: the active plugins in Load order, the Staging tree's `Data`
/// folder, the Profile's settings, and `worlds` when given (they replace the settings' worlds). The output folder
/// is the Profile's own, `<instance>/.raul/lod/<Profile>/out` ([`output_dir`]), where the manifest stays between
/// runs. The request is checked against the
/// `lodwright-request` v1 schema before it is returned.
///
/// An output folder inside a git working tree is refused unless `allow_in_repo` (this run only) or the Profile's
/// `allow_in_repo` key allows it.
///
/// # Errors
///
/// Why no request can be built: the Profile has no Load order or no Staging tree, its settings cannot be read, the
/// output folder is refused, or the request does not pass the schema.
pub fn plan(
    instance: &NvmodInstance,
    profile: &str,
    worlds: &[String],
    allow_in_repo: bool,
) -> Result<Plan, String> {
    let settings = Settings::load(instance, profile)?;
    let allow_in_repo = allow_in_repo || settings.allow_in_repo;
    let (plugins, data_folder) = inputs(instance, profile)?;
    let absolute = |p: &Path| std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()));
    let output_dir = output_dir(&absolute(&instance.root)?, profile);
    // lodwright's own rule for the output folder, checked before it is asked.
    if output_dir.starts_with(&data_folder) {
        return Err(format!(
            "the output folder {} lies inside the data folder {}",
            output_dir.display(),
            data_folder.display()
        ));
    }
    if !allow_in_repo && let Some(repo) = output_dir.ancestors().find(|a| a.join(".git").exists()) {
        return Err(format!(
            "the output folder {} lies inside the git working tree {}, where LOD is not written unless it is \
             allowed: allow it for this Profile with `allow_in_repo = true` in its {SETTINGS_FILE}, or for one run \
             with --allow-in-repo",
            output_dir.display(),
            repo.display()
        ));
    }
    let worlds = if worlds.is_empty() {
        settings.worlds.clone()
    } else {
        worlds.to_vec()
    };
    let parts = request_parts(&settings);
    let request = json!({
        "schema_version": REQUEST_VERSION,
        "contract": REQUEST_CONTRACT,
        "data_folder": data_folder.display().to_string(),
        "plugins": plugins,
        "worlds": worlds,
        "output_dir": output_dir.display().to_string(),
        "terrain": parts["terrain"],
        "objects": parts["objects"],
        "trees": parts["trees"],
        "inclusion": parts["inclusion"],
        "limits": parts["limits"],
    });
    check_request(&request)?;
    Ok(Plan {
        plugins,
        data_folder,
        output_dir,
        request,
        allow_in_repo,
    })
}

/// The Profile's active plugins in Load order and its Staging tree's `Data` folder (absolute).
fn inputs(instance: &NvmodInstance, profile: &str) -> Result<(Vec<String>, PathBuf), String> {
    let slots =
        nvmod_loadorder::read_slots(&instance.profile_dir(profile)).map_err(|e| describe(&e))?;
    let plugins: Vec<String> = slots
        .into_iter()
        .filter(|s| s.active)
        .map(|s| s.name)
        .collect();
    if plugins.is_empty() {
        return Err(format!("profile `{profile}` has no active plugin"));
    }
    let data_folder = nvmod_loadorder::staging_data(instance, profile).map_err(|e| describe(&e))?;
    let data_folder =
        std::path::absolute(&data_folder).map_err(|e| format!("{}: {e}", data_folder.display()))?;
    Ok((plugins, data_folder))
}

/// What the settings of `profile` would quietly build nothing for: world names no plugin defines, inclusion rules
/// naming a plugin that is not active, and rules that leave no plugin for object LOD. Worlds are read from the
/// plugins only when the settings name any. Settings that cannot be read give no warnings (the settings commands
/// refuse them).
#[must_use]
pub fn settings_warnings(instance: &NvmodInstance, profile: &str) -> Vec<settings::Warning> {
    let Ok(s) = Settings::load(instance, profile) else {
        return Vec::new();
    };
    let found = inputs(instance, profile);
    let plugins = found.as_ref().ok().map(|(p, _)| p.as_slice());
    let worlds = if s.worlds.is_empty() {
        Ok(Vec::new())
    } else {
        found.as_ref().map_err(Clone::clone).and_then(|(p, data)| {
            lodwright_api::load::load_worlds(data, p)
                .map(|w| w.iter().map(|w| w.name().to_owned()).collect())
                .map_err(|e| e.summary)
        })
    };
    settings::warnings(&s, plugins, worlds.as_deref().map_err(String::as_str))
}

fn describe(e: &nvmod_loadorder::Error) -> String {
    match e {
        nvmod_loadorder::Error::Refused(why) | nvmod_loadorder::Error::Failed(why) => why.clone(),
    }
}

/// Check `request` against the `lodwright-request` v1 schema.
///
/// # Errors
///
/// The first few ways it does not match, for people.
///
/// # Panics
///
/// Never: the schema is a fixed, valid document.
pub fn check_request(request: &Value) -> Result<(), String> {
    let schema: Value = serde_json::from_str(REQUEST_SCHEMA).expect("the request schema is JSON");
    let validator = jsonschema::validator_for(&schema).expect("the request schema is valid");
    let errors: Vec<String> = validator
        .iter_errors(request)
        .take(5)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the request does not match lodwright-request v{REQUEST_VERSION}: {}",
            errors.join("; ")
        ))
    }
}

/// The manifest lodwright left in `output_dir`, or `None` when there is none.
///
/// # Errors
///
/// [`LodErrorCode::ContractRefused`] for a manifest of a newer version; [`LodErrorCode::ToolFailed`] when it
/// cannot be read or is not a `lodwright-manifest`. Each with a reason for people.
pub fn read_manifest(output_dir: &Path) -> Result<Option<Value>, (LodErrorCode, String)> {
    let failed = |why: String| (LodErrorCode::ToolFailed, why);
    let file = output_dir.join(MANIFEST_FILE);
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(failed(format!("{}: {e}", file.display()))),
    };
    let doc: Value =
        serde_json::from_str(&text).map_err(|e| failed(format!("{}: {e}", file.display())))?;
    if doc["contract"] != MANIFEST_CONTRACT {
        return Err(failed(format!(
            "{} is not a {MANIFEST_CONTRACT} document",
            file.display()
        )));
    }
    let newest = MANIFEST_VERSIONS.iter().max().copied().unwrap_or(1);
    match doc["schema_version"].as_i64() {
        Some(v) if MANIFEST_VERSIONS.contains(&v) => Ok(Some(doc)),
        Some(v) if v > newest => Err((
            LodErrorCode::ContractRefused,
            format!(
                "{} is {MANIFEST_CONTRACT} version {v}; this raul reads version {newest}",
                file.display()
            ),
        )),
        Some(v) => Err(failed(format!(
            "{} is {MANIFEST_CONTRACT} version {v}, which this raul does not read",
            file.display()
        ))),
        None => Err(failed(format!("{} has no schema_version", file.display()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    #[test]
    fn the_manifest_is_read_back_and_one_of_another_kind_or_version_is_refused() {
        let dir = TempDir::new("raul-manifest");
        assert_eq!(read_manifest(dir.path()), Ok(None));
        let write = |text: &str| write_file(dir.path(), MANIFEST_FILE, text.as_bytes());
        write(r#"{"schema_version":1,"contract":"lodwright-manifest","files":[]}"#);
        assert_eq!(
            read_manifest(dir.path()).unwrap().unwrap()["files"],
            json!([])
        );
        write(r#"{"schema_version":2,"contract":"lodwright-manifest"}"#);
        let (code, why) = read_manifest(dir.path()).unwrap_err();
        assert_eq!(code, LodErrorCode::ContractRefused);
        assert!(why.contains("version 2"), "{why}");
        write(r#"{"schema_version":1,"contract":"lodwright-result"}"#);
        assert!(
            read_manifest(dir.path())
                .unwrap_err()
                .1
                .contains("not a lodwright-manifest")
        );
        write(r#"{"contract":"lodwright-manifest"}"#);
        assert!(
            read_manifest(dir.path())
                .unwrap_err()
                .1
                .contains("no schema_version")
        );
        write("not json");
        assert!(read_manifest(dir.path()).is_err());
    }

    #[test]
    fn the_default_settings_switch_everything_on() {
        let s: Settings = toml::from_str("").unwrap();
        assert_eq!(s, Settings::default());
        assert!(s.output.terrain && s.output.objects && s.output.trees);
        assert!(toml::from_str::<Settings>("[output]\nsky = true\n").is_err());
    }

    #[test]
    fn output_in_a_git_folder_is_allowed_only_when_the_settings_say_so() {
        assert!(!Settings::default().allow_in_repo);
        let s: Settings = toml::from_str("allow_in_repo = true\n").unwrap();
        assert!(s.allow_in_repo);
        assert_eq!(s.output, Output::default(), "the key changes nothing else");
    }
}

/// The Contract name of the result of `raul lod generate`.
pub const CONTRACT: &str = "raul-lod-result";

/// The version of that result this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// The result of `raul lod generate` (Contract `raul-lod-result`, version 1), carried by the `result` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LodResult {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-lod-result`.
    #[schemars(extend("const" = "raul-lod-result"))]
    pub contract: String,
    /// How the run ended; the same as the `finished` event's.
    pub status: crate::events::Status,
    /// lodwright found its earlier output current for the same request and inputs and wrote nothing, and the Mod
    /// was already there: nothing changed.
    pub up_to_date: bool,
    /// A dry run: the request was built and checked, and nothing was run or written.
    pub dry_run: bool,
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile.
    pub profile: String,
    /// The Profile's LOD Mod (`Generated LOD (<Profile>)`), or `null` when the run did not get that far.
    #[serde(rename = "mod")]
    pub mod_name: Option<String>,
    /// Files in the Mod, or `null` when no Mod was written.
    pub files: Option<u64>,
    /// The worlds the request names; empty for every world with terrain.
    pub worlds: Vec<String>,
    /// The request file, or `null`: lodwright is called in process with the request, so no file is written.
    pub request: Option<PathBuf>,
    /// lodwright, linked into `raul` (its name and version), or `null` when no generation started.
    pub tool: Option<crate::events::ToolInfo>,
    /// The `lodwright-result` document of the run, as lodwright returned it, or `null` when it returned none.
    pub lodwright: Option<Value>,
    /// Set unless the status is `success`.
    pub error: Option<LodError>,
}

/// Why `raul lod generate` did not succeed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LodError {
    /// The stable code callers match on.
    pub code: LodErrorCode,
    /// For `lodwright_failed` and `unsupported`: lodwright's own `code`, for example `output_not_empty`.
    pub lodwright_code: Option<String>,
    /// Prose for people; never matched on.
    pub message: String,
}

/// The stable error codes of `raul lod generate`. The exit code of each is in `contracts/README.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LodErrorCode {
    /// The Instance does not exist or cannot be opened (exit code 2).
    InstanceInvalid,
    /// The Profile does not exist (exit code 2).
    ProfileMissing,
    /// No request can be built from the Profile: no Load order, no Staging tree, settings that cannot be read, an
    /// output folder lodwright refuses, or a Mod that is not generated LOD has the LOD Mod's name (exit code 2).
    PlanRefused,
    /// Not produced: lodwright is linked into `raul`, so there is no program to find (exit code 2).
    ToolMissing,
    /// Not produced: lodwright is linked into `raul`, so there is no program to check (exit code 3).
    ToolUnverifiable,
    /// The generation stopped unexpectedly, ran past the timeout, reported a status that is not `ok`, or the work
    /// folder could not be made (exit code 4).
    ToolFailed,
    /// The output folder holds a manifest of a version this build does not read (exit code 2).
    ContractRefused,
    /// lodwright's generation failed (exit code 4); `lodwright_code` says why.
    LodwrightFailed,
    /// lodwright cannot do what the request asks yet (exit code 2). Nothing was written. Not produced by the linked
    /// library, which reports such a request as `LodwrightFailed`.
    Unsupported,
    /// The run was cancelled (exit code 4); the Instance is as it was.
    Cancelled,
    /// lodwright succeeded but putting its output into the Instance failed and was rolled back (exit code 4).
    AddModFailed,
}

/// The JSON Schema of [`LodResult`], as `contracts/raul-lod-result.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::schema_text(&schemars::schema_for!(LodResult))
}
