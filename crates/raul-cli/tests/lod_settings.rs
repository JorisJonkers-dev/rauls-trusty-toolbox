// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod settings show|set|unset|reset|undo` on a synthetic Instance: what they change in `raul-lod.toml`, what
//! the request then holds, and that Restore defaults gives the request of a Profile without the file.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use lodwright_testkit::land::Heights;
use lodwright_testkit::world::{SyntheticWorld, world_plugin};
use nvmod_testkit::{BaseBuilder, TempDir};
use raul_drivers::lod::settings::{SettingsDoc, catalogue};
use serde_json::Value;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
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

    fn file(&self) -> PathBuf {
        self.instance().join("profiles/Default/raul-lod.toml")
    }

    fn raul(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"))
            .output()
            .expect("raul runs")
    }

    /// `raul lod settings <words> --json`: the exit code and the document.
    fn settings(&self, words: &[&str]) -> (Option<i32>, SettingsDoc) {
        let inst = self.instance();
        let mut args = vec!["lod", "settings"];
        args.extend_from_slice(words);
        args.extend_from_slice(&["--json", "--instance", s(&inst)]);
        let out = self.raul(&args);
        let doc = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)));
        (out.status.code(), doc)
    }

    /// The request `raul lod plan --json` gives.
    fn request(&self) -> Value {
        let out = self.raul(&["lod", "plan", "--json", "--instance", s(&self.instance())]);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// `value` with the Instance folder written as `<instance>` and the separators after it as `/`, so a fixture is the
/// same on every system.
fn portable(value: Value, instance: &str) -> Value {
    match value {
        Value::String(s) => match s.strip_prefix(instance) {
            Some(rest) => Value::String(format!("<instance>{}", rest.replace('\\', "/"))),
            None => Value::String(s),
        },
        Value::Array(items) => {
            Value::Array(items.into_iter().map(|v| portable(v, instance)).collect())
        }
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, portable(v, instance)))
                .collect(),
        ),
        other => other,
    }
}

fn digest(request: &Value) -> String {
    raul_drivers::lod::digest(request).unwrap()
}

const EVERY_KIND: &str = r#"allow_in_repo = true
worlds = ["SyntheticWorld"]
threads = 2
memory_mib = 4096

[inclusion]
exclude_records = ["0001ABCD"]

[output]
terrain = true
trees = false

[levels.16]
quality = 18.0
"#;

#[test]
fn restore_defaults_gives_the_request_digest_of_a_profile_without_settings() {
    let f = Fixture::new("raul-lodset-digest");
    let without = digest(&f.request());
    std::fs::write(f.file(), EVERY_KIND).unwrap();
    assert_ne!(
        digest(&f.request()),
        without,
        "the settings change the request"
    );
    let (code, _) = f.settings(&["reset"]);
    assert_eq!(code, Some(0));
    assert_eq!(digest(&f.request()), without);
    assert_eq!(
        std::fs::read_to_string(f.file()).unwrap(),
        "allow_in_repo = true\n",
        "the git folder switch is not a generation setting and stays"
    );
}

#[test]
fn a_level_set_through_the_command_reaches_the_request_with_the_other_levels_at_their_defaults() {
    let f = Fixture::new("raul-lodset-level");
    let (code, doc) = f.settings(&["set", "levels.16.quality", "18"]);
    assert_eq!(code, Some(0), "{doc:?}");
    let request = f.request();
    let levels = &request["terrain"]["levels"];
    assert_eq!(
        levels
            .as_array()
            .unwrap()
            .iter()
            .map(|l| (l["level"].as_u64().unwrap(), l["quality"].as_f64().unwrap()))
            .collect::<Vec<_>>(),
        [(4, 10.0), (8, 15.0), (16, 18.0), (32, 25.0)]
    );
    assert_eq!(levels[2]["max_vertices"], 2000);
    raul_drivers::lod::check_request(&request).unwrap();
}

