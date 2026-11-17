// SPDX-License-Identifier: GPL-3.0-only
//! `CliService::ttw` reads the published `raul-events` fixtures, and refuses a stream that is cut short, that
//! disagrees with itself, or that is newer than this build.

use raul_ui_model::events::Status;
use raul_ui_service::dto::TtwReport;
use raul_ui_service::{CliService, Output, ScriptedExec, Service, TtwRequest};

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

fn output(stream: &str, code: i64) -> Output {
    Output {
        code: Some(code),
        stdout: stream.to_owned(),
        stderr: String::new(),
    }
}

#[test]
fn a_finished_job_output_reads_as_the_service_reads_it() {
    let full = fixture("ttw-success.ndjson");
    let report = TtwReport::from_output(&output(&full, 0)).unwrap();
    assert_eq!(report, answer(&full, 0).unwrap());
    let cut: String = full
        .lines()
        .filter(|l| !l.contains("\"finished\""))
        .collect::<Vec<_>>()
        .join("\n");
    let e = TtwReport::from_output(&output(&cut, 0))
        .unwrap_err()
        .view_error();
    assert!(e.what.contains("before it said how it ended"), "{e:?}");
    assert_eq!(e.repro, None, "the caller adds the command line");
    let odd = full.replace(
        "\"event\": \"finished\", \"status\": \"success\"",
        "\"event\": \"finished\", \"status\": \"failed\"",
    );
    assert!(TtwReport::from_output(&output(&odd, 4)).is_err());
    let none = TtwReport::from_output(&Output {
        code: Some(2),
        stdout: String::new(),
        stderr: "raul ttw: refused: no Instance".into(),
    })
    .unwrap_err()
    .view_error();
    assert_eq!(none.what, "no Instance");
    assert_eq!(none.repro, None);
}

#[test]
fn the_service_still_names_the_command_line() {
    let exec = ScriptedExec::new();
    exec.on_run(&[], 4, "raul ttw: it broke");
    let err = CliService::new(exec)
        .ttw(&TtwRequest::default())
        .unwrap_err()
        .view_error();
    assert_eq!(err.repro.as_deref(), Some("raul ttw --json"));
}
