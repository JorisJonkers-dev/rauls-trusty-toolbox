# User stories: pages (Load order, LOD, TTW flow, System, Doctor, Settings)

Source of truth: `../design/demo/src/pages.js`, `actions.js`, `core.js` (checks, farState, jobs), `overlays.js` (dialogs, scenario panel). Script: `scripts/pages.js`. Screens: `redesign/screens/pages/NN-*.png`. Notes: `redesign/notes/pages.md`.

Shared setup used by every story: the demo's first-run wizard is run once (TTW skipped, profile Hardcore), so the state is Hardcore with TTW on but not added. Test situations are toggled with Ctrl+Shift+D.

---

## Load order

### US-PAGES-01 See the load order status and what needs fixing
As a modder, I want to see on Load order how many plugins are out of order and whether any master is missing, so that I know what to fix before I play.
Acceptance:
- Header status reads `17 plugins · 17 active · 4 out of order` on Hardcore (unsorted). After Sort it reads `17 plugins · 17 active · sorted`.
- Chip beside the profile pill: `Tested stable` (shield, ok) when the setup has a stability test, otherwise `Not tested yet` (warn). Shown only when at least one mod is on.
- Sidebar badge on Load order: count of missing masters + out of order + 1 for an untested change. Hardcore shows `5` (4 out of order + untested). Badge title is the joined text, e.g. `4 out of order, untested change`.
- Sections: `BASE GAME` (`10 · always on`), `TALE OF TWO WASTELANDS` (`7 · always on`, only when TTW is on), `YOUR PLUGINS` (count). Base and TTW sections fold open/closed on click.
- Plugin row: number, checkbox, file name, mod name underneath, tag `Out of order` (warn) on rows that the masterlist would move, `Needs <Mod>, which is off or missing` in error colour when a master is absent.
Screens: 01-load-01-hardcore-unsorted
Demo handlers: A.sec, A.sortGo, A.plugToggle, A.nav

### US-PAGES-02 Search plugins
As a modder, I want to type part of a plugin or mod name, so that I can find one plugin in a long list.
Acceptance:
- Search box placeholder and label: `Search plugins`.
- Filtering matches file name or mod name, case-insensitive; matched text is highlighted.
- No match shows `No plugins match.`
- Clearing the box restores the full list.
Screens: 02-load-02-search-wmx, 03-load-03-search-none
Demo handlers: IN.lq (input), A.lfilt

### US-PAGES-03 Show only out-of-order plugins
As a modder, I want a filter for plugins that are out of order, so that I can see only what Sort will move.
Acceptance:
- Segmented filter: `All 7` and `Out of order 4` (before a sort preview).
- Choosing `Out of order 4` shows only those rows.
- During a sort preview the filter reads `All N` and `Moved N` instead.
Screens: 04-load-04-filter-out-of-order
Demo handlers: A.lfilt

### US-PAGES-04 Turn a plugin on or off, and undo it
As a modder, I want to switch one plugin on or off by clicking its row or checkbox, and undo that change, so that I can test a plugin without losing my setup.
Acceptance:
- Clicking the row or the checkbox toggles the plugin. Checkbox label `<file> active` / `<file> inactive`.
- Toast with the text `<file> inactive` (or `active`) and an `Undo` action.
- The action bar shows `Undo` (title `Undo: <label>`). Undo restores the previous state and toast reads `Undone: <label>`.
- Inactive rows keep the name in muted colour.
Screens: 05-load-05-plugin-toggled
Demo handlers: A.plugToggle, A.undo

### US-PAGES-05 Preview a sort and cancel it
As a modder, I want to see what Sort would move before it changes anything, and cancel without changes, so that I stay in control of my order.
Acceptance:
- Clicking `Sort…` replaces the action bar with `Cancel` (ghost) and `Apply 4 moves` (primary, check icon).
- The preview card titled `The 4 moves`, one row per plugin: file name and tag `Up N` or `Down N`.
- Note under the card: `A restore point is made first, and Undo stays after.`
- Filter switches to `All 17` / `Moved 4`. Moved rows show their tag.
- `Cancel` removes the preview. No change was made, no toast.
- When nothing moves the preview title is `Already sorted`, the row says `Every plugin is where the masterlist wants it.` and the apply button reads `Nothing to move` (disabled).
Screens: 06-load-06-sort-preview
Demo handlers: A.sortGo, A.sortCancel

