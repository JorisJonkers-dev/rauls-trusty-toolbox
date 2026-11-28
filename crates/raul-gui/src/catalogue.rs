// SPDX-License-Identifier: GPL-3.0-only
//! The message catalogue (UI-RULES W1 to W3, G5): every string the GUI shows.
//!
//! The shell's strings are here. A page keeps its own strings in its own file with the same two macros,
//! [`texts!`](crate::texts) and [`errors!`](crate::errors), and hands them to the catalogue through
//! [`crate::pages::Page::texts`] and [`crate::pages::Page::errors`]; [`all_texts`] and [`all_errors`] gather them, so
//! the checks below cover every page.
//!
//! Strings are stored as they are written in a sentence. Uppercase is a style the shell applies when it draws a
//! section label ([`crate::shell::section_label`]); no string is stored in capitals.

/// One error the GUI can show: what went wrong, and a plain detail line (UI-RULES W3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorText {
    /// A stable id, for tests and logs.
    pub id: &'static str,
    /// What went wrong, as a short line.
    pub what: &'static str,
    /// The plain detail: what was not changed, or what to do.
    pub detail: &'static str,
}

/// Declares strings: `texts!(LIST { NAME = "text"; ... })` makes a `pub const NAME: &str` for each and
/// `pub const LIST: &[(&str, &str)]` with their names and texts.
#[macro_export]
macro_rules! texts {
    ($list:ident { $($name:ident = $text:expr;)* }) => {
        $(
            #[allow(missing_docs)]
            pub const $name: &str = $text;
        )*
        /// Every string above, by name.
        pub const $list: &[(&str, &str)] = &[$((stringify!($name), $text)),*];
    };
}

/// Declares errors: `errors!(LIST { NAME = ("what", "detail"); ... })` makes a
/// `pub const NAME: ErrorText` for each and `pub const LIST: &[ErrorText]`.
#[macro_export]
macro_rules! errors {
    ($list:ident { $($name:ident = ($what:expr, $detail:expr);)* }) => {
        $(
            #[allow(missing_docs)]
            pub const $name: $crate::catalogue::ErrorText = $crate::catalogue::ErrorText {
                id: stringify!($name),
                what: $what,
                detail: $detail,
            };
        )*
        /// Every error above.
        pub const $list: &[$crate::catalogue::ErrorText] = &[$($name),*];
    };
}

texts!(SHELL {
    BRAND = "Raul's Toolbox";
    NEW_VEGAS = "New Vegas";
    NOT_SET = "Not set";
    ENGINE = "Engine";
    SEARCH = "Search";
    SEARCH_HINT = "Ctrl K";
    PLAY = "Play";
    GROUP_PLAY = "Play";
    GROUP_SYSTEM = "System";
    PAGE_HOME = "Home";
    PAGE_MODS = "Mods";
    PAGE_LOAD_ORDER = "Load order";
    PAGE_LOD = "LOD";
    PAGE_VERSIONS = "About and updates";
    PAGE_SETTINGS = "Settings";
    FLOW_SETUP = "Setup";
    FLOW_TTW = "Tale of Two Wastelands";
    STUB_STATE = "Not built yet";
    STUB_BODY = "This page is still being built.";
    READY = "Ready.";
    ABOUT = "About";
    ABOUT_TITLE = "About Raul's Toolbox";
    ABOUT_BODY = "One all-Rust installer, setup wizard and mod manager for Fallout: New Vegas. You need your own copy of the game; no game files are included.";
    ABOUT_VERSION = "Version";
    ABOUT_ASSETS = "Bundled assets";
    COMMAND = "Command";
    CLOSE = "Close";
    CANCEL = "Cancel";
    BACK = "Back";
    PLAY_BLOCKED_TITLE = "Play is not ready";
    RAUL = "Raul";
    RAUL_HOME = "Everything checks out. Go on, the Mojave is waiting.";
    RAUL_WIZARD = "Let's get you set up. It only takes a minute.";
    RAUL_ALL_CLEAR = "Nothing needs fixing.";
    JOB_RUNNING = "running";
    JOB_DONE = "finished";
    COPY_DETAILS = "Copy details";
    ENGINE_CHECK_ACCEPTED = "The game folder is ready for the engine";
});

