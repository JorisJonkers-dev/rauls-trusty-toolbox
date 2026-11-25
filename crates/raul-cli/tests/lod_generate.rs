// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate` with lodwright linked into `raul`, on a synthetic world built with lodwright's own test kit:
//! success and a second run that is `up_to_date`, a replaced Mod, lodwright's refusals as exit code 4, a stop during
//! the generation that the next run resumes from, the dry run and bad inputs, and that nvmod never receives the
//! command. Nothing is started but the built `raul` binary, as a black box.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use raul_testkit::{covers, ids};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

use lodwright_api::CancelToken;
use lodwright_testkit::land::Heights;
use lodwright_testkit::world::{SyntheticWorld, world_plugin};
use nvmod_testkit::{BaseBuilder, TempDir, snapshot};
use raul_drivers::events::Event;
use raul_drivers::lod::{self, LodRequest};
use serde_json::Value;

const MOD: &str = "Generated LOD (Default)";
const PREVIOUS: &str = "Generated LOD (Default) previous";
const LEGACY: &str = "LOD (lodwright)";

/// The lodwright-rs version this build links, from the Component's own `Cargo.toml`, without its `v`.
fn shipped_lodwright_version() -> String {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../components/lodwright-rs/Cargo.toml"),
    )
    .expect("read components/lodwright-rs/Cargo.toml");
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]" || line == "[workspace.package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(value) = line.strip_prefix("version")
            && let Some(value) = value.trim().strip_prefix('=')
        {
            return value
                .trim()
                .trim_matches('"')
                .trim_start_matches('v')
                .to_owned();
        }
    }
    panic!("components/lodwright-rs/Cargo.toml states no version");
}

struct Fixture {
    root: TempDir,
}

impl Fixture {
    /// An Instance whose game master is a synthetic world of four cells with terrain.
    fn new(label: &str) -> Fixture {
        Fixture::with_world(label, "Island")
    }

    /// The same Instance with the world named `world`.
    fn with_world(label: &str, world: &str) -> Fixture {
        Fixture::with_world_and_plugin(label, world, None)
    }

    /// The same Instance with the world named `world` and, when given, one more plugin (empty, on the game master)
    /// of that name in the game's Data folder, active at the end of the Default Profile's Load order.
    fn with_world_and_plugin(label: &str, world: &str, plugin: Option<&str>) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
        let world = SyntheticWorld::new(0xB00, world)
            .cell(2, 2, Heights::flat(16.0))
            .cell(3, 2, Heights::flat(24.0))
            .cell(2, 3, Heights::flat(32.0))
            .cell(3, 3, Heights::flat(8.0));
        std::fs::write(
            f.path("FalloutNV/Data/FalloutNV.esm"),
            world_plugin(&[], &world, false),
        )
        .unwrap();
        if let Some(name) = plugin {
            let bytes = lodwright_testkit::plugin::PluginBuilder::new()
                .master("FalloutNV.esm")
                .build();
            std::fs::write(f.path("FalloutNV/Data").join(name), bytes).unwrap();
        }
        let (base, inst) = (f.path("FalloutNV"), f.instance());
        for args in [
            &["init", "--base", s(&base), "--instance", s(&inst)][..],
            &["profile", "create", "--instance", s(&inst)],
            &["stage", "--instance", s(&inst), "--copy"],
        ] {
            let out = f.raul(args);
            assert!(out.status.success(), "{args:?}: {out:?}");
        }
        if let Some(name) = plugin {
            for file in ["plugins.txt", "loadorder.txt"] {
                let path = f.instance().join("profiles/Default").join(file);
                let mut text = std::fs::read_to_string(&path).unwrap_or_default();
                text.push_str(name);
                text.push_str("\r\n");
                std::fs::write(path, text).unwrap();
            }
        }
        f
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn instance(&self) -> PathBuf {
        self.path("instance")
    }

    fn out_dir(&self) -> PathBuf {
        self.instance().join(".raul/lod/Default/out")
    }

    /// Where lodwright builds a run: the sibling of the output folder, swapped into place only on success.
    fn staging_dir(&self) -> PathBuf {
        self.instance()
            .join(".raul/lod/Default/out.lodwright-staging")
    }

    fn lod_mod(&self) -> PathBuf {
        self.instance().join("mods").join(MOD)
    }

    /// Replace the Profile's `raul-lod.toml` with `text`.
    fn settings(&self, text: &str) {
        std::fs::write(self.instance().join("profiles/Default/raul-lod.toml"), text).unwrap();
    }

    fn raul(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"))
            .output()
            .expect("raul runs")
    }

    /// Runs `raul lod generate --json` with `extra` flags; returns the exit code, the result and `finished`.
    fn generate(&self, extra: &[&str]) -> (Option<i32>, Value, Value) {
        let inst = self.instance();
        let mut args = vec!["lod", "generate", "--json", "--instance", s(&inst)];
        args.extend_from_slice(extra);
        let out = self.raul(&args);
        let (result, finished) = check_stream(&out);
        (out.status.code(), result, finished)
    }

    /// The same run in process, with `cancel` and the events it emits.
    fn in_process(&self, cancel: &CancelToken, emit: &mut dyn FnMut(Event)) -> lod::LodOutcome {
        let req = LodRequest {
            instance: self.instance(),
            profile: None,
            worlds: Vec::new(),
            dry_run: false,
            allow_in_repo: false,
            timeout: Duration::from_secs(600),
        };
        lod::generate_with_cancel(&req, cancel, emit)
    }

    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn validator(schema: &str) -> jsonschema::Validator {
    jsonschema::validator_for(&serde_json::from_str(schema).unwrap()).unwrap()
}

/// Every line valid, numbered from 1, the result a valid `raul-lod-result`, the last line `finished` with the
/// process's exit code. Returns the result and the finished line.
fn check_stream(out: &Output) -> (Value, Value) {
    let lines: Vec<Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("{e}: {l}\n{}", String::from_utf8_lossy(&out.stderr)))
        })
        .collect();
    let (events, lod) = (
        validator(&raul_drivers::events::schema()),
        validator(&raul_drivers::lod::schema()),
    );
    for (i, l) in lines.iter().enumerate() {
        let errors: Vec<String> = events.iter_errors(l).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{errors:?} {l}");
        assert_eq!(l["seq"], i as u64 + 1);
        assert_eq!(l["command"], "lod");
    }
    let finished = lines.last().expect("a finished line").clone();
    assert_eq!(finished["event"], "finished");
    assert_eq!(
        finished["exit_code"].as_i64(),
        out.status.code().map(i64::from)
    );
    let result = lines[lines.len() - 2]["result"].clone();
    let errors: Vec<String> = lod.iter_errors(&result).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?} {result}");
    assert_eq!(result["status"], finished["status"]);
    (result, finished)
}

