// SPDX-License-Identifier: GPL-3.0-only
//! Whether a Profile's generated LOD is out of date. A successful run records a fingerprint in the Profile's work
//! folder: lodwright's digest of the request, a digest of the settings and worlds the request was built from, and
//! for each active plugin in Load order its size, a digest of its first and last bytes and whether it carries a
//! world. Later the current Load order and settings are compared with it:
//!
//! * the settings or worlds changed (also Restore defaults and Undo restore): `must`;
//! * a plugin that carries a world was added, removed or changed: `must`;
//! * another plugin was added, removed or changed, or the plugins were reordered: `may`;
//! * anything else (Mods without plugins, plugins that are not active): current.
//!
//! "Carries a world" is a fact of the file, not of its name: the plugin has a top-level `WRLD` group.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use lodwright_api::load::find_plugin;
use lodwright_contract::{Sha256Stream, sha256_digest};
use nvmod_instance::NvmodInstance;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Plan, output_dir, plan};

/// The fingerprint file in the Profile's work folder.
pub const FILE: &str = "fingerprint.json";

/// The version of the fingerprint file.
const VERSION: u32 = 1;

/// How many bytes at the start and at the end of a plugin enter its digest.
const EDGE: u64 = 1 << 20;

/// One active plugin as the last run saw it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPrint {
    /// The name, in lower case.
    pub name: String,
    /// Its size in bytes.
    pub size: u64,
    /// A digest of its size and its first and last bytes.
    pub digest: String,
    /// It has a top-level `WRLD` group.
    pub world: bool,
}

/// What a run was built from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// The file version; 1.
    pub schema_version: u32,
    /// lodwright's digest of the request the run used.
    pub request_digest: String,
    /// A digest of the settings and worlds of the request, without the plugins.
    pub settings_digest: String,
    /// The active plugins in Load order.
    pub plugins: Vec<PluginPrint>,
    /// The run packed the LOD into an archive (`output.pack`). Not in the settings digest, so switching it is told
    /// apart from a change that rebuilds blocks; a fingerprint written before it existed was a loose run.
    #[serde(default)]
    pub pack: bool,
}

/// How out of date the LOD is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Nothing recorded, or it cannot be told now.
    Unknown,
    /// Nothing that shows in the LOD changed.
    Current,
    /// A change may show in the LOD.
    May,
    /// A change shows in the LOD: generate again.
    Must,
}

/// The state and why, in a plain line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Staleness {
    /// The state.
    pub state: State,
    /// What changed, or why it is not known; empty when current.
    pub reason: String,
}

fn settings_digest(request: &Value) -> String {
    let parts = json!({
        "worlds": request["worlds"],
        "terrain": request["terrain"],
        "objects": request["objects"],
        "trees": request["trees"],
        "inclusion": request["inclusion"],
        "limits": request["limits"],
    });
    sha256_digest(parts.to_string().as_bytes())
}

/// The folder a Profile keeps its fingerprint in.
#[must_use]
pub fn file_of(root: &Path, profile: &str) -> PathBuf {
    let out = output_dir(root, profile);
    out.parent().unwrap_or(root).join(FILE)
}

/// Whether the plugin file `path` has a top-level `WRLD` group. A file that cannot be read has none.
#[must_use]
pub fn carries_world(path: &Path) -> bool {
    scan_world(path).unwrap_or(false)
}

fn scan_world(path: &Path) -> std::io::Result<bool> {
    let mut f = File::open(path)?;
    let end = f.metadata()?.len();
    let mut head = [0u8; 24];
    f.read_exact(&mut head)?;
    if &head[..4] != b"TES4" {
        return Ok(false);
    }
    let mut at = 24 + u64::from(u32::from_le_bytes([head[4], head[5], head[6], head[7]]));
    while at + 24 <= end {
        f.seek(SeekFrom::Start(at))?;
        f.read_exact(&mut head)?;
        let size = u64::from(u32::from_le_bytes([head[4], head[5], head[6], head[7]]));
        if &head[..4] == b"GRUP" {
            if &head[8..12] == b"WRLD" {
                return Ok(true);
            }
            if size < 24 {
                return Ok(false);
            }
            at += size;
        } else {
            at += 24 + size;
        }
    }
    Ok(false)
}

