// SPDX-License-Identifier: GPL-3.0-only
//! The widget kit of `docs/redesign/SPEC.md` section 4: the redesign's cards, section headers, rows, buttons,
//! tags, badges and progress bars, in the colour tokens of section 2 and the sizes of [`sizes`].
//!
//! Every colour comes from the tokens of the current display ([`crate::theme::display`]); the one literal colour
//! value in the kit is a tag fill's alpha. The pieces follow the product rules the section 4 table rests on:
//! filled rounded blocks instead of lines (rule 2), blocks that grow to the page width (rule 5), a card's header
//! and rows sharing the card's inset (rule 6), and a whole row as one click target (rule 9). High contrast adds
//! the 1 px `line` border of section 2's last paragraph to every filled shape.

use crate::theme;
use eframe::egui::{self, Color32};

/// The sizes of `docs/redesign/SPEC.md` section 4 in egui points (the spec's CSS pixels at window scale 1), one
/// named constant per size the kit draws, so the spec changes in one place.
pub mod sizes {
    /// A row's minimum height (`.r`).
    pub const ROW_MIN_HEIGHT: f32 = 48.0;
    /// A row's corner radius.
    pub const ROW_RADIUS: u8 = 12;
    /// A row's horizontal padding.
    pub const ROW_PADDING_X: f32 = 14.0;
    /// A row's vertical padding.
    pub const ROW_PADDING_Y: f32 = 6.0;
    /// The gap between a row's title block and its trailing content.
    pub const ROW_GAP: f32 = 12.0;
    /// The gap between a page's sibling rows (`.rows`).
    pub const ROWS_GAP: f32 = 6.0;
    /// A row's sub line, the muted text under the title (`.r .sub`).
    pub const ROW_SUB_SIZE: f32 = 13.8;
    /// The gap between a row's title and its sub line.
    pub const ROW_SUB_GAP: f32 = 2.0;
    /// A card's corner radius (`.card`).
    pub const CARD_RADIUS: u8 = 16;
    /// The inset a card's header and its rows share from the card edge (product rule 6).
    pub const CARD_INSET: f32 = 12.0;
    /// A card's vertical padding.
    pub const CARD_PADDING_Y: f32 = 14.0;
    /// The gap between a card and the block below it (the card's margin-bottom).
    pub const CARD_BOTTOM_MARGIN: f32 = 12.0;
    /// A section header's minimum height (`.sech`).
    pub const SECTION_MIN_HEIGHT: f32 = 48.0;
    /// A section header's corner radius.
    pub const SECTION_RADIUS: u8 = 12;
    /// A section header's padding at the top, right and bottom.
    pub const SECTION_PADDING: f32 = 8.0;
    /// A section header's padding before the accent bar.
    pub const SECTION_PADDING_LEFT: f32 = 14.0;
    /// The accent bar's width before a section header's title.
    pub const SECTION_BAR_WIDTH: f32 = 6.0;
    /// The accent bar's height.
    pub const SECTION_BAR_HEIGHT: f32 = 16.0;
    /// The accent bar's corner radius.
    pub const SECTION_BAR_RADIUS: u8 = 3;
    /// The gap between the accent bar and a section header's title.
    pub const SECTION_BAR_GAP: f32 = 10.0;
    /// A section header's title size (uppercase semibold; egui has no letter spacing, so the tracking of
    /// section 3 is skipped).
    pub const SECTION_TITLE_SIZE: f32 = 13.1;
    /// A button's height (`.btn`).
    pub const BUTTON_HEIGHT: f32 = 40.0;
    /// A button's horizontal padding.
    pub const BUTTON_PADDING_X: f32 = 18.0;
    /// A small button's height (`.btn.sm`).
    pub const SMALL_BUTTON_HEIGHT: f32 = 32.0;
    /// A small button's horizontal padding.
    pub const SMALL_BUTTON_PADDING_X: f32 = 12.0;
    /// A small button's text size.
    pub const SMALL_BUTTON_TEXT: f32 = 14.0;
    /// An icon button's width and height (`.btn.icon`).
    pub const ICON_BUTTON: f32 = 36.0;
    /// The share of its colour a disabled control draws at.
    pub const DISABLED_ALPHA: f32 = 0.45;
    /// A tag's height (`.tag`).
    pub const TAG_HEIGHT: f32 = 26.0;
    /// A tag's corner radius: half its height, so a tag is a pill.
    pub const TAG_RADIUS: u8 = 13;
    /// A tag's horizontal padding.
    pub const TAG_PADDING_X: f32 = 11.0;
    /// A tag's text size.
    pub const TAG_TEXT_SIZE: f32 = 12.8;
    /// A tag fill's alpha: its tone colour at 14 % (section 2's "tag fills are the status colour at 14 % alpha").
    pub const TAG_FILL_ALPHA: u8 = 36;
    /// A badge's height (`.bdg`).
    pub const BADGE_HEIGHT: f32 = 22.0;
    /// A badge's minimum width.
    pub const BADGE_MIN_WIDTH: f32 = 22.0;
    /// A badge's corner radius: half its height, so a badge is a pill.
    pub const BADGE_RADIUS: u8 = 11;
    /// A badge's horizontal padding.
    pub const BADGE_PADDING_X: f32 = 6.0;
    /// A badge's text size.
    pub const BADGE_TEXT_SIZE: f32 = 12.0;
    /// A progress bar's height (`.pb`).
    pub const PROGRESS_HEIGHT: f32 = 8.0;
    /// A progress bar's corner radius.
    pub const PROGRESS_RADIUS: u8 = 4;
    /// The indeterminate bar's block width, as a share of the track (section 6's `slide`).
    pub const PROGRESS_SLIDE_WIDTH: f32 = 0.3;
    /// The stroke high contrast draws around a filled shape (section 2's 1 px `line` border).
    pub const HC_STROKE: f32 = 1.0;
}

