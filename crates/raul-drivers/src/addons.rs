// SPDX-License-Identifier: GPL-3.0-only
//! Add-ons: small Mods of raul's own that fix how a mod the player installed looks, made on the player's machine from
//! numbers and from the player's own copy of that mod's plugin, so no game or mod file is in this repository or in
//! raul's downloads. The list is data (`data/addons.json`): each add-on names the plugin it is for (its trigger), the
//! meshes it writes (from numbers) and the records of the trigger plugin it overrides, keyed by their model path.
//!
//! [`apply`] keeps a Profile's add-ons as the list says: a Profile whose Load order has an add-on's trigger plugin
//! active gets the add-on's Mod, enabled, right above the Mod that holds the trigger, and its plugin right after the
//! trigger. Its `meta.ini` carries a marker (`[raulGeneratedAddon]`) with the add-on's version and the digest of the
//! trigger plugin it was made from; a later apply makes it again only when one of them changed. A Profile without
//! the trigger is never touched. A Mod the player turned off or removed stays so: the Profile remembers in
//! `raul-addons.json` that the add-on was given to it.

pub mod nif;
pub mod plugin;

use std::path::{Path, PathBuf};

use nvmod_formats::metaini::{MetaIni, quote_value};
use nvmod_instance::{NvmodInstance, find_ci};
use nvmod_modinstall::mods::{Change, ModsError, Place};
use nvmod_modinstall::{InstallError, Limits, OnConflict, Request};
use nvmod_plugins::{Edit, Place as PluginPlace, PluginsError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use self::nif::GlowSphere;
use self::plugin::RecordRule;

/// The Contract name of `raul addons list|apply --json`.
pub const CONTRACT: &str = "raul-addons";

/// The version of that document this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// The `meta.ini` section of the marker.
pub const SECTION: &str = "raulGeneratedAddon";

/// The file in a Profile's folder that remembers which add-ons it was given.
pub const PROFILE_FILE: &str = "raul-addons.json";

const TABLE: &str = include_str!("../data/addons.json");

/// One mesh an add-on writes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeshSpec {
    /// Its path below `meshes\`, as the records name it.
    pub path: String,
    /// A glow sphere's numbers.
    pub glow_sphere: GlowSphere,
}

/// One add-on of the list.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Addon {
    /// A stable id, for example `small-lamp-glow`.
    pub id: String,
    /// Raised when what the add-on writes changes, so a Mod made by an older raul is made again.
    pub version: u32,
    /// The Mod's folder name.
    pub name: String,
    /// The plugin it writes.
    pub plugin: String,
    /// The plugin it is for: the add-on applies to a Profile that has it active.
    pub trigger_plugin: String,
    /// One sentence for people.
    pub summary: String,
    /// The meshes it writes.
    pub meshes: Vec<MeshSpec>,
    /// The records of the trigger plugin it overrides.
    pub records: Vec<RecordRule>,
}

/// The add-ons raul knows.
///
/// # Panics
///
/// Never: the list is checked by the tests.
#[must_use]
pub fn known() -> Vec<Addon> {
    serde_json::from_str(TABLE).expect("the add-on list is valid")
}

/// Where an add-on stands in a Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// The Profile does not have the trigger plugin active; nothing of the add-on is in it.
    NotNeeded,
    /// The Mod is in place and made from the trigger plugin the Profile has.
    UpToDate,
    /// Without `--dry-run` the add-on would be added or made again.
    OutOfDate,
    /// This apply added the Mod to the Profile.
    Added,
    /// This apply made the Mod again (a new trigger plugin or a new version of the add-on).
    Updated,
    /// The player turned the Mod off or removed it after raul added it; it stays so.
    TurnedOff,
    /// The add-on could not be applied; `error` says why. The Profile is as it was.
    Failed,
}

