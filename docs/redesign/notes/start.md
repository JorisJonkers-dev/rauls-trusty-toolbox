# Start area: observed details (the clickable demo)

Method: scripts/start.js (Playwright, 1366x768, window screenshots). Runs A-D, 54 screens, 0 failed checks, 0 console errors. Lines marked (code) were read in src/ and not exercised.

## Setup wizard
Stepper: Welcome, Game, Storage, Engine, TTW, Profile, LOD, Done. Done steps show a check icon; current has aria-current="step". Header chip "Setup · step N of 8". Footer "Setup", or "Working…" with busy dot while any job runs.
Action bar (floating, bottom right): Back (not on step 1), step buttons. Next is disabled while a job runs or the step is undecided.

- Welcome: Start (primary). Lead text as in stories.
- Game: Row "Fallout: New Vegas". Sub-line = folder path, "Looking in your Steam and GOG libraries…" while searching, "No folder yet" when none. End: status (Looking… / Found / Not found / No game in this folder), Browse…, Find. Find is disabled while searching (busy). Find is a 1.5 s job; with scen "game" it sets Not found.
- Game picker "Pick the game folder": rows Steam library (D:\SteamLibrary\steamapps\common\Fallout New Vegas, ok), GOG Games (C:\GOG Games\Fallout New Vegas, ok), Downloads (C:\Users\Player\Downloads, not ok). Use this folder disabled until a row is picked. Cancel and Escape close it; Enter on a primary confirms (code, actions.js keydown).
- Storage: Row "Storage folder": path or "Not set", end "142 GB free" (ok) / "11 GB free" (warn, under 20 GB). Space row with bar (30 GB reference) and "About 30 GB for mods, LOD and TTW". Picker "Pick a storage folder": Fast drive (E:\RaulsToolbox, 412 GB), Game drive (D:\RaulsToolbox, 142 GB, default), USB stick (F:\RaulsToolbox, 11 GB).
- Engine: Row "Engine 0.24.0" with sub "Runs the game · script extenders built in · 182 MB". Statuses Not installed / NN % (spinning) / Installed / Download failed. Progress card labels by thirds: Downloading, Checking the download, Unpacking. Install job 4.2 s; with scen "dl" it fails at 40 %. Failure banner has no action of its own; the action bar offers Try again (secondary, download icon). Retry clears the error.
- TTW: heading tag "Optional". Rows: Fallout 3 (sub path / "Looking…" / "No folder yet"; status Found / Not found; Browse…, Find) and "TTW installer file" sub "Downloads › TTW_3.4.0.mpi" (Found). Action bar when undecided: Skip for now (ghost), Add TTW (secondary, plus icon, disabled until Fallout 3 found). After skip: Add TTW instead (ghost). Add TTW job 6 s; progress labels Checking the Fallout 3 files / Converting sound / Building archives / Patching plugins with percent. Success banner "TTW 3.4 added" (version from S.g.ttwVer). Skip banner "Skipped". Next enabled when added or skipped.
- Profile: radio cards. Each card: name, optional "Tested stable" tag (shield icon), blurb, mods count line. Hardcore count line "7 mods on · uses TTW" plus ", add it later" if TTW not added (code: `W.ttw !== 'added'`). Empty card uses a plus icon and "0 mods on". Selecting resets quality to the profile default unless the user already changed quality (qSet).
- LOD: hero (farHero) with state; status text variants: "About N min · X GB" (before), building labels Reading your mods / Terrain / Objects / Trees / Packing with percent, "Built", "Later · shows as Not built". Quality segment Performance (8 min, 0.9 GB, "Smaller textures, nearer distance, sparse trees.") / Balanced (15 min, 1.9 GB, "Matches the game’s own LOD.") / High (26 min, 3.4 GB, "Sharper textures, farther, denser trees."). Build job 6 s. Action bar: Later (ghost), Build now (secondary, play icon); after Later: Build now instead (ghost). Quality defaults: Vanilla+ Balanced, Beautiful High, Hardcore Performance, Empty High (the initial wizard quality).
- Done: hero "You're set." / "Press Play any time. Raul checks your setup first." Rows: Fallout: New Vegas (Found), Toolbox storage ("142 GB free" ok / warn under 20 GB), Engine (0.24.0), Tale of Two Wastelands (Added / Not added), Profile (tag with user icon), LOD (Built / Not built). Action bar: Back, Go to Home (primary).
- Finish: profile "Empty" becomes "My profile". Toast "Welcome in. {profile} is your profile". Lands on Home, scroll to top.

