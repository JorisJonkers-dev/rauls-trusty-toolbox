// SPDX-License-Identifier: GPL-3.0-only
//! `raul ttw` against a fake ttwright on synthetic fixtures: every ttwright exit code, the version check of its
//! result, `--resume`, cancelling, the refusals, and that nvmod never receives the command. The fake is a shell
//! script, so these tests run on Linux and macOS.
#![cfg(unix)]
// These tests start the built binary and `kill`, which are not tool calls (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nvmod_testkit::{BaseBuilder, PeBuilder, TempDir, snapshot, write_file, write_script};
use serde_json::Value;

/// The fake ttwright. It reads its mode from `mode` next to it (the Runner passes no environment), records its
/// arguments and process id, and behaves like ttwright v0.3.0 would in that case.
const FAKE: &str = r#"
here=$(dirname "$0")
mode=$(cat "$here/mode")
echo "$@" > "$here/args"
echo $$ > "$here/pid"
dest=""; events=""; resume=false
while [ $# -gt 0 ]; do
  case "$1" in
    --dest) dest="$2"; shift ;;
    --events) events="$2"; shift ;;
    --resume) resume=true ;;
  esac
  shift
done
ev() { echo "{\"schema_version\":1,$1}" >> "$events"; }
result() { # result <status> <exit> <error code or empty> <schema version>
  if [ -n "$3" ]; then err="{\"code\":\"$3\",\"message\":\"fake $3\",\"feature\":null}"; else err=null; fi
  printf '{"schema_version":%s,"contract":"ttwright-result","tool":{"name":"ttwright","version":"0.3.0"},"status":"%s","exit_code":%s,"checks_passed":true,"assets":{"total":2,"done":2,"failed":0},"archives":{"total":0,"done":0,"failed":0},"post_install":{"total":0,"done":0,"failed":0},"resumed":%s,"dry_run":false,"elapsed_ms":5,"output_dir":"%s","error":%s}\n' "${4:-1}" "$1" "$2" "$resume" "$dest" "$err"
}
ev '"event":"started","package":"Synthetic","version":"3.4.0","assets_total":2,"resumed":false'
ev '"event":"phase","phase":"package"'
case "$mode" in
  success)
    ev '"event":"phase","phase":"assets"'
    ev '"event":"progress","phase":"assets","done":1,"of":2,"message":"first"'
    ev '"event":"warning","message":"a synthetic warning"'
    mkdir -p "$dest/textures"
    echo synthetic > "$dest/Example.esm"
    echo synthetic > "$dest/Example - Patch.esp"
    echo synthetic > "$dest/textures/a.dds"
    ev '"event":"progress","phase":"assets","done":2,"of":2'
    ev '"event":"finished","status":"success","code":null'
    result success 0 "" ;;
  failed) echo '{}' > "$dest/.ttwright-journal.json"; result failed 1 check_failed; exit 1 ;;
  usage) result failed 2 usage; exit 2 ;;
  unsupported) result unsupported 3 unsupported_version; exit 3 ;;
  cancelled) echo '{}' > "$dest/.ttwright-journal.json"; echo half > "$dest/half"; result cancelled 4 cancelled; exit 4 ;;
  newer) result success 0 "" 2 ;;
  garbage) echo "this is not a result" ;;
  empty) result success 0 "" ;;
  slow) echo '{}' > "$dest/.ttwright-journal.json"; ev '"event":"phase","phase":"assets"'; exec sleep 30 ;;
