// SPDX-License-Identifier: GPL-3.0-only
//! The Nexus Mods REST API (v1) behind two seams: [`NexusApi`], which callers (the install plan, the CLI) use, and
//! [`Transport`], which moves bytes (curl in production, a fake in tests; no test reaches the network).
//!
//! Every request carries `Application-Name` and `Application-Version`, as Nexus's API policy requires. The key goes
//! in the `apikey` header, marked secret so a `Debug` print and every error message leave it out.

use std::fmt;
use std::sync::Mutex;

use serde::Deserialize;

use super::key::Secret;
use super::redact::redact;

/// The API's address.
pub const BASE_URL: &str = "https://api.nexusmods.com/v1";

/// The `Application-Name` header's value.
pub const APPLICATION_NAME: &str = "raul";

/// One request header; a secret one is never printed.
#[derive(Clone, PartialEq, Eq)]
pub struct Header {
    /// The header name.
    pub name: String,
    /// The value.
    pub value: String,
    /// Whether the value is a secret.
    pub secret: bool,
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = if self.secret {
            "[redacted]"
        } else {
            &self.value
        };
        write!(f, "{}: {value}", self.name)
    }
}

/// A GET request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The address, HTTPS.
    pub url: String,
    /// The headers.
    pub headers: Vec<Header>,
}

impl Request {
    /// The value of the header `name` (any case).
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.as_str())
    }
}

/// What came back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The HTTP status.
    pub status: u16,
    /// The response headers.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: String,
}

/// The request could not be made at all (no network, no `curl`, a timeout).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportError(pub String);

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TransportError {}

/// Moves one request to Nexus and the answer back.
pub trait Transport {
    /// Sends `request` and returns the answer whatever its status is.
    ///
    /// # Errors
    /// [`TransportError`] when no answer came.
    fn send(&self, request: &Request) -> Result<Response, TransportError>;
}

impl<T: Transport + ?Sized> Transport for &T {
    fn send(&self, request: &Request) -> Result<Response, TransportError> {
        (**self).send(request)
    }
}

/// Why a call did not give its answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NexusError {
    /// No answer came (the text has the key removed).
    Transport(String),
    /// Nexus does not accept the key (HTTP 401).
    Unauthorized,
    /// Nexus has no such mod, file or hash (HTTP 404).
    NotFound,
    /// The hourly or daily request limit is used up (HTTP 429).
    RateLimited(RateLimit),
    /// Another HTTP status, with Nexus's message when it sent one.
    Http {
        /// The status.
        status: u16,
        /// The message, key removed.
        message: String,
    },
    /// The answer is not what the API documents.
    Malformed(String),
    /// An argument that cannot be part of an address (a game name or a hash).
    BadArgument(String),
}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::Transport(why) => write!(f, "could not reach Nexus: {why}"),
            NexusError::Unauthorized => f.write_str("Nexus does not accept this API key"),
            NexusError::NotFound => f.write_str("Nexus has no such mod, file or hash"),
            NexusError::RateLimited(limit) => write!(
                f,
                "Nexus request limit reached ({}); try again later",
                limit.describe()
            ),
            NexusError::Http { status, message } => {
                write!(f, "Nexus answered HTTP {status}: {message}")
            }
            NexusError::Malformed(why) => write!(f, "Nexus gave an answer raul cannot read: {why}"),
            NexusError::BadArgument(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for NexusError {}

/// The requests Nexus says are left, from its `X-RL-*` headers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RateLimit {
    /// Requests left this hour.
    pub hourly_remaining: Option<u32>,
    /// Requests left today.
    pub daily_remaining: Option<u32>,
}

impl RateLimit {
    fn from_headers(headers: &[(String, String)]) -> RateLimit {
        let find = |name: &str| {
            headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .and_then(|(_, v)| v.trim().parse().ok())
        };
        RateLimit {
            hourly_remaining: find("x-rl-hourly-remaining"),
            daily_remaining: find("x-rl-daily-remaining"),
        }
    }

