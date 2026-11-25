// SPDX-License-Identifier: GPL-3.0-only
//! `raul sort ... --fetch-masterlist`: the masterlist is the Toolbox's to fetch (rauls-trusty-toolbox#449). The pinned
//! file is fetched, checked and stored by `raul_core::pinned`; nvmod then sorts with `--masterlist <that file>`, so it
//! needs no fetcher, store or `curl` of its own. Every other `raul sort` goes to nvmod as it is.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_core::pinned::{MasterlistPin, PinnedError, masterlist};

/// The options that only said where nvmod fetched the masterlist from; they are the pin's now.
const SOURCE_OPTIONS: [&str; 5] = ["--lock", "--store", "--gh", "--curl", "--retry-delay-ms"];

/// What `args` (the words after `sort`) ask for: the Instance and the source of the masterlist, and the words
/// left for nvmod. `None` when the command line does not ask for a fetch (or does not name an Instance, which nvmod
/// says in its own words).
fn split(args: &[String]) -> Option<(PathBuf, MasterlistPin, Vec<String>)> {
    if !args.iter().any(|a| a == "--fetch-masterlist") {
        return None;
    }
    let mut instance = None;
    let mut pin = MasterlistPin::default();
    let mut rest = Vec::new();
    let mut words = args.iter();
    while let Some(word) = words.next() {
        if word == "--fetch-masterlist" {
            continue;
        }
        if SOURCE_OPTIONS.contains(&word.as_str()) {
            let value = words.next()?.clone();
            match word.as_str() {
                "--lock" => pin.lock = Some(PathBuf::from(value)),
                "--store" => pin.store = Some(PathBuf::from(value)),
                "--curl" => pin.curl = Some(PathBuf::from(value)),
                "--retry-delay-ms" => {
                    pin.retry_delay = Some(Duration::from_millis(value.parse().ok()?));
                }
                // `--gh` checked an attestation the pin no longer has: accepted and ignored.
                _ => {}
            }
            continue;
        }
        rest.push(word.clone());
        if word == "--instance" {
            let value = words.next()?.clone();
            instance = Some(PathBuf::from(&value));
            rest.push(value);
        }
    }
    Some((instance?, pin, rest))
}

/// The refusal, verification and failure exit codes of the fetch, as nvmod's sort gave them.
fn exit_of(e: &PinnedError) -> u8 {
    match e {
        PinnedError::Refused(_) => nvmod_cli::REFUSED,
        PinnedError::Verification(_) => nvmod_cli::VERIFICATION,
        PinnedError::Failed(_) => nvmod_cli::ROLLED_BACK,
    }
}

