// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox's own Contracts: the schema files in `contracts/` equal the ones generated from the Rust types, and
//! every fixture in `contracts/fixtures/<name>/` is valid against its schema and reads back into its type unchanged.
//! `RAUL_BLESS_CONTRACTS=1 cargo test -p raul-core --test contracts` rewrites the schema files.

use std::path::{Path, PathBuf};

use raul_core::config::ConfigDoc;
use raul_core::contracts::OWN;
use serde_json::Value;

fn contracts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts")
}

fn fixtures(name: &str) -> Vec<(PathBuf, Value)> {
    let dir = contracts_dir().join("fixtures").join(name);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap();
            let value =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            (p, value)
        })
        .collect()
}

#[test]
fn every_schema_file_equals_the_generated_schema() {
    for own in OWN {
        let path = contracts_dir().join(own.file_name());
        let generated = (own.schema)();
        if std::env::var_os("RAUL_BLESS_CONTRACTS").is_some() {
            std::fs::write(&path, &generated).unwrap();
        }
        let committed =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(
            committed.replace("\r\n", "\n"),
            generated,
            "{} differs from the type; run with RAUL_BLESS_CONTRACTS=1 and review the change",
            own.file_name()
        );
    }
}

#[test]
fn every_fixture_is_valid_against_its_schema() {
    for own in OWN {
        let schema: Value = serde_json::from_str(&(own.schema)()).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let found = fixtures(own.name);
        assert!(!found.is_empty(), "no fixture for {}", own.name);
        for (path, value) in found {
            let errors: Vec<String> = validator
                .iter_errors(&value)
                .map(|e| e.to_string())
                .collect();
            assert!(errors.is_empty(), "{}: {errors:?}", path.display());
        }
    }
}

#[test]
fn config_fixtures_read_back_unchanged() {
    for (path, value) in fixtures("raul-config") {
        let doc: ConfigDoc = serde_json::from_value(value.clone())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(
            serde_json::to_value(&doc).unwrap(),
            value,
            "{}",
            path.display()
        );
    }
}

#[test]
fn a_config_document_of_another_version_or_contract_is_not_valid() {
    let schema: Value = serde_json::from_str(&raul_core::config::schema()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let (_, good) = fixtures("raul-config").remove(0);
    let mut newer = good.clone();
    newer["schema_version"] = 2.into();
    assert!(!validator.is_valid(&newer));
    let mut other = good;
    other["contract"] = "ttwright-result".into();
    assert!(!validator.is_valid(&other));
}
