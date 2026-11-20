// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox adapter of the engine's crates (ADR-0016, ADR-0017), linked in process.
//!
//! Everything here sits behind the cargo feature `engine`: without it the crate is empty and the engine's closure
//! (wgpu, winit and the engine crates) is not compiled. The typed jobs are added to this crate one at a time.

// The linked crates, so the Toolbox reaches them through this one adapter (and the build compiles and links them).
#[cfg(feature = "engine")]
pub use {fork_api, fork_play};

/// Whether this build links the engine's crates.
pub const ENGINE_LINKED: bool = cfg!(feature = "engine");

/// The reserved first argument that makes `raul` run the game instead of its own commands (ADR-0016): raul starts
/// itself again as `raul --play-mode <DATA> [arguments]` so the game is a process of its own and a crash of the
/// engine stays out of the window that started it.
pub const PLAY_MODE_ARG: &str = "--play-mode";

/// The exit code of a `raul` that was asked for the play mode but does not link the engine.
pub const EXIT_NO_ENGINE: u8 = 2;

/// Whether `args` (without the program name) ask for the play mode.
#[must_use]
pub fn is_play_mode(args: &[String]) -> bool {
    args.first().is_some_and(|a| a == PLAY_MODE_ARG)
}

/// What a `raul` without the engine says when it is asked for the play mode.
pub const NO_ENGINE_LINE: &str = "raul: this build does not include the game; use the full raul";

/// Runs the game as the play mode of this process and never returns when the engine is linked. Call it first in
/// `main`, only when [`is_play_mode`] says so; it reads the process's own arguments. Without the engine it returns
/// the exit code after a plain line on standard error.
#[must_use]
pub fn play_mode(err: &mut dyn std::io::Write) -> u8 {
    #[cfg(feature = "engine")]
    {
        let _ = err;
        fork_play::play_main()
    }
    #[cfg(not(feature = "engine"))]
    {
        let _ = writeln!(err, "{NO_ENGINE_LINE}");
        EXIT_NO_ENGINE
    }
}

/// The version of the engine facade this build was written against (`fork_api::API_VERSION`). The adapters check it
/// against the version they were written for.
#[cfg(feature = "engine")]
pub const FACADE_API_VERSION: u32 = fork_api::API_VERSION;

/// What the installation check found in a game folder. The engine's own words (`detail`) are kept for the copyable
/// error and the log; a screen shows plain words from the message catalogue instead (UI-RULES W1, W3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineCheck {
    /// The folder is accepted; the lines say what the run will find.
    Accepted {
        /// The engine's report, one line each.
        lines: Vec<String>,
    },
    /// Accepted, but this build does not run the game on this system yet. Not a fault of the folder.
    NotProvidedOnThisSystem {
        /// The engine's text.
        detail: String,
    },
    /// The folder is refused (no game executable, another version, an unsupported setting).
    Refused {
        /// The engine's text.
        detail: String,
    },
    /// The check did not answer within the time given, or an earlier check that did not answer is still running.
    TimedOut,
}

#[cfg(any(feature = "engine", test))]
/// How long [`engine_check`] waits for the check.
pub const CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

#[cfg(any(feature = "engine", test))]
/// What a check returns before it is typed: the report lines, or the refusal as (not provided here, text).
type RawCheck = Result<Vec<String>, (bool, String)>;

#[cfg(any(feature = "engine", test))]
/// Types a raw result. Kept apart from the engine call so both refusals can be pinned without a game folder.
fn typed(raw: RawCheck) -> EngineCheck {
    match raw {
        Ok(lines) => EngineCheck::Accepted { lines },
        Err((true, detail)) => EngineCheck::NotProvidedOnThisSystem { detail },
        Err((false, detail)) => EngineCheck::Refused { detail },
    }
}

#[cfg(any(feature = "engine", test))]
/// Whether a check started by [`run_bounded`] is still running (one that outlived its timeout is not killed: the
/// engine's sound probe on Linux can block and is remembered, so a second call would block the same way).
static IN_FLIGHT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(any(feature = "engine", test))]
/// Runs `check` on its own thread and waits at most `timeout` for it. Only one runs at a time; while one is still
/// running (it outlived an earlier timeout) the answer is [`EngineCheck::TimedOut`] without starting another.
fn run_bounded<F>(timeout: std::time::Duration, check: F) -> EngineCheck
where
    F: FnOnce() -> RawCheck + Send + 'static,
{
    use std::sync::atomic::Ordering;
    if IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return EngineCheck::TimedOut;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("engine-check".to_owned())
        .spawn(move || {
            let raw = check();
            IN_FLIGHT.store(false, Ordering::SeqCst);
            let _ = tx.send(raw);
        });
    if spawned.is_err() {
        IN_FLIGHT.store(false, Ordering::SeqCst);
        return EngineCheck::TimedOut;
    }
    rx.recv_timeout(timeout)
        .map_or(EngineCheck::TimedOut, typed)
}

