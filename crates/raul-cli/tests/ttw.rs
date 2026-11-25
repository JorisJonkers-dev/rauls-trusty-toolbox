// SPDX-License-Identifier: GPL-3.0-only
//! `raul ttw` with ttwright installed in process, on synthetic fixtures from `ttwright-testkit`: the success path, every
//! failure's exit code, an unsupported package, cancelling, `--resume`, a dry run, the inputs from `raul config`, and
//! that nvmod never receives the command. The fixtures are made up; no game data is involved.
#![cfg(unix)]
// These tests start the built binary and `kill`, which are not tool calls (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use raul_testkit::{covers, ids};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nvmod_testkit::{BaseBuilder, TempDir, snapshot, write_file};
use serde_json::{Value, json};
use ttwright_testkit::{Manifest, loc};

/// The package's own folder name inside the Instance's `mods`.
const MOD: &str = "Tale of Two Wastelands";

/// The fixture manifest plus two plugins copied from the game copy, so the Mod has plugins to enable.
fn manifest() -> Manifest {
    let mut m = ttwright_testkit::Fixture::manifest();
    m.asset(
        0,
        "",
        loc::GAME_DATA,
        loc::DEST,
        "loose.txt",
        Some("Example.esm"),
    )
    .asset(
        0,
        "",
        loc::GAME_DATA,
        loc::DEST,
        "loose.txt",
        Some("Example - Patch.esp"),
    );
    m
}

/// The same package with many small copies, so an install runs long enough to be stopped while it works.
fn slow_manifest() -> Manifest {
    let mut m = ttwright_testkit::Fixture::manifest();
    for i in 0..4000 {
        m.asset(
            0,
            "",
            loc::GAME_DATA,
            loc::DEST,
            "loose.txt",
            Some(&format!("slow/{i:04}.txt")),
        );
    }
    m
}

struct Fixture {
    root: TempDir,
    /// The ttwright fixture: a Fallout 3 copy, the `.mpi` package and a destination folder.
    ttw: ttwright_testkit::Fixture,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        Self::with_manifest(label, &manifest())
    }

    fn with_manifest(label: &str, manifest: &Manifest) -> Fixture {
        let root = TempDir::new(label);
        let f = Fixture {
            root,
            ttw: ttwright_testkit::Fixture::with_manifest(label, manifest),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.newvegas());
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
        self.ttw.fallout3.clone()
    }
    fn package(&self) -> PathBuf {
        self.ttw.mpi.clone()
    }
    fn instance(&self) -> PathBuf {
        self.path("instance")
    }
    /// The folder ttwright writes into, kept between a cancelled run and `--resume`.
    fn work(&self) -> PathBuf {
        self.instance().join(".raul/ttw")
    }

    fn command<S: AsRef<str>>(&self, args: &[S]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_raul"));
        c.args(args.iter().map(AsRef::<str>::as_ref))
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"));
        c
    }

    fn raul<S: AsRef<str>>(&self, args: &[S]) -> Output {
        self.command(args).output().expect("raul runs")
    }

    /// `raul ttw --json` with every input given as a flag, plus `extra`.
    fn ttw(&self, extra: &[&str]) -> (Option<i32>, Vec<Value>) {
        let out = self.raul(&with_extra(&self.ttw_args(), extra));
        (out.status.code(), lines(&out))
    }

    /// The arguments that name every input by flag.
    fn ttw_args(&self) -> Vec<String> {
        self.ttw_args_for(&self.package())
    }

    /// The same, with `package` as the TTW package.
    fn ttw_args_for(&self, package: &Path) -> Vec<String> {
        [
            "ttw",
            "--json",
            "--instance",
            self.instance().to_str().unwrap(),
            "--fallout3",
            self.fallout3().to_str().unwrap(),
            "--package",
            package.to_str().unwrap(),
        ]
        .map(str::to_owned)
        .into()
    }

    /// What the player downloads: a 7z that holds the package next to a made-up installer program and library.
    fn download(&self) -> PathBuf {
        let path = self.path("Downloads/Tale of Two Wastelands 3.4-133-3.4.0-[mod.pub].7z");
        make_7z(
            &path,
            &[
                (
                    "Tale of Two Wastelands 3.4.mpi",
                    &std::fs::read(self.package()).unwrap(),
                ),
                ("TTW_Installer.exe", b"MZ made-up installer program"),
                ("Lib/made-up.dll", b"MZ made-up library"),
            ],
        );
        path
    }

    /// `raul ttw --json` with the package given as `package`, plus `extra`.
    fn ttw_for(&self, package: &Path, extra: &[&str]) -> (Option<i32>, Vec<Value>) {
        let out = self.raul(&with_extra(&self.ttw_args_for(package), extra));
        (out.status.code(), lines(&out))
    }

    /// What a failed run must leave as it was: the Mods and the Profiles.
    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }

    /// Start `raul ttw` and stop it with SIGINT as soon as ttwright has written its first asset; returns its output.
    /// The slow manifest has thousands more assets (an install takes seconds), so the stop lands mid-install.
    fn ttw_interrupted(&self) -> Output {
        let child = self
            .command(&with_extra(&self.ttw_args(), &[]))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Wait for the first asset (polled every 5 ms), not just the journal that comes before it.
        let out_dir = self.work().join("out");
        let start = Instant::now();
        while written_assets(&out_dir) < 1 {
            assert!(
                start.elapsed() < Duration::from_secs(60),
                "ttwright never started writing"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let raul_pid = child.id().to_string();
        assert!(
            Command::new("kill")
                .args(["-INT", &raul_pid])
                .status()
                .unwrap()
                .success()
        );
        child.wait_with_output().unwrap()
    }
}

/// Write a 7z archive of `files` at `path`, making its folder.
fn make_7z(path: &Path, files: &[(&str, &[u8])]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut writer = sevenz_rust2::ArchiveWriter::create(path).unwrap();
    for (name, data) in files {
        writer
            .push_archive_entry(sevenz_rust2::ArchiveEntry::new_file(name), Some(*data))
            .unwrap();
    }
    writer.finish().unwrap();
}

/// `base` followed by `extra`, as the `&str` slice `Command::args` takes.
fn with_extra(base: &[String], extra: &[&str]) -> Vec<String> {
    base.iter()
        .cloned()
        .chain(extra.iter().map(|s| (*s).to_owned()))
        .collect()
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

/// Files under `dir`, counting links as one file.
fn count_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => count_files(&e.path()),
            _ => 1,
        })
        .sum()
}