errors!(SHELL_ERRORS {
    JOB_NOT_STARTED = (
        "The command could not be started",
        "Nothing changed. Check that raul is installed, then try again."
    );
    JOB_BUSY = (
        "Another change is still running",
        "Nothing changed. Wait for it to finish, or cancel it, then try again."
    );
    PLAY_NO_GAME = (
        "No New Vegas folder is set",
        "Set it in Settings, then press Play again."
    );
    JOB_FAILED = (
        "A change did not finish",
        "The lines below say what it reported."
    );
    STATUS_FAILED = (
        "The last change did not finish",
        "The lines below say what it reported."
    );
    ENGINE_CHECK_REFUSED = (
        "The engine cannot use this game folder",
        "Nothing changed. Check that it is the New Vegas folder, then check again. Copy the details to see why."
    );
    ENGINE_CHECK_NOT_PROVIDED = (
        "The engine does not run the game on this system yet",
        "Nothing changed. The folder itself is fine."
    );
    ENGINE_CHECK_TIMED_OUT = (
        "The check did not answer in time",
        "Nothing changed. Check again in a moment."
    );
});

/// Every string of the shell and of every page in the registry, as `(owner, name, text)`.
pub fn all_texts() -> Vec<(&'static str, &'static str, &'static str)> {
    let mut all: Vec<_> = SHELL.iter().map(|(n, t)| ("shell", *n, *t)).collect();
    for page in crate::pages::registry() {
        let owner = page.view().slug();
        all.extend(page.texts().iter().map(|(n, t)| (owner, *n, *t)));
    }
    all
}

/// Every error of the shell and of every page in the registry, as `(owner, error)`.
pub fn all_errors() -> Vec<(&'static str, ErrorText)> {
    let mut all: Vec<_> = SHELL_ERRORS.iter().map(|e| ("shell", *e)).collect();
    for page in crate::pages::registry() {
        let owner = page.view().slug();
        all.extend(page.errors().iter().map(|e| (owner, *e)));
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_testkit::{covers, ids};

    /// Whether `text` is stored in capitals: two or more letters and none of them lowercase.
    fn shouted(text: &str) -> bool {
        let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
        letters.len() > 1 && !letters.iter().any(|c| c.is_lowercase())
    }

    /// Words that are written in capitals in a sentence too.
    const ACRONYMS: &[&str] = &["LOD", "TTW", "OK", "ID"];

    #[test]
    fn every_error_has_a_plain_detail_line() {
        covers(&[ids::gui::shell::PLAIN_ERRORS]);
        for (owner, e) in all_errors() {
            assert!(!e.what.trim().is_empty(), "{owner}/{}: no message", e.id);
            assert!(
                !e.detail.trim().is_empty(),
                "{owner}/{}: no detail line",
                e.id
            );
            assert!(
                e.detail.ends_with('.'),
                "{owner}/{}: the detail is a sentence",
                e.id
            );
            assert!(
                !e.what.ends_with('.'),
                "{owner}/{}: the message is a short line, not a sentence",
                e.id
            );
        }
    }

    #[test]
    fn no_string_is_stored_in_capitals() {
        for (owner, name, text) in all_texts() {
            assert!(
                !shouted(text) || ACRONYMS.contains(&text),
                "{owner}/{name} is stored in capitals; uppercase is a style"
            );
        }
        for (owner, e) in all_errors() {
            assert!(
                !shouted(e.what) && !shouted(e.detail),
                "{owner}/{} is in capitals",
                e.id
            );
        }
    }

    #[test]
    fn names_are_unique_per_owner() {
        let mut seen = std::collections::HashSet::new();
        for (owner, name, _) in all_texts() {
            assert!(seen.insert((owner, name)), "{owner}/{name} twice");
        }
        let mut seen = std::collections::HashSet::new();
        for (owner, e) in all_errors() {
            assert!(seen.insert((owner, e.id)), "{owner}/{} twice", e.id);
        }
    }

    #[test]
    fn raul_speaks_one_short_line() {
        for line in [RAUL_HOME, RAUL_WIZARD, RAUL_ALL_CLEAR] {
            assert!(line.len() <= 80, "{line}");
            assert!(
                line.matches(". ").count() <= 1,
                "at most two short sentences: {line}"
            );
        }
    }

    #[test]
    fn no_screen_says_profile() {
        // A profile is an internal name: setup makes the default one, so no page shows it, asks for it or names it
        // in a message. The command lines a page shows may still carry it (`--profile Default`).
        for (owner, name, text) in all_texts() {
            assert!(
                !text.to_lowercase().contains("profile"),
                "{owner}/{name} says profile: {text}"
            );
        }
        for (owner, e) in all_errors() {
            for text in [e.what, e.detail] {
                assert!(
                    !text.to_lowercase().contains("profile"),
                    "{owner}/{} says profile: {text}",
                    e.id
                );
            }
        }
    }

    #[test]
    fn capitals_are_detected() {
        assert!(shouted("LOAD ORDER"));
        assert!(!shouted("Load order"));
        assert!(!shouted("x"));
        assert!(shouted("LOD"));
    }
}
