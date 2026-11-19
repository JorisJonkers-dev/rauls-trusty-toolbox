// SPDX-License-Identifier: GPL-3.0-only
//! ADR-0016: nothing the GUI does starts a Component as a separate program. The window's jobs call the command
//! line in process, so its sources must not name the process runner, and must not start a process themselves.
//! The game launcher is the one exception and does not live in this crate yet; when it does, it is listed here.

use std::fs;
use std::path::{Path, PathBuf};

fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

/// Words that start a process. Split in two halves so this file does not match itself.
fn forbidden() -> [String; 3] {
    [
        format!("{}Exec", "Process"),
        format!("{}::new", "Command"),
        format!("{}{}", "process::", "Command"),
    ]
}

#[test]
fn the_window_never_starts_a_process() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 5, "the sources were not found");
    for file in files {
        // The in-process exec has the runner's name inside its own.
        let text = fs::read_to_string(&file)
            .unwrap()
            .replace(&format!("In{}Exec", "Process"), "");
        for word in forbidden() {
            assert!(
                !text.contains(&word),
                "{} names `{word}`: a GUI action must call the work in process (ADR-0016)",
                file.display()
            );
        }
    }
}
