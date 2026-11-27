# Notes: pages (Load order, LOD, TTW flow, System, Doctor, Settings)

Observed by driving `../design/demo/index.html` with `scripts/pages.js` (59 screenshots, 0 console errors, 0 failed checks) and read from `../design/demo/src/pages.js`, `actions.js`, `core.js`, `overlays.js`, `style.css`. Text in quotes is verbatim from the demo. Timings are the demo's mock durations, not product targets.

Conventions used below: "ghost" = `btn` k:'ghost', "sec" = secondary, "primary" = blue-green filled; "ok/warn/fail/info/run/wait" = status colours (`stt`).

---

## Shared patterns

- Every page has a header: icon, title, status (`stt` chip or muted text), right side (profile pill). Action bar (`abar`) sits bottom-right as a floating strip; it is replaced by the active flow's buttons (Cancel, Apply, Done, Close).
- Page change: `.pg` fades and slides in (`pgin` .22 s, opacity 0 to 1, translateY 6 px to 0).
- Folds (`sech` with click): arrow button on the right, rotates on toggle; hidden body is not rendered (Settings) or set `hidden` (Doctor).
- Dialogs (`.dlg`): scrim fades (`fade` .15 s), dialog pops (`pop` .18 s, scale .97 to 1). Esc or scrim click closes; the install dialog ignores scrim while busy. Dialog focus returns to the opener (`U.ret`).
- Menus (`.menu`): pop .12 s. Esc closes and returns focus to the anchor (verified on the profile pill).
- Toasts: bottom-left stack, `pop` .18 s, dismiss X. Toasts with an action (`Undo`) are the only ones with a button. Several toasts can stack (seen: `Tougher Traders.esp inactive` and `Undone: Tougher Traders.esp inactive` together).
- Sidebar badges: small round count, warn (orange) or err (red) colour. The System badge is a blue up-arrow icon with no number.
- Footer: `Ready` dot on the left, `Shortcuts` (`?`) on the right.
- Busy: `btn.busy` shows a spinning icon and the button text changes to a progress or "Looking…" text; a `busy` button stays clickable in the demo.
- Progress bars (`pb`): fill width transitions .35 s with a faint sheen; indeterminate bar slides 1.2 s, 30 % wide.

---

## Load order (`PAGES.load`)

### Header and status
- Status: `17 plugins · 17 active · 4 out of order` (warn) on Hardcore unsorted. `17 plugins · 17 active · sorted` after sort. Number of plugins active is `allActiveFiles`.
- Chip beside profile pill (`stableChip`): only when some mod is on. `Tested stable` (ok, shield) or `Not tested yet` (warn, warning icon).
- Profile pill: `Profile` label + name + chevron. Opens the profile menu.

### Sections
- `BASE GAME` (lock icon rows, `10 · always on`), `TALE OF TWO WASTELANDS` (`7 · always on`, only when TTW is on). Both fold; the base section starts folded in state (`lbase`: false), the TTW one too (`lttw`: false). Locked rows are dim, grip replaced by lock icon, no checkbox.
- `YOUR PLUGINS` (count of visible plugins). Empty copy: `No plugins yet. Mods with plugins add them here.` (no plugins) or `No plugins match.` (filter/search gives none).

### Search and filter
- Search: `input[type=search]` placeholder `Search plugins`, leading search icon, live filter on each keystroke (no debounce). Match on plugin file or mod name (case-insensitive). Matched text highlighted.
- Segmented filter: `All N` and `Out of order N` (before preview). During preview: `All N` and `Moved N`.
- Enter and Esc in the search box: not tested. Esc is global and closes menus, dialogs and the scenario panel (code).

### Rows
- Row structure: grip (drag handle, title `Drag to move. Keyboard: Alt+Up or Alt+Down.`), number, checkbox (role checkbox, aria-label `<file> active` or `<file> inactive`), file name (monospace, muted when off), mod name, optional error line `Needs <Mod>, which is off or missing` (err icon), optional info line (e.g. `Load near the end: weather records win last.`, `LOD: rebuild after adding or removing this mod.`), end tag.
- Whole row (except the drag grip) is a click target for plugin toggle.
- End tags: `Out of order` (warn) when not sorted; `Up N` / `Down N` (acc, arrow icon) during preview.
- Drop line and ghost while dragging: `Moving` (acc tag) ghost follows pointer.