/// The semibold Inter face at `size` (section 3's 600-to-750 weights).
fn semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(theme::SEMIBOLD.into()))
}

/// A whole-point size as the `i8` of an [`egui::Margin`] field; every size in [`sizes`] is whole points.
#[allow(clippy::cast_possible_truncation)]
fn pts(points: f32) -> i8 {
    points as i8
}

/// The 1 px `line` border high contrast adds to a filled shape (section 2's last paragraph, product rule 2).
fn hc_border(display: theme::Display, p: &egui::Painter, rect: egui::Rect, radius: f32) {
    if display.high_contrast {
        p.rect_stroke(
            rect,
            radius,
            egui::Stroke::new(sizes::HC_STROKE, display.tokens().line.color()),
            egui::StrokeKind::Inside,
        );
    }
}

/// A card: a filled rounded block that grows to the available width (product rule 5), whose header and rows
/// share a 12 px inset from its edge (product rule 6), with 12 px below it before the next block (`.card`).
///
/// High contrast adds the 1 px `line` border of section 2. The header and rows drawn by `add` line up because
/// they span the width the card's inset leaves them.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let display = theme::display(ui.ctx());
    let t = display.tokens();
    let mut frame = egui::Frame::new()
        .fill(t.raise.color())
        .corner_radius(sizes::CARD_RADIUS)
        .inner_margin(egui::Margin {
            left: pts(sizes::CARD_INSET),
            right: pts(sizes::CARD_INSET),
            top: pts(sizes::CARD_PADDING_Y),
            bottom: pts(sizes::CARD_PADDING_Y),
        })
        .outer_margin(egui::Margin {
            left: 0,
            right: 0,
            top: 0,
            bottom: pts(sizes::CARD_BOTTOM_MARGIN),
        });
    if display.high_contrast {
        frame = frame.stroke(egui::Stroke::new(sizes::HC_STROKE, t.line.color()));
    }
    frame.show(ui, |ui| {
        // Rule 5: the card takes the full available width, so no maximum width caps it.
        ui.set_width(ui.available_width());
        add(ui)
    })
}

