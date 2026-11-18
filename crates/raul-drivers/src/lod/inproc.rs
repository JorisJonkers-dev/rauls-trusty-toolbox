// SPDX-License-Identifier: GPL-3.0-only
//! lodwright's generate, run in process through `lodwright-api`: its progress becomes [`Event`]s, and a watcher thread
//! cancels the call on a stop request or after the timeout.
//!
//! The call runs on a worker thread so that the events can be emitted on the calling thread, whatever the caller's
//! callback is. The worker waits for each event to be emitted before it goes on, so an emit that asks for a cancel
//! has it in place before the call's next safe point.
//!
//! The timeout is checked by the watcher only. `lodwright-api` checks the cancel token before each terrain block,
//! world and step, so a single step that runs long past `--timeout-secs` is not stopped until it ends.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use lodwright_api::{
    CancelToken, Error, GenerateOutcome, GenerateRequest, ProgressEvent, generate,
};

use crate::events::Event;

/// Nothing has stopped the call yet.
const RUNNING: u8 = 0;
/// A stop request (`nvmod_core::cancel`) stopped the call.
const STOPPED: u8 = 1;
/// The timeout stopped the call.
const TIMED_OUT: u8 = 2;

/// Sets `done` when dropped, also while unwinding, so the watcher ends even when the call panics. On the calling side
/// it also cancels the call when unwinding, so a panicking caller does not leave the worker running to the end.
struct Guard<'a> {
    done: &'a AtomicBool,
    cancel: Option<&'a CancelToken>,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel
            && std::thread::panicking()
        {
            cancel.cancel();
        }
        self.done.store(true, Ordering::SeqCst);
    }
}

/// What a call came to.
pub(crate) struct Run {
    /// The library's answer; when the call panicked, an `internal` error that says so.
    pub answer: Result<GenerateOutcome, Error>,
    /// The call panicked: nothing it wrote is reported as a result.
    pub panicked: bool,
    /// The timeout stopped the call, and no stop request did.
    pub timed_out: bool,
}

/// One progress event as an [`Event`]: a finished terrain block counts; any other event starts its step.
fn map(event: &ProgressEvent) -> Event {
    let phase = event.label_key().trim_start_matches("lod.").to_owned();
    match (event.level, event.done, event.total) {
        (Some(level), Some(done), Some(of)) => Event::Progress {
            phase,
            done,
            of,
            message: Some(format!(
                "{} level {level}",
                event.world.as_deref().unwrap_or("?")
            )),
        },
        _ => Event::Phase { phase },
    }
}

