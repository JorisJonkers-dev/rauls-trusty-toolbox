// SPDX-License-Identifier: GPL-3.0-only
//! `raul setup` through the command line entry (`raul_cli::run`, no process): it is routed to raul, and it answers
//! the transcripts nvmod recorded for its own `setup` (nvmod-rs#90, under `components/nvmod-rs`) with the same exit
//! codes and the same events, for the cases that need no TTW installer and no engine release server. The cases that
//! do are named in `NOT_REPLAYED` with the reason; the setup job's own tests cover those steps with synthetic
//! fixtures (`crates/raul-core/tests/setup.rs`, `setup_command.rs`).

use raul_testkit::{covers, ids};
use std::fs;
use std::path::{Path, PathBuf};

use nvmod_testkit::{BaseBuilder, PeBuilder, PluginBuilder, TempDir, ZipBuilder, write_file};
use raul_cli::{Route, route, run};
use serde_json::Value;

/// The recorded cases this test does not replay, and why.
const NOT_REPLAYED: &[(&str, &str)] = &[
    (
        "dry-run-text",
        "text of nvmod's own wording; the JSON dry run is replayed",
    ),
    (
        "failing-mod-install-keep-partial",
        "runs the TTW installer and the engine release first; kept-partial is tested in raul-core",
    ),
    (
        "failing-mod-install-rolls-back",
        "runs the TTW installer and the engine release first; rollback is tested in raul-core",
    ),
    (
        "failing-nv-rs-check-exit-4-leaves-no-instance",
        "a fake release server; the engine step is tested in raul-core",
    ),
    (
        "full-run-json",
        "runs the TTW installer and the engine release; raul runs TTW in process with ttwright",
    ),
    ("full-run-text", "the same, and the wording of nvmod's text"),
    (
        "skip-nv-rs-json",
        "runs the TTW installer, which raul does not start",
    ),
    (
        "wrong-release-checksum-exit-3-leaves-no-instance",
        "a fake release server; the engine step is tested in raul-core",
    ),
];

/// The options nvmod's `setup` took that `raul setup` does not (they fetched an engine release, or named a tool or a
/// store that no longer exists): they are a usage error here (see `removed_options_are_a_usage_error`), so a replayed
/// recorded command line goes without them. The recorded outcomes do not depend on them.
const REMOVED_WITH_VALUE: &[&str] = &[
    "--release-url",
    "--asset",
    "--repo",
    "--gh",
    "--curl",
    "--retry-delay-ms",
    "--extract-tool",
    "--installer",
    "--lock",
    "--store",
];

fn without_removed_options(args: Vec<String>) -> Vec<String> {
    let mut kept = Vec::new();
    let mut words = args.into_iter();
    while let Some(word) = words.next() {
        if word == "--release" {
            continue;
        }
        if REMOVED_WITH_VALUE.contains(&word.as_str()) {
            words.next();
            continue;
        }
        kept.push(word);
    }
    kept
}

#[test]
fn removed_options_are_a_usage_error() {
    for option in [
        "--release",
        "--release-url x",
        "--gh x",
        "--installer x",
        "--lock x",
    ] {
        let mut args: Vec<String> = [
            "setup",
            "--base",
            "b",
            "--instance",
            "i",
            "--masterlist",
            "m",
        ]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
        args.extend(option.split(' ').map(str::to_owned));
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        assert_eq!(code, raul_cli::USAGE, "{option}");
        assert!(String::from_utf8(err).unwrap().contains("raul"), "{option}");
    }
}

fn baseline_dir() -> PathBuf {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../components/nvmod-rs/crates/nvmod-cli/tests/baseline");
    // Once nvmod no longer has `setup`, its transcripts are kept under `removed/`.
    [tests.join("setup"), tests.join("removed/setup")]
        .into_iter()
        .find(|d| d.is_dir())
        .expect("the recorded transcripts of nvmod's setup")
}