/// The assets ttwright has written under `dir`: not its journal, and not the package's own files it unpacks into
/// `.mpi_package` before the first asset (that folder and the journal both come before any asset).
fn written_assets(dir: &Path) -> usize {
    count_files(dir)
        .saturating_sub(count_files(&dir.join(".mpi_package")))
        .saturating_sub(usize::from(dir.join(".ttwright-journal.json").is_file()))
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

#[test]
fn success_adds_one_mod_and_enables_its_plugins() {
    covers(&[ids::ttw::install::ADDS_ONE_MOD]);
    let f = Fixture::new("raul-ttw-success");
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, finished) = check_stream(code, &lines);
    assert_eq!(result["mod"], MOD);
    let mod_dir = f.instance().join("mods").join(MOD);
    // Six outputs of the fixture's assets (the archive counts once) and the two plugins.
    assert_eq!(result["files"], 8);
    assert_eq!(result["staging"], Value::Null);
    assert_eq!(result["ttwright"]["contract"], "ttwright-result");
    assert!(result["tool"]["version"].is_string(), "{result}");
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
    assert!(kinds.contains(&"progress"));
    assert_eq!(&kinds[kinds.len() - 3..], ["phase", "result", "finished"]);
    assert!(mod_dir.join("Example.esm").is_file());
    let plugins = read(&f.instance().join("profiles/Default/plugins.txt"));
    assert!(plugins.lines().any(|l| l == "Example.esm"), "{plugins}");
    assert!(read(&f.instance().join("profiles/Default/modlist.txt")).contains(&format!("+{MOD}")));
    assert!(!f.work().exists(), "the staging folder is removed");
    // A second run is refused: the Mod is there.
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(2));
    assert_eq!(check_stream(code, &lines).0["error"]["code"], "mod_exists");
}

