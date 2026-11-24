// SPDX-License-Identifier: GPL-3.0-only
//! ADR-0007, as narrowed by ADR-0016 and ADR-0019: `std::process::Command` is used in `raul-runner` and nowhere else in
//! the Toolbox's shipped code or its tests. Clippy's `disallowed-methods` (see `clippy.toml`) catches a new
//! `Command::new`; this test also catches the other ways in (`Command::from`, a renamed import, an `#[allow]` that
//! silences the lint), and it runs under plain `cargo test`.

use std::path::{Path, PathBuf};

/// Files outside the runner crate that may still use `std::process::Command`, with the reason. Each entry is a test
/// that starts the built `raul` binary as a black box, or signals it, and is not a tool call. The game run and the
/// Windows-only tool (`taskkill`) start through `raul-runner`, so they need no entry.
const ALLOWED: &[(&str, &str)] = &[
    (
        "raul-cli/tests/addons.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-cli/tests/cli.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-cli/tests/config.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-cli/tests/run.rs",
        "runs the built raul binary as a black box, with a stand-in program as the viewer",
    ),
    (
        "raul-cli/tests/scene.rs",
        "runs the built raul binary as a black box, with a fake scene tool",
    ),
    (
        "raul-cli/tests/lod.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-cli/tests/lod_generate.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-cli/tests/lod_settings.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-cli/tests/lod_stop.rs",
        "runs the built raul binary as a black box, to make the Instance",
    ),
    (
        "raul-cli/tests/ttw.rs",
        "runs the built raul binary and sends it SIGINT to test cancellation",
    ),
    (
        "raul-gui/tests/binary.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "raul-gui/tests/registry.rs",
        "runs the built raul binary as a black box",
    ),
    (
        "tools/coverage-gate/src/main.rs",
        "a developer tool outside the product; it runs git to read the changed lines",
    ),
    (
        "raul-gui/tests/first_run.rs",
        "silences the lint for the window test, which starts raul through ProcessExec and not itself",
    ),
];

/// The crate that may start processes.
const RUNNER: &str = "raul-runner";

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
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

/// The files under each crate of `crates` (its `src`, `tests` and `build.rs`) that start a process, as
/// `<crate>/<path>`, leaving out the runner and `allowed`. A crate with a `crates` folder of its own is a nested
/// workspace (the GUI's, ADR-0004); its members are scanned too, as `<crate>/crates/<member>/<path>`.
fn offenders(crates: &Path, allowed: &[(&str, &str)]) -> Vec<String> {
    let mut found = Vec::new();
    scan(crates, "", allowed, &mut found);
    // The developer tools (`tools/<name>`) are outside the product but are scanned the same way.
    let tools = crates.parent().map(|root| root.join("tools"));
    if let Some(tools) = tools.filter(|dir| dir.is_dir()) {
        scan(&tools, "tools/", allowed, &mut found);
    }
    found.sort();
    found
}

fn scan(crates: &Path, prefix: &str, allowed: &[(&str, &str)], found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(crates) else {
        return;
    };
    for krate in entries.flatten() {
        let name = krate.file_name().to_string_lossy().into_owned();
        // Only the Toolbox's own runner; a nested crate of the same name gets no pass.
        if (prefix.is_empty() && name == RUNNER) || !krate.path().is_dir() {
            continue;
        }
        let nested = krate.path().join("crates");
        if nested.is_dir() {
            scan(&nested, &format!("{prefix}{name}/crates/"), allowed, found);
        }
        let mut files = Vec::new();
        rust_files(&krate.path().join("src"), &mut files);
        rust_files(&krate.path().join("tests"), &mut files);
        let build = krate.path().join("build.rs");
        if build.is_file() {
            files.push(build);
        }
        for file in files {
            let relative = format!(
                "{prefix}{name}/{}",
                file.strip_prefix(krate.path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            );
            if allowed.iter().any(|(path, _)| *path == relative) {
                continue;
            }
            if starts_a_process(&std::fs::read_to_string(&file).unwrap()) {
                found.push(relative);
            }
        }
    }
}

fn toolbox_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn only_the_runner_crate_uses_process_command() {
    let found = offenders(&toolbox_crates(), ALLOWED);
    assert!(
        found.is_empty(),
        "these files use std::process::Command or silence its lint; go through raul_runner::Runner, or allow-list a test with a reason: {found:?}"
    );
}

#[test]
fn every_allowed_entry_still_exists_and_has_a_reason() {
    let crates = toolbox_crates();
    for (path, reason) in ALLOWED {
        assert!(
            match path.strip_prefix("tools/") {
                Some(tool) => crates.parent().unwrap().join("tools").join(tool).is_file(),
                None => crates.join(path).is_file(),
            },
            "{path} is gone; drop it from ALLOWED"
        );
        assert!(
            !reason.trim().is_empty() && !reason.contains('\n'),
            "{path} needs a one-line reason"
        );
        let file = match path.strip_prefix("tools/") {
            Some(tool) => crates.parent().unwrap().join("tools").join(tool),
            None => crates.join(path),
        };
        assert!(
            starts_a_process(&std::fs::read_to_string(&file).unwrap()),
            "{path} no longer starts a process; drop it from ALLOWED"
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
fn the_guard_covers_the_gui_workspace() {
    let gui = toolbox_crates().join("raul-gui");
    assert!(
        gui.join("crates").is_dir(),
        "the GUI workspace moved; point the guard at it"
    );
    let dir = tree(
        "guard-nested",
        &[
            ("raul-gui/src/main.rs", "use std::process::Command;"),
            (
                "raul-gui/crates/raul-ui-service/src/exec.rs",
                "#![allow(clippy::disallowed_methods)]",
            ),
            (
                "raul-gui/crates/raul-runner/src/lib.rs",
                "use std::process::Command;",
            ),
            (
                "raul-gui/crates/raul-ui-model/src/lib.rs",
                "pub struct Route;",
            ),
        ],
    );
    assert_eq!(
        offenders(dir.path(), &[]),
        [
            "raul-gui/crates/raul-runner/src/lib.rs",
            "raul-gui/crates/raul-ui-service/src/exec.rs",
            "raul-gui/src/main.rs",
        ]
    );
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
            ("raul-f/tests/run.rs", "use std::process::Command;"),
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
            "raul-f/tests/run.rs",
        ]
    );
}

#[test]
fn the_guard_leaves_the_runner_and_allowed_files_alone() {
    let dir = tree(
        "guard-allows",
        &[
            ("raul-runner/src/runner.rs", "use std::process::Command;"),
            ("raul-runner/tests/spawn.rs", "use std::process::Command;"),
            (
                "raul-cli/tests/cli.rs",
                "use std::process::Command; // starts the built binary",
            ),
            ("raul-cli/src/lib.rs", "use raul_runner::{Call, Runner};"),
            ("raul-x/src/listed.rs", "use std::process::Command;"),
        ],
    );
    assert_eq!(
        offenders(
            dir.path(),
            &[
                ("raul-x/src/listed.rs", "a reason"),
                ("raul-cli/tests/cli.rs", "a reason"),
            ]
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        offenders(dir.path(), &[]),
        ["raul-cli/tests/cli.rs", "raul-x/src/listed.rs"]
    );
}
