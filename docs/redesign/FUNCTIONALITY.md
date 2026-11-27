# Raul's Toolbox: functionality inventory

Source of truth: the clickable demo (`../design/demo/index.html`, built from `../design/demo/src/`). This list names every function the
demo shows, grouped by place in the app. Each function has a stable id `F-<AREA>-NN`, the demo handlers that mock it
(`A.<name>` are click or key handlers in `../design/demo/src/actions.js`; `IN.<name>` are input handlers), the user stories
that describe it (`stories/`, Gherkin in `features/`) and screenshots (`../design/screenshots/` are the 89 design pictures;
`screens/<area>/` are the pictures taken while walking each story).

Rules of this file:

- Where the demo and an older description disagree, the demo wins. Appendix A lists what changed against the first draft.
- Wording in quotes is the exact text of the demo.
- `GAP-AUDIT.md` gives the state of the app per id, `TICKETS.md` groups the ids into work.
- Things that look like demo bugs are not copied; they are in `DEMO-BUGS.md` with a decision per item. Until decided, the
  app follows the intended behaviour written here.
- Demo-only scaffolding (scenario panel, example data, drawn pictures) is in Appendix C.

Areas: SHELL (window, top bar, sidebar, status line, action bar), OVERLAY (toasts, undo, dialogs, menus, palette),
ACCESS (focus, keyboard, display options), SETUP (wizard), HOME, PLAY, MODS (list), INSTALL (install dialog), TEST
(stability, benchmark, pictures, log), VERS (versions), PROF (profiles), LOAD, LOD, TTW, SYSTEM, DOCTOR, SETTINGS.

## Shell: window, top bar, sidebar, status line (every page)

### F-SHELL-01 Window

Fixed design size 1280x720 in the demo, scaled down uniformly when the screen is smaller (never above 1). The app must instead use the real window and let the content area grow; the content column keeps the same insets. Title bar (34 px) with app name and Minimise, Maximise, Close. Decision in DEMO-BUGS shell 3.

- Demo handlers: `A.noop`, `A.quitAsk`
- Stories: US-SHELL-12
- Screenshots: `screens/shell/21-viewport-1920.png`

### F-SHELL-02 Top bar

Brand, chip "New Vegas" (opens Settings, red "Not found" tag when the game is missing), engine chip with live build (opens System), profile pill with menu, search button "Search or go to…" with Ctrl K hint (a button that opens the palette, not a field), Play button.

- Demo handlers: `A.nav`, `A.profMenu`, `A.palette`, `A.play`
- Stories: US-START-16, US-SHELL-01
- Screenshots: `../design/screenshots/18-home-after-setup.png`, `screens/shell/01-walk-home.png`

### F-SHELL-03 Play button states

"Play" with tooltip "Checks your setup, then starts the game"; "Checking N/M" while busy; "Starting…"; "Stop · mm:ss" while the game runs.

- Demo handlers: `A.play`, `A.stopGame`
- Stories: US-START-13, US-START-15, US-START-16
- Screenshots: `../design/screenshots/21-play-checking.png`, `../design/screenshots/23-play-running.png`

### F-SHELL-04 Sidebar

Two groups: PLAY (Home, Mods, Load order, LOD) and SYSTEM (System, Doctor, Settings). 216 px wide, items 42 px high. Active item uses the raised fill and an accent icon. LOD stays lit on LOD settings; Mods stays lit on the TTW flow. One Tab stop per item; Enter opens.

- Demo handlers: `A.nav`
- Stories: US-SHELL-01
- Screenshots: `../design/screenshots/18-home-after-setup.png`, `screens/shell/01-walk-home.png`

### F-SHELL-05 Sidebar badges

Counts with colour. Mods: mods with a missing master plus TTW not added while the profile uses it (red with a missing master, else orange). Load order: missing masters + plugins out of order + the untested-change flag (red when a master is missing, else orange). LOD: 1 when not built or out of date (orange). System: icon-only badge "Update available" (blue, no number). Doctor: failures or warnings, only after a Doctor run. Tooltips list each reason. Badges are pure functions of state and clear when the cause is fixed.

- Demo handlers: `A.nav`
- Stories: US-SHELL-14, US-MODS-21
- Screenshots: `../design/screenshots/18-home-after-setup.png`, `../design/screenshots/61-load-missing-master.png`

### F-SHELL-06 Status line

32 px footer: state dot and text. "Ready"; "Setup"/"Working…" in the wizard; "Playing · profile · time"; "<job> · N %" with a pulsing dot, plus "· N more" with several jobs. Folder searches do not show. Shortcuts button with ? hint on the right.

- Demo handlers: `A.shortcuts`
- Stories: US-SHELL-03
- Screenshots: `screens/shell/09-shortcuts-sheet.png`

### F-SHELL-07 Page header

Icon tile, title, one status phrase. Mods, Load order, LOD, LOD settings and Settings also carry the profile pill and the stable chip on the right (a menu, decided as intended in DEMO-BUGS). No page buttons in the header.

- Demo handlers: `A.profMenu`
- Stories: US-PAGES-08, US-TESTS-18, US-SHELL-13
- Screenshots: `../design/screenshots/32-mods-list.png`

### F-SHELL-08 Floating action bar

Pill at the bottom right (20 px from the right, 14 px from the bottom of the content), 48 px high, equal 36 px buttons, primary rightmost, floats over content; content has 84 px bottom padding. Per page: Mods (Undo, New group, Install mod…), Load order (Undo, Sort… or Cancel and Apply), LOD (Fine-tune, Build or Rebuild LOD), LOD settings (Restore defaults, Done), Doctor (Run all checks), TTW and wizard (Back, Next). Home, Settings and System have none.

- Demo handlers: `A.undo`, `A.sortGo`
- Stories: US-SHELL-09
- Screenshots: `../design/screenshots/32-mods-list.png`, `screens/shell/14-abar-mods.png`

### F-SHELL-09 Reduced overlap of last row

The last row of every list scrolls clear of the action bar.

- Demo handlers: none (rendering only)
- Stories: US-SHELL-09
- Screenshots: `screens/shell/14-abar-mods.png`

### F-SHELL-10 Sidebar, top bar and wizard chrome

In the wizard the top bar is brand plus "Setup · step N of 8" only; no sidebar, search or Play. Ctrl K and ? work only after setup is finished.

- Demo handlers: `A.wizNext`
- Stories: US-START-01
- Screenshots: `../design/screenshots/01-wizard-1-welcome.png`