/// Fetches the masterlist when `args` ask for it and returns the words nvmod sorts with, `Ok(None)` when they do not.
///
/// # Errors
/// The exit code, after the reason went to `err`.
pub(crate) fn prepare(args: &[String], err: &mut dyn Write) -> Result<Option<Vec<String>>, u8> {
    let Some((instance, pin, mut rest)) = split(args) else {
        return Ok(None);
    };
    match masterlist(Path::new(&instance), &pin) {
        Ok(file) => {
            rest.push("--masterlist".to_owned());
            rest.push(file.to_string_lossy().into_owned());
            Ok(Some(rest))
        }
        Err(e) => {
            let text = match &e {
                PinnedError::Refused(why) => format!("raul sort: refused: {why}"),
                PinnedError::Verification(why) => {
                    format!("raul sort: verification failed: {why}; nothing was changed")
                }
                PinnedError::Failed(why) => {
                    format!("raul sort: the masterlist could not be fetched: {why}")
                }
            };
            let _ = writeln!(err, "{text}");
            Err(exit_of(&e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_testkit::{covers, ids};

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn a_fetch_becomes_a_masterlist_file_and_the_source_options_are_taken_out() {
        covers(&[ids::mods::sort::FETCH_OPTION]);
        let (instance, pin, rest) = split(&words(
            "--instance /i --profile P --fetch-masterlist --apply --lock /l --store /s --curl /c --gh /g --retry-delay-ms 5 --json",
        ))
        .unwrap();
        assert_eq!(instance, PathBuf::from("/i"));
        assert_eq!(pin.lock, Some(PathBuf::from("/l")));
        assert_eq!(pin.store, Some(PathBuf::from("/s")));
        assert_eq!(pin.curl, Some(PathBuf::from("/c")));
        assert_eq!(pin.retry_delay, Some(Duration::from_millis(5)));
        assert_eq!(rest, words("--instance /i --profile P --apply --json"));
    }

    #[test]
    fn the_sort_mode_goes_to_nvmod_with_the_other_words() {
        let (_, _, rest) = split(&words(
            "--instance /i --mode full --fetch-masterlist --lock /l",
        ))
        .unwrap();
        assert_eq!(rest, words("--instance /i --mode full"));
    }

    #[test]
    fn other_sorts_and_unreadable_ones_go_to_nvmod_unchanged() {
        assert!(split(&words("--instance /i --masterlist /m")).is_none());
        assert!(split(&words("--fetch-masterlist --apply")).is_none());
        assert!(split(&words("--instance /i --fetch-masterlist --lock")).is_none());
        assert!(
            split(&words(
                "--instance /i --fetch-masterlist --retry-delay-ms x"
            ))
            .is_none()
        );
        assert_eq!(
            prepare(&words("--instance /i --keep-order"), &mut Vec::new()),
            Ok(None)
        );
    }

    #[test]
    fn a_failed_fetch_says_why_and_ends_with_the_code_nvmod_gave() {
        let dir = std::env::temp_dir().join(format!("raul-sort-fetch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let lock = dir.join("lock.toml");
        std::fs::write(&lock, "schema_version = 1\n").unwrap();
        let line = format!(
            "--instance {} --fetch-masterlist --lock {}",
            dir.display(),
            lock.display()
        );
        let mut err = Vec::new();
        assert_eq!(prepare(&words(&line), &mut err), Err(nvmod_cli::REFUSED));
        let text = String::from_utf8(err).unwrap();
        assert!(
            text.starts_with("raul sort: refused:") && text.contains("no masterlist source"),
            "{text}"
        );
        assert_eq!(
            exit_of(&PinnedError::Verification(String::new())),
            nvmod_cli::VERIFICATION
        );
        assert_eq!(
            exit_of(&PinnedError::Failed(String::new())),
            nvmod_cli::ROLLED_BACK
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_pinned_file_in_the_store_is_what_nvmod_sorts_with() {
        let dir = std::env::temp_dir().join(format!("raul-sort-store-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let body = b"abc";
        let hash = raul_runner::sha256_hex(body);
        let lock = dir.join("lock.toml");
        std::fs::write(
            &lock,
            format!(
                "schema_version = 1\n[[tool]]\nname = \"loot-masterlist-falloutnv\"\nversion = \"v1\"\nkind = \"third-party\"\n[tool.assets.any]\nurl = \"https://example.test/m.yaml\"\nsha256 = \"{hash}\"\n"
            ),
        )
        .unwrap();
        let store = dir.join("store");
        let stored = store.join(&hash);
        std::fs::create_dir_all(&stored).unwrap();
        let file = stored.join(format!(
            "loot-masterlist-falloutnv{}",
            std::env::consts::EXE_SUFFIX
        ));
        std::fs::write(&file, body).unwrap();
        let line = format!(
            "--instance {} --fetch-masterlist --apply --lock {} --store {}",
            dir.display(),
            lock.display(),
            store.display()
        );
        let rest = prepare(&words(&line), &mut Vec::new()).unwrap().unwrap();
        assert_eq!(rest[rest.len() - 2], "--masterlist");
        assert_eq!(rest[rest.len() - 1], file.to_string_lossy());
        assert!(
            !rest
                .iter()
                .any(|a| a == "--fetch-masterlist" || a == "--lock")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