fn print_of(name: &str, path: &Path) -> std::io::Result<PluginPrint> {
    let mut f = File::open(path)?;
    let size = f.metadata()?.len();
    let mut digest = Sha256Stream::new();
    digest.update(&size.to_le_bytes());
    let mut buf = Vec::new();
    f.by_ref().take(EDGE).read_to_end(&mut buf)?;
    digest.update(&buf);
    if size > EDGE {
        f.seek(SeekFrom::Start(size.saturating_sub(EDGE).max(EDGE)))?;
        buf.clear();
        f.read_to_end(&mut buf)?;
        digest.update(&buf);
    }
    Ok(PluginPrint {
        name: name.to_ascii_lowercase(),
        size,
        digest: digest.finish(),
        world: carries_world(path),
    })
}

fn prints(plan: &Plan) -> Result<Vec<PluginPrint>, String> {
    plan.plugins
        .iter()
        .map(|name| {
            let path = find_plugin(&plan.data_folder, name).map_err(|e| e.summary)?;
            print_of(name, &path).map_err(|e| format!("{name}: {e}"))
        })
        .collect()
}

impl Fingerprint {
    /// The fingerprint of a run built from `plan`, whose request lodwright digested as `request_digest`.
    ///
    /// # Errors
    /// A plugin cannot be read.
    pub fn capture(plan: &Plan, request_digest: &str) -> Result<Fingerprint, String> {
        Ok(Fingerprint {
            schema_version: VERSION,
            request_digest: request_digest.to_owned(),
            settings_digest: settings_digest(&plan.request),
            plugins: prints(plan)?,
            pack: plan.request["output"]["pack"].as_bool().unwrap_or(false),
        })
    }
}