#[test]
fn every_failure_maps_to_its_exit_code_and_leaves_the_instance_unchanged() {
    covers(&[ids::ttw::install::EXIT_CODES]);
    let f = Fixture::new("raul-ttw-codes");
    let before = f.state();
    // A game copy whose checked file is not the expected one: ttwright's check fails before anything is written.
    write_file(
        &f.fallout3().join("Data"),
        "loose.txt",
        b"not the expected file",
    );
    let (got, lines) = f.ttw(&[]);
    assert_eq!(got, Some(4), "{lines:?}");
    let (result, finished) = check_stream(got, &lines);
    assert_eq!(finished["status"], "failed");
    assert_eq!(finished["code"], "ttwright_failed");
    assert_eq!(finished["changed"], false);
    assert_eq!(result["error"]["ttwright_code"], "check_failed");
    assert_eq!(result["mod"], Value::Null);
    assert_eq!(f.state(), before, "the failed check changed the Instance");
    // A file that is neither a package nor an archive is refused before anything runs: exit 2, not a failed step.
    let f = Fixture::new("raul-ttw-badpkg");
    let before = f.state();
    std::fs::write(f.package(), b"not an archive").unwrap();
    let (got, lines) = f.ttw(&[]);
    assert_eq!(got, Some(2), "{lines:?}");
    let (result, finished) = check_stream(got, &lines);
    assert_eq!(finished["status"], "refused");
    assert_eq!(finished["code"], "missing_input");
    assert_eq!(result["error"]["ttwright_code"], Value::Null);
    assert_eq!(f.state(), before);
    assert!(!f.work().exists());
}

#[test]
fn an_unsupported_package_is_refused_with_exit_2() {
    let mut m = manifest();
    m.set(
        "Package",
        json!({"Title": "Tale of Two Wastelands", "Version": "9.9.9", "Author": "synthetic"}),
    );
    let f = Fixture::with_manifest("raul-ttw-unsupported", &m);
    let before = f.state();
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(2), "{lines:?}");
    let (result, finished) = check_stream(code, &lines);
    assert_eq!(finished["status"], "unsupported");
    assert_eq!(finished["code"], "unsupported");
    assert_eq!(result["error"]["ttwright_code"], "unsupported_version");
    assert_eq!(result["resumable"], false);
    assert_eq!(f.state(), before);
}

