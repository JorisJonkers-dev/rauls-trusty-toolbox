// SPDX-License-Identifier: GPL-3.0-only
//! ADR-0005: `std::process::Command` is used in `nvmod-runner` and nowhere else in the shipped code. Clippy's
//! `disallowed-methods` (see `clippy.toml`) catches a new `Command::new`; this test also catches the other ways in
//! (`Command::from`, a renamed import, an `#[allow]` that silences the lint), and it runs under plain `cargo test`.

use std::path::{Path, PathBuf};

/// Source files outside the runner crate that may still start a process, with the reason.
const EXCEPTIONS: &[(&str, &str)] = &[
    (
        "nvmod-ui-service/src/exec.rs",
        "starts nvmod itself with a process group, streamed lines and a graceful stop (a streaming runner is future work)",
    ),
    (
        "nvmod-testkit/src/nvrs.rs",
        "test support that builds fixture archives with tar and chmod",
    ),
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn only_the_runner_crate_uses_process_command() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut offenders = Vec::new();
    for krate in std::fs::read_dir(crates).unwrap().flatten() {
        let name = krate.file_name().to_string_lossy().into_owned();
        if name == "raul-runner" {
            continue;
        }
        let mut files = Vec::new();
        // Only `src`: integration tests start the built binary, which is not a tool call.
        rust_files(&krate.path().join("src"), &mut files);
        for file in files {
            let relative = format!(
                "{name}/{}",
                file.strip_prefix(krate.path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            );
            if EXCEPTIONS.iter().any(|(path, _)| *path == relative) {
                continue;
            }
            let text = std::fs::read_to_string(&file).unwrap();
            if text.contains("process::Command")
                || text.contains("process::{") && text.contains("Command")
                || text.contains("disallowed_methods")
            {
                offenders.push(relative);
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these files use std::process::Command or silence its lint; go through nvmod_runner::Runner instead: {offenders:?}"
    );
}

#[test]
fn every_exception_still_exists() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for (path, _) in EXCEPTIONS {
        assert!(
            crates.join(path).is_file(),
            "{path} is gone; drop it from EXCEPTIONS"
        );
    }
}