/// A section header: the `raise2` bar that opens a section, with its accent bar and uppercase title (`.sech`),
/// clickable along its full width so the caller can fold the section on the response.
pub fn section_header(ui: &mut egui::Ui, title: &str) -> egui::Response {
    let display = theme::display(ui.ctx());
    let t = display.tokens();
    let galley = ui.painter().layout_no_wrap(
        title.to_uppercase(),
        semibold(sizes::SECTION_TITLE_SIZE),
        t.text.color(),
    );
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, sizes::SECTION_MIN_HEIGHT),
        egui::Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, false, title));
    let p = ui.painter();
    p.rect_filled(rect, sizes::SECTION_RADIUS, t.raise2.color());
    hc_border(display, p, rect, f32::from(sizes::SECTION_RADIUS));
    let bar = egui::Rect::from_min_size(
        egui::pos2(
            rect.left() + sizes::SECTION_PADDING_LEFT,
            rect.center().y - sizes::SECTION_BAR_HEIGHT / 2.0,
        ),
        egui::vec2(sizes::SECTION_BAR_WIDTH, sizes::SECTION_BAR_HEIGHT),
    );
    p.rect_filled(bar, sizes::SECTION_BAR_RADIUS, t.accent.color());
    p.galley(
        egui::pos2(
            bar.right() + sizes::SECTION_BAR_GAP,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        t.text.color(),
    );
    response
}

/// A row of a card: a title, an optional muted sub line under it, and trailing content at the right end, over
/// a `raise2` fill (`.r`).
///
/// The whole row is one click target (product rule 9): the returned response covers the row's full rect, so a
/// click anywhere folds the row, while a nested control in `trailing` keeps its own action.
pub struct Row<'a> {
    /// The title, in the base text size.
    pub title: &'a str,
    /// The sub line under the title, in muted 13.8 text.
    pub sub: Option<&'a str>,
    /// Whether the row is selected: filled with `sel` instead of `raise2`.
    pub selected: bool,
}

impl Row<'_> {
    /// Draws the row with `trailing` right-aligned inside it, laid out right to left, and returns the row's
    /// click response.
    pub fn show(self, ui: &mut egui::Ui, trailing: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        let display = theme::display(ui.ctx());
        let t = display.tokens();
        let base = theme::base(ui.ctx());
        let title = ui.painter().layout_no_wrap(
            self.title.to_owned(),
            egui::FontId::proportional(base),
            t.text.color(),
        );
        let sub = self.sub.map(|s| {
            ui.painter().layout_no_wrap(
                s.to_owned(),
                egui::FontId::proportional(sizes::ROW_SUB_SIZE),
                t.muted.color(),
            )
        });
        let title_h = title.size().y;
        let block_h = title_h + sub.as_ref().map_or(0.0, |g| sizes::ROW_SUB_GAP + g.size().y);
        let height = (block_h + 2.0 * sizes::ROW_PADDING_Y).max(sizes::ROW_MIN_HEIGHT);
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Button, true, self.selected, self.title)
        });
        let fill = if self.selected {
            t.sel.color()
        } else if response.hovered() {
            t.hover.color()
        } else {
            t.raise2.color()
        };
        let p = ui.painter();
        p.rect_filled(rect, sizes::ROW_RADIUS, fill);
        hc_border(display, p, rect, f32::from(sizes::ROW_RADIUS));
        let y = rect.center().y - block_h / 2.0;
        p.galley(
            egui::pos2(rect.left() + sizes::ROW_PADDING_X, y),
            title,
            t.text.color(),
        );
        if let Some(sub) = sub {
            p.galley(
                egui::pos2(
                    rect.left() + sizes::ROW_PADDING_X,
                    y + title_h + sizes::ROW_SUB_GAP,
                ),
                sub,
                t.muted.color(),
            );
        }
        // The trailing content runs in a child ui over the row's right end, laid out right to left.
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.left() + sizes::ROW_PADDING_X, rect.top()),
            egui::pos2(rect.right() - sizes::ROW_PADDING_X, rect.bottom()),
        );
        let mut trailing_ui = ui.child_ui(inner, egui::Layout::right_to_left(egui::Align::Center));
        trailing(&mut trailing_ui);
        response
    }
}