    /// For example `42 this hour, 1900 today`.
    #[must_use]
    pub fn describe(&self) -> String {
        let part = |n: Option<u32>, when: &str| {
            n.map_or_else(|| format!("unknown {when}"), |n| format!("{n} {when}"))
        };
        format!(
            "{}, {}",
            part(self.hourly_remaining, "this hour"),
            part(self.daily_remaining, "today")
        )
    }
}

/// The answer of `users/validate.json`: who the key belongs to. The e-mail address Nexus also sends is dropped.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Validated {
    /// The account's id.
    pub user_id: u64,
    /// The account name.
    pub name: String,
    /// Whether the account is premium.
    #[serde(default)]
    pub is_premium: bool,
    /// Whether the account is a supporter.
    #[serde(default)]
    pub is_supporter: bool,
}

/// A mod's page data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModInfo {
    /// The mod's id on its game.
    pub mod_id: u64,
    /// The mod's name.
    #[serde(default)]
    pub name: String,
    /// The current version.
    #[serde(default)]
    pub version: String,
    /// The one-line description.
    #[serde(default)]
    pub summary: String,
    /// The author.
    #[serde(default)]
    pub author: String,
}

/// One file of a mod.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FileEntry {
    /// The file's id.
    pub file_id: u64,
    /// The name shown on the page.
    #[serde(default)]
    pub name: String,
    /// The file's name as uploaded.
    #[serde(default)]
    pub file_name: String,
    /// The file's version.
    #[serde(default)]
    pub version: String,
    /// `MAIN`, `UPDATE`, `OPTIONAL`, `OLD_VERSION` and so on; absent for a removed file.
    #[serde(default)]
    pub category_name: Option<String>,
    /// The size in kilobytes.
    #[serde(default)]
    pub size_kb: Option<u64>,
    /// When it was uploaded, in seconds since the Unix epoch.
    #[serde(default)]
    pub uploaded_timestamp: Option<u64>,
    /// The MD5 of the archive, when Nexus gives it.
    #[serde(default)]
    pub md5: Option<String>,
}

/// One hit of an md5 lookup: the file and the mod it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Md5Match {
    /// The mod.
    #[serde(rename = "mod")]
    pub mod_info: ModInfo,
    /// The file.
    #[serde(rename = "file_details")]
    pub file: FileEntry,
}

/// One place to download from. The address is signed and short-lived: it is never stored, and `Debug` leaves it out.
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct DownloadUri {
    /// The mirror's name.
    #[serde(default)]
    pub name: String,
    /// The mirror's short name.
    #[serde(default)]
    pub short_name: String,
    /// The signed address.
    #[serde(rename = "URI")]
    pub uri: String,
}

impl fmt::Debug for DownloadUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DownloadUri({}, [address hidden])", self.short_name)
    }
}

/// What asking for a download link gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadLink {
    /// Signed addresses to download from (premium account, or a free account with a grant from an `nxm://` link).
    Links(Vec<DownloadUri>),
    /// The account may not get links from the API: it needs premium, or the file has to be downloaded by hand from
    /// `page`.
    NeedsPremiumOrManual {
        /// The file's page, where the "Manual download" button is.
        page: String,
    },
}

/// The `key` and `expires` of an `nxm://` link the person clicked: what lets a free account get a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NxmGrant {
    /// The `key` query value.
    pub key: String,
    /// The `expires` query value.
    pub expires: u64,
}