/// The files under `dir`, counted as the Mod counts them.
fn files_under(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => files_under(&e.path()),
            _ => 1,
        })
        .sum()
}

fn mod_names(f: &Fixture) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(f.instance().join("mods"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn enabled_lines(f: &Fixture) -> usize {
    std::fs::read_to_string(f.instance().join("profiles/Default/modlist.txt"))
        .unwrap()
        .lines()
        .filter(|l| *l == format!("+{MOD}"))
        .count()
}

#[test]
fn success_keeps_one_enabled_lod_mod_and_a_second_run_is_up_to_date() {
    covers(&[ids::lod::generate::ONE_MOD]);
    let f = Fixture::new("raul-lodgen-ok");
    // Packing is the default; this test is about the loose form.
    f.settings("[output]\npack = false\n");
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["status"], "success", "{result}");
    assert_eq!(result["mod"], MOD);
    assert_eq!(result["up_to_date"], false);
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
    assert_eq!(
        result["tool"]["version"],
        shipped_lodwright_version().as_str()
    );
    assert_eq!(finished["changed"], true);
    assert!(
        f.lod_mod()
            .join("lodsettings/island.dlodsettings")
            .is_file()
    );
    assert!(f.lod_mod().join("meshes/landscape/lod/island").is_dir());
    assert!(!f.lod_mod().join("manifest.json").exists());
    // nvmod adds the Mod's `meta.ini`, which the result does not count.
    assert!(f.lod_mod().join("meta.ini").is_file());
    assert_eq!(
        result["files"].as_u64(),
        Some(files_under(&f.lod_mod()) - 1),
        "the Mod holds the files the result counts"
    );
    assert_eq!(enabled_lines(&f), 1);
    // The request is the one `raul lod plan` shows, and it passes lodwright's schema.
    let inst = f.instance();
    let out = f.raul(&["lod", "plan", "--json", "--instance", s(&inst)]);
    let plan: Value = serde_json::from_slice(&out.stdout).unwrap();
    lod::check_request(&plan).unwrap();

    let before = f.state();
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(0));
    assert_eq!(result["up_to_date"], true, "{result}");
    assert_eq!(result["lodwright"]["code"], "up_to_date");
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before, "an up-to-date run changed the Instance");
}

#[test]
fn a_new_output_replaces_the_lod_mod_and_leaves_one() {
    covers(&[ids::lod::generate::REPLACES]);
    let f = Fixture::new("raul-lodgen-replace");
    assert_eq!(f.generate(&[]).0, Some(0));
    // The Profile's settings change the request: lodwright writes new files.
    f.settings("[output]\nwater = false\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], false);
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
    assert_eq!(mod_names(&f), [MOD], "one Mod");
    assert_eq!(enabled_lines(&f), 1);
    // A backup an interrupted run left behind is removed when the next run replaces the Mod.
    let out = f.raul(&["mods", "create", PREVIOUS, "--instance", s(&f.instance())]);
    assert!(out.status.success(), "{out:?}");
    // It is a renamed-aside copy of the LOD Mod, so it carries the marker.
    std::fs::copy(
        f.lod_mod().join("meta.ini"),
        f.instance().join("mods").join(PREVIOUS).join("meta.ini"),
    )
    .unwrap();
    f.settings("[output]\nwater = true\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(mod_names(&f), [MOD], "the backup is gone");
}

/// The lines of the Profile's `modlist.txt`, `+Name` or `-Name`, highest priority first.
fn modlist(f: &Fixture, profile: &str) -> Vec<String> {
    std::fs::read_to_string(f.instance().join(format!("profiles/{profile}/modlist.txt")))
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Runs `raul <args> --instance <instance>` and expects it to succeed.
fn raul_ok(f: &Fixture, args: &[&str]) {
    let inst = f.instance();
    let mut all = args.to_vec();
    all.extend_from_slice(&["--instance", s(&inst)]);
    let out = f.raul(&all);
    assert!(out.status.success(), "{all:?}: {out:?}");
}

#[test]
fn the_lod_mod_is_named_for_its_profile_and_marked_as_generated() {
    covers(&[ids::lod::generate::NAMED_AND_MARKED]);
    let f = Fixture::new("raul-lodgen-marker");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["mod"], MOD);
    assert_eq!(lod::mod_name("Default"), MOD);
    let marker = lod::read_marker(&f.lod_mod()).expect("the Mod carries the marker");
    assert_eq!(marker.profile, "Default");
    assert_eq!(marker.lodwright_version, shipped_lodwright_version());
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(f.out_dir().join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        Some(marker.request_digest.as_str()),
        manifest["request_digest"].as_str(),
        "the marker names the request the LOD was built from"
    );
    assert!(
        marker.request_digest.starts_with("sha256:"),
        "{}",
        marker.request_digest
    );
    assert!(lod::is_generated(&f.instance().join("mods"), MOD));
}

#[test]
fn a_new_run_keeps_the_place_the_state_and_the_meta_ini_keys_of_the_lod_mod() {
    let f = Fixture::new("raul-lodgen-keep");
    assert_eq!(f.generate(&[]).0, Some(0));
    raul_ok(&f, &["mods", "create", "Alpha", "--top"]);
    raul_ok(&f, &["mods", "create", "Beta", "--bottom"]);
    raul_ok(&f, &["mods", "reorder", MOD, "--position", "2"]);
    raul_ok(&f, &["mods", "disable", MOD]);
    let meta = f.lod_mod().join("meta.ini");
    let mut text = std::fs::read_to_string(&meta).unwrap();
    text.push_str("\n[Custom]\nkey=1\n");
    let text = text.replacen("[General]", "[General]\ncategory=\"5,\"", 1);
    std::fs::write(&meta, &text).unwrap();
    let before = modlist(&f, "Default");
    assert_eq!(before[1], format!("-{MOD}"), "{before:?}");

    f.settings("[output]\nwater = false\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
    assert_eq!(
        modlist(&f, "Default"),
        before,
        "the LOD Mod keeps its place and stays disabled"
    );
    let text = std::fs::read_to_string(&meta).unwrap();
    assert!(text.contains("category=\"5,\""), "{text}");
    assert!(text.contains("[Custom]"), "{text}");
    assert!(lod::read_marker(&f.lod_mod()).is_some(), "{text}");
    assert_eq!(mod_names(&f), ["Alpha", "Beta", MOD], "no backup is left");

    // A disabled LOD Mod that is current is left as it is.
    let state = f.state();
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], true, "{result}");
    assert_eq!(f.state(), state);
}

#[test]
fn each_profile_has_its_own_output_folder_and_the_shared_one_becomes_the_first_profiles() {
    covers(&[ids::lod::generate::PER_PROFILE]);
    let f = Fixture::new("raul-lodgen-folders");
    assert_eq!(f.generate(&[]).0, Some(0));
    // The folder of a run before each Profile had its own: the next run takes it over and finds its LOD current.
    let shared = f.instance().join(".raul/lod/out");
    std::fs::rename(f.out_dir(), &shared).unwrap();
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["lodwright"]["code"], "up_to_date", "{result}");
    assert!(!shared.exists(), "the shared folder was moved");
    assert!(f.out_dir().join("manifest.json").is_file());
}