esac
"#;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        let root = TempDir::new(label);
        let f = Fixture { root };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.newvegas());
        write_file(&f.fallout3(), "Fallout3.exe", &PeBuilder::exe().build());
        write_file(&f.fallout3(), "Data/Fallout3.esm", b"synthetic");
        write_file(f.root.path(), "TTW.mpi", b"synthetic package");
        std::fs::create_dir_all(f.tools()).unwrap();
        write_script(&f.tools(), "ttwright", FAKE);
        let out = f.raul(&[
            "init",
            "--base",
            f.newvegas().to_str().unwrap(),
            "--instance",
            f.instance().to_str().unwrap(),
        ]);
        assert!(out.status.success(), "{out:?}");
        f.profile(&f.instance());
        f
    }

    /// Give the Instance at `dir` its `Default` profile.
    fn profile(&self, dir: &Path) {
        let out = self.raul(&["profile", "create", "--instance", dir.to_str().unwrap()]);
        assert!(out.status.success(), "{out:?}");
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }
    fn newvegas(&self) -> PathBuf {
        self.path("FalloutNV")
    }
    fn fallout3(&self) -> PathBuf {
        self.path("Fallout3")
    }
    fn instance(&self) -> PathBuf {
        self.path("instance")
    }
    fn tools(&self) -> PathBuf {
        self.path("tools")
    }
    fn ttwright(&self) -> PathBuf {
        self.tools().join("ttwright")
    }

    fn mode(&self, mode: &str) {
        std::fs::write(self.tools().join("mode"), mode).unwrap();
    }

    fn args(&self) -> String {
        std::fs::read_to_string(self.tools().join("args")).unwrap_or_default()
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

    /// `raul ttw --json` with every input given as a flag, plus `extra`.
    fn ttw(&self, extra: &[&str]) -> (Option<i32>, Vec<Value>) {
        let (fo3, pkg, inst) = (
            self.fallout3(),
            self.path("TTW.mpi"),
            self.instance(),
        );
        let mut args = vec![
            "ttw",
            "--json",
            "--instance",
            inst.to_str().unwrap(),
            "--fallout3",
            fo3.to_str().unwrap(),
            "--package",
            pkg.to_str().unwrap(),
        ];
        args.extend_from_slice(extra);
        let out = self.raul(&args);
        (out.status.code(), lines(&out))
    }

    /// What a failed run must leave as it was: the Mods and the Profiles.
    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }
}

fn lines(out: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("{e}: {l}\n{}", String::from_utf8_lossy(&out.stderr)))
        })
        .collect()
}

fn validator(schema: &str) -> jsonschema::Validator {
    jsonschema::validator_for(&serde_json::from_str(schema).unwrap()).unwrap()
}

/// Every line is a valid `raul-events` line numbered from 1, the result a valid `raul-ttw-result`, and the
/// last line `finished` with the process's exit code. Returns the result and the finished line.
fn check_stream(code: Option<i32>, lines: &[Value]) -> (Value, Value) {
    let events = validator(&raul_drivers::events::schema());
    let ttw = validator(&raul_drivers::ttw::schema());
    for (i, l) in lines.iter().enumerate() {
        let errors: Vec<String> = events.iter_errors(l).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{errors:?} {l}");
        assert_eq!(l["seq"], i as u64 + 1);
        assert_eq!(l["command"], "ttw");
    }
    let finished = lines.last().expect("a finished line").clone();
    assert_eq!(finished["event"], "finished");
    assert_eq!(finished["exit_code"].as_i64(), code.map(i64::from));
    let result = lines[lines.len() - 2]["result"].clone();
    let errors: Vec<String> = ttw.iter_errors(&result).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?} {result}");
    assert_eq!(result["status"], finished["status"]);
    (result, finished)
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

#[test]
fn success_adds_one_mod_and_enables_its_plugins() {
    let f = Fixture::new("raul-ttw-success");
    f.mode("success");
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, finished) = check_stream(code, &lines);
    assert_eq!(result["mod"], "Tale of Two Wastelands");
    assert_eq!(result["files"], 3);
    assert_eq!(result["staging"], Value::Null);
    assert_eq!(result["tool"]["version"], "0.3.0");
    assert_eq!(result["ttwright"]["contract"], "ttwright-result");
    assert_eq!(finished["changed"], true);
    let enabled: Vec<&str> = result["plugins_enabled"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert_eq!(enabled, ["Example.esm", "Example - Patch.esp"]);
    // ttwright's progress came through as raul events, in order, before the result.
    let kinds: Vec<&str> = lines.iter().map(|l| l["event"].as_str().unwrap()).collect();
    assert_eq!(kinds[0], "started");
    assert!(kinds.contains(&"progress") && kinds.contains(&"warning"));
    assert_eq!(&kinds[kinds.len() - 3..], ["phase", "result", "finished"]);
    let mod_dir = f.instance().join("mods/Tale of Two Wastelands");
    assert!(mod_dir.join("textures/a.dds").is_file());
    let plugins = read(&f.instance().join("profiles/Default/plugins.txt"));
    assert!(plugins.lines().any(|l| l == "Example.esm"), "{plugins}");
    assert!(
        read(&f.instance().join("profiles/Default/modlist.txt"))
            .contains("+Tale of Two Wastelands")
    );
    assert!(
        !f.instance().join(".raul/ttw").exists(),
        "the staging folder is removed"
    );
    // The program was started as ttwright v0.3.0 expects it.
    let args = f.args();
    for word in [
        "install --json --no-detect --mpi",
        "--fo3",
        "--fnv",
        "--dest",
        "--events",
    ] {
        assert!(args.contains(word), "{args}");
    }
    assert!(
        !args.contains("--resume") && !args.contains("--dry-run"),
        "{args}"
    );
    // A second run is refused: the Mod is there.
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(2));
    assert_eq!(check_stream(code, &lines).0["error"]["code"], "mod_exists");
}

