// SPDX-License-Identifier: GPL-3.0-only
//! `raul update`: check the Toolbox's GitHub Releases for a newer release when asked, and turn the GUI's check at
//! start on or off. Nothing is downloaded or installed, and nothing identifying is sent (nv-linux ADR-0010).

use std::io::Write;
use std::path::PathBuf;

use nvmod_tools::{Curl, Fetcher};
use raul_core::update::{RELEASES_URL, Settings, newer_release};

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let path = Settings::default_path();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["check"] => check(false, out, err),
        ["check", "--json"] => check(true, out, err),
        ["auto"] => {
            let on = Settings::load(&path).check_for_updates;
            crate::write_or_fail(
                out,
                err,
                &format!("update check at start: {}\n", if on { "on" } else { "off" }),
            )
        }
        ["auto", value @ ("on" | "off")] => {
            let settings = Settings {
                check_for_updates: *value == "on",
            };
            match settings.save(&path) {
                Ok(()) => {
                    crate::write_or_fail(out, err, &format!("update check at start: {value}\n"))
                }
                Err(e) => {
                    let _ = writeln!(err, "raul update: {}: {e}", path.display());
                    crate::FOUND
                }
            }
        }
        _ => crate::usage(err, "update takes `check [--json]` or `auto [on|off]`"),
    }
}

fn check(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let curl = Curl {
        program: PathBuf::from("curl"),
    };
    let current = env!("CARGO_PKG_VERSION");
    let result = curl
        .get(RELEASES_URL)
        .map_err(|e| format!("{e:?}"))
        .and_then(|bytes| newer_release(current, &String::from_utf8_lossy(&bytes)));
    match result {
        Ok(newer) if json => {
            let v = serde_json::json!({ "current": current, "newer": newer });
            crate::write_or_fail(out, err, &format!("{v}\n"))
        }
        Ok(Some(r)) => crate::write_or_fail(
            out,
            err,
            &format!("raul {} is out (you run {current}): {}\n", r.version, r.url),
        ),
        Ok(None) => {
            crate::write_or_fail(out, err, &format!("raul {current} is the newest release\n"))
        }
        Err(e) => {
            let _ = writeln!(err, "raul update check: {e}");
            crate::FOUND
        }
    }
}
