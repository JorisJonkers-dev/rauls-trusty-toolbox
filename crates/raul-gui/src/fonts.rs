// SPDX-License-Identifier: GPL-3.0-only
//! The bundled fonts (UI-RULES T1, L4): Inter for interface text, in a regular and a semibold face. Both are
//! compiled into `raul`, so every release carries them, and their licence is shown in the About dialog and listed in
//! `packaging/NOTICE-assets.toml`. egui's own monospace face stays for paths and versions.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// Inter, regular.
pub const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
/// Inter, semibold.
pub const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");
/// The SIL Open Font License 1.1 text that came with Inter.
pub const INTER_LICENSE: &str = include_str!("../assets/fonts/OFL.txt");

/// One bundled asset, as the About dialog lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notice {
    /// What it is.
    pub name: &'static str,
    /// Its SPDX licence id.
    pub license: &'static str,
    /// Where it comes from.
    pub source: &'static str,
    /// The licence text that ships with it.
    pub text: &'static str,
}

/// Every bundled asset of the GUI.
pub const NOTICES: &[Notice] = &[Notice {
    name: "Inter 4.1",
    license: "OFL-1.1",
    source: "https://github.com/rsms/inter",
    text: INTER_LICENSE,
}];

/// The font definitions: Inter first for proportional text with egui's faces as fallback, and a semibold family.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(INTER_REGULAR)),
    );
    fonts.font_data.insert(
        "inter-semibold".into(),
        Arc::new(FontData::from_static(INTER_SEMIBOLD)),
    );
    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".into());
    let mut semibold = vec!["inter-semibold".to_owned()];
    semibold.extend(fallback);
    fonts
        .families
        .insert(FontFamily::Name(crate::theme::SEMIBOLD.into()), semibold);
    fonts
}

/// Installs the fonts in `ctx`.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_testkit::{covers, ids};

    #[test]
    fn inter_is_bundled_with_its_licence() {
        covers(&[ids::gui::shell::FONTS_LICENSED]);
        // A TrueType file starts with version 1.0.
        assert_eq!(&INTER_REGULAR[..4], &[0, 1, 0, 0]);
        assert_eq!(&INTER_SEMIBOLD[..4], &[0, 1, 0, 0]);
        assert!(INTER_LICENSE.contains("SIL Open Font License, Version 1.1"));
        assert!(
            NOTICES
                .iter()
                .any(|n| n.license == "OFL-1.1" && n.text == INTER_LICENSE)
        );
    }

    #[test]
    fn inter_comes_first_and_the_semibold_family_exists() {
        let fonts = definitions();
        assert_eq!(
            fonts.families[&FontFamily::Proportional]
                .first()
                .map(String::as_str),
            Some("inter")
        );
        let semibold = &fonts.families[&FontFamily::Name(crate::theme::SEMIBOLD.into())];
        assert_eq!(semibold.first().map(String::as_str), Some("inter-semibold"));
        assert!(semibold.len() > 1, "a fallback for missing glyphs");
    }

    #[test]
    fn the_asset_notices_list_every_bundled_font() {
        let notices = include_str!("../../../packaging/NOTICE-assets.toml");
        for file in ["Inter-Regular.ttf", "Inter-SemiBold.ttf", "OFL.txt"] {
            assert!(
                notices.contains(&format!("crates/raul-gui/assets/fonts/{file}")),
                "{file} is not in packaging/NOTICE-assets.toml"
            );
        }
    }
}
