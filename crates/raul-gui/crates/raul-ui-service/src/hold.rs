// SPDX-License-Identifier: GPL-3.0-only
//! A latch a test puts on a streaming job, so the job stands still at a known line while the test acts, instead of
//! racing the job's speed. The job parks when its stream first reaches a line that mentions the marker, and goes on
//! when the test releases it.

use crate::direct::DirectJob;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// How long a test waits for the job to park or a stop to arrive before it is a failure and not a hang.
const LIMIT: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Not reached yet.
    Armed,
    /// The job stands at the line.
    Parked,
    /// The test let it go (`true`: only once its stop request has arrived).
    Released(bool),
}

/// The latch: shared between the job and the test.
pub struct Hold {
    marker: &'static str,
    state: Mutex<State>,
    changed: Condvar,
}

impl Hold {
    /// A latch that parks the job at the first line of its stream that contains `marker`.
    #[must_use]
    pub fn at(marker: &'static str) -> Arc<Self> {
        Arc::new(Self {
            marker,
            state: Mutex::new(State::Armed),
            changed: Condvar::new(),
        })
    }

    fn set(&self, to: State) {
        *self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = to;
        self.changed.notify_all();
    }

    /// Blocks until the job stands at the line.
    ///
    /// # Panics
    /// When it does not get there within a minute.
    pub fn wait_parked(&self) {
        let guard = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (guard, timeout) = self
            .changed
            .wait_timeout_while(guard, LIMIT, |s| *s == State::Armed)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            !timeout.timed_out() && *guard != State::Armed,
            "the job never reached the held line"
        );
    }

    /// Lets the job go on.
    pub fn release(&self) {
        self.set(State::Released(false));
    }

    /// Lets the job go on once its own stop request has reached it: the install then sees the stop at its next event.
    pub fn release_when_stopped(&self) {
        self.set(State::Released(true));
    }

    /// Called with each line of the stream, on the thread that reads it: parks there the first time the marker shows.
    fn pass(&self, line: &str, stopped: &dyn Fn() -> bool) {
        if !line.contains(self.marker) {
            return;
        }
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *guard != State::Armed {
            return;
        }
        *guard = State::Parked;
        self.changed.notify_all();
        let (guard, timeout) = self
            .changed
            .wait_timeout_while(guard, LIMIT, |s| *s == State::Parked)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let after = *guard;
        drop(guard);
        assert!(!timeout.timed_out(), "the held job was never released");
        if after == State::Released(true) {
            let waited = std::time::Instant::now();
            while !stopped() {
                assert!(waited.elapsed() < LIMIT, "the stop never reached the job");
                std::thread::yield_now();
            }
        }
    }
}

impl DirectJob {
    /// This job, held by `hold` at the line its latch names (streaming jobs only; any other job is returned as it was).
    #[must_use]
    pub fn held_by(self, hold: Arc<Hold>) -> Self {
        self.wrap_lines(move |run| {
            Box::new(move |sink, cancel| {
                let stopped = || cancel.is_cancelled();
                run(
                    &mut |line: &str| {
                        hold.pass(line, &stopped);
                        sink(line);
                    },
                    cancel,
                )
            })
        })
    }
}
