// SPDX-License-Identifier: GPL-3.0-only
//! Entry point of the CLI-only `raul` (ADR-0004): arguments in, `raul_cli::run` does the work.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(raul_cli::run(
        &args,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ))
}