/// What the install plan and the commands use. [`NexusClient`] is the real one; tests of other crates can write their
/// own.
pub trait NexusApi {
    /// Who the key belongs to.
    ///
    /// # Errors
    /// [`NexusError::Unauthorized`] when Nexus does not accept the key, or any other [`NexusError`].
    fn validate(&self) -> Result<Validated, NexusError>;
    /// A mod's page data.
    ///
    /// # Errors
    /// [`NexusError::NotFound`] when there is no such mod, or any other [`NexusError`].
    fn mod_info(&self, game: &str, mod_id: u64) -> Result<ModInfo, NexusError>;
    /// The files of a mod.
    ///
    /// # Errors
    /// As [`NexusApi::mod_info`].
    fn file_list(&self, game: &str, mod_id: u64) -> Result<Vec<FileEntry>, NexusError>;
    /// The files whose archive has this MD5 (32 hex digits).
    ///
    /// # Errors
    /// [`NexusError::NotFound`] when Nexus knows no such file, or any other [`NexusError`].
    fn md5_lookup(&self, game: &str, md5: &str) -> Result<Vec<Md5Match>, NexusError>;
    /// A download link for a file. A premium account gets one directly; a free account only with the `grant` of an
    /// `nxm://` link. Otherwise the answer is [`DownloadLink::NeedsPremiumOrManual`], not an error.
    ///
    /// # Errors
    /// Any [`NexusError`] other than the missing permission.
    fn download_link(
        &self,
        game: &str,
        mod_id: u64,
        file_id: u64,
        grant: Option<&NxmGrant>,
    ) -> Result<DownloadLink, NexusError>;
    /// The request budget Nexus reported with its last answer, when there has been one.
    fn rate_limit(&self) -> Option<RateLimit>;
}

/// [`NexusApi`] over a [`Transport`].
pub struct NexusClient<T: Transport> {
    transport: T,
    key: Secret,
    app_version: String,
    base: String,
    limit: Mutex<Option<RateLimit>>,
}

impl<T: Transport> NexusClient<T> {
    /// A client that sends `key` and says it is raul `app_version`.
    #[must_use]
    pub fn new(transport: T, key: Secret, app_version: &str) -> Self {
        NexusClient {
            transport,
            key,
            app_version: app_version.to_owned(),
            base: BASE_URL.to_owned(),
            limit: Mutex::new(None),
        }
    }

    fn get(&self, path: &str) -> Result<Response, NexusError> {
        let header = |name: &str, value: &str, secret| Header {
            name: name.to_owned(),
            value: value.to_owned(),
            secret,
        };
        let request = Request {
            url: format!("{}{path}", self.base),
            headers: vec![
                header("apikey", self.key.expose(), true),
                header("Application-Name", APPLICATION_NAME, false),
                header("Application-Version", &self.app_version, false),
                header("Accept", "application/json", false),
            ],
        };
        let response = self
            .transport
            .send(&request)
            .map_err(|e| NexusError::Transport(redact(&e.0, Some(&self.key))))?;
        let limit = RateLimit::from_headers(&response.headers);
        if limit != RateLimit::default() {
            *self
                .limit
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(limit);
        }
        Ok(response)
    }

    fn fail(&self, response: &Response) -> NexusError {
        match response.status {
            401 => NexusError::Unauthorized,
            404 => NexusError::NotFound,
            429 => NexusError::RateLimited(RateLimit::from_headers(&response.headers)),
            status => NexusError::Http {
                status,
                message: self.message(&response.body),
            },
        }
    }

    /// Nexus's `message` (or the start of the body) with the key removed.
    fn message(&self, body: &str) -> String {
        let text = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str().map(str::to_owned)))
            .unwrap_or_else(|| body.chars().take(200).collect());
        redact(&text, Some(&self.key))
    }

    fn json<D: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<D, NexusError> {
        let response = self.get(path)?;
        if response.status != 200 {
            return Err(self.fail(&response));
        }
        serde_json::from_str(&response.body)
            .map_err(|e| NexusError::Malformed(redact(&e.to_string(), Some(&self.key))))
    }
}

/// A game domain such as `newvegas`.
fn game_ok(game: &str) -> Result<&str, NexusError> {
    if !game.is_empty()
        && game
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    {
        Ok(game)
    } else {
        Err(NexusError::BadArgument(format!(
            "{game:?} is not a Nexus game name (such as newvegas)"
        )))
    }
}