### US-PAGES-06 Apply a sort
As a modder, I want to apply the previewed sort in one step, so that the order matches the masterlist.
Acceptance:
- `Apply 4 moves` writes the order and toast `Sorted: 4 moves`.
- Status becomes `17 plugins · 17 active · sorted`; Out of order tags disappear.
- Undo becomes available with the label `Sorted: 4 moves`.
- The sidebar badge drops to the untested-change count only (1).
Screens: 07-load-07-sorted
Demo handlers: A.sortApply, A.undo

### US-PAGES-07 Move a plugin from the keyboard
As a keyboard user, I want Alt+Up and Alt+Down to move a focused plugin, so that I can reorder without dragging.
Acceptance:
- Focusing a plugin name and pressing Alt+ArrowDown moves it one place down and the row numbers update.
- Grip title reads `Drag to move. Keyboard: Alt+Up or Alt+Down.`
Screens: 10-load-10-alt-down
Demo handlers: keydown handler (Alt + ArrowUp/ArrowDown on an element with data-move="plugin")

### US-PAGES-08 Switch profile from the profile pill
As a modder, I want to switch profile from the header, so that the mods, load order, settings, LOD and tests all change together.
Acceptance:
- Header pill `Profile <name>` with chevron opens menu headed `Profiles`; the active profile has a check icon; untested profiles show `not tested` as a note.
- Menu footer items: `New profile…`, `Duplicate “<name>”…`, `Rename “<name>”…`, `Delete “<name>”…` (danger; disabled when only one profile or a job runs).
- Choosing a profile closes the menu and toasts `Switched to Beautiful: mods, load order, settings, LOD and tests`.
- Escape closes the menu without switching.
Screens: 08-load-08-profile-menu, 09-load-09-beautiful
Demo handlers: A.profMenu, A.profSwitch

---

## LOD

### US-PAGES-09 See whether LOD is built and current
As a player, I want LOD to say whether it has been built and whether it matches my mods, so that I know if distant land looks right.
Acceptance:
- `Up to date`: `Matches your 7 enabled mods.` Right side shows `Built <ago>`, `<size> GB`, quality name. Sidebar LOD badge is hidden.
- `Not built`: `Far away looks flat until you build it.` Primary button `Build LOD`. Sidebar badge `LOD not built` (warn, 1).
- Other states are in US-PAGES-12.
Screens: 11-lod-01-hardcore-up-to-date, 18-lod-07-beautiful-not-built
Demo handlers: A.nav, A.farBuild

### US-PAGES-10 Build LOD with progress
As a player, I want to build LOD and watch progress, so that I know how long it takes.
Acceptance:
- Right side: `Building · NN %`, `<stage> · step k of 5 · about N min left`, progress bar. Stages in order: `Reading your mods`, `Terrain`, `Objects`, `Trees`, `Packing`.
- Action bar shows `Cancel…` in place of Build.
- On completion toast `LOD for Beautiful is ready`; status returns to `Up to date`.
Screens: 19-lod-08-building, 22-lod-11-beautiful-built
Demo handlers: A.farBuild (job far:<id>, 9 s)

### US-PAGES-11 Stop a LOD build, with confirm
As a player, I want to stop a running LOD build after confirming, so that I do not stop it by accident.
Acceptance:
- `Cancel…` opens a dialog titled `Stop the LOD build?` with text `The LOD you had stays in place.` Buttons `Cancel` and `Stop build` (danger).
- Cancel keeps the build running.
- Stop build ends the job and toasts `Build stopped`; the previous LOD (if any) is kept.
Screens: 20-lod-09-stop-confirm, 21-lod-10-build-stopped
Demo handlers: A.farCancel, A.doFarCancel

