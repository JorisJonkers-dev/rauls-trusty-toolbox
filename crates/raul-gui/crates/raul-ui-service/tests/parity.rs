// SPDX-License-Identifier: GPL-3.0-only
//! The GUI cannot do anything the command line cannot (ADR-0003). These tests fail when a screen
//! that changes something, or an action the GUI drives, has no equivalent on the real command line.

#![allow(clippy::assert_is_empty)]

use raul_ui_model::Screen;
use raul_ui_model::keymap::Keymap;
use raul_ui_model::palette::{self, Do};
use raul_ui_service::registry::{
    Action, Coverage, action_of, coverage, help_lists, planned_screens,
};

/// Runs a command line through the same `raul_cli::run` the GUI build's `raul` runs.
fn raul(args: &[&str]) -> (u8, String, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = raul_cli::run(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn every_action_is_a_command_of_the_real_command_line() {
    let (code, help, _) = raul(&["nvmod-help"]);
    assert_eq!(code, 0);
    for action in Action::ALL {
        assert!(
            help_lists(&help, action),
            "`{}` is not in `raul nvmod-help`",
            action.invocation()
        );
        let (_, _, err) = raul(&[action.command(), "--json"]);
        assert!(
            !(err.starts_with("nvmod: unknown command")
                || err.starts_with("raul: unknown command")),
            "`{}` is not dispatched: {err}",
            action.command()
        );
    }
}

#[test]
fn a_command_that_the_help_does_not_list_is_not_covered() {
    let help = "usage:\n  nvmod pe <file>\n  nvmod doctor --base <dir>\n  raul sort --plan\n";
    assert!(help_lists(help, Action::Doctor));
    assert!(help_lists(help, Action::Sort));
    assert!(!help_lists(help, Action::Ttw));
    assert!(!help_lists("  cargo ttw\n", Action::Ttw));
    // A longer command that merely starts with the same letters is not the command.
    assert!(!help_lists("  nvmod setup-extra --x\n", Action::Setup));
}

/// Screens that change something and have no command line equivalent yet. The list may only shrink: a new
/// screen that writes without a command fails the test below, and a gap that gets closed must be removed here.
const KNOWN_WRITE_GAPS: [Screen; 0] = [];

#[test]
fn every_screen_that_changes_something_has_a_command_line_equivalent_or_is_a_known_gap() {
    let gaps: Vec<Screen> = Screen::ALL
        .into_iter()
        .filter(|s| s.mutates() && !matches!(coverage(*s), Coverage::Cli(_)))
        .collect();
    assert_eq!(
        gaps, KNOWN_WRITE_GAPS,
        "a screen can change things without a command line equivalent (or a gap was closed)"
    );
}

#[test]
fn a_planned_screen_without_a_write_gap_only_reads() {
    for screen in planned_screens() {
        assert!(
            !screen.mutates() || KNOWN_WRITE_GAPS.contains(&screen),
            "{screen:?} writes but is not covered"
        );
    }
}

#[test]
fn every_palette_entry_that_opens_a_changing_screen_is_covered() {
    let entries = palette::entries(&Keymap::default(), true);
    assert!(!entries.is_empty());
    for e in entries {
        if let Do::Go(screen) = e.action
            && screen.mutates()
        {
            assert!(
                matches!(coverage(screen), Coverage::Cli(_)) || KNOWN_WRITE_GAPS.contains(&screen),
                "palette entry {:?} opens {screen:?} without a command line equivalent",
                e.title
            );
        }
    }
}

#[test]
fn an_action_is_found_by_its_command_line() {
    assert_eq!(action_of("raul ttw --instance <dir>"), Some(Action::Ttw));
    assert_eq!(
        action_of("raul import-mo2 --mo2 x"),
        Some(Action::ImportMo2)
    );
    assert_eq!(action_of("raul install a.zip"), Some(Action::InstallMod));
    assert_eq!(action_of("raul nothing"), None);
    assert_eq!(action_of("ls"), None);
    for a in Action::ALL {
        assert_eq!(action_of(&a.invocation()), Some(a));
    }
}
