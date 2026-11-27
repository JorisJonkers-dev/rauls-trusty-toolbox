# Use cases

End-to-end use cases built from the user stories in `stories/`. Each lists its story ids; the Gherkin scenario of a story
carries the same id as a tag (for example `@US-START-02` in `features/start.feature`). The same story can sit in more than one use case.
Function ids (F-...) for each story are in `FUNCTIONALITY.md`.

| Use case | Stories |
|---|---|
| UC-1 First start to first play | 15 |
| UC-2 Add and organise mods | 16 |
| UC-3 Organise with groups | 7 |
| UC-4 Change the load order | 9 |
| UC-5 Keep a stable setup | 15 |
| UC-6 Manage profiles | 9 |
| UC-7 Build and maintain LOD | 7 |
| UC-8 Add or update Tale of Two Wastelands | 7 |
| UC-9 Update | 6 |
| UC-10 Troubleshoot | 10 |
| UC-11 Settings, display and keyboard | 19 |

## UC-1 First start to first play

A new player installs Raul, sets up the game, engine, optional TTW, a starting profile and LOD, lands on Home and plays.

Feature files: `features/start.feature`

- US-START-01: First launch and welcome
- US-START-02: Find the game (found, not found, wrong folder, browse)
- US-START-03: Choose a storage folder
- US-START-04: Install the engine (and retry after failure)
- US-START-05: Tale of Two Wastelands: now, later, or skip
- US-START-06: Pick a starting profile
- US-START-07: First LOD: quality, build now or later
- US-START-08: Finish setup into Home
- US-START-09: Setup stage kept after reload
- US-START-11: Home status card and setup rows
- US-START-12: Home after skipping optional steps
- US-START-13: Play with checks: pass, warnings, cancel, Play anyway
- US-START-15: Running, Stop and playtime
- US-START-16: Top bar from Home
- US-START-17: Settings: Start setup again

## UC-2 Add and organise mods

Install an archive with options, find it, switch it, group it, remove it.

Feature files: `features/mods.feature`

- US-MODS-01: Search the mod list
- US-MODS-02: Filter the mod list
- US-MODS-14: Switch a mod on or off
- US-MODS-15: Open a mod's details
- US-MODS-16: Move a mod up or down
- US-MODS-17: Drag a mod into another group
- US-MODS-18: Choose a mod's group in its details
- US-MODS-19: Remove a mod from all profiles
- US-MODS-26: Install a mod with install options
- US-MODS-27: Install a plain archive, browse, or cancel the picker
- US-MODS-28: See a broken archive
- US-MODS-29: Cancel an install in progress
- US-MODS-30: Find a mod with Ctrl K and jump to it
- US-MODS-03: See the built-in engine group
- US-MODS-04: See a mod's bundled plugins
- US-MODS-07: Create a group

## UC-3 Organise with groups

Create, rename, delete, fold, move and drag groups; switch a whole group.

Feature files: `features/mods.feature`

- US-MODS-07: Create a group
- US-MODS-08: Rename a group
- US-MODS-09: Delete a group
- US-MODS-10: Fold a group and switch all its mods
- US-MODS-11: Move a group up and down
- US-MODS-12: Reorder groups by dragging
- US-MODS-13: Close the group options menu

## UC-4 Change the load order

Read the order, toggle plugins, preview and apply a sort, move plugins by keyboard, undo.

Feature files: `features/pages.feature`

- US-PAGES-01: See the load order status and what needs fixing
- US-PAGES-02: Search plugins
- US-PAGES-03: Show only out-of-order plugins
- US-PAGES-04: Turn a plugin on or off, and undo it
- US-PAGES-05: Preview a sort and cancel it
- US-PAGES-06: Apply a sort
- US-PAGES-07: Move a plugin from the keyboard
- US-PAGES-08: Switch profile from the profile pill
- US-MODS-23: Undo a change

## UC-5 Keep a stable setup

Test the setup, benchmark it, take location pictures, read the log, forget or keep stable sets, revert to an earlier or the last stable version.

Feature files: `features/tests.feature`

- US-TESTS-01: Run a stability test and see it pass
- US-TESTS-02: Cancel a running stability test
- US-TESTS-03: A stability test that crashes
- US-TESTS-04: Known stable setups and Forget
- US-TESTS-05: The setup is untested after a mod change
- US-TESTS-06: Run and read a benchmark
- US-TESTS-07: Take location pictures and compare
- US-TESTS-08: Pick a place, search, and see all places
- US-TESTS-09: Zoom and pan a picture
- US-TESTS-10: Read and filter the log
- US-TESTS-11: See versions and revert
- US-TESTS-12: Revert to last stable
- US-TESTS-13: Home banner routes to the right place
- US-MODS-22: Respond to an untested change
- US-START-10: Home stability banner