#[test]
fn a_saved_file_reads_back_as_the_same_options() {
    let f = Fixture::new("raul-lodset-roundtrip");
    std::fs::write(f.file(), EVERY_KIND).unwrap();
    let (_, written) = f.settings(&["show"]);
    // Rewrite every option the file sets through `set`, then compare what show reads.
    std::fs::remove_file(f.file()).unwrap();
    for o in written.options.iter().filter(|o| o.overridden) {
        let words: Vec<String> = match &o.value {
            Value::Array(items) => items
                .iter()
                .map(|i| i.as_str().unwrap().to_owned())
                .collect(),
            other => vec![other.to_string()],
        };
        let mut args = vec!["set", o.key.as_str()];
        args.extend(words.iter().map(String::as_str));
        assert_eq!(f.settings(&args).0, Some(0), "{}", o.key);
    }
    let (_, again) = f.settings(&["show"]);
    assert_eq!(again.options, written.options);
}

#[test]
fn a_key_this_version_does_not_know_is_refused_in_plain_words_and_nothing_changes() {
    let f = Fixture::new("raul-lodset-unknown");
    std::fs::write(f.file(), "colour = \"blue\"\n").unwrap();
    let out = f.raul(&["lod", "settings", "show", "--instance", s(&f.instance())]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(catalogue::UNKNOWN_KEY)
            && err.contains("colour")
            && err.contains(catalogue::NOTHING_CHANGED),
        "{err}"
    );
    let out = f.raul(&["lod", "plan", "--instance", s(&f.instance())]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains(catalogue::UNKNOWN_KEY));
    assert_eq!(
        std::fs::read_to_string(f.file()).unwrap(),
        "colour = \"blue\"\n"
    );
}

#[test]
fn a_value_out_of_range_is_refused_and_the_file_stays_as_it_was() {
    let f = Fixture::new("raul-lodset-range");
    std::fs::write(f.file(), "threads = 2\n").unwrap();
    let (code, doc) = f.settings(&["set", "levels.4.max_vertices", "2"]);
    assert_eq!(code, Some(2));
    let e = doc.error.unwrap();
    assert_eq!(
        (e.what.as_str(), e.detail.as_str()),
        (catalogue::OUT_OF_RANGE, catalogue::NOTHING_CHANGED)
    );
    assert_eq!(e.key.as_deref(), Some("levels.4.max_vertices"));
    assert_eq!(std::fs::read_to_string(f.file()).unwrap(), "threads = 2\n");
}

#[test]
fn undo_puts_back_the_file_restore_defaults_removed() {
    let f = Fixture::new("raul-lodset-undo");
    std::fs::write(f.file(), EVERY_KIND).unwrap();
    let (_, doc) = f.settings(&["reset"]);
    let change = doc.change.unwrap();
    assert_eq!(
        change.keys,
        [
            "worlds",
            "output.terrain",
            "output.trees",
            "inclusion.exclude_records",
            "threads",
            "memory_mib",
            "levels.16.quality"
        ]
    );
    assert!(doc.restore_point.is_some());
    let (code, _) = f.settings(&["undo"]);
    assert_eq!(code, Some(0));
    assert_eq!(std::fs::read_to_string(f.file()).unwrap(), EVERY_KIND);
    let (code, doc) = f.settings(&["undo"]);
    assert_eq!(code, Some(2));
    assert_eq!(doc.error.unwrap().what, catalogue::NOTHING_TO_UNDO);
}