Reload: wizard stage (S.wiz.step, storage key rauls-toolbox-state-1) is saved on every change. Reload resumes on the same step and re-runs the find for the game/Fallout 3 when no result yet. In-flight jobs (engine, TTW, LOD builds) are not saved: after reload the engine row reads Not installed with no message.

## Home
Top bar (all pages): brand "Raul's Toolbox"; chip "New Vegas" (folder icon, opens Settings; adds a "Not found" tag when the game is missing); chip "Engine 0.24.0" (bolt icon, opens System); chip "Profile {name} ⌄" (user icon, opens profile menu aligned left); spacer; "Search or go to…" with "Ctrl K" (opens palette, placeholder "Pages, actions, profiles, mods"; Escape closes; Ctrl K toggles); Play (primary, play icon, tooltip "Checks your setup, then starts the game"). Play is replaced while running by "Stop · m" (secondary), while starting by "Starting…" (disabled), while checking by "Checking n/8" (disabled).

Profile menu: heading "Profiles"; one item per profile (check icon on the active one; note "not tested" when not tested); separator; "New profile…", "Duplicate “{name}”…", "Rename “{name}”…", "Delete “{name}”…" (danger). Delete disabled with one profile or while playing. Other profiles disabled while playing. Switch toast "Switched to {name}: mods, load order, settings, LOD and tests". Escape closes the menu.

Stability banner (rec): flask icon. Title "This mod setup hasn't been tested yet." Sub "A 20-minute test checks this exact set and order." Buttons:
- Not now (ghost): hides for this exact setup signature; persists across reload; returns when the mod setup changes.
- Revert to last stable (secondary, undo icon): only when an earlier version passed. Opens confirm "Revert to the version from {date}?" or "Go to the version from {date}?" when the stable one is newer. Body "Mods, order and game settings become “{summary}” again. Other versions stay in the list."
- Run stability test (primary, play icon): calls A.stabGo, which switches to Mods on the Test tab and starts a 9 s job. Pass toast "Stability test passed. This setup is now Tested stable". Fail path (scen "stab"): toast "Stability test crashed in Freeside. See the log".
- While running: sub "Stability test running · NN %" and a single "Show test" (secondary) that goes to Mods, Test tab.
Shown only when mods are on and the exact setup is not tested stable. Vanilla+ and Beautiful (tested stable) show no banner. Hardcore shows it.

Status card: header "Home" with status text. Status values: Ready to play, Ready, with warnings, Needs attention, Checking before play, Starting, Playing, Play stopped by a check. Big line and sub:
- ok: "Ready to play" / "All set. Go on, the Mojave’s waiting."
- warn: "Ready, with warnings" / "Play shows them before it starts"
- fail: "Needs attention" / "Play checks first and shows the fix"
- running: "Playing" / "{profile} · m min"
- checking: "Checking your setup" / "Check n of 8" (with progress bar and Cancel)
- starting: "Starting the game" / "Loading your mods"
Right column: Profile (user-icon chip + "Tested stable" shield tag or "Not tested yet" warn tag), Last played ("Not yet" or "{Today|date} · m min"), Playtime ("0 h" or total).

SETUP rows (role button, tabindex 0, Enter activates; chevron icon): Fallout: New Vegas → Settings (OK or "Not found"); Tale of Two Wastelands → Mods ("Installed" / "Installed, off here" / "Not added"); Mods → Mods ("OK"); Load order → Load order ("OK", "N out of order" warn, "Missing master" fail with "{plugin} needs {master}"); LOD → LOD ("OK" / "Not built" / "Out of date" / "Building"). Attention-only rows: Toolbox storage → Settings (warn, "N GB free"), Engine → System (fail "Damaged"). Mod setup check is not listed on Home.

