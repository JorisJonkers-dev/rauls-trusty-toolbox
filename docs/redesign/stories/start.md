# Start: setup wizard, Home, Play, top bar, Start setup again

Source: the clickable demo (index.html). Driven by scripts/start.js. Screens in screens/start/.

### US-START-01 First launch and welcome
As a player, I want a short welcome that says what setup will ask, so that I know it is safe to start.
Acceptance:
- First open shows the step bar Welcome, Game, Storage, Engine, TTW, Profile, LOD, Done and the header chip "Setup · step 1 of 8".
- Heading "Let's get New Vegas ready."; lead "A few steps: find the game, install the engine, pick a starting setup. Nothing in your game folder changes."; "About 5 minutes, plus the LOD build if you want it now."
- Only action is "Start" (primary). No Back on this step.
Screens: 01-wizard-welcome.png
Demo handlers: A.wizNext

### US-START-02 Find the game (found, not found, wrong folder, browse)
As a player, I want setup to look for Fallout: New Vegas and let me point at the folder when it cannot, so that the game check passes.
Acceptance:
- Entering the Game step starts "Looking…" (about 1.5 s); Next is disabled until the game is found.
- Found: status "Found", sub-line shows the folder, footer line "Version 1.4.0.525 · all DLC found".
- Not found: status "Not found", row sub "No folder yet", banner "Fallout: New Vegas was not found" with "Pick the folder that holds FalloutNV.exe, usually in your Steam or GOG library." and primary "Browse…".
- Browse… opens "Pick the game folder" with three rows; "Use this folder" stays disabled until a row is picked; Cancel and Escape close it.
- Picking "Downloads" gives status "No game in this folder" and banner "That folder has no FalloutNV.exe"; Next stays disabled.
- Picking "Steam library" gives "Found"; Next enables.
Screens: 02-game-found.png, 17-game-not-found.png, 18-picker-game.png, 19-game-wrong-folder.png, 20-game-found-after-browse.png
Demo handlers: A.findFolder (wfnv), A.pickFolder (wfnv), A.folderSel, A.folderOk, A.dlgClose

### US-START-03 Choose a storage folder
As a player, I want to pick where mods and LOD live and see free space, so that I do not run out mid-setup.
Acceptance:
- Default folder is "D:\RaulsToolbox" with "142 GB free" (status ok). Row text "Storage folder" with path.
- Space bar with note "About 30 GB for mods, LOD and TTW"; footer line "Mods live here, never in the game folder."
- Picking "USB stick" (11 GB) gives status "11 GB free" (warn), banner "Low on space" / "You can go on; TTW and a High LOD may not fit." with "Pick another drive" (secondary). Next stays enabled.
- Picking "Fast drive" clears the banner.
Screens: 03-storage-default.png, 21-storage-low.png
Demo handlers: A.pickFolder (wstore), A.folderSel, A.folderOk, wizEnter

### US-START-04 Install the engine (and retry after failure)
As a player, I want the engine installed from inside setup, with progress and a retry, so that the game can run.
Acceptance:
- Row "Engine 0.24.0", sub "Runs the game · script extenders built in · 182 MB", status "Not installed", button "Install engine".
- Progress card labels "Downloading", then "Checking the download", then "Unpacking" (by thirds), with percent in the row (for example "46 %"). Job about 4.2 s. Next disabled while running; footer "Working…".
- Done: status "Installed", Next enabled.
- Failure (download fails at 40 %): status "Download failed", banner "The download stopped" / "Check your connection, then try again.", button "Try again" replaces "Install engine".
- Retry that succeeds gives "Installed".
Screens: 04-engine-not-installed.png, 05-engine-installing.png, 06-engine-installed.png, 22-engine-failed.png, 23-engine-retry-ok.png
Demo handlers: A.wizEngine, scen "dl"