/// The world of the transcripts: a base copy, a Fallout 3 folder, the masterlist and a mod archive.
fn world(root: &Path, unknown_base: bool, existing_instance: bool) {
    if unknown_base {
        BaseBuilder::unknown().write(&root.join("base"));
    } else {
        BaseBuilder::steam().write(&root.join("base"));
        write_file(
            &root.join("base"),
            "nvse_1_4.dll",
            &PeBuilder::dll().build(),
        );
        write_file(
            &root.join("base"),
            "Data/FalloutNV.esm",
            &PluginBuilder::master().build(),
        );
    }
    write_file(&root.join("fo3"), "Fallout3.exe", b"synthetic fallout 3");
    let work = root.join("work");
    fs::create_dir_all(&work).unwrap();
    fs::write(
        work.join("masterlist.yaml"),
        "globals:\n  - {type: say, content: 'Masterlist for tests'}\nplugins: []\n",
    )
    .unwrap();
    ZipBuilder::new()
        .file("Core.esp", &PluginBuilder::new().build())
        .file("Textures/dirt.dds", b"dirt")
        .write(&work.join("Core.zip"));
    if existing_instance {
        write_file(&root.join("fnv"), "nvmod-instance.json", b"{}");
    }
}

fn shape(stdout: &Value) -> Vec<(String, String, String)> {
    let text = |v: &Value, k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let events: Vec<&Value> = match stdout {
        Value::Array(a) => a.iter().collect(),
        other => vec![other],
    };
    events
        .into_iter()
        .filter(|e| e.is_object())
        .map(|e| (text(e, "event"), text(e, "status"), text(e, "step")))
        .collect()
}

#[test]
fn setup_is_a_command_of_raul_and_nvmod_is_never_reached() {
    assert_eq!(route(&["setup".to_owned()]), Route::Own);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&["setup".to_owned()], &mut out, &mut err);
    let err = String::from_utf8(err).unwrap();
    assert_eq!(code, raul_cli::USAGE);
    assert!(err.contains("raul setup:"), "{err}");
    assert!(!err.contains("nvmod setup"), "{err}");
}

#[test]
fn raul_setup_answers_the_transcripts_nvmod_recorded() {
    covers(&[ids::setup::pipeline::CLI_TRANSCRIPTS]);
    let mut replayed = 0;
    let mut names: Vec<String> = fs::read_dir(baseline_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        if NOT_REPLAYED.iter().any(|(n, _)| *n == name) {
            continue;
        }
        let case: Value = serde_json::from_str(
            &fs::read_to_string(baseline_dir().join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        let root = TempDir::new("setup-replay");
        world(
            root.path(),
            name == "unsupported-base-refused",
            name == "existing-instance-refused",
        );
        let args = without_removed_options(
            case["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| {
                    a.as_str()
                        .unwrap()
                        .replace("<TMP>", &root.path().display().to_string())
                })
                .collect(),
        );
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        let out = String::from_utf8(out).unwrap();
        let err = String::from_utf8(err).unwrap();
        assert_eq!(
            i64::from(code),
            case["exit_code"].as_i64().unwrap(),
            "{name}: {err}"
        );
        let recorded = &case["stdout"];
        let json = args.iter().any(|a| a == "--json");
        if json {
            let got: Vec<Value> = out
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
            // nvmod's transcripts have its engine install step, `nv-rs`; the engine is linked into raul, so raul has
            // no such step and the recorded ones are compared without it.
            let recorded_steps: Vec<_> = shape(recorded)
                .into_iter()
                .filter(|(event, _, step)| !(event == "step" && step == "nv-rs"))
                .collect();
            assert_eq!(
                shape(&Value::Array(got.clone())),
                recorded_steps,
                "{name}: the events differ"
            );
            for (i, e) in got.iter().enumerate() {
                assert_eq!(e["seq"].as_u64(), Some(i as u64 + 1), "{name}");
            }
            if let Some(end) = got.last().filter(|e| e["event"] == "finished") {
                let was = recorded.as_array().map_or(recorded, |a| a.last().unwrap());
                assert_eq!(end["exit_code"], was["exit_code"], "{name}");
            }
        }
        replayed += 1;
    }
    assert!(replayed >= 7, "replayed {replayed} transcripts");
}

#[test]
fn every_recorded_case_is_replayed_or_named_with_its_reason() {
    for (name, why) in NOT_REPLAYED {
        assert!(
            baseline_dir().join(format!("{name}.json")).is_file(),
            "{name}"
        );
        assert_ne!(*why, "");
    }
}
