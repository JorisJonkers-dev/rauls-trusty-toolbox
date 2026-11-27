# Potential tickets

One ticket per coherent piece of work. Ids: `GUI-nn`, `CORE-nn`, `NVMOD-nn`, `LOOT-nn`, `LOD-nn`, `TTW-nn`, `NV-nn`.
F-ids are in `FUNCTIONALITY.md`; the current state of each is in `GAP-AUDIT.md`. Milestones are in `WORKFLOWS.md`.

## Division between projects

| Project | Owns | Why |
|---|---|---|
| raul GUI (`crates/raul-gui`) | Everything drawn and every input: theme, widgets, shell, pages, dialogs, menus, palette, focus and keyboard, view models (`raul-ui-model`), headless UI tests and snapshots. | The GUI is a thin view: it shows state and sends intents. It holds no business rules except view rules (what to show for a state). |
| raul core/orchestration (`raul-core`, app logic in the Toolbox) | State and rules that cross pages: profiles, shared versus per-profile data, versions and undo, the checks model that feeds badges, Home, Play and Doctor, jobs, play flow, setup flow, update flow, settings, the fake service. | Each rule has one home and one test; pages only read it. The Components stay free of app concepts such as profiles and badges. |
| nvmod-rs | Archives and the mod library: inspect, install options, staged install, plugin lists, kinds, missing masters, remove. | It already manages mods; the install dialog and the Mods list show its data. |
| loot-rs | Sorting and masters: proposed order with reasons, masterlist notes, missing masters, masterlist update. | The Load order page and the Sort preview are views of its result. |
| lodwright-rs | LOD builds: stages, progress, estimates, quality presets, fine-tune parameters, stale detection input, cancel that keeps the old LOD. | It does the build; the LOD pages show its progress and parameters. |
| ttwright-rs | The TTW install: Fallout 3 detection, installer file check, space estimate, staged install, update check. | The four-step flow is a view of its result. |
| nv-rs (engine) | Things only the engine can do: engine install, builds and repair, game launch and stop, stability runs, benchmark runs, location picture sets, run logs, graphics settings applied per profile. | The engine is under our control, so runs work without the main menu and without scripts. |

Rule of thumb: if the work needs the game, it is nv-rs. If it needs a file format or an install, it is a Component. If
it holds a rule that more than one page reads, it is core. If it is drawn or clicked, it is GUI.

## Tickets

