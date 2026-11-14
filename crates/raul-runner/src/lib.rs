// SPDX-License-Identifier: GPL-3.0-only
//! The one subprocess runner of nvmod (ADR-0005). Every program nvmod starts goes through [`Runner`]: an argument
//! vector (never a shell line), a timeout, an output size cap and a scrubbed environment, with the program's
//! SHA-256 re-verified before each run when the caller pins one. This is the only crate allowed to use
//! `std::process::Command`; `clippy.toml` forbids it everywhere else and a test of this crate scans the sources.

mod runner;
mod sha256;

pub use runner::{
    Call, EXIT_REFUSED, EXIT_TOOL_FAILED, EXIT_VERIFICATION, NETWORK_ENV, Output, Runner, ToolError,
};
pub use sha256::sha256_hex;