/// The kinds of [`button`] and [`small_button`]: the fills and text colours of section 2.
pub enum ButtonKind {
    /// The accent fill with `on_accent` text: a page's primary action.
    Primary,
    /// A quiet button: a `raise2` fill, `text` text, and the `hover` fill when hovered.
    Ghost,
    /// The error fill with `on_accent` text: a destructive action.
    Danger,
}

/// A button (`.btn`): 40 high, 18 px horizontal padding, a pill of the kind's fill with the semibold label
/// centred in it.
pub fn button(ui: &mut egui::Ui, kind: ButtonKind, label: &str) -> egui::Response {
    text_button(
        ui,
        kind,
        label,
        sizes::BUTTON_HEIGHT,
        sizes::BUTTON_PADDING_X,
        theme::base(ui.ctx()),
    )
}

/// A small button (`.btn.sm`): 32 high, 12 px padding, 14 px text.
pub fn small_button(ui: &mut egui::Ui, kind: ButtonKind, label: &str) -> egui::Response {
    text_button(
        ui,
        kind,
        label,
        sizes::SMALL_BUTTON_HEIGHT,
        sizes::SMALL_BUTTON_PADDING_X,
        sizes::SMALL_BUTTON_TEXT,
    )
}

/// The pill behind [`button`] and [`small_button`]: the label centred in a fully rounded rect whose radius is
/// half the height. A disabled ui draws the button at 45 % of its colours.
fn text_button(
    ui: &mut egui::Ui,
    kind: ButtonKind,
    label: &str,
    height: f32,
    padding_x: f32,
    text_size: f32,
) -> egui::Response {
    let display = theme::display(ui.ctx());
    let t = display.tokens();
    let enabled = ui.is_enabled();
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), semibold(text_size), t.text.color());
    let size = egui::vec2(galley.size().x + 2.0 * padding_x, height);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, enabled, false, label));
    let (fill, text) = match kind {
        ButtonKind::Primary => (t.accent.color(), t.on_accent.color()),
        ButtonKind::Ghost => (
            if response.hovered() {
                t.hover.color()
            } else {
                t.raise2.color()
            },
            t.text.color(),
        ),
        ButtonKind::Danger => (t.err.color(), t.on_accent.color()),
    };
    let dim = |c| if enabled { c } else { c.gamma_multiply(sizes::DISABLED_ALPHA) };
    let radius = height / 2.0;
    ui.painter().rect_filled(rect, radius, dim(fill));
    hc_border(display, ui.painter(), rect, radius);
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        dim(text),
    );
    response
}

/// An icon button (`.btn.icon`): a round 36x36 `raise2` disc with the `icon` glyph centred in `muted`.
pub fn icon_button(ui: &mut egui::Ui, icon: &str) -> egui::Response {
    let display = theme::display(ui.ctx());
    let t = display.tokens();
    let enabled = ui.is_enabled();
    let galley = ui.painter().layout_no_wrap(
        icon.to_owned(),
        egui::FontId::proportional(theme::base(ui.ctx())),
        t.muted.color(),
    );
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(sizes::ICON_BUTTON, sizes::ICON_BUTTON),
        egui::Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, enabled, false, icon));
    let dim = |c| if enabled { c } else { c.gamma_multiply(sizes::DISABLED_ALPHA) };
    ui.painter()
        .rect_filled(rect, sizes::ICON_BUTTON / 2.0, dim(t.raise2.color()));
    hc_border(display, ui.painter(), rect, sizes::ICON_BUTTON / 2.0);
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        dim(t.muted.color()),
    );
    response
}

