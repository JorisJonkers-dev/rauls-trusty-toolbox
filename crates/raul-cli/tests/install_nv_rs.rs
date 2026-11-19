// SPDX-License-Identifier: GPL-3.0-only
//! `raul install-nv-rs` through the command line entry: it is raul's own command, it puts in the engine that ships
//! with raul and never fetches one. On the test build no engine ships next to the program, so every run that gets
//! as far as the engine says so.

use std::path::PathBuf;

use nvmod_testkit::{BaseBuilder, TempDir};
use raul_cli::{Route, route, run};

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(|w| (*w).to_owned()).collect()
}

fn instance(t: &TempDir) -> PathBuf {
    let base = t.path().join("base");
    BaseBuilder::steam().write(&base);
    let root = t.path().join("fnv");
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

fn go(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&words(args), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn it_is_one_of_raul_s_own_commands() {
    assert_eq!(route(&words(&["install-nv-rs"])), Route::Own);
}

#[test]
fn no_instance_is_a_usage_error() {
    let (code, _, err) = go(&["install-nv-rs"]);
    assert_eq!(code, raul_cli::USAGE, "{err}");
    assert!(err.contains("--instance"), "{err}");
}

#[test]
fn a_copy_without_an_engine_is_refused_with_plain_words_and_nothing_is_installed() {
    // A development build has no engine next to it; a build that has one is not this test's subject.
    if raul_core::setup::packaged::locate(&std::env::current_exe().unwrap()).is_some() {
        return;
    }
    let t = TempDir::new("install-nv-rs-missing");
    let root = instance(&t);
    let (code, out, err) = go(&["install-nv-rs", "--instance", root.to_str().unwrap()]);
    assert_eq!(code, raul_cli::REFUSED, "{out}{err}");
    assert!(err.contains("install raul again"), "{err}");
    assert!(!root.join("nv-rs").exists());
    assert!(!root.join(".nv-rs-staging").exists());
}

#[test]
fn the_options_that_fetched_a_release_are_a_usage_error_with_a_reason() {
    let t = TempDir::new("install-nv-rs-old-options");
    let root = instance(&t);
    let root = root.to_str().unwrap();
    for option in [
        &["--release"][..],
        &["--release-url", "http://127.0.0.1:9/none"],
        &["--retry-delay-ms", "0"],
        &["--gh", "gh"],
        &["--curl", "curl"],
        &["--asset", "a"],
        &["--repo", "r"],
    ] {
        let mut args = vec!["install-nv-rs", "--instance", root];
        args.extend(option);
        let (code, _, err) = go(&args);
        assert_eq!(code, raul_cli::USAGE, "{option:?}: {err}");
        assert!(
            err.contains("is gone") && err.contains("never downloaded"),
            "{err}"
        );
    }
    assert!(!std::path::Path::new(root).join("nv-rs").exists());
}

#[test]
fn a_cargo_without_a_source_is_a_usage_error() {
    let (code, _, err) = go(&["install-nv-rs", "--instance", "/i", "--cargo", "/c"]);
    assert_eq!(code, raul_cli::USAGE, "{err}");
    assert!(err.contains("--source"), "{err}");
}

#[test]
fn a_failed_check_ends_with_one_plain_headline_line() {
    // Detail lines come first and the last line is the headline, which is what the wizard shows.
    let e = nvmod_api::JobError::new(
        nvmod_api::ErrorKind::Failed,
        "x",
        "The engine's check did not pass (exit status 1).\nfirst\nsecond",
    );
    let mut err = Vec::new();
    let code = raul_cli::print_install_failure(&e, &mut err);
    assert_eq!(code, raul_cli::FAILED);
    let text = String::from_utf8(err).unwrap();
    let last = text.lines().last().unwrap();
    assert!(
        last.starts_with("raul install-nv-rs: failed and was rolled back: The engine's check"),
        "{text}"
    );
    assert!(text.starts_with("first\nsecond\n"), "{text}");
}

#[test]
fn an_unknown_option_is_a_usage_error() {
    let (code, _, err) = go(&["install-nv-rs", "--instance", "/i", "--nope", "x"]);
    assert_eq!(code, raul_cli::USAGE, "{err}");
}

#[test]
fn a_held_instance_lock_refuses_it() {
    let t = TempDir::new("install-nv-rs-locked");
    let root = instance(&t);
    let _held = raul_core::instance_lock::InstanceGuard::acquire(&root, "test").unwrap();
    let (code, _, err) = go(&["install-nv-rs", "--instance", root.to_str().unwrap()]);
    assert_eq!(code, raul_cli::REFUSED, "{err}");
    assert!(err.contains("in use"), "{err}");
}
