// SPDX-License-Identifier: GPL-3.0-only
//! The view model of the Toolbox GUI (ADR-0003, ADR-0007), moved from nvmod-rs with its history.
//!
//! Nothing here knows a GUI toolkit, a window or a process: every type is plain data and every transition is a
//! pure method, so the whole behaviour of the wizards, the command palette and the keymap is tested with
//! `cargo test`. A toolkit adapter renders these values and forwards user intent as inputs.
//!
//! * [`route`]: the screens, the left rail and the `Ctrl+1` to `Ctrl+8` mapping.
//! * [`events`]: the `raul-events` stream of the driver commands (`raul ttw`, `raul lod generate`).
//! * [`error`]: exit classes of the command line and the error shape every screen shows.
//! * [`job`]: the job events and the table that orders and de-duplicates them.
//! * [`doctor`] and [`mods`]: the state of the Doctor and Mods screens.
//! * [`setup`]: the real `raul setup --json` event stream and the setup wizard it drives.
//! * [`wizard`]: the install and FOMOD, MO2 import and TTW wizards.
//! * [`keymap`] and [`palette`]: the shortcuts and the command palette with CLI equivalents.
//! * [`pages`]: the pages and flows of the minimum viable Toolbox, one module each.

#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
#![cfg_attr(test, allow(clippy::assert_is_empty))]

pub mod doctor;
pub mod error;
pub mod events;
pub mod job;
pub mod keymap;
pub mod mods;
pub mod pages;
pub mod palette;
pub mod route;
pub mod setup;
pub mod wizard;

pub use error::{ExitClass, Load, ViewError};
pub use route::{Route, Screen};