fn md5_ok(md5: &str) -> Result<String, NexusError> {
    if md5.len() == 32 && md5.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(md5.to_ascii_lowercase())
    } else {
        Err(NexusError::BadArgument(
            "an md5 is 32 hexadecimal digits".into(),
        ))
    }
}

impl<T: Transport> NexusApi for NexusClient<T> {
    fn validate(&self) -> Result<Validated, NexusError> {
        self.json("/users/validate.json")
    }

    fn mod_info(&self, game: &str, mod_id: u64) -> Result<ModInfo, NexusError> {
        self.json(&format!("/games/{}/mods/{mod_id}.json", game_ok(game)?))
    }

    fn file_list(&self, game: &str, mod_id: u64) -> Result<Vec<FileEntry>, NexusError> {
        #[derive(Deserialize)]
        struct Files {
            files: Vec<FileEntry>,
        }
        let files: Files = self.json(&format!(
            "/games/{}/mods/{mod_id}/files.json",
            game_ok(game)?
        ))?;
        Ok(files.files)
    }

    fn md5_lookup(&self, game: &str, md5: &str) -> Result<Vec<Md5Match>, NexusError> {
        self.json(&format!(
            "/games/{}/mods/md5_search/{}.json",
            game_ok(game)?,
            md5_ok(md5)?
        ))
    }

    fn download_link(
        &self,
        game: &str,
        mod_id: u64,
        file_id: u64,
        grant: Option<&NxmGrant>,
    ) -> Result<DownloadLink, NexusError> {
        let game = game_ok(game)?;
        let mut path = format!("/games/{game}/mods/{mod_id}/files/{file_id}/download_link.json");
        if let Some(grant) = grant {
            if !grant
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
            {
                return Err(NexusError::BadArgument(
                    "the nxm key has a character it cannot have".into(),
                ));
            }
            path = format!("{path}?key={}&expires={}", grant.key, grant.expires);
        }
        let response = self.get(&path)?;
        match response.status {
            200 => {}
            // Nexus answers 403 to an account that may not use the API for downloads.
            403 => {
                return Ok(DownloadLink::NeedsPremiumOrManual {
                    page: format!(
                        "https://www.nexusmods.com/{game}/mods/{mod_id}?tab=files&file_id={file_id}"
                    ),
                });
            }
            _ => return Err(self.fail(&response)),
        }
        let links: Vec<DownloadUri> = serde_json::from_str(&response.body)
            .map_err(|e| NexusError::Malformed(redact(&e.to_string(), Some(&self.key))))?;
        if links.is_empty() {
            return Err(NexusError::Malformed(
                "no download address in the answer".into(),
            ));
        }
        Ok(DownloadLink::Links(links))
    }

