// SPDX-License-Identifier: GPL-3.0-only
//! Where the stop request of one driver run comes from.
//!
//! The command line stops a run on Ctrl+C or SIGTERM, which set the process-wide request of `nvmod_core::cancel`. A run
//! the window starts has its own request, so that stopping one job never reaches another and a request left over from an
//! earlier job cannot stop the next one.

/// The source of a run's stop request.
#[derive(Clone, Copy)]
pub enum Stop<'a> {
    /// The process-wide request (`nvmod_core::cancel`): Ctrl+C, SIGTERM, or a caller that sets it.
    Process,
    /// The request of this run only, read through the function: true once the run was told to stop.
    Run(&'a (dyn Fn() -> bool + Sync)),
}

impl Stop<'_> {
    /// Whether the run was told to stop.
    #[must_use]
    pub fn requested(&self) -> bool {
        match self {
            Self::Process => nvmod_core::cancel::requested(),
            Self::Run(requested) => requested(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn a_run_request_is_read_through_its_own_flag() {
        let flag = AtomicBool::new(false);
        let read = || flag.load(Ordering::SeqCst);
        let stop = Stop::Run(&read);
        assert!(!stop.requested());
        flag.store(true, Ordering::SeqCst);
        assert!(stop.requested());
    }
}