#[test]
fn a_removed_lod_mod_is_made_again_at_the_top() {
    let f = Fixture::new("raul-lodgen-removed");
    assert_eq!(f.generate(&[]).0, Some(0));
    raul_ok(&f, &["mods", "create", "Alpha", "--top"]);
    raul_ok(&f, &["mods", "remove", MOD]);
    assert!(!f.lod_mod().exists());
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], false, "{result}");
    assert_eq!(finished["changed"], true);
    assert!(f.lod_mod().is_dir());
    assert_eq!(modlist(&f, "Default")[0], format!("+{MOD}"));
}

/// Turns the Profile's LOD Mod into one an earlier raul made: the old shared name and a `meta.ini` without the
/// marker.
fn make_legacy(f: &Fixture) {
    raul_ok(f, &["mods", "rename", MOD, LEGACY]);
    std::fs::write(
        f.instance().join("mods").join(LEGACY).join("meta.ini"),
        "[General]\ngameName=newvegas\n",
    )
    .unwrap();
}

#[test]
fn the_old_shared_lod_mod_becomes_the_profile_s_own_in_its_place() {
    let f = Fixture::new("raul-lodgen-legacy");
    assert_eq!(f.generate(&[]).0, Some(0));
    raul_ok(&f, &["mods", "create", "Alpha", "--top"]);
    make_legacy(&f);
    raul_ok(&f, &["mods", "disable", LEGACY]);
    assert!(lod::is_generated(&f.instance().join("mods"), LEGACY));
    assert_eq!(
        modlist(&f, "Default"),
        ["+Alpha".to_owned(), format!("-{LEGACY}")]
    );

    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["mod"], MOD);
    assert_eq!(
        modlist(&f, "Default"),
        ["+Alpha".to_owned(), format!("-{MOD}")],
        "the new Mod takes the old one's place and state"
    );
    assert_eq!(mod_names(&f), ["Alpha", MOD], "the old Mod is removed");
}

#[test]
fn the_old_shared_lod_mod_stays_for_another_profile_that_lists_it() {
    let f = Fixture::new("raul-lodgen-legacy-shared");
    assert_eq!(f.generate(&[]).0, Some(0));
    make_legacy(&f);
    let other = f.instance().join("profiles/Other");
    std::fs::create_dir_all(&other).unwrap();
    for file in ["plugins.txt", "loadorder.txt"] {
        std::fs::copy(
            f.instance().join("profiles/Default").join(file),
            other.join(file),
        )
        .unwrap();
    }
    std::fs::write(other.join("modlist.txt"), format!("+{LEGACY}\n")).unwrap();

    let (code, result, _) = f.generate(&["--profile", "Default"]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(
        modlist(&f, "Default"),
        [format!("+{MOD}"), format!("-{LEGACY}")],
        "the old Mod is turned off here, not removed"
    );
    assert_eq!(modlist(&f, "Other"), [format!("+{LEGACY}")]);
    assert_eq!(mod_names(&f), [MOD, LEGACY]);
}

/// `raul lod mods --json` for the Fixture's Instance, checked against its schema.
fn lod_mods(f: &Fixture) -> Value {
    let inst = f.instance();
    let out = f.raul(&["lod", "mods", "--json", "--instance", s(&inst)]);
    assert!(out.status.success(), "{out:?}");
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    let errors: Vec<String> = validator(&lod::mods_schema())
        .iter_errors(&doc)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?} {doc}");
    doc
}