    fn rate_limit(&self) -> Option<RateLimit> {
        *self
            .limit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// Answers from a queue and keeps what it was asked.
    #[derive(Default)]
    pub struct FakeTransport {
        pub answers: RefCell<VecDeque<Result<Response, TransportError>>>,
        pub seen: RefCell<Vec<Request>>,
    }

    impl FakeTransport {
        pub fn answering(status: u16, body: &str) -> FakeTransport {
            let t = FakeTransport::default();
            t.push(status, body, &[]);
            t
        }

        pub fn push(&self, status: u16, body: &str, headers: &[(&str, &str)]) {
            self.answers.borrow_mut().push_back(Ok(Response {
                status,
                headers: headers
                    .iter()
                    .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
                    .collect(),
                body: body.to_owned(),
            }));
        }
    }

    impl Transport for FakeTransport {
        fn send(&self, request: &Request) -> Result<Response, TransportError> {
            self.seen.borrow_mut().push(request.clone());
            self.answers
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(TransportError("no answer queued".into())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeTransport;
    use super::*;

    const KEY: &str = "synthetic-key-0123456789==";

    fn client(t: &FakeTransport) -> NexusClient<&FakeTransport> {
        NexusClient::new(t, Secret::new(KEY).unwrap(), "9.9.9")
    }

    #[test]
    fn validate_reads_the_account_and_sends_the_key_and_the_application_headers() {
        let t = FakeTransport::answering(
            200,
            r#"{"user_id":7,"key":"ignored","name":"Courier Six","is_premium":true,"is_supporter":false,"email":"a@b.c"}"#,
        );
        let who = client(&t).validate().unwrap();
        assert_eq!(
            (who.name.as_str(), who.is_premium, who.user_id),
            ("Courier Six", true, 7)
        );
        let seen = t.seen.borrow();
        let request = &seen[0];
        assert_eq!(
            request.url,
            "https://api.nexusmods.com/v1/users/validate.json"
        );
        assert_eq!(request.header("apikey"), Some(KEY));
        assert_eq!(request.header("Application-Name"), Some("raul"));
        assert_eq!(request.header("Application-Version"), Some("9.9.9"));
    }

    #[test]
    fn a_request_never_prints_the_key() {
        let t = FakeTransport::answering(200, r#"{"user_id":1,"name":"x"}"#);
        let _ = client(&t).validate();
        assert!(!format!("{:?}", t.seen.borrow()).contains(KEY));
    }

    #[test]
    fn a_key_nexus_refuses_is_unauthorized() {
        let t = FakeTransport::answering(401, r#"{"message":"Please provide a valid API Key"}"#);
        assert_eq!(client(&t).validate(), Err(NexusError::Unauthorized));
    }

    #[test]
    fn mod_info_and_the_file_list_are_read_from_nexus_json() {
        let t = FakeTransport::answering(
            200,
            r#"{"mod_id":42,"name":"Synthetic Mod","version":"1.2","summary":"s","author":"a","extra":1}"#,
        );
        t.push(
            200,
            r#"{"files":[{"file_id":9,"name":"Main","file_name":"Main-42-1-2.7z","version":"1.2","category_name":"MAIN","size_kb":10,"uploaded_timestamp":1700000000,"md5":"d41d8cd98f00b204e9800998ecf8427e"},{"file_id":10,"name":"Gone","category_name":null}],"file_updates":[]}"#,
            &[],
        );
        let c = client(&t);
        assert_eq!(c.mod_info("newvegas", 42).unwrap().name, "Synthetic Mod");
        let files = c.file_list("newvegas", 42).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(
            files[0].md5.as_deref(),
            Some("d41d8cd98f00b204e9800998ecf8427e")
        );
        assert_eq!(files[1].category_name, None);
        let seen = t.seen.borrow();
        assert_eq!(
            seen[0].url,
            "https://api.nexusmods.com/v1/games/newvegas/mods/42.json"
        );
        assert_eq!(
            seen[1].url,
            "https://api.nexusmods.com/v1/games/newvegas/mods/42/files.json"
        );
    }

    #[test]
    fn an_md5_lookup_returns_the_mod_and_the_file() {
        let t = FakeTransport::answering(
            200,
            r#"[{"mod":{"mod_id":1,"name":"M"},"file_details":{"file_id":2,"name":"F"}}]"#,
        );
        let hits = client(&t)
            .md5_lookup("newvegas", "D41D8CD98F00B204E9800998ECF8427E")
            .unwrap();
        assert_eq!((hits[0].mod_info.mod_id, hits[0].file.file_id), (1, 2));
        assert!(
            t.seen.borrow()[0]
                .url
                .ends_with("/md5_search/d41d8cd98f00b204e9800998ecf8427e.json")
        );
    }

    #[test]
    fn arguments_that_could_change_the_address_are_refused_before_any_request() {
        let t = FakeTransport::default();
        let c = client(&t);
        assert!(matches!(
            c.mod_info("../users", 1),
            Err(NexusError::BadArgument(_))
        ));
        assert!(matches!(
            c.md5_lookup("newvegas", "xyz"),
            Err(NexusError::BadArgument(_))
        ));
        assert!(t.seen.borrow().is_empty());
    }

    #[test]
    fn a_premium_account_gets_signed_links_that_debug_does_not_print() {
        let t = FakeTransport::answering(
            200,
            r#"[{"name":"Nexus CDN","short_name":"Nexus CDN","URI":"https://cdn.example/file.7z?md5=abc&expires=1"}]"#,
        );
        let link = client(&t).download_link("newvegas", 42, 9, None).unwrap();
        let DownloadLink::Links(links) = &link else {
            panic!("{link:?}")
        };
        assert_eq!(
            links[0].uri,
            "https://cdn.example/file.7z?md5=abc&expires=1"
        );
        assert!(!format!("{link:?}").contains("cdn.example"));
    }

    #[test]
    fn a_free_account_is_told_to_download_by_hand_not_given_an_error() {
        let t = FakeTransport::answering(403, r#"{"message":"This is for premium users only"}"#);
        let link = client(&t).download_link("newvegas", 42, 9, None).unwrap();
        assert_eq!(
            link,
            DownloadLink::NeedsPremiumOrManual {
                page: "https://www.nexusmods.com/newvegas/mods/42?tab=files&file_id=9".into()
            }
        );
    }

    #[test]
    fn an_nxm_grant_is_sent_as_the_key_and_expires_query() {
        let t = FakeTransport::answering(
            200,
            r#"[{"name":"n","short_name":"s","URI":"https://x/y"}]"#,
        );
        let grant = NxmGrant {
            key: "AbC-1_2".into(),
            expires: 1_700_000_000,
        };
        client(&t)
            .download_link("newvegas", 1, 2, Some(&grant))
            .unwrap();
        assert!(
            t.seen.borrow()[0]
                .url
                .ends_with("download_link.json?key=AbC-1_2&expires=1700000000")
        );
        let bad = NxmGrant {
            key: "a&b".into(),
            expires: 1,
        };
        assert!(matches!(
            client(&t).download_link("newvegas", 1, 2, Some(&bad)),
            Err(NexusError::BadArgument(_))
        ));
    }

    #[test]
    fn the_request_budget_is_read_from_the_headers_and_a_429_reports_it() {
        let t = FakeTransport::default();
        t.push(
            200,
            r#"{"user_id":1,"name":"x"}"#,
            &[
                ("X-RL-Hourly-Remaining", "98"),
                ("x-rl-daily-remaining", "2400"),
            ],
        );
        t.push(
            429,
            "{}",
            &[
                ("x-rl-hourly-remaining", "0"),
                ("x-rl-daily-remaining", "10"),
            ],
        );
        let c = client(&t);
        c.validate().unwrap();
        assert_eq!(
            c.rate_limit(),
            Some(RateLimit {
                hourly_remaining: Some(98),
                daily_remaining: Some(2400)
            })
        );
        let Err(NexusError::RateLimited(limit)) = c.validate() else {
            panic!()
        };
        assert_eq!(limit.hourly_remaining, Some(0));
    }

    #[test]
    fn errors_never_contain_the_key_even_when_nexus_or_the_transport_echo_it() {
        let t = FakeTransport::answering(500, &format!(r#"{{"message":"bad key {KEY}"}}"#));
        let e = client(&t).validate().unwrap_err().to_string();
        assert!(!e.contains(KEY), "{e}");
        let t = FakeTransport::default();
        t.answers
            .borrow_mut()
            .push_back(Err(TransportError(format!("curl: apikey: {KEY} failed"))));
        let e = client(&t).validate().unwrap_err().to_string();
        assert!(!e.contains(KEY), "{e}");
    }

    #[test]
    fn an_answer_that_is_not_json_is_malformed() {
        let t = FakeTransport::answering(200, "<html>");
        assert!(matches!(
            client(&t).validate(),
            Err(NexusError::Malformed(_))
        ));
    }
}