### US-PAGES-12 LOD goes out of date after a quality or mod change
As a player, I want LOD to say when it no longer matches my setup, so that I rebuild before playing.
Acceptance:
- Quality change after a build: `Out of date` / `Quality changed since the last build.`; button `Rebuild LOD`; sidebar LOD badge `LOD out of date`.
- Enabling or disabling a mod after a build: `Out of date` / `Mods changed since the last build.`
- A rebuild clears the state and the badge.
Screens: 16-lod-05-quality-high-out-of-date
Demo handlers: A.farQ, A.modToggle (Mods page), A.farBuild

### US-PAGES-13 Choose a quality preset
As a player, I want to pick Performance, Balanced or High, so that I trade detail against time and disk space.
Acceptance:
- Segmented control `Performance`, `Balanced`, `High`.
- Line under it shows size and time: `<n> GB · about <m> min`. Help text: Performance `Smaller textures, nearer distance, sparse trees.`; Balanced `Matches the game’s own LOD.`; High `Sharper textures, farther, denser trees.`
- Choosing a preset clears any fine-tuning (see US-PAGES-14).
Screens: 11-lod-01-hardcore-up-to-date (the Quality row is on the LOD page)
Demo handlers: A.farQ

### US-PAGES-14 Fine-tune each control and reset
As a player, I want to set texture size, distance, tree density and far desert plants, and reset them, so that I can tune LOD beyond the presets.
Acceptance:
- `Fine-tune` opens `LOD settings` with subtitle `<quality> · used by the next build`.
- Texture size (256 / 512 / 1024), Distance (range 1 to 5, label `Near`, `Short`, `Normal`, `Far` or `Very far`), Tree density (Sparse / Normal / Dense), Far desert plants (switch, `Costs some frames.`).
- Any change marks the LOD page quality `Custom`.
- `Restore defaults` is disabled until something is fine-tuned, then clears it.
- `Done` returns to LOD.
Screens: 12-lod-02-fine-tune, 13-lod-03-fine-tune-changed, 14-lod-03b-plants-on, 15-lod-04-fine-reset
Demo handlers: A.fine, IN.fineDist, A.fineReset

### US-PAGES-15 Open LOD details
As a player, I want the details folded away, so that the page stays short until I need them.
Acceptance:
- `Details` header folds closed by default; click opens a list: Profile, Built, Quality, Size, Worlds (`Mojave Wasteland and the DLC worlds`, plus `, plus the Capital Wasteland` with TTW), Mods in it.
- Built shows `Never` when not built.
Screens: 17-lod-06-details-open
Demo handlers: A.sec (details)

---

## TTW flow (Add Tale of Two Wastelands)

### US-PAGES-16 Start TTW from Mods and check Fallout 3
As a modder, I want to start adding TTW from Mods and see whether Fallout 3 was found, so that I know the install can proceed.
Acceptance:
- Mods card before TTW is added: `Tale of Two Wastelands`, `Fallout 3 inside New Vegas`, tag `Not added`, button `Add…` (primary).
- Page title `Add Tale of Two Wastelands`, `Step 1 of 4`. Steps: `Fallout 3` (sub `Found`), `TTW file`, `Install`, `Done`.
- Step 1 row: `Fallout 3`, path, status `Found` (ok) with buttons `Browse…` and `Find`. `Next: TTW file` enabled.
Screens: 23-ttw-01-mods-add-card, 24-ttw-02-step1-fallout3-found
Demo handlers: A.ttwStart, A.nav