| Id | Title | Project | Size | Milestone | Depends on | Gap now |
|---|---|---|---|---|---|---|
| GUI-01 | Theme, fonts, tokens and motion | raul GUI (crates/raul-gui) | M | M0 | none | 0/1/3 |
| GUI-02 | Widget kit: card, section, row, button, tag, switch, checkbox, segmented control, input, banner, progress | raul GUI (crates/raul-gui) | L | M0 | GUI-01 | 0/2/1 |
| GUI-03 | App shell: window, top bar, sidebar, badges, status line, floating action bar | raul GUI (crates/raul-gui) | L | M0 | GUI-01, GUI-02 | 0/8/1 |
| GUI-04 | Overlays: toasts, dialog frame, confirm, name dialog, menus | raul GUI (crates/raul-gui) | L | M0 | GUI-02 | 0/1/5 |
| GUI-05 | Folder picker dialog | raul GUI (crates/raul-gui) | S | M1 | GUI-04 | 0/0/1 |
| GUI-06 | Command palette and shortcuts sheet | raul GUI (crates/raul-gui) | M | M1 | GUI-04, CORE-09 | 0/2/1 |
| GUI-07 | Focus look and keyboard map | raul GUI (crates/raul-gui) | M | M1 | GUI-02, GUI-03 | 0/2/0 |
| GUI-08 | Setup wizard pages | raul GUI (crates/raul-gui) | L | M1 | GUI-03, GUI-05, CORE-06 | 0/6/4 |
| GUI-09 | Home page and play-check cards | raul GUI (crates/raul-gui) | L | M1 | GUI-03, CORE-02, CORE-04, CORE-05 | 0/4/4 |
| GUI-10 | Mods page: frame, search, filters, built-in section, rows, details, empty states, TTW row | raul GUI (crates/raul-gui) | L | M2 | GUI-03, NVMOD-02, CORE-01, CORE-02 | 0/9/1 |
| GUI-11 | Groups and drag and drop | raul GUI (crates/raul-gui) | M | M2 | GUI-10, CORE-01 | 0/0/2 |
| GUI-12 | Install dialog | raul GUI (crates/raul-gui) | M | M2 | GUI-04, NVMOD-01 | 0/4/1 |
| GUI-13 | Test tab: stability, known stable, benchmark | raul GUI (crates/raul-gui) | M | M3 | GUI-10, CORE-05, NV-01, NV-02 | 0/0/3 |
| GUI-14 | Location pictures viewer and log | raul GUI (crates/raul-gui) | M | M3 | GUI-13, NV-03, NV-04 | 0/0/3 |
| GUI-15 | Versions tab and revert | raul GUI (crates/raul-gui) | M | M3 | GUI-10, CORE-01 | 0/0/3 |
| GUI-16 | Profiles UI: pill, menu, dialogs | raul GUI (crates/raul-gui) | M | M2 | GUI-04, CORE-01 | 0/0/4 |
| GUI-17 | Load order page and sort preview | raul GUI (crates/raul-gui) | L | M2 | GUI-10, LOOT-01 | 0/2/4 |
| GUI-18 | LOD page and LOD settings page | raul GUI (crates/raul-gui) | M | M2 | GUI-03, LOD-01 | 0/4/2 |
| GUI-19 | Tale of Two Wastelands flow | raul GUI (crates/raul-gui) | M | M2 | GUI-05, TTW-01 | 0/5/0 |
| GUI-20 | System page | raul GUI (crates/raul-gui) | M | M3 | GUI-03, CORE-07 | 0/2/3 |
| GUI-21 | Doctor page | raul GUI (crates/raul-gui) | S | M3 | GUI-03, CORE-02 | 0/0/3 |
| GUI-22 | Settings page | raul GUI (crates/raul-gui) | M | M3 | GUI-03, CORE-08 | 0/1/5 |
| CORE-01 | App state, profiles, shared library, versions and undo | raul core/orchestration (raul-core, Toolbox app logic) | L | M0 | none | 0/1/6 |
| CORE-02 | Checks model for badges, Home rows, Doctor and Play checks | raul core/orchestration (raul-core, Toolbox app logic) | L | M0 | CORE-01 | 0/3/4 |
| CORE-03 | Job runner: progress, cancel, per-profile keys, status line, resume | raul core/orchestration (raul-core, Toolbox app logic) | M | M0 | none | 0/2/0 |
| CORE-04 | Play orchestration: check, start, run, stop, playtime | raul core/orchestration (raul-core, Toolbox app logic) | M | M1 | CORE-02, CORE-03, NV-06 | 0/2/3 |
| CORE-05 | Stability recommendation and known-stable logic | raul core/orchestration (raul-core, Toolbox app logic) | M | M1 | CORE-01 | 0/0/3 |
| CORE-06 | Setup wizard orchestration | raul core/orchestration (raul-core, Toolbox app logic) | M | M1 | CORE-03, NV-05 | 0/5/3 |
| CORE-07 | Update orchestration: check, update parts, engine builds | raul core/orchestration (raul-core, Toolbox app logic) | L | M3 | CORE-03, NV-05 | 0/2/2 |
| CORE-08 | Settings, game folders, storage and cache | raul core/orchestration (raul-core, Toolbox app logic) | M | M1 | CORE-01 | 0/2/5 |
| CORE-09 | Palette index and action registry | raul core/orchestration (raul-core, Toolbox app logic) | S | M1 | none | 0/1/0 |
| CORE-10 | Fake service and scenario states for app tests | raul core/orchestration (raul-core, Toolbox app logic) | M | M0 | none | 0/0/1 |
| NVMOD-01 | Archive inspection and install with options | nvmod-rs | L | M2 | none | 0/4/1 |
| NVMOD-02 | Mod library: toggle, remove, plugins, kinds, masters | nvmod-rs | M | M2 | none | 0/5/0 |
| LOOT-01 | Sort preview with reasons, notes and missing masters | loot-rs | M | M2 | none | 0/2/1 |
| LOOT-02 | Masterlist update with message count | loot-rs | S | M3 | LOOT-01 | 0/0/1 |
| LOD-01 | Build with named stages, estimates and fine-tune parameters | lodwright-rs | M | M2 | none | 0/4/2 |
| TTW-01 | TTW install: Fallout 3 detection, installer file, space check, staged install, update check | ttwright-rs | M | M2 | none | 0/5/0 |
| NV-01 | Stability run over 40 places | nv-rs | L | M3 | none | 0/0/1 |
| NV-02 | Benchmark run | nv-rs | M | M3 | none | 0/0/1 |
| NV-03 | Location picture sets | nv-rs | M | M3 | none | 0/0/1 |
| NV-04 | Run log with levels | nv-rs | S | M3 | none | 0/0/1 |
| NV-05 | Engine install, builds, integrity check and repair | nv-rs | M | M1 | none | 0/1/2 |
| NV-06 | Game launch, stop and session time; per-profile graphics settings | nv-rs | M | M1 | none | 0/1/1 |

Gap now is exists / differs / missing across the ticket's functions.

