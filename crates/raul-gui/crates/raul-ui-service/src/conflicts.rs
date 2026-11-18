// SPDX-License-Identifier: GPL-3.0-only
//! The read-only calls `raul conflicts` and `raul tree`: requests and the typed JSON they print.

use crate::service::{push, push_opt};
use serde::Deserialize;

/// What `raul conflicts` needs.
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
    /// The words after `raul conflicts`, before `--json`.
    pub fn args(&self) -> Vec<String> {
        let mut rest = Vec::new();
        push(&mut rest, "instance", &self.instance);
        push_opt(&mut rest, "profile", self.profile.as_ref());
        push_opt(&mut rest, "mod", self.mod_name.as_ref());
        rest
    }
}

/// What `raul tree` needs.
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
    /// The words after `raul tree`, before `--json`.
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

/// `raul conflicts --json`.
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

/// `raul tree --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TreeReport {
    /// The profile.
    pub profile: String,
    /// Every path once.
    pub entries: Vec<TreeEntry>,
}

/// What `raul plugins list` needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PluginsRequest {
    /// The Instance folder.
    pub instance: String,
    /// The profile, `None` for the default.
    pub profile: Option<String>,
}

/// One master a plugin declares.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MasterRow {
    /// The master's file name.
    pub name: String,
    /// A Mod or the base copy provides the file.
    pub present: bool,
    /// Listed active.
    pub active: bool,
}

/// One plugin of `raul plugins list --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PluginRow {
    /// The file name.
    pub name: String,
    /// Listed in `plugins.txt`.
    pub active: bool,
    /// 1 is the first to load.
    pub position: usize,
    /// The 1-based position it is locked at.
    pub locked: Option<usize>,
    /// A forced primary plugin.
    pub forced: bool,
    /// A Mod or the base copy provides the file.
    pub installed: bool,
    /// The providing Mod.
    pub provider: Option<String>,
    /// The masters it declares.
    pub masters: Vec<MasterRow>,
    /// How many masters are not present.
    pub masters_missing: usize,
    /// Why its header could not be read.
    pub header_error: Option<String>,
}

/// One entry of `lockedorder.txt`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LockRow {
    /// The plugin file name.
    pub name: String,
    /// The 1-based position.
    pub position: usize,
    /// The plugin is in the list.
    pub listed: bool,
}

/// `raul plugins list --json` (schema `schemas/plugins-list.v1.json`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PluginsReport {
    /// The version of the layout.
    pub schema_version: u32,
    /// The profile.
    pub profile: String,
    /// The plugins in load order.
    pub plugins: Vec<PluginRow>,
    /// Every lock.
    pub locks: Vec<LockRow>,
}
