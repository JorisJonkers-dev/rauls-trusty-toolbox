// SPDX-License-Identifier: GPL-3.0-only

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::events::Event;

/// The result of running ttwright in process. The install of the driver switches to it in the next step.
#[allow(dead_code)]
pub(crate) struct InProc {
    /// The exit code the `ttwright` program would have ended with.
    pub exit_code: i32,
    /// The `ttwright-result` document.
    pub doc: serde_json::Value,
}

/// Run ttwright's install in process, mapping progress to `Event`s.
#[allow(dead_code)]
pub(crate) fn install_in_process(
    job: &ttwright_core::api::InstallJob,
    timeout: Duration,
    emit: &mut (dyn FnMut(Event) + Send),
) -> InProc {
    let cancel = ttwright_core::api::CancelToken::new();
    let done = AtomicBool::new(false);
    let started = Instant::now();

    std::thread::scope(|scope| {
        scope.spawn(|| {
            while !done.load(Ordering::Relaxed) {
                if nvmod_core::cancel::requested() || started.elapsed() >= timeout {
                    cancel.cancel();
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });

        let phase_name = |phase: &ttwright_contract::Phase| {
            serde_json::to_value(phase)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default()
        };
        let result = ttwright_core::api::install(
            job,
            &mut |event: &ttwright_contract::Event| match event {
                ttwright_contract::Event::Phase { phase } => emit(Event::Phase {
                    phase: phase_name(phase),
                }),
                ttwright_contract::Event::Progress {
                    phase,
                    done,
                    of,
                    message,
                } => emit(Event::Progress {
                    phase: phase_name(phase),
                    done: *done,
                    of: *of,
                    message: Some(message.clone()),
                }),
                ttwright_contract::Event::Warning { message } => emit(Event::Warning {
                    message: message.clone(),
                }),
                _ => {}
            },
            &cancel,
        );

        done.store(true, Ordering::Relaxed);

        match result {
            Ok(ttwright_core::api::InstallOutput::Done { result }) => InProc {
                exit_code: 0,
                doc: serde_json::to_value(result).unwrap_or_default(),
            },
            Ok(ttwright_core::api::InstallOutput::Cancelled { result }) => InProc {
                exit_code: 4,
                doc: serde_json::to_value(result).unwrap_or_default(),
            },
            Err(e) => InProc {
                exit_code: i32::from(e.result.status.exit_code()),
                doc: serde_json::to_value(&e.result).unwrap_or_default(),
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use ttwright_core::api::InstallJob;

    #[test]
    fn a_missing_mpi_path_fails() {
        let job = InstallJob {
            mpi_path: PathBuf::from("/definitely/does/not/exist.mpi"),
            ..Default::default()
        };
        let mut events = Vec::new();
        let out = install_in_process(&job, Duration::from_secs(1), &mut |e| events.push(e));
        assert_ne!(out.exit_code, 0);
        let status = out.doc.get("status").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            status == "failed" || status == "unsupported",
            "unexpected status: {status}"
        );
    }
}
