// SPDX-License-Identifier: GPL-3.0-only
//! The real [`Fetch`]: `curl` through the Runner, HTTPS only, nothing sent that identifies the person (ADR-0005,
//! ADR-0020). It runs only when the caller asks for a download.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_runner::{Call, NETWORK_ENV, Runner};

use super::verify::{Error, Fetch, REPO, Release};

/// How long an asset may take to download.
pub const ASSET_TIMEOUT: Duration = Duration::from_mins(20);

/// The address of the release page's description of the release tagged `tag`.
#[must_use]
pub fn release_url(tag: &str) -> String {
    format!("https://api.github.com/repos/{REPO}/releases/tags/{tag}")
}

/// The `curl` arguments that write `url` to `to`: the person's `.curlrc` ignored, HTTPS only (redirects too), a time
/// limit, and a size limit that makes `curl` stop at `limit` bytes.
#[must_use]
pub fn curl_args(url: &str, to: &Path, limit: u64) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
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
        &ASSET_TIMEOUT.as_secs().to_string(),
        "--max-filesize",
        &limit.to_string(),
        "--header",
        "Accept: application/octet-stream, application/vnd.github+json",
        "--output",
    ]
    .map(OsString::from)
    .to_vec();
    // The path goes as it is: a name that is not UTF-8 stays what it is.
    args.push(to.as_os_str().to_owned());
    args.push(url.into());
    args
}

/// `curl` as a [`Fetch`].
#[derive(Debug, Clone)]
pub struct CurlFetch {
    /// The `curl` program.
    pub curl: PathBuf,
}

impl Default for CurlFetch {
    fn default() -> Self {
        Self {
            curl: PathBuf::from("curl"),
        }
    }
}

impl Fetch for CurlFetch {
    fn get(&self, url: &str, to: &Path, limit: u64) -> Result<(), Error> {
        let runner = Runner {
            timeout: ASSET_TIMEOUT + Duration::from_secs(30),
            ..Runner::default()
        }
        .inherit(NETWORK_ENV);
        let args = curl_args(url, to, limit);
        // `curl` stops at the size limit by itself (exit 63) and the file is checked once more here. Whatever fails,
        // no partial file stays behind.
        let result = self.run(&runner, &args, url, to, limit);
        if result.is_err() {
            let _ = std::fs::remove_file(to);
        }
        result
    }
}

impl CurlFetch {
    fn run(
        &self,
        runner: &Runner,
        args: &[OsString],
        url: &str,
        to: &Path,
        limit: u64,
    ) -> Result<(), Error> {
        runner
            .run_any(&Call::new("curl", &self.curl, args))
            .map_err(|_| Error::Fetch("the connection failed".to_owned()))
            .and_then(|out| {
                if out.exit_code == 63 {
                    Err(Error::TooBig(url.to_owned()))
                } else if out.exit_code != 0 {
                    Err(Error::Fetch("the server did not give the file".to_owned()))
                } else {
                    Ok(())
                }
            })?;
        match std::fs::metadata(to) {
            Ok(m) if m.len() <= limit => Ok(()),
            Ok(_) => Err(Error::TooBig(url.to_owned())),
            Err(e) => Err(Error::Fetch(e.to_string())),
        }
    }
}

/// The description of the release tagged `tag`, fetched into `work`.
///
/// # Errors
/// [`Error::Fetch`] when it cannot be fetched or read.
pub fn release(fetch: &dyn Fetch, tag: &str, work: &Path) -> Result<Release, Error> {
    std::fs::create_dir_all(work).map_err(|e| Error::Fetch(format!("{}: {e}", work.display())))?;
    let file = work.join("release.json");
    fetch.get(&release_url(tag), &file, 4 * 1024 * 1024)?;
    let text = std::fs::read_to_string(&file).map_err(|e| Error::Fetch(e.to_string()));
    let _ = std::fs::remove_file(&file);
    serde_json::from_str(&text?).map_err(|_| {
        Error::Fetch("the release page gave an answer that could not be read".to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curl_arguments_are_https_only_ignore_curlrc_and_limit_the_size() {
        let args = curl_args("https://github.com/x", Path::new("/tmp/out"), 1234);
        assert_eq!(
            args[0], "--disable",
            "the person's .curlrc is ignored first"
        );
        for pair in [
            ["--proto", "=https"],
            ["--proto-redir", "=https"],
            ["--max-filesize", "1234"],
            ["--output", "/tmp/out"],
        ] {
            assert!(args.windows(2).any(|w| w == pair), "{pair:?}");
        }
        assert_eq!(args.last().unwrap(), "https://github.com/x");
        assert!(!args.iter().any(|a| {
            let a = a.to_string_lossy().to_lowercase();
            a.contains("cookie") || a.contains("user-agent")
        }));
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_download_leaves_no_partial_file() {
        let t = nvmod_testkit::TempDir::new("fetch-partial");
        let out = t.path().join("asset");
        // A stand-in `curl` that writes part of the file and then fails, as a dropped connection does.
        let curl = nvmod_testkit::write_script(
            t.path(),
            "curl",
            &format!("echo partial > '{}'\nexit 18", out.display()),
        );
        let got = CurlFetch { curl }.get("https://github.com/x", &out, 1024);
        assert!(got.is_err());
        assert!(!out.exists(), "the partial file is removed");
        // And one that succeeds keeps its file.
        let ok = nvmod_testkit::write_script(
            t.path(),
            "curl-ok",
            &format!("echo fine > '{}'", out.display()),
        );
        assert!(
            CurlFetch { curl: ok }
                .get("https://github.com/x", &out, 1024)
                .is_ok()
        );
        assert!(out.is_file());
    }

    #[test]
    fn the_release_address_is_the_repositorys_tag_page() {
        assert_eq!(
            release_url("v1.2.3"),
            "https://api.github.com/repos/JorisJonkers-dev/rauls-trusty-toolbox/releases/tags/v1.2.3"
        );
    }

    #[test]
    fn a_release_page_answer_is_read_and_a_bad_one_is_refused() {
        struct Say(&'static str);
        impl Fetch for Say {
            fn get(&self, _: &str, to: &Path, _: u64) -> Result<(), Error> {
                std::fs::write(to, self.0).map_err(|e| Error::Fetch(e.to_string()))
            }
        }
        let t = nvmod_testkit::TempDir::new("fetch-release");
        let ok = release(&Say(r#"{"id":5,"tag_name":"v1.0.0","assets":[{"name":"a","browser_download_url":"u"}]}"#), "v1.0.0", t.path()).unwrap();
        assert_eq!(
            (ok.id, ok.tag_name.as_str(), ok.assets.len()),
            (5, "v1.0.0", 1)
        );
        assert!(release(&Say("not json"), "v1.0.0", t.path()).is_err());
    }
}
