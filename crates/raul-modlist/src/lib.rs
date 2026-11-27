// SPDX-License-Identifier: GPL-3.0-only
//! Mod lists (ADR-0023): a file of references that names Mods, where to get each one and in what order they go.
//!
//! [`ModList::parse`] reads and checks one; [`bundled`] holds the lists raul carries and [`plan`] says what an Instance has and lacks. This crate links no Component and holds no
//! mod data, so it builds and tests on its own.

mod bundled;
mod error;
mod export;
mod model;
mod parse;
pub mod plan;

pub use bundled::{BUNDLED_IDS, bundled};
pub use error::Error;
pub use export::{ExportError, ExportOptions, export_modlist};
pub use model::{Game, Group, ModEntry, ModList, PluginOrder, SCHEMA, Source, Target};
