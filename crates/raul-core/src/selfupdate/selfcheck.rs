// SPDX-License-Identifier: GPL-3.0-only
//! The check a new version runs on itself after an update (ADR-0020), and the way the old version asks for it.
//!
//! [`run_in_process`] is what `raul update self-check` does: the version is the expected one, every file of the
//! unit matches `INSTALLED-FILES.sha256`, the packaged engine matches its list, and the configuration reads.
//! [`run_child`] starts the new program once with that command and reads its answer; a program that cannot start, runs
//! too long, exits with an error or answers anything but a pass counts as a failed check.

use std::path::Path;
use std::time::Duration;

use raul_runner::{Call, Runner};
use serde::{Deserialize, Serialize};

use super::extract::INVENTORY;
use super::verify::hash_file;

/// How long the new program has to answer.
pub const TIMEOUT: Duration = Duration::from_mins(1);

/// What a check found: `ok` and, when not, one plain line for each problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// The version the program says it is.
    pub version: String,
    /// Whether every check passed.
    pub ok: bool,
    /// What did not pass, in plain words.
    pub problems: Vec<String>,
}

/// Run the checks for the install in `root`, as the program that runs there (`running` is its version).
/// `expect` is the version the update was meant to bring, when the caller knows it. `require_list` is whether this
/// install has a list of its files at all (a folder install does; an `AppImage` is one file and has none).
#[must_use]
pub fn run_in_process(
    root: &Path,
    running: &str,
    expect: Option<&str>,
    require_list: bool,
) -> Report {
    let mut problems = Vec::new();
    if let Some(expected) = expect.filter(|e| *e != running) {
        problems.push(format!(
            "This is version {running}, not the version {expected} that was installed."
        ));
    }
    match std::fs::read_to_string(root.join(INVENTORY)) {
        Ok(list) => listed_files_match(root, &list, &mut problems),
        Err(_) if require_list => {
            problems.push("The list of installed files is missing.".to_owned());
        }
        Err(_) => {}
    }
    let engine = root.join("components").join("nv-rs");
    if engine
        .join(super::super::setup::packaged::MANIFEST)
        .is_file()
        && super::super::setup::packaged::verify(&engine).is_err()
    {
        problems.push("The game engine that came with this version is damaged.".to_owned());
    }
    if let Some(file) = crate::config::default_file().filter(|f| f.exists())
        && crate::config::Config::load(&file).is_err()
    {
        problems.push("The settings could not be read by this version.".to_owned());
    }
    Report {
        version: running.to_owned(),
        ok: problems.is_empty(),
        problems,
    }
}

/// Every file the list names must be a plain file with that checksum. Other files in the folder are not looked at: the
/// install folder also holds the update's own folder and whatever the person put there.
fn listed_files_match(root: &Path, list: &str, problems: &mut Vec<String>) {
    let mut bad = 0usize;
    for line in list.lines().filter(|l| !l.trim().is_empty()) {
        let Some((hash, rest)) = line.split_once(char::is_whitespace) else {
            problems.push("The list of installed files is damaged.".to_owned());
            return;
        };
        let name = rest.trim_start().trim_start_matches('*');
        let relative = Path::new(name);
        let inside = relative
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
        let path = root.join(relative);
        let plain = std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_file());
        if !inside || !plain || hash_file(&path).map_or(true, |h| !h.eq_ignore_ascii_case(hash)) {
            bad += 1;
        }
    }
    if bad > 0 {
        problems.push(format!(
            "{bad} installed file{} missing or changed.",
            if bad == 1 { " is" } else { "s are" }
        ));
    }
}