#[test]
fn every_ttwright_failure_maps_to_its_exit_code_and_leaves_the_instance_unchanged() {
    let f = Fixture::new("raul-ttw-codes");
    let before = f.state();
    for (mode, exit, status, code, ttwright_code, resumable) in [
        (
            "failed",
            4,
            "failed",
            "ttwright_failed",
            Some("check_failed"),
            true,
        ),
        (
            "usage",
            4,
            "failed",
            "ttwright_failed",
            Some("usage"),
            false,
        ),
        (
            "unsupported",
            2,
            "unsupported",
            "unsupported",
            Some("unsupported_version"),
            false,
        ),
        (
            "cancelled",
            4,
            "cancelled",
            "cancelled",
            Some("cancelled"),
            true,
        ),
        ("newer", 2, "refused", "contract_refused", None, false),
        ("garbage", 4, "failed", "tool_failed", None, false),
        ("empty", 4, "failed", "tool_failed", None, false),
    ] {
        f.mode(mode);
        let (got, lines) = f.ttw(&[]);
        assert_eq!(got, Some(exit), "{mode}: {lines:?}");
        let (result, finished) = check_stream(got, &lines);
        assert_eq!(finished["status"], status, "{mode}");
        assert_eq!(finished["code"], code, "{mode}");
        assert_eq!(finished["changed"], false, "{mode}");
        assert_eq!(
            result["error"]["ttwright_code"].as_str(),
            ttwright_code,
            "{mode}"
        );
        assert_eq!(result["resumable"], resumable, "{mode}");
        assert_eq!(result["mod"], Value::Null, "{mode}");
        assert_eq!(f.state(), before, "{mode} changed the Instance");
    }
}

#[test]
fn a_newer_ttwright_result_is_refused_with_a_plain_reason() {
    let f = Fixture::new("raul-ttw-newer");
    f.mode("newer");
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(2));
    let message = check_stream(code, &lines).1["message"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        message.contains("version 2") && message.contains("reads version 1"),
        "{message}"
    );
}

#[test]
fn resume_is_passed_through_and_continues_a_cancelled_install() {
    let f = Fixture::new("raul-ttw-resume");
    f.mode("cancelled");
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(4));
    let staging = PathBuf::from(check_stream(code, &lines).0["staging"].as_str().unwrap());
    assert!(staging.join("half").is_file(), "the work so far is kept");
    f.mode("success");
    let (code, lines) = f.ttw(&["--resume"]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, _) = check_stream(code, &lines);
    assert_eq!(result["resumed"], true);
    assert!(f.args().contains("--resume"));
    // The resumed run kept what the cancelled one wrote, and it is part of the Mod.
    assert!(
        f.instance()
            .join("mods/Tale of Two Wastelands/half")
            .is_file()
    );
    // Without --resume, a new run starts from an empty folder.
    let f = Fixture::new("raul-ttw-fresh");
    f.mode("cancelled");
    f.ttw(&[]);
    f.mode("success");
    assert_eq!(f.ttw(&[]).0, Some(0));
    assert!(
        !f.instance()
            .join("mods/Tale of Two Wastelands/half")
            .exists()
    );
}

#[test]
fn a_dry_run_passes_dry_run_and_writes_nothing() {
    let f = Fixture::new("raul-ttw-dry");
    let before = f.state();
    f.mode("success");
    let (code, lines) = f.ttw(&["--dry-run"]);
    assert_eq!(code, Some(0));
    let (result, finished) = check_stream(code, &lines);
    assert_eq!(result["dry_run"], true);
    assert_eq!(finished["changed"], false);
    assert!(f.args().contains("--dry-run"));
    assert_eq!(f.state(), before);
    assert!(!f.instance().join(".raul/ttw").exists());
}