## Feedback and overlays

### F-OVERLAY-01 Toasts

Fixed 440 px, centred, 40 px above the window bottom, 52 px high, radius 14. Text, optional Undo button, 32 px Dismiss X on the right. At most 2 shown (oldest dropped). Plain toast 3.5 s, with Undo 5 s. Hidden while a dialog is open (timers keep running). Entrance "pop" 0.18 s.

- Demo handlers: `A.toastX`, `A.undo`
- Stories: US-SHELL-07
- Screenshots: `screens/shell/13-toast-undo.png`

### F-OVERLAY-02 Undo

Per profile, 30 deep, cleared by a revert. Reachable from the toast and from the Undo button in the Mods and Load order action bars (tooltip names the last change). Message "Undone: <label>". Covers mod toggles, moves, group changes, plugin toggles, sort, mod removal.

- Demo handlers: `A.undo`
- Stories: US-MODS-23, US-PAGES-04, US-SHELL-07
- Screenshots: `screens/shell/13-toast-undo.png`

### F-OVERLAY-03 Dialog frame

520 px wide (760 wide variant), radius 20, padding 20/22, scrim closes the dialog (except while an install runs). Enter runs the last primary or danger button, Escape cancels, Tab wraps. Focus starts on the first of: field, selected item, picker item, option, OK, primary. Focus returns to the opener.

- Demo handlers: `A.dlgClose`, `A.scrim`
- Stories: US-SHELL-05
- Screenshots: `screens/shell/10-quit-confirm.png`, `screens/shell/12-name-dialog.png`

### F-OVERLAY-04 Confirm dialogs

Delete profile, delete group, remove mod from all profiles, stop LOD build, use or remove engine build, revert to a version, clean cache, start setup again, quit. Cancel (ghost) then the primary; danger styling for destructive ones.

- Demo handlers: `A.doDeleteProfile`, `A.doDeleteGroup`, `A.doRemoveMod`, `A.doFarCancel`, `A.doUseBuild`, `A.doRemoveBuild`, `A.doRevert`, `A.doClean`, `A.doReset`
- Stories: US-SHELL-05, US-MODS-19, US-PAGES-11, US-PAGES-25
- Screenshots: `../design/screenshots/48-mods-remove-confirm.png`, `screens/shell/10-quit-confirm.png`, `screens/shell/12-name-dialog.png`

### F-OVERLAY-05 Name dialog

Create, rename, duplicate profile or group. Max 40 characters, OK disabled when empty, duplicates (case-insensitive) blocked with an inline error "A profile with that name exists.", Enter submits.

- Demo handlers: `A.nameOk`, `IN.dlgName`
- Stories: US-TESTS-14, US-TESTS-15, US-TESTS-16, US-MODS-07, US-MODS-08
- Screenshots: `../design/screenshots/38-mods-new-group.png`

### F-OVERLAY-06 Folder picker

One dialog for the game folder, the Fallout 3 folder and the storage folder. Title varies, free space per folder, "Use this folder" disabled until a row is selected, arrow keys move the selection.

- Demo handlers: `A.pickFolder`, `A.folderSel`, `A.folderOk`
- Stories: US-SHELL-15, US-START-02, US-START-03
- Screenshots: `../design/screenshots/05-wizard-2-game-notfound.png`, `../design/screenshots/07-wizard-3-storage.png`

### F-OVERLAY-07 Menus

Anchored under their button with a 6 px gap, flip above when there is no room, clamped 8 px inside the window; min 220 px, items 38 px, radius 14, "pop" 0.12 s. First enabled item focused; Arrow Up/Down wrap, Home/End, Tab closes, Escape closes and returns focus to the anchor. Menu ids: profile, group, benchmark point.

- Demo handlers: `A.profMenu`, `A.grpMenu`, `A.ptMenu`
- Stories: US-SHELL-06, US-MODS-13
- Screenshots: `../design/screenshots/19-profile-menu.png`, `../design/screenshots/37-mods-group-menu.png`, `screens/shell/11-profile-menu.png`

### F-OVERLAY-08 Command palette

Ctrl K toggles. 600 px wide, up to 9 rows of 42 px. Items: seven pages, Play, Install mod…, Sort the load order, Build the LOD, Run a stability test, Run the benchmark, Check for updates, New profile…, Switch to <profile>, every mod. Kind tags Page, Action, Profile, Mod. Arrow keys and Enter; "Nothing found."; a mod opens its details with filters cleared.

- Demo handlers: `A.palette`, `A.palGo`, `A.findMod`, `IN.palQ`
- Stories: US-SHELL-02, US-MODS-30
- Screenshots: `../design/screenshots/85-palette.png`, `screens/shell/08-palette-mod.png`

### F-OVERLAY-09 Shortcuts sheet

Opened by "?" (not while typing, not with a dialog open) or the Shortcuts button. Rows: Ctrl K, ?, Alt ↑/↓, Space, Enter, Esc. Focus on Close.

- Demo handlers: `A.shortcuts`
- Stories: US-SHELL-03
- Screenshots: `../design/screenshots/86-shortcuts.png`, `screens/shell/09-shortcuts-sheet.png`

### F-OVERLAY-10 Quit confirm

Close opens "Quit Raul's Toolbox?" with "Running jobs stop. Your setup is saved." The demo's Quit only closes the dialog; the app must quit and stop jobs.

- Demo handlers: `A.quitAsk`
- Stories: US-SHELL-04
- Screenshots: `screens/shell/10-quit-confirm.png`

### F-OVERLAY-11 Busy states

Spinners spin (1 s), progress bars with sheen or indeterminate slide, "Looking…" text, busy buttons spin their icon. Reduce motion swaps the spin for a slow pulse.

- Demo handlers: none (rendering only)
- Stories: US-SHELL-10
- Screenshots: `../design/screenshots/89-far-building-reduced-motion.png`, `screens/shell/18-settings-reduce-motion-on.png`

### F-OVERLAY-12 Open links

Opening external pages (TTW download, original mod, our repository, file browser). In the demo only a toast.

- Demo handlers: `A.openLink`
- Stories: US-PAGES-18, US-MODS-03
- Screenshots: none


## Focus, keyboard, accessibility

### F-ACCESS-01 Focus look

Text fields: caret only, no outline. Other controls: one 1 px outline in accent at 75 % on keyboard focus only (none on mouse focus). Rows, section headers, log lines and palette items get the selection tint instead.

