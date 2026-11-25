// SPDX-License-Identifier: GPL-3.0-only
//! The crate graph of docs/REORG.md section 3, enforced: which Toolbox crate may depend on which. It reads the
//! `[dependencies]` and `[build-dependencies]` of each crate's manifest (dev-dependencies are tests and do not shape
//! the shipped program). The rules are listed in docs/ARCHITECTURE.md, "Crate graph rules".

use raul_testkit::{covers, ids};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// The names a crate's manifest lists under `[dependencies]` and `[build-dependencies]`.
fn dependencies(crate_dir: &str) -> BTreeSet<String> {
    let path = repository().join(crate_dir).join("Cargo.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let doc: toml::Table = text.parse().unwrap();
    ["dependencies", "build-dependencies"]
        .iter()
        .filter_map(|section| doc.get(*section).and_then(toml::Value::as_table))
        .flat_map(|t| t.keys().cloned())
        .collect()
}

fn starting(set: &BTreeSet<String>, prefix: &str) -> BTreeSet<String> {
    set.iter()
        .filter(|n| n.starts_with(prefix))
        .cloned()
        .collect()
}

#[test]
fn the_runner_and_the_supply_depend_on_no_other_toolbox_crate() {
    covers(&[ids::guarantees::build::CRATE_GRAPH]);
    for dir in ["crates/raul-runner", "crates/raul-supply"] {
        if !repository().join(dir).join("Cargo.toml").exists() {
            // raul-supply is planned (REORG section 3) and not split out yet; the rule holds when it is.
            assert_ne!(dir, "crates/raul-runner", "the runner crate is gone");
            continue;
        }
        let found = starting(&dependencies(dir), "raul-");
        assert!(found.is_empty(), "{dir} depends on {found:?}");
    }
}

#[test]
fn the_view_model_depends_on_no_toolbox_crate_and_no_nvmod_crate() {
    covers(&[ids::guarantees::build::CRATE_GRAPH]);
    let deps = dependencies("crates/raul-gui/crates/raul-ui-model");
    let found: BTreeSet<_> = deps
        .iter()
        .filter(|n| n.starts_with("raul-") || n.starts_with("nvmod-"))
        .collect();
    assert!(found.is_empty(), "raul-ui-model depends on {found:?}");
}

/// The nvmod crates the window's own code still links, beside the service seam. That is debt: the rule is that the
/// window reaches nvmod only through `raul-ui-service`. The list may only shrink: a crate that leaves the window's
/// dependencies must leave this list too, and a new one fails the test.
const WINDOW_NVMOD_DEBT: [&str; 2] = ["nvmod-modinstall", "nvmod-run"];

#[test]
fn the_window_reaches_nvmod_only_through_the_service() {
    let found = starting(&dependencies("crates/raul-gui"), "nvmod-");
    let allowed: BTreeSet<String> = WINDOW_NVMOD_DEBT.iter().map(|s| (*s).to_owned()).collect();
    let new: Vec<_> = found.difference(&allowed).collect();
    assert!(
        new.is_empty(),
        "raul-gui links {new:?} directly: go through raul-ui-service"
    );
    let gone: Vec<_> = allowed.difference(&found).collect();
    assert!(
        gone.is_empty(),
        "raul-gui no longer links {gone:?}: remove it from WINDOW_NVMOD_DEBT"
    );
}

#[test]
fn the_service_seam_is_the_window_s_only_way_to_nvmod_beside_the_debt() {
    // The seam itself must exist and carry nvmod, or the rule above would hold vacuously.
    let service = dependencies("crates/raul-gui/crates/raul-ui-service");
    assert!(
        !starting(&service, "nvmod-").is_empty(),
        "raul-ui-service links no nvmod crate: the window has no way to nvmod"
    );
}
