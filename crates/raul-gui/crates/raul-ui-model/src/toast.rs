// SPDX-License-Identifier: GPL-3.0-only
//! Toast queue: timed bottom-centre messages with optional Undo.
//!
//! Implements F-OVERLAY-01 (toasts) from `docs/redesign/FUNCTIONALITY.md` and section 8 of
//! `docs/redesign/SPEC.md`. Undo targets are named here so a caller can run the matching undo
//! and push the confirming "Undone" toast.

/// Seconds a plain toast stays on screen.
pub const PLAIN_SECS: f64 = 3.5;
/// Seconds a toast with an Undo button stays on screen.
pub const UNDO_SECS: f64 = 5.0;
/// Maximum number of toasts visible at the same time.
pub const MAX_VISIBLE: usize = 2;

/// The change a toast can undo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UndoTarget {
    /// Stable identifier of the change, used to de-duplicate confirmation toasts.
    pub change_id: String,
    /// Human-readable description of the change, shown in the toast and Undo tooltip.
    pub label: String,
}

/// A single toast notification.
#[derive(Clone, Debug)]
pub struct Toast {
    /// Increasing identifier, stable for the lifetime of the toast.
    pub id: u64,
    /// Message shown to the user.
    pub text: String,
    /// The change this toast can undo, if any.
    pub undo: Option<UndoTarget>,
    remaining: f64,
}

impl Toast {
    /// How long this toast lives in seconds, plain or with Undo.
    fn lifetime_secs(undo: Option<&UndoTarget>) -> f64 {
        if undo.is_some() {
            UNDO_SECS
        } else {
            PLAIN_SECS
        }
    }
}

/// Queue of toasts with dialog-aware visibility and pausing.
#[derive(Clone, Debug)]
pub struct Toasts {
    next_id: u64,
    dialog_open: bool,
    last_tick: f64,
    items: Vec<Toast>,
}

