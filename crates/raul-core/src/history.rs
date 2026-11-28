// SPDX-License-Identifier: GPL-3.0-only
//! The undo stack and the version history of a profile (F-OVERLAY-02, F-VERS-01, F-VERS-02; SPEC section 8).
//!
//! The rules are pure: no clock, no I/O. Every call that depends on time takes `now` as milliseconds, so a test
//! fixes the time. The store is `nvmod-journal`, not a second database: each change the Toolbox makes writes one
//! journal entry (`nvmod_journal::begin_kind` ... `complete`), and a [`Change`] points at that entry by its id
//! ([`Change::journal_id`]). A version is a named point in the journal: the journal entries it folded together,
//! oldest first. The journal id starts with the creation time in milliseconds, so [`journal_millis`] gives the
//! `now` of a change that is read back from the journal.
//!
//! Rules:
//! - Undo: a stack per profile, at most [`UNDO_DEPTH`] deep (the oldest is dropped), cleared by a revert.
//!   [`History::undo`] returns the change it takes off, so the toast can name it.
//! - Versions: each change makes a new version, except that a change within [`FOLD_WINDOW_MS`] of the latest
//!   change of the newest version joins that version. A fold is rolling: a burst of quick edits stays one version.
//! - Revert: the profile is back at a version, the undo stack is cleared, newer versions are kept, and the next
//!   change starts a new version even when it comes within the window.

use std::collections::{BTreeMap, VecDeque};

use nvmod_journal::Entry;

/// How many changes a profile's undo stack keeps.
pub const UNDO_DEPTH: usize = 30;

/// Edits closer together than this (milliseconds) are one version.
pub const FOLD_WINDOW_MS: u64 = 30_000;

/// One change to a profile, as the undo stack and the versions hold it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The profile it changed.
    pub profile: String,
    /// What the toast and the Undo tooltip say it was ("Disabled Foo").
    pub label: String,
    /// The id of the `nvmod-journal` entry that holds the backup of this change.
    pub journal_id: String,
    /// When it happened, in milliseconds.
    pub at_ms: u64,
}

impl Change {
    /// A change that wrote the journal entry `entry`; its time is the one the entry's id carries.
    #[must_use]
    pub fn from_entry(profile: &str, label: &str, entry: &Entry) -> Self {
        Self {
            profile: profile.to_owned(),
            label: label.to_owned(),
            journal_id: entry.id.clone(),
            at_ms: journal_millis(&entry.id).unwrap_or(0),
        }
    }
}

/// The creation time, in milliseconds, in a journal id (`{millis:016}-{counter:06}-{pid}`).
#[must_use]
pub fn journal_millis(id: &str) -> Option<u64> {
    id.split('-').next()?.parse().ok()
}

/// A named point in the journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// The number of the version in its profile, from 1; never reused.
    pub number: u64,
    /// The changes folded into it, oldest first.
    pub changes: Vec<Change>,
}

impl Version {
    /// When the newest change in it happened.
    #[must_use]
    pub fn latest_ms(&self) -> u64 {
        self.changes.last().map_or(0, |c| c.at_ms)
    }

    /// The journal entries of the version, oldest first.
    pub fn journal_ids(&self) -> impl Iterator<Item = &str> {
        self.changes.iter().map(|c| c.journal_id.as_str())
    }
}

/// Why a revert did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevertError {
    /// The profile has no such version.
    NoSuchVersion(u64),
}

#[derive(Debug, Default)]
struct ProfileHistory {
    undo: VecDeque<Change>,
    versions: Vec<Version>,
    next_number: u64,
    current: Option<u64>,
    /// The next change starts a new version (after a revert).
    sealed: bool,
}

/// The undo stacks and the versions of every profile.
#[derive(Debug, Default)]
pub struct History {
    profiles: BTreeMap<String, ProfileHistory>,
}

impl History {
    /// An empty history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a change at `now` (milliseconds): push it on its profile's undo stack and put it in a version.
    /// Returns the number of the version it is in.
    pub fn record(&mut self, change: Change, now: u64) -> u64 {
        let p = self.profiles.entry(change.profile.clone()).or_default();
        p.undo.push_back(change.clone());
        while p.undo.len() > UNDO_DEPTH {
            p.undo.pop_front();
        }
        let folds = !p.sealed
            && p.versions
                .last()
                .is_some_and(|v| now.saturating_sub(v.latest_ms()) <= FOLD_WINDOW_MS);
        p.sealed = false;
        let number = if let (true, Some(v)) = (folds, p.versions.last_mut()) {
            v.changes.push(change);
            v.number
        } else {
            p.next_number += 1;
            let number = p.next_number;
            p.versions.push(Version {
                number,
                changes: vec![change],
            });
            number
        };
        p.current = Some(number);
        number
    }

    /// Take the newest change off the profile's undo stack and return it, so the caller undoes that change
    /// (its journal entry) and the toast names it. `None` when there is nothing to undo.
    pub fn undo(&mut self, profile: &str) -> Option<Change> {
        self.profiles.get_mut(profile)?.undo.pop_back()
    }

    /// The change an undo would undo now (for the tooltip), without taking it.
    #[must_use]
    pub fn peek_undo(&self, profile: &str) -> Option<&Change> {
        self.profiles.get(profile)?.undo.back()
    }

    /// How many changes the profile's undo stack holds.
    #[must_use]
    pub fn undo_depth(&self, profile: &str) -> usize {
        self.profiles.get(profile).map_or(0, |p| p.undo.len())
    }

    /// The versions of the profile, newest first.
    #[must_use]
    pub fn versions(&self, profile: &str) -> Vec<&Version> {
        self.profiles
            .get(profile)
            .map_or_else(Vec::new, |p| p.versions.iter().rev().collect())
    }