### GUI-01 Theme, fonts, tokens and motion

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M0. Depends on: none.
- Functions: F-ACCESS-04, F-ACCESS-05, F-ACCESS-06, F-OVERLAY-11
- Stories: US-SHELL-10, US-SHELL-11
- Gap now (exists/differs/missing): 0/1/3
- Acceptance: Palette from SPEC section 2 (default and high contrast), Inter bundled, 14 px base and 15.5 px larger text, every animation of SPEC section 6 with the reduce-motion variant, spinners spin, progress bars with sheen. Picture check: `../design/screenshots/84-settings-high-contrast.png`, `89-far-building-reduced-motion.png`.
- Gherkin: `features/shell.feature`, scenarios `@US-SHELL-10` `@US-SHELL-11`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-02 Widget kit: card, section, row, button, tag, switch, checkbox, segmented control, input, banner, progress

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M0. Depends on: GUI-01.
- Functions: F-SHELL-07, F-SHELL-08, F-ACCESS-02
- Stories: US-SHELL-08, US-SHELL-09, US-SHELL-13
- Gap now (exists/differs/missing): 0/2/1
- Acceptance: Every size and inset of SPEC section 4; header and rows share the 12 px inset; whole-row click targets; cards grow to page width. Measured values of SPEC section 5 asserted in a layout test.
- Gherkin: `features/shell.feature`, scenarios `@US-SHELL-08` `@US-SHELL-09` `@US-SHELL-13`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-03 App shell: window, top bar, sidebar, badges, status line, floating action bar

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M0. Depends on: GUI-01, GUI-02.
- Functions: F-SHELL-01, F-SHELL-02, F-SHELL-03, F-SHELL-04, F-SHELL-05, F-SHELL-06, F-SHELL-08, F-SHELL-09, F-SHELL-10
- Stories: US-SHELL-01, US-SHELL-09, US-SHELL-12, US-SHELL-14, US-START-16
- Gap now (exists/differs/missing): 0/8/1
- Acceptance: Window grows when maximised; top bar chips navigate; Play states; badge colours and tooltips; footer variants; action bar 48 px pill at 20/14 from the corner.
- Gherkin: `features/shell.feature`, `features/start.feature`, scenarios `@US-SHELL-01` `@US-SHELL-09` `@US-SHELL-12` `@US-SHELL-14` `@US-START-16`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-04 Overlays: toasts, dialog frame, confirm, name dialog, menus

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M0. Depends on: GUI-02.
- Functions: F-OVERLAY-01, F-OVERLAY-03, F-OVERLAY-04, F-OVERLAY-05, F-OVERLAY-07, F-OVERLAY-10
- Stories: US-SHELL-04, US-SHELL-05, US-SHELL-06, US-SHELL-07
- Gap now (exists/differs/missing): 0/1/5
- Acceptance: 440 px toasts with Undo and X, max 2, 3.5 s / 5 s, hidden under dialogs; dialog Enter/Escape/Tab/scrim rules; menu placement and keyboard; quit actually quits.
- Gherkin: `features/shell.feature`, scenarios `@US-SHELL-04` `@US-SHELL-05` `@US-SHELL-06` `@US-SHELL-07`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-05 Folder picker dialog