### US-START-05 Tale of Two Wastelands: now, later, or skip
As a player, I want to add TTW now, or skip it and add it later from Mods, so that setup does not force an optional add-on.
Acceptance:
- Heading "Tale of Two Wastelands" with tag "Optional"; "Plays Fallout 3 inside New Vegas. You can add it later from Mods."
- Rows "Fallout 3" (found from library, or "Not found" with Browse… and Find) and "TTW installer file" "Downloads › TTW_3.4.0.mpi" (Found).
- Undecided buttons: "Skip for now" (ghost) and "Add TTW" (secondary, disabled until Fallout 3 is found). Next disabled until decided.
- "Add TTW" runs about 6 s with progress labels "Checking the Fallout 3 files", "Converting sound", "Building archives", "Patching plugins"; banner "TTW 3.4 added" / "It shows in Mods, pinned at the top."
- "Skip for now" gives banner "Skipped" / "Mods shows TTW as Not added, with an Add button." and "Add TTW instead" (ghost) to undo. Next enabled.
- Fallout 3 missing: status "Not found", Add TTW disabled; Browse… then picking "Steam library" makes it "Found".
Screens: 07-ttw-decision.png, 08-ttw-installing.png, 09-ttw-added.png, 24-ttw-fo3-not-found.png, 25-ttw-skipped.png, 54-ttw-added-low-space.png (from Run D, 11 GB free)
Demo handlers: A.wizTtw (add, skip, undo), A.findFolder (wfo3), A.pickFolder (wfo3), scen "fo3"

### US-START-06 Pick a starting profile
As a player, I want four starting setups to choose from, so that I begin with a setup that suits me.
Acceptance:
- Cards (radio group) with the note "Switch any time from the top bar.":
  - Vanilla+ "Tested stable", "The base game with fixes and a few light touches.", "6 mods on".
  - Beautiful "Tested stable", "Textures, light and weather turned up.", "10 mods on" (default).
  - Hardcore "Both wastelands, tougher rules, fewer caps.", "7 mods on · uses TTW" (plus ", add it later" when TTW is not added), not tested.
  - Empty "Every mod off. Build your own setup.", "0 mods on".
- Choosing a card sets aria-checked on it. Next is always enabled.
Screens: 10-profile-cards.png, 11-profile-hard-selected.png, 26-profile-empty.png
Demo handlers: A.wizProf, wizEnter

### US-START-07 First LOD: quality, build now or later
As a player, I want to pick LOD quality and build it now or later, so that distant terrain is set up without blocking me.
Acceptance:
- Heading "First LOD"; "LOD is what you see in the distance. Build it now, or later from the LOD page."
- Quality segment Performance / Balanced / High with the note under it. Default follows the profile: Vanilla+ Balanced, Beautiful High, Hardcore Performance, Empty High.
- Before a decision: "About N min · X GB" and buttons "Later" (ghost) and "Build now" (secondary, play icon). Next disabled.
- "Build now" runs about 6 s with labels "Reading your mods", "Terrain", "Objects", "Trees", "Packing"; then "Built".
- "Later" gives "Later · shows as Not built" and "Build now instead" (ghost, undo). Next enabled.
Screens: 12-lod-default.png, 13-lod-balanced.png, 14-lod-building.png, 27-lod-default-empty.png, 28-lod-later.png
Demo handlers: A.wizQ, A.wizFar (build, later, undo)

### US-START-08 Finish setup into Home
As a player, I want a summary and one button into Home, so that I can start playing.
Acceptance:
- Heading "You're set." with "Press Play any time. Raul checks your setup first."
- Summary rows: Fallout: New Vegas (Found), Toolbox storage (free GB, warn if under 20), Engine (0.24.0), Tale of Two Wastelands (Added / Not added), Profile (chip), LOD (Built / Not built).
- Buttons "Back" and "Go to Home" (primary).
- Finish lands on Home, toast "Welcome in. {profile} is your profile". Empty profile is saved as "My profile".
Screens: 15-done-step.png, 16-home-after-setup.png, 29-home-skipped-setup.png
Demo handlers: A.wizFinish