#[test]
fn lod_mods_lists_the_generated_mods_of_the_profile_and_nothing_else() {
    let f = Fixture::new("raul-lodgen-list");
    let doc = lod_mods(&f);
    assert_eq!(doc["own_name"], MOD);
    assert_eq!(doc["mods"], serde_json::json!([]));
    assert_eq!(f.generate(&[]).0, Some(0));
    raul_ok(&f, &["mods", "create", "Alpha", "--top"]);
    raul_ok(&f, &["mods", "disable", MOD]);
    let doc = lod_mods(&f);
    let mods = doc["mods"].as_array().unwrap();
    assert_eq!(mods.len(), 1, "{doc}");
    assert_eq!(mods[0]["name"], MOD);
    assert_eq!(mods[0]["position"], 2);
    assert_eq!(mods[0]["enabled"], false);
    assert_eq!(mods[0]["own"], true);
    assert_eq!(mods[0]["legacy"], false);
    assert_eq!(mods[0]["profile"], "Default");
    assert!(
        mods[0]["generated_at"]
            .as_u64()
            .is_some_and(|t| t > 1_700_000_000),
        "{doc}"
    );
    let files = mods[0]["files"].as_u64().expect("the file count");
    let listing = f.raul(&["lod", "mods", "--instance", s(&f.instance())]);
    let listing = String::from_utf8_lossy(&listing.stdout).into_owned();
    assert!(listing.contains(&format!(", {files} file")), "{listing}");
    assert!(listing.contains(", generated 20"), "{listing}");
    assert_eq!(
        mods[0]["lodwright_version"],
        shipped_lodwright_version().as_str()
    );
    make_legacy(&f);
    let doc = lod_mods(&f);
    assert_eq!(doc["mods"][0]["name"], LEGACY);
    assert_eq!(doc["mods"][0]["legacy"], true);
    assert_eq!(doc["mods"][0]["own"], false);
    assert_eq!(doc["mods"][0]["profile"], Value::Null);
    let out = f.raul(&["lod", "mods", "--instance", s(&f.path("nowhere"))]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        } else if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
        {
            return Some(path);
        }
    }
    None
}

/// The staleness `raul lod mods` reports for the Profile's own LOD Mod.
fn stale_of(f: &Fixture) -> (String, String) {
    let doc = lod_mods(f);
    let stale = &doc["mods"][0]["stale"];
    (
        stale["state"].as_str().unwrap_or_default().to_owned(),
        stale["reason"].as_str().unwrap_or_default().to_owned(),
    )
}

#[test]
fn a_run_records_what_it_was_built_from_and_a_settings_change_marks_the_lod_out_of_date_until_it_runs_again()
 {
    covers(&[ids::lod::stale::SETTINGS_CHANGE]);
    let f = Fixture::new("raul-lodgen-stale");
    assert_eq!(lod_mods(&f)["mods"], serde_json::json!([]));
    assert_eq!(f.generate(&[]).0, Some(0));
    assert!(
        f.instance()
            .join(".raul/lod/Default/fingerprint.json")
            .is_file()
    );
    assert_eq!(stale_of(&f).0, "current");
    // A Mod without plugins is not what the LOD is built from.
    raul_ok(&f, &["mods", "create", "Alpha", "--top"]);
    assert_eq!(stale_of(&f).0, "current");
    // Any setting, then the same run again.
    f.settings("[output]\nwater = false\n");
    let (state, reason) = stale_of(&f);
    assert_eq!(state, "must", "{reason}");
    assert!(reason.contains("settings"), "{reason}");
    assert_eq!(f.generate(&[]).0, Some(0));
    assert_eq!(stale_of(&f).0, "current");
    // Restore defaults and its undo are settings changes too.
    f.settings("");
    assert_eq!(stale_of(&f).0, "must");
    assert_eq!(f.generate(&[]).0, Some(0));
    assert_eq!(stale_of(&f).0, "current");
}

/// The names of the files of the LOD Mod that are not its `meta.ini`, relative to it, sorted.
fn mod_files(f: &Fixture) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                out.push(
                    path.strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut out = Vec::new();
    walk(&f.lod_mod(), &f.lod_mod(), &mut out);
    out.retain(|n| n != "meta.ini");
    out.sort();
    out
}

#[test]
fn packing_gives_a_mod_of_one_archive_and_its_stub_and_switching_it_either_way_marks_the_lod_out_of_date()
 {
    covers(&[ids::lod::generate::PACKING]);
    let f = Fixture::new("raul-lodgen-pack");
    // A Profile that has a loose generated LOD Mod from before packing was the default: its settings file names no
    // packing and its Mod keeps working until the next generate, which packs.
    f.settings("[output]\npack = false\n");
    assert_eq!(f.generate(&[]).0, Some(0));
    let loose = mod_files(&f);
    assert!(
        loose.iter().any(|n| n.starts_with("meshes/landscape/lod/")),
        "{loose:?}"
    );
    assert!(
        !loose.iter().any(|n| n == "lodwright-warnings.json"),
        "the run's warnings stay out of the Mod: {loose:?}"
    );
    assert_eq!(stale_of(&f).0, "current");

    f.settings("");
    assert!(
        f.lod_mod().join("meta.ini").is_file(),
        "the loose Mod is still there"
    );
    let (state, reason) = stale_of(&f);
    assert_eq!(state, "must", "{reason}");
    assert!(reason.contains("Packing was turned on"), "{reason}");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], false, "{result}");
    assert_eq!(
        mod_files(&f),
        [
            "Lodwright LOD.bsa",
            "Lodwright LOD.esp",
            "Lodwright LOD.override"
        ],
        "the archive, its stub, its override marker and no loose file"
    );
    assert_eq!(
        std::fs::metadata(f.lod_mod().join("Lodwright LOD.override"))
            .unwrap()
            .len(),
        0,
        "the marker is empty"
    );
    assert_eq!(stale_of(&f).0, "current");
    assert_eq!(mod_names(&f), [MOD], "one Mod");
    assert_eq!(enabled_lines(&f), 1);
    // The stub is the run's own output: it is not an input of the next request, so the next run is up to date.
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], true, "{result}");
    let plugins =
        std::fs::read_to_string(f.instance().join("profiles/Default/plugins.txt")).unwrap();
    assert!(
        plugins
            .lines()
            .any(|l| l.trim_start_matches('*') == "Lodwright LOD.esp"),
        "the stub is in the Load order: {plugins}"
    );

    f.settings("[output]\npack = false\n");
    let (state, reason) = stale_of(&f);
    assert_eq!(state, "must", "{reason}");
    assert!(reason.contains("Packing was turned off"), "{reason}");
    assert_eq!(f.generate(&[]).0, Some(0));
    assert_eq!(
        mod_files(&f),
        loose,
        "loose files again and no archive, stub or marker"
    );
    let plugins =
        std::fs::read_to_string(f.instance().join("profiles/Default/plugins.txt")).unwrap();
    assert!(
        !plugins.contains("Lodwright LOD"),
        "the stub went with the packed form: {plugins}"
    );
    assert_eq!(stale_of(&f).0, "current");
}