/// The typed job `engine-check`: the installation check of the linked engine on the game folder `data`, in process
/// (what the engine's own check does, without starting a program). It reads files and asks the sound device what it
/// offers, which can block, so it runs on its own thread and gives up after [`CHECK_TIMEOUT`]; the call itself blocks
/// for at most that long, so a window calls it from its job runner and not from its UI thread.
#[cfg(feature = "engine")]
#[must_use]
pub fn engine_check(data: &std::path::Path) -> EngineCheck {
    let data = data.to_path_buf();
    run_bounded(CHECK_TIMEOUT, move || match fork_api::check(&data) {
        Ok(report) => Ok(report.lines),
        Err(e) => Err((
            e.kind == fork_api::CheckKind::NotProvidedOnThisSystem,
            e.message,
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_reserved_first_argument_selects_the_play_mode() {
        let a = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert!(is_play_mode(&a(&["--play-mode", "/data"])));
        assert!(!is_play_mode(&a(&["run", "--play-mode"])));
        assert!(!is_play_mode(&a(&[])));
    }

    #[cfg(not(feature = "engine"))]
    #[test]
    fn without_the_engine_the_play_mode_is_refused_in_one_plain_line() {
        let mut err = Vec::new();
        assert_eq!(play_mode(&mut err), EXIT_NO_ENGINE);
        assert_eq!(String::from_utf8(err).unwrap(), format!("{NO_ENGINE_LINE}\n"));
    }

    #[test]
    fn linked_flag_follows_the_feature() {
        assert_eq!(ENGINE_LINKED, cfg!(feature = "engine"));
    }

    #[cfg(feature = "engine")]
    #[test]
    fn facade_version_is_set() {
        assert!(std::hint::black_box(FACADE_API_VERSION) >= 1);
        assert!(fork_api::CAPABILITIES.trim_start().starts_with('{'));
    }

    #[cfg(feature = "engine")]
    /// A folder name under the system temp folder that is unique to this call and not created.
    fn temp_path(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        std::env::temp_dir().join(format!(
            "raul-engine-{name}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ))
    }

    #[test]
    fn both_refusals_are_typed_apart() {
        assert_eq!(
            typed(Err((true, "x".to_owned()))),
            EngineCheck::NotProvidedOnThisSystem {
                detail: "x".to_owned()
            }
        );
        assert_eq!(
            typed(Err((false, "y".to_owned()))),
            EngineCheck::Refused {
                detail: "y".to_owned()
            }
        );
        assert_eq!(
            typed(Ok(vec!["a".to_owned()])),
            EngineCheck::Accepted {
                lines: vec!["a".to_owned()]
            }
        );
    }

    // The bounded runner shares one in-flight flag, so its tests run one after the other.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn a_check_that_answers_is_typed() {
        let _g = SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let got = run_bounded(std::time::Duration::from_secs(5), || {
            Err((true, "later".to_owned()))
        });
        assert_eq!(
            got,
            EngineCheck::NotProvidedOnThisSystem {
                detail: "later".to_owned()
            }
        );
    }

    #[test]
    fn a_check_that_blocks_times_out_and_blocks_the_next_until_it_ends() {
        let _g = SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let got = run_bounded(std::time::Duration::from_millis(50), move || {
            let _ = wait.recv();
            Ok(Vec::new())
        });
        assert_eq!(got, EngineCheck::TimedOut);
        // The first is still running: a second is not started.
        let second = run_bounded(std::time::Duration::from_millis(50), || Ok(Vec::new()));
        assert_eq!(second, EngineCheck::TimedOut);
        release.send(()).unwrap();
        while IN_FLIGHT.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::yield_now();
        }
        let third = run_bounded(std::time::Duration::from_secs(5), || {
            Ok(vec!["ok".to_owned()])
        });
        assert_eq!(
            third,
            EngineCheck::Accepted {
                lines: vec!["ok".to_owned()]
            }
        );
    }

    #[cfg(feature = "engine")]
    #[test]
    fn an_empty_folder_is_refused_with_the_engines_text_kept() {
        let _g = SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let dir = temp_path("empty");
        std::fs::create_dir_all(&dir).unwrap();
        let got = engine_check(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        // Off Windows the engine refuses the folder as one without a game; on Windows it may find the same. Either
        // way it is a refusal, never an acceptance, and the text is kept for the detail.
        match got {
            EngineCheck::Refused { detail } | EngineCheck::NotProvidedOnThisSystem { detail } => {
                assert_ne!(detail, "");
            }
            other => panic!("an empty folder must not be accepted, got {other:?}"),
        }
    }

    #[cfg(feature = "engine")]
    #[test]
    fn a_missing_folder_is_not_accepted() {
        let _g = SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let missing = temp_path("missing").join("game");
        let got = engine_check(&missing);
        assert!(!matches!(got, EngineCheck::Accepted { .. }), "{got:?}");
    }
}