- Demo handlers: none (rendering only)
- Stories: US-SHELL-01, US-SHELL-05
- Screenshots: `screens/shell/01-walk-home.png`, `screens/shell/10-quit-confirm.png`, `screens/shell/12-name-dialog.png`

### F-ACCESS-02 Whole-row click targets

Clicking a row folds it or flips its switch; nested buttons, grips, inputs, selects and labels keep their own action. Hit area 44x44 around small controls.

- Demo handlers: `A.rowOpen`, `A.gset`
- Stories: US-SHELL-08
- Screenshots: none

### F-ACCESS-03 Keyboard map

Tab order top bar then sidebar then content. Alt+↑/↓ moves the focused mod, group or plugin. Space switches or ticks. Enter opens the focused row. Escape order: menu, palette, dialog, test panel, drag.

- Demo handlers: `A.modMove`, `A.grpMove`
- Stories: US-SHELL-01, US-SHELL-15, US-MODS-16, US-PAGES-07
- Screenshots: `screens/shell/01-walk-home.png`

### F-ACCESS-04 High contrast

Black background, white text, 1 px outlines on blocks and controls, brighter status colours. Setting under Appearance.

- Demo handlers: `A.gset`
- Stories: US-SHELL-11
- Screenshots: `../design/screenshots/84-settings-high-contrast.png`, `screens/shell/19-settings-high-contrast.png`

### F-ACCESS-05 Larger text

Base font 14 px becomes 15.5 px; no horizontal overflow.

- Demo handlers: `A.gset`
- Stories: US-SHELL-11
- Screenshots: `screens/shell/19-settings-high-contrast.png`

### F-ACCESS-06 Reduce motion

In-app switch and the OS setting: animations and transitions go to 0.01 ms, spinners become a 1.6 s pulse, indeterminate bar parked at 35 %.

- Demo handlers: `A.gset`
- Stories: US-SHELL-10, US-PAGES-31
- Screenshots: `../design/screenshots/89-far-building-reduced-motion.png`, `screens/shell/18-settings-reduce-motion-on.png`


## First start: setup wizard (8 steps)

### F-SETUP-01 Wizard frame

Step bar (Welcome, Game, Storage, Engine, TTW, Profile, LOD, Done) with done and current states. Own action bar with Back (hidden on step 1) and Next. The step is stored and resumes after a reload; a reload during the engine install drops the job (DEMO-BUGS start 5).

- Demo handlers: `A.wizNext`, `A.wizBack`
- Stories: US-START-09, US-START-01
- Screenshots: `../design/screenshots/01-wizard-1-welcome.png`

### F-SETUP-02 Welcome

Copy "About 5 minutes, plus the LOD build".

- Demo handlers: `A.wizNext`
- Stories: US-START-01
- Screenshots: `../design/screenshots/01-wizard-1-welcome.png`

### F-SETUP-03 Game step

Find automatically ("Looking…"), found ("Version 1.4.0.525 · all DLC found"), not found, wrong folder ("That folder has no FalloutNV.exe"), Browse to the picker. Next disabled until found.

- Demo handlers: `A.findFolder`, `A.pickFolder`, `A.folderSel`, `A.folderOk`
- Stories: US-START-02
- Screenshots: `../design/screenshots/02-wizard-2-game-finding.png`, `../design/screenshots/03-wizard-2-game-found.png`, `../design/screenshots/05-wizard-2-game-notfound.png`, `../design/screenshots/06-wizard-2-game-wrongfolder.png`

### F-SETUP-04 Storage step

Defaults to a drive on entry; free space and needed-space bar ("About 30 GB for mods, LOD and TTW"); low-space banner with "Pick another drive"; note "Mods live here, never in the game folder."

- Demo handlers: `A.pickFolder`, `A.folderOk`
- Stories: US-START-03
- Screenshots: `../design/screenshots/07-wizard-3-storage.png`, `../design/screenshots/08-wizard-3-storage-low.png`

### F-SETUP-05 Engine step

Install with staged progress (Downloading, Checking the download, Unpacking); failure with "Try again".

- Demo handlers: `A.wizEngine`
- Stories: US-START-04
- Screenshots: `../design/screenshots/09-wizard-4-engine.png`, `../design/screenshots/10-wizard-4-engine-installing.png`

### F-SETUP-06 TTW step

Optional. Find Fallout 3 on entry, TTW installer row with Browse…, Add TTW, Skip for now, "Add TTW instead" to undo a skip, four install stages, added banner.

- Demo handlers: `A.wizTtw`
- Stories: US-START-05
- Screenshots: `../design/screenshots/11-wizard-5-ttw.png`, `../design/screenshots/12-wizard-5-ttw-installing.png`, `../design/screenshots/13-wizard-5-ttw-added.png`

### F-SETUP-07 Profile step

Four cards: Vanilla+, Beautiful, Hardcore, Empty (created as "My profile"). Each shows "N mods on · uses TTW" and a "Tested stable" tag. The choice sets the profile's default LOD quality.

- Demo handlers: `A.wizProf`
- Stories: US-START-06
- Screenshots: `../design/screenshots/14-wizard-6-profile.png`

### F-SETUP-08 LOD step

Quality presets with "About N min · M GB", preview, Build now, Later, "Build now instead", five build stages with percent.

- Demo handlers: `A.wizQ`, `A.wizFar`
- Stories: US-START-07
- Screenshots: `../design/screenshots/15-wizard-7-farview.png`, `../design/screenshots/16-wizard-7-farview-building.png`

### F-SETUP-09 Done step

Summary rows (game, storage free, engine, TTW, profile, LOD) and Finish into Home. Skipped parts show later on Home.

- Demo handlers: `A.wizFinish`
- Stories: US-START-08, US-START-12
- Screenshots: `../design/screenshots/17-wizard-8-done.png`, `../design/screenshots/88-home-skipped-ttw-later-far.png`

### F-SETUP-10 Start setup again

Settings > Advanced, confirm "forgets every choice, profile change and test result… Your game folder is not touched."

- Demo handlers: `A.resetAll`, `A.doReset`
- Stories: US-START-17
- Screenshots: `../design/screenshots/87-reset-to-wizard.png`


## Home

### F-HOME-01 Layout

Single column; no recent sessions, no run settings, no toolbox or storage rows unless they need attention.

- Demo handlers: `A.nav`
- Stories: US-START-11
- Screenshots: `../design/screenshots/18-home-after-setup.png`, `../design/screenshots/20-home-beautiful.png`

