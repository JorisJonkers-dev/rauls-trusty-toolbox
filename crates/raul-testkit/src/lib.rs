// SPDX-License-Identifier: GPL-3.0-only
//! The ids of the feature catalogue (`catalogue/`, docs/COVERAGE.md "Feature catalogue") for tests.
//!
//! A test says which promise it proves with `covers(&[ids::area::group::ITEM])`; an id the catalogue does not have
//! does not compile. Tests depend on this crate as a dev-dependency only.

// The module is written by tally's generator, which documents its own items in the file it writes.
#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/tally_ids.rs"));