    /// The number of the version the profile is at.
    #[must_use]
    pub fn current(&self, profile: &str) -> Option<u64> {
        self.profiles.get(profile)?.current
    }

    /// Put the profile at version `number` (earlier or later): the undo stack is cleared and no version is
    /// dropped. Returns the version, whose journal entries the caller restores.
    ///
    /// # Errors
    /// [`RevertError::NoSuchVersion`] when the profile has no such version (nothing changes).
    pub fn revert(&mut self, profile: &str, number: u64) -> Result<&Version, RevertError> {
        let p = self
            .profiles
            .get_mut(profile)
            .ok_or(RevertError::NoSuchVersion(number))?;
        let index = p
            .versions
            .iter()
            .position(|v| v.number == number)
            .ok_or(RevertError::NoSuchVersion(number))?;
        p.undo.clear();
        p.current = Some(number);
        p.sealed = true;
        Ok(&p.versions[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(profile: &str, label: &str, at_ms: u64) -> Change {
        Change {
            profile: profile.to_owned(),
            label: label.to_owned(),
            journal_id: format!("{at_ms:016}-000001-1"),
            at_ms,
        }
    }

    fn record(h: &mut History, profile: &str, label: &str, at_ms: u64) -> u64 {
        h.record(change(profile, label, at_ms), at_ms)
    }

    #[test]
    fn undo_stack_keeps_30_and_drops_the_oldest() {
        let mut h = History::new();
        for i in 0..31_u64 {
            record(&mut h, "a", &format!("c{i}"), i * 100_000);
        }
        assert_eq!(h.undo_depth("a"), 30);
        let mut last = None;
        while let Some(c) = h.undo("a") {
            last = Some(c);
        }
        assert_eq!(last.unwrap().label, "c1", "c0 was dropped");
    }

    #[test]
    fn changes_10_s_apart_are_one_version() {
        let mut h = History::new();
        let a = record(&mut h, "a", "one", 1_000);
        let b = record(&mut h, "a", "two", 11_000);
        assert_eq!(a, b);
        assert_eq!(h.versions("a").len(), 1);
        assert_eq!(h.versions("a")[0].changes.len(), 2);
    }

    #[test]
    fn changes_31_s_apart_are_two_versions_newest_first() {
        let mut h = History::new();
        record(&mut h, "a", "one", 1_000);
        record(&mut h, "a", "two", 32_000);
        let numbers: Vec<u64> = h.versions("a").iter().map(|v| v.number).collect();
        assert_eq!(numbers, vec![2, 1]);
    }

    #[test]
    fn a_change_exactly_30_s_after_joins() {
        let mut h = History::new();
        record(&mut h, "a", "one", 0);
        record(&mut h, "a", "two", 30_000);
        assert_eq!(h.versions("a").len(), 1);
    }

    #[test]
    fn revert_clears_the_undo_stack_and_keeps_newer_versions() {
        let mut h = History::new();
        record(&mut h, "a", "one", 0);
        record(&mut h, "a", "two", 100_000);
        assert_eq!(h.undo_depth("a"), 2);
        let v = h.revert("a", 1).unwrap();
        assert_eq!(v.number, 1);
        assert_eq!(h.undo_depth("a"), 0);
        assert!(h.undo("a").is_none());
        assert_eq!(h.versions("a").len(), 2, "the newer version is kept");
        assert_eq!(h.current("a"), Some(1));
    }

    #[test]
    fn revert_to_a_missing_version_changes_nothing() {
        let mut h = History::new();
        record(&mut h, "a", "one", 0);
        assert_eq!(h.revert("a", 9).unwrap_err(), RevertError::NoSuchVersion(9));
        assert_eq!(h.revert("b", 1).unwrap_err(), RevertError::NoSuchVersion(1));
        assert_eq!(h.undo_depth("a"), 1);
    }

    #[test]
    fn the_first_change_after_a_revert_is_a_new_version() {
        let mut h = History::new();
        record(&mut h, "a", "one", 0);
        h.revert("a", 1).unwrap();
        let n = record(&mut h, "a", "two", 1_000);
        assert_eq!(n, 2);
    }

    #[test]
    fn undo_returns_the_change_it_names_newest_first() {
        let mut h = History::new();
        record(&mut h, "a", "Disabled Foo", 0);
        record(&mut h, "a", "Moved Bar", 1_000);
        assert_eq!(h.peek_undo("a").unwrap().label, "Moved Bar");
        let c = h.undo("a").unwrap();
        assert_eq!(c.label, "Moved Bar");
        assert_eq!(c.journal_id, "0000000000001000-000001-1");
        assert_eq!(h.undo("a").unwrap().label, "Disabled Foo");
        assert!(h.undo("a").is_none());
    }

    #[test]
    fn stacks_and_versions_are_per_profile() {
        let mut h = History::new();
        record(&mut h, "a", "a1", 0);
        record(&mut h, "b", "b1", 1_000);
        record(&mut h, "b", "b2", 2_000);
        assert_eq!(h.undo("a").unwrap().label, "a1");
        assert_eq!(h.undo_depth("b"), 2);
        assert_eq!(h.versions("b").len(), 1, "b folded on its own");
        h.revert("b", 1).unwrap();
        record(&mut h, "a", "a2", 3_000);
        assert_eq!(h.undo_depth("a"), 1, "a revert in b leaves a alone");
    }

    #[test]
    fn the_journal_id_carries_the_time() {
        assert_eq!(
            journal_millis("0000001700000000-000003-42"),
            Some(1_700_000_000)
        );
        assert_eq!(journal_millis("junk"), None);
    }
}