### F-HOME-02 Stability banner

Shown when the exact mod set and order is untested. Buttons: Not now (hides until the setup changes again, toast "Hidden until the mod setup changes"), Revert to last stable (only if one exists), Run stability test. While a test runs it shows "Stability test running · N %" and Show test. Same card on Mods and Load order.

- Demo handlers: `A.recDismiss`, `A.vRevertStable`, `A.stabGo`, `A.stabShow`
- Stories: US-START-10, US-TESTS-13, US-MODS-22
- Screenshots: `../design/screenshots/35-mods-untested-card.png`

### F-HOME-03 Status card

State phrase (Ready to play, Ready, with warnings, Needs attention, Checking your setup, Starting the game, Playing, Play stopped), live profile, "Tested stable" or "Not tested yet" tag, Last played ("<day> · <duration>") and one playtime total.

- Demo handlers: `A.nav`
- Stories: US-START-11, US-START-15
- Screenshots: `../design/screenshots/20-home-beautiful.png`, `../design/screenshots/24-home-after-play.png`

### F-HOME-04 Setup list

Whole rows, each goes to the place that fixes it: Fallout: New Vegas OK, Tale of Two Wastelands (Installed / Installed, off here / Not added), Mods, Load order (OK or "N out of order"), LOD (Up to date / Not built / Out of date). Storage, engine (Damaged, Repair engine) and mod-setup rows appear only when they need attention. Missing master shown here.

- Demo handlers: `A.nav`, `A.repairEngine`, `A.cleanCache`
- Stories: US-START-11, US-START-12, US-MODS-20
- Screenshots: `../design/screenshots/18-home-after-setup.png`, `../design/screenshots/62-home-missing-master.png`, `../design/screenshots/88-home-skipped-ttw-later-far.png`


## Play

### F-PLAY-01 Play check

Play rechecks the setup: per-row Checking or Waiting, progress bar, Cancel; top bar "Checking N/M".

- Demo handlers: `A.play`, `A.playCancel`
- Stories: US-START-13
- Screenshots: `../design/screenshots/21-play-checking.png`

### F-PLAY-02 Warnings card

"Before you play: N warnings", per-warning fix buttons, Revert to last stable on the untested warning, Cancel, Play anyway.

- Demo handlers: `A.playAnyway`, `A.vRevertStable`
- Stories: US-START-13
- Screenshots: `../design/screenshots/22-play-result-beautiful.png`

### F-PLAY-03 Failed card

"Play stopped: N checks failed", each failed check with its reason, "Check again" and the first failure's fix. No Play anyway.

- Demo handlers: `A.play`
- Stories: US-START-14
- Screenshots: `../design/screenshots/25-play-failed.png`

### F-PLAY-04 Starting and running

Starting (1.5 s, "Loading your mods"), then running with Stop (no confirm). Stop records a session, toast "Played N. Added to your playtime"; Last played and the playtime total update.

- Demo handlers: `A.stopGame`
- Stories: US-START-15
- Screenshots: `../design/screenshots/23-play-running.png`, `../design/screenshots/24-home-after-play.png`


## Mods page: Mod list tab

### F-MODS-01 Page frame

Tabs Mod list, Test your setup, Versions. Header status "N of M on", profile pill and stable chip. Untested-change card above the tabs.

- Demo handlers: `A.tab`
- Stories: US-MODS-22
- Screenshots: `../design/screenshots/32-mods-list.png`

### F-MODS-02 Search and filters

Search matches mod names and plugin file names, highlights hits. Filters All N / On N / Off N / Attention N. "No mods match" empty text with Clear filters.

- Demo handlers: `IN.q`, `A.filt`, `A.clearFilt`
- Stories: US-MODS-01, US-MODS-02
- Screenshots: `../design/screenshots/36-mods-search.png`

### F-MODS-03 Built-in engine section

Foldable section "Built into the engine" with count and info tooltip. Rows show "Built in" or "In progress", a lock icon, and two icon links "Original mod: X" and "Our version of X". Never a percentage; rows do not fold out.

- Demo handlers: `A.sec`, `A.openLink`
- Stories: US-MODS-03
- Screenshots: `../design/screenshots/33-mods-builtin.png`

### F-MODS-04 Locked plugin lists

Always-on plugin lists live on Load order (Base game, TTW), not on Mods. A mod's own bundled plugins show in its details.

- Demo handlers: `A.rowOpen`
- Stories: US-MODS-04
- Screenshots: none

### F-MODS-05 TTW row

Pinned row with Bundle tag and version. Switch on or off per profile; "Update available" tag; details (always load first, what Off means); "Update TTW…" and "Check for TTW update" (toast "TTW 3.4.1 is ready" or "TTW 3.4 is the newest"). When not added: a row that starts the TTW flow.

- Demo handlers: `A.ttwToggle`, `A.ttwCheck`, `A.ttwStart`
- Stories: US-MODS-05, US-MODS-06
- Screenshots: `../design/screenshots/65-mods-ttw-not-added.png`

### F-MODS-06 Groups

Create (before Ungrouped), rename, delete with confirm (mods move to Ungrouped), fold, member count, tri-state switch (on, off, mixed) to toggle all, move up or down from the menu or Alt+↑/↓, drag the card to reorder. Ungrouped cannot be renamed or deleted. Empty group text "Empty group. Drag mods here, or use Move to group in a mod's details."

- Demo handlers: `A.newGroup`, `A.grpRename`, `A.doDeleteGroup`, `A.grpFold`, `A.grpToggle`, `A.grpMove`, `A.grpMenu`
- Stories: US-MODS-07, US-MODS-08, US-MODS-09, US-MODS-10, US-MODS-11, US-MODS-12, US-MODS-13
- Screenshots: `../design/screenshots/37-mods-group-menu.png`, `../design/screenshots/38-mods-new-group.png`, `../design/screenshots/39-mods-drag.png`, `../design/screenshots/40-mods-after-drag.png`

### F-MODS-07 Mod row

Grip, continuous row number, checkbox (the only on/off control in the row), name, kind tag (Engine plugin, Files only), version, attention tag, chevron. Click the row to fold details open downwards.

- Demo handlers: `A.modToggle`, `A.rowOpen`
- Stories: US-MODS-14, US-MODS-15
- Screenshots: `../design/screenshots/34-mods-row-open.png`

### F-MODS-08 Mod details

What it does, Plugins, chosen install Options, Size, Installed date, Group dropdown, Up and Down buttons, Remove.

