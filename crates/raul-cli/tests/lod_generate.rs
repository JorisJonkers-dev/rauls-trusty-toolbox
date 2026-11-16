// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate` against a fake lodwright on synthetic fixtures: success, `up_to_date`, a replaced Mod, every
//! lodwright exit code, newer result and manifest versions, cancelling, and that nvmod never receives the command.
//! The fake is a shell script, so these tests run on Linux and macOS.
#![cfg(unix)]
// These tests start the built binary and `kill`, which are not tool calls (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nvmod_testkit::{BaseBuilder, TempDir, snapshot, write_script};
use serde_json::Value;

/// The fake lodwright. It reads its mode from `mode` next to it, records its arguments and process id, reads the
/// output folder from the request, and behaves like lodwright would: a folder with a manifest is up to date.
const FAKE: &str = r#"
here=$(dirname "$0")
mode=$(cat "$here/mode")
echo "$@" > "$here/args"
echo $$ > "$here/pid"
req=""; events=""
while [ $# -gt 0 ]; do
  case "$1" in
    --request) req="$2"; shift ;;
    --events) events="$2"; shift ;;
  esac
  shift
done
cp "$req" "$here/request.json"
out=$(sed -n 's/.*"output_dir": *"\([^"]*\)".*/\1/p' "$req")
result() { # result <status> <code> <schema version>
  printf '{"schema_version":%s,"contract":"lodwright-result","tool":{"name":"lodwright","version":"0.3.0"},"status":"%s","code":"%s","reason":"fake %s"}\n' "${3:-1}" "$1" "$2" "$2"
}
manifest() { printf '{"schema_version":%s,"contract":"lodwright-manifest","files":[]}\n' "${1:-1}" > "$out/manifest.json"; }
case "$mode" in
  ok|v2|newermanifest|nomanifest)
    if [ "$mode" = ok ] && [ -f "$out/manifest.json" ]; then result ok up_to_date; exit 0; fi
    mkdir -p "$out/meshes/landscape/lod" "$out/textures/landscape/lod"
    cat "$here/marker" > "$out/meshes/landscape/lod/block.nif" 2>/dev/null || echo first > "$out/meshes/landscape/lod/block.nif"
    echo synthetic > "$out/textures/landscape/lod/block.dds"
    echo '{"event":"block","world":"SyntheticWorld","level":4,"done":1,"of":2,"up_to_date":false}' >> "$events"
    echo '{"event":"block","world":"SyntheticWorld","level":4,"done":2,"of":2,"up_to_date":false}' >> "$events"
    case "$mode" in
      nomanifest) ;;
      newermanifest) manifest 2 ;;
      *) manifest ;;
    esac
    result ok ok ;;
  failed) result failed output_not_empty; exit 1 ;;
  usage) result failed usage; exit 2 ;;
  unsupported) result unsupported missing_capability; exit 3 ;;
  newer) result ok ok 2 ;;
  garbage) echo "this is not a result" ;;
  slow) exec sleep 30 ;;