### US-PAGES-17 Fallout 3 not found: Find, then Browse
As a modder whose Fallout 3 copy is not where Raul looks, I want to search and then browse, so that I can point Raul at it.
Acceptance:
- Not found: status `Not found` (fail), subtitle `No Fallout 3 folder found`, help `TTW needs Fallout 3 GOTY installed. Steam and GOG copies both work.`, `Next: TTW file` disabled, Step subtitle `Not found`.
- `Find` shows `Looking…` for about 1.5 s, then toasts `Fallout 3 not found. Try Browse`.
- `Browse…` opens `Pick the Fallout 3 folder` with three folders. Choosing a non-Fallout folder and `Use this folder` toasts `No Fallout 3 in that folder`; choosing a real one toasts `Fallout 3 folder saved` and the step turns Found.
Screens: 25-ttw-02b-step1-fallout3-not-found, 26-ttw-02c-find-not-found, 27-ttw-02d-browse-picker, 28-ttw-02e-fo3-browsed-found
Demo handlers: A.findFolder('fo3'), A.pickFolder('fo3'), A.folderSel, A.folderOk

### US-PAGES-18 TTW installer file and the TTW download page
As a modder, I want to confirm the installer file and reach the TTW download page, so that I have the right archive.
Acceptance:
- Step 2 row: `TTW installer file`, path `Downloads › TTW_3.4.0.mpi`, `Found`, `Browse…`. Button `Get the file`.
- `Get the file` toasts `The TTW download page opened in your browser`.
- `Browse…` toasts `Your file browser opened`.
- `Back` returns to step 1. `Next: Install` goes on.
Screens: 29-ttw-03-step2-ttw-file, 30-ttw-03b-site-link-toast
Demo handlers: A.openLink (ttwsite, browse:ttw), A.ttwNext, A.ttwBack

### US-PAGES-19 Install is blocked on low disk space
As a modder with little space, I want to be told before I start an install that needs 25 GB, so that I do not fail halfway.
Acceptance:
- Step 3 rows: `Space needed` (`25 GB of 412 GB free`, ok), `Takes about` `22 min`, `Lands in` `Mods, pinned at the top`.
- Under 20 GB free: `25 GB, only 11 GB free` (fail), banner `Not enough space` / `Free some space or move the Toolbox storage.`, buttons `Free up space…` (opens the cache confirm) and `Change storage` (opens the storage folder picker). `Install` is disabled.
Screens: 31-ttw-04a-step3-low-disk, 32-ttw-04-step3-install
Demo handlers: A.scen (disk), A.cleanCache, A.pickFolder('store')

### US-PAGES-20 Install TTW with progress, cancel and finish
As a modder, I want to watch the TTW install and finish it, so that TTW shows up in my mods.
Acceptance:
- `Install` (primary, download icon). Progress card with stage text from `Checking the Fallout 3 files`, `Converting sound`, `Building archives`, `Patching plugins`, `Adding TTW to your mods`, and `NN % · about N min left`.
- `Cancel` (ghost) stops at once with toast `Stopped`. No confirm (unlike LOD).
- On finish, step 4: banner `Tale of Two Wastelands 3.4 is in your mods` / `Switch it on or off per profile in Mods.`; toast `Tale of Two Wastelands added`. `Done` closes the flow and goes to Mods.
- `Close` (ghost, x) on steps 1 to 4 leaves the flow (and goes to Mods); `Back` on steps 2 and 3 only.
Screens: 33-ttw-05-installing, 34-ttw-06-done
Demo handlers: A.ttwInstall (job ttw, 8 s), A.cancelJob('ttw'), A.ttwClose

---

## System

### US-PAGES-21 Check for updates when System opens
As a modder, I want System to check for updates by itself when I open it, so that I see what is new without pressing anything.
Acceptance:
- Opening System starts a check (if the Settings switch is on and the last check was over 5 minutes ago). Header `Checking for updates…`, a banner with the same text and an indeterminate bar; `Check again` is hidden during the check.
- Afterwards the header reads `Update available` or `Up to date`, followed by `Checked just now`.
- Sidebar badge on System: blue up-arrow icon while an update exists.
Screens: 35-system-01-checking, 36-system-02-after-check
Demo handlers: A.nav('system'), autoCheck, A.checkUpdates