## UC-6 Manage profiles

Create, duplicate, rename, delete and switch profiles; know what is shared.

Feature files: `features/tests.feature`

- US-TESTS-14: Create a profile
- US-TESTS-15: Duplicate a profile
- US-TESTS-16: Rename a profile
- US-TESTS-17: Delete a profile
- US-TESTS-18: Switch profile from every page
- US-TESTS-19: What is shared and what is per profile
- US-MODS-24: Empty profile
- US-MODS-25: New profile contents
- US-PAGES-08: Switch profile from the profile pill

## UC-7 Build and maintain LOD

See the LOD state, pick a quality, fine-tune, build with progress, stop, rebuild when out of date.

Feature files: `features/pages.feature`

- US-PAGES-09: See whether LOD is built and current
- US-PAGES-10: Build LOD with progress
- US-PAGES-11: Stop a LOD build, with confirm
- US-PAGES-12: LOD goes out of date after a quality or mod change
- US-PAGES-13: Choose a quality preset
- US-PAGES-14: Fine-tune each control and reset
- US-PAGES-15: Open LOD details

## UC-8 Add or update Tale of Two Wastelands

Four-step flow from the Mods row, Home, Settings or the mod details; switch it on and off per profile.

Feature files: `features/pages.feature`, `features/mods.feature`

- US-PAGES-16: Start TTW from Mods and check Fallout 3
- US-PAGES-17: Fallout 3 not found: Find, then Browse
- US-PAGES-18: TTW installer file and the TTW download page
- US-PAGES-19: Install is blocked on low disk space
- US-PAGES-20: Install TTW with progress, cancel and finish
- US-MODS-05: Switch Tale of Two Wastelands on or off and check for an update
- US-MODS-06: Add Tale of Two Wastelands when it is not added yet

## UC-9 Update

Open System, see what changes per part, update with progress, handle a failure, use or remove an engine build, update the masterlist.

Feature files: `features/pages.feature`

- US-PAGES-21: Check for updates when System opens
- US-PAGES-22: See what an update changes, per part
- US-PAGES-23: Update Raul's Toolbox with progress
- US-PAGES-24: Update fails, then retry works
- US-PAGES-25: Use or remove an engine build, with confirm
- US-PAGES-26: Update the masterlist

## UC-10 Troubleshoot

Play stopped by a failed check, missing master, crash in a stability test, damaged engine, Doctor checks, the log.

Feature files: `features/start.feature`, `features/pages.feature`, `features/tests.feature`

- US-START-14: Play stopped by a failed check
- US-MODS-20: See a missing master
- US-MODS-21: See the Mods badge appear and clear
- US-TESTS-03: A stability test that crashes
- US-TESTS-10: Read and filter the log
- US-PAGES-27: Run the checks and read the results
- US-PAGES-28: Repair a damaged engine from Doctor
- US-PAGES-25: Use or remove an engine build, with confirm
- US-PAGES-24: Update fails, then retry works
- US-START-04: Install the engine (and retry after failure)

## UC-11 Settings, display and keyboard

Game settings per profile, storage and cache, appearance options, shortcuts, palette, dialogs, toasts, window.

Feature files: `features/pages.feature`, `features/shell.feature`

- US-PAGES-29: Game settings for this profile
- US-PAGES-30: Storage folder and download cache
- US-PAGES-31: Appearance: reduce motion, high contrast, larger text
- US-PAGES-32: Updates setting, TTW row and start setup again
- US-SHELL-01: Walk every page by keyboard
- US-SHELL-02: Search or go to with Ctrl K
- US-SHELL-03: Shortcuts sheet
- US-SHELL-04: Quit confirm from the window close button
- US-SHELL-05: Dialogs: placement, focus, Escape, Enter, scrim
- US-SHELL-06: Menus: arrows, Escape, Tab
- US-SHELL-07: Toasts: width, position, Undo, stacking, timing
- US-SHELL-08: Whole-row click targets
- US-SHELL-09: Floating action bar
- US-SHELL-10: Reduce motion (OS setting and in-app switch)
- US-SHELL-11: High contrast and Larger text
- US-SHELL-12: Window size and maximised
- US-SHELL-13: Alignment of card headers and rows
- US-SHELL-14: Sidebar badges
- US-SHELL-15: Arrow keys in the folder picker and Alt+Up/Down moves
