// SPDX-License-Identifier: GPL-3.0-only
//! `raul version`: the Toolbox version of this build.

use std::io::Write;

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return crate::usage(err, "version takes only --json"),
    };
    if json {
        let v = document();
        return crate::write_or_fail(out, err, &format!("{v}\n"));
    }
    crate::write_or_fail(out, err, &format!("raul {}\n", env!("CARGO_PKG_VERSION")))
}

/// The document `raul version --json` prints; the window's direct job prints the same one.
#[must_use]
pub fn document() -> serde_json::Value {
    serde_json::json!({ "raul": env!("CARGO_PKG_VERSION") })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_document_names_this_build() {
        assert_eq!(
            super::document(),
            serde_json::json!({ "raul": env!("CARGO_PKG_VERSION") })
        );
    }
}