- Demo handlers: `A.rowOpen`, `IN.modGroup`, `A.modMove`
- Stories: US-MODS-15, US-MODS-16, US-MODS-18
- Screenshots: `../design/screenshots/34-mods-row-open.png`

### F-MODS-09 Drag and drop

Grips start a pointer drag: ghost "Moving", drop line, auto-scroll near the edge, Escape cancels. Mods drop into other groups; groups reorder.

- Demo handlers: `A.modMove`
- Stories: US-MODS-17, US-MODS-12
- Screenshots: `../design/screenshots/39-mods-drag.png`, `../design/screenshots/40-mods-after-drag.png`

### F-MODS-10 Remove mod

"Remove X from all profiles?" "...archive stays in your downloads, and Undo brings it back." Mods are shared, so removal is global.

- Demo handlers: `A.modRemove`, `A.doRemoveMod`
- Stories: US-MODS-19
- Screenshots: `../design/screenshots/48-mods-remove-confirm.png`

### F-MODS-11 Missing master

Row tag "Needs X", Mods and Load order badges, Home row, Load order rows, Doctor. One shared check.

- Demo handlers: `A.nav`
- Stories: US-MODS-20, US-MODS-21
- Screenshots: `../design/screenshots/61-load-missing-master.png`, `../design/screenshots/62-home-missing-master.png`

### F-MODS-12 Empty and shared library

Empty profile: info banner "No mods in X yet" with Install mod…. A new profile copies the shared library with every mod off in demo data (the text "Starts empty" is wrong, DEMO-BUGS mods 2).

- Demo handlers: `A.profNew`
- Stories: US-MODS-24, US-MODS-25
- Screenshots: `../design/screenshots/57-mods-hardcore-shared-library.png`, `../design/screenshots/64-mods-empty-profile.png`


## Install a mod (dialog)

### F-INSTALL-01 Pick an archive

List with sizes and "has install options"; already installed archives disabled with an "Installed" tag; Browse; cancel the picker.

- Demo handlers: `A.install`, `A.archSel`
- Stories: US-MODS-27
- Screenshots: `../design/screenshots/41-install-pick.png`

### F-INSTALL-02 Install options

Pages of radio or check groups with a live preview picture; defaults chosen; patch options recommended with a "You have this mod" tag.

- Demo handlers: `A.optPick`, `A.instNext`, `A.instBack`
- Stories: US-MODS-26
- Screenshots: `../design/screenshots/42-install-options-1.png`, `../design/screenshots/43-install-options-2.png`

### F-INSTALL-03 Review

Version, Plugins, Options, Group dropdown, missing-master warning, note "Plugins go to the end of the load order. Sort puts them in place."

- Demo handlers: `IN.instGroup`, `A.instGo`
- Stories: US-MODS-26
- Screenshots: `../design/screenshots/44-install-review.png`

### F-INSTALL-04 Install progress

Three stages with progress; Cancel returns to Review; Escape and scrim ignored while busy; done state.

- Demo handlers: `A.instGo`, `A.instCancel`
- Stories: US-MODS-29, US-MODS-27
- Screenshots: `../design/screenshots/45-install-busy.png`, `../design/screenshots/46-install-done.png`

### F-INSTALL-05 Broken archive

Error state with Pick another and Close.

- Demo handlers: `A.archSel`
- Stories: US-MODS-28
- Screenshots: `../design/screenshots/47-install-broken.png`


## Mods page: Test your setup tab

### F-TEST-01 Stability test

Run (about 20 min, 40 places, "Loads 40 places, travels, about 20 min"); live stage text ("Loading cells · N of 40", "Travelling across the map", "Saving and reloading") and "N of 20 min · crashes 0 · hitches N"; Cancel. Passed: result line "Passed: N cells, N min, N hitches", sets "Tested stable", adds the setup to Known stable. Crashed: "Crashed: in <place> after N min", Show in log.

- Demo handlers: `A.stabGo`, `A.cancelJob`, `A.logFilter`
- Stories: US-TESTS-01, US-TESTS-02, US-TESTS-03
- Screenshots: `../design/screenshots/50-test-stability-running.png`, `../design/screenshots/51-test-stability-passed.png`, `../design/screenshots/56-test-stability-crashed.png`

### F-TEST-02 Known stable setups

Foldable list with count; per entry "N mods (+ TTW)", "Passed <date>", "Matches now" tag, Forget. Empty text "None yet. A passed test adds the setup here." Forgetting the current one brings the banner back.

- Demo handlers: `A.forgetStable`
- Stories: US-TESTS-04, US-TESTS-05
- Screenshots: `../design/screenshots/51-test-stability-passed.png`

### F-TEST-03 Benchmark

Run benchmark in the section header; live route stages and fps; Cancel. Result tiles (Average, 1 % low, Frame time with p99, % against the previous run, "First run"); history chart of up to 8 runs (average line, dashed 1 % low, change markers, "Includes example runs"); clickable points open the benchmark point menu (fps, version, "Revert to this version…"). Empty "No benchmark for <profile> yet."

- Demo handlers: `A.benchGo`, `A.ptMenu`
- Stories: US-TESTS-06
- Screenshots: `../design/screenshots/52-test-bench-running.png`, `../design/screenshots/53-test-bench-done.png`

### F-TEST-04 Location pictures

Take a set (Cancel while taking); pick set A and set B from dropdowns, or single mode; 44 places in 5 areas; world tabs New Vegas and, with TTW, Capital Wasteland; one scrolling row of places with "All N" to expand; search across worlds; empty states "No place matches", "No pictures yet."

- Demo handlers: `A.picsGo`, `A.picWorld`, `A.picAll`, `A.picPlace`, `A.picMode`, `IN.picA`, `IN.picB`, `IN.picQ`
- Stories: US-TESTS-07, US-TESTS-08
- Screenshots: `../design/screenshots/54-test-pictures-zoom.png`

### F-TEST-05 Zoom and pan

Wheel zoom around the cursor, drag to pan, double-click resets, chips 1x to 8x; both pictures share zoom and pan.

- Demo handlers: `A.zoom`
- Stories: US-TESTS-09
- Screenshots: `../design/screenshots/54-test-pictures-zoom.png`

### F-TEST-06 Log

Header chips with warning and error counts or "Clean" (static counts, not links; the jump into the filtered log is the "Show in log" button on a crashed stability result); preview of the last 3 non-info lines; Full log fold "Full log · N lines"; filter All, Warnings, Errors; search; expand lines; "Nothing matches."