impl Toasts {
    /// Create an empty toast queue.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            dialog_open: false,
            last_tick: 0.0,
            items: Vec::new(),
        }
    }

    /// Add a toast. Returns its id. If a dialog is open the toast is queued and becomes visible
    /// once the dialog closes. A toast that names the same `change_id` as an existing toast
    /// replaces that toast.
    pub fn push(&mut self, text: impl Into<String>, undo: Option<UndoTarget>, now: f64) -> u64 {
        self.advance(now);
        let id = self.next_id;
        self.next_id += 1;

        if let Some(ref target) = undo {
            let cid = target.change_id.as_str();
            self.items.retain(|t| t.undo.as_ref().map(|u| u.change_id.as_str()) != Some(cid));
        }

        let lifetime = Toast::lifetime_secs(undo.as_ref());
        self.items.push(Toast {
            id,
            text: text.into(),
            undo,
            remaining: lifetime,
        });

        if !self.dialog_open {
            while self.items.len() > MAX_VISIBLE {
                self.items.remove(0);
            }
        }

        self.last_tick = now;
        id
    }

    /// Run timers forward to `now`, expiring toasts whose remaining time reached zero. Time spent
    /// while a dialog is open does not count.
    pub fn tick(&mut self, now: f64) {
        self.advance(now);
    }

    /// Open or close the dialog. While open, toasts are hidden and their timers pause.
    pub fn set_dialog_open(&mut self, open: bool, now: f64) {
        if open == self.dialog_open {
            return;
        }
        self.advance(now);
        self.dialog_open = open;
        if !open {
            while self.items.len() > MAX_VISIBLE {
                self.items.remove(0);
            }
        }
        self.last_tick = now;
    }

    /// Toasts currently on screen, oldest first, at most [`MAX_VISIBLE`]. Empty while a dialog is
    /// open.
    pub fn visible(&self) -> Vec<&Toast> {
        if self.dialog_open {
            return Vec::new();
        }
        self.items
            .iter()
            .filter(|t| t.remaining > 0.0)
            .take(MAX_VISIBLE)
            .collect()
    }

    /// Dismiss the toast with the given id immediately.
    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|t| t.id != id);
    }

    /// Undo the toast with the given id, returning its undo target and removing the toast.
    pub fn undo(&mut self, id: u64) -> Option<UndoTarget> {
        let pos = self.items.iter().position(|t| t.id == id)?;
        let toast = self.items.remove(pos);
        toast.undo
    }

    /// Wall-clock time of the next expiration, for repaint scheduling. `None` while a dialog is
    /// open because timers are paused.
    pub fn next_deadline(&self) -> Option<f64> {
        if self.dialog_open {
            return None;
        }
        let min = self
            .items
            .iter()
            .filter(|t| t.remaining > 0.0)
            .map(|t| t.remaining)
            .min()?;
        Some(self.last_tick + min)
    }

    /// Advance timers to `now`, removing expired toasts. Does nothing while a dialog is open.
    fn advance(&mut self, now: f64) {
        if !self.dialog_open {
            let delta = (now - self.last_tick).max(0.0);
            if delta > 0.0 {
                for t in &mut self.items {
                    t.remaining -= delta;
                }
            }
            self.items.retain(|t| t.remaining > 0.0);
        }
        self.last_tick = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_expires_after_3_5s_not_before() {
        let mut t = Toasts::new();
        let id = t.push("Saved", None, 0.0);
        assert_eq!(t.visible().len(), 1);
        t.tick(3.4);
        assert_eq!(t.visible().len(), 1);
        t.tick(3.6);
        assert!(t.visible().is_empty());
        assert!(t.undo(id).is_none());
    }

    #[test]
    fn with_undo_expires_after_5s() {
        let mut t = Toasts::new();
        t.push(
            "Moved mod",
            Some(UndoTarget {
                change_id: "mv".into(),
                label: "Moved mod".into(),
            }),
            0.0,
        );
        assert_eq!(t.visible().len(), 1);
        t.tick(4.9);
        assert_eq!(t.visible().len(), 1);
        t.tick(5.1);
        assert!(t.visible().is_empty());
    }

    #[test]
    fn third_push_drops_oldest() {
        let mut t = Toasts::new();
        let id1 = t.push("One", None, 0.0);
        let id2 = t.push("Two", None, 0.0);
        let id3 = t.push("Three", None, 0.0);
        let ids: Vec<u64> = t.visible().iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![id2, id3]);
        assert!(t.undo(id1).is_none());
    }

    #[test]
    fn hidden_and_paused_under_a_dialog() {
        let mut t = Toasts::new();
        t.push("Saved", None, 0.0);
        t.tick(1.0);
        t.set_dialog_open(true, 1.0);
        assert!(t.visible().is_empty());
        t.tick(10.0);
        t.set_dialog_open(false, 10.0);
        assert_eq!(t.visible().len(), 1);
        t.tick(11.0);
        assert_eq!(t.visible().len(), 1);
        t.tick(13.6);
        assert!(t.visible().is_empty());
    }

    #[test]
    fn push_during_dialog_shows_after_close() {
        let mut t = Toasts::new();
        t.set_dialog_open(true, 0.0);
        let id = t.push("Queued", None, 1.0);
        assert!(t.visible().is_empty());
        t.set_dialog_open(false, 2.0);
        let vis = t.visible();
        assert_eq!(vis.len(), 1);
        assert_eq!(vis[0].id, id);
    }

    #[test]
    fn undo_returns_target_and_removes_toast() {
        let mut t = Toasts::new();
        let target = UndoTarget {
            change_id: "mv".into(),
            label: "Moved mod".into(),
        };
        let id = t.push("Moved mod", Some(target.clone()), 0.0);
        assert_eq!(t.undo(id), Some(target));
        assert!(t.visible().is_empty());
    }

    #[test]
    fn undone_toast_replaces_same_change_id() {
        let mut t = Toasts::new();
        let target = UndoTarget {
            change_id: "mv".into(),
            label: "Moved mod".into(),
        };
        let id1 = t.push("Moved mod", Some(target.clone()), 0.0);
        let back = t.undo(id1).unwrap();
        let id2 = t.push(format!("Undone: {}", back.label), Some(back), 1.0);
        let vis = t.visible();
        assert_eq!(vis.len(), 1);
        assert_eq!(vis[0].id, id2);
        assert_eq!(vis[0].text, "Undone: Moved mod");
    }

    #[test]
    fn dismiss_removes_at_once() {
        let mut t = Toasts::new();
        let id = t.push("Saved", None, 0.0);
        t.dismiss(id);
        assert!(t.visible().is_empty());
    }

    #[test]
    fn next_deadline_works_and_pauses_under_dialog() {
        let mut t = Toasts::new();
        t.push("A", None, 0.0);
        t.tick(0.0);
        assert_eq!(t.next_deadline(), Some(3.5));
        t.tick(1.0);
        assert_eq!(t.next_deadline(), Some(3.5));
        t.tick(4.0);
        assert_eq!(t.next_deadline(), None);

        let mut d = Toasts::new();
        d.push("A", None, 0.0);
        d.tick(0.0);
        d.set_dialog_open(true, 1.0);
        assert_eq!(d.next_deadline(), None);
    }
}
