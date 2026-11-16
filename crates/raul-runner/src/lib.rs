// SPDX-License-Identifier: GPL-3.0-only
//! The one subprocess runner of the Toolbox (ADR-0007). Every program the Toolbox starts goes through [`Runner`]: an
//! argument vector (never a shell line), a timeout, an output size cap and a scrubbed environment, with the
//! program's SHA-256 re-verified before each run when the caller pins one. This is the only crate allowed to use
//! `std::process::Command`; `clippy.toml` forbids it everywhere else and a test of this crate scans the sources.
//! It was copied with its history from nvmod-rs's `nvmod-runner`, which keeps its own copy until its removal.

mod program;
mod runner;
mod sha256;

pub use program::{Started, start_program};
pub use runner::{
    Call, EXIT_REFUSED, EXIT_TOOL_FAILED, EXIT_VERIFICATION, NETWORK_ENV, Output, Runner, ToolError,
};
pub use sha256::sha256_hex;
