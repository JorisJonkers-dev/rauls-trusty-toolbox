// SPDX-License-Identifier: GPL-3.0-only
//! The Load order page. A stub until its owner builds it; the shell already routes to it and draws its header.

use super::{Fixture, Page, PageCx};
use eframe::egui;
use raul_ui_model::pages::{PageId, View};

/// The Load order page.
#[derive(Debug, Default)]
pub struct LoadOrder {
    /// Its toolkit-free state.
    pub state: raul_ui_model::pages::load_order::State,
}

impl Page for LoadOrder {
    fn view(&self) -> View {
        View::Page(PageId::LoadOrder)
    }

    fn title(&self) -> &'static str {
        crate::catalogue::PAGE_LOAD_ORDER
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
    vec![Fixture::new("stub", LoadOrder::default())]
}
