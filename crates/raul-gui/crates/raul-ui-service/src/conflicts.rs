// SPDX-License-Identifier: GPL-3.0-only
//! The read-only calls `nvmod conflicts` and `nvmod tree`: requests and the typed JSON they print.

use crate::service::{push, push_opt};
use serde::Deserialize;

/// What `nvmod conflicts` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConflictsRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
    /// Report only this Mod.
    pub mod_name: Option<String>,
}

impl ConflictsRequest {
    /// The words after `nvmod conflicts`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &self.instance);
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push_opt(&mut rest, "mod", self.mod_name.as_ref());
        rest
    }
}

/// What `nvmod tree` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TreeRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
    /// Only the paths at or under this folder.
    pub path: Option<String>,
}

impl TreeRequest {
    /// The words after `nvmod tree`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &self.instance);
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push_opt(&mut rest, "path", self.path.as_ref());
        rest
    }
}

/// One spelling of a path and who provides it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Source {
    /// The Mod (or `Overwrite`).
    #[serde(rename = "mod")]
    pub name: String,
    /// The path as it spells it.
    pub path: String,
}

/// A path that more than one Mod provides.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Contested {
    /// The lowercased path.
    pub path: String,
    /// Who wins it.
    pub winner: Source,
    /// Who loses it, highest priority first.
    pub losers: Vec<Source>,
}

/// The conflict flags of one Mod.
// The four flags are MO2's own and are printed as four booleans.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModConflict {
    /// The Mod folder name, or `Overwrite`.
    pub name: String,
    /// `mod` or `overwrite`.
    pub kind: String,
    /// 1 is the highest priority; absent for Overwrite.
    pub position: Option<usize>,
    /// Files it provides.
    pub files: usize,
    /// Contested paths it wins.
    pub wins: usize,
    /// Paths it loses.
    pub losses: usize,
    /// Wins at least one contested path.
    pub overwrites: bool,
    /// Loses at least one path.
    pub overwritten: bool,
    /// Both.
    pub mixed: bool,
    /// Loses every file it provides.
    pub redundant: bool,
    /// Whose files it replaces.
    pub overwrites_mods: Vec<String>,
    /// Who replaces its files.
    pub overwritten_by_mods: Vec<String>,
}

/// `nvmod conflicts --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ConflictsReport {
    /// The profile.
    pub profile: String,
    /// Overwrite and the enabled Mods, highest priority first.
    pub mods: Vec<ModConflict>,
    /// Every contested path once.
    pub conflicts: Vec<Contested>,
}

/// One path of the virtual Data folder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TreeEntry {
    /// The lowercased path.
    pub path: String,
    /// The layer that provides it.
    pub provider: String,
    /// The path as that layer spells it.
    pub source_path: String,
}

/// `nvmod tree --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TreeReport {
    /// The profile.
    pub profile: String,
    /// Every path once.
    pub entries: Vec<TreeEntry>,
}