esac
"#;

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
        std::fs::create_dir_all(f.path("tools")).unwrap();
        write_script(&f.path("tools"), "lodwright", FAKE);
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
    fn mode(&self, mode: &str) {
        std::fs::write(self.path("tools/mode"), mode).unwrap();
    }
    fn args(&self) -> String {
        std::fs::read_to_string(self.path("tools/args")).unwrap_or_default()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_raul"));
        c.args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"));
        c
    }

    fn raul(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("raul runs")
    }

    fn argv(&self, extra: &[&str]) -> Vec<String> {
        let mut v: Vec<String> = ["lod", "generate", "--json", "--instance"]
            .map(str::to_owned)
            .to_vec();
        v.push(s(&self.instance()).to_owned());
        v.push("--lodwright".into());
        v.push(s(&self.path("tools/lodwright")).to_owned());
        v.extend(extra.iter().map(|e| (*e).to_owned()));
        v
    }

    fn generate(&self, extra: &[&str]) -> (Option<i32>, Value, Value) {
        let words = self.argv(extra);
        let args: Vec<&str> = words.iter().map(String::as_str).collect();
        let out = self.raul(&args);
        let (result, finished) = check_stream(&out);
        (out.status.code(), result, finished)
    }

    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }

    fn lod_mod(&self) -> PathBuf {
        self.instance().join("mods/LOD (lodwright)")
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

#[test]
fn success_keeps_one_enabled_lod_mod_and_a_second_run_is_up_to_date() {
    let f = Fixture::new("raul-lodgen-ok");
    f.mode("ok");
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["mod"], "LOD (lodwright)");
    assert_eq!(result["files"], 2, "the manifest is not part of the Mod");
    assert_eq!(result["up_to_date"], false);
    assert_eq!(result["tool"]["version"], "0.3.0");
    assert_eq!(finished["changed"], true);
    assert!(f.lod_mod().join("meshes/landscape/lod/block.nif").is_file());
    assert!(!f.lod_mod().join("manifest.json").exists());
    let modlist =
        std::fs::read_to_string(f.instance().join("profiles/Default/modlist.txt")).unwrap();
    assert!(
        modlist.lines().any(|l| l == "+LOD (lodwright)"),
        "{modlist}"
    );
    // The request lodwright got is the one `raul lod plan` shows.
    let sent: Value =
        serde_json::from_str(&std::fs::read_to_string(f.path("tools/request.json")).unwrap())
            .unwrap();
    raul_drivers::lod::check_request(&sent).unwrap();
    let plan = f.raul(&["lod", "plan", "--json", "--instance", s(&f.instance())]);
    assert_eq!(serde_json::from_slice::<Value>(&plan.stdout).unwrap(), sent);
    assert!(f.args().starts_with("generate --request "), "{}", f.args());
    assert!(f.args().contains("--json") && f.args().contains("--events"));

    let before = f.state();
    let (code, result, finished) = f.generate(&[]);
    assert_eq!(code, Some(0));
    assert_eq!(result["up_to_date"], true);
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before, "an up-to-date run changed the Instance");
}

