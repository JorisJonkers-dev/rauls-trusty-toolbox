// SPDX-License-Identifier: GPL-3.0-only
//! The plan: which steps a [`Request`] asks for, in nvmod's order, and which of them are skipped.

use super::{Planned, Request, SkipReason, Step};

/// The steps of `request` in the order nvmod's `setup` runs them: the MO2 import, or init and the first Profile;
/// the target when nv-rs is wanted; TTW; each Mod archive; Staging; the sort (skipped on an
/// imported Instance unless asked for) or, on an imported Instance, the stamp; and the check. Steps the request
/// does not ask for are left out; steps it lists as done are planned as skipped.
#[must_use]
pub fn plan(request: &Request) -> Vec<Planned> {
    let imported = request.import_mo2.is_some();
    let keeps_order = imported && !request.sort;
    let mut steps = Vec::new();
    if imported {
        steps.push(Step::ImportMo2);
    } else {
        steps.push(Step::Init);
        steps.push(Step::Profile);
    }
    if request.engine {
        steps.push(Step::Target);
    }
    if request.ttw.is_some() {
        steps.push(Step::Ttw);
    }
    steps.extend(request.mods.iter().cloned().map(Step::Mod));
    steps.push(Step::Stage);
    steps.push(Step::Sort);
    if keeps_order {
        steps.push(Step::Stamp);
    }
    steps.push(Step::Verify);
    steps
        .into_iter()
        .map(|step| {
            let skip = if request.done.contains(&step) {
                Some(SkipReason::Done)
            } else if step == Step::Sort && keeps_order {
                Some(SkipReason::KeepsImportedOrder)
            } else {
                None
            };
            Planned { step, skip }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::Ttw;
    use std::path::PathBuf;

    fn request() -> Request {
        Request {
            base: PathBuf::from("/base"),
            instance: PathBuf::from("/inst"),
            masterlist: PathBuf::from("/masterlist.yaml"),
            profile: None,
            import_mo2: None,
            copy: false,
            engine: false,
            ttw: None,
            mods: Vec::new(),
            answers: None,
            sort: false,
            catalog: None,
            done: Vec::new(),
        }
    }

    fn ids(planned: &[Planned]) -> Vec<String> {
        planned.iter().map(|p| p.step.id()).collect()
    }

    #[test]
    fn a_fresh_instance_with_everything_has_the_order_of_nvmod_setup() {
        let mut r = request();
        r.engine = true;
        r.ttw = Some(Ttw {
            fallout3: PathBuf::from("/fo3"),
            newvegas: None,
            package: None,
            resume: false,
            timeout: None,
        });
        r.mods = vec![PathBuf::from("/m/A.zip"), PathBuf::from("/m/B.7z")];
        let planned = plan(&r);
        assert_eq!(
            ids(&planned),
            [
                "init",
                "profile",
                "target",
                "ttw",
                "mod:A.zip",
                "mod:B.7z",
                "stage",
                "sort",
                "verify"
            ]
        );
        assert!(planned.iter().all(|p| p.skip.is_none()));
    }

    #[test]
    fn the_smallest_plan_is_init_profile_stage_sort_verify() {
        assert_eq!(
            ids(&plan(&request())),
            ["init", "profile", "stage", "sort", "verify"]
        );
    }

    #[test]
    fn an_imported_instance_imports_and_keeps_its_order_with_a_stamp() {
        let mut r = request();
        r.import_mo2 = Some(PathBuf::from("/mo2"));
        r.engine = true;
        let planned = plan(&r);
        assert_eq!(
            ids(&planned),
            ["import-mo2", "target", "stage", "sort", "stamp", "verify"]
        );
        let sort = planned.iter().find(|p| p.step == Step::Sort).unwrap();
        assert_eq!(sort.skip, Some(SkipReason::KeepsImportedOrder));
        assert_eq!(planned.iter().filter(|p| p.skip.is_some()).count(), 1);
    }

    #[test]
    fn asking_for_the_sort_on_an_imported_instance_sorts_and_does_not_stamp() {
        let mut r = request();
        r.import_mo2 = Some(PathBuf::from("/mo2"));
        r.sort = true;
        let planned = plan(&r);
        assert_eq!(ids(&planned), ["import-mo2", "stage", "sort", "verify"]);
        assert!(planned.iter().all(|p| p.skip.is_none()));
    }

    #[test]
    fn steps_listed_as_done_are_planned_as_skipped() {
        let mut r = request();
        r.mods = vec![PathBuf::from("/m/A.zip")];
        r.done = vec![
            Step::Init,
            Step::Profile,
            Step::Mod(PathBuf::from("/m/A.zip")),
        ];
        let skipped: Vec<String> = plan(&r)
            .iter()
            .filter(|p| p.skip == Some(SkipReason::Done))
            .map(|p| p.step.id())
            .collect();
        assert_eq!(skipped, ["init", "profile", "mod:A.zip"]);
    }

    #[test]
    fn a_request_round_trips_through_json_with_its_defaults() {
        let json = r#"{"base":"/base","instance":"/inst","masterlist":"/masterlist.yaml"}"#;
        let r: Request = serde_json::from_str(json).unwrap();
        assert_eq!(r, request());
        let back: Request = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(back, r);
    }
}
