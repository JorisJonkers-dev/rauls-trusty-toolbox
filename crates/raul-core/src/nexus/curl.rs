// SPDX-License-Identifier: GPL-3.0-only
//! The real [`Transport`]: `curl` through the Runner (ADR-0002). Everything that varies, the address and every header
//! with the API key among them, goes to `curl` as a config file on its standard input (`--config -`), so nothing but
//! fixed flags is on the command line, where any process on the machine could read it.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_runner::{Call, NETWORK_ENV, Runner};

use super::client::{Request, Response, Transport, TransportError};
use super::redact::redact;

/// How long one request may take.
pub const TIMEOUT_SECS: u64 = 30;

/// The largest answer accepted (a mod with very many files is well below it).
pub const MAX_BYTES: usize = 8 * 1024 * 1024;

/// The fixed `curl` arguments: the person's `.curlrc` ignored (it must come first), HTTPS only, no redirects followed
/// (the API does not redirect, and a redirect must not carry the key elsewhere), a time and size limit, response
/// headers included, and the rest read from standard input.
#[must_use]
pub fn curl_args() -> Vec<OsString> {
    [
        "--disable",
        "--silent",
        "--show-error",
        "--include",
        "--proto",
        "=https",
        "--max-time",
        &TIMEOUT_SECS.to_string(),
        "--max-filesize",
        &MAX_BYTES.to_string(),
        "--config",
        "-",
    ]
    .map(OsString::from)
    .to_vec()
}

/// The `curl` config for `request`: one `url` line and one `header` line per header. Values are quoted and escaped,
/// and a line break in a value is refused, so a value cannot add a line of its own.
///
/// # Errors
/// [`TransportError`] when the address or a header holds a line break.
pub fn config_for(request: &Request) -> Result<String, TransportError> {
    let quote = |text: &str| -> Result<String, TransportError> {
        if text.chars().any(char::is_control) {
            return Err(TransportError(
                "a request value holds a control character".into(),
            ));
        }
        Ok(format!(
            "\"{}\"",
            text.replace('\\', "\\\\").replace('"', "\\\"")
        ))
    };
    let mut config = format!("url = {}\n", quote(&request.url)?);
    for header in &request.headers {
        config.push_str("header = ");
        config.push_str(&quote(&format!("{}: {}", header.name, header.value))?);
        config.push('\n');
    }
    Ok(config)
}

/// Splits `curl --include` output into the status, the headers and the body. Interim `1xx` answers are skipped.
///
/// # Errors
/// [`TransportError`] when the output is not an HTTP answer.
pub fn parse_response(raw: &[u8]) -> Result<Response, TransportError> {
    let text = String::from_utf8_lossy(raw);
    let mut rest: &str = &text;
    loop {
        let (head, body) = rest
            .split_once("\r\n\r\n")
            .or_else(|| rest.split_once("\n\n"))
            .ok_or_else(|| TransportError("the answer is not HTTP".into()))?;
        let mut lines = head.lines();
        let status = lines
            .next()
            .filter(|l| l.starts_with("HTTP/"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| TransportError("the answer is not HTTP".into()))?;
        if (100..200).contains(&status) {
            rest = body;
            continue;
        }
        let headers = lines
            .filter_map(|l| l.split_once(':'))
            .map(|(n, v)| (n.trim().to_owned(), v.trim().to_owned()))
            .collect();
        return Ok(Response {
            status,
            headers,
            body: body.to_owned(),
        });
    }
}

/// `curl` as a [`Transport`].
#[derive(Debug, Clone)]
pub struct CurlTransport {
    /// The `curl` program.
    pub curl: PathBuf,
}

impl Default for CurlTransport {
    fn default() -> Self {
        CurlTransport {
            curl: PathBuf::from("curl"),
        }
    }
}

impl CurlTransport {
    /// Use the `curl` at `curl`.
    #[must_use]
    pub fn at(curl: &Path) -> Self {
        CurlTransport {
            curl: curl.to_path_buf(),
        }
    }
}

impl Transport for CurlTransport {
    fn send(&self, request: &Request) -> Result<Response, TransportError> {
        let config = config_for(request)?;
        let runner = Runner {
            timeout: Duration::from_secs(TIMEOUT_SECS + 15),
            max_output: MAX_BYTES + 64 * 1024,
            ..Runner::default()
        }
        .inherit(NETWORK_ENV);
        let args = curl_args();
        let mut call = Call::new("curl", &self.curl, &args);
        call.stdin = Some(config.as_bytes());
        // `curl`'s own message can quote the header it was given; the Runner's text and curl's are cleaned.
        let out = runner
            .run_any(&call)
            .map_err(|e| TransportError(redact(&e.to_string(), None)))?;
        if out.exit_code != 0 {
            return Err(TransportError(format!(
                "curl exited with {}: {}",
                out.exit_code,
                redact(out.stderr.trim(), None)
            )));
        }
        parse_response(&out.stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::Header;
    use super::*;

    fn request() -> Request {
        Request {
            url: "https://api.nexusmods.com/v1/users/validate.json".into(),
            headers: vec![Header {
                name: "apikey".into(),
                value: "k\"ey\\x".into(),
                secret: true,
            }],
        }
    }

    #[test]
    fn the_key_is_in_the_config_for_standard_input_and_not_in_the_arguments() {
        let args: Vec<String> = curl_args()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            args.iter()
                .all(|a| !a.contains("apikey") && !a.contains("nexusmods")),
            "{args:?}"
        );
        assert_eq!(args[0], "--disable");
        let config = config_for(&request()).unwrap();
        assert!(
            config.contains(r#"header = "apikey: k\"ey\\x""#),
            "{config}"
        );
        assert!(config.starts_with("url = \"https://api.nexusmods.com"));
    }

    #[test]
    fn a_line_break_cannot_add_a_line_to_the_config() {
        let mut r = request();
        r.headers[0].value = "a\nurl = \"https://evil\"".into();
        assert!(config_for(&r).is_err());
    }

    #[test]
    fn the_status_headers_and_body_are_read_from_curl_include_output() {
        let raw = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/2 429 \r\nX-RL-Hourly-Remaining: 0\r\ncontent-type: application/json\r\n\r\n{\"a\":1}";
        let r = parse_response(raw).unwrap();
        assert_eq!(r.status, 429);
        assert_eq!(r.body, "{\"a\":1}");
        assert!(
            r.headers
                .iter()
                .any(|(n, v)| n == "X-RL-Hourly-Remaining" && v == "0")
        );
        assert!(parse_response(b"not http").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_stub_program_receives_the_key_on_standard_input_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("raul-curl-stub-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let stub = dir.join("curl");
        // Prints a fixed HTTP answer, and appends its arguments and its input to a file for the test to read.
        std::fs::write(
            &stub,
            format!(
                "#!/bin/sh\necho \"$@\" > {d}/args\ncat > {d}/stdin\nprintf 'HTTP/2 200\\r\\n\\r\\n{{\"ok\":true}}'\n",
                d = dir.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
        let response = CurlTransport::at(&stub).send(&request()).unwrap();
        assert_eq!(
            (response.status, response.body.as_str()),
            (200, "{\"ok\":true}")
        );
        let args = std::fs::read_to_string(dir.join("args")).unwrap();
        let stdin = std::fs::read_to_string(dir.join("stdin")).unwrap();
        assert!(!args.contains("ey"), "{args}");
        assert!(stdin.contains("apikey: k"), "{stdin}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
