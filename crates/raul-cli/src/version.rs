// SPDX-License-Identifier: GPL-3.0-only
//! `raul version`: the Toolbox version of this build.

use std::io::Write;

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return crate::usage(err, "version takes only --json"),
    };
    let version = env!("CARGO_PKG_VERSION");
    if json {
        let v = serde_json::json!({ "raul": version });
        return crate::write_or_fail(out, err, &format!("{v}\n"));
    }
    crate::write_or_fail(out, err, &format!("raul {version}\n"))
}
