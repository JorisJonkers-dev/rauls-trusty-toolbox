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

const MOD: &str = "LOD (lodwright)";

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
    let out = f.raul(&[
        "mods",
        "create",
        "LOD (lodwright) previous",
        "--instance",
        s(&f.instance()),
    ]);
    assert!(out.status.success(), "{out:?}");
    f.settings("[output]\nwater = true\n");
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(mod_names(&f), [MOD], "the backup is gone");
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
        "LOD (lodwright) previous",
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
    let f = Fixture::new("raul-lodgen-git-neither");
    std::fs::create_dir_all(f.path(".git")).unwrap();
    let before = f.state();
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(2), "{result}");
    assert_eq!(finished["code"], "plan_refused");
    assert!(result["tool"].is_null(), "{result}");
    assert_eq!(f.state(), before);
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
