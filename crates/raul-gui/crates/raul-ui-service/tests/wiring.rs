// SPDX-License-Identifier: GPL-3.0-only
//! The wizards driven by a scripted service: every branch of the real outputs, with no process and no files.

#![allow(clippy::assert_is_empty)]

use raul_ui_model::error::{ExitClass, Retry, ViewError};
use raul_ui_model::job::Outcome;
use raul_ui_model::wizard::{
    FlowError, FomodOption, FomodQuestion, GroupRule, ImportWizard, InstallWizard, Step, TtwWizard,
};
use raul_ui_service::dto::Finding;
use raul_ui_service::fake::sample;
use raul_ui_service::wiring::{
    ImportContext, InstallContext, TtwContext, apply_import, apply_install, apply_ttw, plan_import,
    plan_install, plan_ttw,
};
use raul_ui_service::{FakeService, InstallResult, ProfileRequest, Service, ServiceError};

fn refused(what: &str) -> ServiceError {
    let mut e = ViewError::new(ExitClass::Refused, what, "Nothing was changed");
    e.retry = Retry::AfterFix("fix it".into());
    ServiceError::Failed(e)
}

fn ctx() -> InstallContext {
    InstallContext {
        instance: "/inst".into(),
        profile: None,
        name: None,
    }
}

fn question() -> FomodQuestion {
    FomodQuestion {
        id: "Options/Style".into(),
        step: "Options".into(),
        group: "Style".into(),
        rule: GroupRule::SelectExactlyOne,
        min: 1,
        max: 1,
        options: ["Red", "Blue"]
            .iter()
            .map(|n| FomodOption {
                name: (*n).into(),
                description: String::new(),
                kind: None,
            })
            .collect(),
    }
}

#[test]
fn the_default_profile_is_made_for_an_instance_that_has_none() {
    let svc = FakeService::new();
    svc.on_profile_create(Ok(()));
    svc.profile_create(&ProfileRequest::default_for("/inst"))
        .unwrap();
    let calls = svc.profile_create_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        (calls[0].instance.as_str(), calls[0].name.as_str()),
        ("/inst", "Default")
    );
    assert_eq!(
        calls[0].args(),
        ["create", "--name", "Default", "--instance", "/inst"]
    );
}

#[test]
fn a_plain_install_plans_with_a_dry_run_and_applies_after_review() {
    let svc = FakeService::new();
    svc.on_install(Ok(InstallResult::Report(sample::install(
        true,
        &["a.esp", "b.nif"],
    ))));
    svc.on_install(Ok(InstallResult::Report(sample::install(
        false,
        &["a.esp", "b.nif"],
    ))));
    let mut w = InstallWizard::new();
    w.choose_archive("/x/a.zip").unwrap();
    plan_install(&mut w, &svc, &ctx()).unwrap();
    assert_eq!(w.flow.step(), Step::Review);
    assert_eq!(w.planned_files(), ["a.esp", "b.nif"]);
    assert_eq!(
        apply_install(&mut w, &svc, &ctx()),
        Err(FlowError::NotReviewed)
    );
    w.flow.mark_reviewed().unwrap();
    apply_install(&mut w, &svc, &ctx()).unwrap();
    assert_eq!(w.flow.result(), Some((ExitClass::Ok, Outcome::Completed)));
    let calls = svc.install_calls();
    assert!(calls[0].dry_run && !calls[1].dry_run);
    assert_eq!(calls[0].archive, "/x/a.zip");
}

#[test]
fn a_fomod_hands_over_to_the_question_wizard_and_sends_the_answers_back() {
    let svc = FakeService::new();
    svc.on_install(Ok(InstallResult::Questions(vec![question()])));
    svc.on_install(Ok(InstallResult::Report(sample::install(true, &["s.txt"]))));
    let mut w = InstallWizard::new();
    w.choose_archive("/x/a.zip").unwrap();
    plan_install(&mut w, &svc, &ctx()).unwrap();
    assert_eq!(w.flow.step(), Step::Input);
    assert!(w.fomod().is_some());
    assert_eq!(w.submit_answers(), Err(FlowError::Blocked(1)));
    w.fomod_mut()
        .unwrap()
        .answer("Options/Style", &["Blue"])
        .unwrap();
    w.submit_answers().unwrap();
    plan_install(&mut w, &svc, &ctx()).unwrap();
    assert_eq!(w.flow.step(), Step::Review);
    let answers = svc.install_calls()[1].answers.clone().unwrap();
    assert_eq!(answers["Options/Style"][0], "Blue");
}

