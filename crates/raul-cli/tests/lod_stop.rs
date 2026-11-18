// SPDX-License-Identifier: GPL-3.0-only
//! A stop request (Ctrl+C, SIGTERM, the GUI's cancel: `nvmod_core::cancel::request`) during `raul lod generate` stops
//! the generation at its next safe point, and adds nothing to the Instance. The stop request is process-wide, so this
//! file is its own test binary, and the Instance is made by the built `raul` binary as a black box.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

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
    // The library itself stopped: its answer to a cancel is an interrupted manifest, with no request digest.
    let manifest = std::fs::read(f.instance().join(".raul/lod/out/manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
    assert_eq!(manifest["request_digest"], "", "{manifest}");

    let outcome = lod::generate_with_cancel(&f.request(), &CancelToken::new(), &mut |_| {});
    assert_eq!(outcome.exit_code, 0, "{:?}", outcome.result);
    assert!(outcome.changed);
}