/// Start `program` with `update self-check --expect <version> --json` and read its answer.
///
/// # Errors
/// One plain line when the program cannot start, runs past [`TIMEOUT`], exits with an error, or answers that a check did
/// not pass; the first problem it names is included.
pub fn run_child(program: &Path, expect: &str) -> Result<(), String> {
    let args: Vec<String> = ["update", "self-check", "--expect", expect, "--json"]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let runner = Runner {
        timeout: TIMEOUT,
        max_output: 1024 * 1024,
        ..Runner::default()
    };
    let out = runner
        .run_any(&Call::new("raul", program, &args))
        .map_err(|_| "The new version did not start.".to_owned())?;
    if out.exit_code != 0 {
        return Err("The new version started but said it is not working.".to_owned());
    }
    let report: Report = serde_json::from_slice(&out.stdout)
        .map_err(|_| "The new version gave an answer that could not be read.".to_owned())?;
    if report.ok && report.version == expect {
        Ok(())
    } else {
        Err(report
            .problems
            .first()
            .cloned()
            .unwrap_or_else(|| "The new version did not pass its check.".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file, write_script};

    fn hash(b: &[u8]) -> String {
        raul_runner::sha256_hex(b)
    }

    /// An install of version `2.0.0`: `raul` and a notices file, with their list.
    fn install() -> TempDir {
        let t = TempDir::new("selfcheck");
        write_file(t.path(), "raul", b"binary");
        write_file(t.path(), "NOTICES", b"notices");
        let list = format!("{}  raul\n{}  NOTICES\n", hash(b"binary"), hash(b"notices"));
        write_file(t.path(), INVENTORY, list.as_bytes());
        t
    }

    #[test]
    fn a_sound_install_of_the_expected_version_passes() {
        let t = install();
        let r = run_in_process(t.path(), "2.0.0", Some("2.0.0"), true);
        assert!(r.ok, "{r:?}");
        assert_eq!(r.version, "2.0.0");
    }

    #[test]
    fn the_wrong_version_a_changed_file_a_missing_file_and_a_missing_list_each_fail() {
        let t = install();
        let r = run_in_process(t.path(), "1.0.0", Some("2.0.0"), true);
        assert!(
            !r.ok && r.problems[0].contains("not the version 2.0.0"),
            "{r:?}"
        );
        write_file(t.path(), "raul", b"tampered");
        let r = run_in_process(t.path(), "2.0.0", None, true);
        assert!(
            !r.ok && r.problems[0].contains("1 installed file is"),
            "{r:?}"
        );
        std::fs::remove_file(t.path().join("NOTICES")).unwrap();
        let r = run_in_process(t.path(), "2.0.0", None, true);
        assert!(r.problems[0].contains("2 installed files are"), "{r:?}");
        std::fs::remove_file(t.path().join(INVENTORY)).unwrap();
        let r = run_in_process(t.path(), "2.0.0", None, true);
        assert!(
            r.problems
                .iter()
                .any(|p| p.contains("list of installed files is missing"))
        );
    }

    #[test]
    fn an_install_without_a_list_passes_only_when_it_has_none_by_design() {
        let t = TempDir::new("selfcheck-nolist");
        assert!(run_in_process(t.path(), "2.0.0", None, false).ok);
        assert!(!run_in_process(t.path(), "2.0.0", None, true).ok);
    }

    #[test]
    fn files_the_list_does_not_name_are_left_alone() {
        let t = install();
        write_file(t.path(), "mine.txt", b"the person's own file");
        write_file(t.path(), ".raul-update/journal.json", b"{}");
        assert!(run_in_process(t.path(), "2.0.0", None, true).ok);
    }

    #[test]
    fn a_damaged_packaged_engine_fails() {
        let t = install();
        write_file(t.path(), "components/nv-rs/bin/nv-viewer", b"viewer");
        let line = format!("{}  bin/nv-viewer\n", hash(b"viewer"));
        write_file(t.path(), "components/nv-rs/FILES.sha256", line.as_bytes());
        assert!(run_in_process(t.path(), "2.0.0", None, true).ok);
        write_file(t.path(), "components/nv-rs/bin/nv-viewer", b"tampered");
        let r = run_in_process(t.path(), "2.0.0", None, true);
        assert!(
            !r.ok && r.problems.iter().any(|p| p.contains("engine")),
            "{r:?}"
        );
    }

    #[test]
    fn a_link_standing_for_a_listed_file_fails() {
        #[cfg(unix)]
        {
            let t = install();
            std::fs::remove_file(t.path().join("raul")).unwrap();
            write_file(t.path(), "elsewhere", b"binary");
            std::os::unix::fs::symlink(t.path().join("elsewhere"), t.path().join("raul")).unwrap();
            assert!(!run_in_process(t.path(), "2.0.0", None, true).ok);
        }
    }

    fn child(t: &TempDir, body: &str) -> std::path::PathBuf {
        write_script(t.path(), "new-raul", body)
    }

    #[cfg(unix)]
    #[test]
    fn a_child_that_passes_is_accepted_and_is_asked_in_the_agreed_words() {
        let t = TempDir::new("selfcheck-child");
        let log = t.path().join("args");
        let program = child(
            &t,
            &format!(
                "echo \"$@\" > '{}'\necho '{{\"version\":\"2.0.0\",\"ok\":true,\"problems\":[]}}'",
                log.display()
            ),
        );
        assert_eq!(run_child(&program, "2.0.0"), Ok(()));
        assert_eq!(
            std::fs::read_to_string(log).unwrap().trim(),
            "update self-check --expect 2.0.0 --json"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_child_that_cannot_start_fails_or_says_no_or_answers_badly_is_a_failed_check() {
        let t = TempDir::new("selfcheck-child-bad");
        assert!(
            run_child(&t.path().join("does-not-exist"), "2.0.0")
                .unwrap_err()
                .contains("did not start")
        );
        let not_executable = t.path().join("plain");
        std::fs::write(&not_executable, "x").unwrap();
        assert!(run_child(&not_executable, "2.0.0").is_err());
        let exits = child(&t, "exit 3");
        assert!(
            run_child(&exits, "2.0.0")
                .unwrap_err()
                .contains("not working")
        );
        let babble = write_script(t.path(), "babble", "echo hello");
        assert!(
            run_child(&babble, "2.0.0")
                .unwrap_err()
                .contains("could not be read")
        );
        let no = write_script(
            t.path(),
            "no",
            "echo '{\"version\":\"2.0.0\",\"ok\":false,\"problems\":[\"A file is changed.\"]}'",
        );
        assert_eq!(
            run_child(&no, "2.0.0"),
            Err("A file is changed.".to_owned())
        );
        let other = write_script(
            t.path(),
            "other",
            "echo '{\"version\":\"1.0.0\",\"ok\":true,\"problems\":[]}'",
        );
        assert!(
            run_child(&other, "2.0.0").is_err(),
            "a pass from the wrong version is not a pass"
        );
    }

    #[test]
    fn the_messages_hold_no_command_and_no_path() {
        let t = install();
        write_file(t.path(), "raul", b"tampered");
        let r = run_in_process(t.path(), "9.9.9", Some("2.0.0"), true);
        for p in &r.problems {
            for word in ["raul update", "self-check", "/", "sha256"] {
                assert!(!p.contains(word), "{word}: {p}");
            }
        }
    }
}
