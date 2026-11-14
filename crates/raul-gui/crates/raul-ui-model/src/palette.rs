// SPDX-License-Identifier: GPL-3.0-only
//! The command palette: every action with the command line that does the same (documents 02 and 04).

use crate::keymap::{Chord, Command, Keymap};
use crate::route::{RAIL, Screen};
use serde::{Deserialize, Serialize};

/// What running an entry does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Do {
    /// Go to a screen.
    Go(Screen),
    /// Run a keyboard command.
    Run(Command),
}

/// One row of the palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The label.
    pub title: String,
    /// The shortcut, when one is bound.
    pub shortcut: Option<Chord>,
    /// The command line that does the same, or `None` for presentation only (`gui_only: presentation`).
    pub cli_equivalent: Option<String>,
    /// What it does.
    pub action: Do,
}

/// Every palette entry, in a stable order: screens of the inventory, then keyboard commands.
/// `private` includes the Real data screen (the `private-oracle` build).
pub fn entries(keymap: &Keymap, private: bool) -> Vec<Entry> {
    let mut v = Vec::new();
    for s in Screen::ALL {
        if s == Screen::Oracle && !private {
            continue;
        }
        let shortcut = s.rail_index().and_then(|i| {
            keymap
                .chords_for(Command::Rail(u8::try_from(i + 1).unwrap_or(0)))
                .first()
                .copied()
        });
        v.push(Entry {
            title: format!("Go to {}", s.title()),
            shortcut,
            cli_equivalent: Some(s.cli().to_owned()),
            action: Do::Go(s),
        });
    }
    for c in [
        Command::Filter,
        Command::Toggle,
        Command::MoveUp,
        Command::MoveDown,
        Command::Why,
        Command::Undo,
        Command::ApplyReviewed,
        Command::Help,
    ] {
        v.push(Entry {
            title: c.title(),
            shortcut: keymap.chords_for(c).first().copied(),
            cli_equivalent: None,
            action: Do::Run(c),
        });
    }
    v
}

/// Whether the characters of `query` appear in `text` in order, ignoring case.
fn subsequence(query: &str, text: &str) -> Option<usize> {
    let text = text.to_lowercase();
    let mut pos = 0usize;
    let mut gap = 0usize;
    for q in query.to_lowercase().chars().filter(|c| !c.is_whitespace()) {
        let found = text[pos..].find(q)?;
        gap += found;
        pos += found + q.len_utf8();
    }
    Some(gap)
}

/// The palette: a query and the entries that match it, best first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
    all: Vec<Entry>,
    query: String,
    selected: usize,
    open: bool,
}

impl Palette {
    /// A closed palette over the entries.
    pub fn new(all: Vec<Entry>) -> Self {
        Self {
            all,
            query: String::new(),
            selected: 0,
            open: false,
        }
    }

    /// Whether it is shown.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// `Ctrl+K`: opens with an empty query.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
    }

    /// `Esc`.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// The query typed.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Sets the query and selects the first result.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.selected = 0;
    }

    /// The matches, best first: a smaller gap between the matched letters ranks higher, ties keep the order of
    /// the table.
    pub fn results(&self) -> Vec<&Entry> {
        let mut scored: Vec<(usize, usize, &Entry)> = self
            .all
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let by_title = subsequence(&self.query, &e.title);
                let by_cli = e
                    .cli_equivalent
                    .as_deref()
                    .and_then(|c| subsequence(&self.query, c));
                by_title
                    .into_iter()
                    .chain(by_cli.map(|g| g + 100))
                    .min()
                    .map(|g| (g, i, e))
            })
            .collect();
        scored.sort_by_key(|(g, i, _)| (*g, *i));
        scored.into_iter().map(|(_, _, e)| e).collect()
    }

    /// The highlighted entry.
    pub fn selected(&self) -> Option<&Entry> {
        self.results().get(self.selected).copied()
    }

    /// Moves the highlight; it stops at both ends.
    pub fn move_selection(&mut self, down: bool) {
        let n = self.results().len();
        if down {
            if self.selected + 1 < n {
                self.selected += 1;
            }
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Enter: closes the palette and returns what to do.
    pub fn run_selected(&mut self) -> Option<Do> {
        let action = self.selected().map(|e| e.action.clone());
        if action.is_some() {
            self.open = false;
        }
        action
    }
}

/// The screens `Ctrl+1` to `Ctrl+8` reach, as the rail shows them.
pub fn rail_titles() -> Vec<&'static str> {
    RAIL.iter().map(|s| s.title()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::Key;

    fn palette(private: bool) -> Palette {
        Palette::new(entries(&Keymap::default(), private))
    }

    #[test]
    fn public_build_has_no_real_data_entry() {
        assert!(
            palette(false)
                .results()
                .iter()
                .all(|e| e.action != Do::Go(Screen::Oracle))
        );
        assert!(
            palette(true)
                .results()
                .iter()
                .any(|e| e.action == Do::Go(Screen::Oracle))
        );
    }

    #[test]
    fn screens_show_their_cli_and_rail_shortcut() {
        let p = palette(false);
        let doctor = p
            .results()
            .into_iter()
            .find(|e| e.action == Do::Go(Screen::Doctor))
            .unwrap();
        assert_eq!(
            doctor.cli_equivalent.as_deref(),
            Some("nvmod doctor --json")
        );
        assert_eq!(doctor.shortcut, Some(Chord::ctrl(Key::Char('5'))));
        let fomod = p
            .results()
            .into_iter()
            .find(|e| e.action == Do::Go(Screen::Fomod))
            .unwrap();
        assert_eq!(fomod.shortcut, None);
        let help = p
            .results()
            .into_iter()
            .find(|e| e.action == Do::Run(Command::Help))
            .unwrap();
        assert!(help.cli_equivalent.is_none());
        assert_eq!(help.shortcut, Some(Chord::plain(Key::F(1))));
    }

    #[test]
    fn typing_filters_and_ranks() {
        let mut p = palette(false);
        p.open();
        assert!(p.is_open());
        let all = p.results().len();
        p.set_query("doc");
        let r = p.results();
        assert!(r.len() < all);
        assert_eq!(r[0].action, Do::Go(Screen::Doctor));
        p.set_query("nvmod ttw");
        assert_eq!(p.results()[0].action, Do::Go(Screen::Ttw));
        p.set_query("zzzzqq");
        assert!(p.results().is_empty());
        assert!(p.selected().is_none());
        assert!(p.run_selected().is_none());
        assert!(p.is_open());
        assert_eq!(p.query(), "zzzzqq");
    }

    #[test]
    fn selection_moves_and_runs() {
        let mut p = palette(false);
        p.open();
        p.set_query("go to h");
        p.move_selection(false);
        let first = p.selected().unwrap().clone();
        p.move_selection(true);
        assert_ne!(p.selected().unwrap(), &first);
        for _ in 0..100 {
            p.move_selection(true);
        }
        assert!(p.selected().is_some());
        p.set_query("home");
        assert_eq!(p.run_selected(), Some(Do::Go(Screen::Home)));
        assert!(!p.is_open());
        p.open();
        p.close();
        assert!(!p.is_open());
    }

    #[test]
    fn rail_titles_match_the_rail() {
        assert_eq!(rail_titles().len(), 8);
        assert_eq!(rail_titles()[0], "Home");
    }
}
