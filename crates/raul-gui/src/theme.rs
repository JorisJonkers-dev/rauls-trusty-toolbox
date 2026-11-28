// SPDX-License-Identifier: GPL-3.0-only
//! The themes: the colour tokens of `docs/redesign/SPEC.md` section 2 (default and high contrast) as egui styles
//! (ADR-0013).
//!
//! Every colour a page uses comes from [`Tokens`]; a page never writes a colour of its own. The contrast rule is
//! checked by a test over the table: every text colour is at least 4.5:1 on every surface, and control borders at
//! least 3:1.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle};

/// One colour as red, green and blue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// The colour for egui.
    pub const fn color(self) -> Color32 {
        Color32::from_rgb(self.0, self.1, self.2)
    }

    /// The relative luminance of WCAG 2.
    pub fn luminance(self) -> f64 {
        fn channel(c: u8) -> f64 {
            let c = f64::from(c) / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.0) + 0.7152 * channel(self.1) + 0.0722 * channel(self.2)
    }

    /// The WCAG 2 contrast ratio of this colour against `other`, from 1 to 21.
    pub fn contrast(self, other: Rgb) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        let (light, dark) = if a > b { (a, b) } else { (b, a) };
        (light + 0.05) / (dark + 0.05)
    }
}

/// The colour tokens of a theme (`docs/redesign/SPEC.md` section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tokens {
    /// Window and page background.
    pub bg: Rgb,
    /// Cards, top bar, banners, dialogs.
    pub raise: Rgb,
    /// Rows inside cards, chips, action bar, icon tiles, menus.
    pub raise2: Rgb,
    /// Selected or focused row tint, toast fill, open mod row.
    pub sel: Rgb,
    /// Hover on rows and buttons.
    pub hover: Rgb,
    /// Switch border, timeline, dashed empty box, menu separator.
    pub line: Rgb,
    /// Checkbox border, grip, switch off knob variants.
    pub ctl: Rgb,
    /// Main text.
    pub text: Rgb,
    /// Secondary text, ghost buttons, icons.
    pub muted: Rgb,
    /// Primary button, active icon, section bar, progress, selection ring.
    pub accent: Rgb,
    /// Text on accent.
    pub on_accent: Rgb,
    /// Brand mark and name only.
    pub brand: Rgb,
    /// Success.
    pub ok: Rgb,
    /// Warnings, untested-change dashed border.
    pub warn: Rgb,
    /// Errors, danger.
    pub err: Rgb,
    /// Information, update badge.
    pub info: Rgb,
    /// Input fields, switch track, log box.
    pub sunk: Rgb,
    /// Choice marker colour.
    pub choice: Rgb,
}

/// The dark theme.
pub const DARK: Tokens = Tokens {
    bg: Rgb(0x10, 0x12, 0x15),
    raise: Rgb(0x18, 0x1C, 0x22),
    raise2: Rgb(0x22, 0x28, 0x31),
    sel: Rgb(0x2A, 0x32, 0x40),
    hover: Rgb(0x29, 0x30, 0x3B),
    line: Rgb(0x2F, 0x35, 0x3F),
    ctl: Rgb(0x7A, 0x84, 0x8F),
    text: Rgb(0xE6, 0xE8, 0xEB),
    muted: Rgb(0xA3, 0xAC, 0xB7),
    accent: Rgb(0x3D, 0xD6, 0xC6),
    on_accent: Rgb(0x0B, 0x15, 0x13),
    brand: Rgb(0xF5, 0x9E, 0x3B),
    ok: Rgb(0x4C, 0xC3, 0x8A),
    warn: Rgb(0xFF, 0x9A, 0x4D),
    err: Rgb(0xFF, 0x7A, 0x7A),
    info: Rgb(0x6E, 0xA8, 0xFE),
    sunk: Rgb(0x0C, 0x0E, 0x10),
    choice: Rgb(0xB7, 0x9C, 0xFF),
};

