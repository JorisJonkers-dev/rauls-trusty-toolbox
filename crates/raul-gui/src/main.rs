// SPDX-License-Identifier: GPL-3.0-only
//! Entry point of the release `raul` (ADR-0004): with no arguments it opens the window, with any it runs the same
//! `raul_cli::run` as the command-line-only build.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = raul_cli::settle_update_at_start(&args, &mut std::io::stderr()) {
        return ExitCode::from(code);
    }
    if args.is_empty() {
        return raul_gui::open();
    }
    ExitCode::from(raul_cli::run(
        &args,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ))
}
