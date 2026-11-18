// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox's own Contracts: the schema files in `contracts/` equal the ones generated from the Rust types, and
//! every fixture in `contracts/fixtures/<name>/` (a `.json` document, or a `.ndjson` stream of one per line) is
//! valid against its schema and reads back into its type unchanged. The `ttwright-result` documents the fixtures
//! carry are valid against the snapshot of ttwright-rs v0.3.0's schema in `contracts/snapshots/`, and every snapshot
//! equals the schema its pinned Component publishes.
//! `RAUL_BLESS_CONTRACTS=1 cargo test -p raul-core --test contracts` rewrites the schema files.

use std::path::{Path, PathBuf};

use raul_core::config::{ConfigDoc, Key};
use raul_core::contracts::OWN;
use raul_drivers::events::{Event, EventLine};
use raul_drivers::lod::LodResult;
use raul_drivers::ttw::TtwResult;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn contracts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts")
}

fn fixtures(name: &str) -> Vec<(PathBuf, Value)> {
    let dir = contracts_dir().join("fixtures").join(name);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json" || e == "ndjson"))
        .collect();
    files.sort();
    let mut found = Vec::new();
    for p in files {
        let text = std::fs::read_to_string(&p).unwrap();
        let parse = |t: &str| -> Value {
            serde_json::from_str(t).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        };
        if p.extension().is_some_and(|e| e == "ndjson") {
            found.extend(text.lines().map(|l| (p.clone(), parse(l))));
        } else {
            found.push((p.clone(), parse(&text)));
        }
    }
    found
}

fn validator(schema_text: &str) -> jsonschema::Validator {
    jsonschema::validator_for(&serde_json::from_str(schema_text).unwrap()).unwrap()
}

fn assert_valid(validator: &jsonschema::Validator, value: &Value, what: &str) {
    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{what}: {errors:?}");
}

fn reads_back<T: Serialize + DeserializeOwned>(name: &str) -> Vec<T> {
    fixtures(name)
        .into_iter()
        .map(|(path, value)| {
            let doc: T = serde_json::from_value(value.clone())
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert_eq!(
                serde_json::to_value(&doc).unwrap(),
                value,
                "{}",
                path.display()
            );
            doc
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
        let validator = validator(&(own.schema)());
        let found = fixtures(own.name);
        assert!(!found.is_empty(), "no fixture for {}", own.name);
        for (path, value) in found {
            assert_valid(&validator, &value, &path.display().to_string());
        }
    }
}

#[test]
fn config_fixtures_read_back_unchanged() {
    reads_back::<ConfigDoc>("raul-config");
    for (path, value) in fixtures("raul-config") {
        let doc: ConfigDoc = serde_json::from_value(value).unwrap();
        for entry in &doc.entries {
            assert_eq!(entry.read_only, entry.key.read_only(), "{}", path.display());
        }
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("list"))
        {
            let keys: Vec<Key> = doc.entries.iter().map(|e| e.key).collect();
            assert_eq!(keys, Key::ALL, "{}", path.display());
        }
    }
}

#[test]
fn every_event_stream_is_numbered_from_one_and_ends_with_finished() {
    let lines = reads_back::<EventLine>("raul-events");
    let streams: Vec<&[EventLine]> = lines
        .split_inclusive(|l| matches!(l.event, Event::Finished { .. }))
        .collect();
    assert!(streams.len() >= 4);
    let ttw = validator(&raul_drivers::ttw::schema());
    let lod = validator(&raul_drivers::lod::schema());
    for stream in streams {
        assert!(matches!(
            stream.last().unwrap().event,
            Event::Finished { .. }
        ));
        for (i, line) in stream.iter().enumerate() {
            assert_eq!(line.seq, i as u64 + 1, "{line:?}");
            if let Event::Result { result } = &line.event {
                match line.command.as_str() {
                    "ttw" => assert_valid(&ttw, result, "the result event of a ttw stream"),
                    "lod" => assert_valid(&lod, result, "the result event of a lod stream"),
                    other => panic!("no result schema for {other}"),
                }
                let Event::Finished { status, .. } = &stream.last().unwrap().event else {
                    unreachable!()
                };
                assert_eq!(result["status"], serde_json::to_value(status).unwrap());
                let Event::Finished {
                    code, resumable, ..
                } = &stream.last().unwrap().event
                else {
                    unreachable!()
                };
                // A command whose runs cannot be resumed has no `resumable` in its result.
                if let Some(r) = result.get("resumable") {
                    assert_eq!(r, resumable);
                } else {
                    assert!(!resumable);
                }
                assert_eq!(result["error"]["code"].as_str(), code.as_deref());
            }
        }
    }
}

