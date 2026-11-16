// SPDX-License-Identifier: GPL-3.0-only
//! ADR-0007: `std::process::Command` is used in `raul-runner` and nowhere else in the Toolbox's shipped code.
//! Clippy's `disallowed-methods` (see `clippy.toml`) catches a new `Command::new`; this test also catches the other
//! ways in (`Command::from`, a renamed import, an `#[allow]` that silences the lint), and it runs under plain
//! `cargo test`.

use std::path::{Path, PathBuf};

/// Source files outside the runner crate that may still start a process, with the reason. Empty: every process
/// the Toolbox starts goes through the Runner.
const EXCEPTIONS: &[(&str, &str)] = &[];

/// The crate that may start processes.
const RUNNER: &str = "raul-runner";

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Whether this source text starts a process or silences the lint that forbids it.
fn starts_a_process(text: &str) -> bool {
    text.contains("process::Command")
        || text.contains("process::{") && text.contains("Command")
        || text.contains("disallowed_methods")
}

/// The files under each crate of `crates` (its `src` and its `build.rs`) that start a process, as
/// `<crate>/<path>`, leaving out the runner and `exceptions`.
fn offenders(crates: &Path, exceptions: &[(&str, &str)]) -> Vec<String> {
    let mut found = Vec::new();
    for krate in std::fs::read_dir(crates).unwrap().flatten() {
        let name = krate.file_name().to_string_lossy().into_owned();
        if name == RUNNER || !krate.path().is_dir() {
            continue;
        }
        let mut files = Vec::new();
        // Only shipped code: integration tests start the built binary, which is not a tool call.
        rust_files(&krate.path().join("src"), &mut files);
        let build = krate.path().join("build.rs");
        if build.is_file() {
            files.push(build);
        }
        for file in files {
            let relative = format!(
                "{name}/{}",
                file.strip_prefix(krate.path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            );
            if exceptions.iter().any(|(path, _)| *path == relative) {
                continue;
            }
            if starts_a_process(&std::fs::read_to_string(&file).unwrap()) {
                found.push(relative);
            }
        }
    }
    found.sort();
    found
}

fn toolbox_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn only_the_runner_crate_uses_process_command() {
    let found = offenders(&toolbox_crates(), EXCEPTIONS);
    assert!(
        found.is_empty(),
        "these files use std::process::Command or silence its lint; go through raul_runner::Runner instead: {found:?}"
    );
}

#[test]
fn every_exception_still_exists() {
    let crates = toolbox_crates();
    for (path, _) in EXCEPTIONS {
        assert!(
            crates.join(path).is_file(),
            "{path} is gone; drop it from EXCEPTIONS"
        );
    }
}

/// A synthetic crates folder: `files` are `(path under the folder, text)`.
fn tree(name: &str, files: &[(&str, &str)]) -> nvmod_testkit::TempDir {
    let dir = nvmod_testkit::TempDir::new(name);
    for (path, text) in files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

#[test]
fn the_guard_finds_every_way_to_start_a_process() {
    let dir = tree(
        "guard-catches",
        &[
            (
                "raul-a/src/lib.rs",
                "use std::process::Command;\nfn f() { Command::new(\"x\"); }",
            ),
            (
                "raul-b/src/deep/mod.rs",
                "use std::process::{Command as C, Stdio};",
            ),
            (
                "raul-c/src/lib.rs",
                "fn f() { std::process::Command::from(todo!()); }",
            ),
            (
                "raul-d/src/lib.rs",
                "#[allow(clippy::disallowed_methods)]\nfn f() {}",
            ),
            ("raul-e/build.rs", "use std::process::Command;"),
        ],
    );
    assert_eq!(
        offenders(dir.path(), &[]),
        [
            "raul-a/src/lib.rs",
            "raul-b/src/deep/mod.rs",
            "raul-c/src/lib.rs",
            "raul-d/src/lib.rs",
            "raul-e/build.rs",
        ]
    );
}

#[test]
fn the_guard_leaves_the_runner_tests_and_exceptions_alone() {
    let dir = tree(
        "guard-allows",
        &[
            ("raul-runner/src/runner.rs", "use std::process::Command;"),
            (
                "raul-cli/tests/cli.rs",
                "use std::process::Command; // starts the built binary",
            ),
            ("raul-cli/src/lib.rs", "use raul_runner::{Call, Runner};"),
            ("raul-x/src/listed.rs", "use std::process::Command;"),
        ],
    );
    assert_eq!(
        offenders(dir.path(), &[("raul-x/src/listed.rs", "a reason")]),
        Vec::<String>::new()
    );
    assert_eq!(offenders(dir.path(), &[]), ["raul-x/src/listed.rs"]);
}
