// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate` with lodwright linked into `raul`, on a synthetic world built with lodwright's own test kit:
//! success and a second run that is `up_to_date`, a replaced Mod, lodwright's refusals as exit code 4, a stop during
//! the generation that the next run resumes from, the dry run and bad inputs, and that nvmod never receives the
//! command. Nothing is started but the built `raul` binary, as a black box.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

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
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
        let world = SyntheticWorld::new(0xB00, "Island")
            .cell(2, 2, Heights::flat(16.0))
            .cell(3, 2, Heights::flat(24.0))
            .cell(2, 3, Heights::flat(32.0))
            .cell(3, 3, Heights::flat(8.0));
        std::fs::write(
            f.path("FalloutNV/Data/FalloutNV.esm"),
            world_plugin(&[], &world, false),
        )
        .unwrap();
        let (base, inst) = (f.path("FalloutNV"), f.instance());
        for args in [
            &["init", "--base", s(&base), "--instance", s(&inst)][..],
            &["profile", "create", "--instance", s(&inst)],
            &["stage", "--instance", s(&inst), "--copy"],
        ] {
            let out = f.raul(args);
            assert!(out.status.success(), "{args:?}: {out:?}");
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
        self.instance().join(".raul/lod/out")
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
    let f = Fixture::new("raul-lodgen-ok");
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

/// A FOMOD that needs answers in the output makes the Mod's install fail after the old Mod has been moved aside, which
/// is the add failure the rollback is for. The stray folder is not in lodwright's manifest, so lodwright keeps it.
fn fomod_in_output(f: &Fixture) {
    let dir = f.out_dir().join("fomod");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ModuleConfig.xml"), "<config/>").unwrap();
}

#[test]
fn a_failed_replacement_puts_the_old_mod_back() {
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
    let partial: Value =
        serde_json::from_slice(&std::fs::read(f.out_dir().join("manifest.json")).unwrap()).unwrap();
    // An interrupted manifest is the library's own answer to the cancel (a finished run writes a digest).
    assert_eq!(partial["request_digest"], "", "{partial}");
    assert!(
        partial["blocks"].as_array().is_some_and(|b| !b.is_empty()),
        "the finished block is in the manifest: {partial}"
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
}

#[test]
fn a_dry_run_or_a_bad_input_starts_nothing() {
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