- Demo handlers: `A.logF`, `A.logOpen`, `A.logFilter`, `IN.logQ`
- Stories: US-TESTS-10
- Screenshots: `../design/screenshots/55-test-log.png`


## Versions

### F-VERS-01 Version history

Every change to mods, order, plugins, TTW or game settings records a version; edits within 30 s fold into one. Newest first with timeline dots (stable, crashed, untested), "Current" tag, fps tag, status chip. Expand for Mods, Plugins, Game settings, Stability, Benchmark.

- Demo handlers: `A.vexp`, `A.vopen`
- Stories: US-TESTS-11
- Screenshots: none

### F-VERS-02 Revert

"Revert…" for earlier, "Use this version…" for later versions, with confirm; newer versions are kept; clears the undo stack.

- Demo handlers: `A.vRevert`, `A.doRevert`
- Stories: US-TESTS-11
- Screenshots: none

### F-VERS-03 Revert to last stable

One function, four entry points: Home banner, Play warnings card, Mods and Load order banners, Versions.

- Demo handlers: `A.vRevertStable`
- Stories: US-TESTS-12, US-TESTS-13
- Screenshots: `../design/screenshots/22-play-result-beautiful.png`


## Profiles

### F-PROF-01 Profile menu

Lists profiles with a "not tested" note and a check on the current; New, Duplicate, Rename, Delete. Switching and Delete are disabled while a job runs; Delete disabled with one profile. Reachable from the top bar and the page headers.

- Demo handlers: `A.profMenu`
- Stories: US-TESTS-18, US-PAGES-08
- Screenshots: `../design/screenshots/19-profile-menu.png`

### F-PROF-02 Create, duplicate, rename, delete

New copies the shared library with the mods off; Duplicate copies mods, groups, order, settings, LOD and known stable but not test history ("Copied from X"); delete confirms and switches to the first profile.

- Demo handlers: `A.profNew`, `A.profDup`, `A.profRename`, `A.profDelete`, `A.doDeleteProfile`
- Stories: US-TESTS-14, US-TESTS-15, US-TESTS-16, US-TESTS-17
- Screenshots: `../design/screenshots/63-profile-duplicate.png`

### F-PROF-03 Switch

Toast says what changes; resets sort preview, open rows, picture selections; builds and tests are keyed per profile.

- Demo handlers: `A.profSwitch`
- Stories: US-TESTS-18
- Screenshots: `../design/screenshots/19-profile-menu.png`

### F-PROF-04 Shared and per profile

Shared: installed mods, group names and members. Per profile: enabled set, order, settings, LOD, test history, versions.

- Demo handlers: `A.profSwitch`
- Stories: US-TESTS-19
- Screenshots: `../design/screenshots/57-mods-hardcore-shared-library.png`


## Load order

### F-LOAD-01 Plugin list

Header "N plugins · M active · sorted/preview/missing masters/out of order". Locked sections Base game (10) and Tale of Two Wastelands (7, with TTW) marked "N · always on", foldable, with lock icons. Your plugins with grip, number, checkbox, owning mod, masterlist note, tags "Out of order", "Up N", "Down N".

- Demo handlers: `A.plugToggle`
- Stories: US-PAGES-01, US-PAGES-04
- Screenshots: `../design/screenshots/58-load-hardcore.png`

### F-LOAD-02 Search and filters

Search matches plugin and owning mod names; All and Out of order filters (All and Moved in preview).

- Demo handlers: `IN.lq`, `A.lfilt`
- Stories: US-PAGES-02, US-PAGES-03
- Screenshots: `../design/screenshots/58-load-hardcore.png`

### F-LOAD-03 Missing masters

Red line "Needs X, which is off or missing" and red badge.

- Demo handlers: `A.nav`
- Stories: US-PAGES-01
- Screenshots: `../design/screenshots/61-load-missing-master.png`

### F-LOAD-04 Move plugins

Drag with grip or Alt+↑/↓; drag disabled in preview.

- Demo handlers: `A.modMove`
- Stories: US-PAGES-07
- Screenshots: none

### F-LOAD-05 Sort

Sort… shows a preview: list in the proposed order, moved rows marked, card "The N moves" or "Already sorted"; Apply (disabled "Nothing to move") or Cancel; note "A restore point is made first, and Undo stays after."

- Demo handlers: `A.sortGo`, `A.sortApply`, `A.sortCancel`
- Stories: US-PAGES-05, US-PAGES-06
- Screenshots: `../design/screenshots/59-load-sort-preview.png`, `../design/screenshots/60-load-sorted.png`

### F-LOAD-06 Page profile switch and untested card

Profile pill in header; untested-change card.

- Demo handlers: `A.profMenu`, `A.recDismiss`
- Stories: US-PAGES-08
- Screenshots: none


## LOD (render distance)

### F-LOD-01 States

Never built, building, done ("Built N ago", size, quality tags), out of date after a quality or mod change. Button flips Build LOD and Rebuild LOD. Tooltip "Render distance"; never "Far view".

- Demo handlers: `A.farBuild`
- Stories: US-PAGES-09, US-PAGES-12
- Screenshots: `../design/screenshots/26-far-never.png`, `../design/screenshots/28-far-done.png`, `../design/screenshots/29-far-stale-quality.png`

### F-LOD-02 Build progress

Five named stages (Reading your mods, Terrain, Objects, Trees, Packing), "step N of 5", minutes left, sidebar badge, status line.

- Demo handlers: `A.farBuild`
- Stories: US-PAGES-10
- Screenshots: `../design/screenshots/27-far-building.png`

### F-LOD-03 Stop a build

Confirm; the old LOD stays.

- Demo handlers: `A.farCancel`, `A.doFarCancel`
- Stories: US-PAGES-11
- Screenshots: none

### F-LOD-04 Quality presets

Presets with description and estimate "X GB · about N min"; picking one clears fine-tune values without a confirm (DEMO-BUGS pages 2).

- Demo handlers: `A.farQ`
- Stories: US-PAGES-13
- Screenshots: `../design/screenshots/29-far-stale-quality.png`

### F-LOD-05 LOD settings page

Separate page: Distance slider 1-5 (Near to Very far), texture size 256/512/1024, tree density, Far desert plants switch; Restore defaults and Done; "Custom" tag on LOD.

- Demo handlers: `A.fine`, `A.fineReset`, `IN.fineDist`, `A.nav`
- Stories: US-PAGES-14
- Screenshots: `../design/screenshots/30-far-finetune.png`