/// One add-on in a Profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddonState {
    /// The add-on's id.
    pub id: String,
    /// Its version.
    pub version: u32,
    /// The Mod's folder name.
    pub name: String,
    /// The plugin it writes.
    pub plugin: String,
    /// The plugin it is for.
    pub trigger_plugin: String,
    /// One sentence for people.
    pub summary: String,
    /// Where it stands.
    pub status: Status,
    /// Whether the Profile lists the Mod, and if so whether it enables it; `null` when it does not list it.
    pub enabled: Option<bool>,
    /// What this run changed, one sentence each; empty when nothing changed.
    pub changes: Vec<String>,
    /// Why it failed; `null` unless `status` is `failed`.
    pub error: Option<String>,
}

/// The add-ons of a Profile (Contract `raul-addons`, version 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddonsReport {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-addons`.
    #[schemars(extend("const" = "raul-addons"))]
    pub contract: String,
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile.
    pub profile: String,
    /// Nothing was written: `raul addons list`, or `apply --dry-run`.
    pub dry_run: bool,
    /// Every add-on raul knows, in the list's order.
    pub addons: Vec<AddonState>,
}

impl AddonsReport {
    /// Whether this run changed the Profile.
    #[must_use]
    pub fn changed(&self) -> bool {
        self.addons.iter().any(|a| !a.changes.is_empty())
    }

    /// The add-ons that failed.
    #[must_use]
    pub fn failed(&self) -> Vec<&AddonState> {
        self.addons
            .iter()
            .filter(|a| a.status == Status::Failed)
            .collect()
    }
}

/// The JSON Schema of [`AddonsReport`], as `contracts/raul-addons.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::schema_text(&schemars::schema_for!(AddonsReport))
}

/// What the Profile remembers (`raul-addons.json`).
#[derive(Debug, Default, Serialize, Deserialize)]
struct Remembered {
    #[serde(default)]
    schema_version: u32,
    /// Add-ons raul gave this Profile.
    #[serde(default)]
    given: Vec<String>,
    /// Add-ons raul turned off because the trigger plugin went away; they come back with it.
    #[serde(default)]
    paused: Vec<String>,
}

impl Remembered {
    fn read(dir: &Path) -> Remembered {
        std::fs::read(dir.join(PROFILE_FILE))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn write(&self, dir: &Path) -> Result<(), String> {
        let mut text = serde_json::to_string_pretty(&Remembered {
            schema_version: 1,
            given: self.given.clone(),
            paused: self.paused.clone(),
        })
        .expect("serialises");
        text.push('\n');
        std::fs::write(dir.join(PROFILE_FILE), text)
            .map_err(|e| format!("{PROFILE_FILE} could not be written: {e}"))
    }

    fn set(list: &mut Vec<String>, id: &str, on: bool) {
        list.retain(|x| x != id);
        if on {
            list.push(id.to_owned());
        }
    }
}

/// What the marker of a made Mod says.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Marker {
    addon: String,
    version: u32,
    source_digest: String,
}

fn read_marker(dir: &Path) -> Option<Marker> {
    let meta = MetaIni::parse(&std::fs::read(find_ci(dir, "meta.ini")?).ok()?);
    if !meta.get(SECTION, "generated")?.eq_ignore_ascii_case("true") {
        return None;
    }
    Some(Marker {
        addon: meta.get_string(SECTION, "addon")?,
        version: meta.get(SECTION, "version")?.parse().ok()?,
        source_digest: meta.get_string(SECTION, "sourceDigest")?,
    })
}

fn meta_ini(previous: Option<&[u8]>, addon: &Addon, marker: &Marker) -> Vec<u8> {
    let mut meta = MetaIni::parse(previous.unwrap_or_default());
    meta.set(SECTION, "generated", "true");
    meta.set(SECTION, "addon", &quote_value(&marker.addon));
    meta.set(SECTION, "version", &marker.version.to_string());
    meta.set(SECTION, "sourceDigest", &quote_value(&marker.source_digest));
    if previous.is_none() {
        let note = format!(
            "Made by raul for {}: {} Turn it off here to keep the mod's own look; raul then leaves it off.",
            addon.trigger_plugin, addon.summary
        );
        meta.set("General", "notes", &quote_value(&note));
    }
    meta.to_bytes()
}

