// SPDX-License-Identifier: GPL-3.0-only
//! The setup wizard. A stub until its owner builds it; the shell already routes to it and draws its header.

use super::{Fixture, Page, PageCx};
use eframe::egui;
use raul_ui_model::pages::{Flow, View};

/// The setup wizard.
#[derive(Debug, Default)]
pub struct Setup {
    /// Its toolkit-free state.
    pub state: raul_ui_model::pages::setup::State,
}

impl Page for Setup {
    fn view(&self) -> View {
        View::Flow(Flow::Setup)
    }

    fn title(&self) -> &'static str {
        crate::catalogue::FLOW_SETUP
    }

    fn state_line(&self) -> String {
        crate::catalogue::STUB_STATE.to_owned()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _cx: &mut PageCx<'_>) {
        super::stub(ui);
    }
}

/// The states the snapshot tests render.
pub fn fixtures() -> Vec<Fixture> {
    vec![Fixture::new("stub", Setup::default())]
}
