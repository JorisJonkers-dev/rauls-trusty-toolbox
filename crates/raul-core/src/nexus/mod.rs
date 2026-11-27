// SPDX-License-Identifier: GPL-3.0-only
//! The Nexus Mods API key and a small client for the Nexus REST API (v1).
//!
//! * [`key`]: where the key lives (environment variable, OS keyring, fallback file) and the [`Secret`] type that
//!   never prints itself.
//! * [`client`]: the [`NexusApi`] trait (validate, mod info, file list, md5 lookup, download link) and its
//!   implementation over a [`Transport`], so tests use a fake transport and no network.
//! * [`curl`]: the real [`Transport`]: `curl` through the Runner, with the key handed over on standard input.
//! * [`redact`]: removes the key and `apikey` headers from any text before it is shown or logged.
//!
//! The key is never written to `settings.toml` ([`identity`] stores only the name and premium flag), never put on a
//! command line, and never shown.

pub mod client;
pub mod curl;
pub mod identity;
pub mod key;
pub mod redact;

pub use client::{
    DownloadLink, DownloadUri, FileEntry, Header, Md5Match, ModInfo, NexusApi, NexusClient,
    NexusError, NxmGrant, RateLimit, Request, Response, Transport, TransportError, Validated,
};
pub use identity::Identity;
pub use key::{KeyError, KeySource, KeyStore, Secret};