#[test]
fn a_new_output_replaces_the_lod_mod_and_leaves_one() {
    let f = Fixture::new("raul-lodgen-replace");
    f.mode("ok");
    assert_eq!(f.generate(&[]).0, Some(0));
    // The inputs changed: lodwright writes new files.
    std::fs::remove_file(f.instance().join(".raul/lod/out/manifest.json")).unwrap();
    std::fs::write(f.path("tools/marker"), "second").unwrap();
    let (code, result, _) = f.generate(&[]);
    assert_eq!(code, Some(0), "{result}");
    let block =
        std::fs::read_to_string(f.lod_mod().join("meshes/landscape/lod/block.nif")).unwrap();
    assert_eq!(block.trim(), "second");
    let mods: Vec<String> = std::fs::read_dir(f.instance().join("mods"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(mods, ["LOD (lodwright)"], "{mods:?}");
    let modlist =
        std::fs::read_to_string(f.instance().join("profiles/Default/modlist.txt")).unwrap();
    assert_eq!(modlist.matches("LOD (lodwright)").count(), 1, "{modlist}");
    // A backup an interrupted run left behind is removed by the next run.
    let out = f.raul(&[
        "mods",
        "create",
        "LOD (lodwright) previous",
        "--instance",
        s(&f.instance()),
    ]);
    assert!(out.status.success(), "{out:?}");
    std::fs::remove_file(f.instance().join(".raul/lod/out/manifest.json")).unwrap();
    assert_eq!(f.generate(&[]).0, Some(0));
    assert!(!f.instance().join("mods/LOD (lodwright) previous").exists());
}

#[test]
fn every_lodwright_failure_maps_to_its_exit_code_and_leaves_the_instance_unchanged() {
    let f = Fixture::new("raul-lodgen-codes");
    f.mode("ok");
    assert_eq!(f.generate(&[]).0, Some(0));
    for (mode, exit, status, code, lodwright_code) in [
        (
            "failed",
            4,
            "failed",
            "lodwright_failed",
            Some("output_not_empty"),
        ),
        ("usage", 4, "failed", "lodwright_failed", Some("usage")),
        (
            "unsupported",
            2,
            "unsupported",
            "unsupported",
            Some("missing_capability"),
        ),
        ("newer", 2, "refused", "contract_refused", None),
        ("garbage", 4, "failed", "tool_failed", None),
        ("nomanifest", 4, "failed", "tool_failed", None),
        ("newermanifest", 2, "refused", "contract_refused", None),
    ] {
        std::fs::remove_file(f.instance().join(".raul/lod/out/manifest.json")).ok();
        let before = f.state();
        f.mode(mode);
        let (got, result, finished) = f.generate(&[]);
        assert_eq!(got, Some(exit), "{mode}: {result}");
        assert_eq!(finished["status"], status, "{mode}");
        assert_eq!(finished["code"], code, "{mode}");
        assert_eq!(finished["changed"], false, "{mode}");
        assert_eq!(
            result["error"]["lodwright_code"].as_str(),
            lodwright_code,
            "{mode}"
        );
        assert_eq!(f.state(), before, "{mode} changed the Instance");
    }
}

#[test]
fn a_newer_lodwright_result_is_refused_with_a_plain_reason() {
    let f = Fixture::new("raul-lodgen-newer");
    f.mode("newer");
    let (code, _, finished) = f.generate(&[]);
    assert_eq!(code, Some(2));
    let message = finished["message"].as_str().unwrap();
    assert!(
        message.contains("version 2") && message.contains("reads version 1"),
        "{message}"
    );
}

#[test]
fn sigint_stops_lodwright_and_exits_4_with_nothing_added() {
    let f = Fixture::new("raul-lodgen-sigint");
    let before = f.state();
    f.mode("slow");
    let words = f.argv(&[]);
    let args: Vec<&str> = words.iter().map(String::as_str).collect();
    let child = f
        .command(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid_file = f.path("tools/pid");
    let start = Instant::now();
    while std::fs::read_to_string(&pid_file)
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "the fake never started"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let fake = std::fs::read_to_string(&pid_file)
        .unwrap()
        .trim()
        .to_owned();
    std::thread::sleep(Duration::from_millis(300));
    let raul_pid = child.id().to_string();
    assert!(
        Command::new("kill")
            .args(["-INT", &raul_pid])
            .status()
            .unwrap()
            .success()
    );
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(4),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let (result, finished) = check_stream(&out);
    assert_eq!(finished["status"], "cancelled");
    assert_eq!(result["mod"], Value::Null);
    let alive = Command::new("kill")
        .args(["-0", &fake])
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(!alive.success(), "lodwright is still running");
    assert_eq!(f.state(), before);
}

#[test]
fn a_dry_run_or_a_missing_program_starts_nothing() {
    let f = Fixture::new("raul-lodgen-dry");
    let before = f.state();
    f.mode("ok");
    let (code, result, _) = f.generate(&["--dry-run"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        (result["dry_run"].as_bool(), result["tool"].is_null()),
        (Some(true), true)
    );
    let out = f.raul(&[
        "lod",
        "generate",
        "--json",
        "--instance",
        s(&f.instance()),
        "--lodwright",
        "/no/such/lodwright",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(check_stream(&out).0["error"]["code"], "tool_missing");
    let out = f.raul(&[
        "lod",
        "generate",
        "--json",
        "--instance",
        s(&f.path("nowhere")),
    ]);
    assert_eq!(check_stream(&out).0["error"]["code"], "instance_invalid");
    assert!(f.args().is_empty(), "lodwright was started");
    assert_eq!(f.state(), before);
    assert_eq!(
        f.raul(&["lod", "generate", "--timeout-secs", "0"])
            .status
            .code(),
        Some(5)
    );
    let out = f.raul(&[
        "lod",
        "generate",
        "--instance",
        s(&f.instance()),
        "--dry-run",
    ]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("dry run passed"));
}

/// nvmod has its own `lod`, and writes a run log for every command it runs: `raul lod generate` leaves none.
#[test]
fn lod_generate_never_reaches_nvmod() {
    let f = Fixture::new("raul-lodgen-dispatch");
    let logs = f.path("state/logs");
    std::fs::remove_dir_all(&logs).unwrap();
    f.mode("ok");
    assert_eq!(f.generate(&[]).0, Some(0));
    let out = f.raul(&[
        "lod",
        "generate",
        "--instance",
        s(&f.instance()),
        "--lodwright",
        s(&f.path("tools/lodwright")),
    ]);
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("up to date"),
        "{out:?}"
    );
    assert!(!logs.exists(), "nvmod ran");
}
