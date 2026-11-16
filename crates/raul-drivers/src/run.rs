// SPDX-License-Identifier: GPL-3.0-only
//! The memory-capped run guard (#115): every game run the Toolbox starts needs the engine's guard, which caps host
//! and GPU memory before and during the run and stops it when the desktop session is in trouble. This module asks the
//! guard whether it is on and what its caps are; a missing guard, one that is off, or one that answers anything but
//! a `nv-rs-run-guard` v1 document refuses the run. There is no way around it: no flag or variable skips the check.
//!
//! The guard is built in nv-rs. Until it ships, its interface here is provisional: `<guard> status --json` prints
//! `{"schema_version":1,"contract":"nv-rs-run-guard","enabled":true,"caps":{"host_mib":..,"vram_mib":..,"gtt_mib":..}}`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_runner::{Call, Runner, sha256_hex};
use serde::{Deserialize, Serialize};

/// The guard program's name, without the `.exe` Windows adds.
pub const GUARD: &str = "nv-rs-guard";

/// The Contract name of the guard's answer.
pub const GUARD_CONTRACT: &str = "nv-rs-run-guard";

/// The guard answer versions this build reads.
pub const GUARD_VERSIONS: &[i64] = &[1];

/// The caps the guard enforces, in MiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caps {
    /// Host memory.
    pub host_mib: u64,
    /// Video memory.
    pub vram_mib: u64,
    /// GPU memory in system RAM (GTT).
    pub gtt_mib: u64,
}

/// A guard that answered that it is on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Guard {
    /// The guard program.
    pub path: PathBuf,
    /// Its SHA-256.
    pub sha256: String,
    /// Its caps.
    pub caps: Caps,
}

/// Ask the guard at `program` whether it is on.
///
/// # Errors
///
/// Why no game run may start, for people: there is no guard, it cannot be run, it is off, or its answer is not a
/// `nv-rs-run-guard` document this build reads.
pub fn check(program: Option<&Path>) -> Result<Guard, String> {
    let Some(program) = program.filter(|p| p.is_file()) else {
        return Err(match program {
            Some(p) => format!(
                "the memory-capped run guard {} is not there; no game run starts without it",
                p.display()
            ),
            None => "the engine has no memory-capped run guard; no game run starts without it"
                .to_owned(),
        });
    };
    let sha256 = std::fs::read(program)
        .map(|b| sha256_hex(&b))
        .map_err(|e| format!("{}: {e}", program.display()))?;
    let runner = Runner {
        timeout: Duration::from_secs(30),
        max_output: 1024 * 1024,
        ..Runner::default()
    };
    let args = ["status".to_owned(), "--json".to_owned()];
    let mut call = Call::new(GUARD, program, &args);
    call.expected_sha256 = Some(&sha256);
    let output = runner
        .run(&call)
        .map_err(|e| format!("the run guard did not answer: {e}"))?;
    let doc: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("the run guard's answer is not JSON: {e}"))?;
    if doc["contract"] != GUARD_CONTRACT {
        return Err(format!(
            "the run guard answered a {} document, not {GUARD_CONTRACT}",
            doc["contract"]
        ));
    }
    match doc["schema_version"].as_i64() {
        Some(v) if GUARD_VERSIONS.contains(&v) => {}
        v => {
            return Err(format!(
                "the run guard answered {GUARD_CONTRACT} version {}; this raul reads version 1",
                v.map_or_else(|| "none".to_owned(), |v| v.to_string())
            ));
        }
    }
    if doc["enabled"] != true {
        let why = doc["reason"].as_str().unwrap_or("no reason given");
        return Err(format!(
            "the memory-capped run guard is off ({why}); no game run starts without it"
        ));
    }
    let caps: Caps = serde_json::from_value(doc["caps"].clone())
        .map_err(|e| format!("the run guard names no caps: {e}"))?;
    if caps.host_mib == 0 || caps.vram_mib == 0 || caps.gtt_mib == 0 {
        return Err("the run guard names a cap of 0, which is no cap".to_owned());
    }
    Ok(Guard {
        path: program.to_path_buf(),
        sha256,
        caps,
    })
}
