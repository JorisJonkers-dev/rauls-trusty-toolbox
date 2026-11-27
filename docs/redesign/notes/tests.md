# Notes: Mods > Test your setup, Versions, profiles (area: tests)

Script: scripts/tests.js (re-runnable, exits 1 on console error or failed check, exit 0 on the last run).
Pictures: screens/tests/NN-*.png (48 files incl. run.log, observations.txt). Observations: screens/tests/observations.txt.
Environment of the run: 1366x768, state set by wizard-done flag, TTW installed. The game folder is "Not found" in the top bar because the demo state has no game folder; this does not change the tests.

## Mods page header and tabs
- Header: title "Mods", "N of M on", chip "Tested stable" (shield) or "Not tested yet" (warn), profile pill with menu.
- Tabs (role tablist, aria "Mods views"): "Mod list", "Test your setup", "Versions". Tab click = A.tab, no animation, re-renders.
- Page transition: pgin 6px rise, opacity fade (style.css line 59).

## Recommendation banner (recCard) on Mods, Load order and Home
- Hidden when: no mod is on, or the current set equals a known stable setup, or the set was dismissed ("Not now").
- Text: "This mod setup hasn't been tested yet." / sub "A 20-minute test checks this exact set and order."
- While a stability job runs: sub "Stability test running · NN %", single button "Show test" (A.stabShow -> Mods, Test tab).
- Buttons when idle: "Not now" (ghost, A.recDismiss: hides until the set changes), "Revert to last stable" (secondary, only if a passing older version exists), "Run stability test" (primary, A.stabGo with the page name as arg).
- Verified: changing a mod (toast "NMC's Texture Pack off" + "Undo") shows the banner; toggling it back restores the stable state only if the old set had passed.
- Not run in this pass: "Not now" button (the demo state was already stable when reached). Source: recDismiss pushes the set signature to p.dismissed.

## Stability test (A.stabGo, job 'stab:<profile>', 9 s)
- Idle card: section title "Stability test" + chip. Text: "This exact mod set and order passed a stability test." or "This exact mod set and order has no passed test yet."
- Running card: spinner (refresh icon, .spin 1 s linear) + bold text by progress: "Loading cells · N of 40" (p < 55 %), "Travelling across the map" (p < 90 %), "Saving and reloading"; sub "N of 20 min · crashes 0 · hitches H"; progress bar (pbar).
- Footer: note "Loads 40 places, travels, about 20 min"; button "Cancel" (ghost) while running, else "Run stability test" (primary when untested, secondary when tested).
- Pass: toast "Stability test passed. This setup is now Tested stable"; new row "Passed" + "40 cells, 20 min, N hitches" + "Show in log" only on crash rows.
- Crash: toast "Stability test crashed in Freeside. See the log"; row "Crashed" + "in Freeside after 11 min" + "Show in log" (A.logFilter err: opens full log, filter Errors).
- Cancel: toast "Stopped", no history row (verified: stab count stays 1).
- Starting a run from the Home banner sets U.tab = 'test' and navigates to Mods (verified: page mods, tab test).
- Second start while running: returns without a new job (A.stabGo checks J['stab:<id>']).

## Known stable setups
- Row title: "Known stable setups · N" with fold chevron (A.sec 'stablelist'). Closed by default.
- Entry: "N mods" (+ " + TTW" for TTW sets), sub "Passed <date>", tag "Matches now" when it is the current set, button "Forget" (ghost small, A.forgetStable index).
- Forget: no confirm; toast "Forgotten". Forgetting the current set makes the banner return (source; not separately run).
- Empty: "None yet. A passed test adds the setup here."

## Benchmark (A.benchGo, job 'bench:<id>', 6 s)
- Header "Benchmark" with icon bolt; button "Run benchmark" (primary small, play icon) or "Cancel" while running.
- Running card: spinner + "Running the benchmark route" + sub "<phase> · NN fps", phases by progress: "Loading Goodsprings", "Walking to Primm", "Crossing the Strip", "Back to the start". Progress bar.
- Result tiles (4): "Average" (value green when >= previous), "1 % low", "Frame time" (ms) with "p99 N ms", last tile "Against <date>" with delta percent (green up, amber down) or "First run" with "–".
- Empty: "No benchmark for <name> yet."
- Below: history chart (benchChart) with dashed 1 % low line, tags "Average fps" (accent), "1 % low, dashed", "Changes to the setup" (info, when events exist), "Includes example runs".
- Toast "Benchmark done: N fps average". Cancel: toast "Stopped", no run added (verified).

