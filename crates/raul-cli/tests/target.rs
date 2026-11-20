// SPDX-License-Identifier: GPL-3.0-only
//! `raul target` through the command line entry: nvmod's command, plus the linked engine's capabilities written into
//! the Instance when nv-rs becomes the target (#398). `raul install-nv-rs` is gone and never reaches nvmod's fetching
//! command of the same name.

use std::path::PathBuf;

use nvmod_testkit::{BaseBuilder, TempDir};
use raul_cli::run;

const CAPABILITIES: &str = r#"{"schema_version":1,"extenders":[]}"#;

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(|w| (*w).to_owned()).collect()
}

fn instance(t: &TempDir, name: &str) -> PathBuf {
    let base = t.path().join(format!("{name}-base"));
    BaseBuilder::steam().write(&base);
    let root = t.path().join(name);
    let made = nvmod_api::init::create(
        &nvmod_api::init::CreateRequest {
            instance: root.clone(),
            base,
        },
        &mut nvmod_api::NoProgress,
        &nvmod_api::CancelToken::new(),
    );
    assert!(made.is_ok(), "{made:?}");
    root
}

fn go(args: &[&str]) -> (u8, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&words(args), &mut out, &mut err);
    (code, String::from_utf8(err).unwrap())
}

#[test]
fn the_nv_rs_target_writes_the_capabilities_the_engine_provided_and_vanilla_writes_none() {
    raul_core::capabilities::provide(CAPABILITIES);
    let t = TempDir::new("target-capabilities");
    let (nv_rs, vanilla) = (instance(&t, "nv-rs"), instance(&t, "vanilla"));
    let (code, err) = go(&["target", "nv-rs", "--instance", nv_rs.to_str().unwrap()]);
    assert_eq!(code, raul_cli::OK, "{err}");
    assert_eq!(
        std::fs::read_to_string(nv_rs.join("nv-rs/capabilities.json")).unwrap(),
        CAPABILITIES
    );
    let (code, err) = go(&["target", "vanilla", "--instance", vanilla.to_str().unwrap()]);
    assert_eq!(code, raul_cli::OK, "{err}");
    assert!(!vanilla.join("nv-rs").exists());
}

#[test]
fn a_refused_target_writes_nothing() {
    let t = TempDir::new("target-refused");
    let missing = t.path().join("no-such-instance");
    let (code, _) = go(&["target", "nv-rs", "--instance", missing.to_str().unwrap()]);
    assert_ne!(code, raul_cli::OK);
    assert!(!missing.exists());
}

#[test]
fn install_nv_rs_is_gone_and_nvmod_s_command_is_not_reached() {
    let t = TempDir::new("install-gone");
    let root = instance(&t, "fnv");
    let (code, err) = go(&["install-nv-rs", "--instance", root.to_str().unwrap()]);
    assert_eq!(code, raul_cli::USAGE, "{err}");
    assert!(err.contains("gone"), "{err}");
    assert!(!root.join("nv-rs").exists());
}
