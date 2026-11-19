// SPDX-License-Identifier: GPL-3.0-only
//! Checking a release before anything of it is installed (ADR-0020): the release is pinned by tag and id, the asset
//! must match its line in the release's `SHA256SUMS`, and the sums must not change between two fetches. Releases are
//! not signed, so this catches damage and mix-ups, not someone who controls the release (see the ADR). Everything
//! outside is behind [`Fetch`], so tests use synthetic releases and no network.

use std::fmt::{self, Write as _};
use std::io::Read;
use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// The repository whose releases are read.
pub const REPO: &str = "JorisJonkers-dev/rauls-trusty-toolbox";
/// The name of the checksum list asset.
pub const SUMS: &str = "SHA256SUMS";
/// The most bytes accepted for the checksum list.
pub const MAX_SUMS_BYTES: u64 = 1024 * 1024;
/// The most bytes accepted for an asset.
pub const MAX_ASSET_BYTES: u64 = 500 * 1024 * 1024;

/// Why an update was refused. Each text is plain; none holds a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The release is a draft or a pre-release, or its tag is not the version it offers.
    NotThatRelease(String),
    /// The version offered is not newer than the running one and the person did not ask for it.
    NotNewer {
        /// The version offered.
        offered: String,
        /// The version running.
        running: String,
    },
    /// The release has no such file.
    Missing(String),
    /// A file is not at the address that belongs to this release.
    WrongAddress(String),
    /// The checksum list could not be read.
    BadSums(String),
    /// The file's checksum is not the one in the list (damaged or incomplete download).
    Mismatch(String),
    /// The checksum list changed while the update was fetched.
    SumsChanged,
    /// A file is bigger than the limit.
    TooBig(String),
    /// Fetching or writing failed; the reason.
    Fetch(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotThatRelease(why) => write!(f, "This release cannot be installed: {why}."),
            Self::NotNewer { offered, running } => write!(
                f,
                "Version {offered} is not newer than the version you have ({running}), so it was not installed."
            ),
            Self::Missing(name) => write!(f, "The release has no file {name}."),
            Self::WrongAddress(name) => {
                write!(
                    f,
                    "{name} is not at the address that belongs to this release."
                )
            }
            Self::BadSums(why) => write!(f, "The checksum list is not readable: {why}."),
            Self::Mismatch(name) => write!(
                f,
                "{name} does not match its checksum; the download may be damaged. Try again."
            ),
            Self::SumsChanged => {
                f.write_str("The checksum list changed while it was being fetched. Try again.")
            }
            Self::TooBig(name) => write!(f, "{name} is bigger than an update may be."),
            Self::Fetch(why) => write!(f, "The download failed: {why}"),
        }
    }
}

impl std::error::Error for Error {}

/// One file of a release as the release page lists it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    /// The file name.
    pub name: String,
    /// The address it is downloaded from.
    pub browser_download_url: String,
}

/// A release as the release page describes it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Release {
    /// The release's id as the release page gives it. The release is pinned by its tag and by the tag's own download
    /// addresses ([`Release::address_of`]); the id is kept for the person's details and is not what is trusted.
    pub id: u64,
    /// The tag, `v<version>`.
    pub tag_name: String,
    /// A draft is never installed.
    #[serde(default)]
    pub draft: bool,
    /// A pre-release is never installed.
    #[serde(default)]
    pub prerelease: bool,
    /// The files of the release.
    #[serde(default)]
    pub assets: Vec<Asset>,
}

impl Release {
    /// The version the tag names (without the `v`).
    ///
    /// # Errors
    /// [`Error::NotThatRelease`] when the tag is not `v` and a semantic version.
    pub fn version(&self) -> Result<Version, Error> {
        self.tag_name
            .strip_prefix('v')
            .and_then(|v| Version::parse(v).ok())
            .ok_or_else(|| Error::NotThatRelease("its name is not a version".to_owned()))
    }