/// The high contrast theme.
pub const HIGH_CONTRAST: Tokens = Tokens {
    bg: Rgb(0x00, 0x00, 0x00),
    raise: Rgb(0x00, 0x00, 0x00),
    raise2: Rgb(0x00, 0x00, 0x00),
    sel: Rgb(0x11, 0x11, 0x11),
    hover: Rgb(0x15, 0x15, 0x15),
    line: Rgb(0xB8, 0xB8, 0xB8),
    ctl: Rgb(0xFF, 0xFF, 0xFF),
    text: Rgb(0xFF, 0xFF, 0xFF),
    muted: Rgb(0xE0, 0xE0, 0xE0),
    accent: Rgb(0x5C, 0xF2, 0xE2),
    on_accent: Rgb(0x0B, 0x15, 0x13),
    brand: Rgb(0xF5, 0x9E, 0x3B),
    ok: Rgb(0x6F, 0xE8, 0xA6),
    warn: Rgb(0xFF, 0xB0, 0x66),
    err: Rgb(0xFF, 0x8A, 0x8A),
    info: Rgb(0x8D, 0xBB, 0xFF),
    sunk: Rgb(0x00, 0x00, 0x00),
    choice: Rgb(0xB7, 0x9C, 0xFF),
};

impl Tokens {
    /// The surfaces text is drawn on.
    pub fn surfaces(&self) -> [(&'static str, Rgb); 6] {
        [
            ("bg", self.bg),
            ("raise", self.raise),
            ("raise2", self.raise2),
            ("sel", self.sel),
            ("hover", self.hover),
            ("sunk", self.sunk),
        ]
    }

    /// The colours text is drawn in on those surfaces.
    pub fn text_colours(&self) -> [(&'static str, Rgb); 8] {
        [
            ("text", self.text),
            ("muted", self.muted),
            ("accent", self.accent),
            ("brand", self.brand),
            ("ok", self.ok),
            ("warn", self.warn),
            ("err", self.err),
            ("info", self.info),
        ]
    }
}

/// The base text size in points (UI-RULES T1: 14 px).
pub const BASE: f32 = 14.0;
/// The family of the semibold Inter face (see [`crate::fonts`]).
pub const SEMIBOLD: &str = "inter-semibold";

/// The text style of a page title (1.4 rem, semibold).
pub fn title_style() -> TextStyle {
    TextStyle::Name("title".into())
}

/// The text style of a section label (0.74 rem, semibold, shown in uppercase).
pub fn section_style() -> TextStyle {
    TextStyle::Name("section".into())
}

/// The text style of secondary text (0.9 rem).
pub fn secondary_style() -> TextStyle {
    TextStyle::Small
}

fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// Applies `tokens` and the type scale to `ctx`.
pub fn apply(ctx: &egui::Context, tokens: &Tokens) {
    ctx.set_visuals_of(egui::Theme::Dark, visuals(tokens));
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Body, FontId::proportional(BASE)),
            (TextStyle::Button, FontId::proportional(BASE)),
            (TextStyle::Small, FontId::proportional(BASE * 0.9)),
            (TextStyle::Monospace, FontId::monospace(BASE * 0.92)),
            (TextStyle::Heading, semibold(BASE * 1.12)),
            (title_style(), semibold(BASE * 1.4)),
            (section_style(), semibold(BASE * 0.74)),
        ]
        .into();
        let spacing = &mut style.spacing;
        spacing.item_spacing = egui::vec2(8.0, 8.0);
        spacing.button_padding = egui::vec2(12.0, 6.0);
        spacing.interact_size.y = 28.0;
        spacing.window_margin = Margin::same(20);
    });
}

