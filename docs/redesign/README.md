# Raul's Toolbox redesign set

Everything needed to make the egui app work and look like the clickable demo. The demo is the reference.

## What is here

| Part | What it is |
|---|---|
| `FUNCTIONALITY.md` | The final inventory: 107 functions with stable ids `F-<AREA>-NN`, demo handlers, story ids and screenshots. |
| `SPEC.md` | Colours, type, spacing, sizes, alignment, animations, focus and keyboard rules, product rules, and where the demo implements each. |
| `USE-CASES.md` | The user stories grouped into end-to-end use cases. |
| `GAP-AUDIT.md` | The current app against each function: exists, differs or missing, with file references. |
| `TICKETS.md` | Potential tickets per project, with sizes and dependencies. |
| `WORKFLOWS.md` | Milestones, parallel lanes and how each ticket is checked. |
| `DEMO-BUGS.md` | Things that look like demo bugs, each with a decision to take. |
| `../design/demo/` | The clickable demo (not copied here). `index.html` is built from `src/` by `build.sh`. `test/` holds the demo's own checks. |
| `../design/screenshots/` | The 89 design pictures, numbered (not copied here). |
| `stories/` | 113 user stories in five areas (start, mods, tests, pages, shell). |
| `features/` | One Gherkin feature per area, one scenario per story, tagged with the story id. |
| `screens/` | 232 pictures taken while walking each story. |
| `notes/` | Observed details per area: texts, rules, timings, measurements. |
| `scripts/` | One Playwright walk per area and `run-all.sh`. |

## Run the demo

Open `../design/demo/index.html` in a browser (it is one self-contained file). After editing `../design/demo/src/`, rebuild with
`../design/demo/build.sh`. Reset the state with `localStorage.clear()` and a reload. The scenario panel (Ctrl+Shift+D) fakes
states such as a missing game or a failing update.

## Run the checks

```
scripts/run-all.sh
```

This runs the five area scripts (`start`, `mods`, `tests`, `pages`, `shell`) and the demo's own `../design/demo/test/run.js`.
Playwright must be resolvable from node: install it (`npm i playwright`) or point `NODE_PATH` at a `node_modules`
that has it, so that `require('playwright')` works. Each script writes its pictures to `screens/<area>/`, prints `ok`
and `FAIL` lines, and exits non-zero on a console error or a failed
check. Run a single area with `node scripts/<area>.js`.

## From Gherkin to app tests

Each story in `stories/<area>.md` has one scenario in `features/<area>.feature` with the tag `@US-<AREA>-NN`. The
steps use the exact UI text of the demo. For the app:

1. A ticket in `TICKETS.md` lists the story ids it covers.
2. Each scenario becomes a headless test in `crates/raul-gui/tests/` using `egui_kittest` (the harness already used
   by `shell.rs`, `mods.rs` and the others). Steps map to queries by label (`Given` sets the service state through the
   fake service, `When` clicks or presses keys, `Then` queries the labels and states).
3. The picture of the same step in `screens/<area>/` or `../design/screenshots/` is the target for a snapshot test
   (`snapshots/`), compared with a tolerance; layout values are asserted from `SPEC.md` section 5.
4. A ticket is done when its scenarios pass, its snapshots are accepted and the unit tests of its rules pass.
