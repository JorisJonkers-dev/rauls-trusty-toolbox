// SPDX-License-Identifier: GPL-3.0-only
//! `raul space`: the free space on the storage a folder is on, read only.

use std::io::Write;
use std::path::PathBuf;

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let mut json = false;
    let mut folder = None;
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--json" => json = true,
            "--folder" => match rest.next() {
                Some(dir) => folder = Some(PathBuf::from(dir)),
                None => return crate::usage(err, "--folder needs a folder"),
            },
            other => return crate::usage(err, &format!("unknown option {other}")),
        }
    }
    let Some(folder) = folder else {
        return crate::usage(err, "space needs --folder <dir>");
    };
    match raul_core::space::free_space(&folder) {
        Ok(space) if json => {
            let text = serde_json::to_string(&space).unwrap_or_default();
            crate::write_or_fail(out, err, &format!("{text}\n"))
        }
        Ok(space) => crate::write_or_fail(
            out,
            err,
            &format!(
                "{} bytes free of {} on the storage of {}\n",
                space.free_bytes,
                space.total_bytes,
                space.folder.display()
            ),
        ),
        Err(why) => {
            let _ = writeln!(err, "raul: {}: {why}", folder.display());
            crate::FOUND
        }
    }
}
