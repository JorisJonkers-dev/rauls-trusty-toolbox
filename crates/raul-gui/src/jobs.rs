// SPDX-License-Identifier: GPL-3.0-only
//! Jobs: `raul` commands running in the background while the window stays responsive (UI-RULES K3).
//!
//! A page starts a job with [`Jobs::start`]; the command runs on a thread of its own through
//! `raul_ui_service::CliService`, which starts `raul` only through the streaming Runner. Its events arrive in the
//! job table of `raul-ui-model` on the next frame, and the window is asked to repaint as each arrives. A crash or an
//! out-of-memory of the command ends that job, never the window.

use eframe::egui;
use raul_ui_model::ViewError;
use raul_ui_model::job::{JobError, JobEvent, JobId, JobTable};
use raul_ui_service::registry::Action;
use raul_ui_service::{
    CancelHandle, CliService, DirectJob, Exec, JobSpec, Output, ServiceError, cancel_job,
    run_direct,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

/// An [`Exec`] that threads can share.
#[derive(Clone)]
pub struct SharedExec(pub Arc<dyn Exec + Send + Sync>);

impl Exec for SharedExec {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        self.0.run(args)
    }

    fn run_streaming(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        self.0.run_streaming(args, on_line)
    }

    fn run_cancellable(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
        cancel: &CancelHandle,
    ) -> Result<Output, String> {
        self.0.run_cancellable(args, on_line, cancel)
    }
}

/// What a job thread reports.
enum Report {
    Event(JobEvent),
    Done(JobId, Result<Output, ServiceError>),
}

/// The jobs of this window.
pub struct Jobs {
    exec: SharedExec,
    table: JobTable,
    tx: Sender<Report>,
    rx: Receiver<Report>,
    next: u64,
    cancels: HashMap<JobId, CancelHandle>,
    outputs: HashMap<JobId, Output>,
    failures: HashMap<JobId, ViewError>,
    /// The writing job started but not ended yet; the table learns of it only when its first event arrives.
    writer: Option<JobId>,
    repaint: Option<egui::Context>,
    /// Registry ids that a test scripts as command lines through the runner instead of running them in process.
    scripted: Vec<&'static str>,
    /// The folders and switches the in-process runs use; the default is the person's own.
    context: raul_core::context::Context,
    /// A latch a test puts on the TTW install, so it stands still at a known line.
    hold_ttw: Option<Arc<raul_ui_service::Hold>>,
}

impl std::fmt::Debug for Jobs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Jobs")
            .field("jobs", &self.table.jobs().len())
            .finish_non_exhaustive()
    }
}

impl Jobs {
    /// Jobs that run their commands through `exec`.
    pub fn new(exec: Arc<dyn Exec + Send + Sync>) -> Jobs {
        let (tx, rx) = channel();
        Jobs {
            exec: SharedExec(exec),
            table: JobTable::default(),
            tx,
            rx,
            next: 1,
            cancels: HashMap::new(),
            outputs: HashMap::new(),
            failures: HashMap::new(),
            writer: None,
            repaint: None,
            scripted: Vec::new(),
            context: raul_core::context::Context::default(),
            hold_ttw: None,
        }
    }

    /// Jobs that run in this process with the folders and switches of `context` instead of the person's own: every
    /// command line goes through `raul_cli::run_in` with it, and so does every typed job that reads `raul config`. How
    /// a test gives a window a configuration of its own without an environment variable and without a `raul` program.
    pub fn in_context(context: raul_core::context::Context) -> Jobs {
        let exec = raul_ui_service::InProcessExec::with_context(context.clone());
        Jobs::new(Arc::new(exec)).with_context(context)
    }

    /// Reads `raul config` and the settings from `context` in the typed jobs; the `exec` the jobs were made with decides
    /// where the command lines run (give it the same context).
    #[must_use]
    pub fn with_context(mut self, context: raul_core::context::Context) -> Jobs {
        self.context = context;
        self
    }

    /// The context the in-process runs use.
    pub fn context(&self) -> &raul_core::context::Context {
        &self.context
    }

