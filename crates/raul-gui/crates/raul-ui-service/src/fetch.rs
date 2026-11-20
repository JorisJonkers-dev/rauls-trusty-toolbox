// SPDX-License-Identifier: GPL-3.0-only
//! The fetcher of the tool store, owned by Raul. nvmod's tool store (`nvmod_tools::Acquire`) asks a
//! [`nvmod_tools::Fetcher`] for the bytes behind a URL and starts no process itself; Raul gives it one. For now it
//! starts `curl` through the one Runner (ADR-0007), which every supported system has and which gives `https`,
//! redirects, a hard timeout and `file://` without a TLS stack in the Toolbox. An in-process HTTPS client is a later
//! decision (an ADR comes before any HTTP crate).

use std::path::PathBuf;
use std::time::Duration;

use nvmod_tools::{FetchError, Fetcher, TIMEOUT_SECS};
use raul_runner::{Call, NETWORK_ENV, Runner, ToolError};

/// The largest response accepted.
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curl_command_line_has_the_timeout_and_ends_with_the_url() {
        let args = CurlFetcher::args("http://h/x").join(" ");
        assert!(args.contains("--max-time 60"), "{args}");
        assert!(args.ends_with("http://h/x"));
    }

    #[test]
    fn a_file_url_is_read() {
        let t = nvmod_testkit::TempDir::new("curl-fetcher-file");
        let file = t.path().join("body");
        std::fs::write(&file, b"hello").unwrap();
        let got = CurlFetcher::default().get(&format!("file://{}", file.display()));
        assert_eq!(got, Ok(b"hello".to_vec()));
    }

    #[test]
    fn an_unreadable_file_url_is_fatal_and_a_missing_curl_is_fatal() {
        let t = nvmod_testkit::TempDir::new("curl-fetcher-missing");
        let url = format!("file://{}", t.path().join("nope").display());
        assert!(matches!(
            CurlFetcher::default().get(&url),
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
        let t = nvmod_testkit::TempDir::new("curl-fetcher-exit");
        for (code, malformed, fatal) in [(18, true, false), (37, false, true), (28, false, false)] {
            let curl = nvmod_testkit::write_script(
                t.path(),
                &format!("curl-{code}"),
                &format!("echo no >&2\nexit {code}"),
            );
            let got = CurlFetcher { program: curl }.get("https://h/x");
            match got {
                Err(FetchError::Malformed(_)) => assert!(malformed, "{code}"),
                Err(FetchError::Fatal(_)) => assert!(fatal, "{code}"),
                Err(FetchError::Transient(_)) => assert!(!malformed && !fatal, "{code}"),
                other => panic!("{code}: {other:?}"),
            }
        }
    }
}