- Project: raul GUI (crates/raul-gui)
- Size: S. Milestone: M1. Depends on: GUI-04.
- Functions: F-OVERLAY-06
- Stories: US-SHELL-15, US-START-02, US-START-03
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: Title per use, free space per folder, Use this folder disabled until a selection, arrow keys.
- Gherkin: `features/shell.feature`, `features/start.feature`, scenarios `@US-SHELL-15` `@US-START-02` `@US-START-03`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-06 Command palette and shortcuts sheet

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M1. Depends on: GUI-04, CORE-09.
- Functions: F-OVERLAY-08, F-OVERLAY-09, F-OVERLAY-12
- Stories: US-SHELL-02, US-SHELL-03, US-MODS-30
- Gap now (exists/differs/missing): 0/2/1
- Acceptance: Ctrl K toggles; items and kind tags; mod jumps and opens the row; ? opens the sheet when allowed.
- Gherkin: `features/shell.feature`, `features/mods.feature`, scenarios `@US-SHELL-02` `@US-SHELL-03` `@US-MODS-30`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-07 Focus look and keyboard map

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M1. Depends on: GUI-02, GUI-03.
- Functions: F-ACCESS-01, F-ACCESS-03
- Stories: US-SHELL-01, US-SHELL-05, US-SHELL-15
- Gap now (exists/differs/missing): 0/2/0
- Acceptance: Caret-only text fields, one 1 px keyboard indicator, row tint, Tab order of SPEC section 7, Alt+Up/Down, Escape order, focus return.
- Gherkin: `features/shell.feature`, scenarios `@US-SHELL-01` `@US-SHELL-05` `@US-SHELL-15`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-08 Setup wizard pages

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M1. Depends on: GUI-03, GUI-05, CORE-06.
- Functions: F-SETUP-01, F-SETUP-02, F-SETUP-03, F-SETUP-04, F-SETUP-05, F-SETUP-06, F-SETUP-07, F-SETUP-08, F-SETUP-09, F-SETUP-10
- Stories: US-START-01, US-START-02, US-START-03, US-START-04, US-START-05, US-START-06, US-START-07, US-START-08, US-START-09, US-START-12, US-START-17
- Gap now (exists/differs/missing): 0/6/4
- Acceptance: All eight steps with every state of `../design/screenshots/01` to `17` and `87`, `88`.
- Gherkin: `features/start.feature`, scenarios `@US-START-01` `@US-START-02` `@US-START-03` `@US-START-04` `@US-START-05` `@US-START-06` `@US-START-07` `@US-START-08` `@US-START-09` `@US-START-12` `@US-START-17`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-09 Home page and play-check cards

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M1. Depends on: GUI-03, CORE-02, CORE-04, CORE-05.
- Functions: F-HOME-01, F-HOME-02, F-HOME-03, F-HOME-04, F-PLAY-01, F-PLAY-02, F-PLAY-03, F-PLAY-04
- Stories: US-START-10, US-START-11, US-START-13, US-START-14, US-START-15, US-TESTS-13
- Gap now (exists/differs/missing): 0/4/4
- Acceptance: Status card words, setup rows, banner, checking, warnings and failed cards, running state, playtime.
- Gherkin: `features/start.feature`, `features/tests.feature`, scenarios `@US-START-10` `@US-START-11` `@US-START-13` `@US-START-14` `@US-START-15` `@US-TESTS-13`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-10 Mods page: frame, search, filters, built-in section, rows, details, empty states, TTW row

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M2. Depends on: GUI-03, NVMOD-02, CORE-01, CORE-02.
- Functions: F-MODS-01, F-MODS-02, F-MODS-03, F-MODS-04, F-MODS-05, F-MODS-07, F-MODS-08, F-MODS-10, F-MODS-11, F-MODS-12
- Stories: US-MODS-01, US-MODS-02, US-MODS-03, US-MODS-04, US-MODS-05, US-MODS-06, US-MODS-14, US-MODS-15, US-MODS-16, US-MODS-18, US-MODS-19, US-MODS-20, US-MODS-21, US-MODS-22, US-MODS-24, US-MODS-25
- Gap now (exists/differs/missing): 0/9/1
- Acceptance: Click a row to fold details downwards; one checkbox per row; filters with counts; remove with confirm; Undo button in the action bar.
- Gherkin: `features/mods.feature`, scenarios `@US-MODS-01` `@US-MODS-02` `@US-MODS-03` `@US-MODS-04` `@US-MODS-05` `@US-MODS-06` `@US-MODS-14` `@US-MODS-15` `@US-MODS-16` `@US-MODS-18` `@US-MODS-19` `@US-MODS-20` `@US-MODS-21` `@US-MODS-22` `@US-MODS-24` `@US-MODS-25`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-11 Groups and drag and drop

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M2. Depends on: GUI-10, CORE-01.
- Functions: F-MODS-06, F-MODS-09
- Stories: US-MODS-07, US-MODS-08, US-MODS-09, US-MODS-10, US-MODS-11, US-MODS-12, US-MODS-13, US-MODS-17
- Gap now (exists/differs/missing): 0/0/2
- Acceptance: Create, rename, delete, fold, tri-state switch, move; pointer drag with ghost and drop line, auto-scroll, Escape cancels.
- Gherkin: `features/mods.feature`, scenarios `@US-MODS-07` `@US-MODS-08` `@US-MODS-09` `@US-MODS-10` `@US-MODS-11` `@US-MODS-12` `@US-MODS-13` `@US-MODS-17`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-12 Install dialog

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M2. Depends on: GUI-04, NVMOD-01.
- Functions: F-INSTALL-01, F-INSTALL-02, F-INSTALL-03, F-INSTALL-04, F-INSTALL-05
- Stories: US-MODS-26, US-MODS-27, US-MODS-28, US-MODS-29
- Gap now (exists/differs/missing): 0/4/1
- Acceptance: Pick, options pages with preview, review with group, busy with cancel, broken archive.
- Gherkin: `features/mods.feature`, scenarios `@US-MODS-26` `@US-MODS-27` `@US-MODS-28` `@US-MODS-29`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-13 Test tab: stability, known stable, benchmark

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M3. Depends on: GUI-10, CORE-05, NV-01, NV-02.
- Functions: F-TEST-01, F-TEST-02, F-TEST-03
- Stories: US-TESTS-01, US-TESTS-02, US-TESTS-03, US-TESTS-04, US-TESTS-05, US-TESTS-06
- Gap now (exists/differs/missing): 0/0/3
- Acceptance: Live stage text, results, Known stable fold with Matches now and Forget, benchmark tiles and chart with point menu.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-01` `@US-TESTS-02` `@US-TESTS-03` `@US-TESTS-04` `@US-TESTS-05` `@US-TESTS-06`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-14 Location pictures viewer and log

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M3. Depends on: GUI-13, NV-03, NV-04.
- Functions: F-TEST-04, F-TEST-05, F-TEST-06
- Stories: US-TESTS-07, US-TESTS-08, US-TESTS-09, US-TESTS-10
- Gap now (exists/differs/missing): 0/0/3
- Acceptance: A/B sets, world tabs, place strip with All N, search, wheel zoom, drag pan, double-click reset, log filters.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-07` `@US-TESTS-08` `@US-TESTS-09` `@US-TESTS-10`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-15 Versions tab and revert

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M3. Depends on: GUI-10, CORE-01.
- Functions: F-VERS-01, F-VERS-02, F-VERS-03
- Stories: US-TESTS-11, US-TESTS-12
- Gap now (exists/differs/missing): 0/0/3
- Acceptance: Timeline, chips, expand, Revert and Use this version with confirm, Revert to last stable.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-11` `@US-TESTS-12`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-16 Profiles UI: pill, menu, dialogs

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M2. Depends on: GUI-04, CORE-01.
- Functions: F-PROF-01, F-PROF-02, F-PROF-03, F-PROF-04
- Stories: US-TESTS-14, US-TESTS-15, US-TESTS-16, US-TESTS-17, US-TESTS-18, US-TESTS-19, US-PAGES-08
- Gap now (exists/differs/missing): 0/0/4
- Acceptance: Menu rules, name validation, delete confirm, switch toast, pill in headers.
- Gherkin: `features/tests.feature`, `features/pages.feature`, scenarios `@US-TESTS-14` `@US-TESTS-15` `@US-TESTS-16` `@US-TESTS-17` `@US-TESTS-18` `@US-TESTS-19` `@US-PAGES-08`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-17 Load order page and sort preview

- Project: raul GUI (crates/raul-gui)
- Size: L. Milestone: M2. Depends on: GUI-10, LOOT-01.
- Functions: F-LOAD-01, F-LOAD-02, F-LOAD-03, F-LOAD-04, F-LOAD-05, F-LOAD-06
- Stories: US-PAGES-01, US-PAGES-02, US-PAGES-03, US-PAGES-04, US-PAGES-05, US-PAGES-06, US-PAGES-07
- Gap now (exists/differs/missing): 0/2/4
- Acceptance: Locked sections, rows with notes and tags, preview mode, apply and cancel, Alt+Up/Down, Undo.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-01` `@US-PAGES-02` `@US-PAGES-03` `@US-PAGES-04` `@US-PAGES-05` `@US-PAGES-06` `@US-PAGES-07`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-18 LOD page and LOD settings page

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M2. Depends on: GUI-03, LOD-01.
- Functions: F-LOD-01, F-LOD-02, F-LOD-03, F-LOD-04, F-LOD-05, F-LOD-06
- Stories: US-PAGES-09, US-PAGES-10, US-PAGES-11, US-PAGES-12, US-PAGES-13, US-PAGES-14, US-PAGES-15
- Gap now (exists/differs/missing): 0/4/2
- Acceptance: All states of `../design/screenshots/26` to `31`; stop confirm; presets and fine-tune page.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-09` `@US-PAGES-10` `@US-PAGES-11` `@US-PAGES-12` `@US-PAGES-13` `@US-PAGES-14` `@US-PAGES-15`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-19 Tale of Two Wastelands flow

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M2. Depends on: GUI-05, TTW-01.
- Functions: F-TTW-01, F-TTW-02, F-TTW-03, F-TTW-04, F-TTW-05
- Stories: US-PAGES-16, US-PAGES-17, US-PAGES-18, US-PAGES-19, US-PAGES-20
- Gap now (exists/differs/missing): 0/5/0
- Acceptance: Four steps with stepper sub-status, space check, low-space block, progress, done.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-16` `@US-PAGES-17` `@US-PAGES-18` `@US-PAGES-19` `@US-PAGES-20`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-20 System page

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M3. Depends on: GUI-03, CORE-07.
- Functions: F-SYSTEM-01, F-SYSTEM-02, F-SYSTEM-03, F-SYSTEM-04, F-SYSTEM-05
- Stories: US-PAGES-21, US-PAGES-22, US-PAGES-23, US-PAGES-24, US-PAGES-25, US-PAGES-26
- Gap now (exists/differs/missing): 0/2/3
- Acceptance: Check on open, parts and changes, Update with progress and failure, engine builds with confirms, masterlist.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-21` `@US-PAGES-22` `@US-PAGES-23` `@US-PAGES-24` `@US-PAGES-25` `@US-PAGES-26`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-21 Doctor page

- Project: raul GUI (crates/raul-gui)
- Size: S. Milestone: M3. Depends on: GUI-03, CORE-02.
- Functions: F-DOCTOR-01, F-DOCTOR-02, F-DOCTOR-03
- Stories: US-PAGES-27, US-PAGES-28
- Gap now (exists/differs/missing): 0/0/3
- Acceptance: Groups, progressive reveal, per-item fixes, Cancel.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-27` `@US-PAGES-28`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### GUI-22 Settings page

