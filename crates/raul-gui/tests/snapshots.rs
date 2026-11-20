// SPDX-License-Identifier: GPL-3.0-only
//! Every page fixture and the shell's own dialogs, rendered headless at 1280 by 800 and compared with the reviewed
//! snapshots under `tests/snapshots/<view>/` (UI-RULES S1). Rendering goes through wgpu; CI uses a software adapter,
//! so no GPU is needed.
//!
//! A changed picture fails the test and leaves `<name>.new.png` and `<name>.diff.png` next to the snapshot. After a
//! look at them, `UPDATE_SNAPSHOTS=1 cargo test --test snapshots` accepts the change, and the pull request shows the
//! new picture for review.
//!
//! The pictures are compared on Linux only: the software renderers of Windows and macOS rasterise glyph edges a few
//! pixels differently (1 to 15 pixels on most pages, a full text line on the Versions pages), so a picture made on
//! Linux cannot match there. Every other GUI test runs on all three systems (decision recorded in issue 434).
#![cfg(target_os = "linux")]

use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions, SnapshotResults};
use raul_gui::catalogue as msg;
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

fn harness(app: App) -> Harness<'static, App> {
    Harness::builder()
        .with_size([1280.0, 800.0])
        .build_eframe(|cc| {
            raul_gui::set_up(&cc.egui_ctx);
            app
        })
}

fn options(folder: &str) -> SnapshotOptions {
    SnapshotOptions::new().output_path(format!("tests/snapshots/{folder}"))
}

#[test]
fn every_page_fixture_matches_its_snapshot() {
    let mut results = SnapshotResults::new();
    for fixture in pages::fixtures() {
        let slug = fixture.page.view().slug();
        let name = format!("{slug}-{}", fixture.name);
        let mut h = harness(App::fixture(fixture, Jobs::new(Arc::new(Quiet))));
        h.run();
        h.snapshot_options(&name, &options(slug));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn the_shell_dialogs_match_their_snapshots() {
    let mut results = SnapshotResults::new();
    for (label, name) in [
        (msg::ABOUT, "shell-about"),
        (msg::PLAY, "shell-play-blocked"),
    ] {
        let mut h = harness(App::new(Jobs::new(Arc::new(Quiet))));
        h.run();
        // The Instance lookup at start ends before the picture, so the footer shows no running job.
        assert!(
            h.state_mut()
                .wait_for_jobs(std::time::Duration::from_secs(10))
        );
        h.run();
        h.get_by_label(label).click();
        h.run();
        h.snapshot_options(name, &options("shell"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
