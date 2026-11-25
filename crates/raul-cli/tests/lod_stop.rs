// SPDX-License-Identifier: GPL-3.0-only
//! A stop request (Ctrl+C, SIGTERM, the GUI's cancel: `nvmod_core::cancel::request`) during `raul lod generate` stops
//! the generation at its next safe point, and adds nothing to the Instance. The stop request is process-wide, so this
//! file is its own test binary, and the Instance is made by the built `raul` binary as a black box.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use raul_testkit::{covers, ids};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use lodwright_api::CancelToken;
use lodwright_testkit::land::Heights;
use lodwright_testkit::world::{SyntheticWorld, world_plugin};
use nvmod_testkit::{BaseBuilder, TempDir, snapshot};
use raul_drivers::events::{Event, Status};
use raul_drivers::lod::{self, LodRequest};

struct Fixture {
    root: TempDir,
}

impl Fixture {
    /// An Instance whose game master is a synthetic world of four cells with terrain, one thread for the run.
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
        std::fs::write(
            f.instance().join("profiles/Default/raul-lod.toml"),
            "threads = 1\n",
        )
        .unwrap();
        f
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn instance(&self) -> PathBuf {
        self.path("instance")
    }

    fn raul(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"))
            .output()
            .expect("raul runs")
    }

    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }

    fn request(&self) -> LodRequest {
        LodRequest {
            instance: self.instance(),
            profile: None,
            worlds: Vec::new(),
            dry_run: false,
            allow_in_repo: false,
            timeout: Duration::from_secs(600),
        }
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
fn a_stop_request_during_the_generation_adds_nothing_and_the_next_run_completes() {
    let f = Fixture::new("raul-lodstop");
    let before = f.state();
    // The stop request arrives with the first finished block. The emit does not return until the watcher has cancelled
    // the token, and the call waits for the emit before it goes on, so the cancel is in place at its next safe point.
    let cancel = CancelToken::new();
    let watched = cancel.clone();
    let mut requested = false;
    let outcome = lod::generate_with_cancel(&f.request(), &cancel, &mut |e: Event| {
        if matches!(e, Event::Progress { .. }) && !requested {
            requested = true;
            nvmod_core::cancel::request();
            let waited = Instant::now();
            while !watched.is_cancelled() {
                assert!(
                    waited.elapsed() < Duration::from_secs(60),
                    "the watcher never cancelled the call"
                );
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    });
    nvmod_core::cancel::reset();
    assert_eq!(outcome.exit_code, 4, "{:?}", outcome.result);
    assert_eq!(outcome.result.status, Status::Cancelled);
    assert_eq!(outcome.result.mod_name, None);
    assert!(!outcome.changed);
    assert_eq!(f.state(), before, "a stopped run changed the Instance");
    // The library itself stopped: it kept a checkpoint in the staging folder and wrote no output folder.
    let work = f.instance().join(".raul/lod/Default");
    assert!(
        !work.join("out").exists(),
        "the output is written only at the swap"
    );
    assert!(
        work.join("out.lodwright-staging/checkpoint.json").is_file(),
        "the cancel kept a checkpoint"
    );

    let outcome = lod::generate_with_cancel(&f.request(), &CancelToken::new(), &mut |_| {});
    assert_eq!(outcome.exit_code, 0, "{:?}", outcome.result);
    assert!(outcome.changed);
}

#[test]
fn a_run_that_outlasts_its_timeout_is_stopped_and_leaves_the_mod_alone() {
    covers(&[ids::lod::generate::TIMEOUT]);
    let f = Fixture::new("raul-lodtimeout");
    let before = f.state();
    let mut request = f.request();
    request.timeout = Duration::from_millis(1);
    let outcome = lod::generate_with_cancel(&request, &CancelToken::new(), &mut |_| {});
    assert_eq!(outcome.exit_code, 4, "{:?}", outcome.result);
    assert_eq!(outcome.result.status, Status::Failed);
    let message = outcome.result.error.as_ref().map(|e| e.message.clone());
    let message = message.unwrap_or_default();
    assert!(message.contains("longer than"), "{message}");
    assert!(message.contains("was not touched"), "{message}");
    assert_eq!(f.state(), before, "a timed out run changed the Instance");
}

/// A process kill (no chance to clean up) in the middle of a run leaves the LOD Mod and its place in the Profile as
/// they were; the next run with the same settings finishes and replaces the Mod only at its end.
#[test]
fn a_killed_run_leaves_the_existing_lod_mod_alone_and_the_next_run_finishes() {
    let f = Fixture::new("raul-lodkill");
    // The built binary, not the library: a stop request in the other tests of this file is process-wide.
    let first = f.raul(&["lod", "generate", "--instance", s(&f.instance())]);
    assert!(first.status.success(), "{first:?}");
    let first_state = f.state();
    // Other settings make the LOD out of date, so the next run builds again.
    std::fs::write(
        f.instance().join("profiles/Default/raul-lod.toml"),
        "threads = 1\n[output]\ntrees = false\n",
    )
    .unwrap();
    let before = f.state();

    let mut child = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["lod", "generate", "--instance", s(&f.instance())])
        .env("RAUL_CONFIG_DIR", f.path("config"))
        .env("NVMOD_STATE_DIR", f.path("state"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let staging = f.instance().join(".raul/lod/Default/out.lodwright-staging");
    let waited = Instant::now();
    while !staging.is_dir() {
        assert!(
            waited.elapsed() < Duration::from_secs(60),
            "the run never started its staging folder"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    child.kill().unwrap();
    let status = child.wait().unwrap();
    assert!(
        !status.success(),
        "the run finished before the kill: {status:?}"
    );
    assert_eq!(
        f.state(),
        before,
        "a killed run changed the Mod or the Profile"
    );

    let again = f.raul(&["lod", "generate", "--instance", s(&f.instance())]);
    assert!(again.status.success(), "{again:?}");
    assert_ne!(
        f.state(),
        first_state,
        "the next run did not replace the Mod"
    );
    assert!(
        !f.instance()
            .join(".raul/lod/Default/out.lodwright-staging")
            .exists(),
        "the staging folder is gone after the swap"
    );
}