/// The active plugins of `profile`, in Load order.
fn active_plugins(f: &Fixture, profile: &str) -> Vec<String> {
    std::fs::read_to_string(f.instance().join(format!("profiles/{profile}/plugins.txt")))
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().trim_start_matches('*').to_owned())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// The stub of a packed LOD is active and last in `profile`'s Load order, so the game registers its archive.
fn assert_stub_active_and_last(f: &Fixture, profile: &str) {
    let active = active_plugins(f, profile);
    assert_eq!(
        active.last().map(String::as_str),
        Some("Lodwright LOD.esp"),
        "the stub must be active and last: {active:?}"
    );
    assert_eq!(
        active.iter().filter(|p| *p == "Lodwright LOD.esp").count(),
        1,
        "{active:?}"
    );
}

#[test]
fn a_new_packed_lod_keeps_its_stub_active_and_last() {
    let f = Fixture::new("raul-lodgen-stub-rerun");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_stub_active_and_last(&f, "Default");
    // A changed request writes a new Mod in place of the old one; its stub has the same name.
    f.settings("[output]\nwater = false\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
    assert_stub_active_and_last(&f, "Default");
    // An up-to-date run puts back a stub that was turned off, and says it changed the Instance.
    raul_ok(&f, &["plugins", "disable", "Lodwright LOD.esp"]);
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], true, "{result}");
    assert_eq!(finished["changed"], true, "{finished}");
    assert_stub_active_and_last(&f, "Default");
    // A stub moved away from the end goes back there on the next new Mod.
    raul_ok(&f, &["plugins", "move", "Lodwright LOD.esp", "--top"]);
    f.settings("[output]\nwater = true\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_stub_active_and_last(&f, "Default");
}

#[test]
fn a_copied_profile_whose_old_lod_mod_is_off_gets_its_own_stub_active() {
    let f = Fixture::new("raul-lodgen-stub-copy");
    assert_eq!(f.generate(&[]).0, Some(0));
    raul_ok(&f, &["profile", "copy", "Default", "Copy"]);
    raul_ok(&f, &["mods", "disable", MOD, "--profile", "Copy"]);
    assert!(
        !active_plugins(&f, "Copy").contains(&"Lodwright LOD.esp".to_owned()),
        "turning the old Mod off takes its stub out"
    );
    raul_ok(&f, &["stage", "--profile", "Copy", "--copy"]);
    let (code, result, _) = f.generate(&["--profile", "Copy"]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["mod"], "Generated LOD (Copy)", "{result}");
    assert_stub_active_and_last(&f, "Copy");
    assert_stub_active_and_last(&f, "Default");
}

#[test]
fn packing_is_refused_with_a_plain_sentence_when_no_plugin_slot_is_left_for_the_stub() {
    let f = Fixture::new("raul-lodgen-slots");
    let inst = f.instance();
    // 254 more active plugins on top of the game's master: 255 in all, so the stub has no slot.
    let dir = inst.join("profiles/Default");
    let lines = (0..254)
        .map(|i| format!("Filler{i}.esp"))
        .collect::<Vec<_>>()
        .join("\r\n")
        + "\r\n";
    for name in ["plugins.txt", "loadorder.txt"] {
        let path = dir.join(name);
        let mut text = std::fs::read_to_string(&path).unwrap_or_default();
        text.push_str(&lines);
        std::fs::write(path, text).unwrap();
    }
    let out = f.raul(&["lod", "plan", "--json", "--instance", s(&inst)]);
    assert!(!out.status.success(), "{out:?}");
    let text =
        String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("no room"), "{text}");
    assert!(
        text.contains("Pack the generated LOD into one archive"),
        "{text}"
    );
    // Turned off, the same Load order plans.
    f.settings("[output]\npack = false\n");
    let out = f.raul(&["lod", "plan", "--json", "--instance", s(&inst)]);
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn a_changed_plugin_with_a_world_marks_the_lod_out_of_date() {
    let f = Fixture::new("raul-lodgen-stale-plugin");
    assert_eq!(f.generate(&[]).0, Some(0));
    let esm = find_file(&f.instance(), "falloutnv.esm").expect("a staged FalloutNV.esm");
    let mut bytes = std::fs::read(&esm).unwrap();
    bytes.extend_from_slice(&[0; 8]);
    std::fs::write(&esm, bytes).unwrap();
    let (state, reason) = stale_of(&f);
    assert_eq!(state, "must", "{reason}");
    assert!(reason.contains("falloutnv.esm"), "{reason}");
}

#[test]
fn a_mod_that_is_not_generated_lod_but_has_its_name_is_refused() {
    let f = Fixture::new("raul-lodgen-taken");
    raul_ok(&f, &["mods", "create", MOD]);
    let before = f.state();
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(2), "{result}");
    assert_eq!(result["error"]["code"], "plan_refused", "{result}");
    assert!(
        result["error"]["message"].as_str().unwrap().contains(MOD),
        "{result}"
    );
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before, "nothing changed");
}

#[test]
fn a_mod_that_is_not_generated_lod_but_has_the_backup_name_is_refused_and_kept() {
    let f = Fixture::new("raul-lodgen-taken-previous");
    let previous = format!("{MOD} previous");
    raul_ok(&f, &["mods", "create", &previous]);
    let before = f.state();
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(2), "{result}");
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap()
            .contains(&previous),
        "{result}"
    );
    assert_eq!(
        f.state(),
        before,
        "the user's folder is neither removed nor renamed"
    );
}