## Location pictures (A.picsGo, job 'pics:<id>', 5 s)
- Header "Location pictures", count "N sets". Button "Take pictures" (primary, camera icon) or "Cancel".
- Running: spinner + "Taking pictures · k of M places" + progress bar.
- Done: toast "New pictures taken"; new set becomes B, previous newest becomes A.
- Compare: segmented "Side by side" / "Single" (aria "Compare"). Single shows one picture with caption "Latest". Side by side: captions "A" and "B", each with a select of sets: "<date> · <quality>" with " · example" for example sets.
- Quality names come from QUAL map ("High" shown in the demo; "Custom" when unknown).
- Note under pictures: "Same spot, same time of day, side by side".
- Empty: "No pictures yet."
- World seg "New Vegas" / "Capital Wasteland" (aria "World") appears only when TTW is installed AND on in this profile. Otherwise bold text "New Vegas". Verified: TTW on shows the seg and 29 place buttons; Capital Wasteland selects first CW place (Megaton) and shows Megaton, Rivet City, Tenpenny Tower, The Citadel, Washington Monument, Dupont Circle, Georgetown, Arefu...; TTW off hides the seg.
- Search "Search places" (input). Result: a single "Freeside" place chip and a right-side "1 match" (plural: "N matches"). No match: "No place matches.". Searching overrides world filter and shows all matching places (no All/Fewer button).
- Observed issue: searching does not change the picture's place. After searching "freeside" the picture caption still shows "Primm South" (screens/tests/15-pics-search.png). The user must click the chip. Possible demo bug, decide if intended.
- Place row: class "places strip" = single line that scrolls (mouse wheel vertical scrolls it sideways); selected button is primary (filled) with aria-pressed true. Scroll-into-view on change (afterDraw).
- "All N" (ghost small) expands to wrapped list; label becomes "Fewer".
- Caption: bold place name, muted region, muted hint "Wheel zooms, drag pans, double-click resets".
- Compare seg and zoom seg sit on the right of the caption row. Zoom seg labels "1x", "2x", "4x", "8x"; active one is pressed (aria-pressed). Zoom chip applies at picture centre (fraction 0.5).
- Zoom: wheel up = x1.25, wheel down = x0.8, clamped 1x to 8x; pan clamped so the frame stays full. Drag pans both pictures together. Double-click resets to 1x, pan (0,0).
- Verified: wheel up gives 1.25x; drag moves the picture; double-click resets to 1 and 0,0; chip 8x shows pressed; chip 1x returns.
- Toast on pictures done: "New pictures taken" (screenshot 12 shows it over the content).

## Log
- Header "Log" with tags: "N warnings" (warn), "N error(s)" (err), or "Clean" (ok) when none.
- Always shows first three non-info lines (time, text). Empty: "No warnings or errors from the last runs."
- Footer row "Full log · N lines" with fold chevron (A.sec 'fulllog'). Open when a crash was reported.
- Full log: seg "All" / "Warnings" / "Errors" (aria "Log filter"), search "Search the log", list in a log box. No match: "Nothing matches.".
- Verified: search "zzz-none" gives "Nothing matches."; filter Errors shows only error lines.
- Chip "Show in log" on crash row sets filter to Errors and opens the full log (A.logFilter). Not clicked in this pass (set by crash path already).

