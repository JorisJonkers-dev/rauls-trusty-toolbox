// SPDX-License-Identifier: GPL-3.0-only
//! The GUI cannot do anything the command line cannot (ADR-0003). These tests fail when a screen
//! that changes something, or an action the GUI drives, has no equivalent on the real command line.

#![allow(clippy::assert_is_empty)]

use raul_testkit::{covers, ids};
use raul_ui_model::Screen;
use raul_ui_model::keymap::Keymap;
use raul_ui_model::palette::{self, Do};
use raul_ui_service::registry::{
    Action, COMMANDS, Coverage, action_of, command, coverage, help_block, help_lists,
    planned_screens,
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
    covers(&[ids::gui::pages::ACTIONS_ARE_COMMANDS]);
    let (code, nvmod, _) = raul(&["nvmod-help"]);
    assert_eq!(code, 0);
    let (_, own, _) = raul(&["help"]);
    let help = format!("{own}\n{nvmod}");
    for action in Action::ALL {
        assert!(
            help_lists(&help, action),
            "`{}` is not in `raul help` or `raul nvmod-help`",
            action.invocation()
        );
        let mut args = vec![action.command()];
        if action.takes_json() {
            args.push("--json");
        }
        let (_, _, err) = raul(&args);
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
    let help = "usage:\n  nvmod pe <file>\n  nvmod doctor --base <dir>\n  raul sort --plan\n  nvmod profile create --name <name> --instance <dir>\n";
    assert!(help_lists(help, Action::Doctor));
    assert!(help_lists(help, Action::Sort));
    assert!(help_lists(help, Action::Profile));
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
        // A page opens the screen of the earlier inventory it continues; the registry covers it through that.
        if let Do::Page(page) = e.action
            && let Some(screen) = page.screen()
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

#[test]
fn every_mvp_command_and_its_flags_are_in_the_help() {
    let (_, own, _) = raul(&["help"]);
    let (_, nvmod, _) = raul(&["nvmod-help"]);
    let help = format!("{own}\n{nvmod}");
    for command in COMMANDS {
        let line = command.line();
        if let Some(pr) = command.awaits {
            assert!(
                help_block(&own, &line).is_none(),
                "`{line}` is in `raul help` now: drop its mark of {pr}"
            );
            continue;
        }
        let block =
            help_block(&help, &line).unwrap_or_else(|| panic!("`{line}` is not in the help"));
        for flag in command.flags {
            assert!(
                block
                    .split(|c: char| !(c.is_alphanumeric() || c == '-'))
                    .any(|w| w == *flag),
                "`{line}` has no {flag} in its help:\n{block}"
            );
        }
    }
}

#[test]
fn a_help_block_is_its_usage_line_and_the_lines_that_continue_it() {
    let help = "usage:\n  raul config list|get <key>|set <key> <path>|unset <key> [--json]\n        more\n  raul lod plan [--world <name>]\n                 the plan\n  nvmod sort --instance <dir> [--apply]\n  nvmod plugins list --instance <dir>\n";
    for sub in ["list", "get", "set", "unset"] {
        let block = help_block(help, &format!("raul config {sub}")).unwrap();
        assert!(
            block.contains("[--json]") && block.contains("more"),
            "{block}"
        );
    }
    assert!(help_block(help, "raul config drop").is_none());
    let lod = help_block(help, "raul lod plan").unwrap();
    assert!(lod.contains("the plan") && !lod.contains("sort"));
    assert!(help_block(help, "raul lod generate").is_none());
    assert!(help_block(help, "raul sort").unwrap().contains("--apply"));
    let forms = "  raul --version | raul version [--json]\n  raul run <title|path|game|nv-rs> --instance <dir> [--guard <path>]\n";
    assert!(
        help_block(forms, "raul version")
            .unwrap()
            .contains("[--json]")
    );
    assert!(
        help_block(forms, "raul run nv-rs")
            .unwrap()
            .contains("--guard")
    );
    assert!(help_block(forms, "raul run steam").is_none());
    let two = "  nvmod doctor --base <dir>\n  nvmod doctor --instance <dir>\n";
    let both = help_block(two, "raul doctor").unwrap();
    assert!(
        both.contains("--base") && both.contains("--instance"),
        "{both}"
    );
    assert!(help_block(help, "raul plugins list").is_some());
    assert!(help_block(help, "raul plugins").is_some());
    assert!(
        help_block(help, "nvmod sort").is_none(),
        "a line is asked for as raul"
    );
}

#[test]
fn every_mvp_command_is_a_registered_action_with_its_words() {
    for command in COMMANDS {
        assert_eq!(action_of(&command.line()), Some(command.action));
        let args = command.args(["--x".to_owned()]);
        assert_eq!(args.len(), command.words.len() + 1);
        assert_eq!(args.last().map(String::as_str), Some("--x"));
    }
    assert!(
        !Action::Init.takes_json() && !Action::Profile.takes_json() && !Action::Target.takes_json()
    );
    assert!(Action::Config.takes_json() && Action::Lod.cancellable());
}

#[test]
fn ids_are_unique_and_json_follows_the_action() {
    let mut ids: Vec<&str> = COMMANDS.iter().map(|c| c.id).collect();
    ids.sort_unstable();
    let n = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), n, "two commands share an id");
    for c in COMMANDS {
        assert_eq!(command(c.id), Some(c));
        assert_eq!(
            c.flags.contains(&"--json"),
            c.action.takes_json(),
            "{}: --json in its flags and Action::takes_json disagree",
            c.id
        );
    }
    assert!(command("sort-apply").unwrap().writes);
    assert!(!command("sort-preview").unwrap().writes);
    assert!(command("nothing").is_none());
}

/// Every published `raul-ttw-result` fixture reads as a report; another contract or a newer version does not.
#[test]
fn every_published_ttw_result_reads() {
    use raul_ui_service::dto::TtwReport;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../contracts/fixtures/raul-ttw-result");
    let mut read = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let report = TtwReport::from_value(value.clone())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(
            report.error.is_none(),
            report.status == raul_ui_model::events::Status::Success,
            "{}",
            path.display()
        );
        let mut newer = value.clone();
        newer["schema_version"] = 2.into();
        assert!(
            TtwReport::from_value(newer)
                .unwrap_err()
                .contains("version 2")
        );
        let mut other = value;
        other["contract"] = "raul-lod-result".into();
        assert!(TtwReport::from_value(other).is_err());
        read += 1;
    }
    assert!(read >= 6, "{read}");
}
