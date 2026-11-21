// SPDX-License-Identifier: GPL-3.0-only
//! The pinned data file the Toolbox fetches and checks itself: the LOOT masterlist behind
//! `raul sort --fetch-masterlist` (rauls-trusty-toolbox#449; the Toolbox owns the fetcher and the store, nvmod-rs
//! no longer needs them).
//!
//! The pin is a `tools.lock.toml` entry (the format nvmod-rs used, so `--lock <file>` takes either): a URL and a
//! SHA-256 per platform, or `any` for a data file. [`fetch`] gets the bytes from that URL and nowhere else, checks
//! the SHA-256, and keeps the file in the store under its hash (`<store>/<sha256>/<name>`), where it is hashed again
//! every time it is used. A failed check stores nothing. A third-party file whose entry pins no checksum is refused
//! unless the Instance holds a confirmation (`tools.trust.toml`) for exactly that version. The store is the one
//! nvmod-rs kept (`NVMOD_TOOLS_DIR`, else the user's data folder under `nvmod/tools`), so a file fetched earlier is
//! found again.
//!
//! The request goes out through `curl` started by the one Runner (ADR-0007), which every supported system has and
//! which gives `https`, redirects, a hard timeout and `file://` without a TLS stack in the Toolbox. An in-process
//! HTTPS client is a later decision (an ADR comes before any HTTP crate). An entry that pins a build attestation is
//! refused: this build checks checksums only.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_runner::{Call, NETWORK_ENV, Runner, ToolError, sha256_hex};
use serde::Deserialize;

/// The tool of the lock that pins the masterlist.
pub const MASTERLIST_TOOL: &str = "loot-masterlist-falloutnv";

/// The time one request may take.
pub const TIMEOUT_SECS: u32 = 60;
/// How often a failed request is tried again after the first try.
pub const RETRIES: u32 = 3;
/// The largest response accepted.
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
/// The file name of the Instance's confirmations.
pub const TRUST_FILE: &str = "tools.trust.toml";

/// The pin file that ships with the Toolbox.
const BUILTIN_LOCK: &str = include_str!("../data/pinned.lock.toml");

/// Why a request gave no usable answer.
#[derive(Debug, PartialEq, Eq)]
pub enum FetchError {
    /// The answer is not a well-formed response (not HTTP, or cut short). Never retried.
    Malformed(String),
    /// The server or the network failed or timed out. Retried.
    Transient(String),
    /// Retrying cannot help (the client program is missing, the URL is not usable).
    Fatal(String),
}

/// Something that fetches the bytes behind a URL.
pub trait Fetcher {
    /// Fetch `url`.
    ///
    /// # Errors
    /// See [`FetchError`].
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

/// The `curl` command line as a [`Fetcher`].
#[derive(Debug, Clone)]
pub struct CurlFetcher {
    /// The `curl` program.
    pub program: PathBuf,
}

impl Default for CurlFetcher {
    fn default() -> Self {
        Self {
            program: PathBuf::from("curl"),
        }
    }
}

impl CurlFetcher {
    /// The `curl` arguments that write the body of `url` to standard output.
    #[must_use]
    pub fn args(url: &str) -> Vec<String> {
        [
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--proto",
            "=http,https,file",
            "--proto-redir",
            "=http,https",
            "--max-time",
            &TIMEOUT_SECS.to_string(),
            "--max-filesize",
            &MAX_BYTES.to_string(),
            url,
        ]
        .map(str::to_owned)
        .to_vec()
    }
}

impl Fetcher for CurlFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        // curl has its own `--max-time`; the runner's timeout only backs it up.
        let runner = Runner {
            timeout: Duration::from_secs(u64::from(TIMEOUT_SECS) + 30),
            max_output: usize::try_from(MAX_BYTES).unwrap_or(usize::MAX),
            ..Runner::default()
        }
        .inherit(NETWORK_ENV)
        .inherit(&["HOME"]);
        let args = Self::args(url);
        let out = runner
            .run_any(&Call::new("curl", &self.program, &args))
            .map_err(|e| match e {
                ToolError::Failed { reason, .. } if reason.starts_with("cannot start") => {
                    FetchError::Fatal(reason)
                }
                other => FetchError::Transient(format!("{url}: curl: {other}")),
            })?;
        if out.exit_code == 0 {
            return Ok(out.stdout);
        }
        let why = format!("{url}: curl exit {}: {}", out.exit_code, out.stderr.trim());
        Err(match out.exit_code {
            // 1 unsupported or unparsable reply, 8 weird reply, 18 partial file, 61 bad encoding.
            1 | 8 | 18 | 61 => FetchError::Malformed(why),
            // 3 bad URL, 37 a file:// that cannot be read.
            3 | 37 => FetchError::Fatal(why),
            _ => FetchError::Transient(why),
        })
    }
}