/// A FOMOD that needs answers in the output makes the Mod's install fail after the old Mod has been moved aside, which
/// is the add failure the rollback is for. The stray folder is not in lodwright's manifest, so lodwright keeps it.
fn fomod_in_output(f: &Fixture) {
    let dir = f.out_dir().join("fomod");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ModuleConfig.xml"), "<config/>").unwrap();
}

#[test]
fn a_failed_replacement_puts_the_old_mod_back() {
    covers(&[ids::lod::generate::FAILED_REPLACEMENT]);
    let f = Fixture::new("raul-lodgen-rollback");
    assert_eq!(f.generate(&[]).0, Some(0));
    let before = snapshot(&f.lod_mod());
    f.settings("[output]\nwater = false\n");
    fomod_in_output(&f);
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(4), "{result}");
    assert_eq!(result["error"]["code"], "add_mod_failed", "{result}");
    assert_eq!(finished["changed"], false);
    assert_eq!(snapshot(&f.lod_mod()), before, "the old Mod is as it was");
    assert_eq!(mod_names(&f), [MOD], "no backup is left");
    assert_eq!(enabled_lines(&f), 1);
}

#[test]
fn a_backup_left_alone_by_an_interrupted_run_is_put_back_when_the_new_mod_cannot_be_added() {
    let f = Fixture::new("raul-lodgen-lone-backup");
    assert_eq!(f.generate(&[]).0, Some(0));
    let before = snapshot(&f.lod_mod());
    // An earlier run stopped after it moved the LOD Mod out of the way.
    let out = f.raul(&[
        "mods",
        "rename",
        MOD,
        PREVIOUS,
        "--instance",
        s(&f.instance()),
    ]);
    assert!(out.status.success(), "{out:?}");
    f.settings("[output]\nwater = false\n");
    fomod_in_output(&f);
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(4), "{result}");
    assert_eq!(result["error"]["code"], "add_mod_failed", "{result}");
    assert_eq!(finished["changed"], false);
    assert_eq!(snapshot(&f.lod_mod()), before, "the old Mod is back");
    assert_eq!(
        mod_names(&f),
        [MOD],
        "the backup is put back under its name"
    );
    assert_eq!(enabled_lines(&f), 1);
}

#[test]
fn lodwright_refusals_keep_their_codes_and_exit_4_and_leave_the_instance_unchanged() {
    covers(&[ids::lod::generate::REFUSALS_EXIT_4]);
    let f = Fixture::new("raul-lodgen-codes");
    assert_eq!(f.generate(&[]).0, Some(0));
    let before = f.state();

    // A folder with files and no manifest is not an output folder lodwright may write into.
    std::fs::remove_dir_all(f.out_dir()).unwrap();
    std::fs::create_dir_all(f.out_dir()).unwrap();
    std::fs::write(f.out_dir().join("stray.txt"), "not lodwright's").unwrap();
    let (got, result, finished) = f.generate(&[]);
    assert_eq!(got, Some(4), "{result}");
    assert_eq!(finished["status"], "failed");
    assert_eq!(finished["code"], "lodwright_failed");
    assert_eq!(result["error"]["lodwright_code"], "output_not_empty");
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before, "output_not_empty changed the Instance");

    // A world the plugins do not define.
    std::fs::remove_dir_all(f.out_dir()).unwrap();
    let (got, result, finished) = f.generate(&["--world", "NoSuchWorld"]);
    assert_eq!(got, Some(4), "{result}");
    assert_eq!(finished["code"], "lodwright_failed");
    assert_eq!(result["error"]["lodwright_code"], "world_not_found");
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before, "world_not_found changed the Instance");
}

#[test]
fn a_stop_during_the_generation_adds_nothing_and_the_next_run_resumes() {
    covers(&[ids::lod::generate::STOP_RESUMES]);
    let f = Fixture::new("raul-lodgen-stop");
    // One thread, so the blocks finish in order and a stop after the first one leaves the rest unbuilt.
    f.settings("threads = 1\n");
    let before = f.state();

    // Stopped before the call starts: nothing is written into the Instance.
    let cancelled = CancelToken::new();
    cancelled.cancel();
    let outcome = f.in_process(&cancelled, &mut |_| {});
    assert_eq!(outcome.exit_code, 4, "{:?}", outcome.result);
    assert_eq!(
        outcome.result.status,
        raul_drivers::events::Status::Cancelled
    );
    assert!(!outcome.changed);
    assert_eq!(f.state(), before);

    // Stopped after the first finished block: what finished is kept in the work folder, the Instance is unchanged.
    // The cancel is set inside the emit, before the call goes on to its next safe point.
    let cancel = CancelToken::new();
    let stop = cancel.clone();
    let outcome = f.in_process(&cancel, &mut |e: Event| {
        if matches!(e, Event::Progress { .. }) {
            stop.cancel();
        }
    });
    assert_eq!(outcome.exit_code, 4, "{:?}", outcome.result);
    assert_eq!(
        outcome.result.status,
        raul_drivers::events::Status::Cancelled
    );
    assert_eq!(outcome.result.mod_name, None);
    assert!(!outcome.changed);
    assert_eq!(f.state(), before, "a stopped run changed the Instance");
    assert!(
        outcome
            .result
            .error
            .as_ref()
            .is_some_and(|e| e.message.contains("not touched") && e.message.contains("checkpoint")),
        "the message says the Mod is untouched and a checkpoint is kept: {:?}",
        outcome.result.error
    );
    // The output folder does not exist before a run has finished: the work is in the staging folder.
    assert!(
        !f.out_dir().exists(),
        "the output is written only at the swap"
    );
    let checkpoint: Value =
        serde_json::from_slice(&std::fs::read(f.staging_dir().join("checkpoint.json")).unwrap())
            .unwrap();
    assert!(
        checkpoint.is_object(),
        "the cancel kept a checkpoint in staging: {checkpoint}"
    );

    // The next run keeps the finished block and completes the Mod.
    let outcome = f.in_process(&CancelToken::new(), &mut |_| {});
    assert_eq!(outcome.exit_code, 0, "{:?}", outcome.result);
    assert!(outcome.changed);
    assert_eq!(outcome.result.mod_name.as_deref(), Some(MOD));
    let lodwright = outcome.result.lodwright.expect("the lodwright result");
    assert_eq!(lodwright["code"], "generated");
    assert!(
        lodwright["blocks"]["up_to_date"].as_u64().unwrap_or(0) >= 1,
        "{lodwright}"
    );
    assert_eq!(mod_names(&f), [MOD]);
    assert!(f.out_dir().join("manifest.json").is_file(), "swapped in");
    assert!(!f.staging_dir().exists(), "staging is gone after the swap");
}