/// Whether the Mod folder `name` of `instance` was made by raul as an add-on.
#[must_use]
pub fn is_addon_mod(instance: &NvmodInstance, name: &str) -> bool {
    find_ci(&instance.root.join("mods"), name).is_some_and(|d| read_marker(&d).is_some())
}

/// The Mod's place in the Profile: `None` when it does not list it, else whether it enables it.
fn listed(instance: &NvmodInstance, profile: &str, name: &str) -> Option<bool> {
    nvmod_modinstall::mods::list(instance, profile)
        .into_iter()
        .find(|r| r.kind == "mod" && r.name.eq_ignore_ascii_case(name))
        .map(|r| r.state == "enabled")
}

/// Whether `plugin` is active in the Profile's Load order.
fn active(instance: &NvmodInstance, profile: &str, plugin: &str) -> bool {
    nvmod_loadorder::read_slots(&instance.profile_dir(profile)).is_ok_and(|slots| {
        slots
            .iter()
            .any(|s| s.active && s.name.eq_ignore_ascii_case(plugin))
    })
}

/// The file of `plugin` the game would load and the Mod it comes from: the highest-priority enabled Mod that holds it
/// (not the add-on's own Mod), else the base copy's `Data` folder (`None` for the Mod).
fn trigger_file(
    instance: &NvmodInstance,
    profile: &str,
    plugin: &str,
    own: &str,
) -> Option<(PathBuf, Option<String>)> {
    for row in nvmod_modinstall::mods::list(instance, profile) {
        if row.kind != "mod" || row.state != "enabled" || row.name.eq_ignore_ascii_case(own) {
            continue;
        }
        if let Some(file) = find_ci(&instance.mod_dir(&row.name), plugin).filter(|p| p.is_file()) {
            return Some((file, Some(row.name)));
        }
    }
    find_ci(&instance.config.base, "Data")
        .and_then(|data| find_ci(&data, plugin))
        .filter(|p| p.is_file())
        .map(|p| (p, None))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// The files of `addon` made from `source` (the trigger plugin's bytes) into the folder `dir`.
///
/// # Errors
///
/// The trigger plugin cannot be read as a plugin, has none of the records, or a file cannot be written.
pub fn make_files(addon: &Addon, source: &[u8], dir: &Path) -> Result<(), String> {
    let made = plugin::override_plugin(
        source,
        &addon.trigger_plugin,
        &addon.records,
        "raul",
        &addon.summary,
    )?;
    let write = |rel: &Path, bytes: &[u8]| -> Result<(), String> {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    };
    write(Path::new(&addon.plugin), &made.bytes)?;
    for mesh in &addon.meshes {
        let rel: PathBuf = std::iter::once("meshes")
            .chain(mesh.path.split(['\\', '/']))
            .collect();
        write(&rel, &nif::glow_mesh(&mesh.glow_sphere, "raul"))?;
    }
    Ok(())
}

fn mods_error(e: ModsError) -> String {
    match e {
        ModsError::Refused(w) | ModsError::Failed(w) => w,
    }
}

fn plugins_error(e: PluginsError) -> String {
    match e {
        PluginsError::Refused(w) | PluginsError::Failed(w) => w,
    }
}

/// Install the made files as the add-on's Mod, replacing an older copy in place (its line and `meta.ini` stay),
/// and write the marker.
fn install(
    instance: &NvmodInstance,
    profile: &str,
    addon: &Addon,
    source: &[u8],
    marker: &Marker,
) -> Result<(), String> {
    let work = instance
        .root
        .join(".raul")
        .join("addons")
        .join(format!("{}.making", addon.id));
    let _ = std::fs::remove_dir_all(&work);
    let result = (|| {
        let folder = work.join(&addon.name);
        make_files(addon, source, &folder)?;
        let previous = find_ci(&instance.root.join("mods"), &addon.name)
            .and_then(|d| std::fs::read(d.join("meta.ini")).ok());
        std::fs::write(
            folder.join("meta.ini"),
            meta_ini(previous.as_deref(), addon, marker),
        )
        .map_err(|e| format!("meta.ini: {e}"))?;
        let request = Request {
            instance,
            profile,
            archive: &folder,
            name: Some(&addon.name),
            dry_run: false,
            answers: None,
            answers_inline: None,
            limits: Limits::default(),
            on_conflict: OnConflict::Replace,
        };
        nvmod_modinstall::install(&request).map_err(|e| match e {
            InstallError::Refused(w) | InstallError::Failed(w) => w,
            InstallError::Questions(_) => {
                "the add-on's folder asked installer questions".to_owned()
            }
        })?;
        // A replaced Mod keeps its old meta.ini: the marker is written into it.
        let dir = find_ci(&instance.root.join("mods"), &addon.name)
            .ok_or("the add-on's Mod is not in the Instance after installing it")?;
        let kept = std::fs::read(dir.join("meta.ini")).ok();
        std::fs::write(
            dir.join("meta.ini"),
            meta_ini(kept.as_deref().or(previous.as_deref()), addon, marker),
        )
        .map_err(|e| format!("meta.ini: {e}"))
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

/// Put a newly added Mod right above the Mod that holds the trigger, and its plugin active right after the trigger.
fn place(
    instance: &NvmodInstance,
    profile: &str,
    addon: &Addon,
    provider: Option<&str>,
    changes: &mut Vec<String>,
) -> Result<(), String> {
    if let Some(provider) = provider {
        nvmod_modinstall::mods::apply(
            instance,
            profile,
            &Change::Move {
                name: addon.name.clone(),
                to: Place::Before(provider.to_owned()),
            },
            false,
        )
        .map_err(mods_error)?;
    }
    for edit in [
        Edit::Enable {
            names: vec![addon.plugin.clone()],
            with_masters: false,
        },
        Edit::Move {
            name: addon.plugin.clone(),
            to: PluginPlace::After(addon.trigger_plugin.clone()),
        },
    ] {
        nvmod_plugins::apply(instance, profile, &edit, false).map_err(plugins_error)?;
    }
    changes.push(format!(
        "added the Mod `{}`{} and its plugin `{}` right after `{}`",
        addon.name,
        provider.map_or_else(String::new, |p| format!(" above `{p}`")),
        addon.plugin,
        addon.trigger_plugin
    ));
    Ok(())
}

/// One add-on in one Profile: what it is, what it should be, and, unless `dry_run`, the change.
#[allow(clippy::too_many_lines)] // the cases in order: not needed, turned off, up to date, made
fn one(instance: &NvmodInstance, profile: &str, addon: &Addon, dry_run: bool) -> AddonState {
    let dir = instance.profile_dir(profile);
    let mut remembered = Remembered::read(&dir);
    let given = remembered.given.contains(&addon.id);
    let paused = remembered.paused.contains(&addon.id);
    let mut state = AddonState {
        id: addon.id.clone(),
        version: addon.version,
        name: addon.name.clone(),
        plugin: addon.plugin.clone(),
        trigger_plugin: addon.trigger_plugin.clone(),
        summary: addon.summary.clone(),
        status: Status::NotNeeded,
        enabled: listed(instance, profile, &addon.name),
        changes: Vec::new(),
        error: None,
    };
    let fail = |mut state: AddonState, why: String| {
        state.status = Status::Failed;
        state.error = Some(why);
        state.enabled = listed(instance, profile, &addon.name);
        state
    };
    let trigger = active(instance, profile, &addon.trigger_plugin)
        .then(|| trigger_file(instance, profile, &addon.trigger_plugin, &addon.name))
        .flatten();
    let Some((file, provider)) = trigger else {
        // Without the trigger the add-on's plugin would miss its master: an enabled Mod raul gave is turned off, and
        // comes back with the trigger.
        if state.enabled == Some(true) && given {
            if dry_run {
                state.status = Status::OutOfDate;
                return state;
            }
            let off = nvmod_modinstall::mods::apply(
                instance,
                profile,
                &Change::Disable(vec![addon.name.clone()]),
                false,
            )
            .map_err(mods_error)
            .and_then(|_| {
                Remembered::set(&mut remembered.paused, &addon.id, true);
                remembered.write(&dir)
            });
            if let Err(why) = off {
                return fail(state, why);
            }
            state.changes.push(format!(
                "turned off the Mod `{}`: `{}` is not active any more; it comes back with it",
                addon.name, addon.trigger_plugin
            ));
            state.enabled = Some(false);
        }
        return state;
    };
    // The player's choice wins: a Mod that is off (not by raul), or one raul gave that is gone, stays so.
    if !paused && (state.enabled == Some(false) || (given && state.enabled.is_none())) {
        state.status = Status::TurnedOff;
        return state;
    }
    let source = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => return fail(state, format!("{}: {e}", file.display())),
    };
    let marker = Marker {
        addon: addon.id.clone(),
        version: addon.version,
        source_digest: format!("sha256:{}", hex(&Sha256::digest(&source))),
    };
    let folder = find_ci(&instance.root.join("mods"), &addon.name);
    let current = folder.as_deref().and_then(read_marker).as_ref() == Some(&marker);
    if folder.is_some() && folder.as_deref().and_then(read_marker).is_none() {
        return fail(
            state,
            format!(
                "a Mod `{}` that raul did not make is in the Instance; rename it to let raul add its own",
                addon.name
            ),
        );
    }
    let needs_line = state.enabled.is_none();
    let needs_on = paused && state.enabled == Some(false);
    if current && !needs_line && !needs_on {
        state.status = Status::UpToDate;
        return state;
    }
    if dry_run {
        state.status = Status::OutOfDate;
        return state;
    }
    let result = (|| -> Result<Status, String> {
        let mut status = Status::UpToDate;
        if !current || needs_line {
            install(instance, profile, addon, &source, &marker)?;
            if !current && folder.is_some() {
                state.changes.push(format!(
                    "made the Mod `{}` again from this `{}`",
                    addon.name, addon.trigger_plugin
                ));
                status = Status::Updated;
            }
        }
        if needs_line {
            place(
                instance,
                profile,
                addon,
                provider.as_deref(),
                &mut state.changes,
            )?;
            status = Status::Added;
        }
        if needs_on {
            nvmod_modinstall::mods::apply(
                instance,
                profile,
                &Change::Enable(vec![addon.name.clone()]),
                false,
            )
            .map_err(mods_error)?;
            state.changes.push(format!(
                "turned the Mod `{}` on again: `{}` is active again",
                addon.name, addon.trigger_plugin
            ));
            if status == Status::UpToDate {
                status = Status::Updated;
            }
        }
        Remembered::set(&mut remembered.given, &addon.id, true);
        Remembered::set(&mut remembered.paused, &addon.id, false);
        remembered.write(&dir)?;
        Ok(status)
    })();
    match result {
        Ok(status) => {
            state.status = status;
            state.enabled = listed(instance, profile, &addon.name);
            state
        }
        Err(why) => fail(state, why),
    }
}

/// The add-ons of `profile`, and with `dry_run` false the changes that keep them as the list says. Each add-on is
/// handled on its own: one that fails is reported and the others go on.
#[must_use]
pub fn apply(instance: &NvmodInstance, profile: &str, dry_run: bool) -> AddonsReport {
    apply_list(instance, profile, &known(), dry_run)
}

/// [`apply`] with the add-ons `addons` instead of raul's own list.
#[must_use]
pub fn apply_list(
    instance: &NvmodInstance,
    profile: &str,
    addons: &[Addon],
    dry_run: bool,
) -> AddonsReport {
    AddonsReport {
        schema_version: SCHEMA_VERSION,
        contract: CONTRACT.to_owned(),
        instance: instance.root.clone(),
        profile: profile.to_owned(),
        dry_run,
        addons: addons
            .iter()
            .map(|a| one(instance, profile, a, dry_run))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_testkit::{covers, ids};

    #[test]
    fn the_list_is_valid_and_every_add_on_is_complete() {
        covers(&[ids::mods::addons::LIST_COMPLETE]);
        let list = known();
        assert_ne!(list.len(), 0);
        let mut ids: Vec<&str> = list.iter().map(|a| a.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), list.len(), "ids are unique");
        for a in &list {
            assert!(a.version >= 1);
            assert!(
                std::path::Path::new(&a.plugin)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("esp")),
                "{a:?}"
            );
            assert!(!a.records.is_empty() && !a.meshes.is_empty(), "{a:?}");
            for r in &a.records {
                assert_eq!(r.record_type.len(), 4, "{r:?}");
                // Every new model is a mesh the add-on writes.
                assert!(
                    a.meshes
                        .iter()
                        .any(|m| plugin::model_key(&m.path) == plugin::model_key(&r.new_model)),
                    "{r:?}"
                );
            }
        }
    }

    #[test]
    fn the_small_lamp_glow_is_the_approved_size() {
        covers(&[ids::mods::addons::LAMP_GLOW_SIZE]);
        let glow = known()
            .into_iter()
            .find(|a| a.id == "small-lamp-glow")
            .unwrap();
        assert_eq!(glow.trigger_plugin, "Street Lights TTW.esm");
        let sphere = &glow.meshes[0].glow_sphere;
        assert!((32.0..=36.0).contains(&sphere.radius), "{sphere:?}");
        assert!(sphere.alpha <= 0.3, "dim: {sphere:?}");
        assert_eq!(glow.records[0].bounds, Some(36));
    }

    #[test]
    fn the_files_are_the_plugin_and_the_meshes_under_their_paths() {
        let dir = nvmod_testkit::TempDir::new("raul-addon-files");
        let glow = known().remove(0);
        let source = plugin::synthetic_plugin(
            &["FalloutNV.esm"],
            &[(
                "ACTI",
                0x0100_0801,
                "Glow",
                "Effects\\Ambient\\FXGlowSimpFillRnd.nif",
            )],
        );
        make_files(&glow, &source, dir.path()).unwrap();
        assert!(dir.path().join("Raul Small Lamp Glow.esp").is_file());
        assert!(
            dir.path()
                .join("meshes/raul/effects/raulsmallglow01.nif")
                .is_file()
        );
    }

    #[test]
    fn a_marker_is_written_and_read_back_and_user_keys_stay() {
        let glow = known().remove(0);
        let marker = Marker {
            addon: glow.id.clone(),
            version: 1,
            source_digest: "sha256:00".to_owned(),
        };
        let user = b"[General]\r\ncategory=\"3,\"\r\nnotes=mine\r\n";
        let bytes = meta_ini(Some(user), &glow, &marker);
        let dir = nvmod_testkit::TempDir::new("raul-addon-marker");
        std::fs::write(dir.path().join("meta.ini"), &bytes).unwrap();
        assert_eq!(read_marker(dir.path()), Some(marker.clone()));
        let text = String::from_utf8(bytes).unwrap();
        assert!(
            text.contains("notes=mine") && text.contains("category=\"3,\""),
            "{text}"
        );
        let fresh = String::from_utf8(meta_ini(None, &glow, &marker)).unwrap();
        assert!(fresh.contains("Street Lights TTW.esm"), "{fresh}");
    }
}