### US-START-09 Setup stage kept after reload
As a player, I want a reload to bring me back to the step I was on, so that I do not repeat finished steps.
Acceptance:
- Reload on the Game step shows "Setup · step 2 of 8" again.
- Reload while the engine install runs keeps the step (4) but drops the running job: the engine row shows "Not installed" again and no message says why.
- Finished choices stay (storage, TTW, profile decisions).
Screens: 04-engine-not-installed.png, 05-engine-installing.png (reload check in Run A)
Demo handlers: store.save / store.load, wizEnter

### US-START-10 Home stability banner
As a player, I want to see when my exact mod setup has not been tested and act on it, so that I know whether it is safe.
Acceptance:
- Shown when mods are on and the exact setup has no stability test. Title "This mod setup hasn't been tested yet."; sub "A 20-minute test checks this exact set and order."
- Three buttons when a passed earlier version exists: "Not now" (ghost), "Revert to last stable" (secondary), "Run stability test" (primary). Without an earlier passing version, two buttons.
- "Not now" hides it for this exact setup; it stays hidden after reload until the mod setup changes.
- "Revert to last stable" opens "Revert to the version from {date}?" with the body "Mods, order and game settings become “…” again. Other versions stay in the list." and buttons Cancel and Revert.
- "Run stability test" leaves Home for Mods > Test your setup and starts a 9 s job; banner text on Home becomes "Stability test running · NN %" with "Show test". On pass, toast "Stability test passed. This setup is now Tested stable" and the banner goes.
- Vanilla+ and Beautiful (tested stable) show no banner.
Screens: 33-stability-banner.png, 34-revert-to-last-stable-dialog.png, 35-stability-test-tab.png, 36-stability-running-banner.png, 37-stability-passed-home.png
Demo handlers: A.recDismiss, A.vRevertStable, A.stabGo, A.stabShow

### US-START-11 Home status card and setup rows
As a player, I want a clear status and one row per part of my setup, each taking me to its fix, so that I can act from Home.
Acceptance:
- Header status: "Ready to play", "Ready, with warnings" or "Needs attention". Big line: "All set. Go on, the Mojave’s waiting." / "Play shows them before it starts" / "Play checks first and shows the fix".
- Right column: Profile (chip + "Tested stable" or "Not tested yet"), Last played ("Not yet" or "Today · N min"), Playtime ("0 h" or total).
- SETUP rows (role button, Enter opens): Fallout: New Vegas → Settings; Tale of Two Wastelands → Mods; Mods → Mods; Load order → Load order; LOD → LOD. Shown only as attention rows: Toolbox storage → Settings, Engine → System.
- Mod setup row is not listed on Home (it is in the banner and Play).
Screens: 30-home-hardcore.png, 39-row-game.png, 40-row-ttw.png, 41-row-mods.png, 42-row-load.png, 43-row-far.png
Demo handlers: A.nav, ROWNAV map

### US-START-12 Home after skipping optional steps
As a player who skipped TTW and LOD, I want Home to show what is still open, so that I know what to do later.
Acceptance:
- Empty profile, TTW skipped, LOD later: status "Ready, with warnings"; rows "Tale of Two Wastelands" "Not added" and "LOD" "Not built"; Playtime "0 h", Last played "Not yet".
- No whole-setup skip exists: the wizard covers the window until "Go to Home".
Screens: 29-home-skipped-setup.png
Demo handlers: A.wizFinish

