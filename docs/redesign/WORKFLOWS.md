# Delivery workflow

How to deliver the tickets in `TICKETS.md`: the order, what can run side by side, and how each ticket is checked.

## Milestones

| Milestone | Goal | Tickets |
|---|---|---|
| M0 Foundations | The look, the shell and the state that every page needs. The app looks like the demo's frame with empty pages. | GUI-01, GUI-02, GUI-03, GUI-04, CORE-01, CORE-02, CORE-03, CORE-10 |
| M1 First start to first play | A new player can set up, see Home, press Play and stop. Use case UC-1. | GUI-05, GUI-06, GUI-07, GUI-08, GUI-09, CORE-04, CORE-05, CORE-06, CORE-08, CORE-09, NV-05, NV-06 |
| M2 Mods, order, LOD, TTW, profiles | Add and organise mods, change the load order, build LOD, add TTW, manage profiles. Use cases UC-2, 3, 4, 6, 7, 8. | GUI-10, GUI-11, GUI-12, GUI-16, GUI-17, GUI-18, GUI-19, NVMOD-01, NVMOD-02, LOOT-01, LOD-01, TTW-01 |
| M3 Test, versions, update, doctor, settings | Keep a stable setup, update, troubleshoot. Use cases UC-5, 9, 10, 11. | GUI-13, GUI-14, GUI-15, GUI-20, GUI-21, GUI-22, CORE-07, LOOT-02, NV-01, NV-02, NV-03, NV-04 |
| M4 Acceptance | Every one of the 89 design pictures matched, every Gherkin scenario passing, demo-bug decisions applied, keyboard and display-option walk. | no new tickets; closes gaps found |

## Order and parallel lanes

Component work (nvmod-rs, loot-rs, lodwright-rs, ttwright-rs, nv-rs) does not depend on the GUI. It can start in M0 so
the interface is ready one milestone before its page. Each component ticket first publishes its API and a fake of it
(so the GUI can build against the fake), then the real work.

Lanes that can run at the same time inside a milestone:

- M0: lane A theme and widgets (GUI-01, GUI-02), then shell (GUI-03) and overlays (GUI-04); lane B state, checks and jobs (CORE-01, CORE-02, CORE-03); lane C the test fake (CORE-10). Lane B and C do not touch GUI code.
- M1: lane A wizard (GUI-08 with CORE-06, NV-05); lane B Home and Play (GUI-09 with CORE-04, CORE-05, NV-06); lane C small overlays (GUI-05, GUI-06, GUI-07, CORE-08, CORE-09).
- M2: lane A Mods list (GUI-10, then GUI-11 and GUI-12 in parallel; with NVMOD-01, NVMOD-02); lane B Load order (GUI-17 with LOOT-01); lane C LOD and TTW (GUI-18 with LOD-01, GUI-19 with TTW-01); lane D profiles (GUI-16).
- M3: lane A tests (GUI-13, GUI-14 with NV-01 to NV-04); lane B versions (GUI-15); lane C System and Doctor (GUI-20, GUI-21, CORE-07, LOOT-02); lane D Settings (GUI-22).

Keep each ticket small enough for one pull request. One writer per file: GUI tickets that touch the same page file
(`GUI-10`, `GUI-11`, `GUI-12` on the Mods page) are done one after another, not at once. Component changes land in the
component repository first, then the Toolbox bumps the pointer.

Order inside the GUI lane: widgets before pages, pages before their polish; the Mods page (GUI-10) before groups,
install, test tab and versions, because they sit on it.

## How each ticket is checked

Every ticket is done only when all four checks pass.

1. Unit tests. Rules that are logic (badge counts, checks, undo stack, version folding, signature, sort moves, toast
   limits) get unit tests in the model crate (`crates/raul-gui/crates/raul-ui-model`) or the component. Each rule in
   `FUNCTIONALITY.md` and `SPEC.md` section 8 needs a test that fails without it.
2. The Gherkin scenario against the app. For every story id listed in the ticket, the scenario in `features/<area>.feature`
   (tag `@US-...`) is written as a headless `egui_kittest` test (see README). It must use the exact UI text. Component
   tickets are checked by the same scenario with the component behind the real service (an end-to-end test in
   `raul-ui-service/tests/e2e.rs`) rather than the fake.
3. A screenshot compared to the design picture. The state of the scenario is rendered and compared to the matching
   file in `../design/screenshots/` (or `screens/<area>/` for states the design set does not have) with a snapshot test. A
   reviewer looks at the two side by side, labelled, before the snapshot is accepted. Measured sizes from `SPEC.md`
   section 5 are asserted in code, not only by eye.
4. Keyboard and display options. Tab order, Enter and Escape of the ticket's dialogs and menus, and the three
   display options (high contrast, larger text, reduce motion) are exercised for any ticket that adds a surface.

Review: every change gets a second read and a red-team pass before it merges. Regressions get an issue, a record and
a regression test before they are closed.

## Milestone gates

- M0 gate: the shell scenarios US-SHELL-01 to 15 pass against an empty app; the snapshot of the frame matches `../design/screenshots/18`.
- M1 gate: UC-1 passes end to end against the fake service, and once against the real engine on a test machine.
- M2 gate: UC-2, 3, 4, 6, 7, 8 pass; load order and LOD pictures match `58` to `61` and `26` to `31`.
- M3 gate: UC-5, 9, 10, 11 pass; pictures `49` to `56` and `72` to `86` match.
- M4 gate: all 113 scenarios pass, all 89 design pictures have a snapshot, the owner has seen the app side by side with the demo.

## Reporting

Each milestone ends with a short list: tickets closed, scenarios passing, pictures matched, open items. The gap table
in `GAP-AUDIT.md` is rerun at each gate and its counts go in that list.