## Versions tab
- Header "Versions" with "N · newest first". Rows: date label (vLabel, "17 Nov 10:49"), summary (e.g. "Started with 9 mods on").
- Row tags: "Current" (accent) on the active one; cached fps "N fps" when a benchmark exists; status "Stable" (shield), "Crashed" (err), "Not tested" (muted).
- Row button: "Revert…" for older versions, "Use this version…" for newer ones (later than the current). Current has none.
- Click row to expand details: Mods "N of M on · TTW", Plugins "N active", Game settings, Stability (run list or "Same setup passed before" or "No test"), Benchmark list or "No run". Aria expanded on the row.
- Revert confirm: title "Revert to the version from <date>?" or "Go to the version from <date>?" (later); text "Mods, order and game settings become “<summary>” again. Other versions stay in the list."; buttons "Cancel" (ghost) and "Revert" / "Use this version" (button style not checked).
- Escape closes it: no change (verified).
- Confirm: toast "Now on the version from <date>"; current moves; version count unchanged (verified count 2 stays 2).
- Undo state for the profile is cleared on revert (U.undo reset).

## Profiles
- Top bar pill (Profile <name> with chevron) and page-header profile pill open the same menu (A.profMenu, anchor top or pill).
- Menu head "PROFILES"; items: each profile (check mark on active, "not tested" stat tag in the demo), then "New profile…", "Duplicate “<name>”…", "Rename “<name>”…", "Delete “<name>”…".
- Name dialog: field label "Name", max 40 chars, Enter confirms, error text "A profile with that name exists." (case-insensitive).
  - New: title "New profile", help "Starts empty. Use Duplicate to start from this one.", OK "Create", toast "Created <name>". Creates empty profile, active, with one version.
  - Duplicate: title "Duplicate “<name>”", initial value "<name> copy", help "Copies mods, groups, load order, settings, LOD and known stable setups. Test history starts fresh.", OK "Duplicate", toast "Duplicated as <name>". Version 1 summary "Copied from <source>". Verified: 9 mods, 1 stable copied, stab/bench/pics 0.
  - Rename: title "Rename profile", OK "Rename", toast "Renamed".
- Delete: confirm "Delete “<name>”?" with text "Its mod choices, load order, settings and test history go. Installed mod files stay for other profiles.", buttons "Cancel", "Delete profile" (danger). Toast "Deleted <name>. Now on <next>". <next> is the first profile in the list, not the neighbour (verified: deleting Beautiful copy went to Vanilla+). Possible demo bug or intended; decide.
- Switch: toast "Switched to <name>: mods, load order, settings, LOD and tests"; page stays the same (verified from Load order and LOD). Switching to the already active profile re-renders only.
- Verified from the top bar, Load order and LOD. Mods page header profile pill was not clicked in the run (only the top bar was used on Mods).

## Shared versus per profile
- Shared: installed mods (library), group names, group membership and group order follow the profile that changed them (core.js syncShared). The demo lists 15 mods in every profile.
- Per profile: mod on/off choices, load order, game settings, LOD, stability history, known stable setups, benchmarks, pictures, versions, dismissed banner.
- Remove a mod (A.modRemove) states "Mods are shared by every profile."; not run here.

## Known gaps (not run in this pass)
- "Show in log" chip click.
- "Not now" (recDismiss) button click.
- Forget of the current set (banner should return) - source only.
- Revert to last stable on Home after a pass in the same session (banner hides when stable).
- Stability cancel during Home-started run.
- TTW picture world with a Capital Wasteland set (only the place list was checked).

## Animations and timing
- Spinner: .spin 1 s linear infinite (refresh icon) in running cards; reduced motion swaps it to pulse 1.6 s.
- Progress bar: pbar with sheen and slide keyframes (style.css 149-150; timings not measured).
- Toasts: .toast pop .18 s ease-out, 3.5 s visible (5 s with undo). Toasts stack at bottom centre, width 440 px, max 2 (oldest removed).
- Dialogs: pop .18 s scale .97 to 1, fade backdrop.
- Stability job 9 s, benchmark 6 s, pictures 5 s (demo timings, not real).

## Focus and keyboard
- Escape closes dialogs and menus (verified on revert and Home revert dialogs).
- Name dialog focuses the input on open; Enter submits (keydown handler).
- Tabs and seg controls are buttons with aria-pressed or aria-selected.
- Expandable version rows are role=button tabindex=0 with aria-expanded.

## Possible demo bugs
1. Search in places does not move the picture to the found place (caption stays on the previous place). See screens/tests/15-pics-search.png.
2. Deleting a profile sends the user to the first profile in the list, not the one next to it.
3. Toast text duplicates in the DOM during tests (stacking is fine visually; only a test-selector artifact).
