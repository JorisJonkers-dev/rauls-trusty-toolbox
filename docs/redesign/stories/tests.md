# Stories: Mods > Test your setup, Versions, profiles (area: tests)

### US-TESTS-01 Run a stability test and see it pass
As a player, I want to run a stability test on my exact mod set, so that I know this setup is safe to play.
Acceptance:
- Test tab shows "Stability test" with "This exact mod set and order has no passed test yet." when untested.
- "Run stability test" (primary when not tested, secondary when tested) starts the run; the card shows "Loading cells · N of 40", then "Travelling across the map", then "Saving and reloading", with "N of 20 min · crashes 0 · hitches H" and a progress bar.
- A "Cancel" button is shown during the run; the recommendation banner is hidden while it runs.
- On pass: toast "Stability test passed. This setup is now Tested stable"; a passed row appears with "Passed" and "N cells, N min, N hitches"; the Mod setup chip reads "Tested stable"; the text becomes "This exact mod set and order passed a stability test."
- A new "Known stable setups" entry is added.
Screens: test-tab-initial, stab-running, stab-passed
Demo handlers: A.stabGo, A.cancelJob, job()

### US-TESTS-02 Cancel a running stability test
As a player, I want to stop a stability test I no longer need, so that it does not record a result.
Acceptance:
- Clicking "Cancel" while running removes the job and shows toast "Stopped".
- No stability run is added to history and the card returns to its idle text.
Screens: stab-running, stab-cancelled
Demo handlers: A.cancelJob

### US-TESTS-03 A stability test that crashes
As a player, I want a crashed test to point me to the log, so that I can see why it failed.
Acceptance:
- Toast "Stability test crashed in Freeside. See the log".
- Passed/crashed row shows "Crashed" and "in Freeside after N min", with a "Show in log" button.
- Full log opens and the log filter is set to Errors.
- Setup is not marked stable.
Screens: stab-crashed
Demo handlers: A.stabGo (fail callback), A.logFilter

### US-TESTS-04 Known stable setups and Forget
As a player, I want to see and remove known stable setups, so that the list only holds setups I still trust.
Acceptance:
- "Known stable setups · N" row toggles open; each entry reads "N mods" (plus " + TTW" when TTW is on) and "Passed <date>".
- The entry matching the current setup shows the tag "Matches now".
- "Forget" removes the entry and shows toast "Forgotten".
- Empty list shows "None yet. A passed test adds the setup here."
Screens: known-stable-open, known-stable-forgotten
Demo handlers: A.forgetStable, A.sec(stablelist)

### US-TESTS-05 The setup is untested after a mod change
As a player, I want the app to tell me when my mod set changed since the last test, so that I test the new setup before I play.
Acceptance:
- Turning a mod on or off (toast "<Mod> off" or "<Mod> on" with "Undo") makes the banner appear: title "This mod setup hasn't been tested yet.", text "A 20-minute test checks this exact set and order.".
- Banner buttons: "Not now", "Revert to last stable" (only when a passing older version exists), "Run stability test".
- Turning the mod back on hides the banner again when the old setup had passed.
- The Mod setup chip reads "Not tested yet" in the header.
Screens: rec-after-toggle, rec-after-toggle-back
Demo handlers: A.modToggle, A.stabGo, A.vRevertStable, A.recDismiss

### US-TESTS-06 Run and read a benchmark
As a player, I want to benchmark my setup, so that I can see fps over time.
Acceptance:
- "Benchmark" section with "Run benchmark" button; running shows "Running the benchmark route" with place phases "Loading Goodsprings", "Walking to Primm", "Crossing the Strip", "Back to the start" and an fps value; a Cancel button replaces Run.
- Cancel stops the run with toast "Stopped" and adds nothing.
- Result: four tiles "Average" (fps), "1 % low", "Frame time" (ms, with p99), and a delta tile "Against <date>" or "First run"; toast "Benchmark done: N fps average".
- Empty state: "No benchmark for <profile> yet."
- Chart is the history graph, with tags "Average fps", "1 % low, dashed", and "Changes to the setup" / "Includes example runs" when relevant.
Screens: bench-running, bench-result, bench-history-graph
Demo handlers: A.benchGo, A.cancelJob