/// The egui visuals of `tokens`.
pub fn visuals(t: &Tokens) -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    let radius = CornerRadius::same(6);
    v.override_text_color = Some(t.text.color());
    v.weak_text_color = Some(t.muted.color());
    v.hyperlink_color = t.accent.color();
    v.panel_fill = t.bg.color();
    v.window_fill = t.raise.color();
    v.window_stroke = Stroke::new(1.0, t.line.color());
    v.window_corner_radius = CornerRadius::same(12);
    v.extreme_bg_color = t.sunk.color();
    v.faint_bg_color = t.hover.color();
    v.code_bg_color = t.bg.color();
    v.warn_fg_color = t.warn.color();
    v.error_fg_color = t.err.color();
    v.selection.bg_fill = t.sel.color();
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.bg.color();
    w.noninteractive.weak_bg_fill = t.bg.color();
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.line.color());
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.text.color());
    w.noninteractive.corner_radius = radius;
    for (state, fill) in [
        (&mut w.inactive, t.bg),
        (&mut w.hovered, t.sel),
        (&mut w.active, t.sel),
        (&mut w.open, t.sel),
    ] {
        state.bg_fill = fill.color();
        state.weak_bg_fill = fill.color();
        state.bg_stroke = Stroke::new(1.0, t.ctl.color());
        state.fg_stroke = Stroke::new(1.0, t.text.color());
        state.corner_radius = radius;
        state.expansion = 0.0;
    }
    // The keyboard focus ring and text selection (UI-RULES X1): 2 px of the accent.
    v.selection.stroke = Stroke::new(2.0, t.accent.color());
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_testkit::{covers, ids};

    /// The themes the app ships.
    const THEMES: [(&str, Tokens); 2] = [("dark", DARK), ("high contrast", HIGH_CONTRAST)];

    #[test]
    fn every_text_colour_reads_on_every_surface() {
        covers(&[ids::gui::shell::THEME_CONTRAST]);
        for (theme, t) in THEMES {
            let mut low = Vec::new();
            for (surface, bg) in t.surfaces() {
                for (name, fg) in t.text_colours() {
                    let ratio = fg.contrast(bg);
                    if ratio < 4.5 {
                        low.push(format!("{name} on {surface}: {ratio:.2}"));
                    }
                }
            }
            assert!(low.is_empty(), "{theme}: below 4.5:1: {low:?}");
        }
    }

    #[test]
    fn text_on_the_accent_reads() {
        for (theme, t) in THEMES {
            let ratio = t.on_accent.contrast(t.accent);
            assert!(ratio >= 4.5, "{theme}: on_accent on accent: {ratio:.2}");
        }
    }

    #[test]
    fn control_borders_stand_out_from_every_surface_a_control_sits_on() {
        // Controls sit on the canvas, in dialogs, in a row, in a hovered row and on a switch track; a selected
        // row holds no control.
        for (theme, t) in THEMES {
            for (surface, bg) in [
                ("bg", t.bg),
                ("raise", t.raise),
                ("raise2", t.raise2),
                ("hover", t.hover),
                ("sunk", t.sunk),
            ] {
                let ratio = t.ctl.contrast(bg);
                assert!(ratio >= 3.0, "{theme}: ctl on {surface}: {ratio:.2}");
            }
        }
    }

    #[test]
    fn the_accent_is_never_the_brand_or_a_warning() {
        for (theme, t) in THEMES {
            assert_ne!(t.accent, t.brand, "{theme}: the accent is the brand");
            assert_ne!(t.accent, t.warn, "{theme}: the accent is a warning");
        }
    }

    #[test]
    fn contrast_matches_known_values() {
        let black = Rgb(0, 0, 0);
        let white = Rgb(255, 255, 255);
        assert!((white.contrast(black) - 21.0).abs() < 1e-9);
        assert!((black.contrast(white) - 21.0).abs() < 1e-9);
        assert!((white.contrast(white) - 1.0).abs() < 1e-9);
        // #767676 on white is the well-known 4.54:1.
        assert!((Rgb(0x76, 0x76, 0x76).contrast(white) - 4.54).abs() < 0.01);
    }

    #[test]
    fn high_contrast_follows_the_spec() {
        // `docs/redesign/SPEC.md` section 2, "High contrast" column.
        assert_eq!(HIGH_CONTRAST.bg, Rgb(0x00, 0x00, 0x00));
        assert_eq!(HIGH_CONTRAST.text, Rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(HIGH_CONTRAST.accent, Rgb(0x5C, 0xF2, 0xE2));
        assert_eq!(HIGH_CONTRAST.line, Rgb(0xB8, 0xB8, 0xB8));
    }

    #[test]
    fn the_visuals_use_the_tokens() {
        for (theme, t) in THEMES {
            let v = visuals(&t);
            assert_eq!(v.panel_fill, t.bg.color(), "{theme}: panel fill");
            assert_eq!(v.override_text_color, Some(t.text.color()), "{theme}: text colour");
            assert_eq!(v.widgets.inactive.bg_stroke.color, t.ctl.color(), "{theme}: control stroke");
            assert_eq!(v.hyperlink_color, t.accent.color(), "{theme}: hyperlink colour");
            assert_eq!(v.extreme_bg_color, t.sunk.color(), "{theme}: input fill");
        }
    }
}