#[test]
fn a_refused_plan_returns_to_the_input_with_the_error() {
    let svc = FakeService::new();
    svc.on_install(Err(refused("no such archive")));
    let mut w = InstallWizard::new();
    w.choose_archive("/x/a.zip").unwrap();
    plan_install(&mut w, &svc, &ctx()).unwrap();
    assert_eq!(w.flow.step(), Step::Input);
    assert_eq!(w.flow.error().unwrap().what, "no such archive");
}

#[test]
fn a_failed_install_ends_as_a_rollback_or_a_refusal() {
    for (class, outcome) in [
        (ExitClass::StepFailedRolledBack, Outcome::RolledBack),
        (ExitClass::Refused, Outcome::Refused),
    ] {
        let svc = FakeService::new();
        svc.on_install(Ok(InstallResult::Report(sample::install(true, &[]))));
        svc.on_install(Err(ServiceError::Failed(ViewError::new(
            class, "boom", "x",
        ))));
        let mut w = InstallWizard::new();
        w.choose_archive("/x/a.zip").unwrap();
        plan_install(&mut w, &svc, &ctx()).unwrap();
        w.flow.mark_reviewed().unwrap();
        apply_install(&mut w, &svc, &ctx()).unwrap();
        assert_eq!(w.flow.result(), Some((class, outcome)));
    }
}

fn mo2_ready() -> ImportWizard {
    let mut w = ImportWizard::new();
    w.set_mo2("/mo2");
    w.set_profile("P");
    w.set_copy_only(true);
    plan_import(&mut w).unwrap();
    w.flow.mark_reviewed().unwrap();
    w
}

fn import_ctx() -> ImportContext {
    ImportContext {
        instance: "/new".into(),
        base: "/base".into(),
    }
}

#[test]
fn an_import_creates_the_instance_with_the_wizards_choices() {
    let svc = FakeService::new();
    svc.on_import(Ok(sample::import()));
    let mut w = mo2_ready();
    apply_import(&mut w, &svc, &import_ctx()).unwrap();
    assert_eq!(w.flow.result(), Some((ExitClass::Ok, Outcome::Completed)));
    let c = &svc.import_calls()[0];
    assert_eq!(
        (c.mo2.as_str(), c.profile.as_deref(), c.copy_only),
        ("/mo2", Some("P"), true)
    );
}

#[test]
fn an_import_whose_profile_files_disagree_returns_to_the_review_with_blockers() {
    let svc = FakeService::new();
    svc.on_import(Err(ServiceError::Disagree(vec![Finding {
        code: "import.list-missing".into(),
        severity: "error".into(),
        subject: "plugins.txt".into(),
        message: "missing".into(),
    }])));
    let mut w = mo2_ready();
    apply_import(&mut w, &svc, &import_ctx()).unwrap();
    assert_eq!(w.flow.step(), Step::Review);
    assert_eq!(w.blockers()[0].text, "plugins.txt: missing");
    assert!(!w.flow.can_apply(), "the rows must be seen again");
}

#[test]
fn ttw_needs_its_inputs_then_runs_and_its_status_is_the_outcome() {
    let mut w = TtwWizard::new();
    assert!(plan_ttw(&mut w).is_err());
    w.set_fallout3("/fo3");
    w.set_package("/ttw.mpi");
    plan_ttw(&mut w).unwrap();
    w.flow.mark_reviewed().unwrap();
    let svc = FakeService::new();
    svc.on_ttw(Ok(sample::ttw_failed("ttwright_failed")));
    let c = TtwContext {
        instance: Some("/inst".into()),
        resume: true,
        ..TtwContext::default()
    };
    apply_ttw(&mut w, &svc, &c).unwrap();
    assert_eq!(
        w.flow.result(),
        Some((ExitClass::StepFailedRolledBack, Outcome::RolledBack))
    );
    let call = &svc.ttw_calls()[0];
    assert_eq!(call.fallout3.as_deref(), Some("/fo3"));
    assert!(call.resume && !call.dry_run);
    assert_eq!(
        call.args(),
        [
            "--instance",
            "/inst",
            "--fallout3",
            "/fo3",
            "--package",
            "/ttw.mpi",
            "--resume"
        ]
    );

    let mut w = TtwWizard::new();
    w.set_fallout3("/fo3");
    w.set_package("/ttw.mpi");
    plan_ttw(&mut w).unwrap();
    w.flow.mark_reviewed().unwrap();
    svc.on_ttw(Ok(sample::ttw()));
    apply_ttw(&mut w, &svc, &c).unwrap();
    assert_eq!(w.flow.result(), Some((ExitClass::Ok, Outcome::Completed)));
}