    /// The address a file of this release must have: GitHub's download address for this repository and tag.
    #[must_use]
    pub fn address_of(&self, name: &str) -> String {
        format!(
            "https://github.com/{REPO}/releases/download/{}/{name}",
            self.tag_name
        )
    }

    fn asset(&self, name: &str) -> Result<&Asset, Error> {
        let asset = self
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| Error::Missing(name.to_owned()))?;
        if asset.browser_download_url == self.address_of(name) {
            Ok(asset)
        } else {
            Err(Error::WrongAddress(name.to_owned()))
        }
    }
}

/// Decide whether `release` may be installed over `running`.
///
/// # Errors
/// [`Error::NotThatRelease`] for a draft, a pre-release or a tag that is not a version;
/// [`Error::NotNewer`] for a version that is not higher than `running`, unless `allow_older` (the person asked for it
/// by name).
pub fn accept(release: &Release, running: &str, allow_older: bool) -> Result<Version, Error> {
    if release.draft {
        return Err(Error::NotThatRelease("it is a draft".to_owned()));
    }
    if release.prerelease {
        return Err(Error::NotThatRelease("it is a pre-release".to_owned()));
    }
    let offered = release.version()?;
    let current = Version::parse(running)
        .map_err(|_| Error::NotThatRelease("the running version is not a version".to_owned()))?;
    if offered <= current && !allow_older {
        return Err(Error::NotNewer {
            offered: offered.to_string(),
            running: running.to_owned(),
        });
    }
    Ok(offered)
}

/// The lines of a `SHA256SUMS` file: file name and lower-case hash.
///
/// # Errors
/// [`Error::BadSums`] for a line that is not `<64 hex digits>  <name>` (a `*` before the name is allowed) or a name
/// listed twice.
pub fn parse_sums(text: &str) -> Result<Vec<(String, String)>, Error> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (hash, rest) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| Error::BadSums("a line has no file name".to_owned()))?;
        let name = rest.trim_start().trim_start_matches('*');
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) || name.is_empty() {
            return Err(Error::BadSums("a line is malformed".to_owned()));
        }
        if out.iter().any(|(n, _)| n == name) {
            return Err(Error::BadSums("a file is listed twice".to_owned()));
        }
        out.push((name.to_owned(), hash.to_ascii_lowercase()));
    }
    Ok(out)
}

/// The SHA-256 of the file at `path`, as lower-case hex, read in blocks.
///
/// # Errors
/// [`Error::Fetch`] when the file cannot be read.
pub fn hash_file(path: &Path) -> Result<String, Error> {
    let io = |e: std::io::Error| Error::Fetch(format!("a file could not be read ({})", e.kind()));
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut block = vec![0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut block).map_err(io)?;
        if n == 0 {
            break;
        }
        hasher.update(&block[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut hex, b| {
            let _ = write!(hex, "{b:02x}");
            hex
        }))
}

/// Where fetched bytes come from. The one real implementation reads over HTTPS; tests use a fake.
pub trait Fetch {
    /// Download `url` to the file `to`, writing at most `limit` bytes.
    ///
    /// # Errors
    /// [`Error::TooBig`] past the limit, [`Error::Fetch`] otherwise.
    fn get(&self, url: &str, to: &Path, limit: u64) -> Result<(), Error>;
}

/// What a verified download is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Downloaded {
    /// The asset's file in the work folder.
    pub file: PathBuf,
    /// Its SHA-256.
    pub sha256: String,
    /// The version it is.
    pub version: Version,
}

