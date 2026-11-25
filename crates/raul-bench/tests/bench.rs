// SPDX-License-Identifier: GPL-3.0-only
//! The benchmark suite and the comparison of two reports.

use raul_bench::{Measurement, Options, Report, compare, run_suite};
use raul_testkit::{covers, ids};

fn measurement(name: &str, median_ms: f64) -> Measurement {
    Measurement {
        name: name.into(),
        iterations: 5,
        median_ms,
        mean_ms: median_ms,
        min_ms: median_ms,
        max_ms: median_ms,
        unit_per_s: None,
    }
}

fn report(results: Vec<Measurement>) -> Report {
    Report {
        schema_version: 1,
        raul: "0.0.0".into(),
        os: "linux".into(),
        arch: "x86_64".into(),
        results,
        skipped: vec![],
    }
}

#[test]
fn the_quick_suite_measures_sorting_plugin_reading_and_mod_install() {
    covers(&[ids::bench::suite::QUICK_SUITE]);
    let r = run_suite(&Options { quick: true });
    let names: Vec<&str> = r.results.iter().map(|m| m.name.as_str()).collect();
    for want in [
        "sort/small",
        "sort/medium",
        "sort/ttw-sized",
        "plugin-headers/ttw-sized",
        "install/zip",
    ] {
        assert!(names.contains(&want), "{want} missing from {names:?}");
    }
    assert!(
        r.results
            .iter()
            .all(|m| m.iterations >= 1 && m.median_ms >= 0.0)
    );
}

#[test]
fn lod_and_frame_time_are_skipped_with_a_reason_when_their_tools_are_missing() {
    covers(&[ids::bench::suite::SKIP_REASON]);
    let r = run_suite(&Options { quick: true });
    let skipped: Vec<&str> = r.skipped.iter().map(|s| s.name.as_str()).collect();
    assert!(
        skipped.contains(&"lod/generate") && skipped.contains(&"frame-time/camera-path"),
        "{skipped:?}"
    );
    assert!(r.skipped.iter().all(|s| !s.reason.is_empty()));
}

#[test]
fn the_markdown_report_has_a_row_for_every_result_and_every_skip() {
    covers(&[ids::bench::suite::MARKDOWN_REPORT]);
    let mut r = report(vec![measurement("sort/small", 1.5)]);
    r.skipped.push(raul_bench::Skipped {
        name: "lod/generate".into(),
        reason: "no lodwright binary".into(),
    });
    let md = r.to_markdown();
    assert!(md.contains("| sort/small |"), "{md}");
    assert!(
        md.contains("lod/generate") && md.contains("no lodwright binary"),
        "{md}"
    );
}

#[test]
fn a_result_slower_than_the_threshold_is_a_regression() {
    covers(&[ids::bench::suite::REGRESSION_THRESHOLD]);
    let base = report(vec![measurement("sort/small", 100.0)]);
    let head = report(vec![measurement("sort/small", 120.0)]);
    let c = compare(&base, &head, 10.0);
    assert_eq!(c.regressions, vec!["sort/small".to_string()]);
}

#[test]
fn a_result_slower_by_less_than_the_threshold_is_not_a_regression() {
    let base = report(vec![measurement("sort/small", 100.0)]);
    let head = report(vec![measurement("sort/small", 105.0)]);
    assert_eq!(
        compare(&base, &head, 10.0).regressions,
        Vec::<String>::new()
    );
}

#[test]
fn the_comparison_shows_the_change_in_percent() {
    let base = report(vec![measurement("sort/small", 100.0)]);
    let head = report(vec![measurement("sort/small", 80.0)]);
    let md = compare(&base, &head, 10.0).to_markdown();
    assert!(md.contains("-20.0%"), "{md}");
}

#[test]
fn every_sort_case_sorts_successfully_before_it_is_timed() {
    let r = run_suite(&Options { quick: true });
    assert!(
        !r.skipped.iter().any(|s| s.name.starts_with("sort/")),
        "{:?}",
        r.skipped
    );
}
