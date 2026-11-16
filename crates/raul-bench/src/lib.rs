// SPDX-License-Identifier: GPL-3.0-only
//! The `raul bench` suite (docs/PLAN.md, Phase 4). Every fixture is synthetic, built by nvmod's test kit: no game
//! data is read or kept. Results are wall-clock medians on the machine that runs them, so two reports are only
//! comparable from the same machine. Shared CI runners are noisy, so `compare` reports regressions as warnings,
//! never as failures.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use loot_rs::api::{Masterlist, PluginSlot, SortOptions, sort};
use nvmod_testkit::{BaseBuilder, PluginBuilder, TempDir, ZipBuilder, write_file};
use serde::{Deserialize, Serialize};

/// What to run.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Fewer iterations, for tests and pull-request CI.
    pub quick: bool,
    /// A lodwright binary; without one the LOD case is skipped.
    pub lodwright: Option<PathBuf>,
}

/// One measured case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    /// Stable case name, such as `sort/medium`.
    pub name: String,
    /// How many timed runs.
    pub iterations: u32,
    /// Median wall-clock time of one run.
    pub median_ms: f64,
    /// Mean wall-clock time of one run.
    pub mean_ms: f64,
    /// Fastest run.
    pub min_ms: f64,
    /// Slowest run.
    pub max_ms: f64,
    /// Throughput at the median, when the case has one, with its unit (`plugins`, `MiB`).
    pub unit_per_s: Option<(f64, String)>,
}

/// A case that did not run, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skipped {
    /// The case name.
    pub name: String,
    /// Why it did not run.
    pub reason: String,
}

/// The whole report, written as `bench.json` and `bench.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Version of this layout.
    pub schema_version: u32,
    /// The `raul` version that ran it.
    pub raul: String,
    /// Operating system.
    pub os: String,
    /// CPU architecture.
    pub arch: String,
    /// The measured cases.
    pub results: Vec<Measurement>,
    /// The cases that did not run.
    pub skipped: Vec<Skipped>,
}

/// Two reports side by side.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    /// Case name, base median, head median, change in percent.
    pub rows: Vec<(String, f64, f64, f64)>,
    /// Cases whose median got slower by more than the threshold.
    pub regressions: Vec<String>,
    /// The threshold in percent.
    pub threshold_pct: f64,
}

const SIZES: [(&str, usize); 3] = [("small", 60), ("medium", 300), ("ttw-sized", 1000)];
const INSTALL_BYTES: usize = 8 * 1024 * 1024;

/// Runs the standard suite.
///
/// # Panics
///
/// Never: an empty masterlist always parses (loot-rs tests it).
#[must_use]
pub fn run_suite(options: &Options) -> Report {
    let iterations = if options.quick { 3 } else { 15 };
    let mut results = Vec::new();
    let mut skipped = Vec::new();
    let masterlist = Masterlist::parse("").expect("an empty masterlist is valid");
    for (label, count) in SIZES {
        let data = TempDir::new(&format!("raul-bench-{label}"));
        let slots = write_plugins(data.path(), count);
        let mut m = time(&format!("sort/{label}"), iterations, || {
            let _ = sort(&slots, &masterlist, data.path(), &SortOptions::default());
        });
        m.unit_per_s = Some((per_second(count, m.median_ms), "plugins".into()));
        results.push(m);
        if label == "ttw-sized" {
            let mut m = time("plugin-headers/ttw-sized", iterations, || {
                for s in &slots {
                    let _ = std::fs::read(data.path().join(&s.name)).map(|b| b.len());
                }
            });
            m.unit_per_s = Some((per_second(count, m.median_ms), "plugins".into()));
            results.push(m);
        }
    }
    match install_case(iterations) {
        Ok(m) => results.push(m),
        Err(reason) => skipped.push(Skipped {
            name: "install/zip".into(),
            reason,
        }),
    }
    skipped.push(Skipped {
        name: "lod/generate".into(),
        reason: match &options.lodwright {
            None => "no lodwright binary given (--lodwright <path>)".into(),
            Some(p) => format!("{}: no synthetic lodwright plan fixture yet", p.display()),
        },
    });
    skipped.push(Skipped {
        name: "frame-time/camera-path".into(),
        reason: "nv-rs has no scripted camera path or frame-time log yet".into(),
    });
    Report {
        schema_version: 1,
        raul: env!("CARGO_PKG_VERSION").into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        results,
        skipped,
    }
}

/// Compares `head` with `base`; a case slower by more than `threshold_pct` percent is a regression.
#[must_use]
pub fn compare(base: &Report, head: &Report, threshold_pct: f64) -> Comparison {
    let mut rows = Vec::new();
    let mut regressions = Vec::new();
    for h in &head.results {
        let Some(b) = base.results.iter().find(|b| b.name == h.name) else {
            continue;
        };
        let change = if b.median_ms > 0.0 {
            (h.median_ms - b.median_ms) / b.median_ms * 100.0
        } else {
            0.0
        };
        if change > threshold_pct {
            regressions.push(h.name.clone());
        }
        rows.push((h.name.clone(), b.median_ms, h.median_ms, change));
    }
    Comparison {
        rows,
        regressions,
        threshold_pct,
    }
}