- Project: raul GUI (crates/raul-gui)
- Size: M. Milestone: M3. Depends on: GUI-03, CORE-08.
- Functions: F-SETTINGS-01, F-SETTINGS-02, F-SETTINGS-03, F-SETTINGS-04, F-SETTINGS-05, F-SETTINGS-06
- Stories: US-PAGES-29, US-PAGES-30, US-PAGES-31, US-PAGES-32
- Gap now (exists/differs/missing): 0/1/5
- Acceptance: Sections with fold defaults, game rows, storage and cache, per-profile settings with toast, appearance switches.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-29` `@US-PAGES-30` `@US-PAGES-31` `@US-PAGES-32`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-01 App state, profiles, shared library, versions and undo

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: L. Milestone: M0. Depends on: none.
- Functions: F-PROF-01, F-PROF-02, F-PROF-03, F-PROF-04, F-VERS-01, F-VERS-02, F-OVERLAY-02
- Stories: US-TESTS-11, US-TESTS-14, US-TESTS-15, US-TESTS-16, US-TESTS-17, US-TESTS-19, US-MODS-23, US-MODS-24, US-MODS-25
- Gap now (exists/differs/missing): 0/1/6
- Acceptance: Shared and per-profile state split; version per change with 30 s folding; per-profile undo stack of 30 cleared by revert; persisted; unit tests per rule.
- Gherkin: `features/tests.feature`, `features/mods.feature`, scenarios `@US-TESTS-11` `@US-TESTS-14` `@US-TESTS-15` `@US-TESTS-16` `@US-TESTS-17` `@US-TESTS-19` `@US-MODS-23` `@US-MODS-24` `@US-MODS-25`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-02 Checks model for badges, Home rows, Doctor and Play checks

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: L. Milestone: M0. Depends on: CORE-01.
- Functions: F-SHELL-05, F-HOME-04, F-DOCTOR-02, F-MODS-11, F-PLAY-01, F-PLAY-02, F-PLAY-03
- Stories: US-SHELL-14, US-MODS-20, US-MODS-21, US-START-11, US-START-13, US-START-14, US-PAGES-27
- Gap now (exists/differs/missing): 0/3/4
- Acceptance: One pure function from state to check list; badges, rows, doctor items and play checks are views of it; tests on every state.
- Gherkin: `features/shell.feature`, `features/mods.feature`, `features/start.feature`, `features/pages.feature`, scenarios `@US-SHELL-14` `@US-MODS-20` `@US-MODS-21` `@US-START-11` `@US-START-13` `@US-START-14` `@US-PAGES-27`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-03 Job runner: progress, cancel, per-profile keys, status line, resume

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: M. Milestone: M0. Depends on: none.
- Functions: F-SHELL-06, F-OVERLAY-11
- Stories: US-SHELL-10, US-START-09
- Gap now (exists/differs/missing): 0/2/0
- Acceptance: Jobs with stage, percent, minutes left, cancel; several at once; per-profile keys; interrupted jobs reported after a restart.
- Gherkin: `features/shell.feature`, `features/start.feature`, scenarios `@US-SHELL-10` `@US-START-09`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-04 Play orchestration: check, start, run, stop, playtime

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: M. Milestone: M1. Depends on: CORE-02, CORE-03, NV-06.
- Functions: F-PLAY-01, F-PLAY-02, F-PLAY-03, F-PLAY-04, F-SHELL-03
- Stories: US-START-13, US-START-14, US-START-15
- Gap now (exists/differs/missing): 0/2/3
- Acceptance: Checks, Starting, Running, Stop records a session; Last played and playtime totals.
- Gherkin: `features/start.feature`, scenarios `@US-START-13` `@US-START-14` `@US-START-15`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-05 Stability recommendation and known-stable logic

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: M. Milestone: M1. Depends on: CORE-01.
- Functions: F-HOME-02, F-TEST-02, F-VERS-03
- Stories: US-START-10, US-TESTS-04, US-TESTS-05, US-TESTS-12, US-TESTS-13, US-MODS-22
- Gap now (exists/differs/missing): 0/0/3
- Acceptance: Setup signature; banner shows when untested; dismissal per signature; last stable lookup; Forget; revert.
- Gherkin: `features/start.feature`, `features/tests.feature`, `features/mods.feature`, scenarios `@US-START-10` `@US-TESTS-04` `@US-TESTS-05` `@US-TESTS-12` `@US-TESTS-13` `@US-MODS-22`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-06 Setup wizard orchestration

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: M. Milestone: M1. Depends on: CORE-03, NV-05.
- Functions: F-SETUP-01, F-SETUP-03, F-SETUP-04, F-SETUP-06, F-SETUP-07, F-SETUP-08, F-SETUP-09, F-SETUP-10
- Stories: US-START-01, US-START-02, US-START-03, US-START-05, US-START-06, US-START-07, US-START-08, US-START-09, US-START-17
- Gap now (exists/differs/missing): 0/5/3
- Acceptance: Resume at the stored step, defaults on entry, skipped parts remembered, reset.
- Gherkin: `features/start.feature`, scenarios `@US-START-01` `@US-START-02` `@US-START-03` `@US-START-05` `@US-START-06` `@US-START-07` `@US-START-08` `@US-START-09` `@US-START-17`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-07 Update orchestration: check, update parts, engine builds

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: L. Milestone: M3. Depends on: CORE-03, NV-05.
- Functions: F-SYSTEM-01, F-SYSTEM-02, F-SYSTEM-03, F-SYSTEM-04
- Stories: US-PAGES-21, US-PAGES-22, US-PAGES-23, US-PAGES-24, US-PAGES-25
- Gap now (exists/differs/missing): 0/2/2
- Acceptance: Check on open (throttled 5 min, setting), per-part change list, staged update, failure leaves current version, builds list.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-21` `@US-PAGES-22` `@US-PAGES-23` `@US-PAGES-24` `@US-PAGES-25`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-08 Settings, game folders, storage and cache

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: M. Milestone: M1. Depends on: CORE-01.
- Functions: F-SETTINGS-01, F-SETTINGS-02, F-SETTINGS-03, F-SETTINGS-04, F-SETTINGS-05, F-SETUP-03, F-SETUP-04
- Stories: US-PAGES-29, US-PAGES-30, US-PAGES-31, US-PAGES-32, US-START-02, US-START-03
- Gap now (exists/differs/missing): 0/2/5
- Acceptance: Find the game in Steam and GOG libraries, validate folders, free space, clean cache, appearance flags, per-profile game settings that record a version.
- Gherkin: `features/pages.feature`, `features/start.feature`, scenarios `@US-PAGES-29` `@US-PAGES-30` `@US-PAGES-31` `@US-PAGES-32` `@US-START-02` `@US-START-03`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-09 Palette index and action registry

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: S. Milestone: M1. Depends on: none.
- Functions: F-OVERLAY-08
- Stories: US-SHELL-02, US-MODS-30
- Gap now (exists/differs/missing): 0/1/0
- Acceptance: Items of FUNCTIONALITY F-OVERLAY-08 from one registry; mods and short ids searchable.
- Gherkin: `features/shell.feature`, `features/mods.feature`, scenarios `@US-SHELL-02` `@US-MODS-30`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### CORE-10 Fake service and scenario states for app tests

