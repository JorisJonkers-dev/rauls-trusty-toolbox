# Demo bugs and gaps found while recording user stories

Decide per item: fix in the demo (then the app follows), specify the app's behaviour directly, or mark it intended. Each item has a "Decision needed" line with the proposed outcome and the function id it affects. The owner confirms before work starts; until then the app follows the proposed outcome.

## Decisions (JOB 1)

Items marked "fix demo" are fixed in the demo (see its NOTES.md). "app spec only" means the app follows the proposed outcome and the demo stays. Open questions for the owner:

1. Place search: should searching a place also move the shown picture to the first match, or only filter the row?
2. Deleting a profile: switch to the neighbouring profile, or to the first one in the list?
3. Home "Run stability test": jump to Mods > Test your setup, or run in place on Home?
4. Picking a LOD quality preset: ask before dropping fine-tune values, silently drop them, or keep them as Custom?
5. Settings (and its Appearance section): which sections start open on the first visit?
6. Stopping a running job: always ask, ask only when work is lost, or never ask?
7. System update: add a way back to the old version (engine builds list), or drop the "so you can go back" sentence?

## shell
1. Quit confirm's Quit only closes the dialog; the mock app does not quit (Enter the same).
   Decision needed: Specify app: Quit stops running jobs and exits; ask only when jobs run (F-OVERLAY-10). Demo: fix.
   DECISION: fix demo. Done: Quit closes the mock window ("Raul’s Toolbox has closed", Open again). App spec unchanged.
2. Minimise and Maximise are no-ops but are Tab stops.
   Decision needed: Fix demo: remove from Tab order or make them work; app: real window buttons.
   DECISION: fix demo. Done: Minimise and Maximise left out of the Tab order (tabindex -1).
3. The mock window is fixed at 1280x720 (scales down only), so "pages grow when maximised" cannot be seen in the demo; the app must still grow.
   Decision needed: Specify app: content grows with the window (F-SHELL-01). Demo: leave.
   DECISION: app spec only. No demo change.
4. Folder picker: first ArrowDown from no selection selects the second item.
   Decision needed: Fix demo (first ArrowDown selects the first item); app follows the fix.
   DECISION: fix demo. Done: the first ArrowDown with no selection selects the first item.
5. Ctrl K then Escape leaves focus on the page body (mouse-opened palette returns focus correctly).
   Decision needed: Specify app: focus returns to the opener when the palette closes, also by keyboard (F-OVERLAY-08).
   DECISION: fix demo. Done: when no control had focus, Ctrl K then Escape returns focus to the search button.
6. Settings > Appearance is folded by default.
   Decision needed: Intended or fix demo: owner decides whether Appearance starts open (F-SETTINGS-05).
   DECISION: owner choice (see Q5).
7. Toast Undo undoes the latest change, not the one the toast names; toasts that expire behind a dialog are lost.
   Decision needed: Specify app: Undo undoes the change the toast names; toast timers pause while a dialog is open (F-OVERLAY-01).
   DECISION: app spec only (Undo names its change; timers pause under dialogs). No demo change.

## tests
1. Place search filters the place row but does not change the shown place (caption keeps the old place) — decide if intended.
   Decision needed: Specify app: the caption follows the shown place; search only filters the row (fix demo).
   DECISION: owner choice (see Q1).
2. Deleting a profile switches to the first profile in the list, not a neighbour.
   Decision needed: Specify app: after delete switch to the neighbouring profile (fix demo).
   DECISION: owner choice (see Q2).
3. Not yet exercised: "Show in log" chip, "Not now", Forget of the current set (banner should return), Home "Revert to last stable" after a pass, Capital Wasteland picture set (gaps in notes/tests.md).
   Decision needed: Specify app: cover these in the Gherkin when written; no demo change.
   DECISION: app spec only. No demo change.

