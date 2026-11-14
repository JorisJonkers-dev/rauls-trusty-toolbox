// SPDX-License-Identifier: GPL-3.0-only
//! A [`Service`] that answers from a script, for tests of screens and wizards without a command line.

use crate::dto::{
    DoctorReport, ImportReport, InstallReport, RollbackReport, SortReport, TargetReport,
    ToolsReport, TtwReport, VerifyReport,
};
use crate::exec::{Exec, Output};
use crate::service::{
    DoctorRequest, ImportRequest, InstallRequest, InstallResult, RollbackRequest, Service,
    ServiceError, SortRequest, TargetRequest, ToolsRequest, TtwRequest, VerifyRequest,
};
use std::cell::RefCell;
use std::collections::VecDeque;

/// The scripted answers of one command and the requests it received.
#[derive(Debug)]
struct Script<Q, A> {
    answers: RefCell<VecDeque<Result<A, ServiceError>>>,
    calls: RefCell<Vec<Q>>,
}

impl<Q, A> Default for Script<Q, A> {
    fn default() -> Self {
        Self {
            answers: RefCell::new(VecDeque::new()),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl<Q: Clone, A> Script<Q, A> {
    fn queue(&self, answer: Result<A, ServiceError>) {
        self.answers.borrow_mut().push_back(answer);
    }

    fn call(&self, request: &Q, what: &str) -> Result<A, ServiceError> {
        self.calls.borrow_mut().push(request.clone());
        next(&self.answers, what)
    }

    fn calls(&self) -> Vec<Q> {
        self.calls.borrow().clone()
    }
}

/// One scripted answer per call, in order, per command. A call with nothing left in its queue panics, so a test
/// notices a call it did not expect. Every request is recorded.
#[derive(Debug, Default)]
pub struct FakeService {
    imports: RefCell<VecDeque<Result<ImportReport, ServiceError>>>,
    installs: RefCell<VecDeque<Result<InstallResult, ServiceError>>>,
    ttws: RefCell<VecDeque<Result<TtwReport, ServiceError>>>,
    doctors: RefCell<VecDeque<Result<DoctorReport, ServiceError>>>,
    import_calls: RefCell<Vec<ImportRequest>>,
    install_calls: RefCell<Vec<InstallRequest>>,
    ttw_calls: RefCell<Vec<TtwRequest>>,
    doctor_calls: RefCell<Vec<DoctorRequest>>,
    sorts: Script<SortRequest, SortReport>,
    verifies: Script<VerifyRequest, VerifyReport>,
    targets: Script<TargetRequest, TargetReport>,
    tools_lists: Script<ToolsRequest, ToolsReport>,
    tools_verifies: Script<ToolsRequest, ToolsReport>,
    rollbacks: Script<RollbackRequest, RollbackReport>,
}

impl FakeService {
    /// A fake with an empty script.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues the answer of the next `import_mo2` call.
    pub fn on_import(&self, answer: Result<ImportReport, ServiceError>) -> &Self {
        self.imports.borrow_mut().push_back(answer);
        self
    }

    /// Queues the answer of the next `install_mod` call.
    pub fn on_install(&self, answer: Result<InstallResult, ServiceError>) -> &Self {
        self.installs.borrow_mut().push_back(answer);
        self
    }

    /// Queues the answer of the next `ttw` call.
    pub fn on_ttw(&self, answer: Result<TtwReport, ServiceError>) -> &Self {
        self.ttws.borrow_mut().push_back(answer);
        self
    }

    /// Queues the answer of the next `doctor` call.
    pub fn on_doctor(&self, answer: Result<DoctorReport, ServiceError>) -> &Self {
        self.doctors.borrow_mut().push_back(answer);
        self
    }

    /// Queues the answer of the next `sort` call.
    pub fn on_sort(&self, answer: Result<SortReport, ServiceError>) -> &Self {
        self.sorts.queue(answer);
        self
    }

    /// Queues the answer of the next `verify` call.
    pub fn on_verify(&self, answer: Result<VerifyReport, ServiceError>) -> &Self {
        self.verifies.queue(answer);
        self
    }

    /// Queues the answer of the next `set_target` call.
    pub fn on_target(&self, answer: Result<TargetReport, ServiceError>) -> &Self {
        self.targets.queue(answer);
        self
    }

    /// Queues the answer of the next `tools_list` call.
    pub fn on_tools_list(&self, answer: Result<ToolsReport, ServiceError>) -> &Self {
        self.tools_lists.queue(answer);
        self
    }

    /// Queues the answer of the next `tools_verify` call.
    pub fn on_tools_verify(&self, answer: Result<ToolsReport, ServiceError>) -> &Self {
        self.tools_verifies.queue(answer);
        self
    }

    /// Queues the answer of the next `rollback` call.
    pub fn on_rollback(&self, answer: Result<RollbackReport, ServiceError>) -> &Self {
        self.rollbacks.queue(answer);
        self
    }

    /// The `sort` requests so far.
    pub fn sort_calls(&self) -> Vec<SortRequest> {
        self.sorts.calls()
    }

    /// The `verify` requests so far.
    pub fn verify_calls(&self) -> Vec<VerifyRequest> {
        self.verifies.calls()
    }

    /// The `set_target` requests so far.
    pub fn target_calls(&self) -> Vec<TargetRequest> {
        self.targets.calls()
    }

    /// The `tools_list` requests so far.
    pub fn tools_list_calls(&self) -> Vec<ToolsRequest> {
        self.tools_lists.calls()
    }

    /// The `tools_verify` requests so far.
    pub fn tools_verify_calls(&self) -> Vec<ToolsRequest> {
        self.tools_verifies.calls()
    }

    /// The `rollback` requests so far.
    pub fn rollback_calls(&self) -> Vec<RollbackRequest> {
        self.rollbacks.calls()
    }

    /// The `import_mo2` requests so far.
    pub fn import_calls(&self) -> Vec<ImportRequest> {
        self.import_calls.borrow().clone()
    }

    /// The `install_mod` requests so far.
    pub fn install_calls(&self) -> Vec<InstallRequest> {
        self.install_calls.borrow().clone()
    }

    /// The `ttw` requests so far.
    pub fn ttw_calls(&self) -> Vec<TtwRequest> {
        self.ttw_calls.borrow().clone()
    }

    /// The `doctor` requests so far.
    pub fn doctor_calls(&self) -> Vec<DoctorRequest> {
        self.doctor_calls.borrow().clone()
    }
}

fn next<T>(queue: &RefCell<VecDeque<T>>, what: &str) -> T {
    queue
        .borrow_mut()
        .pop_front()
        .unwrap_or_else(|| panic!("FakeService: no scripted answer left for {what}"))
}

impl Service for FakeService {
    fn import_mo2(&self, request: &ImportRequest) -> Result<ImportReport, ServiceError> {
        self.import_calls.borrow_mut().push(request.clone());
        next(&self.imports, "import_mo2")
    }

    fn install_mod(&self, request: &InstallRequest) -> Result<InstallResult, ServiceError> {
        self.install_calls.borrow_mut().push(request.clone());
        next(&self.installs, "install_mod")
    }

    fn ttw(&self, request: &TtwRequest) -> Result<TtwReport, ServiceError> {
        self.ttw_calls.borrow_mut().push(request.clone());
        next(&self.ttws, "ttw")
    }

    fn doctor(&self, request: &DoctorRequest) -> Result<DoctorReport, ServiceError> {
        self.doctor_calls.borrow_mut().push(request.clone());
        next(&self.doctors, "doctor")
    }

    fn sort(&self, request: &SortRequest) -> Result<SortReport, ServiceError> {
        self.sorts.call(request, "sort")
    }

    fn verify(&self, request: &VerifyRequest) -> Result<VerifyReport, ServiceError> {
        self.verifies.call(request, "verify")
    }

    fn set_target(&self, request: &TargetRequest) -> Result<TargetReport, ServiceError> {
        self.targets.call(request, "set_target")
    }

    fn tools_list(&self, request: &ToolsRequest) -> Result<ToolsReport, ServiceError> {
        self.tools_lists.call(request, "tools_list")
    }

    fn tools_verify(&self, request: &ToolsRequest) -> Result<ToolsReport, ServiceError> {
        self.tools_verifies.call(request, "tools_verify")
    }

    fn rollback(&self, request: &RollbackRequest) -> Result<RollbackReport, ServiceError> {
        self.rollbacks.call(request, "rollback")
    }
}

/// An [`Exec`] that answers each run from a script and records the arguments, for tests of job streaming
/// without a command line. A run with nothing left in the script panics.
#[derive(Debug, Default)]
pub struct ScriptedExec {
    runs: RefCell<VecDeque<Result<Output, String>>>,
    calls: RefCell<Vec<Vec<String>>>,
}

impl ScriptedExec {
    /// An exec with an empty script.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a run that prints `lines` (one per line) and exits with `code`.
    pub fn on_run(&self, lines: &[&str], code: i64, stderr: &str) -> &Self {
        let mut stdout = lines.join("\n");
        if !lines.is_empty() {
            stdout.push('\n');
        }
        self.runs.borrow_mut().push_back(Ok(Output {
            code: Some(code),
            stdout,
            stderr: stderr.to_owned(),
        }));
        self
    }

    /// Queues a run that cannot start.
    pub fn on_start_failure(&self, why: &str) -> &Self {
        self.runs.borrow_mut().push_back(Err(why.to_owned()));
        self
    }

    /// The arguments of every run so far.
    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }
}

impl Exec for ScriptedExec {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        self.calls.borrow_mut().push(args.to_vec());
        next(&self.runs, "run")
    }
}

/// Reports for tests: small, valid values.
pub mod sample {
    use super::{ImportReport, InstallReport, RollbackReport, SortReport, ToolsReport, TtwReport};
    use crate::dto::{SortRow, ToolRow, TtwRecord};