#[test]
fn a_stop_after_a_finished_mod_keeps_the_mod_and_a_different_request_drops_the_checkpoint() {
    let f = Fixture::new("raul-lodgen-stop-again");
    f.settings("threads = 1\n");
    assert_eq!(f.in_process(&CancelToken::new(), &mut |_| {}).exit_code, 0);
    let mod_before = snapshot(&f.lod_mod());
    let out_before = snapshot(&f.out_dir());

    // A changed request is built again; stopped after its first block, the Mod and its output are as they were.
    f.settings("threads = 1\n[output]\nwater = false\n");
    let cancel = CancelToken::new();
    let stop = cancel.clone();
    let outcome = f.in_process(&cancel, &mut |e: Event| {
        if matches!(e, Event::Progress { .. }) {
            stop.cancel();
        }
    });
    assert_eq!(outcome.exit_code, 4, "{:?}", outcome.result);
    assert!(!outcome.changed);
    assert_eq!(snapshot(&f.lod_mod()), mod_before, "the Mod is untouched");
    assert_eq!(
        snapshot(&f.out_dir()),
        out_before,
        "the output is untouched"
    );
    assert!(f.staging_dir().is_dir(), "the checkpoint is kept");

    // Another request again removes that checkpoint and runs clean; the Mod is replaced only now.
    f.settings("threads = 1\n[output]\nwater = true\nobjects = false\n");
    let outcome = f.in_process(&CancelToken::new(), &mut |_| {});
    assert_eq!(outcome.exit_code, 0, "{:?}", outcome.result);
    assert!(!f.staging_dir().exists());
    assert_eq!(mod_names(&f), [MOD]);
}