### F-LOD-06 Details fold

Profile, built time, quality, size, worlds (Mojave and DLC, plus Capital Wasteland with TTW), mods included. Each profile keeps its own LOD.

- Demo handlers: `A.sec`
- Stories: US-PAGES-15
- Screenshots: `../design/screenshots/31-far-details.png`


## Tale of Two Wastelands flow

### F-TTW-01 Entry points

Mods row, Home fix "Add TTW…", Settings row "Add…", mod details "Update TTW…" (jumps to the Install step). Mods stays lit in the sidebar.

- Demo handlers: `A.ttwStart`
- Stories: US-PAGES-16, US-MODS-06
- Screenshots: `../design/screenshots/65-mods-ttw-not-added.png`

### F-TTW-02 Step 1 Fallout 3

Auto-find, Find, Browse via the folder picker; found, not found states; stepper with sub-status.

- Demo handlers: `A.findFolder`, `A.pickFolder`, `A.ttwNext`
- Stories: US-PAGES-16, US-PAGES-17
- Screenshots: `../design/screenshots/66-ttw-1-notfound.png`, `../design/screenshots/67-ttw-1-found.png`

### F-TTW-03 Step 2 TTW file

File row "Downloads › TTW_3.4.0.mpi", Browse…, "Get the file" link to the TTW site.

- Demo handlers: `A.openLink`, `A.ttwNext`
- Stories: US-PAGES-18
- Screenshots: `../design/screenshots/68-ttw-2.png`

### F-TTW-04 Step 3 Install

Space check (25 GB, free space, about 22 min); low space blocks Install with a banner offering "Free up space…" and "Change storage"; staged progress with minutes left; Cancel.

- Demo handlers: `A.ttwInstall`, `A.cleanCache`, `A.pickFolder`
- Stories: US-PAGES-19, US-PAGES-20
- Screenshots: `../design/screenshots/69-ttw-3.png`, `../design/screenshots/70-ttw-3-installing.png`

### F-TTW-05 Step 4 Done

"is in your mods; switch it on or off per profile in Mods"; lands pinned at the top; Close.

- Demo handlers: `A.ttwClose`, `A.ttwBack`
- Stories: US-PAGES-20
- Screenshots: `../design/screenshots/71-ttw-4-done.png`


## System

### F-SYSTEM-01 Check for updates

Runs when System opens (setting can turn it off; throttled to once per 5 min), spinner, "Checked N ago", Check again. Palette command too.

- Demo handlers: `A.checkUpdates`
- Stories: US-PAGES-21
- Screenshots: `../design/screenshots/72-system-checking.png`

### F-SYSTEM-02 Parts list

