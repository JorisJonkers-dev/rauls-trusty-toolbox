// SPDX-License-Identifier: GPL-3.0-only
//! The service seam of the nvmod GUI (`docs/design/gui/04-view-model-api.md` in nv-linux).
//!
//! * [`Service`]: what screens and wizards ask of the world. [`CliService`] answers by running the `nvmod`
//!   command line as a subprocess and reading its `--json` output, [`FakeService`] from a script.
//! * [`registry`]: the table that ties every GUI action to the command line that does the same, and the checks
//!   that fail when a screen or an action has no equivalent.
//! * [`wiring`]: drives the install, MO2 import and TTW wizards of `nvmod-ui-model` with a service.
//!
//! No GUI toolkit is used here.

#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod dto;
pub mod exec;
pub mod fake;
pub mod registry;
pub mod service;
pub mod wiring;

pub use exec::{Exec, Output, ProcessExec};
pub use fake::FakeService;
pub use service::{
    CliService, DoctorRequest, ImportRequest, InstallRequest, InstallResult, Service, ServiceError,
    TtwRequest,
};