## start (wizard, Home, Play)
1. Playtime unit mismatch: a 3 s run shows "3 min" (seconds passed as minutes); short runs floored to 1 min.
   Decision needed: Fix demo: seconds are seconds; short runs show seconds or "under a minute" (F-PLAY-04).
   DECISION: fix demo. Done: a run is stored in seconds; under a minute it shows "N s".
2. TTW install completes with 11 GB free while the low-space texts say it is blocked / may not fit.
   Decision needed: Fix demo: allow the install only when the space check passes (F-TTW-04).
   DECISION: fix demo. Done: TTW needs 25 GB; install and the wizard Add TTW are disabled below that, with a text in the wizard.
3. Home "Run stability test" navigates to Mods > Test your setup; Home then shows only "Show test" while it runs. Decide if intended.
   Decision needed: Intended: Run stability test goes to Mods > Test your setup; Home shows Show test while it runs (F-HOME-02).
   DECISION: owner choice (see Q3).
4. Wizard Done step: the floating action bar overlaps the end of the LOD summary row.
   Decision needed: Fix demo: pad the end of the wizard page so the bar never covers the last row (F-SHELL-08).
   DECISION: fix demo. Done: wizard page bottom padding 84 to 120 px so the bar never covers the last row.
5. Reload during engine install silently drops the job (row back to "Not installed", no message).
   Decision needed: Specify app: an interrupted install resumes or reports "The install was interrupted. Try again." (F-SETUP-05).
   DECISION: app spec only. No demo change.
6. Wizard TTW step: Next label identical in both branches (code only).
   Decision needed: Fix demo (code only): label the Next button per branch.
   DECISION: fix demo. Done: the Next button reads "Next without TTW" after Skip.

## pages (Load order, LOD, TTW flow, System, Doctor, Settings)
1. Load order badge (5) counts the untested-change flag while the page says "4 out of order".
   Decision needed: Specify app: the badge counts missing masters, plugins out of order and the untested flag, and the page text says the same total (F-SHELL-05).
   DECISION: fix demo. Done: the page status also says "untested change" when the badge counts it.
2. Picking a LOD quality preset silently drops fine-tune values (no confirm).
   Decision needed: Specify app: choosing a preset asks before it drops fine-tune values, or keeps them as Custom (owner decides) (F-LOD-04).
   DECISION: owner choice (see Q4).
3. Stop confirms differ: LOD asks; TTW install and System update cancel at once.
   Decision needed: Specify app: every stop of a running job asks first when it loses work, and never when the old result stays (owner decides) (F-OVERLAY-04).
   DECISION: owner choice (see Q6).
4. System success says the old version "is kept, so you can go back", but there is no UI to go back.
   Decision needed: Specify app: either add a way back (engine builds list) or drop the sentence (F-SYSTEM-03).
   DECISION: owner choice (see Q7).
5. Hard-coded version and count strings (update line, "Updating to 0.6.0", Doctor "0.6.0 · newest", masterlist "4 new messages").
   Decision needed: Fix demo: strings come from data; app computes them.
   DECISION: fix demo. Done: versions come from the PARTS data; masterlist count from one constant, with plural.
6. Storage folder status shows the same text in its warning and ok branches.
   Decision needed: Fix demo: the two branches say different things (F-SETTINGS-02).
   DECISION: fix demo. Done: the warning branch now says "N GB free, low on space".
7. Settings sections fold by default, hiding resolution and frame-cap rows on first visit.
   Decision needed: Intended or fix demo: owner decides fold defaults (F-SETTINGS-06).
   DECISION: owner choice (see Q5, same question as shell 6).
8. An undone toast stacks under the earlier toast, showing the same plugin name twice.
   Decision needed: Fix demo: an undone toast replaces the older toast for the same change (F-OVERLAY-01).
   DECISION: fix demo. Done: Undo removes the earlier toast of the same change before showing its own.

## mods
1. Install review for The Living Desert's JSawyer patch lists "B42 Optics - WRP.esp" (hard-coded plugin name).
   Decision needed: Fix demo: derive the name from the archive data (F-INSTALL-03).
   DECISION: fix demo. Done: the JSawyer patch lists "The Living Desert - JSUE.esp".