### Plugin toggle
- Click row or checkbox: plugin on/off. Toast `<file> active` or `<file> inactive` with `Undo` action. Undo stack entry label is the same text. Event log: `Load order changed`. Action bar gains `Undo` (ghost, title `Undo: <label>`).
- Undo toast: `Undone: <label>`.

### Sort preview
- `Sort…` (primary when out of order, sec otherwise, load icon). Click enters preview: `U.sortPrev` holds the moves; filter becomes `All N` / `Moved N`.
- Action bar during preview: `Cancel` (ghost) and `Apply N moves` (primary, check icon) or `Nothing to move` (disabled, when no moves). Single move: `Apply 1 move` (plural helper).
- Preview card: title `The N moves` or `Already sorted`. Rows: file + `Up N`/`Down N` tag. Empty copy: `Every plugin is where the masterlist wants it.` Footnote: `A restore point is made first, and Undo stays after.`
- Apply: toast `Sorted: N moves`, event `Load order sorted`, undo label same. Cancel: no toast.
- Preview is cleared when switching profile (`sortPrev` = null).

### Keyboard
- Focus a plugin name (`button.mono[data-move=plugin]`), Alt+ArrowUp / Alt+ArrowDown moves the plugin one place. Verified Alt+Down moves the row (screenshot 10).
- Space on a focused checkbox toggles it. Verified: `true` to `false`.
- Enter on a row: row click handler (`plugToggle`) fires. Not separately verified.

### Sidebar badge (Load order)
- Count = missing masters + out-of-order count + 1 if the setup is untested (`stableNow` false). Colour err when any missing master, else warn. Title = joined text, e.g. `4 out of order, untested change`. Hardcore: badge 5. After sort: 1 (untested). Note the count is not the number of out-of-order plugins alone (see Bugs).

### Missing masters
- Row line `Needs <Mod>, which is off or missing` in err colour. Doctor reports `Masters` fail with `<plugin> needs <master>` and fix `Show load order`. Not reached in the demo's sorted state; the Hardcore profile has a `B42 Optics - WMX.esp` line that reads `Needs Weapon Mods Expanded.` at the bottom in the unsorted screenshot. Blocked with a plugin off: see `missingMasters`.

