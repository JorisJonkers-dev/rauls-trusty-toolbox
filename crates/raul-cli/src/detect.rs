// SPDX-License-Identifier: GPL-3.0-only
//! `raul detect`: the game copies on this machine, or the one in a folder the person picks.

use std::fmt::Write as _;
use std::io::Write;
use std::path::PathBuf;

use raul_core::detect::{Game, GameCopy, SearchRoots, detect, inspect};

/// The game copies `raul detect` reports: the one in `folder`, or every copy on this machine. The window's direct
/// job calls the same function.
#[must_use]
pub fn copies(folder: Option<&std::path::Path>) -> Vec<GameCopy> {
    match folder {
        Some(dir) => inspect(dir).into_iter().collect(),
        None => detect(&SearchRoots::for_this_machine()),
    }
}

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
    let copies = copies(folder.as_deref());
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

#[cfg(test)]
mod tests {
    #[test]
    fn a_folder_without_a_game_has_no_copies() {
        let dir = std::env::temp_dir().join(format!("raul-detect-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(super::copies(Some(&dir)), Vec::new());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