2. New profile says "Starts empty" but copies the 15-mod library.
   Decision needed: Specify app: a new profile starts with every mod off; text "Starts with every mod off. Your installed mods are shared." (F-PROF-02).
   DECISION: fix demo (text). Done: dialog says "Starts with every mod off. Your installed mods are shared." The behaviour already was that.
3. Plugin notes (e.g. "Loads after LonesomeRoad.esm...") are stored but never shown.
   Decision needed: Specify app: show the masterlist note in the load order and the row details (F-LOAD-01, F-MODS-08).
   DECISION: app spec only. No demo change.
4. Badge tooltip grammar: "1 mod need attention".
   Decision needed: Fix demo: "1 mod needs attention" (plural rule).
   DECISION: fix demo. Done: "1 mod needs attention".
5. Ctrl K matches mod names only (short ids find nothing).
   Decision needed: Specify app: the palette also finds mods by short id and plugin file name (F-OVERLAY-08).
   DECISION: app spec only. No demo change.
6. Toasts are hidden while any dialog is open.
   Decision needed: Specify app: toasts stay hidden under dialogs but are queued and shown after (F-OVERLAY-01).
   DECISION: app spec only. No demo change.

## Owner decisions (2026-11-27)

- Home "Run stability test": opens Mods > Test your setup with the test running (as the demo does).
- LOD quality preset over changed fine-tune values: ask first ("Replace your fine-tune settings with the <preset> preset?").
- Stopping a running job: ask only when stopping throws work away (LOD build, TTW install); otherwise stop at once.
- After an update, System offers "Go back to <old version>" for the kept version.
- Place search shows the first match when the shown place is filtered out; deleting a profile switches to the neighbouring profile; Settings sections start open on first visit (confirmed 2026-11-28).

## Owner decisions (2026-11-28)

Decisions taken after checking the tickets against the code. The demo is changed first where a decision changes what it shows.

- Engine (F-SETUP-05, F-SYSTEM-04, F-DOCTOR-03): ADR-0016 stays. The setup engine step shows the check only. The engine builds on System are kept Toolbox versions, with "Go back to <old version>". "Repair engine" checks the game folder again and fixes it. NV-05 is narrowed to the check and repair; builds move to CORE-07.
- TTW space (F-TTW-04): the space needed is the installer manifest's figure once a file is picked, 25 GB before that. No hard-coded figure.
- LOD stages (F-LOD-02): the Toolbox maps lodwright's phases onto the five stage names; lodwright adds minutes left.
- LOD fine-tune (F-LOD-05): all four parameters (distance 1 to 5, texture 256/512/1024, tree density, far desert plants) are built in lodwright.
- Stability run (F-TEST-01): the full sweeps, 80 Mojave places plus 42 Capital Wasteland places with TTW. The demo's counts and estimates change to match.
- Location pictures (F-TEST-04): 44 places in 5 areas, curated from the sweep places and kept in nv-rs.
- Old GUI rules: the redesign replaces `docs/UI-RULES.md` and `docs/design/mockup.html`. The test `no_screen_says_profile` goes in GUI-16.
- Mod lists: their screens are designed in the demo first (GUI-23, #720); #690 waits for that.
- Playtime (F-PLAY-04, F-HOME-03): Home's total is the Toolbox's session record per profile. New: a "Continue from" save picker on Home (GUI-24 / CORE-11, #719) that shows each save's in-game time and profile, offers "Switch to <profile>", and starts the game into the save.
- Undo and versions (CORE-01): built on nvmod-journal, one store.
- Checks (CORE-02): a typed in-process call, no parsing of text output.
- Game settings (F-SETTINGS-03): Resolution, V-sync and Frame rate ship first; Upscaling with #569, Look with #586.
- Start order: GUI-01, GUI-02, CORE-01, CORE-02 and LOOT-01's structured missing masters start together.