#[test]
fn resume_is_passed_through_and_continues_a_cancelled_install() {
    covers(&[ids::ttw::install::RESUME]);
    let f = Fixture::with_manifest("raul-ttw-resume", &slow_manifest());
    let out = f.ttw_interrupted();
    assert_eq!(
        out.status.code(),
        Some(4),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let (result, _) = check_stream(Some(4), &lines(&out));
    assert_eq!(result["resumable"], true);
    assert!(
        f.work().join("out/.ttwright-journal.json").is_file(),
        "the journal is kept"
    );
    let (code, lines) = f.ttw(&["--resume"]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, _) = check_stream(code, &lines);
    assert_eq!(result["resumed"], true);
    // The resumed run finishes the package, so the Mod has every asset the slow manifest copies.
    assert!(
        f.instance()
            .join("mods")
            .join(MOD)
            .join("slow/3999.txt")
            .is_file()
    );
}

#[test]
fn without_resume_a_new_run_starts_from_an_empty_folder() {
    let f = Fixture::with_manifest("raul-ttw-fresh", &slow_manifest());
    let out = f.ttw_interrupted();
    assert_eq!(out.status.code(), Some(4));
    let (code, lines) = f.ttw(&[]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, _) = check_stream(code, &lines);
    assert_eq!(result["resumed"], false);
}

#[test]
fn sigint_stops_ttwright_and_exits_4_with_nothing_added() {
    covers(&[ids::ttw::install::STOP]);
    let f = Fixture::with_manifest("raul-ttw-sigint", &slow_manifest());
    let before = f.state();
    let out = f.ttw_interrupted();
    assert_eq!(
        out.status.code(),
        Some(4),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let (result, finished) = check_stream(Some(4), &lines(&out));
    assert_eq!(finished["status"], "cancelled");
    assert_eq!(result["resumable"], true);
    assert_eq!(result["mod"], Value::Null);
    assert_eq!(f.state(), before);
}

/// `--timeout-secs` cancels the install like SIGINT does: the run ends `cancelled` (resumable), not `failed`.
#[test]
fn a_dry_run_reads_and_checks_and_writes_nothing() {
    covers(&[ids::ttw::install::DRY_RUN]);
    let f = Fixture::new("raul-ttw-dry");
    let before = f.state();
    let (code, lines) = f.ttw(&["--dry-run"]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, finished) = check_stream(code, &lines);
    assert_eq!(result["dry_run"], true);
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before);
    assert!(!f.work().exists());
}

#[test]
fn missing_inputs_are_refused_before_ttwright_starts() {
    let f = Fixture::new("raul-ttw-refuse");
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
    let pkg = f.package();
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
    assert!(!f.work().exists(), "ttwright was started");
    assert_eq!(f.raul(&["ttw", "--bogus"]).status.code(), Some(5));
    assert_eq!(
        f.raul(&["ttw", "--timeout-secs", "0"]).status.code(),
        Some(5)
    );
}

#[test]
fn the_inputs_come_from_raul_config_when_no_flag_names_them() {
    covers(&[ids::ttw::install::INPUTS_FROM_CONFIG]);
    let f = Fixture::new("raul-ttw-config");
    let storage = f.path("storage");
    for (key, value) in [
        ("game.fallout3", f.fallout3()),
        ("ttw.package", f.package()),
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
    assert!(String::from_utf8_lossy(&out.stdout).contains(&format!("added `{MOD}`")));
    assert!(
        storage
            .join("instance/mods")
            .join(MOD)
            .join("Example.esm")
            .is_file()
    );
}

/// `--ttwright` named the program raul used to run. raul runs no program now, so the flag is an unknown option:
/// exit code 5, nothing is started.
#[test]
fn the_ttwright_flag_is_refused_as_an_unknown_option() {
    let f = Fixture::new("raul-ttw-flag");
    let out = f.raul(&["ttw", "--ttwright", "/x"]);
    assert_eq!(out.status.code(), Some(5), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("raul: unknown option --ttwright"),
        "{out:?}"
    );
    assert!(!f.work().exists(), "ttwright was started");
}

/// nvmod has its own `ttw`, and writes a run log for every command it runs: `raul ttw` leaves none.
#[test]
fn ttw_is_the_toolboxs_own_command_and_never_reaches_nvmod() {
    let f = Fixture::new("raul-ttw-dispatch");
    let logs = f.path("state/logs");
    let _ = std::fs::remove_dir_all(&logs);
    assert_eq!(f.ttw(&[]).0, Some(0));
    assert_eq!(f.raul(&["ttw", "--fallout3"]).status.code(), Some(5));
    assert!(!logs.exists(), "nvmod ran");
}

#[test]
fn the_download_archive_is_accepted_and_its_package_is_unpacked_into_the_work_folder_and_removed() {
    covers(&[ids::ttw::install::DOWNLOAD_ARCHIVE]);
    let f = Fixture::new("raul-ttw-download");
    let download = f.download();
    let before = f.state();
    // The dry run unpacks the package, reads it and leaves no folder behind.
    let (code, lines) = f.ttw_for(&download, &["--dry-run"]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, _) = check_stream(code, &lines);
    assert_eq!(result["dry_run"], true);
    let phases: Vec<&str> = lines
        .iter()
        .filter(|l| l["event"] == "phase")
        .map(|l| l["phase"].as_str().unwrap())
        .collect();
    assert_eq!(phases.first(), Some(&"unpack"), "{lines:?}");
    assert_eq!(phases.iter().filter(|p| **p == "unpack").count(), 1);
    assert_eq!(f.state(), before);
    assert!(!f.work().exists(), "a dry run leaves no work folder");
    // The install runs from the download, adds the Mod and removes what it unpacked.
    let (code, lines) = f.ttw_for(&download, &[]);
    assert_eq!(code, Some(0), "{lines:?}");
    let (result, finished) = check_stream(code, &lines);
    assert_eq!(result["mod"], MOD);
    assert_eq!(finished["changed"], true);
    assert!(
        f.instance()
            .join("mods")
            .join(MOD)
            .join("Example.esm")
            .is_file()
    );
    assert!(!f.work().exists(), "the unpacked package is removed");
}

#[test]
fn an_interrupted_install_from_the_download_keeps_the_unpacked_package_for_resume() {
    let f = Fixture::with_manifest("raul-ttw-download-resume", &slow_manifest());
    let download = f.download();
    let child = f
        .command(&f.ttw_args_for(&download))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let out_dir = f.work().join("out");
    let start = Instant::now();
    while written_assets(&out_dir) < 1 {
        assert!(
            start.elapsed() < Duration::from_secs(60),
            "ttwright never started"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        Command::new("kill")
            .args(["-INT", &child.id().to_string()])
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
    let kept = f.work().join("package/Tale of Two Wastelands 3.4.mpi");
    assert!(kept.is_file(), "the unpacked package is kept for --resume");
    // Resuming does not unpack again: the package is taken from the work folder, and the run ends with it removed.
    let (code, lines) = f.ttw_for(&download, &["--resume"]);
    assert_eq!(code, Some(0), "{lines:?}");
    assert!(
        lines.iter().all(|l| l["phase"] != "unpack"),
        "unpacked again: {lines:?}"
    );
    assert!(!f.work().exists());
}

#[test]
fn a_patch_archive_is_refused_with_exit_2_before_anything_is_written() {
    covers(&[ids::ttw::install::PATCH_ARCHIVE_REFUSED]);
    let f = Fixture::new("raul-ttw-patch");
    let patch = f.path("Downloads/YUPTTW 13.9.1-133-3.4.0-2026.05.09-[mod.pub].7z");
    make_7z(
        &patch,
        &[
            ("YUPTTW.esm", b"TES4 made up"),
            (
                "Fallout - Meshes.bsa",
                b"BSA\0 a Mod's archive, not a package",
            ),
            ("YUPTTW.esm.override", b""),
        ],
    );
    let before = f.state();
    let out = f.raul(&with_extra(&f.ttw_args_for(&patch), &[]));
    let lines = lines(&out);
    let (result, finished) = check_stream(out.status.code(), &lines);
    assert_eq!(out.status.code(), Some(2), "{lines:?}");
    assert_eq!(result["error"]["code"], "missing_input");
    assert_eq!(finished["status"], "refused");
    assert_eq!(lines.len(), 2, "refused before anything ran: {lines:?}");
    let message = result["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("Tale of Two Wastelands <version>.mpi"),
        "{message}"
    );
    assert_eq!(f.state(), before);
    assert!(!f.work().exists());
}

#[test]
fn a_file_that_is_neither_a_package_nor_an_archive_is_refused_with_exit_2() {
    let f = Fixture::new("raul-ttw-neither");
    let text = f.path("Downloads/notes.txt");
    std::fs::create_dir_all(text.parent().unwrap()).unwrap();
    std::fs::write(&text, "not a package").unwrap();
    let (code, lines) = f.ttw_for(&text, &[]);
    assert_eq!(code, Some(2), "{lines:?}");
    let (result, _) = check_stream(code, &lines);
    let message = result["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("`BSA\\0`") && message.contains(".7z"),
        "{message}"
    );
    assert!(!f.work().exists());
}

#[test]
fn a_failed_dry_run_leaves_no_work_folder_and_the_text_has_no_empty_checksum() {
    let f = Fixture::new("raul-ttw-dry-failed");
    // Starts like a package, is not one: ttwright refuses it after the work folder was made.
    let broken = f.path("Downloads/broken.mpi");
    std::fs::create_dir_all(broken.parent().unwrap()).unwrap();
    std::fs::write(&broken, b"BSA\0 broken").unwrap();
    let args: Vec<String> = f
        .ttw_args_for(&broken)
        .into_iter()
        .filter(|a| a != "--json")
        .chain(["--dry-run".to_owned()])
        .collect();
    let out = f.raul(&args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(4), "{stderr}");
    assert!(
        !f.work().exists(),
        "a dry run leaves nothing behind: {stderr}"
    );
    assert!(!stderr.contains("SHA-256"), "{stderr}");
    assert!(stderr.contains("running ttwright inside"), "{stderr}");
}

/// One asset that cannot be copied (its source is unreadable) fails the install: plain `raul ttw` names it and the file
/// in the state folder that holds the details, so nobody has to repeat a run with `--json` to learn why.
#[test]
fn a_failed_asset_is_named_in_the_plain_output_and_in_a_file_in_the_state_folder() {
    use std::os::unix::fs::PermissionsExt;
    let mut m = manifest();
    m.asset(
        0,
        "",
        loc::GAME_DATA,
        loc::DEST,
        "unreadable-marker.txt",
        Some("unreadable-marker.txt"),
    );
    let f = Fixture::with_manifest("raul-ttw-asset-failed", &m);
    let source = f.fallout3().join("Data/unreadable-marker.txt");
    std::fs::write(&source, b"made up").unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o000)).unwrap();
    if std::fs::read(&source).is_ok() {
        // Running as a user that reads everything (root): the asset cannot be made to fail this way.
        return;
    }
    let args: Vec<String> = f.ttw_args().into_iter().filter(|a| a != "--json").collect();
    let out = f.raul(&args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(4), "{stderr}");
    assert!(stderr.contains("unreadable-marker.txt"), "{stderr}");
    let log = f.path("state/logs/ttw-last-failure.txt");
    assert!(
        stderr.contains(&log.display().to_string()),
        "the file is named: {stderr}"
    );
    let kept = std::fs::read_to_string(&log).unwrap();
    assert!(kept.contains("unreadable-marker.txt"), "{kept}");
}