    /// A sort of two plugins that was only proposed.
    pub fn sort() -> SortReport {
        let row = |position, name: &str| SortRow {
            position,
            name: name.to_owned(),
            active: true,
            installed: true,
            group: Some("default".to_owned()),
            messages: Vec::new(),
            generated_messages: Vec::new(),
            findings: Vec::new(),
        };
        SortReport {
            profile: "Default".into(),
            reordered: true,
            order: vec![row(1, "FalloutNV.esm"), row(2, "A.esp")],
            general_messages: Vec::new(),
            not_installed: Vec::new(),
            warnings: Vec::new(),
            applied: None,
        }
    }

    /// An undo of an install.
    pub fn rollback() -> RollbackReport {
        RollbackReport {
            journal: "j1".into(),
            kind: "install".into(),
            mod_name: "Mod".into(),
            profile: "Default".into(),
        }
    }

    /// Two pinned tools, one fetched and one not.
    pub fn tools(verified: bool) -> ToolsReport {
        let row = |name: &str, state: &str| ToolRow {
            name: name.to_owned(),
            version: "1.0".to_owned(),
            state: state.to_owned(),
            detail: String::new(),
        };
        ToolsReport {
            rows: vec![row("alpha", "verified"), row("beta", "not fetched")],
            verified,
        }
    }