/// The tone a status piece draws in: one of section 2's status colours, or none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Success.
    Ok,
    /// A warning.
    Warn,
    /// An error.
    Err,
    /// Information, and update badges.
    Info,
    /// The accent.
    Accent,
    /// No colour of its own: `muted` text, and the plain `raise2` fill on a tag.
    Neutral,
}

/// The token a tone paints in: `Neutral` has no colour of its own and reads as `muted`.
fn tone_colour(t: theme::Tokens, tone: Tone) -> theme::Rgb {
    match tone {
        Tone::Ok => t.ok,
        Tone::Warn => t.warn,
        Tone::Err => t.err,
        Tone::Info => t.info,
        Tone::Accent => t.accent,
        Tone::Neutral => t.muted,
    }
}

/// A status tag (`.tag`): a 26 px pill of the tone colour at 14 % alpha with the tone-coloured semibold text
/// centred in it; `Neutral` keeps the plain `raise2` fill with `muted` text.
pub fn tag(ui: &mut egui::Ui, tone: Tone, text: &str) -> egui::Response {
    let display = theme::display(ui.ctx());
    let t = display.tokens();
    let colour = tone_colour(t, tone);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        semibold(sizes::TAG_TEXT_SIZE),
        colour.color(),
    );
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(galley.size().x + 2.0 * sizes::TAG_PADDING_X, sizes::TAG_HEIGHT),
        egui::Sense::hover(),
    );
    // The kit's one literal colour value (section 2: a tag fill is its status colour at 14 % alpha).
    let fill = if tone == Tone::Neutral {
        t.raise2.color()
    } else {
        Color32::from_rgba_unmultiplied(colour.0, colour.1, colour.2, sizes::TAG_FILL_ALPHA)
    };
    ui.painter().rect_filled(rect, sizes::TAG_RADIUS, fill);
    hc_border(display, ui.painter(), rect, f32::from(sizes::TAG_RADIUS));
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        colour.color(),
    );
    response
}

/// A count badge (`.bdg`): a 22 px pill, at least 22 wide, of the tone colour with the count in `bg` text
/// centred in it.
pub fn badge(ui: &mut egui::Ui, tone: Tone, count: usize) -> egui::Response {
    let t = theme::display(ui.ctx()).tokens();
    let colour = tone_colour(t, tone);
    let galley = ui.painter().layout_no_wrap(
        count.to_string(),
        semibold(sizes::BADGE_TEXT_SIZE),
        t.bg.color(),
    );
    let width = (galley.size().x + 2.0 * sizes::BADGE_PADDING_X).max(sizes::BADGE_MIN_WIDTH);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, sizes::BADGE_HEIGHT), egui::Sense::hover());
    ui.painter().rect_filled(rect, sizes::BADGE_RADIUS, colour.color());
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        t.bg.color(),
    );
    response
}

