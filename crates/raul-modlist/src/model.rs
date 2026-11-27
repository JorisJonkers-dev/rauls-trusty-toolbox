// SPDX-License-Identifier: GPL-3.0-only
//! The `raul-modlist/1` data model. Unknown fields are refused, so a typo is an error and not a silent omission.

use serde::{Deserialize, Serialize};

/// The schema name every list must carry.
pub const SCHEMA: &str = "raul-modlist/1";

/// The game a list is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    /// Fallout: New Vegas.
    Fnv,
}

/// The base game a list builds on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    /// New Vegas alone.
    Vanilla,
    /// Tale of Two Wastelands.
    Ttw,
}

/// How the Plugins are ordered after the list is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginOrder {
    /// loot-rs sorts them; a Mod's `plugins` are the names it brings.
    Loot,
}

/// A heading for a run of Mods, for display and for leaving a run out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// Unique slug.
    pub id: String,
    /// What a person reads.
    pub title: String,
}

/// Where a Mod comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Source {
    /// A file on Nexus Mods.
    Nexus {
        /// Nexus's game domain, for example `newvegas`.
        game: String,
        /// The Nexus mod id.
        #[serde(rename = "mod")]
        mod_id: u64,
        /// The Nexus file id; absent when only the upload time is known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file: Option<u64>,
        /// The archive's name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
        /// The archive's size in bytes.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<u64>,
        /// The archive's md5, lower case hex.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        md5: Option<String>,
        /// The upload time (Unix seconds), which names the file when the id is not known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        uploaded: Option<u64>,
    },
    /// A direct download from another host; the hash is required.
    Url {
        /// An `https` address of the file itself.
        url: String,
        /// The file's SHA-256, lower case hex.
        sha256: String,
        /// The archive's name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
    },
    /// A file the person finds or already has; raul only lists it.
    Manual {
        /// Where to look (a page, not necessarily the file).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// The archive's name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
        /// What the person needs to know.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    /// Built by a Component from the person's own files, for example Tale of Two Wastelands.
    Tool {
        /// The Component.
        tool: String,
        /// What it reads.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        inputs: Vec<String>,
    },
    /// Generated last by a Component, for example the LOD.
    Generated {
        /// The Component.
        tool: String,
    },
    /// One of the add-ons raul carries (`raul addons`).
    Addon {
        /// The add-on's id.
        id: String,
    },
}

fn yes() -> bool {
    true
}

/// One Mod of the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModEntry {
    /// Unique slug.
    pub id: String,
    /// The Mod folder name in the Instance.
    pub name: String,
    /// The [`Group`] it belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// `false` lists the Mod but leaves it switched off; `why_off` then says why.
    #[serde(default = "yes")]
    pub enabled: bool,
    /// Why a disabled Mod is off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why_off: Option<String>,
    /// The version a person reads; the file id is what is fetched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The Plugins the Mod brings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<String>,
    /// Files inside the Mod to hide, relative to the Mod.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hide: Vec<String>,
    /// Where it comes from.
    pub source: Source,
    /// A remark for the person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A whole Mod list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModList {
    /// Always [`SCHEMA`].
    pub schema: String,
    /// Unique slug of the list.
    pub id: String,
    /// What a person reads.
    pub name: String,
    /// One paragraph.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The game.
    pub game: Game,
    /// The base game it builds on.
    pub target: Target,
    /// How Plugins are ordered.
    pub plugins: PluginOrder,
    /// The games a person must own, as `fnv` and `fo3`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// The headings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
    /// The Mods in priority order, lowest first: a later Mod wins a file conflict.
    #[serde(default)]
    pub mods: Vec<ModEntry>,
}
