// SPDX-License-Identifier: GPL-3.0-only
//! The opt-in update check (ADR-0005): pull-only, run only when the person asks or has turned
//! it on, and it sends nothing that identifies them. It reads the Toolbox's GitHub Releases and offers the newest
//! published release that is newer than the running one; drafts and pre-releases are never offered.

use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_runner::{Call, NETWORK_ENV, Runner};
use semver::Version;
use serde::{Deserialize, Serialize};

/// Where the Toolbox's releases are listed.
pub const RELEASES_URL: &str =
    "https://api.github.com/repos/JorisJonkers-dev/rauls-trusty-toolbox/releases?per_page=20";

/// How long `curl` may take for the release list (its own `--max-time`); the Runner's timeout backs it up.
pub const FETCH_TIMEOUT_SECS: u64 = 30;

/// The largest release list accepted, in bytes; `curl` stops at it and the Runner kills a program that writes more.
pub const MAX_LIST_BYTES: usize = 4 * 1024 * 1024;

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
    /// The settings file in the Toolbox's configuration folder ([`crate::config::folder`]), or `None` when the
    /// environment names no such folder (a relative fallback would scatter settings files across working folders).
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        crate::config::folder().map(|f| f.join("settings.toml"))
    }

    /// Reads `path`; a missing or unreadable file gives the defaults (checks off).
    #[must_use]
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Writes these settings into `path`, keeping every other key the file holds, and creating its folder.
    ///
    /// # Errors
    ///
    /// When the existing file does not parse (it is left as it is), or the folder or file cannot be written.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let mut table = match std::fs::read_to_string(path) {
            Ok(text) => text
                .parse::<toml::Table>()
                .map_err(|e| std::io::Error::other(format!("{}: {e}", path.display())))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => toml::Table::new(),
            Err(e) => return Err(e),
        };
        table.insert(
            "check_for_updates".into(),
            toml::Value::Boolean(self.check_for_updates),
        );
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(
            path,
            toml::to_string(&table).map_err(std::io::Error::other)?,
        )
    }
}

/// A version as semver, with or without a leading `v`: `v0.2.0`, `0.3.0-rc.1`, `0.2.0+linux.5`.
fn parse(version: &str) -> Option<Version> {
    Version::parse(version.strip_prefix('v').unwrap_or(version)).ok()
}

/// The newest published release in `releases_json` (the GitHub Releases API list) that is newer than `current`.
///
/// Versions compare by semver precedence: a pre-release comes before its release (`0.2.0-dev.7` is older than
/// `0.2.0`), and build metadata is ignored (`0.2.0+linux.5` is the same release as `0.2.0`). A release whose tag
/// has a pre-release part is never offered, even when GitHub does not mark it as a pre-release; a tag that is not
/// semver is skipped.
///
/// # Errors
///
/// When `releases_json` is not that list, or `current` is not a semver version.
pub fn newer_release(current: &str, releases_json: &str) -> Result<Option<Release>, String> {
    let releases: Vec<GithubRelease> =
        serde_json::from_str(releases_json).map_err(|e| format!("release list: {e}"))?;
    let running = parse(current).ok_or_else(|| format!("{current} is not a version"))?;
    Ok(releases
        .into_iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter_map(|r| parse(&r.tag_name).map(|v| (v, r)))
        .filter(|(v, _)| v.pre.is_empty())
        .filter(|(v, _)| v.cmp_precedence(&running).is_gt())
        .max_by(|a, b| a.0.cmp_precedence(&b.0))
        .map(|(_, r)| Release {
            version: r.tag_name.trim_start_matches('v').to_string(),
            url: r.html_url,
        }))
}

/// The `curl` arguments that fetch `url`: HTTPS only (redirects too), no `.curlrc`, a time and size limit, and
/// nothing that identifies the person beyond what any `curl` sends.
#[must_use]
pub fn curl_args(url: &str) -> Vec<String> {
    [
        // Must come first: ignore the person's `.curlrc`.
        "--disable",
        "--silent",
        "--show-error",
        "--fail",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--max-time",
        &FETCH_TIMEOUT_SECS.to_string(),
        "--max-filesize",
        &MAX_LIST_BYTES.to_string(),
        "--header",
        "Accept: application/vnd.github+json",
        url,
    ]
    .map(str::to_owned)
    .to_vec()
}

/// The Runner the update check uses: the default scrubbed environment plus the proxy and certificate variables
/// a network client needs, a timeout a little past `curl`'s own, and output capped at [`MAX_LIST_BYTES`].
#[must_use]
pub fn runner() -> Runner {
    Runner {
        timeout: Duration::from_secs(FETCH_TIMEOUT_SECS + 15),
        max_output: MAX_LIST_BYTES,
        ..Runner::default()
    }
    .inherit(NETWORK_ENV)
}

/// Fetch the release list with `curl` through `runner` and return the newest release newer than `current`.
///
/// # Errors
///
/// When `curl` cannot be started, fails, runs too long or writes too much, or its answer is not a release list.
pub fn check(runner: &Runner, curl: &Path, current: &str) -> Result<Option<Release>, String> {
    let args = curl_args(RELEASES_URL);
    let out = runner
        .run(&Call::new("curl", curl, &args))
        .map_err(|e| e.to_string())?;
    newer_release(current, &String::from_utf8_lossy(&out.stdout))
}