    /// Lets a test script the registry commands `ids` as command lines through the runner, where they would run in
    /// process: an action that creates or installs things on a real filesystem has nothing to run against in a
    /// page test.
    #[must_use]
    pub fn scripted(mut self, ids: &[&'static str]) -> Jobs {
        self.scripted.extend_from_slice(ids);
        self
    }

    /// Holds the TTW install at the line `hold` names, so a test acts while the install stands still instead of racing it.
    #[must_use]
    pub fn holding_ttw(mut self, hold: Arc<raul_ui_service::Hold>) -> Jobs {
        self.hold_ttw = Some(hold);
        self
    }

    /// The in-process job that does what the registry command `id` does, if it is converted and not scripted.
    pub fn direct_job(&self, id: &str, rest: &[String]) -> Option<raul_ui_service::DirectJob> {
        if self.scripted.contains(&id) {
            return None;
        }
        let job = raul_ui_service::direct::for_command_in(id, rest, &self.context)?;
        Some(match (&self.hold_ttw, id) {
            (Some(hold), "ttw") if !rest.iter().any(|a| a == "--dry-run") => {
                job.held_by(Arc::clone(hold))
            }
            _ => job,
        })
    }

    /// Jobs that run inside this process (ADR-0016): the actions with an nvmod job call it directly, and every other
    /// command line is run by `raul`'s own entry point. No `raul` or nvmod process is started; the game, which the
    /// launcher starts, is the only program a job leaves running.
    pub fn real() -> Jobs {
        Jobs::new(Arc::new(raul_ui_service::InProcessExec::new()))
    }

    /// Asks `ctx` to repaint whenever a job reports.
    pub fn repaint_with(&mut self, ctx: &egui::Context) {
        self.repaint = Some(ctx.clone());
    }

    /// Starts `raul <action> <args> --json`, then `<then> --json` when `then` is given and the first went through.
    /// A job that `writes` is refused while another writing job runs.
    ///
    /// # Errors
    /// [`JobError::Busy`] when another writing job runs.
    pub fn start(
        &mut self,
        action: Action,
        args: Vec<String>,
        writes: bool,
    ) -> Result<JobId, JobError> {
        self.start_then(action, args, None, writes)
    }

    /// [`Jobs::start`] with a second command run as part of the same job.
    ///
    /// # Errors
    /// As [`Jobs::start`].
    pub fn start_then(
        &mut self,
        action: Action,
        args: Vec<String>,
        then: Option<Vec<String>>,
        writes: bool,
    ) -> Result<JobId, JobError> {
        let id = self.begin(writes)?;
        let mut spec = JobSpec::new(id, action, args);
        if let Some(then) = then {
            spec = spec.then(then);
        }
        if writes {
            spec = spec.writing();
        }
        let exec = self.exec.clone();
        self.spawn(spec, move |spec, sink, cancel| {
            CliService::new(exec).run_job_cancellable(spec, sink, cancel)
        });
        Ok(id)
    }

    /// Starts a typed job of a Component that runs in this process (see `raul_ui_service::direct`) as a job of the
    /// same table: its events, its Cancel and its writer slot are the ones of a command line, and a writing job
    /// holds the Instance lock while it runs. Its result is read like the output of a command (`output`, `failure`).
    ///
    /// # Errors
    /// [`JobError::Busy`] when another writing job runs.
    pub fn start_direct(
        &mut self,
        action: Action,
        job: DirectJob,
        writes: bool,
    ) -> Result<JobId, JobError> {
        let id = self.begin(writes)?;
        let mut spec = JobSpec::new(id, action, Vec::new());
        if writes {
            spec = spec.writing();
        }
        self.spawn(spec, move |spec, sink, cancel| {
            Ok(run_direct(spec, job, sink, cancel))
        });
        Ok(id)
    }

    /// Takes the next job id, after refusing a writing job while another writes.
    fn begin(&mut self, writes: bool) -> Result<JobId, JobError> {
        if writes {
            if let Some(busy) = self.writer {
                return Err(JobError::Busy(busy));
            }
            self.table.check_can_start_writer()?;
        }
        let id = JobId(self.next);
        self.next += 1;
        if writes {
            self.writer = Some(id);
        }
        Ok(id)
    }

    /// Runs `work` for `spec` on a thread of its own and reports its events and its end.
    fn spawn<W>(&mut self, spec: JobSpec, work: W)
    where
        W: FnOnce(
                &JobSpec,
                &mut dyn FnMut(JobEvent),
                &CancelHandle,
            ) -> Result<Output, ServiceError>
            + Send
            + 'static,
    {
        let cancel = CancelHandle::new();
        self.cancels.insert(spec.id, cancel.clone());
        let (tx, repaint) = (self.tx.clone(), self.repaint.clone());
        std::thread::spawn(move || {
            let wake = || {
                if let Some(ctx) = &repaint {
                    ctx.request_repaint();
                }
            };
            // A panic in the command's handling ends this job, never the window, and its end is still reported.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                work(
                    &spec,
                    &mut |event| {
                        let _ = tx.send(Report::Event(event));
                        wake();
                    },
                    &cancel,
                )
            }))
            .unwrap_or_else(|_| {
                Err(ServiceError::Failed(ViewError::new(
                    raul_ui_model::ExitClass::Internal,
                    "the job stopped unexpectedly",
                    "The state is not known; run the Doctor",
                )))
            });
            let _ = tx.send(Report::Done(spec.id, result));
            wake();
        });
    }

    /// Asks the job `id` to stop: it is asked to end and forced to after a grace period.
    ///
    /// # Errors
    /// The job is unknown, not offered Cancel, or not running.
    pub fn cancel(&mut self, id: JobId) -> Result<(), JobError> {
        let handle = self.cancels.get(&id).cloned().unwrap_or_default();
        cancel_job(&mut self.table, id, &handle)
    }

    /// Takes in what the job threads reported since the last call. The shell calls it once per frame.
    pub fn poll(&mut self) {
        while let Ok(report) = self.rx.try_recv() {
            match report {
                Report::Event(event) => {
                    self.table.apply(event);
                }
                Report::Done(id, result) => {
                    self.cancels.remove(&id);
                    if self.writer == Some(id) {
                        self.writer = None;
                    }
                    match result {
                        Ok(out) => {
                            self.outputs.insert(id, out);
                        }
                        Err(e) => {
                            self.failures.insert(id, e.view_error());
                        }
                    }
                }
            }
        }
    }

    /// The job table.
    pub fn table(&self) -> &JobTable {
        &self.table
    }

    /// What the finished job `id` printed, once it has ended.
    pub fn output(&self, id: JobId) -> Option<&Output> {
        self.outputs.get(&id)
    }

    /// Why the job `id` could not be started, if it could not.
    pub fn failure(&self, id: JobId) -> Option<&ViewError> {
        self.failures.get(&id)
    }

    /// Whether the job `id` has ended: it finished, or it could not be started.
    pub fn is_over(&self, id: JobId) -> bool {
        self.outputs.contains_key(&id) || self.failures.contains_key(&id)
    }

    /// Waits until every started job has reported its end; for tests.
    pub fn wait_all(&mut self, timeout: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            self.poll();
            if self.cancels.is_empty() {
                return true;
            }
            if std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_ui_model::job::{JobState, Outcome};
    use std::sync::Mutex;
    use std::time::Duration;

    /// Answers every command with `code` and records what it was asked.
    #[derive(Default)]
    struct Answering {
        code: i64,
        calls: Mutex<Vec<Vec<String>>>,
        fail_start: bool,
    }

    impl Exec for Answering {
        fn run(&self, args: &[String]) -> Result<Output, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            if self.fail_start {
                return Err("could not start raul".into());
            }
            Ok(Output {
                code: Some(self.code),
                stdout: "{}".into(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn a_job_runs_in_the_background_and_lands_in_the_table() {
        let exec = Arc::new(Answering::default());
        let mut jobs = Jobs::new(exec.clone());
        let id = jobs
            .start(
                Action::Doctor,
                vec!["--instance".into(), "/i".into()],
                false,
            )
            .unwrap();
        assert!(jobs.wait_all(Duration::from_secs(10)));
        assert!(jobs.is_over(id));
        assert_eq!(
            jobs.table().get(id).map(|j| j.state),
            Some(JobState::Finished(Outcome::Completed))
        );
        assert_eq!(jobs.output(id).and_then(|o| o.code), Some(0));
        assert_eq!(
            exec.calls.lock().unwrap()[0],
            ["doctor", "--instance", "/i", "--json"]
        );
    }

    #[test]
    fn a_command_that_cannot_start_is_a_failure_not_a_crash() {
        let exec = Arc::new(Answering {
            fail_start: true,
            ..Answering::default()
        });
        let mut jobs = Jobs::new(exec);
        let id = jobs.start(Action::Doctor, vec![], false).unwrap();
        assert!(jobs.wait_all(Duration::from_secs(10)));
        let why = jobs.failure(id).unwrap();
        assert!(why.what.contains("could not start raul"), "{why:?}");
    }

    #[test]
    fn a_second_writer_is_refused_before_the_first_reports() {
        let mut jobs = Jobs::new(Arc::new(Answering::default()));
        let first = jobs.start(Action::Sort, vec![], true).unwrap();
        assert_eq!(
            jobs.start(Action::Rollback, vec![], true),
            Err(JobError::Busy(first))
        );
        assert!(jobs.start(Action::Doctor, vec![], false).is_ok());
        assert!(jobs.wait_all(Duration::from_secs(10)));
        assert!(jobs.start(Action::Rollback, vec![], true).is_ok());
        assert!(jobs.wait_all(Duration::from_secs(10)));
    }

    /// Panics on every command.
    struct Panicking;

    impl Exec for Panicking {
        fn run(&self, _args: &[String]) -> Result<Output, String> {
            panic!("a bug in the command's handling")
        }
    }

    #[test]
    fn a_panicking_job_still_ends() {
        let mut jobs = Jobs::new(Arc::new(Panicking));
        let id = jobs.start(Action::Sort, vec![], true).unwrap();
        assert!(jobs.wait_all(Duration::from_secs(10)));
        assert!(jobs.failure(id).is_some());
        assert!(
            jobs.start(Action::Sort, vec![], true).is_ok(),
            "the writer slot was freed"
        );
    }

    #[test]
    fn ids_count_up_and_an_unknown_job_cannot_be_cancelled() {
        let mut jobs = Jobs::new(Arc::new(Answering::default()));
        let a = jobs.start(Action::Doctor, vec![], false).unwrap();
        let b = jobs.start(Action::Verify, vec![], false).unwrap();
        assert!(b.0 > a.0);
        assert!(jobs.cancel(JobId(999)).is_err());
        assert!(jobs.wait_all(Duration::from_secs(10)));
        assert!(format!("{jobs:?}").contains("Jobs"));
    }

    #[test]
    fn a_direct_job_lands_in_the_table_and_holds_the_writer_slot() {
        let mut jobs = Jobs::new(Arc::new(Panicking));
        let job = DirectJob::reading(
            |_, _| -> Result<nvmod_api::Outcome<u8>, nvmod_api::JobError> {
                Ok(nvmod_api::Outcome::Done(7))
            },
        );
        let id = jobs.start_direct(Action::Rollback, job, true).unwrap();
        assert_eq!(
            jobs.start_direct(
                Action::Rollback,
                DirectJob::reading(
                    |_, _| -> Result<nvmod_api::Outcome<u8>, nvmod_api::JobError> {
                        Ok(nvmod_api::Outcome::Done(0))
                    }
                ),
                true
            ),
            Err(JobError::Busy(id)),
            "a direct writer is a writer"
        );
        assert!(jobs.wait_all(Duration::from_secs(10)));
        assert_eq!(
            jobs.table().get(id).map(|j| j.state),
            Some(JobState::Finished(Outcome::Completed))
        );
        assert_eq!(
            jobs.output(id).map(|o| o.stdout.trim().to_owned()),
            Some("7".into())
        );
    }

    #[test]
    fn a_panicking_direct_job_still_ends_and_frees_the_slot() {
        let mut jobs = Jobs::new(Arc::new(Panicking));
        let job = DirectJob::reading(
            |_, _| -> Result<nvmod_api::Outcome<u8>, nvmod_api::JobError> {
                panic!("a bug in the job")
            },
        );
        let id = jobs.start_direct(Action::Rollback, job, true).unwrap();
        assert!(jobs.wait_all(Duration::from_secs(10)));
        assert_eq!(jobs.output(id).and_then(|o| o.code), None);
        assert_eq!(
            jobs.table().get(id).map(|j| j.state),
            Some(JobState::Finished(Outcome::Failed))
        );
        let again = DirectJob::reading(
            |_, _| -> Result<nvmod_api::Outcome<u8>, nvmod_api::JobError> {
                Ok(nvmod_api::Outcome::Done(1))
            },
        );
        assert!(
            jobs.start_direct(Action::Rollback, again, true).is_ok(),
            "the writer slot was freed"
        );
        assert!(jobs.wait_all(Duration::from_secs(10)));
    }
}
