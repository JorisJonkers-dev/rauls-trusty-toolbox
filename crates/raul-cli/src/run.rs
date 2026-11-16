// SPDX-License-Identifier: GPL-3.0-only
//! `raul run <nvmod run arguments>`: a game run, which the Toolbox starts only with the engine's memory-capped run
//! guard (#115). Without a guard that is on, the run is refused with exit 2 and nothing is started; no flag or
//! variable changes that. `--dry-run` names the guard, its caps and nvmod's plan of the run.

use std::io::Write;
use std::path::PathBuf;

use raul_drivers::run::{self, GUARD};
use raul_drivers::ttw::bundled;
use serde_json::{Value, json};

/// The guard bundled with the engine: `components/nv-rs/nv-rs-guard` next to `raul`.
fn bundled_guard() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    bundled(exe.parent()?, &format!("nv-rs/{GUARD}"))
}

fn answer(out: &mut dyn Write, err: &mut dyn Write, json: bool, doc: &Value, text: &str) {
    if json {
        let _ = writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(doc).unwrap_or_default()
        );
    } else {
        let _ = writeln!(err, "{text}");
    }
}

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (own, extra) = match args.iter().position(|a| a == "--") {
        Some(i) => (&args[..i], &args[i..]),
        None => (args, &args[args.len()..]),
    };
    let mut guard = None;
    let mut nvmod: Vec<String> = Vec::new();
    let mut rest = own.iter();
    while let Some(a) = rest.next() {
        if a == "--guard" {
            match rest.next() {
                Some(p) => guard = Some(PathBuf::from(p)),
                None => return crate::usage(err, "--guard needs a path"),
            }
        } else if let Some(p) = a.strip_prefix("--guard=") {
            guard = Some(PathBuf::from(p));
        } else {
            nvmod.push(a.clone());
        }
    }
    let json = nvmod.iter().any(|a| a == "--json");
    let dry_run = nvmod.iter().any(|a| a == "--dry-run");
    let guard = guard
        .map(|p| std::path::absolute(&p).unwrap_or(p))
        .or_else(bundled_guard);
    let checked = match run::check(guard.as_deref()) {
        Ok(g) => g,
        Err(reason) => {
            let doc = json!({
                "schema_version": 1, "contract": "raul-run", "status": "refused",
                "started": false, "reason": reason,
            });
            answer(
                out,
                err,
                json,
                &doc,
                &format!("raul run: refused: {reason}"),
            );
            return crate::REFUSED;
        }
    };
    if !dry_run {
        let reason = "the guard is on, but starting a game run through it comes with the engine's guard \
                      release; nothing was started";
        let doc = json!({
            "schema_version": 1, "contract": "raul-run", "status": "refused", "started": false,
            "reason": reason, "guard": checked,
        });
        answer(
            out,
            err,
            json,
            &doc,
            &format!("raul run: refused: {reason}"),
        );
        return crate::REFUSED;
    }
    // nvmod's own plan of the run, read only; the caps are what the guard will hold the run to.
    let mut plan_args = vec!["run".to_owned()];
    plan_args.extend(nvmod.iter().filter(|a| *a != "--json").cloned());
    plan_args.push("--json".to_owned());
    plan_args.extend(extra.iter().cloned());
    let (mut plan_out, mut plan_err) = (Vec::new(), Vec::new());
    let code = nvmod_cli::run(&plan_args, &mut plan_out, &mut plan_err);
    let plan: Value = serde_json::from_slice(&plan_out).unwrap_or(Value::Null);
    let doc = json!({
        "schema_version": 1, "contract": "raul-run", "status": "dry_run", "started": false,
        "guard": checked, "caps": checked.caps, "plan": plan,
        "plan_error": (code != crate::OK).then(|| String::from_utf8_lossy(&plan_err).trim().to_owned()),
    });
    let caps = checked.caps;
    answer(
        out,
        err,
        json,
        &doc,
        &format!(
            "raul run: dry run; the guard {} holds the run to {} MiB of memory, {} MiB of VRAM and {} MiB of GTT",
            checked.path.display(),
            caps.host_mib,
            caps.vram_mib,
            caps.gtt_mib
        ),
    );
    let _ = code;
    crate::OK
}
