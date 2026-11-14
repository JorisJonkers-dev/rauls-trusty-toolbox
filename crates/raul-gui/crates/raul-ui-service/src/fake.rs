// SPDX-License-Identifier: GPL-3.0-only
//! A [`Service`] that answers from a script, for tests of screens and wizards without a command line.

use crate::dto::{DoctorReport, ImportReport, InstallReport, TtwReport};
use crate::service::{
    DoctorRequest, ImportRequest, InstallRequest, InstallResult, Service, ServiceError, TtwRequest,
};
use std::cell::RefCell;
use std::collections::VecDeque;

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
}

/// Reports for tests: small, valid values.
pub mod sample {
    use super::{ImportReport, InstallReport, TtwReport};
    use crate::dto::TtwRecord;

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
