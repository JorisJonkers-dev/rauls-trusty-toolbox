// SPDX-License-Identifier: GPL-3.0-only
//! `raul version`: the Toolbox version and the Component versions of the Known-good set this build ships.

use std::fmt::Write as _;
use std::io::Write;

use raul_core::known_good::{COMPONENTS, shipped};

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return crate::usage(err, "version takes only --json"),
    };
    let kg = shipped();
    let Some(set) = kg.latest_known_good() else {
        return crate::usage(err, "this build has no known-good set");
    };
    let version = env!("CARGO_PKG_VERSION");
    if json {
        let components: serde_json::Map<String, serde_json::Value> = COMPONENTS
            .iter()
            .map(|c| ((*c).to_string(), set.components[*c].version.clone().into()))
            .collect();
        let v = serde_json::json!({ "raul": version, "known_good_set": set.id, "components": components });
        return crate::write_or_fail(out, err, &format!("{v}\n"));
    }
    let mut text = format!("raul {version}\nknown-good set {}\n", set.id);
    for c in COMPONENTS {
        let _ = writeln!(text, "  {c:<13} {}", set.components[c].version);
    }
    crate::write_or_fail(out, err, &text)
}
