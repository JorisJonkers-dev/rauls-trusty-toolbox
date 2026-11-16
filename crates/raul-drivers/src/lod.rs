// SPDX-License-Identifier: GPL-3.0-only
//! LOD through lodwright-rs: the request document the Toolbox builds from a Profile and passes to
//! `lodwright generate --request <file>`, checked against the `lodwright-request` v1 schema lodwright publishes
//! (a snapshot is in `contracts/snapshots/lodwright-rs/`), and the manifest lodwright leaves in its output folder.
//! [`generate`] runs lodwright with that request and keeps its output as one Mod.

mod generate;

pub use generate::{LODWRIGHT, LODWRIGHT_RESULT_VERSIONS, LodOutcome, LodRequest, generate};

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

/// The folder inside the Instance where lodwright writes and keeps its manifest between runs.
pub const WORK_DIR: &str = ".raul/lod";

/// The LOD settings of a Profile.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
        let file = instance.profile_dir(profile).join(SETTINGS_FILE);
        match std::fs::read_to_string(&file) {
            Ok(text) => toml::from_str(&text).map_err(|e| format!("{}: {e}", file.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(e) => Err(format!("{}: {e}", file.display())),
        }
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
}

/// Build the request for `profile` of `instance`: the active plugins in Load order, the Staging tree's `Data`
/// folder, the Profile's settings, and `worlds` when given (they replace the settings' worlds). The output folder
/// is `<instance>/.raul/lod/out`, where the manifest stays between runs. The request is checked against the
/// `lodwright-request` v1 schema before it is returned.
///
/// # Errors
///
/// Why no request can be built: the Profile has no Load order or no Staging tree, its settings cannot be read, or
/// the request does not pass the schema.
pub fn plan(instance: &NvmodInstance, profile: &str, worlds: &[String]) -> Result<Plan, String> {
    let settings = Settings::load(instance, profile)?;
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
    let absolute = |p: &Path| std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()));
    let data_folder = absolute(&data_folder)?;
    let output_dir = absolute(&instance.root)?.join(WORK_DIR).join("out");
    // lodwright's own rule for the output folder, checked before it is asked.
    if output_dir.starts_with(&data_folder) {
        return Err(format!(
            "the output folder {} lies inside the data folder {}",
            output_dir.display(),
            data_folder.display()
        ));
    }
    if let Some(repo) = output_dir.ancestors().find(|a| a.join(".git").exists()) {
        return Err(format!(
            "the output folder {} lies inside the git working tree {}; lodwright refuses that",
            output_dir.display(),
            repo.display()
        ));
    }
    let worlds = if worlds.is_empty() {
        settings.worlds.clone()
    } else {
        worlds.to_vec()
    };
    let mut limits = serde_json::Map::new();
    if let Some(t) = settings.threads {
        limits.insert("threads".into(), t.into());
    }
    if let Some(m) = settings.memory_mib {
        limits.insert("memory_mib".into(), m.into());
    }
    let o = &settings.output;
    let i = &settings.inclusion;
    let request = json!({
        "schema_version": REQUEST_VERSION,
        "contract": REQUEST_CONTRACT,
        "data_folder": data_folder.display().to_string(),
        "plugins": plugins,
        "worlds": worlds,
        "output_dir": output_dir.display().to_string(),
        "terrain": { "enabled": o.terrain, "levels": [], "land_textures": o.land_textures, "water": o.water },
        "objects": { "enabled": o.objects },
        "trees": { "enabled": o.trees },
        "inclusion": {
            "exclude_plugins": i.exclude_plugins,
            "only_plugins": i.only_plugins,
            "exclude_records": i.exclude_records,
            "exclude_models": i.exclude_models,
        },
        "limits": limits,
    });
    check_request(&request)?;
    Ok(Plan {
        plugins,
        data_folder,
        output_dir,
        request,
    })
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
        .map(|e| format!("{}: {e}", e.instance_path))
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
}

/// The Contract name of the result of `raul lod generate`.
pub const CONTRACT: &str = "raul-lod-result";

/// The version of that result this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// The Mod the LOD output becomes.
pub const MOD_NAME: &str = "LOD (lodwright)";

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
    /// The LOD Mod (`LOD (lodwright)`), or `null` when the run did not get that far.
    #[serde(rename = "mod")]
    pub mod_name: Option<String>,
    /// Files in the Mod, or `null` when no Mod was written.
    pub files: Option<u64>,
    /// The worlds the request names; empty for every world with terrain.
    pub worlds: Vec<String>,
    /// The request file passed to lodwright, or `null` when none was written.
    pub request: Option<PathBuf>,
    /// The lodwright program that ran, or `null` when none did.
    pub tool: Option<crate::events::ToolInfo>,
    /// The `lodwright-result` document lodwright printed, as it printed it, or `null` when it printed none.
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
    /// No request can be built from the Profile: no Load order, no Staging tree, settings that cannot be read, or
    /// an output folder lodwright refuses (exit code 2).
    PlanRefused,
    /// No lodwright program: none is bundled next to `raul` and `--lodwright` was not given (exit code 2).
    ToolMissing,
    /// The lodwright program changed between its checksum and its start (exit code 3); it was not started.
    ToolUnverifiable,
    /// lodwright could not be started, ended without a result, or printed one that is not a `lodwright-result`
    /// (exit code 4).
    ToolFailed,
    /// lodwright printed a `lodwright-result` or left a manifest of a version this build does not read (exit
    /// code 2).
    ContractRefused,
    /// lodwright ran and failed (its exit code 1 or 2; exit code 4); `lodwright_code` says why.
    LodwrightFailed,
    /// lodwright cannot do what the request asks yet (its exit code 3; exit code 2). Nothing was written.
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