### US-TESTS-07 Take location pictures and compare
As a player, I want before and after pictures of the same places, so that I can see what my mods changed.
Acceptance:
- "Location pictures" header shows "N sets"; "Take pictures" starts a run with "Taking pictures · k of M places" and a progress bar; "Cancel" replaces it while running.
- Done: toast "New pictures taken"; the newest picture is B, the previous is A.
- Compare segmented control "Side by side" / "Single"; single shows caption "Latest".
- A and B each have a select of sets, showing date, quality and "· example" for example runs.
- Empty: "No pictures yet."; note text "Same spot, same time of day, side by side".
Screens: pics-running, pics-side-by-side, pics-single
Demo handlers: A.picsGo, A.picMode, IN.picA, IN.picB

### US-TESTS-08 Pick a place, search, and see all places
As a player, I want to choose a place and find one by name, so that I can take the picture I want.
Acceptance:
- Place row is a single scrolling strip with place buttons; the selected one is pressed (aria-pressed) and the strip scrolls it into view; mouse wheel scrolls the strip sideways.
- "All N" button expands the list; "Fewer" collapses it.
- Search field "Search places" lists matches across worlds with "N match" / "N matches" or "No place matches.".
- Caption line shows place name and region (for example "Primm  South").
- World control "World" with "New Vegas" / "Capital Wasteland" only when TTW is on; otherwise plain bold "New Vegas".
- Choosing Capital Wasteland selects its first place (Megaton) and clears search.
Screens: pics-all, pics-search, pics-capital-wasteland
Demo handlers: A.picPlace, A.picAll, A.picWorld, IN.picQ

### US-TESTS-09 Zoom and pan a picture
As a player, I want to zoom and pan both pictures together, so that I can inspect a detail.
Acceptance:
- Mouse wheel up zooms by x1.25 and down by x0.8 around the pointer, to a limit of 1x to 8x; drag pans; double-click resets to 1x and centre.
- Chips "1x", "2x", "4x", "8x" set zoom around the centre; the active chip is pressed.
- Pan is clamped so the picture always fills the frame.
Screens: pics-zoom-wheel, pics-pan, pics-reset, pics-zoom-chips
Demo handlers: A.zoom, zoomAt, pointerdown on .pic .vp, dblclick on .pic .vp

### US-TESTS-10 Read and filter the log
As a player, I want to filter and search the log, so that I find warnings and errors fast.
Acceptance:
- Log header tags: "N warnings", "N error(s)", or "Clean" when none.
- Top three non-info lines are shown; "Full log · N lines" expands the full log.
- Filter "All" / "Warnings" / "Errors" (label "Log filter"); search "Search the log"; no match shows "Nothing matches.".
- Clicking "Show in log" on a crash row opens the log filtered to Errors.
Screens: log-open, log-warnings, log-errors, log-search-none, log-search-freeside
Demo handlers: A.logFilter, IN.logQ, A.sec(fulllog)

### US-TESTS-11 See versions and revert
As a player, I want to see every version of my setup and go back to one, so that a bad change can be undone.
Acceptance:
- Versions tab header "Versions" with "N · newest first"; each row: date label, summary (for example "Started with 9 mods on"), tags "Current", "N fps", and one of "Stable", "Crashed", "Not tested".
- Clicking a row expands details: Mods (on of total, TTW), Plugins, Game settings, Stability, Benchmark.
- Older versions show "Revert…"; newer versions show "Use this version…"; the current one shows no button.
- Dialog: "Revert to the version from <date>?" (or "Go to the version from <date>?"), text "Mods, order and game settings become “<summary>” again. Other versions stay in the list.", buttons "Cancel" and "Revert" (or "Use this version").
- Escape or Cancel changes nothing; confirm moves Current, shows toast "Now on the version from <date>", and keeps all versions.
Screens: versions-list, versions-expanded, revert-dialog, versions-after-revert
Demo handlers: A.vexp, A.vRevert, A.doRevert

