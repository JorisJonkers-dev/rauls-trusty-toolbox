// SPDX-License-Identifier: GPL-3.0-only
//! Every page fixture fits its window (UI-RULES A6, ticket #177): at the smallest supported window size and at a
//! large one, no label, button or field reaches past the right edge of the window. Content that is taller than the
//! window scrolls inside its own area; the accessibility tree reports that content whole, so only the width is
//! checked here.
//!
//! A page that is known not to fit yet is listed in [`KNOWN`] with the ticket that fixes it; the list shrinks to
//! nothing as those tickets land, and a listed page that fits fails the test so it is taken off the list.

use egui_kittest::Harness;
use egui_kittest::kittest::{By, NodeT, Queryable};
use raul_gui::jobs::Jobs;
use raul_gui::pages;
use raul_gui::shell::App;
use raul_ui_service::{Exec, Output};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The smallest window the app is asked to work in (`MIN_WINDOW` in `raul_gui`).
const SMALLEST: [f32; 2] = [402.0, 480.0];
/// A large window.
const LARGE: [f32; 2] = [1920.0, 1080.0];

/// Views that do not fit yet, and the ticket that fixes each.
const KNOWN: &[(&str, &str)] = &[("setup", "#211")];

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

/// The views whose fixtures put something outside a window of `size`, with what and where.
fn overflowing(size: [f32; 2]) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for fixture in pages::fixtures() {
        let view = fixture.page.view().slug().to_owned();
        let name = format!("{view}-{}", fixture.name);
        let app = App::fixture(fixture, Jobs::new(Arc::new(Quiet)));
        let mut h = Harness::builder().with_size(size).build_eframe(|cc| {
            raul_gui::set_up(&cc.egui_ctx);
            app
        });
        h.run_steps(4);
        for node in h.root().query_all(By::new()) {
            // A text field scrolls its text inside itself; the tree reports that text run whole, not the field.
            let in_text_field = node.accesskit_node().parent().is_some_and(|p| {
                matches!(
                    p.role(),
                    eframe::egui::accesskit::Role::TextInput
                        | eframe::egui::accesskit::Role::MultilineTextInput
                )
            });
            if in_text_field {
                continue;
            }
            let Some(rect) = node.accesskit_node().bounding_box() else {
                continue;
            };
            if rect.x1 > f64::from(size[0]) + 1.0 {
                found.push((
                    view.clone(),
                    format!(
                        "{name}: {:?} {:?} reaches x {:.0} (y {:.0}) in a {:.0} by {:.0} window",
                        node.accesskit_node().role(),
                        node.accesskit_node()
                            .label()
                            .or(node.accesskit_node().value()),
                        rect.x1,
                        rect.y1,
                        size[0],
                        size[1]
                    ),
                ));
            }
        }
    }
    found
}

#[test]
fn no_page_fixture_is_wider_or_taller_than_its_window() {
    let mut failing: BTreeSet<String> = BTreeSet::new();
    let mut report = Vec::new();
    for size in [SMALLEST, LARGE] {
        for (view, what) in overflowing(size) {
            failing.insert(view);
            report.push(what);
        }
    }
    let known: BTreeSet<String> = KNOWN.iter().map(|(v, _)| (*v).to_owned()).collect();
    let new: Vec<&String> = failing.difference(&known).collect();
    assert!(
        new.is_empty(),
        "pages that do not fit their window: {new:?}\n{}",
        report.join("\n")
    );
    let fixed: Vec<&String> = known.difference(&failing).collect();
    assert!(
        fixed.is_empty(),
        "these pages fit now, take them off KNOWN: {fixed:?}"
    );
}
