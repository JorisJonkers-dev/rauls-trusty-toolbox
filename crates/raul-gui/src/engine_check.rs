// SPDX-License-Identifier: GPL-3.0-only
//! The result of the engine's installation check, in the words a screen shows (UI-RULES W1, W3).
//!
//! The engine words its refusals for a command line: they name its program and flags and the folder it looked in.
//! None of that reaches a screen. The screen shows the catalogue's plain error; the engine's own text is kept only as
//! the detail behind "Copy details".

use raul_engine::EngineCheck;

use crate::catalogue::{self, ErrorText};

/// What a screen shows for one check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    /// The folder is accepted: a plain line, and the engine's report for the copyable details.
    Ready {
        /// The catalogue's line.
        message: &'static str,
        /// The engine's report lines; for the copyable details only.
        detail: Vec<String>,
    },
    /// The folder was not accepted, or the check did not answer.
    Problem {
        /// The catalogue's error: what went wrong and what to do.
        error: ErrorText,
        /// The engine's text, for the copyable details only; empty when the check did not answer.
        detail: String,
    },
}

/// Words one check result for the screen.
#[must_use]
pub fn outcome(check: EngineCheck) -> CheckOutcome {
    match check {
        EngineCheck::Accepted { lines } => CheckOutcome::Ready {
            message: catalogue::ENGINE_CHECK_ACCEPTED,
            detail: lines,
        },
        EngineCheck::NotProvidedOnThisSystem { detail } => CheckOutcome::Problem {
            error: catalogue::ENGINE_CHECK_NOT_PROVIDED,
            detail,
        },
        EngineCheck::Refused { detail } => CheckOutcome::Problem {
            error: catalogue::ENGINE_CHECK_REFUSED,
            detail,
        },
        EngineCheck::TimedOut => CheckOutcome::Problem {
            error: catalogue::ENGINE_CHECK_TIMED_OUT,
            detail: String::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the engine says on a refusal: its program, its flag, an environment variable and a user path.
    const RAW: &str =
        "nv-viewer --check: no FalloutNV.exe in /home/someone/Games/NV (set NV_CODE_PAGE)";

    fn shown(o: &CheckOutcome) -> String {
        match o {
            CheckOutcome::Ready { message, .. } => (*message).to_owned(),
            CheckOutcome::Problem { error, .. } => format!("{} {}", error.what, error.detail),
        }
    }

    #[test]
    fn each_kind_gets_its_own_plain_error_and_keeps_the_engines_text_as_detail() {
        let refused = outcome(EngineCheck::Refused {
            detail: RAW.to_owned(),
        });
        assert_eq!(
            refused,
            CheckOutcome::Problem {
                error: catalogue::ENGINE_CHECK_REFUSED,
                detail: RAW.to_owned()
            }
        );
        let not_provided = outcome(EngineCheck::NotProvidedOnThisSystem {
            detail: RAW.to_owned(),
        });
        assert_eq!(
            not_provided,
            CheckOutcome::Problem {
                error: catalogue::ENGINE_CHECK_NOT_PROVIDED,
                detail: RAW.to_owned()
            }
        );
        assert_ne!(
            catalogue::ENGINE_CHECK_REFUSED.id,
            catalogue::ENGINE_CHECK_NOT_PROVIDED.id
        );
        let timed_out = outcome(EngineCheck::TimedOut);
        assert_eq!(
            timed_out,
            CheckOutcome::Problem {
                error: catalogue::ENGINE_CHECK_TIMED_OUT,
                detail: String::new()
            }
        );
        let ready = outcome(EngineCheck::Accepted {
            lines: vec![RAW.to_owned()],
        });
        assert_eq!(
            ready,
            CheckOutcome::Ready {
                message: catalogue::ENGINE_CHECK_ACCEPTED,
                detail: vec![RAW.to_owned()]
            }
        );
    }

    #[test]
    fn what_a_screen_shows_has_no_command_tool_name_variable_or_path() {
        let all = [
            outcome(EngineCheck::Refused {
                detail: RAW.to_owned(),
            }),
            outcome(EngineCheck::NotProvidedOnThisSystem {
                detail: RAW.to_owned(),
            }),
            outcome(EngineCheck::TimedOut),
            outcome(EngineCheck::Accepted {
                lines: vec![RAW.to_owned()],
            }),
        ];
        for o in &all {
            let text = shown(o);
            for bad in [
                "nv-viewer",
                "--",
                "NV_",
                "FalloutNV.exe",
                "/home",
                "\\\\",
                ".exe",
            ] {
                assert!(!text.contains(bad), "{text:?} contains {bad:?}");
            }
            assert!(!text.contains(RAW), "{text:?} echoes the engine");
        }
    }
}