/// Fetch `url`, trying again up to [`RETRIES`] times after a transient failure, waiting `delay` between tries.
///
/// # Errors
/// The last error. A malformed or fatal error ends the attempts at once.
pub fn get_with_retries(
    fetcher: &dyn Fetcher,
    url: &str,
    delay: Duration,
) -> Result<Vec<u8>, FetchError> {
    let mut tries = 0;
    loop {
        match fetcher.get(url) {
            Err(FetchError::Transient(_)) if tries < RETRIES => {
                tries += 1;
                std::thread::sleep(delay);
            }
            other => return other,
        }
    }
}

/// Why a pinned file was not obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedError {
    /// The lock or the trust rules forbid it; nothing was changed.
    Refused(String),
    /// A checksum or a response did not check out; nothing was stored.
    Verification(String),
    /// The fetch or the write failed; nothing was stored.
    Failed(String),
}

impl std::fmt::Display for PinnedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (Self::Refused(why) | Self::Verification(why) | Self::Failed(why)) = self;
        f.write_str(why)
    }
}

impl std::error::Error for PinnedError {}

/// What kind of entry it is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Fetched, with a checksum and an attestation identity.
    #[default]
    Download,
    /// Fetched, built by someone else: the checksum may be left to a confirmation.
    ThirdParty,
    /// Linked into a program, never fetched.
    Linked,
}

/// One downloadable file, for one platform.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    /// The only URL the file is fetched from.
    pub url: String,
    /// The SHA-256 of the file, 64 lower-case hex digits.
    #[serde(default)]
    pub sha256: Option<String>,
}

/// The identity of a build attestation an entry pins.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AttestationPin {
    /// The repository, `owner/name`.
    pub repository: String,
    /// The workflow file that built it.
    #[serde(default)]
    pub workflow: Option<String>,
}

/// One entry of the lock. Fields this build has no use for are not read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entry {
    /// The file's name.
    pub name: String,
    /// The pinned version.
    pub version: String,
    /// What kind of entry it is.
    #[serde(default)]
    pub kind: Kind,
    /// The attestation identity, which this build cannot check.
    #[serde(default)]
    attestation: Option<AttestationPin>,
    /// The files, by platform (`linux-x86_64`, ..., or `any`).
    #[serde(default)]
    pub assets: BTreeMap<String, Asset>,
}

/// The lock file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Lock {
    /// The layout version of the file.
    pub schema_version: u32,
    /// The entries.
    #[serde(default, rename = "tool")]
    pub tools: Vec<Entry>,
}

