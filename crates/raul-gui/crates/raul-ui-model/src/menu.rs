// SPDX-License-Identifier: GPL-3.0-only
//! Pure model for anchored dropdown menus (F-OVERLAY-07, SPEC section 5 and section 7).

/// Rectangle in window coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

/// Gap between the anchor and the menu.
pub const GAP: f32 = 6.0;

/// Minimum clearance from the window edges.
pub const MARGIN: f32 = 8.0;

/// Minimum menu width.
pub const MIN_WIDTH: f32 = 220.0;

/// Height of one selectable menu row.
pub const ITEM_HEIGHT: f32 = 38.0;

/// Menu outer padding.
pub const PADDING: f32 = 6.0;

/// Height of a separator line.
pub const SEPARATOR_HEIGHT: f32 = 9.0;

/// Total height of a menu with the given numbers of items and separators.
pub fn menu_height(items: usize, separators: usize) -> f32 {
    PADDING * 2.0 + items as f32 * ITEM_HEIGHT + separators as f32 * SEPARATOR_HEIGHT
}

/// Places a menu left-aligned with its anchor, below by default, flipping above when needed,
/// then clamping it inside the window margins.
pub fn place(anchor: Rect, size: (f32, f32), window: (f32, f32)) -> Rect {
    let w = size.0.max(MIN_WIDTH);
    let h = size.1;
    let mut x = anchor.x;
    let mut y = anchor.y + anchor.h + GAP;
    if y + h > window.1 - MARGIN {
        y = anchor.y - GAP - h;
    }
    let max_x = window.0 - w - MARGIN;
    x = if max_x < MARGIN {
        MARGIN
    } else {
        x.clamp(MARGIN, max_x)
    };
    let max_y = window.1 - h - MARGIN;
    y = if max_y < MARGIN {
        MARGIN
    } else {
        y.clamp(MARGIN, max_y)
    };
    Rect { x, y, w, h }
}

/// One row of a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// Visible text.
    pub label: String,
    /// Whether it can be focused or activated.
    pub enabled: bool,
    /// Whether a separator line is drawn before this item.
    pub separator_before: bool,
}

/// A key the menu reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Move to the previous enabled item.
    Up,
    /// Move to the next enabled item.
    Down,
    /// Move to the first enabled item.
    Home,
    /// Move to the last enabled item.
    End,
    /// Close the menu without returning focus to the anchor.
    Tab,
    /// Close the menu and return focus to the anchor.
    Escape,
    /// Activate the focused item.
    Enter,
}

/// Result of handling a key in an open menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    /// Keep the menu open.
    Stay,
    /// Close the menu; `focus_anchor` tells the toolkit whether to focus the anchor.
    Close {
        /// Whether focus should return to the anchor.
        focus_anchor: bool,
    },
    /// Activate the item at this index and close the menu.
    Activate(usize),
}

/// Focus and keyboard state of an open menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuState {
    focused: Option<usize>,
}

impl MenuState {
    /// Opens a menu, focusing the first enabled item.
    pub fn open(items: &[MenuItem]) -> Self {
        Self {
            focused: items.iter().position(|it| it.enabled),
        }
    }

    /// The index of the currently focused item, if any.
    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    /// Handles a key and returns what the toolkit should do.
    pub fn key(&mut self, items: &[MenuItem], key: Key) -> Outcome {
        match key {
            Key::Up | Key::Down => {
                if let Some(i) = self.focused {
                    if let Some(next) = next_enabled(items, i, key == Key::Down) {
                        self.focused = Some(next);
                    }
                }
                Outcome::Stay
            }
            Key::Home => {
                self.focused = items.iter().position(|it| it.enabled);
                Outcome::Stay
            }
            Key::End => {
                self.focused = items.iter().rposition(|it| it.enabled);
                Outcome::Stay
            }
            Key::Tab => Outcome::Close {
                focus_anchor: false,
            },
            Key::Escape => Outcome::Close {
                focus_anchor: true,
            },
            Key::Enter => match self.focused {
                Some(i) => Outcome::Activate(i),
                None => Outcome::Stay,
            },
        }
    }
}

