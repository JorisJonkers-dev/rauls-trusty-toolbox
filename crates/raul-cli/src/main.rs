// SPDX-License-Identifier: GPL-3.0-only
//! Entry point of the CLI-only `raul` (ADR-0004): arguments in, `raul_cli::run` does the work.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The play mode is the full raul's (ADR-0016); this build has no engine.
    if args.first().is_some_and(|a| a == "--play-mode") {
        eprintln!("raul: this build does not include the game; use the full raul");
        return ExitCode::from(2);
    }
    if let Some(code) = raul_cli::settle_update_at_start(&args, &mut std::io::stderr()) {
        return ExitCode::from(code);
    }
    ExitCode::from(raul_cli::run(
        &args,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ))
}
