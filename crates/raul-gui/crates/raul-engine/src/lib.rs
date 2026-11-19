// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox adapter of the engine's crates (ADR-0016, ADR-0017), linked in process.
//!
//! Everything here sits behind the cargo feature `engine`: without it the crate is empty and the engine's closure
//! (wgpu, winit and the engine crates) is not compiled. The typed jobs are added to this crate one at a time.

// The linked crates, so the Toolbox reaches them through this one adapter (and the build compiles and links them).
#[cfg(feature = "engine")]
pub use {fork_api, fork_play};

/// Whether this build links the engine's crates.
pub const ENGINE_LINKED: bool = cfg!(feature = "engine");

/// The version of the engine facade this build was written against (`fork_api::API_VERSION`). The adapters check it
/// against the version they were written for.
#[cfg(feature = "engine")]
pub const FACADE_API_VERSION: u32 = fork_api::API_VERSION;

/// What the installation check found in a game folder, as the Toolbox shows it.
#[cfg(feature = "engine")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineCheck {
    /// The folder is accepted; the lines say what the run will find.
    Accepted(Vec<String>),
    /// Accepted, but this build does not run the game on this system yet. Not a fault of the folder.
    NotProvidedOnThisSystem(String),
    /// The folder is refused (no `FalloutNV.exe`, another version, an unsupported setting).
    Refused(String),
}

/// The typed job `engine-check`: the installation check of the linked engine on the game folder `data`, in process
/// (what `nv-viewer --check` does, without starting a program). It reads files and asks the sound device what it
/// offers, so a window calls it off its UI thread.
#[cfg(feature = "engine")]
#[must_use]
pub fn engine_check(data: &std::path::Path) -> EngineCheck {
    use fork_api::{CheckKind, check};
    match check(data) {
        Ok(report) => EngineCheck::Accepted(report.lines),
        Err(e) => match e.kind {
            CheckKind::NotProvidedOnThisSystem => EngineCheck::NotProvidedOnThisSystem(e.message),
            _ => EngineCheck::Refused(e.message),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_flag_follows_the_feature() {
        assert_eq!(ENGINE_LINKED, cfg!(feature = "engine"));
    }

    #[cfg(feature = "engine")]
    #[test]
    fn facade_version_is_set() {
        assert!(std::hint::black_box(FACADE_API_VERSION) >= 1);
        assert!(fork_api::CAPABILITIES.trim_start().starts_with('{'));
    }

    #[cfg(feature = "engine")]
    #[test]
    fn an_empty_folder_is_refused_not_accepted() {
        let dir = std::env::temp_dir().join(format!("raul-engine-check-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let got = engine_check(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        match got {
            EngineCheck::Refused(message) => assert_ne!(message, ""),
            other => panic!("an empty folder must be refused, got {other:?}"),
        }
    }

    #[cfg(feature = "engine")]
    #[test]
    fn a_missing_folder_is_refused() {
        let got = engine_check(std::path::Path::new("/nonexistent/raul-engine-check"));
        assert!(matches!(got, EngineCheck::Refused(_)), "{got:?}");
    }
}