/// The fixtures in `contracts/fixtures/raul-lod-settings/` are what the command prints, with the Instance written as
/// `<instance>`.
#[test]
fn the_fixtures_are_what_the_command_prints() {
    let f = Fixture::new("raul-lodset-fixtures");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/raul-lod-settings");
    let inst = f.instance().display().to_string();
    let as_fixture = |doc: &SettingsDoc| portable(serde_json::to_value(doc).unwrap(), &inst);
    std::fs::write(
        f.file(),
        "allow_in_repo = true\n\n[output]\ntrees = false\n",
    )
    .unwrap();
    let mut cases = vec![("show.json", f.settings(&["show"]).1)];
    cases.push((
        "set.json",
        f.settings(&["set", "levels.8.max_vertices", "3000"]).1,
    ));
    cases.push(("reset-dry-run.json", f.settings(&["reset", "--dry-run"]).1));
    cases.push(("reset.json", f.settings(&["reset"]).1));
    cases.push(("refused.json", f.settings(&["set", "threads", "0"]).1));
    for (name, doc) in cases {
        let path = dir.join(name);
        let got = as_fixture(&doc);
        if std::env::var_os("RAUL_BLESS_CONTRACTS").is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                &path,
                format!("{}\n", serde_json::to_string_pretty(&got).unwrap()),
            )
            .unwrap();
        }
        let want: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn without_json_the_options_and_each_change_are_one_line_each() {
    let f = Fixture::new("raul-lodset-text");
    let inst = f.instance();
    let run = |words: &[&str]| {
        let mut args = vec!["lod", "settings"];
        args.extend_from_slice(words);
        args.extend_from_slice(&["--instance", s(&inst)]);
        let out = f.raul(&args);
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).unwrap()
    };
    assert!(run(&["set", "output.trees", "off"]).contains("changed output.trees"));
    let shown = run(&["show"]);
    assert!(
        shown.contains("* output.trees = false  (default true)"),
        "{shown}"
    );
    assert!(
        shown.contains("  threads = (decided at run time)"),
        "{shown}"
    );
    assert!(run(&["reset", "--dry-run"]).contains("would remove output.trees"));
    assert!(run(&["reset"]).contains("removed output.trees"));
    assert!(run(&["show"]).contains("a restore point is kept"));
    assert!(run(&["undo"]).contains("put back output.trees"));
    assert!(run(&["unset", "output.trees"]).contains("changed output.trees"));
    assert!(run(&["unset", "output.trees"]).contains("nothing to change"));
}

#[test]
fn a_world_no_active_plugin_defines_is_shown_as_a_warning_and_is_still_saved() {
    let f = Fixture::new("raul-lodset-world");
    // A game master that defines one world, Island.
    let world = SyntheticWorld::new(0xB00, "Island").cell(2, 2, Heights::flat(16.0));
    std::fs::write(
        f.path("FalloutNV/Data/FalloutNV.esm"),
        world_plugin(&[], &world, false),
    )
    .unwrap();
    let staged = f.raul(&["stage", "--instance", s(&f.instance()), "--copy"]);
    assert!(staged.status.success(), "{staged:?}");
    let (code, doc) = f.settings(&["set", "worlds", "island", "NoSuchWorld"]);
    assert_eq!(code, Some(0));
    assert!(doc.change.unwrap().changed);
    let w = &doc.warnings;
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].key, "worlds");
    assert_eq!(w[0].what, catalogue::UNKNOWN_WORLD, "{w:?}");
    assert_eq!(w[0].value.as_deref(), Some("NoSuchWorld"));
    let out = f.raul(&["lod", "settings", "show", "--instance", s(&f.instance())]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("warning: worlds"),
        "{out:?}"
    );
}

#[test]
fn a_command_line_the_settings_verbs_do_not_take_is_a_usage_error() {
    let f = Fixture::new("raul-lodset-usage");
    for args in [
        &["lod", "settings"][..],
        &["lod", "settings", "drop"],
        &["lod", "settings", "show", "extra"],
        &["lod", "settings", "unset"],
        &["lod", "settings", "set"],
        &["lod", "settings", "show", "--dry-run"],
    ] {
        assert_eq!(f.raul(args).status.code(), Some(5), "{args:?}");
    }
    let out = f.raul(&[
        "lod",
        "settings",
        "show",
        "--json",
        "--instance",
        s(&f.path("nowhere")),
    ]);
    assert_eq!(out.status.code(), Some(2));
    let doc: SettingsDoc = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc.error.unwrap().what, catalogue::INSTANCE_INVALID);
}