#[test]
fn sigint_stops_ttwright_and_exits_4_with_nothing_added() {
    let f = Fixture::new("raul-ttw-sigint");
    let before = f.state();
    f.mode("slow");
    let (fo3, pkg, inst) = (f.fallout3(), f.path("TTW.mpi"), f.instance());
    let child = f
        .command(&[
            "ttw",
            "--json",
            "--instance",
            inst.to_str().unwrap(),
            "--fallout3",
            fo3.to_str().unwrap(),
            "--package",
            pkg.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Wait until the fake is running (it records its process id first).
    let pid_file = f.tools().join("pid");
    let start = Instant::now();
    while read(&pid_file).trim().is_empty() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "the fake never started"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let fake = read(&pid_file).trim().to_owned();
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
    let lines = lines(&out);
    let (result, finished) = check_stream(Some(4), &lines);
    assert_eq!(finished["status"], "cancelled");
    assert_eq!(result["resumable"], true);
    assert_eq!(result["mod"], Value::Null);
    // ttwright's process is gone, and nothing was added.
    let alive = Command::new("kill")
        .args(["-0", &fake])
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(!alive.success(), "ttwright is still running");
    assert_eq!(f.state(), before);
}

#[test]
fn missing_inputs_are_refused_before_ttwright_starts() {
    let f = Fixture::new("raul-ttw-refuse");
    f.mode("success");
    let (inst, fo3) = (f.instance(), f.fallout3());
    let run = |args: &[&str]| {
        let out = f.raul(args);
        let lines = lines(&out);
        let (result, _) = check_stream(out.status.code(), &lines);
        assert_eq!(lines.len(), 2, "only result and finished: {lines:?}");
        (
            out.status.code(),
            result["error"]["code"].as_str().unwrap().to_owned(),
        )
    };
    let inst = inst.to_str().unwrap();
    let fo3 = fo3.to_str().unwrap();
    assert_eq!(
        run(&["ttw", "--json", "--instance", inst]),
        (Some(2), "missing_input".into())
    );
    let nowhere = f.path("nowhere");
    assert_eq!(
        run(&["ttw", "--json", "--instance", nowhere.to_str().unwrap()]),
        (Some(2), "instance_invalid".into())
    );
    let pkg = f.path("TTW.mpi");
    let pkg = pkg.to_str().unwrap();
    let base = [
        "ttw",
        "--json",
        "--instance",
        inst,
        "--fallout3",
        fo3,
        "--package",
        pkg,
    ];
    assert_eq!(
        run(&[&base[..], &["--profile", "Nope"]].concat()),
        (Some(2), "profile_missing".into())
    );
    assert!(f.args().is_empty(), "ttwright was started");
    assert_eq!(f.raul(&["ttw", "--bogus"]).status.code(), Some(5));
    assert_eq!(
        f.raul(&["ttw", "--timeout-secs", "0"]).status.code(),
        Some(5)
    );
}

#[test]
fn the_inputs_come_from_raul_config_when_no_flag_names_them() {
    let f = Fixture::new("raul-ttw-config");
    f.mode("success");
    let storage = f.path("storage");
    for (key, value) in [
        ("game.fallout3", f.fallout3()),
        ("ttw.package", f.path("TTW.mpi")),
        ("storage.root", storage.clone()),
    ] {
        let out = f.raul(&["config", "set", key, value.to_str().unwrap()]);
        assert!(out.status.success(), "{out:?}");
    }
    let out = f.raul(&[
        "init",
        "--base",
        f.newvegas().to_str().unwrap(),
        "--instance",
        storage.join("instance").to_str().unwrap(),
    ]);
    assert!(out.status.success());
    f.profile(&storage.join("instance"));
    let out = f.raul(&["ttw"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("added `Tale of Two Wastelands`"));
    assert!(
        storage
            .join("instance/mods/Tale of Two Wastelands/Example.esm")
            .is_file()
    );
    let args = f.args();
    assert!(
        args.contains(f.fallout3().to_str().unwrap()) && args.contains("TTW.mpi"),
        "{args}"
    );
}

/// nvmod has its own `ttw`, and writes a run log for every command it runs: `raul ttw` leaves none.
#[test]
fn ttw_is_the_toolboxs_own_command_and_never_reaches_nvmod() {
    let f = Fixture::new("raul-ttw-dispatch");
    let logs = f.path("state/logs");
    let _ = std::fs::remove_dir_all(&logs);
    f.mode("success");
    assert_eq!(f.ttw(&[]).0, Some(0));
    assert_eq!(f.raul(&["ttw", "--fallout3"]).status.code(), Some(5));
    assert!(!logs.exists(), "nvmod ran");
}
