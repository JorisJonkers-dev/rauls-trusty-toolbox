// SPDX-License-Identifier: GPL-3.0-only
//! `raul update status|download|install|rollback|self-check` through the command line entry. The test program is a
//! development build, which is no package `raul` can replace, so these cases cover what the commands say and refuse
//! there; the steps themselves are tested in raul-core (`selfupdate::flow`) with synthetic releases.

use raul_cli::{FAILED, OK, REFUSED, USAGE, run};
use serde_json::Value;

fn go(args: &[&str]) -> (u8, String, String) {
    let args: Vec<String> = args.iter().map(|w| (*w).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn status_says_a_development_build_cannot_update_itself_and_fetches_nothing() {
    let (code, out, _) = go(&["update", "status"]);
    assert_eq!(code, OK);
    assert!(out.contains("new version from the release page"), "{out}");
    let (code, out, _) = go(&["update", "status", "--json"]);
    assert_eq!(code, OK);
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(doc["status"]["updatable"], false);
    assert_eq!(doc["current"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn download_install_and_rollback_are_refused_with_a_reason_before_anything_is_fetched() {
    for command in ["download", "install", "rollback"] {
        let (code, out, err) = go(&["update", command, "--json"]);
        assert_eq!(code, REFUSED, "{command}: {err}");
        let doc: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(doc["ok"], false);
        assert!(
            err.contains("new version from the release page"),
            "{command}: {err}"
        );
        for word in ["curl", "sha256", "SHA256SUMS"] {
            assert!(!err.contains(word), "{command}: {err}");
        }
    }
}

#[test]
fn self_check_passes_for_this_build_and_fails_for_another_expected_version() {
    let (code, out, _) = go(&["update", "self-check", "--json"]);
    assert_eq!(code, OK, "{out}");
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        (doc["ok"].as_bool(), doc["version"].as_str()),
        (Some(true), Some(env!("CARGO_PKG_VERSION")))
    );
    let (code, out, _) = go(&["update", "self-check", "--expect", "99.0.0", "--json"]);
    assert_eq!(code, FAILED);
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(doc["ok"], false);
    assert!(
        doc["problems"][0]
            .as_str()
            .unwrap()
            .contains("not the version 99.0.0")
    );
}

#[test]
fn a_mistyped_update_command_is_a_usage_error() {
    for args in [
        &["update", "download", "--nope"][..],
        &["update", "download", "--tag"],
        &["update", "self-check", "--expect"],
        &["update", "install", "--now"],
        &["update", "bogus"],
    ] {
        let (code, _, err) = go(args);
        assert_eq!(code, USAGE, "{args:?}: {err}");
    }
}

#[test]
fn the_help_lists_every_update_command() {
    let (_, out, _) = go(&["help"]);
    for command in [
        "update status",
        "update download",
        "update install",
        "update rollback",
        "update self-check",
    ] {
        assert!(out.contains(command), "{command}");
    }
    assert!(out.contains("--allow-older"));
}