Play flow: checks in order: game, storage, engine, TTW, mods, load order, LOD, mod setup. ~380 ms each. Failure stops the flow at the first failed check; card role=alert:
- Heading "Play stopped: N check(s) failed"; rows with tag = why (e.g. "3 engine files changed", "Game folder not found"); buttons Check again (ghost, refresh icon) and the first failure's fix (primary): Repair engine, Find the game….
- Status "Play stopped by a check"; big "Play stopped" / "Fix the failed check".
Warnings card role=alert: heading "Before you play: N warning(s)". Rows with tag and a small secondary fix: Add TTW…, Sort…, Build LOD / Rebuild LOD, Run stability test, Revert to last stable (when available). Buttons Cancel (ghost) and Play anyway (primary, play icon). Status "Ready, with warnings"; big "You can play, or fix these first".
Warnings-free run goes straight to Starting…, then Playing after 1.5 s; toast "New Vegas is running".
Cancel during check: back to normal Home, no toast.

Playing: top "Stop · m" (secondary, stop icon). Footer "Playing · {profile} · m min" with busy dot. Stop toast "Played N min. Added to your playtime". Home: Last played "Today · N min", Playtime "N min".

## Settings: Start setup again
Settings page; section "ADVANCED" is collapsed by default (header click, data-act secS adv). Row "Start setup again": text "Forgets every choice and profile change, then runs setup from the start." Button "Start setup again…" (secondary, danger, refresh icon). Confirm dialog "Start setup again?": "Raul’s Toolbox forgets every choice, profile change and test result, then runs setup from the start. Your game folder is not touched." Cancel (ghost); "Start setup again" (danger primary). After confirm: wizard at step 1 (Welcome); toast none; no Ctrl K until finished.

## Behaviour checked
- Escape closes the picker, the profile menu, the search dialog and the Revert confirm (Cancel path).
- Keyboard: Ctrl+Shift+D opens the test situations panel from the wizard too.
- Reload on Home keeps the banner dismissal and profile.

## Bugs and oddities
1. Playtime unit mismatch (confirmed). A 3 s run shows "Stop · 3 min", footer "Playing · Hardcore · 3 min", toast "Played 3 min. Added to your playtime", Last played "Today · 3 min" and Playtime "3 min". Cause: actions.js stopGame and topHTML/footHTML pass Math.floor(ms/1000) (seconds) to hm(), which formats minutes. stopGame also uses Math.max(1, …), so a short run is recorded as 1 min.
2. TTW add ignores low storage (confirmed). With 11 GB free (USB stick) Add TTW completes: banner "TTW 3.4 added". The test situation "Low disk space" says "TTW install is blocked", and the Storage banner says "TTW and a High LOD may not fit", so the three texts disagree. (A.wizTtw has no disk check.)
3. Run stability test from the Home banner takes the user away from Home to Mods > Test your setup (A.stabGo sets tab test and navigates). Home's banner then only offers "Show test" while it runs. Decide whether it should stay on Home.
4. Done step: the floating action bar (Back, Go to Home) overlaps the right end of the last summary row (LOD "Built"). Seen in screens/start/15-done-step.png.
5. Reload during an install silently drops the job (no message, row back to Not installed). Consider resuming or saying so.
6. (code) The wizard's Next on the TTW step has the same label in both branches (dead ternary, no visible effect).
7. (code) Enter on a focused row or a dialog primary triggers the action (actions.js keydown). Not exercised.

## Animations and timings (style.css and jobs)
- Page content fades up on change: pgin .22s ease-out (translateY 6px to 0). Mod detail rows .18s.
- Dialogs: scrim fade .15s; dialog pop .18s (scale .97 to 1). Menu pop .12s. Toasts pop .18s; auto-hide 3.5 s, or 5 s with Undo; at most 2 visible; hidden while a dialog is open.
- Progress bars: width transition .35s ease, with a sheen gradient. Indeterminate slide 1.2s ease-in-out (not used on these steps).
- Running icons spin 1 s linear; busy dot pulses 1.4 s; switch knob .15s; chevron rotate .18s.
- Reduced motion (Settings) sets all animation and transition durations to .01 ms.
- Job durations: find 1.5 s; engine 4.2 s (fails at 40 %); TTW 6 s (wizard) / 8 s (Mods); LOD build 6 s (wizard) / 9 s (LOD page); stability test 9 s; Play checks 8 x 380 ms; Starting 1.5 s; repair 3 s; update 7 s.