#[test]
fn ttw_result_fixtures_read_back_and_carry_valid_ttwright_results() {
    let snapshot = std::fs::read_to_string(
        contracts_dir().join("snapshots/ttwright-rs/ttwright-result.v1.json"),
    )
    .unwrap();
    let ttwright = validator(&snapshot);
    for doc in reads_back::<TtwResult>("raul-ttw-result") {
        if let Some(inner) = &doc.ttwright {
            assert_valid(&ttwright, inner, "ttwright-result");
        }
        assert_eq!(
            doc.error.is_none(),
            doc.status == raul_drivers::events::Status::Success
        );
    }
}

#[test]
fn a_config_document_without_file_or_value_is_not_valid() {
    let schema: Value = serde_json::from_str(&raul_core::config::schema()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let (_, good) = fixtures("raul-config").remove(0);
    let mut no_file = good.clone();
    no_file.as_object_mut().unwrap().remove("file");
    assert!(!validator.is_valid(&no_file));
    let mut no_value = good;
    no_value["entries"][0]
        .as_object_mut()
        .unwrap()
        .remove("value");
    assert!(!validator.is_valid(&no_value));
}

#[test]
fn a_config_document_of_another_version_or_contract_is_not_valid() {
    let validator = validator(&raul_core::config::schema());
    let (_, good) = fixtures("raul-config").remove(0);
    let mut newer = good.clone();
    newer["schema_version"] = 2.into();
    assert!(!validator.is_valid(&newer));
    let mut other = good;
    other["contract"] = "ttwright-result".into();
    assert!(!validator.is_valid(&other));
}

#[test]
fn lod_result_fixtures_read_back_and_carry_valid_lodwright_results() {
    let snapshot = std::fs::read_to_string(
        contracts_dir().join("snapshots/lodwright-rs/lodwright-result.v1.json"),
    )
    .unwrap();
    let lodwright = validator(&snapshot);
    for doc in reads_back::<LodResult>("raul-lod-result") {
        if let Some(inner) = &doc.lodwright {
            assert_valid(&lodwright, inner, "lodwright-result");
        }
        assert_eq!(
            doc.error.is_none(),
            doc.status == raul_drivers::events::Status::Success
        );
        assert!(
            !doc.up_to_date
                || doc
                    .lodwright
                    .as_ref()
                    .is_some_and(|l| l["code"] == "up_to_date")
        );
    }
}

/// Where each pinned Component keeps the schemas it attaches to its releases.
const SNAPSHOT_SOURCES: [(&str, &str); 2] = [
    ("lodwright-rs", "components/lodwright-rs/schemas"),
    ("ttwright-rs", "components/ttwright-rs/contracts"),
];

#[test]
fn every_snapshot_equals_the_schema_of_the_pinned_component() {
    let root = contracts_dir().join("..");
    let text = |p: &Path| {
        std::fs::read_to_string(p)
            .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
            .replace("\r\n", "\n")
    };
    let snapshots = contracts_dir().join("snapshots");
    let mut dirs: Vec<String> = std::fs::read_dir(&snapshots)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    dirs.sort();
    let mut known: Vec<String> = SNAPSHOT_SOURCES.iter().map(|(c, _)| (*c).into()).collect();
    known.sort();
    assert_eq!(dirs, known, "a snapshot folder without a source here");
    for (component, source) in SNAPSHOT_SOURCES {
        let dir = snapshots.join(component);
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        files.sort();
        assert!(!files.is_empty(), "{}: no snapshot", dir.display());
        for snapshot in files {
            let published = root.join(source).join(snapshot.file_name().unwrap());
            assert_eq!(
                text(&snapshot),
                text(&published),
                "{} differs from {} at the pinned release",
                snapshot.display(),
                published.display()
            );
        }
    }
}

#[test]
fn journal_fixtures_read_back_and_the_old_version_is_not_valid() {
    use raul_core::journal::JournalDoc;
    for doc in reads_back::<JournalDoc>("raul-journal") {
        assert!(doc.finished.iter().all(|f| doc.plan.contains(f)));
        assert_eq!(doc.complete, doc.finished.len() == doc.plan.len());
    }
    let validator = validator(&raul_core::journal::schema());
    for (path, old) in fixtures("raul-journal-v0") {
        assert!(!validator.is_valid(&old), "{}", path.display());
    }
    let (_, good) = fixtures("raul-journal").remove(0);
    let mut newer = good;
    newer["schema_version"] = 2.into();
    assert!(!validator.is_valid(&newer));
}
