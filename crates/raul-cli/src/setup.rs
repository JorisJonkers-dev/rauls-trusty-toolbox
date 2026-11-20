// SPDX-License-Identifier: GPL-3.0-only
//! `raul setup`: a base copy to a modded, Doctor-clean Instance in one run, resumable. It is a thin wrapper over the
//! setup job of `raul-core` (`raul_core::setup::execute`), which runs in this process with each finished step
//! written to the Instance's journal; nothing is started as a command and nvmod's `setup` is not reached. With
//! `--json` the events are one JSON object per line on standard output (the stream the setup wizard reads);
//! without it they are lines of text, and the reason a run did not end well goes to standard error.

use std::io::Write;

use raul_core::instance_lock::InstanceGuard;
use raul_core::setup::{CancelToken, Line, Mode, execute, parse_words};

use crate::{REFUSED, USAGE};

const USAGE_LINE: &str = "usage: raul setup --base <dir> --instance <dir> --masterlist <file> [--import-mo2 <dir>] [--mod <archive>]... [--fallout3 <dir> [--package <file>]] [--profile <name>] [--answers <file>] [--catalog <file>] [--skip-nv-rs] [--sort] [--copy] [--keep-partial] [--dry-run] [--json]\n       raul setup --resume --instance <dir> [--json]\n       raul setup --restart --instance <dir> --yes [--json]";

/// What the words ask for, or the usage error.
fn mode_of(args: &[String]) -> Result<(Mode, bool), String> {
    let parsed = parse_words(args)?;
    if parsed.resume || parsed.restart {
        // They take the Instance and nothing else (a restart also its `--yes`).
        let misplaced = (parsed.resume && parsed.restart) || (parsed.yes && !parsed.restart);
        if parsed.has_run_options || misplaced || !parsed.has_instance {
            return Err("this form takes the Instance and nothing else".to_owned());
        }
        let instance = parsed.request.instance;
        return Ok((
            if parsed.restart {
                Mode::Restart {
                    instance,
                    confirmed: parsed.yes,
                }
            } else {
                Mode::Resume { instance }
            },
            parsed.json,
        ));
    }
    if parsed.yes || !(parsed.has_base && parsed.has_instance && parsed.has_masterlist) {
        return Err("a setup needs --base, --instance and --masterlist".to_owned());
    }
    Ok((
        Mode::Fresh {
            request: parsed.request,
            dry_run: parsed.dry_run,
            keep_partial: parsed.keep_partial,
        },
        parsed.json,
    ))
}

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (mode, json) = match mode_of(args) {
        Ok(m) => m,
        Err(why) => {
            let _ = writeln!(err, "raul setup: {why}\n{USAGE_LINE}");
            return USAGE;
        }
    };
    // A stop request (Ctrl+C, SIGTERM, the GUI's cancel) ends the run before its next step; with the journal it can
    // be resumed.
    let _ = ctrlc::set_handler(nvmod_core::cancel::request);
    // A resume writes to an Instance that exists: it takes the Instance lock. A first run has no Instance to lock yet,
    // and a restart removes the folder.
    let _guard = match &mode {
        Mode::Resume { instance } => match InstanceGuard::acquire(instance, "setup") {
            Ok(g) => Some(g),
            Err(e) => {
                let _ = writeln!(err, "raul setup: refused: {e}");
                if json {
                    let why = e.to_string();
                    let event = serde_json::json!({"seq": 1, "event": "finished", "status": "refused", "exit_code": REFUSED, "rolled_back": false, "reason": why});
                    let _ = writeln!(out, "{event}");
                }
                return REFUSED;
            }
        },
        _ => None,
    };
    let cancel = CancelToken::new();
    let done = std::sync::atomic::AtomicBool::new(false);
    let ended = std::thread::scope(|scope| {
        scope.spawn(|| {
            while !done.load(std::sync::atomic::Ordering::Relaxed) {
                if nvmod_core::cancel::requested() {
                    cancel.cancel();
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        let ended = execute(
            &mode,
            &mut |line: Line| {
                if json {
                    let _ = writeln!(out, "{}", line.json);
                    let _ = out.flush();
                } else if !line.text.is_empty() {
                    let _ = writeln!(out, "{}", line.text);
                }
            },
            &cancel,
        );
        done.store(true, std::sync::atomic::Ordering::Relaxed);
        ended
    });
    if let Some(note) = &ended.note {
        let _ = writeln!(err, "raul setup: {note}");
    }
    ended.code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn a_first_run_needs_the_base_the_instance_and_the_masterlist() {
        assert!(mode_of(&words("--instance /i --base /b")).is_err());
        let (mode, json) = mode_of(&words(
            "--instance /i --base /b --masterlist /m --json --dry-run",
        ))
        .unwrap();
        assert!(json && matches!(mode, Mode::Fresh { dry_run: true, .. }));
    }

    #[test]
    fn a_resume_and_a_restart_take_the_instance_and_nothing_else() {
        assert!(matches!(
            mode_of(&words("--resume --instance /i")).unwrap().0,
            Mode::Resume { .. }
        ));
        assert!(matches!(
            mode_of(&words("--restart --instance /i --yes")).unwrap().0,
            Mode::Restart {
                confirmed: true,
                ..
            }
        ));
        for line in [
            "--resume --restart --instance /i",
            "--resume --instance /i --yes",
            "--resume --instance /i --base /b",
            "--resume --instance /i --dry-run",
            "--resume",
        ] {
            assert!(mode_of(&words(line)).is_err(), "{line}");
        }
    }
}
