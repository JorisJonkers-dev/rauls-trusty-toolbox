// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod mods`: the generated LOD Mods a Profile lists, recognised by their marker (or the old shared name), in
//! priority order (Contract `raul-lod-mods`, version 1). The Mods page reads it to give those Mods their own
//! treatment: Regenerate instead of Remove and Update, no update check, no conflict warnings.

use std::path::PathBuf;

use nvmod_instance::{NvmodInstance, find_ci};
use serde::{Deserialize, Serialize};

use super::marker::{LEGACY_MOD_NAME, mod_name, read_marker};

/// The Contract name of `raul lod mods --json`.
pub const MODS_CONTRACT: &str = "raul-lod-mods";

/// The version of that document this build prints.
pub const MODS_SCHEMA_VERSION: u32 = 1;

/// The generated LOD Mods of a Profile (Contract `raul-lod-mods`, version 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LodMods {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-lod-mods`.
    #[schemars(extend("const" = "raul-lod-mods"))]
    pub contract: String,
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile.
    pub profile: String,
    /// The name `raul lod generate` gives this Profile's LOD Mod, whether or not it exists yet.
    pub own_name: String,
    /// The generated LOD Mods the Profile lists, highest priority first; empty when it lists none.
    pub mods: Vec<GeneratedMod>,
}

/// One generated LOD Mod in a Profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GeneratedMod {
    /// The Mod's folder name.
    pub name: String,
    /// Its place in the Profile's list, 1 the highest priority (separators count, as in `raul mods list`).
    pub position: usize,
    /// Whether the Profile enables it.
    pub enabled: bool,
    /// It is this Profile's own LOD Mod, the one `raul lod generate` replaces.
    pub own: bool,
    /// It has the name every Profile shared before and no marker: the Profile's next `raul lod generate` hands its
    /// place and state to the Profile's own Mod.
    pub legacy: bool,
    /// The Profile it was generated for; `null` for a Mod of the old name without a marker.
    pub profile: Option<String>,
    /// lodwright's digest of the request it was built from; `null` without a marker.
    pub request_digest: Option<String>,
    /// The lodwright version that built it; `null` without a marker.
    pub lodwright_version: Option<String>,
}

/// The generated LOD Mods `profile` of `instance` lists, highest priority first.
#[must_use]
pub fn generated_mods(instance: &NvmodInstance, profile: &str) -> LodMods {
    let mods_dir = instance.root.join("mods");
    let own_name = mod_name(profile);
    let mods = nvmod_modinstall::mods::list(instance, profile)
        .into_iter()
        .filter(|r| r.kind == "mod")
        .filter_map(|r| {
            let dir = find_ci(&mods_dir, &r.name)?;
            let marker = read_marker(&dir);
            let legacy = marker.is_none() && r.name.eq_ignore_ascii_case(LEGACY_MOD_NAME);
            if marker.is_none() && !legacy {
                return None;
            }
            Some(GeneratedMod {
                own: r.name.eq_ignore_ascii_case(&own_name),
                enabled: r.state == "enabled",
                position: r.position,
                legacy,
                profile: marker.as_ref().map(|m| m.profile.clone()),
                request_digest: marker.as_ref().map(|m| m.request_digest.clone()),
                lodwright_version: marker.map(|m| m.lodwright_version),
                name: r.name,
            })
        })
        .collect();
    LodMods {
        schema_version: MODS_SCHEMA_VERSION,
        contract: MODS_CONTRACT.to_owned(),
        instance: instance.root.clone(),
        profile: profile.to_owned(),
        own_name,
        mods,
    }
}

/// The JSON Schema of [`LodMods`], as `contracts/raul-lod-mods.v1.json` holds it.
#[must_use]
pub fn mods_schema() -> String {
    crate::schema_text(&schemars::schema_for!(LodMods))
}