/// Keep `fingerprint` as the Profile's own.
///
/// # Errors
/// The file cannot be written.
pub fn record(root: &Path, profile: &str, fingerprint: &Fingerprint) -> std::io::Result<()> {
    let file = file_of(root, profile);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(fingerprint).map_err(std::io::Error::other)?;
    let tmp = file.with_extension("json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, &file)
}

/// The Profile's fingerprint, or `None` when there is none or it is not one this build reads.
#[must_use]
pub fn load(root: &Path, profile: &str) -> Option<Fingerprint> {
    let text = std::fs::read_to_string(file_of(root, profile)).ok()?;
    let f: Fingerprint = serde_json::from_str(&text).ok()?;
    (f.schema_version == VERSION).then_some(f)
}

fn answer(state: State, reason: impl Into<String>) -> Staleness {
    Staleness {
        state,
        reason: reason.into(),
    }
}

/// Compares what a run recorded with what is now. Each side carries its own world flags.
#[must_use]
pub fn classify(then: &Fingerprint, now: &Fingerprint) -> Staleness {
    if then.settings_digest != now.settings_digest {
        return answer(State::Must, "The LOD settings changed since the last run.");
    }
    if then.pack != now.pack {
        return answer(
            State::Must,
            if now.pack {
                "Packing was turned on since the last run."
            } else {
                "Packing was turned off since the last run."
            },
        );
    }
    let names = |f: &Fingerprint| f.plugins.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
    let find = |f: &Fingerprint, n: &str| f.plugins.iter().find(|p| p.name == n).cloned();
    let mut must = Vec::new();
    let mut may = Vec::new();
    for p in &then.plugins {
        match find(now, &p.name) {
            None => (if p.world { &mut must } else { &mut may })
                .push(format!("{} was turned off", p.name)),
            Some(n) if n.digest != p.digest => (if p.world || n.world {
                &mut must
            } else {
                &mut may
            })
            .push(format!("{} changed", p.name)),
            Some(_) => {}
        }
    }
    for n in &now.plugins {
        if find(then, &n.name).is_none() {
            (if n.world { &mut must } else { &mut may }).push(format!("{} was turned on", n.name));
        }
    }
    if !must.is_empty() {
        return answer(
            State::Must,
            format!("A plugin with world data changed: {}.", must.join(", ")),
        );
    }
    if !may.is_empty() {
        return answer(
            State::May,
            format!(
                "The plugins changed since the last run: {}.",
                may.join(", ")
            ),
        );
    }
    let (a, b) = (names(then), names(now));
    if a != b {
        return answer(State::May, "The Load order changed since the last run.");
    }
    answer(State::Current, "")
}

/// How out of date the LOD of `profile` is. Reads the Load order and the settings and a little of each active
/// plugin; writes nothing.
#[must_use]
pub fn check(instance: &NvmodInstance, profile: &str) -> Staleness {
    let Some(then) = load(&instance.root, profile) else {
        return answer(State::Unknown, "No run of this build is recorded.");
    };
    // The output folder's place does not matter to what is read here.
    let now = plan(instance, profile, &[], true)
        .and_then(|p| Fingerprint::capture(&p, &then.request_digest));
    match now {
        Ok(now) => classify(&then, &now),
        Err(why) => answer(
            State::Unknown,
            format!("The current state could not be read: {why}"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};
    use raul_testkit::{covers, ids};

    fn p(name: &str, digest: &str, world: bool) -> PluginPrint {
        PluginPrint {
            name: name.to_owned(),
            size: 1,
            digest: digest.to_owned(),
            world,
        }
    }

    fn fp(settings: &str, plugins: Vec<PluginPrint>) -> Fingerprint {
        Fingerprint {
            schema_version: VERSION,
            request_digest: "sha256:r".to_owned(),
            settings_digest: settings.to_owned(),
            plugins,
            pack: false,
        }
    }

    fn base() -> Fingerprint {
        fp(
            "s",
            vec![
                p("a.esm", "1", true),
                p("b.esp", "2", false),
                p("c.esp", "3", false),
            ],
        )
    }

    #[test]
    fn switching_packing_must_regenerate_and_says_which_way() {
        let mut on = base();
        on.pack = true;
        let turned_on = classify(&base(), &on);
        assert_eq!(turned_on.state, State::Must);
        assert!(turned_on.reason.contains("turned on"));
        let turned_off = classify(&on, &base());
        assert_eq!(turned_off.state, State::Must);
        assert!(turned_off.reason.contains("turned off"));
        assert_eq!(classify(&on, &on).state, State::Current);
    }

    #[test]
    fn packing_stays_out_of_the_settings_digest_and_a_fingerprint_without_it_reads_as_loose() {
        let loose = json!({ "terrain": {}, "objects": {} });
        let mut packed = loose.clone();
        packed["output"] = json!({ "pack": true });
        assert_eq!(settings_digest(&loose), settings_digest(&packed));
        let old = r#"{"schema_version":1,"request_digest":"d","settings_digest":"s","plugins":[]}"#;
        assert!(!serde_json::from_str::<Fingerprint>(old).unwrap().pack);
    }

    #[test]
    fn nothing_changed_is_current() {
        covers(&[ids::lod::stale::NOTHING_CHANGED]);
        assert_eq!(classify(&base(), &base()).state, State::Current);
    }

    #[test]
    fn a_settings_change_must_regenerate() {
        covers(&[ids::lod::stale::MUST_REGENERATE]);
        let mut now = base();
        now.settings_digest = "t".to_owned();
        let s = classify(&base(), &now);
        assert_eq!(s.state, State::Must);
        assert!(s.reason.contains("settings"));
    }

    #[test]
    fn a_reorder_may() {
        covers(&[ids::lod::stale::REORDER_MAY]);
        let mut now = base();
        now.plugins.swap(1, 2);
        assert_eq!(classify(&base(), &now).state, State::May);
    }

    #[test]
    fn enable_and_disable_of_a_plugin_without_a_world_may() {
        let mut now = base();
        now.plugins.push(p("d.esp", "4", false));
        assert_eq!(classify(&base(), &now).state, State::May);
        let mut now = base();
        now.plugins.pop();
        assert_eq!(classify(&base(), &now).state, State::May);
    }

    #[test]
    fn enable_and_disable_of_a_plugin_with_a_world_must() {
        let mut now = base();
        now.plugins.push(p("d.esm", "4", true));
        assert_eq!(classify(&base(), &now).state, State::Must);
        let mut now = base();
        now.plugins.remove(0);
        assert_eq!(classify(&base(), &now).state, State::Must);
    }

    #[test]
    fn an_update_is_must_with_a_world_and_may_without() {
        let mut now = base();
        now.plugins[0].digest = "9".to_owned();
        assert_eq!(classify(&base(), &now).state, State::Must);
        let mut now = base();
        now.plugins[1].digest = "9".to_owned();
        assert_eq!(classify(&base(), &now).state, State::May);
    }

    fn record_bytes(kind: [u8; 4], data: usize) -> Vec<u8> {
        let mut v = kind.to_vec();
        v.extend_from_slice(&u32::try_from(data).unwrap().to_le_bytes());
        v.extend_from_slice(&[0; 16]);
        v.extend(std::iter::repeat_n(0u8, data));
        v
    }

    fn group(label: [u8; 4], inner: &[u8]) -> Vec<u8> {
        let mut v = b"GRUP".to_vec();
        v.extend_from_slice(&u32::try_from(24 + inner.len()).unwrap().to_le_bytes());
        v.extend_from_slice(&label);
        v.extend_from_slice(&[0; 12]);
        v.extend_from_slice(inner);
        v
    }

    #[test]
    fn a_world_is_a_top_level_wrld_group_and_not_a_name() {
        let dir = TempDir::new("raul-stale-scan");
        let mut with = record_bytes(*b"TES4", 10);
        with.extend(group(*b"GMST", &record_bytes(*b"GMST", 7)));
        with.extend(group(*b"WRLD", &record_bytes(*b"WRLD", 3)));
        let mut without = record_bytes(*b"TES4", 10);
        without.extend(group(*b"CELL", &record_bytes(*b"CELL", 5)));
        write_file(dir.path(), "world.esp", &with);
        write_file(dir.path(), "wasteland-looking-name.esp", &without);
        write_file(
            dir.path(),
            "junk.esp",
            b"not a plugin at all, not at all......",
        );
        assert!(carries_world(&dir.path().join("world.esp")));
        assert!(!carries_world(
            &dir.path().join("wasteland-looking-name.esp")
        ));
        assert!(!carries_world(&dir.path().join("junk.esp")));
        assert!(!carries_world(&dir.path().join("missing.esp")));
    }

    #[test]
    fn the_digest_follows_the_file_and_a_fingerprint_round_trips() {
        let dir = TempDir::new("raul-stale-print");
        write_file(dir.path(), "a.esp", &record_bytes(*b"TES4", 4));
        let one = print_of("A.esp", &dir.path().join("a.esp")).unwrap();
        assert_eq!(one.name, "a.esp");
        write_file(dir.path(), "a.esp", &record_bytes(*b"TES4", 5));
        let two = print_of("A.esp", &dir.path().join("a.esp")).unwrap();
        assert_ne!(one.digest, two.digest);
        let f = fp("s", vec![one]);
        record(dir.path(), "Default", &f).unwrap();
        assert_eq!(load(dir.path(), "Default"), Some(f));
        assert_eq!(load(dir.path(), "Other"), None);
    }
}