/// Download `asset` of `release` into `work`, checking it against the release's `SHA256SUMS`: the release must be
/// acceptable over `running`, both files must be at this release's own addresses, the asset's hash must be its line
/// in the list, and the list must be the same when fetched again after the asset.
///
/// # Errors
/// Any [`Error`]; nothing is left in `work` that looks like a verified download when one is returned (the file is removed).
pub fn download(
    fetch: &dyn Fetch,
    release: &Release,
    running: &str,
    allow_older: bool,
    asset: &str,
    work: &Path,
) -> Result<Downloaded, Error> {
    // The name is joined onto the work folder below: it must be one plain file name.
    let bare = !asset.is_empty()
        && asset != "."
        && asset != ".."
        && !asset.contains(['/', '\\', ':', '\0']);
    if !bare {
        return Err(Error::WrongAddress(asset.to_owned()));
    }
    let version = accept(release, running, allow_older)?;
    let sums_asset = release.asset(SUMS)?;
    let wanted = release.asset(asset)?;
    let io = |_: &Path, e: &std::io::Error| {
        Error::Fetch(format!("a file could not be written ({})", e.kind()))
    };
    std::fs::create_dir_all(work).map_err(|e| io(work, &e))?;
    let first = work.join("SHA256SUMS.1");
    let second = work.join("SHA256SUMS.2");
    let file = work.join(asset);
    let run = || -> Result<String, Error> {
        fetch.get(&sums_asset.browser_download_url, &first, MAX_SUMS_BYTES)?;
        let sums = parse_sums(&std::fs::read_to_string(&first).map_err(|e| io(&first, &e))?)?;
        let expected = sums
            .iter()
            .find(|(n, _)| n == asset)
            .map(|(_, h)| h.clone())
            .ok_or_else(|| Error::BadSums(format!("{asset} is not in the list")))?;
        fetch.get(&wanted.browser_download_url, &file, MAX_ASSET_BYTES)?;
        let actual = hash_file(&file)?;
        if actual != expected {
            return Err(Error::Mismatch(asset.to_owned()));
        }
        fetch.get(&sums_asset.browser_download_url, &second, MAX_SUMS_BYTES)?;
        let a = std::fs::read(&first).map_err(|e| io(&first, &e))?;
        let b = std::fs::read(&second).map_err(|e| io(&second, &e))?;
        if a != b {
            return Err(Error::SumsChanged);
        }
        Ok(actual)
    };
    let result = run();
    let _ = std::fs::remove_file(&first);
    let _ = std::fs::remove_file(&second);
    match result {
        Ok(sha256) => Ok(Downloaded {
            file,
            sha256,
            version,
        }),
        Err(e) => {
            let _ = std::fs::remove_file(&file);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// A release whose files are held in memory; `get` counts how often each address was asked for.
    struct Fake {
        files: RefCell<HashMap<String, Vec<u8>>>,
        asked: RefCell<Vec<String>>,
        /// Replace the sums with these bytes from the second request on.
        second_sums: Option<Vec<u8>>,
    }

    impl Fetch for Fake {
        fn get(&self, url: &str, to: &Path, limit: u64) -> Result<(), Error> {
            let nth = self.asked.borrow().iter().filter(|u| *u == url).count();
            self.asked.borrow_mut().push(url.to_owned());
            let mut bytes = self
                .files
                .borrow()
                .get(url)
                .cloned()
                .ok_or_else(|| Error::Fetch("404".to_owned()))?;
            if nth >= 1
                && url.ends_with("/SHA256SUMS")
                && let Some(other) = &self.second_sums
            {
                bytes.clone_from(other);
            }
            if bytes.len() as u64 > limit {
                return Err(Error::TooBig(url.to_owned()));
            }
            std::fs::write(to, bytes).map_err(|e| Error::Fetch(e.to_string()))
        }
    }

    const ASSET: &str = "raul-2.0.0-linux-x64.tar.gz";

    fn release(tag: &str) -> Release {
        Release {
            id: 7,
            tag_name: tag.to_owned(),
            draft: false,
            prerelease: false,
            assets: [SUMS, ASSET]
                .iter()
                .map(|n| Asset {
                    name: (*n).to_owned(),
                    browser_download_url: format!(
                        "https://github.com/{REPO}/releases/download/{tag}/{n}"
                    ),
                })
                .collect(),
        }
    }

    fn fake(release: &Release, body: &[u8], sums_hash: &str) -> Fake {
        let mut files = HashMap::new();
        files.insert(release.address_of(ASSET), body.to_vec());
        files.insert(
            release.address_of(SUMS),
            format!("{sums_hash}  {ASSET}\n{}  other\n", "0".repeat(64)).into_bytes(),
        );
        Fake {
            files: RefCell::new(files),
            asked: RefCell::new(Vec::new()),
            second_sums: None,
        }
    }

    fn hash(bytes: &[u8]) -> String {
        raul_runner::sha256_hex(bytes)
    }

    #[test]
    fn a_matching_asset_of_the_pinned_release_downloads_and_verifies() {
        let t = TempDir::new("verify-ok");
        let r = release("v2.0.0");
        let f = fake(&r, b"payload", &hash(b"payload"));
        let d = download(&f, &r, "1.0.0", false, ASSET, t.path()).expect("verified");
        assert_eq!(d.version, Version::new(2, 0, 0));
        assert_eq!(d.sha256, hash(b"payload"));
        assert_eq!(std::fs::read(&d.file).unwrap(), b"payload");
        // The list was fetched twice, the asset once; no scratch files are left.
        assert_eq!(
            f.asked
                .borrow()
                .iter()
                .filter(|u| u.ends_with("/SHA256SUMS"))
                .count(),
            2
        );
        assert!(!t.path().join("SHA256SUMS.1").exists());
    }

    #[test]
    fn a_damaged_download_is_refused_and_removed() {
        let t = TempDir::new("verify-damaged");
        let r = release("v2.0.0");
        let f = fake(&r, b"payload", &hash(b"another"));
        let e = download(&f, &r, "1.0.0", false, ASSET, t.path()).unwrap_err();
        assert_eq!(e, Error::Mismatch(ASSET.to_owned()));
        assert!(!t.path().join(ASSET).exists());
    }

    #[test]
    fn sums_that_change_between_the_two_fetches_are_refused() {
        let t = TempDir::new("verify-changed");
        let r = release("v2.0.0");
        let mut f = fake(&r, b"payload", &hash(b"payload"));
        f.second_sums = Some(format!("{}  {ASSET}\n", hash(b"payload")).into_bytes());
        let e = download(&f, &r, "1.0.0", false, ASSET, t.path()).unwrap_err();
        assert_eq!(e, Error::SumsChanged);
        assert!(!t.path().join(ASSET).exists());
    }

    #[test]
    fn a_draft_a_prerelease_and_a_tag_that_is_no_version_are_refused() {
        let t = TempDir::new("verify-kinds");
        let mut r = release("v2.0.0");
        r.draft = true;
        assert!(matches!(
            accept(&r, "1.0.0", false),
            Err(Error::NotThatRelease(_))
        ));
        r.draft = false;
        r.prerelease = true;
        assert!(matches!(
            accept(&r, "1.0.0", false),
            Err(Error::NotThatRelease(_))
        ));
        let r = release("latest");
        let f = fake(&r, b"x", &hash(b"x"));
        assert!(matches!(
            download(&f, &r, "1.0.0", false, ASSET, t.path()),
            Err(Error::NotThatRelease(_))
        ));
        assert!(
            f.asked.borrow().is_empty(),
            "nothing is fetched for a release that is refused"
        );
    }

    #[test]
    fn an_asset_name_that_is_not_one_plain_file_name_is_refused_before_anything_happens() {
        let t = TempDir::new("verify-name");
        let r = release("v2.0.0");
        let f = fake(&r, b"x", &hash(b"x"));
        for bad in ["../x", "a/b", "a\\b", "..", ".", "", "C:x"] {
            assert!(
                matches!(
                    download(&f, &r, "1.0.0", false, bad, t.path()),
                    Err(Error::WrongAddress(_))
                ),
                "{bad}"
            );
        }
        assert!(f.asked.borrow().is_empty());
        assert!(std::fs::read_dir(t.path()).unwrap().next().is_none());
    }

    #[test]
    fn an_older_or_equal_version_is_refused_unless_asked_for() {
        let r = release("v2.0.0");
        for running in ["2.0.0", "2.1.0"] {
            assert!(
                matches!(accept(&r, running, false), Err(Error::NotNewer { .. })),
                "{running}"
            );
            assert_eq!(accept(&r, running, true).unwrap(), Version::new(2, 0, 0));
        }
        assert!(accept(&r, "1.9.9", false).is_ok());
    }

    #[test]
    fn a_file_at_another_address_is_refused() {
        let t = TempDir::new("verify-address");
        let mut r = release("v2.0.0");
        r.assets[1].browser_download_url = "https://evil.example/raul.tar.gz".to_owned();
        let f = fake(&r, b"x", &hash(b"x"));
        let e = download(&f, &r, "1.0.0", false, ASSET, t.path()).unwrap_err();
        assert_eq!(e, Error::WrongAddress(ASSET.to_owned()));
        assert!(f.asked.borrow().is_empty());
    }

    #[test]
    fn a_missing_asset_or_missing_line_is_refused() {
        let t = TempDir::new("verify-missing");
        let mut r = release("v2.0.0");
        r.assets.retain(|a| a.name != ASSET);
        let f = fake(&r, b"x", &hash(b"x"));
        assert_eq!(
            download(&f, &r, "1.0.0", false, ASSET, t.path()).unwrap_err(),
            Error::Missing(ASSET.to_owned())
        );
        let r = release("v2.0.0");
        let f = fake(&r, b"x", &hash(b"x"));
        f.files.borrow_mut().insert(
            r.address_of(SUMS),
            format!("{}  other\n", hash(b"o")).into_bytes(),
        );
        assert!(matches!(
            download(&f, &r, "1.0.0", false, ASSET, t.path()),
            Err(Error::BadSums(_))
        ));
    }

    #[test]
    fn a_file_past_its_limit_is_refused() {
        let t = TempDir::new("verify-limit");
        let r = release("v2.0.0");
        let f = fake(&r, b"x", &hash(b"x"));
        let big = vec![b'a'; usize::try_from(MAX_SUMS_BYTES).unwrap() + 1];
        f.files.borrow_mut().insert(r.address_of(SUMS), big);
        assert!(matches!(
            download(&f, &r, "1.0.0", false, ASSET, t.path()),
            Err(Error::TooBig(_))
        ));
    }

    #[test]
    fn the_checksum_list_reads_both_styles_and_refuses_the_malformed() {
        let h = hash(b"a");
        let text = format!("{h}  one\n{} *two\n\n", h.to_uppercase());
        assert_eq!(
            parse_sums(&text).unwrap(),
            vec![("one".to_owned(), h.clone()), ("two".to_owned(), h.clone())]
        );
        for bad in [
            "nospace".to_owned(),
            format!("{h}  "),
            format!("{}  x", "z".repeat(64)),
            format!("{h}  a\n{h}  a"),
        ] {
            assert!(matches!(parse_sums(&bad), Err(Error::BadSums(_))), "{bad}");
        }
    }

    #[test]
    fn the_messages_hold_no_command_and_no_tool_name() {
        for e in [
            Error::Mismatch(ASSET.to_owned()),
            Error::SumsChanged,
            Error::NotNewer {
                offered: "1".into(),
                running: "2".into(),
            },
            Error::NotThatRelease("it is a draft".into()),
            hash_file(Path::new("/definitely/not/here")).unwrap_err(),
        ] {
            let text = e.to_string();
            for word in ["raul update", "curl", "sha256sum", "--", "/", "\\"] {
                assert!(!text.contains(word), "{word}: {text}");
            }
        }
    }
}
