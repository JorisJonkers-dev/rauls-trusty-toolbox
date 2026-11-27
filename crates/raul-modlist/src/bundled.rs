// SPDX-License-Identifier: GPL-3.0-only
//! The lists raul carries (`lists/` at the repository root), embedded so they work offline (ADR-0023).

use crate::error::Error;
use crate::model::ModList;

const BEAUTIFUL: &str = include_str!("../../../lists/beautiful.toml");

/// The ids of the bundled lists.
pub const BUNDLED_IDS: &[&str] = &["beautiful"];

/// The bundled list `id`: `None` when raul carries no such list, an error when the embedded text is broken
/// (a test makes that impossible in a release).
///
/// # Errors
/// The same as [`ModList::parse`].
#[must_use]
pub fn bundled(id: &str) -> Option<Result<ModList, Error>> {
    match id {
        "beautiful" => Some(ModList::parse(BEAUTIFUL)),
        _ => None,
    }
}
