// SPDX-License-Identifier: GPL-3.0-only
//! `raul detect`: the game copies on this machine, or the one in a folder the person picks.

use std::fmt::Write as _;
use std::io::Write;
use std::path::PathBuf;

use raul_core::detect::{Game, GameCopy, SearchRoots, detect, inspect};

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
    let copies: Vec<GameCopy> = match &folder {
        Some(dir) => inspect(dir).into_iter().collect(),
        None => detect(&SearchRoots::for_this_machine()),
    };
    let code = if copies.is_empty() {
        crate::FOUND
    } else {
        crate::OK
    };
    if json {
        let text = serde_json::to_string(&copies).unwrap_or_else(|_| "[]".into());
        crate::write_or_fail(out, err, &format!("{text}\n"));
        return code;
    }
    if copies.is_empty() {
        let _ = writeln!(
            err,
            "no game copy found{}",
            folder.map_or_else(String::new, |f| format!(" in {}", f.display()))
        );
        return code;
    }
    let mut text = String::new();
    for c in &copies {
        let game = match c.game {
            Game::NewVegas => "Fallout: New Vegas",
            Game::Fallout3 => "Fallout 3",
        };
        let version = c.version.as_deref().unwrap_or("unknown version");
        let support = if c.supported_version == Some(false) {
            " (unsupported version)"
        } else {
            ""
        };
        let _ = writeln!(
            text,
            "{game} {version}{support}, {}: {}",
            c.storefront,
            c.path.display()
        );
        if !c.missing_dlc.is_empty() {
            let _ = writeln!(text, "  missing DLC: {}", c.missing_dlc.join(", "));
        }
    }
    crate::write_or_fail(out, err, &text);
    code
}
