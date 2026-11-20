// SPDX-License-Identifier: GPL-3.0-only

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::Stop;
use crate::events::Event;

/// The result of running ttwright in process.
pub(crate) struct InProc {
    /// The exit code the `ttwright` program would have ended with.
    pub exit_code: i32,
    /// The `ttwright-result` document.
    pub doc: serde_json::Value,
}

/// Sets the flag when it is dropped, so the watcher thread ends even when the install panics. A panic in the
/// install skips every statement after it, and `thread::scope` waits for the watcher before it lets the panic out.
struct StopWatcher<'a>(&'a AtomicBool);

impl Drop for StopWatcher<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Run ttwright's install in process, mapping progress to `Event`s.
pub(crate) fn install_in_process(
    job: &ttwright_core::api::InstallJob,
    stop: Stop<'_>,
    timeout: Duration,
    emit: &mut (dyn FnMut(Event) + Send),
) -> InProc {
    let cancel = ttwright_core::api::CancelToken::new();
    let done = AtomicBool::new(false);
    let started = Instant::now();

    std::thread::scope(|scope| {
        let _stop_watcher = StopWatcher(&done);
        scope.spawn(|| {
            while !done.load(Ordering::Relaxed) {
                if stop.requested() || started.elapsed() >= timeout {
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

/// [`install_in_process`] for a caller whose event callback is not `Send`: the events cross a channel and `emit`
/// is called on the calling thread, while the install runs on another.
pub(crate) fn install_with_events(
    job: &ttwright_core::api::InstallJob,
    stop: Stop<'_>,
    timeout: Duration,
    emit: &mut dyn FnMut(Event),
) -> InProc {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let worker = scope.spawn(move || {
            install_in_process(job, stop, timeout, &mut |event| {
                let _ = tx.send(event);
            })
        });
        for event in rx {
            emit(event);
        }
        worker.join().unwrap_or_else(|_| InProc {
            exit_code: 1,
            doc: serde_json::json!({
                "error": { "message": "ttwright stopped unexpectedly" },
            }),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use ttwright_core::api::InstallJob;

    #[test]
    fn a_panic_in_the_install_still_stops_the_watcher_thread() {
        let done = AtomicBool::new(false);
        let started = Instant::now();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::thread::scope(|scope| {
                let _stop = StopWatcher(&done);
                scope.spawn(|| {
                    while !done.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                });
                panic!("the install panicked");
            })
        }));
        assert!(outcome.is_err(), "the panic comes out of the scope");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the watcher ended at once, not at a timeout"
        );
    }

    #[test]
    fn a_missing_mpi_path_fails() {
        let job = InstallJob {
            mpi_path: PathBuf::from("/definitely/does/not/exist.mpi"),
            ..Default::default()
        };
        let mut events = Vec::new();
        let out = install_in_process(&job, Stop::Process, Duration::from_secs(1), &mut |e| {
            events.push(e);
        });
        assert_eq!(out.exit_code, 1, "{}", out.doc);
        assert_eq!(out.doc["status"], "failed", "{}", out.doc);
        assert_eq!(out.doc["error"]["code"], "mpi_not_found", "{}", out.doc);
    }

    #[test]
    fn events_reach_a_callback_that_is_not_send() {
        let job = InstallJob {
            mpi_path: PathBuf::from("/definitely/does/not/exist.mpi"),
            ..Default::default()
        };
        let seen = std::cell::RefCell::new(Vec::new());
        let out = install_with_events(&job, Stop::Process, Duration::from_secs(1), &mut |e| {
            seen.borrow_mut().push(e);
        });
        assert_ne!(out.exit_code, 0);
        assert!(
            seen.borrow()
                .iter()
                .any(|e| matches!(e, Event::Phase { .. })),
            "no phase event arrived: {} events",
            seen.borrow().len()
        );
    }
}