### US-PAGES-22 See what an update changes, per part
As a modder, I want each part's installed and new version and its change list, so that I know what I am accepting.
Acceptance:
- Banner `Update available` / `Updates Raul's Toolbox 0.5.0 → 0.6.0, including nvmod 0.18.0, loot-rs 0.10.0, ttwright 0.6.0`.
- Parts: `Raul's Toolbox` (This app), `Engine` (Runs the game), `lodwright` (Builds the LOD), `nvmod` (Manages mods), `loot-rs` (Sorts the load order), `ttwright` (Builds Tale of Two Wastelands). Parts with a newer version show `<new> · will be updated` in muted text and the installed version on the right.
- Clicking a part row (or pressing Enter when focused) expands its versions: version, date, change tags `Feature` (acc) or `Fix`, and `With the update` (info) / `Installed` (ok) tags.
Screens: 37-system-03-part-toolbox-open, 38-system-04-part-engine-open
Demo handlers: A.vopen

### US-PAGES-23 Update Raul's Toolbox with progress
As a modder, I want to start an update and see its progress and the result, so that I know it finished.
Acceptance:
- `Update` (primary, download icon, title is the update line). Banner `Updating to 0.6.0 · NN %` with stages `Downloading`, `Checking the download`, `Installing the parts`, `Restarting Raul’s Toolbox`.
- Action bar shows `Cancel` (ghost) during the update; cancel toasts `Stopped`.
- On success: toast `Updated to Raul's Toolbox 0.6.0`; banner `Updated to 0.6.0` / `The change list is under Raul's Toolbox below. 0.5.0 is kept, so you can go back.`; the Toolbox row is expanded.
Screens: 43-system-08-update-progress, 45-system-10-updated
Demo handlers: A.update (job update, 7 s)

### US-PAGES-24 Update fails, then retry works
As a modder on a bad connection, I want a clear failure and a retry, so that my current version is safe and I can try again.
Acceptance:
- Failure banner (alert): `The download stopped` / `The server didn't answer. Your current version is untouched.`; toast `The download stopped`.
- The action bar button changes from `Update` to `Try again`.
- Retry runs the update again and reaches the success state of US-PAGES-23.
Screens: 44-system-09-update-failed
Demo handlers: A.update (fail callback), scenario `dl`

### US-PAGES-25 Use or remove an engine build, with confirm
As a modder, I want to switch back to an older engine build, or remove one, after confirming, so that a new engine that misbehaves can be undone.
Acceptance:
- `Engine builds` is folded by default with a count. Rows: `Engine 0.24.0` tag `In use`; `Engine 0.23.0` with `Use this build…` and `Remove…`.
- `Use this build…` dialog `Use engine 0.23.0?`, text `The game runs on 0.23.0 from the next start. You can switch back here.`, button `Use 0.23.0`. Result toast `Engine 0.23.0 in use`.
- `Remove…` dialog `Remove engine 0.23.0?`, text `You can’t switch back to it after this.`, danger button `Remove`. Result toast `Engine 0.23.0 removed`.
- Footnote: `Older builds stay so you can go back if a new one misbehaves.`
Screens: 40-system-05b-engine-builds-open, 41-system-06-use-build-confirm, 42-system-07-remove-build-confirm
Demo handlers: A.useBuild, A.doUseBuild, A.removeBuild, A.doRemoveBuild

### US-PAGES-26 Update the masterlist
As a modder, I want to refresh the masterlist that Sort uses, so that the sort rules are current.
Acceptance:
- Masterlist row: `Load order rules and plugin messages`, sub `Used by Sort on Load order` (or `N new messages`), button `Update masterlist`.
- While updating the button is replaced by a progress bar and the header reads `Updating…`.
- Toast `Masterlist updated: 4 new messages`; header reads `Updated just now`.
Screens: 39-system-05-masterlist-updating
Demo handlers: A.mlUpdate (job ml, 2.5 s)

---

## Doctor