/// lodwright checks the git working tree itself too: these runs only succeed when `raul` passes the allowance on.
#[test]
fn a_profile_that_allows_a_git_folder_generates_its_lod_inside_one() {
    let f = Fixture::new("raul-lodgen-git-key");
    std::fs::create_dir_all(f.path(".git")).unwrap();
    f.settings("allow_in_repo = true\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
}

#[test]
fn the_flag_lets_one_generation_write_inside_a_git_folder() {
    let f = Fixture::new("raul-lodgen-git-flag");
    std::fs::create_dir_all(f.path(".git")).unwrap();
    let (code, result, _) = f.generate(&["--allow-in-repo"]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
}

#[test]
fn a_generation_inside_a_git_folder_is_refused_before_anything_starts() {
    covers(&[ids::lod::generate::GIT_FOLDER_REFUSED]);
    let f = Fixture::new("raul-lodgen-git-neither");
    std::fs::create_dir_all(f.path(".git")).unwrap();
    let before = f.state();
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(2), "{result}");
    assert_eq!(finished["code"], "plan_refused");
    assert!(result["tool"].is_null(), "{result}");
    assert_eq!(f.state(), before);
}

/// One run per Profile at a time (#343): a second generate while one runs is refused and changes nothing, and a
/// settings edit made while it runs does not change the running request.
#[test]
fn a_second_generate_while_one_runs_is_refused_and_an_edit_waits_for_the_next_run() {
    covers(&[ids::lod::generate::SECOND_RUN_REFUSED]);
    let f = Fixture::new("raul-lodgen-one-run");
    let inst = f.instance();
    let plan = |f: &Fixture| -> Value {
        let out = f.raul(&["lod", "plan", "--json", "--instance", s(&inst)]);
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let before = lod::digest(&plan(&f)).unwrap();
    let mut second = None;
    let outcome = f.in_process(&CancelToken::new(), &mut |e| {
        if second.is_none() && matches!(e, Event::Progress { .. } | Event::Phase { .. }) {
            // While the run is going: edit the settings and start a second run of the same Profile.
            f.settings("[output]\nwater = false\n");
            second = Some(f.generate(&[]));
        }
    });
    assert_eq!(outcome.exit_code, 0, "{:?}", outcome.result);
    let (code, result, finished) = second.expect("the run reported progress");
    assert_eq!(code, Some(2), "{result}");
    assert_eq!(finished["code"], "run_in_progress", "{finished}");
    assert!(
        result["tool"].is_null(),
        "the second run started nothing: {result}"
    );
    // The run built the request it started with; the edit makes the LOD out of date for the next run.
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(f.out_dir().join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["request_digest"], before.as_str());
    assert_ne!(lod::digest(&plan(&f)).unwrap(), before);
    // Once it has finished, the next run goes ahead.
    assert_eq!(f.generate(&[]).0, Some(0));
}

#[test]
fn a_dry_run_or_a_bad_input_starts_nothing() {
    covers(&[ids::lod::generate::DRY_RUN]);
    let f = Fixture::new("raul-lodgen-dry");
    let before = f.state();
    // A dry run builds and checks the request, and runs and writes nothing.
    let (code, result, _) = f.generate(&["--dry-run"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        (result["dry_run"].as_bool(), result["tool"].is_null()),
        (Some(true), true)
    );
    assert!(!f.out_dir().exists(), "the dry run wrote an output folder");
    let out = f.raul(&[
        "lod",
        "generate",
        "--instance",
        s(&f.instance()),
        "--dry-run",
    ]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("dry run passed"));
    assert_eq!(f.state(), before, "a dry run changed the Instance");
    let out = f.raul(&[
        "lod",
        "generate",
        "--json",
        "--instance",
        s(&f.path("nowhere")),
    ]);
    assert_eq!(check_stream(&out).0["error"]["code"], "instance_invalid");
    // The program flag is gone: lodwright is not a program `raul` is given.
    assert_eq!(
        f.raul(&[
            "lod",
            "generate",
            "--instance",
            s(&f.instance()),
            "--lodwright",
            "/no/such/lodwright",
        ])
        .status
        .code(),
        Some(5)
    );
    assert_eq!(
        f.raul(&["lod", "generate", "--timeout-secs", "0"])
            .status
            .code(),
        Some(5)
    );
    assert_eq!(f.state(), before, "a refused run changed the Instance");
}

/// nvmod has its own `lod`, and writes a run log for every command it runs: `raul lod generate` leaves none.
#[test]
fn lod_generate_never_reaches_nvmod() {
    let f = Fixture::new("raul-lodgen-dispatch");
    let logs = f.path("state/logs");
    std::fs::remove_dir_all(&logs).unwrap();
    assert_eq!(f.generate(&[]).0, Some(0));
    let out = f.raul(&["lod", "generate", "--instance", s(&f.instance())]);
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("up to date"),
        "{out:?}"
    );
    assert!(!logs.exists(), "nvmod ran");
}

/// The far-shape rules in raul's data reach the request only for a load order that has the rule's world.
#[test]
fn far_shape_rules_are_requested_only_for_a_load_order_with_their_world() {
    let plan = |f: &Fixture, extra: &[&str]| -> Value {
        let inst = f.instance();
        let mut args = vec!["lod", "plan", "--json", "--instance", s(&inst)];
        args.extend_from_slice(extra);
        let out = f.raul(&args);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let with = Fixture::with_world("raul-lodgen-rules-on", "Wasteland");
    let request = plan(&with, &[]);
    lod::check_request(&request).unwrap();
    let rules = request["objects"]["shapes_from_shipped_blocks"]
        .as_array()
        .expect("the rules are in the request");
    assert_eq!(rules.len(), 1, "{request}");
    assert_eq!(rules[0]["world"], "Wasteland");
    assert_eq!(rules[0]["models"].as_array().unwrap().len(), 18);

    // A run limited to another world does not name it.
    let other = plan(&with, &["--world", "Elsewhere"]);
    assert!(other["objects"].get("shapes_from_shipped_blocks").is_none());
    // With object LOD off there is nothing to apply them to.
    with.settings("[output]\nobjects = false\n");
    let off = plan(&with, &[]);
    assert!(off["objects"].get("shapes_from_shipped_blocks").is_none());

    // A load order without the world gets exactly the request it had without rules.
    let without = Fixture::with_world("raul-lodgen-rules-off", "Island");
    let request = plan(&without, &[]);
    assert_eq!(
        request["objects"],
        serde_json::json!({ "enabled": true, "far_lights": true }),
        "{request}"
    );
}

/// Object LOD asks lodwright for the far light points of lamps (the engine shows them by night in its `modern`
/// preset only); with object LOD off the request does not name them, and the request passes lodwright's schema.
#[test]
fn far_lights_are_requested_whenever_object_lod_is_built() {
    let plan = |f: &Fixture| -> Value {
        let out = f.raul(&["lod", "plan", "--json", "--instance", s(&f.instance())]);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let f = Fixture::with_world("raul-lodgen-far-lights", "Island");
    let request = plan(&f);
    lod::check_request(&request).unwrap();
    assert_eq!(
        request["objects"]["far_lights"],
        serde_json::json!(true),
        "{request}"
    );

    f.settings("[output]\nobjects = false\n");
    let off = plan(&f);
    lod::check_request(&off).unwrap();
    assert!(off["objects"].get("far_lights").is_none(), "{off}");
}

/// The include rules in raul's data reach the request: those for the base game's lamp posts for every load order, the
/// rule of a plugin only for a load order that has it; the request still passes lodwright's schema.
#[test]
fn include_rules_are_requested_only_for_a_load_order_with_their_plugin() {
    let plan = |f: &Fixture| -> Value {
        let out = f.raul(&["lod", "plan", "--json", "--instance", s(&f.instance())]);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let rules_of = |request: &Value| -> Vec<Value> {
        request["inclusion"]["include_rules"]
            .as_array()
            .unwrap_or_else(|| panic!("the rules are in the request: {request}"))
            .clone()
    };
    let with = Fixture::with_world_and_plugin(
        "raul-lodgen-include-on",
        "Island",
        Some("Street Lights TTW.esm"),
    );
    let request = plan(&with);
    lod::check_request(&request).unwrap();
    let with_rules = rules_of(&request);
    let by_plugin: Vec<&Value> = with_rules
        .iter()
        .filter(|r| r.get("plugin").is_some())
        .collect();
    assert_eq!(by_plugin.len(), 1, "{request}");
    assert_eq!(by_plugin[0]["plugin"], "Street Lights TTW.esm");
    assert!(
        with_rules.iter().all(|r| r["generate"]["keep"].is_number()),
        "{request}"
    );
    assert!(
        with_rules
            .iter()
            .any(|r| r["models"] == "architecture/strip/nvstripstreetlamp*.nif"),
        "{request}"
    );
    // With object LOD off there is nothing to apply them to.
    with.settings("[output]\nobjects = false\n");
    assert!(plan(&with)["inclusion"].get("include_rules").is_none());

    // A load order without the plugin gets the base game's rules only.
    let without = Fixture::new("raul-lodgen-include-off");
    let request = plan(&without);
    lod::check_request(&request).unwrap();
    let general = rules_of(&request);
    assert_eq!(general.len(), with_rules.len() - 1, "{request}");
    assert!(
        general.iter().all(|r| r.get("plugin").is_none()),
        "{request}"
    );
}