- Project: raul core/orchestration (raul-core, Toolbox app logic)
- Size: M. Milestone: M0. Depends on: none.
- Functions: F-OVERLAY-04
- Stories: US-SHELL-05
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: Test seam to put the app in every state the demo scenario panel reaches (no panel in the product).
- Gherkin: `features/shell.feature`, scenarios `@US-SHELL-05`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NVMOD-01 Archive inspection and install with options

- Project: nvmod-rs
- Size: L. Milestone: M2. Depends on: none.
- Functions: F-INSTALL-01, F-INSTALL-02, F-INSTALL-03, F-INSTALL-04, F-INSTALL-05
- Stories: US-MODS-26, US-MODS-27, US-MODS-28, US-MODS-29
- Gap now (exists/differs/missing): 0/4/1
- Acceptance: API: list archives, detect options pages, recommended defaults, review data (version, plugins, missing masters), staged install with cancel, broken archive error.
- Gherkin: `features/mods.feature`, scenarios `@US-MODS-26` `@US-MODS-27` `@US-MODS-28` `@US-MODS-29`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NVMOD-02 Mod library: toggle, remove, plugins, kinds, masters

- Project: nvmod-rs
- Size: M. Milestone: M2. Depends on: none.
- Functions: F-MODS-02, F-MODS-07, F-MODS-08, F-MODS-10, F-MODS-11
- Stories: US-MODS-14, US-MODS-15, US-MODS-19, US-MODS-20
- Gap now (exists/differs/missing): 0/5/0
- Acceptance: API: per-mod plugins, kind (engine plugin, files only), size, install date, needs-master list, remove from library.
- Gherkin: `features/mods.feature`, scenarios `@US-MODS-14` `@US-MODS-15` `@US-MODS-19` `@US-MODS-20`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### LOOT-01 Sort preview with reasons, notes and missing masters

