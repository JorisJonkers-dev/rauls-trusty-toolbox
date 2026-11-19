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
}