### US-PAGES-27 Run the checks and read the results
As a modder, I want to run all checks and see which are fine, so that I know what to fix.
Acceptance:
- Before a run: header `Not run yet`, button `Run all checks` (primary). Rows show `Not checked`.
- During: header `Checking N of 12`, current row `Checking…`, button `Cancel` (ghost).
- After: header `2 warnings` or `N problem(s)` with warning count, then `Ran just now · <profile>`; button `Run all checks again`. Sidebar Doctor badge shows the count.
- Groups fold: `Game and storage`, `Engine and Toolbox`, `Mods and plugins`, `This profile` (click the header).
Screens: 46-doctor-01-not-run, 47-doctor-02-running, 48-doctor-03-results, 49-doctor-04-folded
Demo handlers: A.docRun, A.docFold, A.cancelJob('doctor')

### US-PAGES-28 Repair a damaged engine from Doctor
As a modder whose engine files changed, I want Doctor to offer a repair, so that I can fix it without leaving the page.
Acceptance:
- Engine row: `Damaged` (fail) with button `Repair engine` (primary). Header `1 problem`.
- Repair runs about 3 s and toasts `Engine repaired`; the row returns to `Engine 0.24.0 · OK`.
Screens: 50-doctor-05-engine-damaged, 51-doctor-06-repairing, 52-doctor-07-repaired
Demo handlers: A.repairEngine, scenario `engine`

---

## Settings

### US-PAGES-29 Game settings for this profile
As a player, I want resolution, frame cap, upscaling, look and V-sync per profile, so that each profile has its own graphics.
Acceptance:
- Section `Game settings for <profile>`: Resolution (1280 × 720, 1920 × 1080, 2560 × 1440, 3840 × 2160), Frame rate limit (60, 90, 120, 144, Off), Upscaling (Off / FSR, `Renders smaller and sharpens. More frames.`), Look (Original / Modern, `Original keeps the game’s own light.`), V-sync (switch).
- Select changes toast `Saved for <profile>`. Segment and switch changes save silently.
Screens: 55-settings-02-game-settings-changed
Demo handlers: IN.gfx, A.gfxSeg, A.gfxVsync

### US-PAGES-30 Storage folder and download cache
As a modder, I want to see and change where Toolbox keeps its files, and clear old archives, so that I can free space.
Acceptance:
- `Storage folder` (`Mods, LOD and backups live here`) with path, `Browse…`, and `<n> GB free`. Under 20 GB the status is a warning.
- `Download cache`: `<n> GB of archives you already installed`, `Clean up…` (disabled under 0.1 GB). Confirm `Clean up the download cache?` with button `Clean up`; toast `Freed <n> GB`.
Screens: 54-settings-01b-all-sections-open, 57-settings-04-clean-cache-confirm, 59-settings-06-storage-picker
Demo handlers: A.pickFolder('store'), A.cleanCache, A.doClean

### US-PAGES-31 Appearance: reduce motion, high contrast, larger text
As a player with sensitivity to motion or low contrast, I want switches for these, so that the app is comfortable to use.
Acceptance:
- `Reduce motion` (`Spinners pulse, bars stay still.`), `High contrast` (`Black background, outlines on every block.`), `Larger text` (no help text). Each is a switch that applies at once.
Screens: 56-settings-03-appearance-on
Demo handlers: A.gset (motion, hc, big)

### US-PAGES-32 Updates setting, TTW row and start setup again
As a modder, I want the auto-check switch, the TTW row and a clear way to restart setup, so that I control updates and can start over safely.
Acceptance:
- `Check for updates when System opens` (`At most once every 5 minutes.`) with switch `Check for updates automatically`.
- TTW row: `Added` or `Not added`; `Show in Mods` when added, `Add…` when not (starts the TTW flow).
- Advanced: `Start setup again` (`Forgets every choice and profile change, then runs setup from the start.`), button `Start setup again…` (danger) with confirm `Start setup again?` and `Start setup again` (danger) button. Cancel keeps everything.
Screens: 58-settings-05-reset-confirm
Demo handlers: A.gset (autoCheck), A.resetAll, A.doReset

---

Totals: 32 stories (Load order 8, LOD 7, TTW 5, System 6, Doctor 2, Settings 4). See `redesign/features/pages.feature` for one scenario per story.