/// Finds the next enabled item in the given direction, wrapping and skipping disabled items.
fn next_enabled(items: &[MenuItem], from: usize, down: bool) -> Option<usize> {
    let n = items.len();
    if n == 0 {
        return None;
    }
    let mut i = from;
    for _ in 0..n {
        i = if down {
            if i + 1 < n {
                i + 1
            } else {
                0
            }
        } else if i == 0 {
            n - 1
        } else {
            i - 1
        };
        if items[i].enabled {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(label: &str, enabled: bool) -> MenuItem {
        MenuItem {
            label: label.into(),
            enabled,
            separator_before: false,
        }
    }

    #[test]
    fn menu_is_placed_below_anchor_with_gap() {
        let r = place(
            Rect {
                x: 100.0,
                y: 50.0,
                w: 80.0,
                h: 30.0,
            },
            (120.0, 80.0),
            (1280.0, 720.0),
        );
        assert_eq!(r.x, 100.0);
        assert_eq!(r.y, 50.0 + 30.0 + GAP);
        // Narrower than the minimum, so it widens to 220.
        assert_eq!(r.w, MIN_WIDTH);
        assert_eq!(r.h, 80.0);
    }

    #[test]
    fn menu_flips_above_when_no_room_below() {
        let anchor = Rect {
            x: 100.0,
            y: 650.0,
            w: 80.0,
            h: 30.0,
        };
        let r = place(anchor, (120.0, 100.0), (1280.0, 720.0));
        assert_eq!(r.y, anchor.y - GAP - 100.0);
    }

    #[test]
    fn menu_is_clamped_inside_window_margins() {
        let r = place(
            Rect {
                x: 950.0,
                y: 100.0,
                w: 30.0,
                h: 30.0,
            },
            (200.0, 200.0),
            (1000.0, 200.0),
        );
        assert_eq!(r.x, 1000.0 - MIN_WIDTH - MARGIN);
        assert_eq!(r.y, MARGIN);
        assert_eq!(r.w, MIN_WIDTH);
        assert_eq!(r.h, 200.0);
    }

    #[test]
    fn menu_uses_minimum_width() {
        let r = place(
            Rect {
                x: 100.0,
                y: 100.0,
                w: 80.0,
                h: 30.0,
            },
            (100.0, 80.0),
            (1280.0, 720.0),
        );
        assert_eq!(r.w, MIN_WIDTH);
    }

    #[test]
    fn open_focuses_first_enabled_item() {
        let items = vec![item("A", false), item("B", true), item("C", true)];
        let state = MenuState::open(&items);
        assert_eq!(state.focused(), Some(1));
    }

    #[test]
    fn down_wraps_from_last_to_first_and_skips_disabled() {
        let items = vec![item("A", true), item("B", false), item("C", true)];
        let mut state = MenuState::open(&items);
        assert_eq!(state.focused(), Some(0));
        assert_eq!(state.key(&items, Key::Down), Outcome::Stay);
        assert_eq!(state.focused(), Some(2));
        assert_eq!(state.key(&items, Key::Down), Outcome::Stay);
        assert_eq!(state.focused(), Some(0));
    }

    #[test]
    fn up_wraps_from_first_to_last_and_skips_disabled() {
        let items = vec![item("A", true), item("B", false), item("C", true)];
        let mut state = MenuState::open(&items);
        assert_eq!(state.key(&items, Key::Up), Outcome::Stay);
        assert_eq!(state.focused(), Some(2));
        assert_eq!(state.key(&items, Key::Up), Outcome::Stay);
        assert_eq!(state.focused(), Some(0));
    }

    #[test]
    fn home_and_end_jump_to_enabled_ends() {
        let items = vec![item("A", false), item("B", true), item("C", true), item("D", false)];
        let mut state = MenuState::open(&items);
        state.key(&items, Key::Down);
        assert_eq!(state.focused(), Some(2));
        assert_eq!(state.key(&items, Key::Home), Outcome::Stay);
        assert_eq!(state.focused(), Some(1));
        assert_eq!(state.key(&items, Key::End), Outcome::Stay);
        assert_eq!(state.focused(), Some(2));
    }

    #[test]
    fn tab_closes_without_focusing_anchor() {
        let items = vec![item("A", true)];
        let mut state = MenuState::open(&items);
        assert_eq!(
            state.key(&items, Key::Tab),
            Outcome::Close {
                focus_anchor: false
            }
        );
    }

    #[test]
    fn escape_closes_and_focuses_anchor() {
        let items = vec![item("A", true)];
        let mut state = MenuState::open(&items);
        assert_eq!(
            state.key(&items, Key::Escape),
            Outcome::Close {
                focus_anchor: true
            }
        );
    }

    #[test]
    fn enter_activates_focused_index() {
        let items = vec![item("A", true), item("B", true)];
        let mut state = MenuState::open(&items);
        state.key(&items, Key::Down);
        assert_eq!(state.key(&items, Key::Enter), Outcome::Activate(1));
    }

    #[test]
    fn no_enabled_item_means_no_focus_and_enter_stays() {
        let items = vec![item("A", false), item("B", false)];
        let mut state = MenuState::open(&items);
        assert_eq!(state.focused(), None);
        assert_eq!(state.key(&items, Key::Enter), Outcome::Stay);
    }

    #[test]
    fn height_counts_items_and_separators() {
        assert_eq!(
            menu_height(3, 1),
            PADDING * 2.0 + 3.0 * ITEM_HEIGHT + SEPARATOR_HEIGHT
        );
    }
}