### US-TESTS-12 Revert to last stable
As a player, I want one button to return to the last version that passed, so that I recover quickly after a crash.
Acceptance:
- "Revert to last stable" shows only when an older version has a pass; it opens the same confirm dialog as US-TESTS-11 for that version.
- Available on the Test tab banner and on the Home banner.
Screens: home-untested-banner, home-banner-revert-dialog
Demo handlers: A.vRevertStable, A.vRevert, A.doRevert

### US-TESTS-13 Home banner routes to the right place
As a player, I want the Home stability banner's actions to land on the matching screen, so that I am not lost.
Acceptance:
- Home shows the same banner as Mods when the setup is untested; "Run stability test" opens Mods on the Test your setup tab and starts the run.
- While running, the banner reads "Stability test running · N %" with a "Show test" button, which opens Mods > Test your setup.
Screens: home, home-banner-run-lands-on-test
Demo handlers: A.stabGo(home), A.stabShow

### US-TESTS-14 Create a profile
As a player, I want to create an empty profile, so that I can try a new mod set without touching the old one.
Acceptance:
- Menu item "New profile…" opens dialog "New profile", field "Name", note "Starts empty. Use Duplicate to start from this one.", buttons "Cancel" and "Create".
- A name already in use shows "A profile with that name exists." and keeps the dialog open.
- Success: toast "Created <name>"; the new profile is active with no mods.
Screens: profile-menu-top, profile-new-dialog, profile-new-duplicate-name, profile-new-created
Demo handlers: A.profNew, A.nameOk

### US-TESTS-15 Duplicate a profile
As a player, I want to copy a profile, so that I can try changes from a known setup.
Acceptance:
- Menu item "Duplicate “<name>”…" opens dialog with OK "Duplicate" and note "Copies mods, groups, load order, settings, LOD and known stable setups. Test history starts fresh.".
- Result: toast "Duplicated as <name>"; mods, groups, order, settings, LOD and Known stable setups are copied; stability, benchmark and pictures start empty; the first version reads "Copied from <source>".
Screens: profile-duplicate-dialog, profile-duplicate-of-beautiful
Demo handlers: A.profDup, A.nameOk

### US-TESTS-16 Rename a profile
As a player, I want to rename a profile, so that its name fits its purpose.
Acceptance:
- Menu item "Rename “<name>”…" opens dialog "Rename profile" with OK "Rename".
- Success: toast "Renamed"; the top bar shows the new name.
- Duplicate name shows the same error as US-TESTS-14.
Screens: profile-rename-dialog
Demo handlers: A.profRename, A.nameOk

### US-TESTS-17 Delete a profile
As a player, I want to delete a profile I no longer use, so that my list stays tidy.
Acceptance:
- Menu item "Delete “<name>”…" opens confirm "Delete “<name>”?", text "Its mod choices, load order, settings and test history go. Installed mod files stay for other profiles.", buttons "Cancel" and "Delete profile" (danger).
- Escape keeps the profile.
- Confirm removes it and shows toast "Deleted <name>. Now on <next>"; the next active profile is the first one in the list.
Screens: profile-delete-confirm, profile-after-delete
Demo handlers: A.profDelete, A.doDeleteProfile

### US-TESTS-18 Switch profile from every page
As a player, I want to switch profile from the top bar, Mods, Load order and LOD, so that I never have to go back to Home.
Acceptance:
- The top-bar pill and the profile chip in each page header open the same "PROFILES" menu; the active profile is marked with a check.
- Switching shows toast "Switched to <name>: mods, load order, settings, LOD and tests" and keeps the current page.
Screens: profile-menu-top, profile-menu-from-load, profile-menu-from-lod
Demo handlers: A.profMenu, A.profSwitch

### US-TESTS-19 What is shared and what is per profile
As a player, I want to know which parts are shared, so that a change in one profile does not surprise me in another.
Acceptance:
- Installed mods, group names and the library are shared across profiles (every profile lists the same 15 mods in the demo).
- Mod choices, load order, game settings, LOD, stability, benchmark, pictures, versions and known stable setups are per profile.
- Removing a mod from all profiles (A.modRemove) says "Mods are shared by every profile."
Screens: profile-menu-top
Demo handlers: syncShared, libraryCopy, A.modRemove
