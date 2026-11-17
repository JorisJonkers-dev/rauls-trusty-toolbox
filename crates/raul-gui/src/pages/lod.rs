// SPDX-License-Identifier: GPL-3.0-only
//! The LOD page. A stub until its owner builds it; the shell already routes to it and draws its header.

use super::{Fixture, Page, PageCx};
use eframe::egui;
use raul_ui_model::pages::{PageId, View};

/// The LOD page.
#[derive(Debug, Default)]
pub struct Lod {
    /// Its toolkit-free state.
    pub state: raul_ui_model::pages::lod::State,
}

impl Page for Lod {
    fn view(&self) -> View {
        View::Page(PageId::Lod)
    }

    fn title(&self) -> &'static str {
        crate::catalogue::PAGE_LOD
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
    vec![Fixture::new("stub", Lod::default())]
}
