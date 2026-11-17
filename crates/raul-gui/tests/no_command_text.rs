// SPDX-License-Identifier: GPL-3.0-only
//! No user-facing text holds a command line (UI-RULES G6, ticket #170): every page fixture is rendered and every
//! label and value on screen is checked.
//!
//! By design this does not look at what is not on screen: the copyable error details ("Copy details") and the
//! `Problem.command` fields keep the command that was run, because that is what a bug report needs. They only reach
//! the clipboard.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use raul_gui::jobs::Jobs;
use raul_gui::pages;
use raul_gui::shell::App;
use raul_ui_service::{Exec, Output};
use std::sync::Arc;

struct Quiet;

impl Exec for Quiet {
    fn run(&self, _args: &[String]) -> Result<Output, String> {
        Ok(Output {
            code: Some(0),
            stdout: "{}".into(),
            stderr: String::new(),
        })
    }
}

/// Whether `text` reads as a command line: it starts with the program name and a word, or holds a command flag.
fn is_command_line(text: &str) -> bool {
    let t = text.trim();
    let mut words = t.split_whitespace();
    let first = words.next().unwrap_or_default();
    let second = words.next().unwrap_or_default();
    let program = matches!(first, "raul" | "nvmod");
    (program
        && second.chars().next().is_some_and(char::is_alphabetic)
        && second.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        || t.split_whitespace()
            .any(|w| matches!(w, "--instance" | "--json" | "--dry-run" | "--profile"))
}

#[test]
fn no_page_fixture_shows_a_command_line() {
    let mut found = Vec::new();
    for fixture in pages::fixtures() {
        let name = format!("{}-{}", fixture.page.view().slug(), fixture.name);
        let app = App::fixture(fixture, Jobs::new(Arc::new(Quiet)));
        let mut h = Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                raul_gui::set_up(&cc.egui_ctx);
                app
            });
        h.run_steps(4);
        let root = h.root();
        for node in root.query_all(egui_kittest::kittest::By::new()) {
            let accesskit = node.accesskit_node();
            for text in [accesskit.label(), accesskit.value()].into_iter().flatten() {
                if is_command_line(&text) {
                    found.push(format!("{name}: {text}"));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "command lines on screen:\n{}",
        found.join("\n")
    );
}