/// Run `job` in process, emitting its progress through `emit` while it runs. `cancel` stops the call at its next safe
/// point, as does a stop request (`nvmod_core::cancel`) or `timeout`, whichever the watcher sees first.
pub(crate) fn generate_with_events(
    job: &GenerateRequest,
    cancel: &CancelToken,
    timeout: Duration,
    emit: &mut dyn FnMut(Event),
) -> Run {
    let done = AtomicBool::new(false);
    // The one place the reason is recorded: only the watcher writes it, once.
    let fired = AtomicU8::new(RUNNING);
    let started = Instant::now();

    std::thread::scope(|scope| {
        // The channels live in this closure, so that a panic here drops the acknowledgement sender before the scope
        // waits for the worker, which then does not wait for an acknowledgement that never comes.
        let (tx, rx) = mpsc::channel::<Event>();
        let (ack_tx, ack_rx) = mpsc::channel::<()>();
        scope.spawn(|| {
            while !done.load(Ordering::SeqCst) {
                // The stop request is read first, then the clock; the first reason seen wins.
                let reason = if nvmod_core::cancel::requested() {
                    STOPPED
                } else if started.elapsed() >= timeout {
                    TIMED_OUT
                } else {
                    RUNNING
                };
                if reason != RUNNING {
                    let _ =
                        fired.compare_exchange(RUNNING, reason, Ordering::SeqCst, Ordering::SeqCst);
                    cancel.cancel();
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });

        // Ends the watcher, and cancels the call, if the emitting side panics before the worker is joined.
        let _stop_on_panic = Guard {
            done: &done,
            cancel: Some(cancel),
        };
        let stop = &done;
        let worker = scope.spawn(move || {
            let _stop = Guard {
                done: stop,
                cancel: None,
            };
            // Owns the senders: each event goes out, then the call waits until the caller has emitted it.
            let mut progress = move |event: &ProgressEvent| {
                if tx.send(map(event)).is_ok() {
                    let _ = ack_rx.recv();
                }
            };
            generate(job, &mut progress, cancel)
        });

        for event in rx {
            emit(event);
            let _ = ack_tx.send(());
        }
        let timed_out = fired.load(Ordering::SeqCst) == TIMED_OUT;
        match worker.join() {
            Ok(answer) => Run {
                answer,
                panicked: false,
                timed_out,
            },
            Err(_) => Run {
                answer: Err(Error::new(
                    "internal",
                    "LOD generation stopped unexpectedly",
                )),
                panicked: true,
                timed_out,
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use std::panic::AssertUnwindSafe;
    use std::time::Instant;

    use lodwright_api::Step;
    use lodwright_contract::Request;
    use lodwright_testkit::land::Heights;
    use lodwright_testkit::world::{SyntheticWorld, world_plugin};
    use nvmod_testkit::TempDir;

    use super::*;

    #[test]
    fn a_block_is_progress_and_a_start_is_a_phase() {
        let start = ProgressEvent::start(Step::Terrain, Some("Island"));
        assert_eq!(
            map(&start),
            Event::Phase {
                phase: "terrain".to_owned()
            }
        );
        let block = ProgressEvent {
            step: Step::Terrain,
            world: Some("Island".to_owned()),
            level: Some(4),
            done: Some(1),
            total: Some(2),
            up_to_date: false,
        };
        assert_eq!(
            map(&block),
            Event::Progress {
                phase: "terrain".to_owned(),
                done: 1,
                of: 2,
                message: Some("Island level 4".to_owned()),
            }
        );
        assert_eq!(
            map(&ProgressEvent::start(Step::ReadPlugins, None)),
            Event::Phase {
                phase: "read_plugins".to_owned()
            }
        );
    }

    /// A terrain-only job over a synthetic world of `side` by `side` cells, on one thread.
    fn job(dir: &std::path::Path, side: i32) -> GenerateRequest {
        let data = dir.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let mut world = SyntheticWorld::new(0xB00, "Island");
        for x in 0..side {
            for y in 0..side {
                world = world.cell(
                    x,
                    y,
                    Heights::flat(8.0 + f32::from(u8::try_from(x + y).unwrap_or(0))),
                );
            }
        }
        std::fs::write(data.join("Island.esm"), world_plugin(&[], &world, false)).unwrap();
        let mut request = Request::new(
            data.to_str().unwrap(),
            &["Island.esm"],
            dir.join("out").to_str().unwrap(),
        );
        request.objects.enabled = false;
        request.trees.enabled = false;
        request.terrain.land_textures = false;
        request.limits.threads = Some(1);
        GenerateRequest::new(request)
    }

    #[test]
    fn a_caller_that_panics_cancels_the_call_and_the_call_returns() {
        let dir = TempDir::new("raul-inproc-panic");
        let job = job(dir.path(), 6);
        let cancel = CancelToken::new();
        let watched = cancel.clone();
        let started = Instant::now();
        let caught = std::panic::catch_unwind(AssertUnwindSafe(|| {
            generate_with_events(&job, &cancel, Duration::from_secs(600), &mut |_| {
                panic!("the caller's emit fails")
            });
        }));
        assert!(caught.is_err(), "the panic reaches the caller");
        assert!(watched.is_cancelled(), "the call was not cancelled");
        assert!(started.elapsed() < Duration::from_secs(60));
    }
}
