// SPDX-License-Identifier: GPL-3.0-only
//! `raul target <vanilla|nv-rs> --instance <dir>`: nvmod's command, and, when it made nv-rs the target, the engine's
//! capabilities written into the Instance for the Doctor (#398). The engine is linked into `raul`, so no package
//! brings that file.

use std::io::Write;
use std::path::PathBuf;

use crate::FAILED;

/// The Instance folder named by `--instance <dir>`, the only spelling nvmod's `target` reads.
fn instance_of(args: &[String]) -> Option<PathBuf> {
    let at = args.iter().position(|a| a == "--instance")?;
    args.get(at + 1).map(PathBuf::from)
}

/// Runs `args` (with `target` first) as nvmod does, then writes the capabilities when nv-rs became the target.
pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let code = nvmod_cli::run_logged(args, out, err);
    if code != crate::OK || args.get(1).map(String::as_str) != Some("nv-rs") {
        return code;
    }
    let Some(instance) = instance_of(&args[2..]) else {
        return code;
    };
    match raul_core::capabilities::write_provided(&instance) {
        Ok(_) => code,
        Err(e) => {
            let _ = writeln!(
                err,
                "raul target: the engine's capabilities could not be written to {}: {e}",
                instance.display()
            );
            FAILED
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        text.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn the_instance_is_the_word_after_the_flag() {
        assert_eq!(
            instance_of(&words("--instance /i --x")),
            Some(PathBuf::from("/i"))
        );
        assert_eq!(instance_of(&words("--instance")), None);
        assert_eq!(instance_of(&words("nv-rs")), None);
    }
}
