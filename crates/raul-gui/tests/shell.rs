// SPDX-License-Identifier: GPL-3.0-only
//! The shell, driven headless with `egui_kittest` (no window, no GPU): routing, the sidebar, Play, the dialogs and
//! the narrow layout.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use raul_gui::catalogue as msg;
use raul_gui::jobs::Jobs;
use raul_gui::shell::App;
use raul_ui_model::pages::{PageId, View};
use raul_ui_service::{Exec, Output};
use std::sync::Arc;

/// Answers every command with success and an empty report.
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

fn app() -> App {
    App::new(Jobs::new(Arc::new(Quiet)))
}

fn harness(width: f32, app: App) -> Harness<'static, App> {
    Harness::builder()
        .with_size([width, 800.0])
        .build_eframe(|cc| {
            raul_gui::set_up(&cc.egui_ctx);
            app
        })
}

#[test]
fn the_shell_opens_on_home_with_the_five_pages_in_the_sidebar() {
    let h = harness(1280.0, app());
    assert_eq!(h.state().view(), View::Page(PageId::Home));
    for title in [
        msg::PAGE_HOME,
        msg::PAGE_LOAD_ORDER,
        msg::PAGE_LOD,
        msg::PAGE_VERSIONS,
        msg::PAGE_SETTINGS,
    ] {
        assert!(h.query_all_by_label(title).count() >= 1, "{title}");
    }
    assert!(h.query_by_label(msg::BRAND).is_some());
    assert!(h.query_by_label(msg::READY).is_some());
}

#[test]
fn number_keys_open_the_pages_in_order() {
    let mut h = harness(1280.0, app());
    for page in PageId::ALL.into_iter().rev() {
        let key = match page.key() {
            1 => egui::Key::Num1,
            2 => egui::Key::Num2,
            3 => egui::Key::Num3,
            4 => egui::Key::Num4,
            _ => egui::Key::Num5,
        };
        h.key_press(key);
        h.run();
        assert_eq!(h.state().view(), View::Page(page));
    }
}

#[test]
fn a_sidebar_entry_opens_its_page() {
    let mut h = harness(1280.0, app());
    h.get_all_by_label(msg::PAGE_LOD).next().unwrap().click();
    h.run();
    assert_eq!(h.state().view(), View::Page(PageId::Lod));
}

#[test]
fn the_setup_strip_opens_the_home_of_each_value() {
    let mut h = harness(1280.0, app());
    h.get_by_label_contains(msg::NEW_VEGAS).click();
    h.run();
    assert_eq!(h.state().view(), View::Page(PageId::Settings));
    h.get_by_label_contains(msg::ENGINE).click();
    h.run();
    assert_eq!(h.state().view(), View::Page(PageId::Versions));
}

#[test]
fn play_without_a_game_explains_why_and_offers_the_fix() {
    let mut h = harness(1280.0, app());
    h.get_by_label(msg::PLAY).click();
    h.run();
    let shown = h.state().dialogs().current().map(|d| d.title.clone());
    assert_eq!(shown.as_deref(), Some(msg::PLAY_BLOCKED_TITLE));
    assert_ne!(h.state().dialogs().current().unwrap().body[0], "");
    // The dialog's action opens Settings, where the folder is set.
    h.get_all_by_label(msg::PAGE_SETTINGS)
        .last()
        .unwrap()
        .click();
    h.run();
    assert!(!h.state().dialogs().is_open());
    assert_eq!(h.state().view(), View::Page(PageId::Settings));
}

#[test]
fn play_with_a_game_asks_home_to_start_it() {
    let mut app = app();
    app.shell_mut().new_vegas = Some("/games/New Vegas".into());
    app.open(View::Page(PageId::Versions));
    let mut h = harness(1280.0, app);
    h.get_by_label(msg::PLAY).click();
    h.run();
    assert!(!h.state().dialogs().is_open());
    assert_eq!(h.state().view(), View::Page(PageId::Home));
}

#[test]
fn about_lists_inter_and_esc_closes_it() {
    let mut h = harness(1280.0, app());
    h.get_by_label(msg::ABOUT).click();
    h.run();
    let body = h.state().dialogs().current().unwrap().body.join("\n");
    assert!(body.contains("Inter") && body.contains("OFL-1.1"), "{body}");
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(!h.state().dialogs().is_open());
}

#[test]
fn keys_do_nothing_while_a_dialog_is_open() {
    let mut h = harness(1280.0, app());
    h.get_by_label(msg::ABOUT).click();
    h.run();
    h.key_press(egui::Key::Num3);
    h.run();
    assert_eq!(h.state().view(), View::Page(PageId::Home));
}

#[test]
fn a_narrow_window_puts_the_pages_in_a_bar_without_group_labels() {
    let h = harness(600.0, app());
    for title in [msg::PAGE_HOME, msg::PAGE_SETTINGS] {
        assert!(h.query_all_by_label(title).count() >= 1, "{title}");
    }
    assert!(
        h.query_by_label(&msg::GROUP_SYSTEM.to_uppercase())
            .is_none()
    );
    let wide = harness(1280.0, app());
    assert!(
        wide.query_by_label(&msg::GROUP_SYSTEM.to_uppercase())
            .is_some()
    );
}

#[test]
fn a_page_can_ask_the_shell_to_open_another_view() {
    let mut app = app();
    app.shell_mut().go_to = Some(View::Page(PageId::LoadOrder));
    let mut h = harness(1280.0, app);
    h.run();
    assert_eq!(h.state().view(), View::Page(PageId::LoadOrder));
}