### Profile switcher
- Menu headed `Profiles`. Each profile item: label, check icon for active or user icon, `not tested` note when unstable. Disabled while a run is active (except the active item).
- Footer: separator, `New profile…`, `Duplicate “<name>”…`, `Rename “<name>”…`, `Delete “<name>”…` (danger). Delete disabled when only one profile or a run is active.
- Switch: toast `Switched to <name>: mods, load order, settings, LOD and tests`. Clears preview, open rows, picks, play state. Menu closes.
- New/Duplicate/Rename open a name dialog (out of this page's scope; described by the dialog text `Starts empty. Use Duplicate to start from this one.` and `Copies mods, groups, load order, settings, LOD and known stable setups. Test history starts fresh.`).
- Delete confirm: `Delete “<name>”?` / `Its mod choices, load order, settings and test history go. Installed mod files stay for other profiles.` / `Delete profile` (danger).

---

## LOD (`PAGES.far`, `PAGES.farset`)

### States (`farState`) and copy
| State | Status chip | Headline | Sub line | Button | Sidebar badge |
|---|---|---|---|---|---|
| ok | `Up to date` (ok) | `Up to date` | `Matches your N enabled mods.` | `Rebuild LOD` (sec) | none |
| never | `Not built` (warn) | `Not built` | `Far away looks flat until you build it.` | `Build LOD` (primary, play icon) | `LOD not built` (warn, 1) |
| stale-mods | `Out of date` (warn) | `Out of date` | `Mods changed since the last build.` | `Rebuild LOD` (primary) | `LOD out of date` (warn, 1) |
| stale-q | `Out of date` (warn) | `Out of date` | `Quality changed since the last build.` | `Rebuild LOD` (primary) | `LOD out of date` (warn, 1) |
| building | `Building` (run) | `Building · NN %` | stage line | `Cancel…` (sec) | none |

- Mod-change signature: enabled mods, excluding DLL kind, plus TTW flag. Toggling a mod on the Mods page makes it stale-mods.
- Quality change (`farQ`) or fine-tune change makes stale-q when it differs from the built one (fine JSON compared).
- Built tags (right side, `bar`): `Built <ago>` (clock icon), `<size> GB` (box icon), quality label (`Custom` when no preset).

### Building
- Right side: `Building · NN %` (run colour, spin), `<stage> · step k of 5 · about N min left` where stage is `Reading your mods`, `Terrain`, `Objects`, `Trees` or `Packing` by progress; progress bar.
- Job `far:<profile id>`, 9 s in the demo. Build for a profile continues if you switch profile (keyed by profile id). Completion toast `LOD for <name> is ready`, sets built signature, size and mod count.
- The `Cancel…` button sits where Build was; `Cancel…` opens confirm (below).

### Stop build
- Dialog `Stop the LOD build?`, text `The LOD you had stays in place.`, buttons `Cancel` (ghost) and `Stop build` (danger). Stop: toast `Build stopped`, job removed, previous LOD kept.

### Quality
- Segmented: `Performance`, `Balanced`, `High` (labels from `QUAL`). Selecting one sets `far.q` and clears `far.fine` (fine-tune values are discarded with no confirm; see Bugs).
- Line: `<size> GB · about <min> min` (size from `farSize`, minutes from QUAL: 8, 15, 26). `Custom` tag (info) when fine-tune is active.
- Help text: Performance `Smaller textures, nearer distance, sparse trees.`; Balanced `Matches the game’s own LOD.`; High `Sharper textures, farther, denser trees.`; Custom `Your own values from Fine-tune.`
- Size math: base per preset (0.9, 1.9, 3.4 GB) plus 0.3 GB if a `wfo` mod is on, plus 0.8 GB with TTW.
- Demo measured: Performance 1.7 GB built on Hardcore; Beautiful High 3.7 GB, about 26 min.

### Fine-tune (`farset`)
- Title `LOD settings`, subtitle `<quality or Custom> · used by the next build`. Buttons: `Restore defaults` (ghost, undo icon, disabled until fine-tuned) and `Done` (primary, returns to LOD).
- Rows (switch rows are full-row click targets):
  - `Texture size` / `Bigger looks sharper and uses more video memory.` Segmented `256`, `512`, `1024`.
  - `Distance` / `How far LOD reaches · <Near|Short|Normal|Far|Very far>`. Range input 1 to 5, aria-label `Distance`. Label updates live (1 Near, 2 Short, 3 Normal, 4 Far, 5 Very far).
  - `Tree density` / `Trees and big shrubs in the distance.` Segmented `Sparse`, `Normal`, `Dense`.
  - `Far desert plants` / `Small desert plants in the distance. Costs some frames.` Switch.
- Defaults per preset: Performance 256 / 2 / sparse / no plants; Balanced 512 / 3 / normal / no plants; High 1024 / 5 / dense / plants on.
- Each change writes `far.fine` (copy of current values) and focus goes back to the control.
- `Restore defaults` clears `far.fine`; the page shows the preset again.

### Details
- Header `Details` folds (default closed). Body: `Profile` (`<name> · each profile keeps its own LOD`), `Built` (date and time or `Never`), `Quality`, `Size`, `Worlds` (`Mojave Wasteland and the DLC worlds`, plus `, plus the Capital Wasteland` with TTW), `Mods in it` (`N mods`) or `–`.

---

## TTW flow (`PAGES.ttw`, `ttwStart`, `ttwNext`, `ttwBack`, `ttwClose`, `ttwInstall`)

### Entry points
- Mods page card when TTW not added: name `Tale of Two Wastelands`, sub `Fallout 3 inside New Vegas`, tag `Not added`, button `Add…` (primary, plus icon).
- Settings row `Add…` (ghost). Doctor row `Add TTW…` (fix action, when this profile uses TTW and not added).
- Flow starts at step 1 (index 0). If no Fallout 3 path is saved, a Find runs automatically.
- Mods card when added (for reference): switch `ttwToggle`, `Update available` tag if update, `Bundle` tag, version, chevron. Expanded: `Check for TTW update` (sec) or `Update TTW…` (primary, when update), and `What it does`, `Plugins`, `Off` rows.

### Header and stepper
- Title `Add Tale of Two Wastelands`, status `Step N of 4`.
- Stepper labels: `Fallout 3`, `TTW file`, `Install`, `Done`. Sub-labels: `Found`/`Not found`, `Found`, `Waiting` or `NN %` or `Done`, `Up to date`/`Update available`.

### Step 1: Fallout 3
- Row: name `Fallout 3`, sub = path (mono) or `No Fallout 3 folder found`. End: status `Looking…` (run) while Find runs, `Found` (ok) or `Not found` (fail), then buttons `Browse…` (ghost, folder icon) and `Find` (ghost, search icon, busy while searching).
- Help under row when not found: `TTW needs Fallout 3 GOTY installed. Steam and GOG copies both work.`
- Next: `Next: TTW file` (primary), disabled when not found.
- Find (1.5 s): found toasts nothing and sets path; not found: toast `Fallout 3 not found. Try Browse`.
- Browse: dialog `Pick the Fallout 3 folder`, list of 3 folders (Steam library `D:\SteamLibrary\steamapps\common\Fallout 3 goty`, GOG Games, Documents). Buttons `Cancel` (ghost) and `Use this folder` (primary). Non-Fallout folder: toast `No Fallout 3 in that folder`, path not saved. Real folder: toast `Fallout 3 folder saved`, status Found.

### Step 2: TTW file
- Row `TTW installer file`, sub `Downloads › TTW_3.4.0.mpi` (mono), status `Found` (ok), button `Browse…` (opens file browser: toast `Your file browser opened`).
- Button `Get the file` (ghost, link icon): toast `The TTW download page opened in your browser`.
- Actions: `Back` (ghost), `Next: Install` (primary).

### Step 3: Install
- Rows: `Space needed` with `25 GB of <n> GB free` (ok) or `25 GB, only <n> GB free` (fail); `Takes about` `22 min`; `Lands in` `Mods, pinned at the top`.
- Low space: free < 20 GB. Banner (err) `Not enough space` / `Free some space or move the Toolbox storage.` with `Free up space…` (sec, runs cache clean confirm) and `Change storage` (sec, storage folder picker).
- `Install` (primary, download icon), disabled when low. Back and Close present.

### Install progress and finish
- Card with spinner, stage text by progress: `Checking the Fallout 3 files`, `Converting sound`, `Building archives`, `Patching plugins`, `Adding TTW to your mods`; sub `NN % · about N min left`; bar. Job `ttw`, 8 s.
- Action bar: `Cancel` (ghost) only. Cancel runs at once with toast `Stopped`. No confirm.
- Finish: TTW marked added, switched on for this profile, toast `Tale of Two Wastelands added`, step 4. Banner (ok): `Tale of Two Wastelands 3.4 is in your mods` / `Switch it on or off per profile in Mods.`. Button `Done` (primary) which closes the flow.
- Close (ghost, x) on steps 1 to 4 when not installing: returns to Mods. Back shown on steps 2 and 3 only. Close and Back are hidden while the job runs; Cancel replaces both.

### Demo quirks
- Finishing an install leaves the flow even if the user had already gone (job checks `U.ttwFlow`).
- TTW version `3.4` is shown in the banner; the file name is `TTW_3.4.0.mpi`.

---

## System (`PAGES.system`)

### Header and checks
- Status: `Checking for updates…` (run), `Updating` (run), `Update available` (info), `Up to date` (ok). Then muted `Checked <ago>` when a check has run.
- Auto check: when System opens and Settings `Check for updates when System opens` is on and last check is over 5 minutes ago and no check runs. Also run by a render guard when the wizard is done. Job `check`, 1.8 s.
- Checking banner (acc): `Checking for updates…` with indeterminate bar. `Check again` (sec, refresh icon) hidden during a check.
- Update available banner (info): `Update available` / `Updates Raul's Toolbox 0.5.0 → 0.6.0, including nvmod 0.18.0, loot-rs 0.10.0, ttwright 0.6.0`. Lists only parts with a newer version (not the Toolbox itself, not engine or lodwright).
- Action bar when available: `Check again` (sec) and `Update` (primary, download icon, tooltip is the update line) or `Try again` after a failure.

### Parts list
- Heading `Toolbox and its parts` with info icon (tooltip `The parts ship inside the app and update with it.`).
- Row: icon, name, role (muted), end: `<new> · will be updated` (muted) when newer, installed version (mono). Chevron rotates when open.
- Parts: `Raul's Toolbox` (This app, 0.5.0 to 0.6.0), `Engine` (Runs the game, 0.24.0 current), `lodwright` (Builds the LOD, 0.17.0), `nvmod` (Manages mods, 0.17.1 to 0.18.0), `loot-rs` (Sorts the load order, 0.9.0 to 0.10.0), `ttwright` (Builds Tale of Two Wastelands, 0.5.0 to 0.6.0).
- Expanded: per version: version (mono), date, summary line (e.g. `Setup installs the engine without a separate program`), tag `With the update` (info) or `Installed` (ok), then change rows with tag `Feature` (acc) or `Fix` and text.
- Keyboard: part row is `role=button`, `tabindex=0`. Enter expands/collapses (verified). Only one part is open at a time (`vopen` is a single id).
- No part menu exists. The row only expands.

### Update progress, failure, success
- Banner (acc): `Updating to 0.6.0 · NN %` with spinner, stage: `Downloading`, `Checking the download`, `Installing the parts`, `Restarting Raul’s Toolbox`. Progress bar. Job `update`, 7 s.
- Action bar: `Cancel` (ghost) only. Cancel: toast `Stopped`, no confirm.
- Failure (scenario `dl` on): job fails at 35 %. Alert banner (err, role alert): `The download stopped` / `The server didn't answer. Your current version is untouched.` Toast `The download stopped`. Action button becomes `Try again`.
- Success: toolbox version 0.6.0, toast `Updated to Raul's Toolbox 0.6.0`, banner (ok) `Updated to 0.6.0` / `The change list is under Raul's Toolbox below. 0.5.0 is kept, so you can go back.` Toolbox row is opened. After success `Update available` banner and badge go, and Doctor shows `0.6.0 · newest`.

### Engine builds
- Section `Engine builds` (folded by default; count 2). Footnote `Older builds stay so you can go back if a new one misbehaves.`
- Row: bolt icon, `Engine <ver>` (mono version). Current: tag `In use` (ok). Others: `Use this build…` (sec, small) and `Remove…` (ghost, small).
- Use confirm: title `Use engine 0.23.0?`, text `The game runs on 0.23.0 from the next start. You can switch back here.`, button `Use 0.23.0` (primary). Result toast `Engine 0.23.0 in use` (current build changes).
- Remove confirm: title `Remove engine 0.23.0?`, text `You can’t switch back to it after this.`, button `Remove` (danger). Result toast `Engine 0.23.0 removed`.

### Masterlist
- Section `Masterlist` with right text `Updated <ago>` or `Updating…`. Row: `Load order rules and plugin messages` with sub `Used by Sort on Load order` (or `N new messages`), button `Update masterlist` (sec, small, refresh icon). While running: progress bar (140 px) in place of button. Job `ml`, 2.5 s. Result toast `Masterlist updated: 4 new messages`.

---

## Doctor (`PAGES.doctor`)

### Header and run
- Not run: `Not run yet` (muted). Button `Run all checks` (primary, refresh icon).
- Running: header `Checking N of 12` (run, spin), current row shows `Checking…`, button `Cancel` (ghost). Job `doctor`, 11 × 320 ms = 3.5 s.
- After: header with `N problem(s)` (fail) and `N warning(s)` (warn), or `N warnings`, or `All good` (ok). Then muted `Ran <ago> · <profile>`. Button `Run all checks again` (primary).
- Sidebar Doctor badge: red count when any problem, otherwise warn count (`2 warnings` as title).

### Groups and rows (12 checks)
- `Game and storage` (3): `Game folder` (Found), `Game files` (Intact, or `Needs the game folder` when folder is missing), `Toolbox storage` (`412 GB free`, or `Low on space` warn).
- `Engine and Toolbox` (2): `Engine` (`0.24.0 · OK`, or `Damaged` with `Repair engine`), `Raul's Toolbox` (`Update available` info with `Open System`, or `0.6.0 · newest` ok).
- `Mods and plugins` (4): `Masters` (`All present`, or `<plugin> needs <master>` fail with `Show load order`), `Load order` (`Sorted` ok, or `N out of order` warn with `Sort…`), `Plugin count` (`N of 255`, warn above 230), `Script extenders` (`Built into the engine`).
- `This profile` (3): `LOD` (`Up to date`, or `Not built`/`Out of date` with `Build LOD`/`Rebuild LOD`), `Mod setup` (`Tested stable` or `Not tested yet` with `Run stability test`), `Tale of Two Wastelands` (`Not used`, `Not added` with `Add TTW…`, or `Added`).
- Row states before a run: `Not checked` (wait). Current row: `Checking…` (run). Done rows show the status. Fix button only on rows not ok; `primary` when fail, `sec` otherwise.
- Group header count = rows in group. Fold: `docFold` toggles; default open. Hidden rows use `hidden`.

### Repair engine
- Button `Repair engine` (primary, small) from the Engine row. Job `repair`, 3 s. Toast `Engine repaired`. Engine status returns to OK. (Repair is a fix button on the check; the repair job itself is not shown in a dialog.)

---

## Settings (`PAGES.settings`)

### Sections (fold, click header)
- `Game installations` (open by default), `Toolbox storage` (closed), `Game settings for <profile>` (closed), `Appearance` (closed), `Updates` (closed), `Advanced` (closed). Header right: `Changes save as you go`.

### Game installations
- `Fallout: New Vegas` / `Required`: path (mono, ellipsis, 340 px) + `Browse…` (ghost, folder) + `Find` (ghost, search, busy `Looking…`) + status: `Found` (ok), `No game here` (fail, when a path exists but scenario game), `Not found` (fail).
- `Fallout 3` / `For TTW`: same without the required note, status `Found` or `Not found`.
- `Tale of Two Wastelands` row: `Added` (ok) / `Not added` (wait) and button `Show in Mods` (ghost, goes to Mods) or `Add…` (ghost, starts the TTW flow).
- Folder dialog (`Browse…`): title `Pick the game folder` / `Pick the Fallout 3 folder` / `Pick the storage folder`. Listbox of folders with free-space for storage. Buttons `Cancel` and `Use this folder`. Game folder without `FalloutNV.exe`: toast `No FalloutNV.exe in that folder`. Saved: toast `Game folder saved` (game), `Fallout 3 folder saved` (fo3), `Storage moved. <n> GB free` (store).

### Toolbox storage
- `Storage folder` / help `Mods, LOD and backups live here`: path, `Browse…` (no Find), status `<n> GB free` (warn under 20 GB, ok otherwise).
- `Download cache` / help `<n> GB of archives you already installed`: `Clean up…` (sec, small). Disabled under 0.1 GB. Confirm `Clean up the download cache?` / `Frees <n> GB. These are archives of mods you already installed; you can download them again.` / button `Clean up`. Result toast `Freed <n> GB`. Under 0.1 GB click: toast `The download cache is already empty`.

### Game settings for <profile>
- `Resolution` (select: 1280 × 720, 1920 × 1080, 2560 × 1440, 3840 × 2160). Change: toast `Saved for <profile>`.
- `Frame rate limit` (select: 60, 90, 120, 144, Off). Change: toast `Saved for <profile>`.
- `Upscaling` / `Renders smaller and sharpens. More frames.`: segmented `Off`, `FSR`. Silent save (version recorded).
- `Look` / `Original keeps the game’s own light.`: segmented `Original`, `Modern`. Silent save.
- `V-sync` (switch). Silent save.

### Appearance
- `Reduce motion` / `Spinners pulse, bars stay still.` Switch. Verified: with it on, the indeterminate bar stays at one position (121.7 px in the test) and spinners pulse at 1.6 s.
- `High contrast` / `Black background, outlines on every block.` Switch.
- `Larger text` (no help). Switch.

### Updates
- `Check for updates when System opens` / `At most once every 5 minutes.` Switch labelled `Check for updates automatically`.

### Advanced
- `Start setup again` / `Forgets every choice and profile change, then runs setup from the start.` Button `Start setup again…` (sec, danger text, refresh icon).
- Confirm: `Start setup again?` / `Raul’s Toolbox forgets every choice, profile change and test result, then runs setup from the start. Your game folder is not touched.` / button `Start setup again` (danger). Result resets everything and returns to the wizard.

---

## Keyboard and focus (verified or read from code)

- Esc: closes the profile menu and returns focus to the pill (verified); closes dialogs (code); closes the scenario panel (code). Does not reset page state.
- Ctrl+K opens the palette (code, not on these pages' scope). Ctrl+Shift+D toggles the test-situations panel (used by the script).
- Alt+ArrowUp/ArrowDown on focused plugin name moves it (verified Down).
- Space on a focused plugin checkbox toggles it (verified).
- Enter on a System part row expands it (verified).
- Focus outline: 1 px teal, 1 px offset on `:focus-visible`. Rows with focus get a selected background instead of an outline (`.r:focus-visible`).
- Inputs: no outline, only a state change.
- Not verified: Tab order across the action bar; Enter on the confirm dialogs.

## Alignment

- Status chip sits next to the page title; right side of header holds the profile pill. Pill is at the right of the header row in all pages.
- Action bar is bottom-right, toast stack bottom-left, both above the footer.
- Row numbers in Load order are right-aligned in a fixed column; tags align at the far right.
- Settings: label on left (min 200 px), controls right in a fixed-width end column. Path field is 340 px.
- LOD hero: two columns (left: headline, right: status and build tags).

## Animations (from style.css)

- Page fade and slide in: `pgin` .22 s.
- Detail expansion (`mdet`): `pgin` .18 s.
- Dialogs and toasts: `pop` .18 s; menus `pop` .12 s; scrim `fade` .15 s.
- Spinners (`.spin`, busy buttons): `spin` 1 s linear, infinite.
- Progress fill: width .35 s ease, sheen gradient; indeterminate bar `slide` 1.2 s ease-in-out, infinite, 30 % wide.
- Switch knob and track: .15 s. Chevron: .18 s transform. Row background: .15 s. Button background: transition.
- Footer busy dot: `pulse` 1.4 s.
- Reduced motion (OS `prefers-reduced-motion` or the Settings switch, which sets `data-motion="0"`): all animation and transition durations 0.01 ms; spinners become opacity pulse 1.6 s. Verified the indeterminate bar stops.

## Empty, busy, error states (summary)

- Load order: `No plugins yet. Mods with plugins add them here.`; `No plugins match.`; sorted card `Already sorted`.
- LOD: never built (flat copy); building (progress); out of date (two copies).
- TTW: Fallout 3 not found (Next disabled); low space (Install disabled + banner); installing (Cancel only).
- System: checking (indeterminate bar); update failed (alert banner and `Try again`); up to date (no banner).
- Doctor: not run (`Not run yet`); running (`Checking N of 12`); results.
- Settings: `Not set` for an empty path; `Not found` / `No game here` statuses; cache empty toast.

---

## Demo bugs and oddities found

1. Load order sidebar badge mixes counts: Hardcore shows badge `5` and page header says `4 out of order`, because the badge adds 1 for the untested change. The number does not match the visible count, only the title explains it.
2. Quality preset click (`farQ`) silently throws away fine-tune values with no confirm or undo. Easy to lose a tuned setup.
3. Stop control consistency: LOD `Cancel…` asks to confirm; TTW `Cancel` and System update `Cancel` stop at once with no confirm, though the LOD one says the existing LOD stays.
4. System success banner says `0.5.0 is kept, so you can go back.` There is no UI to go back to 0.5.0 on this page.
5. Hard-coded version strings: `updLine` (`Raul's Toolbox 0.5.0 → 0.6.0`), `Updating to 0.6.0`, `Updated to 0.6.0`, Doctor `0.6.0 · newest`, masterlist toast `4 new messages`, TTW banner `3.4`. Only partly tied to state.
6. Engine remove copy `You can’t switch back to it after this.` sits next to the footnote `Older builds stay so you can go back`. Not wrong, but the two messages pull in opposite directions.
7. The Settings `Storage folder` status uses the same text in both warn and ok branches (`<n> GB free`), so the colour is the only signal under 20 GB.
8. Settings sections start folded except `Game installations`, so the resolution and frame-cap rows are hidden until the user opens `Game settings for <profile>`. Intended, but a new user may not find them.
9. `Undone:` toast stacks under the previous `inactive` toast, so the same plugin name shows twice on screen after an undo (seen in screenshot 06). Acceptable, but noisy.

Scripts: `scripts/pages.js` (re-runnable; exits 1 on console error or failed check).