/// A progress bar (`.pb`): a full-width 8 px track of `sunk` with an `accent` fill rounded to the track.
///
/// `Some(fraction)` fills `fraction` of the track, clamped to 0..1. `None` is indeterminate: a 30 % block
/// travels across the track once per slide period (section 6's `slide`), and asks for the repaints that move
/// it; with reduced motion the block sits still, starting at its parking spot of the width.
pub fn progress(ui: &mut egui::Ui, fraction: Option<f32>) -> egui::Response {
    let display = theme::display(ui.ctx());
    let t = display.tokens();
    let motion = display.motion();
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, sizes::PROGRESS_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, sizes::PROGRESS_RADIUS, t.sunk.color());
    let (left, filled) = match fraction {
        Some(fraction) => (rect.left(), rect.width() * fraction.clamp(0.0, 1.0)),
        None => {
            let block = rect.width() * sizes::PROGRESS_SLIDE_WIDTH;
            let left = match motion.indeterminate_park {
                // Reduced motion: the block sits still, starting at its parking spot of the width.
                Some(park) => rect.left() + rect.width() * park,
                None => {
                    // One pass of the slide per period; the repaints keep the block moving.
                    let time = ui.input(|i| i.time);
                    ui.ctx().request_repaint();
                    #[allow(clippy::cast_precision_loss)]
                    let phase = (time / f64::from(motion.slide_period)).fract() as f32;
                    rect.left() + phase * (rect.width() - block).max(0.0)
                }
            };
            (left, block)
        }
    };
    ui.painter().rect_filled(
        egui::Rect::from_min_size(
            egui::pos2(left, rect.top()),
            egui::vec2(filled, rect.height()),
        ),
        sizes::PROGRESS_RADIUS,
        t.accent.color(),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    /// Reads back what the drawing closure recorded on its last frame.
    fn drawn<T: Copy>(seen: &Arc<Mutex<Option<T>>>) -> T {
        seen.lock().unwrap().expect("the widget was drawn")
    }

    #[test]
    fn a_row_is_at_least_48_high_and_spans_the_available_width() {
        let seen = Arc::new(Mutex::new(None::<(egui::Rect, f32)>));
        let mut h = {
            let seen = seen.clone();
            Harness::builder()
                .with_size([600.0, 400.0])
                .with_context_setup(crate::set_up)
                .build_ui(move |ui| {
                    let width = ui.available_width();
                    let row = Row {
                        title: "Scan the mods",
                        sub: Some("12 mods changed"),
                        selected: false,
                    }
                    .show(ui, |_| ());
                    *seen.lock().unwrap() = Some((row.rect, width));
                })
        };
        h.run();
        let (rect, width) = drawn(&seen);
        assert!(
            rect.height() >= sizes::ROW_MIN_HEIGHT,
            "the row is only {:?} high",
            rect.height()
        );
        assert!(
            (rect.width() - width).abs() < 0.5,
            "the row spans {rect:?}, the available width is {width}"
        );
    }

    #[test]
    fn buttons_small_buttons_and_icon_buttons_measure_to_spec() {
        let seen = Arc::new(Mutex::new(None::<(egui::Rect, egui::Rect, egui::Rect)>));
        let mut h = {
            let seen = seen.clone();
            Harness::builder()
                .with_size([600.0, 400.0])
                .with_context_setup(crate::set_up)
                .build_ui(move |ui| {
                    let big = button(ui, ButtonKind::Primary, "Update");
                    let small = small_button(ui, ButtonKind::Ghost, "Check again");
                    let icon = icon_button(ui, "✓");
                    *seen.lock().unwrap() = Some((big.rect, small.rect, icon.rect));
                })
        };
        h.run();
        let (big, small, icon) = drawn(&seen);
        assert!(
            (big.height() - sizes::BUTTON_HEIGHT).abs() < 0.5,
            "a button is {:?} high",
            big.height()
        );
        assert!(
            (small.height() - sizes::SMALL_BUTTON_HEIGHT).abs() < 0.5,
            "a small button is {:?} high",
            small.height()
        );
        assert!(
            (icon.width() - sizes::ICON_BUTTON).abs() < 0.5
                && (icon.height() - sizes::ICON_BUTTON).abs() < 0.5,
            "an icon button is {icon:?}"
        );
    }

    #[test]
    fn tags_badges_and_progress_bars_measure_to_spec() {
        let seen = Arc::new(Mutex::new(None::<(egui::Rect, egui::Rect, egui::Rect, f32)>));
        let mut h = {
            let seen = seen.clone();
            Harness::builder()
                .with_size([600.0, 400.0])
                .with_context_setup(crate::set_up)
                .build_ui(move |ui| {
                    let width = ui.available_width();
                    let tag = tag(ui, Tone::Ok, "Up to date");
                    let badge = badge(ui, Tone::Info, 3);
                    let bar = progress(ui, Some(0.5));
                    *seen.lock().unwrap() = Some((tag.rect, badge.rect, bar.rect, width));
                })
        };
        h.run();
        let (tag, badge, bar, width) = drawn(&seen);
        assert!(
            (tag.height() - sizes::TAG_HEIGHT).abs() < 0.5,
            "a tag is {:?} high",
            tag.height()
        );
        assert!(
            (badge.height() - sizes::BADGE_HEIGHT).abs() < 0.5
                && badge.width() >= sizes::BADGE_MIN_WIDTH - 0.5,
            "a badge is {badge:?}"
        );
        assert!(
            (bar.height() - sizes::PROGRESS_HEIGHT).abs() < 0.5,
            "a progress bar is {:?} high",
            bar.height()
        );
        assert!(
            (bar.width() - width).abs() < 0.5,
            "the progress bar spans {bar:?}, the available width is {width}"
        );
    }

    #[test]
    fn a_section_header_fills_the_width_and_is_clickable() {
        let seen = Arc::new(Mutex::new(None::<(egui::Rect, f32)>));
        let clicked = Arc::new(AtomicBool::new(false));
        let mut h = {
            let seen = seen.clone();
            let clicked = clicked.clone();
            Harness::builder()
                .with_size([600.0, 400.0])
                .with_context_setup(crate::set_up)
                .build_ui(move |ui| {
                    let width = ui.available_width();
                    let header = section_header(ui, "Profiles");
                    *seen.lock().unwrap() = Some((header.rect, width));
                    clicked.store(
                        clicked.load(Ordering::Relaxed) || header.clicked(),
                        Ordering::Relaxed,
                    );
                })
        };
        h.run();
        let (rect, width) = drawn(&seen);
        assert!(
            rect.height() >= sizes::SECTION_MIN_HEIGHT,
            "the header is only {:?} high",
            rect.height()
        );
        assert!(
            (rect.width() - width).abs() < 0.5,
            "the header spans {rect:?}, the available width is {width}"
        );
        h.get_by_label("Profiles").click();
        h.run_steps(2);
        assert!(
            clicked.load(Ordering::Relaxed),
            "the header's own response reported the click"
        );
    }

    #[test]
    fn a_cards_first_child_starts_12_px_from_the_cards_left_edge() {
        let seen = Arc::new(Mutex::new(None::<(egui::Rect, egui::Rect)>));
        let mut h = {
            let seen = seen.clone();
            Harness::builder()
                .with_size([600.0, 400.0])
                .with_context_setup(crate::set_up)
                .build_ui(move |ui| {
                    let shown = card(ui, |ui| section_header(ui, "Profiles").rect);
                    *seen.lock().unwrap() = Some((shown.response.rect, shown.inner));
                })
        };
        h.run();
        let (card_rect, first) = drawn(&seen);
        assert!(
            (first.left() - card_rect.left() - sizes::CARD_INSET).abs() < 0.5,
            "the first child starts at {}, the card's left edge at {}",
            first.left(),
            card_rect.left()
        );
    }

    #[test]
    fn a_click_anywhere_on_a_row_reports_on_the_row() {
        let clicked = Arc::new(AtomicBool::new(false));
        let mut h = {
            let clicked = clicked.clone();
            Harness::builder()
                .with_size([600.0, 400.0])
                .with_context_setup(crate::set_up)
                .build_ui(move |ui| {
                    let row = Row {
                        title: "Scan the mods",
                        sub: None,
                        selected: false,
                    }
                    .show(ui, |ui| {
                        tag(ui, Tone::Ok, "Ready");
                    });
                    clicked.store(
                        clicked.load(Ordering::Relaxed) || row.clicked(),
                        Ordering::Relaxed,
                    );
                })
        };
        h.run();
        // The whole row is one click target (rule 9): the row's node covers its full width, so the element's
        // click lands far from the short title at the row's left end, and still reports on the row.
        h.get_by_label("Scan the mods").click();
        h.run_steps(2);
        assert!(
            clicked.load(Ordering::Relaxed),
            "the click reported on the row's response"
        );
    }
}
