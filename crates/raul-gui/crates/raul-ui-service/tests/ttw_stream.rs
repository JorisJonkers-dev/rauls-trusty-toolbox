// SPDX-License-Identifier: GPL-3.0-only
//! `CliService::ttw` reads the published `raul-events` fixtures, and refuses a stream that is cut short, that
//! disagrees with itself, or that is newer than this build.

use raul_ui_model::events::Status;
use raul_ui_service::{CliService, ScriptedExec, Service, TtwRequest};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../contracts/fixtures/raul-events")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn answer(stream: &str, code: i64) -> Result<raul_ui_service::dto::TtwReport, String> {
    let exec = ScriptedExec::new();
    let lines: Vec<&str> = stream.lines().collect();
    exec.on_run(&lines, code, "");
    CliService::new(exec)
        .ttw(&TtwRequest::default())
        .map_err(|e| e.view_error().what)
}

#[test]
fn every_published_ttw_stream_is_a_report_with_its_status() {
    for (name, code, status) in [
        ("ttw-success.ndjson", 0, Status::Success),
        ("ttw-refused.ndjson", 2, Status::Refused),
        ("ttw-unsupported.ndjson", 2, Status::Unsupported),
        ("ttw-cancelled.ndjson", 4, Status::Cancelled),
    ] {
        let report = answer(&fixture(name), code).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(report.status, status, "{name}");
    }
}

#[test]
fn a_stream_cut_short_or_at_odds_with_itself_is_not_a_report() {
    let full = fixture("ttw-success.ndjson");
    let cut: String = full
        .lines()
        .filter(|l| !l.contains("\"finished\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        answer(&cut, 0)
            .unwrap_err()
            .contains("before it said how it ended")
    );
    let odd = full.replace(
        "\"event\": \"finished\", \"status\": \"success\"",
        "\"event\": \"finished\", \"status\": \"failed\"",
    );
    assert_ne!(odd, full, "the fixture's finished line changed shape");
    assert!(answer(&odd, 4).unwrap_err().contains("ended as"));
    let newer = full.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1);
    assert!(answer(&newer, 0).unwrap_err().contains("version 2"));
    assert!(answer("", 4).is_err(), "no result at all");
}