/// The platform key of this machine, such as `linux-x86_64`.
#[must_use]
pub fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn is_sha256(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Only `https`, or `http` to this machine (for a test's release server), may be a URL of the lock.
fn url_allowed(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://127.0.0.1:")
        || url.starts_with("http://127.0.0.1/")
        || url.starts_with("http://localhost:")
}

impl Lock {
    /// Read a lock file, strictly: a downloaded file needs its checksum, only `https` URLs, and a plain name.
    ///
    /// # Errors
    /// A reason, when the text is not a lock of schema 1 or breaks one of those rules.
    pub fn parse(text: &str) -> Result<Lock, String> {
        let lock: Lock = toml::from_str(text).map_err(|e| format!("tools.lock.toml: {e}"))?;
        if lock.schema_version != 1 {
            return Err(format!(
                "tools.lock.toml: schema {} is not known to this raul",
                lock.schema_version
            ));
        }
        let mut seen = Vec::new();
        for tool in &lock.tools {
            if seen.contains(&&tool.name) {
                return Err(format!("tools.lock.toml: {} is listed twice", tool.name));
            }
            seen.push(&tool.name);
            tool.check()?;
        }
        Ok(lock)
    }

    /// The lock that ships with the Toolbox.
    ///
    /// # Panics
    /// Never: a test parses the compiled-in file.
    #[must_use]
    pub fn builtin() -> Lock {
        Lock::parse(BUILTIN_LOCK).expect("the shipped lock file is valid")
    }

    /// The entry called `name`.
    #[must_use]
    pub fn tool(&self, name: &str) -> Option<&Entry> {
        self.tools.iter().find(|t| t.name == name)
    }
}

impl Entry {
    fn check(&self) -> Result<(), String> {
        let bad = |why: &str| Err(format!("tools.lock.toml: {}: {why}", self.name));
        if self.name.is_empty() || self.name.contains(['/', '\\']) || self.name.starts_with('.') {
            return bad("the name is not a plain file name");
        }
        if self.version.is_empty() {
            return bad("no version");
        }
        if self.kind == Kind::Linked {
            return Ok(());
        }
        if self.assets.is_empty() {
            return bad("no assets");
        }
        for (platform, asset) in &self.assets {
            if !url_allowed(&asset.url) {
                return bad(&format!("the {platform} URL must be https"));
            }
            match (&asset.sha256, self.kind) {
                (Some(hash), _) if !is_sha256(hash) => {
                    return bad(&format!(
                        "the {platform} sha256 is not 64 lower-case hex digits"
                    ));
                }
                (None, Kind::Download) => {
                    return bad(&format!(
                        "the {platform} asset has no sha256 (only a third-party file may leave it to a confirmation)"
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// The file of this machine's platform, or the one under the key `any` (a data file such as a masterlist).
    #[must_use]
    pub fn asset_here(&self) -> Option<&Asset> {
        self.assets
            .get(&platform())
            .or_else(|| self.assets.get("any"))
    }
}

/// The confirmations of one Instance: the checksums an owner confirmed for third-party files whose entry pins none.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Trust {
    #[serde(default = "one")]
    schema_version: u32,
    #[serde(default)]
    confirmed: Vec<Confirmed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Confirmed {
    tool: String,
    version: String,
    sha256: String,
}

fn one() -> u32 {
    1
}

impl Trust {
    /// Read the confirmations of the Instance at `root`; none when the file does not exist.
    ///
    /// # Errors
    /// A reason, when the file exists but cannot be read or is not valid.
    pub fn load(root: &Path) -> Result<Trust, String> {
        let path = root.join(TRUST_FILE);
        match std::fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Trust::default()),
            Err(e) => Err(format!("{}: {e}", path.display())),
            Ok(text) => {
                let trust: Trust =
                    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
                if trust.schema_version == 1 {
                    Ok(trust)
                } else {
                    Err(format!(
                        "{}: schema {} is not known to this raul",
                        path.display(),
                        trust.schema_version
                    ))
                }
            }
        }
    }

    /// The checksum confirmed for `tool` at `version`.
    #[must_use]
    pub fn find(&self, tool: &str, version: &str) -> Option<&str> {
        self.confirmed
            .iter()
            .find(|c| c.tool == tool && c.version == version)
            .map(|c| c.sha256.as_str())
    }
}

/// A folder of files stored by content hash.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// A store in `root` (made when first written to).
    #[must_use]
    pub fn new(root: PathBuf) -> Store {
        Store { root }
    }

    /// The default place: `NVMOD_TOOLS_DIR`, else the user's data folder under `nvmod/tools`.
    #[must_use]
    pub fn default_location() -> PathBuf {
        let var = |name: &str| {
            std::env::var_os(name)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        if let Some(dir) = var("NVMOD_TOOLS_DIR") {
            return dir;
        }
        let data = var("XDG_DATA_HOME")
            .or_else(|| var("LOCALAPPDATA"))
            .or_else(|| var("HOME").map(|h| h.join(".local").join("share")))
            .unwrap_or_else(|| PathBuf::from("."));
        data.join("nvmod").join("tools")
    }

    /// Where the file `name` with content hash `sha256` is (or would be) stored.
    #[must_use]
    pub fn path(&self, sha256: &str, name: &str) -> PathBuf {
        self.root
            .join(sha256.to_ascii_lowercase())
            .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
    }

    /// Whether the stored file is there and hashes to what it is stored under.
    fn verified(&self, sha256: &str, name: &str) -> bool {
        std::fs::read(self.path(sha256, name))
            .is_ok_and(|bytes| sha256_hex(&bytes).eq_ignore_ascii_case(sha256))
    }

    fn put(&self, sha256: &str, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        let target = self.path(sha256, name);
        let io = |e: std::io::Error| format!("{}: {e}", target.display());
        std::fs::create_dir_all(target.parent().unwrap_or(&self.root)).map_err(io)?;
        let temp = self.root.join(format!(
            ".fetching-{}-{}",
            std::process::id(),
            &sha256[..16.min(sha256.len())]
        ));
        std::fs::write(&temp, bytes).map_err(io)?;
        std::fs::rename(&temp, &target).map_err(|e| {
            let _ = std::fs::remove_file(&temp);
            io(e)
        })?;
        Ok(target)
    }
}

/// Where a file's checksum comes from: the lock, or the Instance's confirmation.
fn expected_sha256<'a>(
    entry: &'a Entry,
    asset: &'a Asset,
    trust: Option<&'a Trust>,
) -> Option<(&'a str, &'static str)> {
    if let Some(hash) = &asset.sha256 {
        return Some((hash, "the lock file"));
    }
    trust
        .and_then(|t| t.find(&entry.name, &entry.version))
        .map(|hash| (hash, "the owner's confirmation"))
}

/// What [`fetch`] needs besides the entry.
pub struct Fetch<'a> {
    /// Gets URLs.
    pub fetcher: &'a dyn Fetcher,
    /// The wait between retries.
    pub retry_delay: Duration,
    /// Where files are kept.
    pub store: &'a Store,
    /// The Instance's confirmations, if an Instance was named.
    pub trust: Option<&'a Trust>,
}

impl Fetch<'_> {
    /// The file of `entry` for this machine, verified and in the store.
    ///
    /// # Errors
    /// `Refused` for a linked entry, one with a pinned attestation (this build checks checksums only), a platform
    /// with no file, or a file with neither a pinned nor a confirmed checksum. `Verification` for a malformed
    /// response or a checksum mismatch: nothing is stored. `Failed` when the fetch (after its retries) or the write
    /// fails.
    pub fn file(&self, entry: &Entry) -> Result<PathBuf, PinnedError> {
        if entry.kind == Kind::Linked {
            return Err(PinnedError::Refused(format!(
                "{} is linked into a program, not fetched",
                entry.name
            )));
        }
        if entry.attestation.is_some() {
            return Err(PinnedError::Refused(format!(
                "{} pins a build attestation, which this raul does not check: it only fetches files pinned by checksum",
                entry.name
            )));
        }
        let platform = platform();
        let asset = entry.asset_here().ok_or_else(|| {
            PinnedError::Refused(format!(
                "{} {} pins no file for {platform}",
                entry.name, entry.version
            ))
        })?;
        let Some((expected, source)) = expected_sha256(entry, asset, self.trust) else {
            return Err(PinnedError::Refused(format!(
                "{} {} has no pinned checksum and no confirmation in the Instance: check the download yourself first",
                entry.name, entry.version
            )));
        };
        if self.store.verified(expected, &entry.name) {
            return Ok(self.store.path(expected, &entry.name));
        }
        let bytes =
            get_with_retries(self.fetcher, &asset.url, self.retry_delay).map_err(|e| match e {
                FetchError::Malformed(why) => {
                    PinnedError::Verification(format!("malformed response: {why}"))
                }
                FetchError::Transient(why) | FetchError::Fatal(why) => {
                    PinnedError::Failed(format!("fetch failed: {why}"))
                }
            })?;
        let actual = sha256_hex(&bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(PinnedError::Verification(format!(
                "checksum mismatch for {}: {source} says {expected}, the download is {actual}",
                entry.name
            )));
        }
        self.store
            .put(expected, &entry.name, &bytes)
            .map_err(PinnedError::Failed)
    }
}

/// Where the masterlist comes from, for [`masterlist`].
#[derive(Debug, Clone, Default)]
pub struct MasterlistPin {
    /// A lock file of the person's own, instead of the one that ships.
    pub lock: Option<PathBuf>,
    /// A store folder of the person's own, instead of the default.
    pub store: Option<PathBuf>,
    /// The `curl` program, instead of the one on `PATH`.
    pub curl: Option<PathBuf>,
    /// The wait between retries, instead of one second.
    pub retry_delay: Option<Duration>,
}

/// The pinned LOOT masterlist of Fallout: New Vegas as a file in the store, fetched and checked first when it is not
/// there yet. `instance` is the Instance folder, whose confirmations count.
///
/// # Errors
/// See [`PinnedError`]: a missing pin is a refusal, a bad checksum a verification failure, a failed fetch a failure.
pub fn masterlist(instance: &Path, pin: &MasterlistPin) -> Result<PathBuf, PinnedError> {
    masterlist_with(
        instance,
        pin,
        &CurlFetcher {
            program: pin.curl.clone().unwrap_or_else(|| PathBuf::from("curl")),
        },
    )
}

/// [`masterlist`] with `fetcher` instead of `curl`.
///
/// # Errors
/// As [`masterlist`].
pub fn masterlist_with(
    instance: &Path,
    pin: &MasterlistPin,
    fetcher: &dyn Fetcher,
) -> Result<PathBuf, PinnedError> {
    let refused = PinnedError::Refused;
    let lock = match &pin.lock {
        None => Lock::builtin(),
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| refused(format!("{}: {e}", path.display())))?;
            Lock::parse(&text).map_err(refused)?
        }
    };
    let entry = lock.tool(MASTERLIST_TOOL).ok_or_else(|| {
        refused(format!(
            "the lock file pins no file called `{MASTERLIST_TOOL}`, so no masterlist source is pinned: \
             give a masterlist file instead"
        ))
    })?;
    let store = Store::new(pin.store.clone().unwrap_or_else(Store::default_location));
    let trust = Trust::load(instance).map_err(PinnedError::Refused)?;
    Fetch {
        fetcher,
        retry_delay: pin.retry_delay.unwrap_or(Duration::from_millis(1000)),
        store: &store,
        trust: Some(&trust),
    }
    .file(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;
    use std::cell::Cell;

    const BODY: &[u8] = b"abc";
    const HASH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn lock_text(extra_entry: &str, sha: &str) -> String {
        format!(
            "schema_version = 1\n[[tool]]\nname = \"{MASTERLIST_TOOL}\"\nversion = \"v1\"\nkind = \"third-party\"\n{extra_entry}\n[tool.assets.any]\nurl = \"https://example.test/masterlist.yaml\"\n{sha}\n"
        )
    }

    struct Fixed(Vec<Result<Vec<u8>, FetchError>>, Cell<usize>);

    impl Fixed {
        fn new(answers: Vec<Result<Vec<u8>, FetchError>>) -> Self {
            Fixed(answers, Cell::new(0))
        }
    }

    impl Fetcher for Fixed {
        fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
            let at = self.1.get();
            self.1.set(at + 1);
            match &self.0[at.min(self.0.len() - 1)] {
                Ok(bytes) => Ok(bytes.clone()),
                Err(FetchError::Malformed(w)) => Err(FetchError::Malformed(w.clone())),
                Err(FetchError::Transient(w)) => Err(FetchError::Transient(w.clone())),
                Err(FetchError::Fatal(w)) => Err(FetchError::Fatal(w.clone())),
            }
        }
    }

    struct Rig {
        dir: TempDir,
    }

    impl Rig {
        fn new(lock: &str) -> Rig {
            let dir = TempDir::new("raul-pinned");
            std::fs::write(dir.path().join("lock.toml"), lock).unwrap();
            Rig { dir }
        }

        fn pin(&self) -> MasterlistPin {
            MasterlistPin {
                lock: Some(self.dir.path().join("lock.toml")),
                store: Some(self.dir.path().join("store")),
                curl: None,
                retry_delay: Some(Duration::ZERO),
            }
        }

        fn get(&self, fetcher: &dyn Fetcher) -> Result<PathBuf, PinnedError> {
            masterlist_with(self.dir.path(), &self.pin(), fetcher)
        }
    }

    #[test]
    fn the_shipped_lock_pins_the_masterlist_by_checksum() {
        let lock = Lock::builtin();
        let entry = lock
            .tool(MASTERLIST_TOOL)
            .expect("the masterlist is pinned");
        let asset = entry.asset_here().unwrap();
        assert!(asset.url.starts_with("https://") && asset.sha256.is_some());
        assert!(entry.attestation.is_none());
    }

    #[test]
    fn the_shipped_pin_equals_the_one_nvmod_rs_keeps_while_it_keeps_one() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../components/nvmod-rs/tools.lock.toml");
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let Some(theirs) = Lock::parse(&text)
            .ok()
            .and_then(|l| l.tool(MASTERLIST_TOOL).cloned())
        else {
            return;
        };
        assert_eq!(Lock::builtin().tool(MASTERLIST_TOOL), Some(&theirs));
    }

    #[test]
    fn a_pinned_file_is_fetched_checked_and_kept_under_its_hash() {
        let rig = Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\"")));
        let fetcher = Fixed::new(vec![Ok(BODY.to_vec())]);
        let path = rig.get(&fetcher).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), BODY);
        assert!(path.to_string_lossy().contains(HASH));
        // Found again without a request.
        let again = rig.get(&Fixed::new(vec![Err(FetchError::Fatal("offline".into()))]));
        assert_eq!(again.unwrap(), path);
        assert_eq!(fetcher.1.get(), 1);
    }

    #[test]
    fn a_stored_file_that_no_longer_matches_its_hash_is_fetched_again() {
        let rig = Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\"")));
        let path = rig.get(&Fixed::new(vec![Ok(BODY.to_vec())])).unwrap();
        std::fs::write(&path, b"tampered").unwrap();
        let fetcher = Fixed::new(vec![Ok(BODY.to_vec())]);
        let path = rig.get(&fetcher).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), BODY);
        assert_eq!(fetcher.1.get(), 1);
    }

    #[test]
    fn a_wrong_checksum_stores_nothing_and_says_both_hashes() {
        let rig = Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\"")));
        let why = rig.get(&Fixed::new(vec![Ok(b"abd".to_vec())])).unwrap_err();
        let PinnedError::Verification(text) = &why else {
            panic!("{why:?}");
        };
        assert!(
            text.contains(HASH) && text.contains("the lock file says"),
            "{text}"
        );
        assert!(!rig.dir.path().join("store").join(HASH).exists());
    }

    #[test]
    fn a_transient_failure_is_retried_and_a_malformed_or_fatal_one_is_not() {
        let rig = Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\"")));
        let flaky = Fixed::new(vec![
            Err(FetchError::Transient("busy".into())),
            Err(FetchError::Transient("busy".into())),
            Ok(BODY.to_vec()),
        ]);
        assert!(rig.get(&flaky).is_ok());
        assert_eq!(flaky.1.get(), 3);
        let down = Fixed::new(vec![Err(FetchError::Transient("down".into()))]);
        let Err(PinnedError::Failed(why)) =
            Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\""))).get(&down)
        else {
            panic!("a failed fetch is a failure");
        };
        assert!(why.contains("down"));
        assert_eq!(down.1.get(), 1 + RETRIES as usize);
        for (answer, verification) in [
            (FetchError::Malformed("cut".into()), true),
            (FetchError::Fatal("no curl".into()), false),
        ] {
            let once = Fixed::new(vec![Err(answer)]);
            let got = Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\"")))
                .get(&once)
                .unwrap_err();
            assert_eq!(matches!(got, PinnedError::Verification(_)), verification);
            assert_eq!(once.1.get(), 1);
        }
    }

    #[test]
    fn a_file_without_a_pinned_checksum_needs_the_instance_to_confirm_that_version() {
        let rig = Rig::new(&lock_text("", ""));
        let fetcher = Fixed::new(vec![Ok(BODY.to_vec())]);
        let Err(PinnedError::Refused(why)) = rig.get(&fetcher) else {
            panic!("refused without a confirmation");
        };
        assert!(why.contains("no pinned checksum"), "{why}");
        assert_eq!(fetcher.1.get(), 0, "nothing was requested");
        let confirm = |version: &str| {
            std::fs::write(
                rig.dir.path().join(TRUST_FILE),
                format!("[[confirmed]]\ntool = \"{MASTERLIST_TOOL}\"\nversion = \"{version}\"\nsha256 = \"{HASH}\"\n"),
            )
            .unwrap();
        };
        confirm("v0");
        assert!(matches!(rig.get(&fetcher), Err(PinnedError::Refused(_))));
        confirm("v1");
        let path = rig.get(&fetcher).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), BODY);
    }

    #[test]
    fn a_broken_or_future_trust_file_is_refused() {
        let rig = Rig::new(&lock_text("", &format!("sha256 = \"{HASH}\"")));
        for text in ["confirmed = 5", "schema_version = 7\n"] {
            std::fs::write(rig.dir.path().join(TRUST_FILE), text).unwrap();
            let got = rig.get(&Fixed::new(vec![Ok(BODY.to_vec())]));
            assert!(matches!(got, Err(PinnedError::Refused(_))), "{text}");
        }
    }

    #[test]
    fn an_entry_that_pins_an_attestation_or_is_linked_is_refused() {
        let rig = Rig::new(&lock_text(
            "attestation = { repository = \"o/r\" }",
            &format!("sha256 = \"{HASH}\""),
        ));
        let Err(PinnedError::Refused(why)) = rig.get(&Fixed::new(vec![Ok(BODY.to_vec())])) else {
            panic!("refused");
        };
        assert!(why.contains("attestation"), "{why}");
        let linked = Rig::new(&format!(
            "schema_version = 1\n[[tool]]\nname = \"{MASTERLIST_TOOL}\"\nversion = \"1\"\nkind = \"linked\"\n"
        ));
        assert!(matches!(
            linked.get(&Fixed::new(vec![Ok(BODY.to_vec())])),
            Err(PinnedError::Refused(_))
        ));
    }

    #[test]
    fn a_lock_without_the_masterlist_or_unreadable_or_invalid_is_refused() {
        let other = Rig::new(
            "schema_version = 1\n[[tool]]\nname = \"other\"\nversion = \"1\"\nkind = \"third-party\"\n[tool.assets.any]\nurl = \"https://e.test/x\"\n",
        );
        let Err(PinnedError::Refused(why)) = other.get(&Fixed::new(vec![Ok(BODY.to_vec())])) else {
            panic!("refused");
        };
        assert!(why.contains(MASTERLIST_TOOL), "{why}");
        let mut pin = other.pin();
        pin.lock = Some(other.dir.path().join("missing.toml"));
        assert!(matches!(
            masterlist_with(other.dir.path(), &pin, &Fixed::new(vec![Ok(Vec::new())])),
            Err(PinnedError::Refused(_))
        ));
    }

    #[test]
    fn the_lock_is_read_strictly() {
        let sha = format!("sha256 = \"{HASH}\"");
        for (text, word) in [
            ("schema_version = 2\n".to_owned(), "schema 2"),
            (lock_text("", "sha256 = \"ABC\""), "64 lower-case"),
            (
                lock_text("", &sha).replace("https://example.test", "http://example.test"),
                "must be https",
            ),
            (
                lock_text("", &sha).replace(MASTERLIST_TOOL, "../x"),
                "plain file name",
            ),
            (lock_text("", &sha).replace("v1", ""), "no version"),
            (
                lock_text("", "").replace("third-party", "download"),
                "has no sha256",
            ),
            (
                format!(
                    "{}{}",
                    lock_text("", &sha),
                    lock_text("", &sha).replace("schema_version = 1\n", "")
                ),
                "listed twice",
            ),
        ] {
            let why = Lock::parse(&text).unwrap_err();
            assert!(why.contains(word), "{word}: {why}");
        }
    }

    #[test]
    fn the_curl_command_line_has_the_timeout_and_ends_with_the_url() {
        let args = CurlFetcher::args("http://h/x").join(" ");
        assert!(args.contains("--max-time 60"), "{args}");
        assert!(args.ends_with("http://h/x"));
    }

    #[test]
    fn a_file_url_is_read_by_curl_and_a_missing_one_or_a_missing_curl_is_fatal() {
        let t = TempDir::new("raul-curl-file");
        let file = t.path().join("body");
        std::fs::write(&file, b"hello").unwrap();
        let url = format!("file://{}", file.display());
        assert_eq!(CurlFetcher::default().get(&url), Ok(b"hello".to_vec()));
        let nope = format!("file://{}", t.path().join("nope").display());
        assert!(matches!(
            CurlFetcher::default().get(&nope),
            Err(FetchError::Fatal(_))
        ));
        let no_curl = CurlFetcher {
            program: t.path().join("no-such-curl"),
        };
        assert!(matches!(no_curl.get(&url), Err(FetchError::Fatal(_))));
    }

    #[cfg(unix)]
    #[test]
    fn the_exit_code_of_curl_decides_whether_the_failure_is_retried() {
        let t = TempDir::new("raul-curl-exit");
        for (code, malformed, fatal) in [(18, true, false), (37, false, true), (28, false, false)] {
            let curl = nvmod_testkit::write_script(
                t.path(),
                &format!("curl-{code}"),
                &format!("echo no >&2\nexit {code}"),
            );
            match (CurlFetcher { program: curl }).get("https://h/x") {
                Err(FetchError::Malformed(_)) => assert!(malformed, "{code}"),
                Err(FetchError::Fatal(_)) => assert!(fatal, "{code}"),
                Err(FetchError::Transient(_)) => assert!(!malformed && !fatal, "{code}"),
                other => panic!("{code}: {other:?}"),
            }
        }
    }
}
