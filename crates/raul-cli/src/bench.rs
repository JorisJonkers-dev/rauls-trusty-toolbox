// SPDX-License-Identifier: GPL-3.0-only
//! `raul bench`: run the standard suite and write `bench.json` and `bench.md`, or compare two reports.

use std::io::Write;
use std::path::PathBuf;

use raul_bench::{Options, Report, compare, run_suite};

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    if args.first().map(String::as_str) == Some("compare") {
        return compare_command(&args[1..], out, err);
    }
    let mut options = Options::default();
    let mut out_dir = PathBuf::from(".");
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--quick" => options.quick = true,
            "--out" | "--lodwright" => {
                let Some(value) = rest.next() else {
                    return crate::usage(err, &format!("{a} needs a path"));
                };
                if a == "--out" {
                    out_dir = PathBuf::from(value);
                } else {
                    options.lodwright = Some(PathBuf::from(value));
                }
            }
            other => return crate::usage(err, &format!("unknown option {other}")),
        }
    }
    let report = run_suite(&options);
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    let written = std::fs::create_dir_all(&out_dir)
        .and_then(|()| std::fs::write(out_dir.join("bench.json"), format!("{json}\n")))
        .and_then(|()| std::fs::write(out_dir.join("bench.md"), report.to_markdown()));
    if let Err(e) = written {
        let _ = writeln!(err, "raul bench: {}: {e}", out_dir.display());
        return crate::FOUND;
    }
    crate::write_or_fail(out, err, &report.to_markdown())
}

fn compare_command(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let mut threshold = 10.0;
    let mut files = Vec::new();
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        if a == "--threshold" {
            match rest.next().and_then(|v| v.parse::<f64>().ok()) {
                Some(t) => threshold = t,
                None => return crate::usage(err, "--threshold needs a number (percent)"),
            }
        } else {
            files.push(PathBuf::from(a));
        }
    }
    let [base, head] = files.as_slice() else {
        return crate::usage(err, "bench compare needs <base.json> <head.json>");
    };
    let read = |p: &PathBuf| -> Result<Report, String> {
        let bytes = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", p.display()))
    };
    match (read(base), read(head)) {
        (Ok(b), Ok(h)) => crate::write_or_fail(out, err, &compare(&b, &h, threshold).to_markdown()),
        (Err(e), _) | (_, Err(e)) => {
            let _ = writeln!(err, "raul bench compare: {e}");
            crate::FOUND
        }
    }
}