### US-START-13 Play with checks: pass, warnings, cancel, Play anyway
As a player, I want Play to check my setup first and warn me, so that I do not start a broken game by accident.
Acceptance:
- Play runs 8 checks in order (game, storage, engine, TTW, mods, load order, LOD, mod setup), about 380 ms each. Top button "Checking n/8" (disabled); status "Checking before play"; big line "Checking your setup" with "Check n of 8"; progress bar; Cancel.
- Cancel stops the checks and returns to the normal Home.
- With warnings: heading "Before you play: N warnings" with rows, each with its tag and a fix button (for example "Add TTW…", "Sort…", "Build LOD", "Run stability test", "Revert to last stable"), and buttons Cancel and "Play anyway" (primary).
- No warnings: goes straight to "Starting…", then "Starting the game" / "Loading your mods" for 1.5 s, then Playing.
Screens: 44-play-checking.png, 45-play-warned.png (Beautiful, LOD)
Demo handlers: A.play, A.playCancel, A.playAnyway, startGame, checks()

### US-START-14 Play stopped by a failed check
As a player, I want a failed check to stop Play with the fix on the same card, so that I can repair it and try again.
Acceptance:
- Engine damaged: status "Play stopped by a check", card "Play stopped: 1 check failed", row "Engine" with tag "3 engine files changed", buttons "Check again" (ghost) and "Repair engine" (primary). Repair runs about 3 s, then toast "Engine repaired".
- Game missing: row "Fallout: New Vegas" with "Game folder not found", fix "Find the game…" opens "Pick the game folder".
- Top bar New Vegas chip shows "Not found" tag.
Screens: 48-play-failed-engine.png, 49-engine-repaired.png, 50-play-failed-game.png, 51-picker-from-play.png
Demo handlers: A.play, A.repairEngine, A.pickFolder (fnv), scen "engine", scen "game"

### US-START-15 Running, Stop and playtime
As a player, I want to see the game running, stop it, and see my playtime, so that I know how long I played.
Acceptance:
- Playing: top button "Stop · m" (secondary), footer "Playing · {profile} · m min", Home status "Playing", big line "Playing" with profile and time.
- Stop: toast "Played N min. Added to your playtime"; Home shows Last played "Today · N min" and Playtime "N min".
- Observed: N is the elapsed SECONDS shown as minutes (a 3 s run shows "Played 3 min"). See notes.
Screens: 46-play-running.png, 47-play-stopped.png
Demo handlers: A.stopGame, U.run

### US-START-16 Top bar from Home
As a player, I want the top bar to show game, engine and profile and give quick access to search and Play, so that I can jump anywhere from Home.
Acceptance:
- Chips: "New Vegas" (opens Settings), "Engine 0.24.0" (opens System), "Profile {name} ⌄" (opens profile menu).
- Profile menu: heading "Profiles", one row per profile (active has a check), then "New profile…", "Duplicate “{name}”…", "Rename “{name}”…", "Delete “{name}”…" (danger, disabled when it is the only profile or while playing). Escape closes. Switching shows toast "Switched to {name}: mods, load order, settings, LOD and tests" and Home changes (for example Beautiful shows "Tested stable").
- "Search or go to…" with "Ctrl K" opens the search dialog; Escape closes it; Ctrl K opens it from Home.
- "Play" (primary) is replaced by "Stop · m" while running.
Screens: 31-profile-menu-open.png, 32-search-palette.png, 38-home-after-profile-switch.png
Demo handlers: A.profMenu, A.profSwitch, A.palette, A.play, A.stopGame

### US-START-17 Settings: Start setup again
As a player, I want to start setup over on purpose, with a clear warning, so that I can redo it cleanly.
Acceptance:
- Settings > Advanced (collapsed by default) > row "Start setup again" with text "Forgets every choice and profile change, then runs setup from the start." and button "Start setup again…" (danger).
- Dialog "Start setup again?" with "Raul’s Toolbox forgets every choice, profile change and test result, then runs setup from the start. Your game folder is not touched." Cancel keeps everything; "Start setup again" (danger) resets and shows the wizard at step 1 "Let's get New Vegas ready.".
Screens: 52-start-again-confirm.png, 53-after-start-again.png
Demo handlers: A.resetAll, A.doReset, A.secS (adv)