Six parts (Raul's Toolbox, Engine, lodwright, nvmod, loot-rs, ttwright) with role, version, "will be updated" text and a fold with changes (Feature or Fix lines, tags "With the update" and "Installed").

- Demo handlers: `A.nav`
- Stories: US-PAGES-22
- Screenshots: `../design/screenshots/73-system.png`, `../design/screenshots/74-system-changes.png`

### F-SYSTEM-03 Update

One Update button; progress with stages (Downloading, Checking the download, Installing the parts, Restarting); Cancel; failure banner "The server didn't answer. Your current version is untouched." with Try again; success banner. There is no UI to go back (DEMO-BUGS pages 4).

- Demo handlers: `A.update`
- Stories: US-PAGES-23, US-PAGES-24
- Screenshots: `../design/screenshots/75-system-update-failed.png`, `../design/screenshots/76-system-updating.png`, `../design/screenshots/77-system-updated.png`

### F-SYSTEM-04 Engine builds

Foldable list with count, "In use" tag, Use this build… and Remove… with confirms; note "Older builds stay so you can go back if a new one misbehaves."

- Demo handlers: `A.useBuild`, `A.removeBuild`, `A.doUseBuild`, `A.doRemoveBuild`
- Stories: US-PAGES-25
- Screenshots: `../design/screenshots/78-system-engines.png`

### F-SYSTEM-05 Masterlist

Own section: "Updated N ago", "N new messages", Update button with progress, "Used by Sort on Load order".

- Demo handlers: `A.mlUpdate`
- Stories: US-PAGES-26
- Screenshots: none


## Doctor

### F-DOCTOR-01 Run checks

Run all checks / again with Cancel; items reveal as Not checked, Checking…, result; "Ran N ago · profile".

- Demo handlers: `A.docRun`
- Stories: US-PAGES-27
- Screenshots: `../design/screenshots/79-doctor-not-run.png`, `../design/screenshots/80-doctor-running.png`, `../design/screenshots/81-doctor-results.png`

### F-DOCTOR-02 Results

Four foldable groups (Game folder and files and storage; Engine and Toolbox; Masters, Load order, Plugin count N of 255, Script extenders; LOD, Mod setup, TTW), each item with a fix button: Find the game…, Free up space…, Repair engine, Open System, Show load order, Sort…, Add TTW…, Build LOD, Run stability test. Warns above 230 plugins.

- Demo handlers: `A.docFold`, `A.nav`
- Stories: US-PAGES-27
- Screenshots: `../design/screenshots/81-doctor-results.png`

### F-DOCTOR-03 Repair engine

Fix for a damaged engine.

- Demo handlers: `A.repairEngine`
- Stories: US-PAGES-28
- Screenshots: none


## Settings

### F-SETTINGS-01 Game installations

Fallout: New Vegas (Required) and Fallout 3 (For TTW) rows with Browse…, Find and status; TTW row with Add.

- Demo handlers: `A.findFolder`, `A.pickFolder`
- Stories: US-PAGES-32
- Screenshots: `../design/screenshots/82-settings.png`

### F-SETTINGS-02 Toolbox storage

Storage folder, free space, download cache with Clean up… (disabled under 0.1 GB, confirm, "already empty" toast).

- Demo handlers: `A.pickFolder`, `A.cleanCache`, `A.doClean`
- Stories: US-PAGES-30
- Screenshots: `../design/screenshots/83-settings-storage-profile.png`

### F-SETTINGS-03 Game settings for profile

Look, Resolution, Upscaling, V-sync, Frame rate limit; "Changes save as you go"; toast "Saved for <profile>"; each change records a version.

- Demo handlers: `A.gfxSeg`, `A.gfxVsync`, `IN.gfx`
- Stories: US-PAGES-29
- Screenshots: `../design/screenshots/83-settings-storage-profile.png`

### F-SETTINGS-04 Updates setting

Check for updates when System opens.

- Demo handlers: `A.gset`
- Stories: US-PAGES-32
- Screenshots: none

### F-SETTINGS-05 Appearance

High contrast, Larger text, Reduce motion (section folded by default, DEMO-BUGS pages 7).

- Demo handlers: `A.gset`
- Stories: US-PAGES-31
- Screenshots: `../design/screenshots/84-settings-high-contrast.png`

### F-SETTINGS-06 Section folds

Sections fold; Game installations open by default.

- Demo handlers: `A.secS`
- Stories: US-PAGES-32
- Screenshots: none


## Appendix A: findings of the three reviews that were folded in

Three independent reviews of the first draft were checked line by line against `../design/demo/src`. The points they all or
mostly agreed on, and that were confirmed, are now part of the entries above:

- Window is fixed at 1280x720 and scaled down; maximise and minimise do nothing (F-SHELL-01). The draft said content reflows. The app must reflow.
- Top bar chips are buttons; the search is a button that opens the palette (F-SHELL-02).
- Play button has four states (F-SHELL-03). Status line has variants (F-SHELL-06).
- System badge is an icon without a number; Doctor badge appears only after a run; badge colours are red for a missing master else orange (F-SHELL-05).
- Mod rows use a checkbox, not a switch; switches are for the TTW row, the group header (tri-state), V-sync and far desert plants (F-MODS-07, F-MODS-06).
- Mod details show What it does, Plugins, Options, Size, Installed, Group; not masters or notes (F-MODS-08).
- Filters are All, On, Off, Attention with counts (F-MODS-02).
- The built-in engine section folds; its rows do not (F-MODS-03).
- Bundled (always-on) plugin lists belong to Load order, not Mods (F-MODS-04, F-LOAD-01).
- Page headers do hold the profile pill and stable chip on five pages (F-SHELL-07).
- Home engine row appears when the check fails ("Damaged"), not when an update exists; storage and mod-setup rows likewise (F-HOME-04).
- Home status words, Play result cards (warnings card has Play anyway; failed card has Check again and a fix), Play goes through "Starting" (F-HOME-03, F-PLAY-01 to 04).
- Wizard profile cards are Vanilla+, Beautiful, Hardcore, Empty (F-SETUP-07). Wizard step details (F-SETUP-03 to 09).
- `ptMenu` is the benchmark point menu, not a System part menu (F-OVERLAY-07, F-TEST-03). System parts are six named parts; the masterlist is its own section (F-SYSTEM-02, F-SYSTEM-05).
- Toast limits and timing, Undo stack and action-bar Undo button, dialog and menu keyboard rules, palette contents, shortcuts list (F-OVERLAY-01 to 09, F-ACCESS-03).
- Reduce motion keeps a slow pulse on spinners (F-ACCESS-06). High contrast and larger text values (F-ACCESS-04, 05).
- Versions capture TTW and game settings; quick edits fold into one version (F-VERS-01). Versioning is cross-cutting.
- LOD fine-tune is its own page "LOD settings" with its own action bar (F-LOD-05); build shows five stages (F-LOD-02).
- TTW flow has four entry points, a space check and a low-space block (F-TTW-01, F-TTW-04).
- Doctor has four groups and per-item fixes (F-DOCTOR-02). Settings has Game installations and Toolbox storage sections (F-SETTINGS-01, 02).
- Only the LOD stop has a confirm among the job cancels; other cancels act at once (F-OVERLAY-04, F-LOD-03).
- Grouping advice taken for tickets: Revert to last stable is one function with four entry points (F-VERS-03); Undo is cross-cutting (F-OVERLAY-02); the install dialog is its own area (INSTALL); the TTW flow is one flow with four entry points (TTW); folder picker is one dialog with many triggers (F-OVERLAY-06); one shared checks model feeds Home, Play, Doctor and badges (F-HOME-04, F-DOCTOR-02, F-SHELL-05); missing masters is one shared check (F-MODS-11); the library and groups are profile-level state (F-PROF-04).

## Appendix B: review points that were dropped

| Point | Reason dropped |
|---|---|
| System badge has no colour ("neutral icon") | It uses the default badge style, which is blue (info); the walk-through shows a blue badge. |
| `docs/design/README.md` does not exist in the repository | A path remark about where the demo was first kept, not a function. The demo now lives in this folder. |
| "Ctrl+Shift+D opens the scenario panel" as an app shortcut | Demo scaffolding only (Appendix C). |
| Header warning and error chips on the log jump into the filtered log | Wrong: the chips are static counts; the jump is the "Show in log" button on a crashed stability result (F-TEST-06). |
| "Group header keeps the same colour when selected" | Not found in `style.css`; group header focus uses the same tint as other section headers. |
| "Doctor badge on Student sidebar", "she old list" | Typing noise in the review, no content. |
| "Quit confirm only when jobs run" (draft) vs "always" | Both checked: the demo always asks. Kept as demo behaviour, with the text "Running jobs stop." The app should ask when jobs run and may skip the dialog otherwise (decision in DEMO-BUGS shell 1). |
| Per-section "all sections fold" including Appearance open | In the demo only Game installations is open by default; Appearance is folded (DEMO-BUGS pages 7). Kept as a decision, not a function. |
| Review point that mod-detail "Up/Down" are a separate function from Alt+Up/Down | Merged: same move, two inputs (F-MODS-08, F-ACCESS-03). |

## Appendix C: demo only, not part of the product

- Scenario panel (`A.scen`, `A.scenClose`, `A.scenHistory`, `A.scenTtwUpd`, Ctrl+Shift+D) that fakes states: game missing, engine damaged, low space, update failing, crashes. The app needs its own way to reach these states in tests (fake service), not a panel.
- Example data (mods, plugins, places, benchmark history, "example runs") and pictures drawn as vector art (`../design/demo/src/art.js`).
- Hard-coded strings (versions, counts) listed in DEMO-BUGS pages 5.
- Mock job durations: find 1.5 s; engine 4.2 s (fails at 40 % once); TTW 6 s in the wizard and 8 s from Mods; LOD build 6 s (wizard) and 9 s (page); stability 9 s; benchmark 6 s; pictures 5 s; play checks 8 x 380 ms; starting 1.5 s; repair 3 s; update 7 s. Real durations come from the work.
- Persistence to browser storage (`core.js`); the app persists to its own state files.