    /// An import of two Mods.
    pub fn import() -> ImportReport {
        ImportReport {
            instance: "/inst".into(),
            profile: "Default".into(),
            enabled_mods: vec!["B".into(), "A".into()],
            plugins: vec!["A.esp".into()],
            load_order: vec!["A.esp".into()],
            mods: 2,
            linked: 3,
            copied: 0,
            unmapped: Vec::new(),
        }
    }

    /// A dry run or an install of one Mod with the given files.
    pub fn install(dry_run: bool, files: &[&str]) -> InstallReport {
        InstallReport {
            status: if dry_run { "dry-run" } else { "installed" }.into(),
            dry_run,
            mod_name: "Mod".into(),
            profile: "Default".into(),
            files: files.iter().map(|f| (*f).to_owned()).collect(),
            plugins: Vec::new(),
            inferred: Vec::new(),
            staged_files: if dry_run { None } else { Some(files.len()) },
            journal: if dry_run { None } else { Some("j1".into()) },
        }
    }

    /// A TTW install that worked.
    pub fn ttw() -> TtwReport {
        TtwReport {
            status: "installed".into(),
            mod_name: "Tale of Two Wastelands".into(),
            profile: "Default".into(),
            files: 2,
            plugins: vec!["TaleOfTwoWastelands.esm".into()],
            journal: Some("j2".into()),
            result: TtwRecord {
                outcome: "success".into(),
                exit_code: 0,
                fallout3_verified: true,
                files_written: 2,
                messages: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::RuntimeTarget;

    #[test]
    fn every_typed_call_answers_in_order_and_records_its_request() {
        let fake = FakeService::new();
        fake.on_sort(Ok(sample::sort()))
            .on_verify(Err(ServiceError::Failed(
                nvmod_ui_model::error::ViewError::new(
                    nvmod_ui_model::error::ExitClass::Refused,
                    "no",
                    "Nothing was changed",
                ),
            )))
            .on_target(Ok(TargetReport {
                instance: "/i".into(),
                target: RuntimeTarget::NvRs,
            }))
            .on_tools_list(Ok(sample::tools(true)))
            .on_tools_verify(Ok(sample::tools(false)))
            .on_rollback(Ok(sample::rollback()));
        let sort = SortRequest {
            instance: "/i".into(),
            masterlist: "m.yaml".into(),
            ..SortRequest::default()
        };
        assert!(fake.sort(&sort).unwrap().reordered);
        assert!(fake.verify(&VerifyRequest::default()).is_err());
        let target = TargetRequest {
            instance: "/i".into(),
            target: RuntimeTarget::NvRs,
        };
        assert_eq!(
            fake.set_target(&target).unwrap().target,
            RuntimeTarget::NvRs
        );
        let tools = ToolsRequest::default();
        assert!(fake.tools_list(&tools).unwrap().verified);
        assert!(!fake.tools_verify(&tools).unwrap().verified);
        let rb = RollbackRequest {
            instance: "/i".into(),
        };
        assert_eq!(fake.rollback(&rb).unwrap().kind, "install");
        assert_eq!(fake.sort_calls(), [sort]);
        assert_eq!(fake.verify_calls().len(), 1);
        assert_eq!(fake.target_calls(), [target]);
        assert_eq!(fake.tools_list_calls().len(), 1);
        assert_eq!(fake.tools_verify_calls().len(), 1);
        assert_eq!(fake.rollback_calls(), [rb]);
    }

    #[test]
    #[should_panic(expected = "no scripted answer left for rollback")]
    fn a_call_without_a_scripted_answer_panics() {
        let _ = FakeService::new().rollback(&RollbackRequest::default());
    }

    #[test]
    fn request_arguments_are_the_words_after_the_command() {
        let sort = SortRequest {
            instance: "/i".into(),
            profile: Some("P".into()),
            masterlist: "m".into(),
            apply: true,
        };
        assert_eq!(
            sort.args(),
            [
                "--instance",
                "/i",
                "--profile",
                "P",
                "--masterlist",
                "m",
                "--apply"
            ]
        );
        let target = TargetRequest {
            instance: "/i".into(),
            target: RuntimeTarget::Vanilla,
        };
        assert_eq!(target.args(), ["vanilla", "--instance", "/i"]);
    }
}
