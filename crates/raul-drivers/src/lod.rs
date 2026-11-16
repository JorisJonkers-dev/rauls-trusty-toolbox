// SPDX-License-Identifier: GPL-3.0-only
//! LOD through lodwright-rs: the request document the Toolbox builds from a Profile and passes to
//! `lodwright generate --request <file>`, checked against the `lodwright-request` v1 schema lodwright publishes
//! (a snapshot is in `contracts/snapshots/lodwright-rs/`), and the manifest lodwright leaves in its output folder.

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
    let data_folder = std::path::absolute(&data_folder).unwrap_or(data_folder);
    let root = std::path::absolute(&instance.root).unwrap_or_else(|_| instance.root.clone());
    let output_dir = root.join(WORK_DIR).join("out");
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
/// The manifest cannot be read, is not a `lodwright-manifest`, or is of a version this build does not read.
pub fn read_manifest(output_dir: &Path) -> Result<Option<Value>, String> {
    let file = output_dir.join(MANIFEST_FILE);
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", file.display())),
    };
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    if doc["contract"] != MANIFEST_CONTRACT {
        return Err(format!(
            "{} is not a {MANIFEST_CONTRACT} document",
            file.display()
        ));
    }
    match doc["schema_version"].as_i64() {
        Some(v) if MANIFEST_VERSIONS.contains(&v) => Ok(Some(doc)),
        Some(v) => Err(format!(
            "{} is {MANIFEST_CONTRACT} version {v}; this raul reads version {}",
            file.display(),
            MANIFEST_VERSIONS.iter().max().copied().unwrap_or(1)
        )),
        None => Err(format!("{} has no schema_version", file.display())),
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
        assert!(read_manifest(dir.path()).unwrap_err().contains("version 2"));
        write(r#"{"schema_version":1,"contract":"lodwright-result"}"#);
        assert!(
            read_manifest(dir.path())
                .unwrap_err()
                .contains("not a lodwright-manifest")
        );
        write(r#"{"contract":"lodwright-manifest"}"#);
        assert!(
            read_manifest(dir.path())
                .unwrap_err()
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
