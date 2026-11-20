// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod mods`: the generated LOD Mods a Profile lists, recognised by their marker (or the old shared name), in
//! priority order (Contract `raul-lod-mods`, version 1). The Mods page reads it to give those Mods their own
//! treatment: Regenerate instead of Remove and Update, no update check, no conflict warnings.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use nvmod_instance::{NvmodInstance, find_ci};
use serde::{Deserialize, Serialize};

use super::marker::{LEGACY_MOD_NAME, META_FILE, mod_name, read_marker};
use super::stale::{self, Staleness};

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
    #[schemars(range(min = 1))]
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
    /// How out of date it is: only this Profile's own Mod has it, and `null` for the others.
    pub stale: Option<Staleness>,
    /// When it was generated, in seconds since 1970-01-01 UTC: the time of its marker file; `null` when the file
    /// cannot say.
    #[serde(default)]
    pub generated_at: Option<u64>,
    /// How many files the Mod holds (its folder, without `meta.ini`); `null` when the folder cannot be read.
    #[serde(default)]
    pub files: Option<u64>,
}

/// `secs` (since 1970-01-01 UTC) as `YYYY-MM-DD HH:MM UTC`.
#[must_use]
pub fn utc_minute(secs: u64) -> String {
    let (days, rest) = (secs / 86_400, secs % 86_400);
    // Civil date from a day count (days since 1970-01-01), the proleptic Gregorian calendar.
    let z = days.cast_signed() + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        rest / 3_600,
        rest % 3_600 / 60
    )
}

/// The seconds since 1970 at which `meta.ini` of the Mod folder `dir` was last written.
fn generated_at(dir: &Path) -> Option<u64> {
    let modified = std::fs::metadata(find_ci(dir, META_FILE)?)
        .ok()?
        .modified()
        .ok()?;
    Some(modified.duration_since(UNIX_EPOCH).ok()?.as_secs())
}

/// The files under `dir` except the top-level `meta.ini`, `None` when a folder cannot be read.
fn file_count(dir: &Path) -> Option<u64> {
    fn walk(dir: &Path, top: bool) -> Option<u64> {
        let mut count = 0;
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let kind = entry.file_type().ok()?;
            if kind.is_dir() {
                count += walk(&entry.path(), false)?;
            } else if !(top && entry.file_name().eq_ignore_ascii_case(META_FILE)) {
                count += 1;
            }
        }
        Some(count)
    }
    walk(dir, true)
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
            let own = r.name.eq_ignore_ascii_case(&own_name);
            Some(GeneratedMod {
                stale: (own && marker.is_some()).then(|| stale::check(instance, profile)),
                own,
                enabled: r.state == "enabled",
                position: r.position,
                legacy,
                generated_at: generated_at(&dir),
                files: file_count(&dir),
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

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    #[test]
    fn a_time_is_written_as_a_utc_date_and_minute() {
        assert_eq!(utc_minute(0), "1970-01-01 00:00 UTC");
        assert_eq!(utc_minute(951_782_400), "2000-02-29 00:00 UTC");
        assert_eq!(utc_minute(1_795_221_599), "2026-11-21 00:39 UTC");
    }

    #[test]
    fn the_files_of_a_mod_are_counted_without_its_meta_ini() {
        let dir = TempDir::new("raul-mod-files");
        write_file(dir.path(), "meta.ini", b"[General]\n");
        write_file(dir.path(), "textures/a.dds", b"x");
        write_file(dir.path(), "meshes/lod/b.nif", b"x");
        write_file(dir.path(), "meshes/lod/meta.ini", b"x");
        assert_eq!(file_count(dir.path()), Some(3));
        assert_eq!(file_count(&dir.path().join("none")), None);
    }
}