impl Report {
    /// The report as Markdown.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# raul bench\n\nraul {} on {} {}\n\n| case | runs | median ms | min ms | max ms | throughput |\n|---|---|---|---|---|---|\n",
            self.raul, self.os, self.arch
        );
        for m in &self.results {
            let rate = m
                .unit_per_s
                .as_ref()
                .map_or_else(String::new, |(v, u)| format!("{v:.0} {u}/s"));
            let _ = writeln!(
                md,
                "| {} | {} | {:.3} | {:.3} | {:.3} | {rate} |",
                m.name, m.iterations, m.median_ms, m.min_ms, m.max_ms
            );
        }
        if !self.skipped.is_empty() {
            md.push_str("\n## Skipped\n\n");
            for s in &self.skipped {
                let _ = writeln!(md, "- {}: {}", s.name, s.reason);
            }
        }
        md
    }
}

impl Comparison {
    /// The comparison as Markdown, with regressions marked.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::from("| case | base ms | head ms | change |\n|---|---|---|---|\n");
        for (name, b, h, change) in &self.rows {
            let mark = if self.regressions.contains(name) {
                " (slower)"
            } else {
                ""
            };
            let _ = writeln!(md, "| {name} | {b:.3} | {h:.3} | {change:+.1}%{mark} |");
        }
        if self.regressions.is_empty() {
            let _ = writeln!(
                md,
                "\nNo case is slower by more than {:.0}%.",
                self.threshold_pct
            );
        } else {
            let _ = writeln!(
                md,
                "\nWarning: {} slower by more than {:.0}%. Shared runners are noisy; rerun before acting.",
                self.regressions.join(", "),
                self.threshold_pct
            );
        }
        md
    }
}

fn per_second(count: usize, median_ms: f64) -> f64 {
    if median_ms > 0.0 {
        f64::from(u32::try_from(count).unwrap_or(u32::MAX)) / (median_ms / 1000.0)
    } else {
        0.0
    }
}

fn time(name: &str, iterations: u32, mut f: impl FnMut()) -> Measurement {
    f();
    let mut runs: Vec<f64> = (0..iterations.max(1))
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    runs.sort_by(f64::total_cmp);
    let mean = runs.iter().sum::<f64>() / f64::from(iterations.max(1));
    Measurement {
        name: name.into(),
        iterations: iterations.max(1),
        median_ms: runs[runs.len() / 2],
        mean_ms: mean,
        min_ms: runs[0],
        max_ms: runs[runs.len() - 1],
        unit_per_s: None,
    }
}

/// `count` synthetic plugins: one master, then plugins that each need it and, every tenth, the one before.
fn write_plugins(dir: &Path, count: usize) -> Vec<PluginSlot> {
    let mut slots = vec![PluginSlot::new("Synthetic.esm", true)];
    write_file(dir, "Synthetic.esm", &PluginBuilder::master().build());
    for i in 1..count {
        let name = format!("Plugin{i:04}.esp");
        let mut b = PluginBuilder::new().requires("Synthetic.esm");
        if i % 10 == 0 {
            b = b.requires(&format!("Plugin{:04}.esp", i - 1));
        }
        write_file(dir, &name, &b.build());
        slots.push(PluginSlot::new(name, true));
    }
    slots
}

/// Installs a synthetic 8 MiB zip into a fresh Instance through nvmod's `init`, `profile create` and `install`, once
/// per run.
fn install_case(iterations: u32) -> Result<Measurement, String> {
    let work = TempDir::new("raul-bench-install");
    let base = work.path().join("base");
    BaseBuilder::steam()
        .exe_version([1, 4, 0, 525])
        .write(&base);
    let payload: Vec<u8> = (0..INSTALL_BYTES)
        .map(|i| u8::try_from(i % 251).unwrap_or(0))
        .collect();
    let archive = work.path().join("mod.zip");
    std::fs::write(
        &archive,
        ZipBuilder::new()
            .file("Data/textures/bench.dds", &payload)
            .bytes(),
    )
    .map_err(|e| e.to_string())?;
    let run = |args: &[&str]| {
        let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
        let mut err = Vec::new();
        let code = nvmod_cli::run(&args, &mut std::io::sink(), &mut err);
        (code, String::from_utf8_lossy(&err).trim().to_string())
    };
    let (b, a) = (
        base.to_string_lossy().into_owned(),
        archive.to_string_lossy().into_owned(),
    );
    let mut n = 0u32;
    let mut failure = None;
    let mut m = time("install/zip", iterations, || {
        n += 1;
        let i = work
            .path()
            .join(format!("instance-{n}"))
            .to_string_lossy()
            .into_owned();
        for step in [
            vec!["init", "--base", b.as_str(), "--instance", i.as_str()],
            vec!["profile", "create", "--instance", i.as_str()],
            vec!["install", a.as_str(), "--instance", i.as_str()],
        ] {
            let (code, err) = run(&step);
            if code != nvmod_cli::OK && failure.is_none() {
                failure = Some(format!("nvmod {} exited {code}: {err}", step[0]));
            }
        }
    });
    if let Some(reason) = failure {
        return Err(reason);
    }
    m.unit_per_s = Some((8.0 / (m.median_ms / 1000.0), "MiB".into()));
    Ok(m)
}
