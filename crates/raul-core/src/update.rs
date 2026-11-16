// SPDX-License-Identifier: GPL-3.0-only
//! The opt-in update check (ADR-0005; nv-linux ADR-0010): pull-only, run only when the person asks or has turned
//! it on, and it sends nothing that identifies them. It reads the Toolbox's GitHub Releases and offers the newest
//! published release that is newer than the running one; drafts and pre-releases are never offered.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Where the Toolbox's releases are listed.
pub const RELEASES_URL: &str =
    "https://api.github.com/repos/JorisJonkers-dev/rauls-trusty-toolbox/releases?per_page=20";

/// A release newer than the running one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Release {
    /// Version without the leading `v`.
    pub version: String,
    /// The release page.
    pub url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    html_url: String,
}

/// The person's settings that the update check needs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Check for a newer release when the GUI starts. Off unless the person turns it on.
    #[serde(default)]
    pub check_for_updates: bool,
}

impl Settings {
    /// The settings file under the platform's configuration folder (`$XDG_CONFIG_HOME`, `~/.config`,
    /// `%APPDATA%` or `~/Library/Application Support`).
    #[must_use]
    pub fn default_path() -> PathBuf {
        let env = |k| std::env::var_os(k).map(PathBuf::from);
        let base = if cfg!(windows) {
            env("APPDATA")
        } else if cfg!(target_os = "macos") {
            env("HOME").map(|h| h.join("Library/Application Support"))
        } else {
            env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
        };
        base.unwrap_or_default().join("raul/settings.toml")
    }

    /// Reads `path`; a missing or unreadable file gives the defaults (checks off).
    #[must_use]
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Writes the settings to `path`, creating its folder.
    ///
    /// # Errors
    ///
    /// When the folder or the file cannot be written.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string(self).map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }
}

fn parse(version: &str) -> Option<Vec<u64>> {
    version
        .trim_start_matches('v')
        .split('.')
        .map(|p| p.parse().ok())
        .collect()
}

/// The newest published release in `releases_json` (the GitHub Releases API list) that is newer than `current`.
///
/// # Errors
///
/// When `releases_json` is not that list.
pub fn newer_release(current: &str, releases_json: &str) -> Result<Option<Release>, String> {
    let releases: Vec<GithubRelease> =
        serde_json::from_str(releases_json).map_err(|e| format!("release list: {e}"))?;
    let running = parse(current).ok_or_else(|| format!("{current} is not a version"))?;
    Ok(releases
        .into_iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter_map(|r| parse(&r.tag_name).map(|v| (v, r)))
        .filter(|(v, _)| *v > running)
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, r)| Release {
            version: r.tag_name.trim_start_matches('v').to_string(),
            url: r.html_url,
        }))
}