- Project: loot-rs
- Size: M. Milestone: M2. Depends on: none.
- Functions: F-LOAD-01, F-LOAD-03, F-LOAD-05
- Stories: US-PAGES-01, US-PAGES-05, US-PAGES-06
- Gap now (exists/differs/missing): 0/2/1
- Acceptance: API: proposed order with moves and reasons, masterlist note per plugin, missing masters, out-of-order flags, plugin count.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-01` `@US-PAGES-05` `@US-PAGES-06`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### LOOT-02 Masterlist update with message count

- Project: loot-rs
- Size: S. Milestone: M3. Depends on: LOOT-01.
- Functions: F-SYSTEM-05
- Stories: US-PAGES-26
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: API: update with progress, "N new messages", updated-at.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-26`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### LOD-01 Build with named stages, estimates and fine-tune parameters

- Project: lodwright-rs
- Size: M. Milestone: M2. Depends on: none.
- Functions: F-LOD-01, F-LOD-02, F-LOD-03, F-LOD-04, F-LOD-05, F-LOD-06
- Stories: US-PAGES-09, US-PAGES-10, US-PAGES-11, US-PAGES-12, US-PAGES-13, US-PAGES-14, US-PAGES-15
- Gap now (exists/differs/missing): 0/4/2
- Acceptance: API: five stages (Reading your mods, Terrain, Objects, Trees, Packing) with percent and minutes left; per-preset size and time estimates; fine-tune (distance 1 to 5, texture 256/512/1024, tree density, far desert plants); cancel keeps the old LOD; out-of-date check from mod and quality changes; worlds in the result.
- Gherkin: `features/pages.feature`, scenarios `@US-PAGES-09` `@US-PAGES-10` `@US-PAGES-11` `@US-PAGES-12` `@US-PAGES-13` `@US-PAGES-14` `@US-PAGES-15`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### TTW-01 TTW install: Fallout 3 detection, installer file, space check, staged install, update check

