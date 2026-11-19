// SPDX-License-Identifier: GPL-3.0-only
//! `raul update`: check the Toolbox's GitHub Releases for a newer release when asked, and turn the GUI's check at
//! start on or off. Nothing is downloaded or installed, and nothing identifying is sent (ADR-0005).

use std::io::Write;
use std::path::Path;

use raul_core::update::{self, Settings};

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let Some(path) = Settings::default_path() else {
        let _ = writeln!(
            err,
            "raul update: no configuration folder (HOME, XDG_CONFIG_HOME or APPDATA is not set)"
        );
        return crate::FOUND;
    };
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
            // The file holds the app's other preferences too; they stay as they are.
            let settings = Settings {
                check_for_updates: *value == "on",
                ..Settings::load(&path)
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
    let current = env!("CARGO_PKG_VERSION");
    // `curl` is found on the `PATH`; the Runner passes only that, the proxy and certificate variables, and limits.
    let result = update::check(&update::runner(), Path::new("curl"), current);
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
