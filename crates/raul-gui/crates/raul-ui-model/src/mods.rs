// SPDX-License-Identifier: GPL-3.0-only
//! The state of the Mods screen: the list, the search, the selection and the edits staged but not yet applied.
//!
//! This state only stages edits and says what they are ([`ModsScreen::pending`]) and which `raul mods` steps make
//! them ([`ModsScreen::plan`]); nothing here changes an Instance. The service crate runs the steps.

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

/// One `raul mods` call that carries out part of the staged edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModStep {
    /// `raul mods enable <mod>...`.
    Enable(Vec<String>),
    /// `raul mods disable <mod>...`.
    Disable(Vec<String>),
    /// `raul mods reorder <mod> --position <n>`: the Mod ends at this 1-based place in the whole list.
    Move {
        /// The Mod.
        name: String,
        /// Where it ends up, counted from 1 (the highest priority).
        position: usize,
    },
}

/// The moves that turn the order `from` into the order `to` (the same names), each as the Mod and the 1-based
/// place it is moved to. The Mods of a longest run that already keep their relative order stay where they are,
/// so moving one Mod is one step.
fn moves_between(from: &[String], to: &[String]) -> Vec<(String, usize)> {
    let rank = |name: &String| to.iter().position(|t| t == name).unwrap_or(usize::MAX);
    let ranks: Vec<usize> = from.iter().map(rank).collect();
    // Longest increasing subsequence of the ranks (quadratic: Mod lists are a few hundred long at most).
    let n = ranks.len();
    let mut len = vec![1_usize; n];
    let mut prev = vec![usize::MAX; n];
    for i in 0..n {
        for j in 0..i {
            if ranks[j] < ranks[i] && len[j] + 1 > len[i] {
                len[i] = len[j] + 1;
                prev[i] = j;
            }
        }
    }
    let mut keep = vec![false; n];
    if let Some(mut at) = (0..n).max_by_key(|&i| (len[i], std::cmp::Reverse(i))) {
        loop {
            keep[at] = true;
            if prev[at] == usize::MAX {
                break;
            }
            at = prev[at];
        }
    }
    let stay: Vec<&String> = from
        .iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(name, _)| name)
        .collect();
    let mut current = from.to_vec();
    let mut moves = Vec::new();
    for (i, name) in to.iter().enumerate() {
        if stay.contains(&name) {
            continue;
        }
        current.retain(|c| c != name);
        let at = if i == 0 {
            0
        } else {
            current
                .iter()
                .position(|c| *c == to[i - 1])
                .map_or(0, |p| p + 1)
        };
        current.insert(at, name.clone());
        moves.push((name.clone(), at + 1));
    }
    moves
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

    /// The `raul mods` calls that carry out [`ModsScreen::pending`], in the order to run them: one enable, one
    /// disable, then the moves. Enabling and disabling do not change places and moving does not change state, so
    /// the first two are independent of the moves; each move is counted against the list after the moves before it.
    pub fn plan(&self) -> Vec<ModStep> {
        let mut steps = Vec::new();
        let mut order = None;
        let (mut on, mut off) = (Vec::new(), Vec::new());
        for edit in self.pending() {
            match edit {
                ModEdit::Enable(n) => on.push(n),
                ModEdit::Disable(n) => off.push(n),
                ModEdit::Reorder(names) => order = Some(names),
            }
        }
        if !on.is_empty() {
            steps.push(ModStep::Enable(on));
        }
        if !off.is_empty() {
            steps.push(ModStep::Disable(off));
        }
        if let Some(to) = order {
            let from: Vec<String> = self.baseline.iter().map(|r| r.name.clone()).collect();
            steps.extend(
                moves_between(&from, &to)
                    .into_iter()
                    .map(|(name, position)| ModStep::Move { name, position }),
            );
        }
        steps
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

    fn apply_moves(from: &[String], moves: &[ModStep]) -> Vec<String> {
        let mut cur = from.to_vec();
        for m in moves {
            let ModStep::Move { name, position } = m else {
                panic!("not a move")
            };
            cur.retain(|c| c != name);
            cur.insert(position - 1, name.clone());
        }
        cur
    }

    #[test]
    fn the_plan_is_an_enable_a_disable_and_the_fewest_moves() {
        let mut s = screen();
        s.set_enabled("Beta", true).unwrap();
        s.set_enabled("Alpha", false).unwrap();
        s.set_enabled("Gamma", false).unwrap();
        s.move_to("Alpha", 2).unwrap();
        assert_eq!(
            s.plan(),
            [
                ModStep::Enable(vec!["Beta".into()]),
                ModStep::Disable(vec!["Gamma".into(), "Alpha".into()]),
                ModStep::Move {
                    name: "Alpha".into(),
                    position: 3
                },
            ]
        );
        assert!(screen().plan().is_empty());
    }

    #[test]
    fn every_reorder_of_five_mods_is_carried_out_by_the_planned_moves() {
        fn perms(items: &mut Vec<String>, k: usize, out: &mut Vec<Vec<String>>) {
            if k == items.len() {
                out.push(items.clone());
                return;
            }
            for i in k..items.len() {
                items.swap(k, i);
                perms(items, k + 1, out);
                items.swap(k, i);
            }
        }
        let base: Vec<String> = ["A", "B", "C", "D", "E"].map(String::from).to_vec();
        let mut all = Vec::new();
        perms(&mut base.clone(), 0, &mut all);
        for target in &all {
            let moves = moves_between(&base, target);
            let steps: Vec<ModStep> = moves
                .into_iter()
                .map(|(name, position)| ModStep::Move { name, position })
                .collect();
            assert_eq!(&apply_moves(&base, &steps), target);
            assert!(steps.len() < base.len());
        }
        // One Mod moved to the end is one step.
        let target: Vec<String> = ["B", "C", "D", "E", "A"].map(String::from).to_vec();
        assert_eq!(moves_between(&base, &target).len(), 1);
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