- Project: ttwright-rs
- Size: M. Milestone: M2. Depends on: none.
- Functions: F-TTW-02, F-TTW-03, F-TTW-04, F-TTW-05, F-MODS-05
- Stories: US-PAGES-16, US-PAGES-17, US-PAGES-18, US-PAGES-19, US-PAGES-20, US-MODS-05
- Gap now (exists/differs/missing): 0/5/0
- Acceptance: API: find Fallout 3, check installer file, space needed (25 GB) and time estimate, staged install with cancel, check for a newer TTW, result lands pinned first.
- Gherkin: `features/pages.feature`, `features/mods.feature`, scenarios `@US-PAGES-16` `@US-PAGES-17` `@US-PAGES-18` `@US-PAGES-19` `@US-PAGES-20` `@US-MODS-05`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NV-01 Stability run over 40 places

- Project: nv-rs
- Size: L. Milestone: M3. Depends on: none.
- Functions: F-TEST-01
- Stories: US-TESTS-01, US-TESTS-02, US-TESTS-03
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: Engine-driven run without the main menu: stage text, cells, minutes, hitches, crash location, cancel, result record for the exact mod set.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-01` `@US-TESTS-02` `@US-TESTS-03`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NV-02 Benchmark run

- Project: nv-rs
- Size: M. Milestone: M3. Depends on: none.
- Functions: F-TEST-03
- Stories: US-TESTS-06
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: Route stage names, live fps, average, 1 % low, frame time p99, stored per profile.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-06`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NV-03 Location picture sets

- Project: nv-rs
- Size: M. Milestone: M3. Depends on: none.
- Functions: F-TEST-04
- Stories: US-TESTS-07, US-TESTS-08
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: 44 places in 5 areas (plus Capital Wasteland with TTW), set record with date and quality, cancel, same camera each time.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-07` `@US-TESTS-08`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NV-04 Run log with levels

- Project: nv-rs
- Size: S. Milestone: M3. Depends on: none.
- Functions: F-TEST-06
- Stories: US-TESTS-10
- Gap now (exists/differs/missing): 0/0/1
- Acceptance: Structured lines (time, level, text, detail) from the last runs.
- Gherkin: `features/tests.feature`, scenarios `@US-TESTS-10`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NV-05 Engine install, builds, integrity check and repair

- Project: nv-rs
- Size: M. Milestone: M1. Depends on: none.
- Functions: F-SETUP-05, F-SYSTEM-04, F-DOCTOR-03
- Stories: US-START-04, US-PAGES-25, US-PAGES-28
- Gap now (exists/differs/missing): 0/1/2
- Acceptance: API: install with stages, installed builds, use and remove build, "Damaged" check, repair.
- Gherkin: `features/start.feature`, `features/pages.feature`, scenarios `@US-START-04` `@US-PAGES-25` `@US-PAGES-28`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.

### NV-06 Game launch, stop and session time; per-profile graphics settings

- Project: nv-rs
- Size: M. Milestone: M1. Depends on: none.
- Functions: F-PLAY-04, F-SETTINGS-03
- Stories: US-START-15, US-PAGES-29
- Gap now (exists/differs/missing): 0/1/1
- Acceptance: Start, running state, stop, session length; apply Look, Resolution, Upscaling, V-sync, Frame rate limit per profile.
- Gherkin: `features/start.feature`, `features/pages.feature`, scenarios `@US-START-15` `@US-PAGES-29`
- Check: unit tests for the rules, the scenarios above against the app, snapshot against the design pictures listed in the functions.
