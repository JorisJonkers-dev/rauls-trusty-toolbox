// SPDX-License-Identifier: GPL-3.0-only
//! The state of the Mods screen: the list, the search, the selection and the edits staged but not yet applied.
//!
//! The command line has no `nvmod mods` yet that applies an edit (ticket 49), so this state only stages edits and
//! says what they are ([`ModsScreen::pending`]); nothing here changes an Instance.

use crate::error::{Load, ViewError};

/// One Mod of the profile, in load priority order (the first has the highest priority).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModRow {
    /// The Mod's folder name.
    pub name: String,
    /// Whether the profile enables it.
    pub enabled: bool,
}

/// One staged change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModEdit {
    /// Turn a Mod on.
    Enable(String),
    /// Turn a Mod off.
    Disable(String),
    /// The whole new priority order.
    Reorder(Vec<String>),
}

/// The refusals of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModsError {
    /// There is no Mod of this name.
    Unknown(String),
    /// The position is past the end of the list.
    OutOfRange(usize),
    /// Reordering needs the whole list in view: clear the search first.
    SearchActive,
}

/// The Mods screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsScreen {
    load: Load<Vec<ModRow>>,
    baseline: Vec<ModRow>,
    query: String,
    selected: Option<String>,
}

impl Default for ModsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ModsScreen {
    /// A screen whose list is on its way.
    pub fn new() -> Self {
        Self {
            load: Load::Loading,
            baseline: Vec::new(),
            query: String::new(),
            selected: None,
        }
    }

    /// The load state.
    pub fn load(&self) -> &Load<Vec<ModRow>> {
        &self.load
    }

    /// The list was read. Staged edits are dropped: they were against an older list.
    pub fn loaded(&mut self, rows: Vec<ModRow>) {
        self.baseline.clone_from(&rows);
        self.selected = None;
        self.load = if rows.is_empty() {
            Load::Empty("No Mods yet: install one".to_owned())
        } else {
            Load::Ready(rows)
        };
    }

    /// The list could not be read.
    pub fn failed(&mut self, error: ViewError) {
        self.baseline.clear();
        self.selected = None;
        self.load = Load::Failed(error);
    }

    /// Starts a refresh.
    pub fn begin_load(&mut self) {
        self.load = Load::Loading;
    }

    fn rows(&self) -> &[ModRow] {
        self.load.value().map_or(&[], Vec::as_slice)
    }

    fn rows_mut(&mut self) -> Option<&mut Vec<ModRow>> {
        match &mut self.load {
            Load::Ready(v) | Load::Partial { value: v, .. } => Some(v),
            _ => None,
        }
    }

    /// Filters the list by a case-insensitive part of a name. An empty text shows every Mod.
    pub fn set_query(&mut self, text: impl Into<String>) {
        self.query = text.into();
    }

    /// The search text.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The Mods that match the search, in priority order.
    pub fn visible(&self) -> Vec<&ModRow> {
        let q = self.query.to_lowercase();
        self.rows()
            .iter()
            .filter(|r| q.is_empty() || r.name.to_lowercase().contains(&q))
            .collect()
    }

    /// What to show in place of the list when it has no rows.
    pub fn empty_hint(&self) -> Option<String> {
        match &self.load {
            Load::Empty(h) => Some(h.clone()),
            Load::Ready(_) | Load::Partial { .. } if self.visible().is_empty() => {
                Some(format!("No Mod matches \"{}\"", self.query))
            }
            _ => None,
        }
    }

    /// Selects a Mod.
    ///
    /// # Errors
    /// [`ModsError::Unknown`].
    pub fn select(&mut self, name: &str) -> Result<(), ModsError> {
        if !self.rows().iter().any(|r| r.name == name) {
            return Err(ModsError::Unknown(name.to_owned()));
        }
        self.selected = Some(name.to_owned());
        Ok(())
    }

    /// The selected Mod.
    pub fn selected(&self) -> Option<&ModRow> {
        let name = self.selected.as_deref()?;
        self.rows().iter().find(|r| r.name == name)
    }

    /// Stages turning a Mod on or off.
    ///
    /// # Errors
    /// [`ModsError::Unknown`].
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<(), ModsError> {
        let row = self
            .rows_mut()
            .and_then(|r| r.iter_mut().find(|r| r.name == name))
            .ok_or_else(|| ModsError::Unknown(name.to_owned()))?;
        row.enabled = enabled;
        Ok(())
    }

    /// Stages moving a Mod to `index` in the whole list.
    ///
    /// # Errors
    /// [`ModsError::SearchActive`], [`ModsError::Unknown`] or [`ModsError::OutOfRange`].
    pub fn move_to(&mut self, name: &str, index: usize) -> Result<(), ModsError> {
        if !self.query.is_empty() {
            return Err(ModsError::SearchActive);
        }
        let rows = self
            .rows_mut()
            .ok_or_else(|| ModsError::Unknown(name.to_owned()))?;
        let from = rows
            .iter()
            .position(|r| r.name == name)
            .ok_or_else(|| ModsError::Unknown(name.to_owned()))?;
        if index >= rows.len() {
            return Err(ModsError::OutOfRange(index));
        }
        let row = rows.remove(from);
        rows.insert(index, row);
        Ok(())
    }

