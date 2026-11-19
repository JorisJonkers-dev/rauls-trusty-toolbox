// SPDX-License-Identifier: GPL-3.0-only
//! The page registry: every page and flow of the minimum viable Toolbox, and what the shell asks of each.
//!
//! The shell owns this file and draws everything around a page: the top bar, the sidebar with its chips, the
//! page header (title, one-line state, tools), the footer and the dialogs. A page draws only its content. Each page
//! has its own file that its owner fills: the [`Page`] implementation, its strings and errors (with
//! [`crate::texts!`] and [`crate::errors!`]) and its [`Fixture`]s. Nothing else needs an edit to change a page.

pub mod home;
pub mod load_order;
pub mod lod;
pub mod lod_settings;
pub mod mods;
pub mod settings;
pub mod setup;
pub mod ttw;
pub mod versions;

use crate::catalogue::ErrorText;
use crate::dialog::Dialogs;
use crate::jobs::Jobs;
use crate::theme::Tokens;
use eframe::egui;
use raul_ui_model::pages::{Attention, View};

/// What the shell shares with the page it draws.
#[derive(Debug, Default)]
pub struct ShellState {
    /// The New Vegas folder the top bar shows, `None` when it is not set (its home is Settings).
    pub new_vegas: Option<String>,
    /// The Instance folder, as `raul config get instance` derives it from the storage folder; `None` while it is
    /// not known or not set. The shell reads it once at start and again after each `raul config set` or `unset`.
    pub instance: Option<String>,
    /// The active engine build the top bar shows, `None` when there is none (its home is Versions).
    pub engine: Option<String>,
    /// The polite status line of the footer (UI-RULES G4).
    pub status: Option<String>,
    /// Set when the status line reports a failure: the footer then shows it as an error with a "Copy details"
    /// button. The page that reports a failure sets it; a status line is replaced by the next report.
    pub status_error: Option<ErrorText>,
    /// Why Play is blocked, shown under it and when it is pressed (UI-RULES G3); `None` when it is not.
    pub play_blocked: Option<ErrorText>,
    /// Set by the shell when Play was pressed and is not blocked; the page that starts the game clears it.
    pub play_requested: bool,
    /// A view a page asks the shell to open next.
    pub go_to: Option<View>,
}

/// What a page gets while it draws.
pub struct PageCx<'a> {
    /// The theme's colours.
    pub tokens: &'a Tokens,
    /// The state the shell shares.
    pub shell: &'a mut ShellState,
    /// The dialog layer.
    pub dialogs: &'a mut Dialogs,
    /// The jobs: `raul` commands running in the background.
    pub jobs: &'a mut Jobs,
    /// The window, so a page that waits for work of its own can ask for the frame that shows it.
    pub repaint: Option<egui::Context>,
}

/// A page or a flow.
pub trait Page {
    /// Which view it is.
    fn view(&self) -> View;

    /// Its title, from the catalogue.
    fn title(&self) -> &'static str;

    /// The one-line state of its header ("3 of 6 blocks out of date"; UI-RULES A1).
    fn state_line(&self) -> String;

    /// What needs attention, shown as a chip in the sidebar (UI-RULES G2).
    fn attention(&self) -> Option<Attention> {
        None
    }

    /// The page tools at the right of its header; at most one primary button (UI-RULES A1).
    fn tools(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>) {
        let _ = (ui, cx);
    }

    /// Draws the page's content.
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut PageCx<'_>);

    /// Its strings, for the catalogue checks.
    fn texts(&self) -> &'static [(&'static str, &'static str)] {
        &[]
    }

    /// Its errors, for the catalogue checks.
    fn errors(&self) -> &'static [ErrorText] {
        &[]
    }
}

/// A named state of a page that the snapshot tests render (UI-RULES S1): empty, loading, error, long text, large
/// list, done.
pub struct Fixture {
    /// The state's name; the snapshot is `<view>-<name>.png` in `tests/snapshots/<view>/`.
    pub name: &'static str,
    /// The page in that state.
    pub page: Box<dyn Page>,
    /// The shell state around it.
    pub shell: ShellState,
}

impl Fixture {
    /// A fixture of `page` with an empty shell state.
    pub fn new(name: &'static str, page: impl Page + 'static) -> Fixture {
        Fixture {
            name,
            page: Box::new(page),
            shell: ShellState::default(),
        }
    }
}

/// Every page and flow, each in its starting state: the sidebar pages in their order, then the flows.
pub fn registry() -> Vec<Box<dyn Page>> {
    vec![
        Box::new(home::Home::default()),
        Box::new(mods::Mods::default()),
        Box::new(load_order::LoadOrder::default()),
        Box::new(lod::Lod::default()),
        Box::new(versions::Versions::default()),
        Box::new(settings::Settings::default()),
        Box::new(setup::Setup::default()),
        Box::new(ttw::Ttw::default()),
        Box::new(lod_settings::LodSettings::default()),
    ]
}

/// Every page's fixtures.
pub fn fixtures() -> Vec<Fixture> {
    [
        home::fixtures(),
        mods::fixtures(),
        load_order::fixtures(),
        lod::fixtures(),
        versions::fixtures(),
        settings::fixtures(),
        setup::fixtures(),
        ttw::fixtures(),
        lod_settings::fixtures(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Draws the content of a page that is not built yet.
pub fn stub(ui: &mut egui::Ui) {
    ui.weak(crate::catalogue::STUB_BODY);
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_ui_model::pages::{Flow, PageId};

    #[test]
    fn the_registry_has_every_page_in_order_then_the_flows() {
        let views: Vec<View> = registry().iter().map(|p| p.view()).collect();
        let mut expected: Vec<View> = PageId::ALL.into_iter().map(View::Page).collect();
        expected.extend(Flow::ALL.into_iter().map(View::Flow));
        assert_eq!(views, expected);
    }

    #[test]
    fn every_view_has_a_title_and_a_fixture() {
        let fixtures = fixtures();
        for page in registry() {
            assert_ne!(page.title(), "");
            assert!(
                fixtures.iter().any(|f| f.page.view() == page.view()),
                "{:?} has no fixture",
                page.view()
            );
        }
    }

    #[test]
    fn fixture_names_are_unique_per_view() {
        let mut seen = std::collections::HashSet::new();
        for f in fixtures() {
            assert!(
                seen.insert((f.page.view(), f.name)),
                "{:?}/{} twice",
                f.page.view(),
                f.name
            );
        }
    }
}