    /// Stages moving a Mod up (`-1`) or down (`1`) by `delta` places, stopping at the ends.
    ///
    /// # Errors
    /// [`ModsError::SearchActive`] or [`ModsError::Unknown`].
    pub fn move_by(&mut self, name: &str, delta: isize) -> Result<(), ModsError> {
        let from = self
            .rows()
            .iter()
            .position(|r| r.name == name)
            .ok_or_else(|| ModsError::Unknown(name.to_owned()))?;
        let last = self.rows().len() - 1;
        let to = from.saturating_add_signed(delta).min(last);
        self.move_to(name, to)
    }

    /// The staged changes against the list as it was read: enables and disables in list order, then the new
    /// order when it changed.
    pub fn pending(&self) -> Vec<ModEdit> {
        let mut edits = Vec::new();
        for row in self.rows() {
            if let Some(old) = self.baseline.iter().find(|b| b.name == row.name)
                && old.enabled != row.enabled
            {
                edits.push(if row.enabled {
                    ModEdit::Enable(row.name.clone())
                } else {
                    ModEdit::Disable(row.name.clone())
                });
            }
        }
        let names = |rows: &[ModRow]| rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>();
        if names(self.rows()) != names(&self.baseline) {
            edits.push(ModEdit::Reorder(names(self.rows())));
        }
        edits
    }

    /// True when there is something to apply or discard.
    pub fn is_dirty(&self) -> bool {
        !self.pending().is_empty()
    }

    /// Drops every staged change.
    pub fn discard(&mut self) {
        let baseline = self.baseline.clone();
        if let Some(rows) = self.rows_mut() {
            *rows = baseline;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExitClass;

    fn row(name: &str, enabled: bool) -> ModRow {
        ModRow {
            name: name.to_owned(),
            enabled,
        }
    }

    fn screen() -> ModsScreen {
        let mut s = ModsScreen::new();
        s.loaded(vec![
            row("Alpha", true),
            row("Beta", false),
            row("Gamma", true),
        ]);
        s
    }

    fn names(s: &ModsScreen) -> Vec<&str> {
        s.visible().iter().map(|r| r.name.as_str()).collect()
    }

    #[test]
    fn loading_empty_and_failed_states() {
        let mut s = ModsScreen::new();
        assert!(s.load().is_loading());
        s.loaded(Vec::new());
        assert!(s.empty_hint().unwrap().contains("install one"));
        s.failed(ViewError::new(
            ExitClass::Refused,
            "no",
            "Nothing was changed",
        ));
        assert_eq!(s.load().errors().len(), 1);
        assert!(s.visible().is_empty());
        s.begin_load();
        assert!(s.load().is_loading());
    }

    #[test]
    fn search_is_case_insensitive_and_says_when_nothing_matches() {
        let mut s = screen();
        s.set_query("ET");
        assert_eq!(names(&s), ["Beta"]);
        s.set_query("zzz");
        assert_eq!(s.empty_hint().unwrap(), "No Mod matches \"zzz\"");
        s.set_query("");
        assert_eq!(names(&s).len(), 3);
        assert!(s.empty_hint().is_none());
        assert_eq!(s.query(), "");
    }

    #[test]
    fn selection_needs_a_known_mod() {
        let mut s = screen();
        s.select("Gamma").unwrap();
        assert_eq!(s.selected().unwrap().name, "Gamma");
        assert_eq!(s.select("Nope"), Err(ModsError::Unknown("Nope".into())));
    }

    #[test]
    fn staged_edits_are_the_difference_from_the_list_as_read() {
        let mut s = screen();
        assert!(!s.is_dirty());
        s.set_enabled("Beta", true).unwrap();
        s.set_enabled("Alpha", false).unwrap();
        s.move_to("Gamma", 0).unwrap();
        assert_eq!(
            s.pending(),
            [
                ModEdit::Disable("Alpha".into()),
                ModEdit::Enable("Beta".into()),
                ModEdit::Reorder(vec!["Gamma".into(), "Alpha".into(), "Beta".into()]),
            ]
        );
        s.set_enabled("Beta", false).unwrap();
        s.set_enabled("Alpha", true).unwrap();
        s.move_to("Gamma", 2).unwrap();
        assert!(!s.is_dirty(), "undoing every edit leaves nothing to apply");
    }

    #[test]
    fn move_by_stops_at_the_ends_and_reorder_needs_the_whole_list() {
        let mut s = screen();
        s.move_by("Alpha", -1).unwrap();
        s.move_by("Gamma", 5).unwrap();
        assert_eq!(names(&s), ["Alpha", "Beta", "Gamma"]);
        assert!(!s.is_dirty());
        s.move_by("Beta", 1).unwrap();
        assert_eq!(names(&s), ["Alpha", "Gamma", "Beta"]);
        assert_eq!(s.move_to("Alpha", 9), Err(ModsError::OutOfRange(9)));
        s.set_query("a");
        assert_eq!(s.move_to("Alpha", 1), Err(ModsError::SearchActive));
        assert_eq!(
            s.set_enabled("Nope", true),
            Err(ModsError::Unknown("Nope".into()))
        );
    }

    #[test]
    fn discard_restores_the_list_and_a_reload_drops_edits() {
        let mut s = screen();
        s.set_enabled("Beta", true).unwrap();
        s.move_to("Gamma", 0).unwrap();
        s.discard();
        assert!(!s.is_dirty());
        assert_eq!(names(&s), ["Alpha", "Beta", "Gamma"]);
        s.set_enabled("Beta", true).unwrap();
        s.loaded(vec![row("Alpha", true)]);
        assert!(!s.is_dirty());
        assert_eq!(names(&s), ["Alpha"]);
    }
}
