# Gap audit: current app against the inventory

The state of the app (Toolbox repository, `origin/main` at commit `1634aea`, GUI crate `crates/raul-gui` with its
`raul-ui-model` and `raul-ui-service` sub-crates, plus `raul-core` and the other crates) against each function in
`FUNCTIONALITY.md`.

- **exists**: the app does it the same way.
- **differs**: it exists but behaves or looks different; the note says how.
- **missing**: not in the app.

## Summary

The app is a smaller working Toolbox with its own page set (Home, Mods, Load order, LOD, a Versions/About page, Settings, plus the
Setup, TTW and LOD settings flows). It follows an earlier mock-up, not the demo. The headline gaps: no toasts, no
menus, no shortcuts sheet, no quit confirm, no floating action bar, no profiles in the UI, no groups, no stability test,
no benchmark or pictures page, no versions or revert, no Doctor page, no System parts list, no accessibility options, and
a five-step setup with no profile or LOD step. Page text and sizes mostly differ from the demo (for example the TTW flow
asks for 40 GB where the demo says 25 GB, LOD says "Generate", the dialogs are 560 px).

| Area | exists | differs | missing |
|---|---|---|---|
| SHELL | 0 | 9 | 1 |
| OVERLAY | 0 | 5 | 7 |
| ACCESS | 0 | 2 | 4 |
| SETUP | 0 | 6 | 4 |
| HOME | 0 | 3 | 1 |
| PLAY | 0 | 1 | 3 |
| MODS | 0 | 9 | 3 |
| INSTALL | 0 | 4 | 1 |
| TEST | 0 | 0 | 6 |
| VERS | 0 | 0 | 3 |
| PROF | 0 | 0 | 4 |
| LOAD | 0 | 2 | 4 |
| LOD | 0 | 4 | 2 |
| TTW | 0 | 5 | 0 |
| SYSTEM | 0 | 2 | 3 |
| DOCTOR | 0 | 0 | 3 |
| SETTINGS | 0 | 1 | 5 |
| **All 107** | **0** | **53** | **54** |

Nothing counts as "exists" yet, because every function has at least a text, size, step or behaviour difference. Several
"differs" entries are close and need small changes (for example F-LOAD-05, F-LOD-03, F-SETUP-05, F-TTW-04, F-PLAY-04).

Useful things that already exist and should be reused: the headless test harness (`egui_kittest` with snapshots in
`crates/raul-gui/tests/`), the pure palette model (`raul-ui-model/src/palette.rs`, not drawn yet), the key map
(`raul-ui-model/src/keymap.rs`, only number keys wired), the rollback journal behind the Undo buttons, the fake
service (`raul-ui-service/src/fake.rs`), the dialog host (`src/dialog.rs`), job tracking (`src/jobs.rs`) and the sidebar
severity chips.

How this was checked: the code was read per function by two independent readers, each with the list of functions and
their texts. A sample of the answers was then checked by hand against the code (no toast code in the GUI; palette model
present and unwired; no profile-choice or LOD step in the setup; Load order has no search; the TTW flow needs 40 GB;
the update flow has "Go back to version"; the shell wires number keys only; the footer says "Ready."). All matched.
Treat file references as starting points, not proof of every word.

## Shell

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-SHELL-01 | Window | differs | Real resizable 1280x800 window with growing panels; native OS title bar, no custom 34px Min/Max/Close. | `crates/raul-gui/src/lib.rs:native_options`<br>`crates/raul-gui/src/shell.rs:App::show` |
| F-SHELL-02 | Top bar | differs | Brand and New Vegas chip open Settings; no engine chip, profile pill, search button or Ctrl K. | `crates/raul-gui/src/shell.rs:App::top_bar`<br>`crates/raul-gui/src/shell.rs:App::play_block` |
| F-SHELL-03 | Play button states | differs | Plain Play with blocked-only tooltip; Home shows Starting and Stop; no Checking N/M or Stop · mm:ss. | `crates/raul-gui/src/shell.rs:App::top_bar`<br>`crates/raul-gui/src/pages/home.rs:Home::launch_panel` |
| F-SHELL-04 | Sidebar | differs | Two groups but Versions/Settings, not Doctor; 224px, 34px rows, accent bar not icon; no Enter-to-open. | `crates/raul-gui/src/shell.rs:App::nav`<br>`crates/raul-gui/src/shell.rs:nav_entry` |
| F-SHELL-05 | Sidebar badges | differs | Severity chips exist; no Mods/System/Doctor badges, reason tooltips or specified colour rules. | `crates/raul-gui/src/shell.rs:nav_entry`<br>`crates/raul-gui/src/pages/load_order.rs:LoadOrder::attention` |
| F-SHELL-06 | Status line | differs | "Ready.", About button and "N running"; no state dot, job percent, "N more" or Shortcuts. | `crates/raul-gui/src/shell.rs:App::footer`<br>`crates/raul-gui/src/catalogue.rs:SHELL` |
| F-SHELL-07 | Page header | differs | Title plus one state line, but no icon tile, profile pill or stable chip; header holds page buttons. | `crates/raul-gui/src/shell.rs:App::content`<br>`crates/raul-gui/src/pages/mod.rs:Page::tools` |
| F-SHELL-08 | Floating action bar | differs | Actions live in header tools or in-page footer; no floating pill, equal 36px buttons or 84px padding. | `crates/raul-gui/src/pages/mod.rs:Page::tools`<br>`crates/raul-gui/src/pages/setup.rs:Setup::footer` |
| F-SHELL-09 | Reduced overlap of last row | missing | No floating action bar exists; scroll area reserves no bottom clearance for one. | `crates/raul-gui/src/shell.rs:App::content`<br>`crates/raul-gui/src/pages/mod.rs:Page::ui` |
| F-SHELL-10 | Sidebar, top bar and wizard chrome | differs | Wizard keeps full shell with sidebar and Play; header says "Step N of 5", no Ctrl K/? gating. | `crates/raul-gui/src/pages/setup.rs:Setup::state_line`<br>`crates/raul-gui/src/shell.rs:App::show` |

## Overlays

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-OVERLAY-01 | Toasts | missing | No toast state or widget; errors render as inline selectable cards, not 440px toasts. | `crates/raul-gui/src/shell.rs:error_widget` |
| F-OVERLAY-02 | Undo | differs | Undo buttons with last-change tooltips via rollback journal; no 30-deep stack, toast or "Undone:". | `crates/raul-gui/src/pages/mods.rs:Mods::undo_row`<br>`crates/raul-gui/src/pages/load_order.rs:LoadOrder::tools` |
| F-OVERLAY-03 | Dialog frame | differs | egui Modal 560px, radius 12, margin 22/20; scrim closes; no 520/760, Enter/Tab or focus rules. | `crates/raul-gui/src/dialog.rs:Dialogs::show` |
| F-OVERLAY-04 | Confirm dialogs | missing | Generic confirm exists, but none of the listed profile/group/engine/revert/clean/setup/quit confirmations. | `crates/raul-gui/src/dialog.rs:Dialog::confirm` |
| F-OVERLAY-05 | Name dialog | missing | No create/rename/duplicate profile or group UI; no name dialog, 40-char limit or duplicate error. | `crates/raul-gui/src/pages/settings.rs:Settings::ui` |
| F-OVERLAY-06 | Folder picker | missing | Folders are plain text fields; no picker dialog, free space, row selection or "Use this folder". | `crates/raul-gui/src/pages/settings.rs:Settings::form` |
| F-OVERLAY-07 | Menus | missing | No menu widget or anchored popups; profile, group and benchmark-point menus absent. | `crates/raul-gui/src/shell.rs:App::top_bar` |
| F-OVERLAY-08 | Command palette | differs | Pure palette model, never rendered or wired in GUI; no kind tags, mods or "Nothing found." | `crates/raul-gui/crates/raul-ui-model/src/palette.rs:entries`<br>`crates/raul-gui/crates/raul-ui-model/src/palette.rs:Palette::results` |
| F-OVERLAY-09 | Shortcuts sheet | missing | Chords exist as data; no shortcuts sheet, "?" key or Shortcuts button. | `crates/raul-gui/crates/raul-ui-model/src/keymap.rs:Keymap::default`<br>`crates/raul-gui/src/shell.rs:App::keys` |
| F-OVERLAY-10 | Quit confirm | missing | No quit confirmation; viewport Close used only by smoke test and update restart. | `crates/raul-gui/src/shell.rs:App::closing_after`<br>`crates/raul-gui/src/pages/versions.rs:about_section` |
| F-OVERLAY-11 | Busy states | differs | Plain unanimated progress bar and static icon; no spin, sheen, "Looking…" or reduce-motion pulse. | `crates/raul-gui/src/pages/ttw.rs:Ttw::install`<br>`crates/raul-gui/src/pages/settings.rs:Settings::waiting` |
| F-OVERLAY-12 | Open links | differs | TTW download opens browser tab; no original-mod, repo or file-browser links. | `crates/raul-gui/src/pages/ttw.rs:Ttw::package`<br>`crates/raul-gui/src/pages/versions.rs:about_section` |

## Focus, keyboard, display options

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-ACCESS-01 | Focus look | differs | Focus ring is 2px accent on clicks too; no 1px 75% keyboard-only outline or palette tint. | `crates/raul-gui/src/theme.rs:visuals`<br>`crates/raul-gui/src/shell.rs:nav_entry` |
| F-ACCESS-02 | Whole-row click targets | missing | Row click selects, checkbox toggles; no whole-row fold/switch or 44x44 hit areas. | `crates/raul-gui/src/pages/mods.rs:Mods::list` |
| F-ACCESS-03 | Keyboard map | differs | Chords declared but shell only handles number keys; no Alt+Up/Down, Space, Enter or Escape chain. | `crates/raul-gui/crates/raul-ui-model/src/keymap.rs:Keymap::default`<br>`crates/raul-gui/src/shell.rs:App::keys` |
| F-ACCESS-04 | High contrast | missing | Only dark tokens installed; no high-contrast setting under Appearance. | `crates/raul-gui/src/lib.rs:set_up`<br>`crates/raul-gui/src/pages/settings.rs:Settings::ui` |
| F-ACCESS-05 | Larger text | missing | Base font stays 14px; no larger-text mode or 15.5px switch. | `crates/raul-gui/src/theme.rs:apply`<br>`crates/raul-gui/src/pages/settings.rs:Settings::ui` |
| F-ACCESS-06 | Reduce motion | missing | No reduce-motion switch or OS detection; indeterminate bar parks at 0, not 35%. | `crates/raul-gui/src/pages/settings.rs:Settings::ui`<br>`crates/raul-gui/src/pages/ttw.rs:Ttw::install` |

## Setup wizard

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-SETUP-01 | Wizard frame | differs | Five-step rail with done/current and Back/Next; no eight steps or reload resume. | `crates/raul-gui/src/pages/setup.rs:rail`<br>`crates/raul-gui/src/pages/setup.rs:Setup::footer` |
| F-SETUP-02 | Welcome | missing | No "About 5 minutes, plus the LOD build"; only "It only takes a minute." | `crates/raul-gui/src/pages/setup.rs:Setup::ui` |
| F-SETUP-03 | Game step | differs | Find, not-found note and Next gate; no "Looking…", version/DLC, wrong-folder text or Browse. | `crates/raul-gui/src/pages/setup.rs:Setup::new_vegas`<br>`crates/raul-gui/src/pages/setup.rs:Setup::footer` |
| F-SETUP-04 | Storage step | missing | Storage path is a text field; no default drive, free-space bar, 30 GB copy or low-space banner. | `crates/raul-gui/src/pages/setup.rs:Setup::ui` |
| F-SETUP-05 | Engine step | differs | Init/target stages with Try again; no Downloading/Checking/Unpacking engine stages. | `crates/raul-gui/src/pages/setup.rs:Setup::instance`<br>`crates/raul-gui/src/pages/setup.rs:Setup::set_up` |
| F-SETUP-06 | TTW step | differs | Optional "Not now"/"I want TTW" cards; install is separate flow, no Browse, skip-undo or four wizard stages here. | `crates/raul-gui/src/pages/setup.rs:Setup::ttw`<br>`crates/raul-gui/src/pages/ttw.rs:Ttw::ui` |
| F-SETUP-07 | Profile step | missing | No profile-choice step; no Vanilla+/Beautiful/Hardcore/Empty cards, TTW line or Tested stable tag. | `crates/raul-gui/src/pages/setup.rs:Setup::ui` |
| F-SETUP-08 | LOD step | missing | No LOD step in setup; no quality presets, preview, Build now/Later or "Build now instead". | `crates/raul-gui/src/pages/setup.rs:Setup::ui` |
| F-SETUP-09 | Done step | differs | Summary lists Game, Storage path, TTW, Instance, Checks; no free space, engine, profile or LOD rows. | `crates/raul-gui/src/pages/setup.rs:Setup::ready`<br>`crates/raul-gui/src/pages/setup.rs:Setup::footer` |
| F-SETUP-10 | Start setup again | differs | "Run setup again" opens wizard directly; no Advanced section or forget-confirmation wording. | `crates/raul-gui/src/pages/settings.rs:Settings::ui` |

## Home

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-HOME-01 | Layout | differs | Two columns when wide; Storage always shown; single-column attention-only layout absent. | `crates/raul-gui/src/pages/home.rs:Home::ui`<br>`crates/raul-gui/crates/raul-ui-model/src/pages/home.rs:State::rows` |
| F-HOME-02 | Stability banner | missing | No stability-test card, Not now, Revert to last stable, Run stability test or running-test progress. | `crates/raul-gui/src/pages/home.rs:Home::ui` |
| F-HOME-03 | Status card | differs | "Ready to play", Checking and running states; no live profile, stability tag, Last played or playtime total. | `crates/raul-gui/src/pages/home.rs:Home::state_line`<br>`crates/raul-gui/src/pages/home.rs:Home::launch_panel` |
| F-HOME-04 | Setup list | differs | Fixed renamed rows, Storage always shown; no Mods, engine-damaged, "N out of order" or mod-setup rows. | `crates/raul-gui/src/pages/home.rs:readiness_row`<br>`crates/raul-gui/crates/raul-ui-model/src/pages/home.rs:State::rows` |

## Play

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-PLAY-01 | Play check | missing | Play performs no recheck: no per-row Checking/Waiting, progress, Cancel or "Checking N/M". | `crates/raul-gui/src/pages/home.rs:Home::launch_panel` |
| F-PLAY-02 | Warnings card | missing | No "Before you play: N warnings" card with fix buttons, Revert or Play anyway. | `crates/raul-gui/src/pages/home.rs:Home::launch_panel` |
| F-PLAY-03 | Failed card | missing | No "Play stopped: N checks failed" card, reasons, Check again or first-failure fix. | `crates/raul-gui/src/pages/home.rs:Home::launch_panel` |
| F-PLAY-04 | Starting and running | differs | Starting and Stop without confirm exist; no 1.5s "Loading your mods", session record or playtime toast. | `crates/raul-gui/src/pages/home.rs:Home::launch_panel`<br>`crates/raul-gui/src/pages/home.rs:Home::stop_game` |

## Mods list

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-MODS-01 | Page frame | differs | Header shows "{n} Mods, {e} enabled"; no tabs, profile pill, stable chip or untested card. | `crates/raul-gui/src/pages/mods.rs:Mods::state_line`<br>`crates/raul-gui/src/pages/mod.rs:Page::pinned` |
| F-MODS-02 | Search and filters | differs | Search matches names only, no highlighting; filters All/Enabled/Disabled, no counts or Attention. | `crates/raul-gui/src/pages/mods.rs:Mods::search`<br>`crates/raul-gui/crates/raul-ui-model/src/pages/mods.rs:State::visible` |
| F-MODS-03 | Built-in engine section | missing | No foldable "Built into the engine" section, lock icon, "Built in" states or two links. | `crates/raul-gui/src/pages/mods.rs:Mods::ui` |
| F-MODS-04 | Locked plugin lists | differs | All plugins in one load-order table; no Base game/TTW sections or bundled-plugin detail list. | `crates/raul-gui/src/pages/load_order.rs:LoadOrder::table`<br>`crates/raul-gui/src/pages/mods.rs:Mods::detail` |
| F-MODS-05 | TTW row | differs | No pinned TTW row on Mods; separate flow, no Bundle tag, update buttons or per-profile switch. | `crates/raul-gui/src/pages/mods.rs:Mods::ui`<br>`crates/raul-gui/src/pages/ttw.rs:Ttw::ui` |
| F-MODS-06 | Groups | missing | No groups at all: no create/rename/delete, fold, tri-state switch, move, drag or Ungrouped. | `crates/raul-gui/src/pages/mods.rs:Mods::list` |
| F-MODS-07 | Mod row | differs | Row has number, checkbox, name, tag; no grip, chevron, version, attention tag or fold-out. | `crates/raul-gui/src/pages/mods.rs:Mods::list`<br>`crates/raul-gui/src/pages/mods.rs:tag` |
| F-MODS-08 | Mod details | differs | Detail shows state, priority, Enable/Disable, Move up/down, Remove; no What it does, Plugins, Options, Size, date. | `crates/raul-gui/src/pages/mods.rs:Mods::detail` |
| F-MODS-09 | Drag and drop | missing | No pointer drag, Moving ghost, drop line, auto-scroll, Escape cancel or group drops. | `crates/raul-gui/src/pages/mods.rs:Mods::list` |
| F-MODS-10 | Remove mod | differs | Dialog asks "Remove this Mod?" per Instance, not all profiles; no archive/Undo wording. | `crates/raul-gui/src/pages/mods.rs:Mods::ask_remove` |
| F-MODS-11 | Missing master | differs | Sidebar chips and Doctor command exist; no "Needs X" tag, shared badge rule or Doctor page. | `crates/raul-gui/src/shell.rs:nav_entry`<br>`crates/raul-gui/src/pages/load_order.rs:LoadOrder::attention` |
| F-MODS-12 | Empty and shared library | differs | Empty says "Your mod list has no Mods yet."; no profile-named banner or shared-library copy. | `crates/raul-gui/src/pages/mods.rs:Mods::ui`<br>`crates/raul-gui/src/pages/mods_install.rs:Panel::ui` |

## Install dialog

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-INSTALL-01 | Pick an archive | differs | Archive-name list from Downloads; no sizes, install-options, Installed tags, Browse or picker cancel. | `crates/raul-gui/src/pages/mods_install.rs:downloads`<br>`crates/raul-gui/src/pages/mods_install.rs:Panel::choosing` |
| F-INSTALL-02 | Install options | differs | One checkbox page with defaults and Recommended tag; no radio groups, preview picture or "You have this mod". | `crates/raul-gui/src/pages/mods_install.rs:Panel::question` |
| F-INSTALL-03 | Review | missing | No Review step: no Version/Plugins/Options/Group dropdown, missing-master warning or load-order note. | `crates/raul-gui/src/pages/mods_install.rs:Panel::asking` |
| F-INSTALL-04 | Install progress | differs | Inline status, no staged progress or Review return; Cancel stops job; no dialog scrim. | `crates/raul-gui/src/pages/mods_install.rs:Panel::running_ui`<br>`crates/raul-gui/src/pages/mods_install.rs:Panel::cancel` |
| F-INSTALL-05 | Broken archive | differs | Failed phase shows error block plus Install/Close; no "Pick another". | `crates/raul-gui/src/pages/mods_install.rs:Panel::choosing`<br>`crates/raul-gui/crates/raul-ui-model/src/pages/mods_install.rs:State::failed` |

## Test your setup

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-TEST-01 | Stability test | missing | No stability test in the GUI; only a CLI scene; no "hitches"/stage strings anywhere. | `crates/raul-gui/crates/raul-ui-model/src/pages/mod.rs:PageId`<br>`crates/raul-cli/src/lib.rs (stability sweep)` |
| F-TEST-02 | Known stable setups | missing | No "Known stable" list; no matching text in any GUI file. | `crates/raul-gui/crates/raul-ui-model/src/pages/mod.rs:PageId` |
| F-TEST-03 | Benchmark | missing | Benchmark exists only in CLI/bench crate; no GUI benchmark page, tiles or chart. | `crates/raul-bench`<br>`crates/raul-cli/src/scene.rs` |
| F-TEST-04 | Location pictures | missing | No location-pictures feature; "pictures" only in tests/snapshots.rs. | `crates/raul-gui/crates/raul-ui-model/src/pages/mod.rs:PageId` |
| F-TEST-05 | Zoom and pan | missing | No zoom/pan code anywhere in the GUI. | `crates/raul-gui/src/pages/mod.rs` |
| F-TEST-06 | Log | missing | No log page or viewer; no "Full log"/"Nothing matches." strings. | `crates/raul-gui/src/shell.rs` |

## Versions

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-VERS-01 | Version history | missing | "Versions" page is About/Update for the Toolbox itself; no change history, timeline or tags. | `crates/raul-gui/src/pages/versions.rs:TEXTS`<br>`crates/raul-ui-model/src/pages/versions.rs` |
| F-VERS-02 | Revert | missing | Only Toolbox-update rollback ("Go back to version"); no Revert/Use-this-version for setup versions. | `crates/raul-gui/src/pages/versions.rs (GO_BACK)` |
| F-VERS-03 | Revert to last stable | missing | Home has readiness rows only; no stable banner or revert-to-stable action. | `crates/raul-gui/src/pages/home.rs:TEXTS` |

## Profiles

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-PROF-01 | Profile menu | missing | Test asserts no screen mentions "profile"; top bar has only brand, game entry, Play. | `crates/raul-gui/src/catalogue.rs:no_screen_says_profile`<br>`crates/raul-gui/src/shell.rs:top_bar` |
| F-PROF-02 | Create, duplicate, rename, delete | missing | No profile create/duplicate/rename/delete UI at all. | `crates/raul-gui/src/catalogue.rs:no_screen_says_profile` |
| F-PROF-03 | Switch | missing | No switching, so no toast or state resets. | `crates/raul-gui/src/catalogue.rs:no_screen_says_profile` |
| F-PROF-04 | Shared and per profile | missing | One internal default profile made by setup; no shared/per-profile model surfaced. | `crates/raul-gui/src/pages/setup.rs:init_and_profile_command` |

## Load order

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-LOAD-01 | Plugin list | differs | "{n} plugins, {a} active" yes; but no locked Base game/TTW sections, grip, owning mod, or Up/Down tags. | `crates/raul-gui/src/pages/load_order.rs:STATE_LISTED,table` |
| F-LOAD-02 | Search and filters | missing | No search or filters on Load order; Mods page has its own search (src/pages/mods.rs:SEARCH_HINT). | `crates/raul-gui/src/pages/load_order.rs` |
| F-LOAD-03 | Missing masters | missing | Field is parsed but never rendered; no red line or badge. | `crates/raul-gui/crates/raul-ui-service/src/conflicts.rs:masters_missing` |
| F-LOAD-04 | Move plugins | missing | No drag, no Alt+↑/↓; plugins move only through the sort preview. | `crates/raul-gui/src/pages/load_order.rs:table` |
| F-LOAD-05 | Sort | differs | Inline preview with moved rows, "Apply N moves", Cancel, "Already sorted"; different notes, no restore-point card. | `crates/raul-gui/src/pages/load_order.rs:tools,step_note` |
| F-LOAD-06 | Page profile switch and untested card | missing | No profile pill (profiles internal) and no untested-change card anywhere. | `crates/raul-gui/crates/raul-ui-model/src/pages/mod.rs:PageId` |

## LOD

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-LOD-01 | States | differs | "Generate" button (not Build/Rebuild LOD); "Made {when}, N files"; no "Render distance" tooltip. | `crates/raul-gui/src/pages/lod.rs:GENERATE,STATE_MOD_MUST,EXISTING_BUILT` |
| F-LOD-02 | Build progress | differs | Six differently named steps, % plus elapsed; no "step N of 5", no minutes left; sidebar attention chip exists. | `crates/raul-gui/src/pages/lod.rs:STEP_*,attention`<br>`crates/raul-gui/src/shell.rs:nav_entry` |
| F-LOD-03 | Stop a build | differs | Stop is immediate, no confirm; old LOD and checkpoint kept, as spec's outcome. | `crates/raul-gui/src/pages/lod.rs:tools,GENERATE_STOPPED` |
| F-LOD-04 | Quality presets | missing | No quality presets, estimates or "X GB · about N min" anywhere. | `crates/raul-gui/src/pages/lod_settings.rs` |
| F-LOD-05 | LOD settings page | differs | Separate page exists with expert fields and Restore defaults (with confirm) and "Back to LOD"; no distance slider, texture presets, tree density or desert-plants switch. | `crates/raul-gui/src/pages/lod_settings.rs:TEXTS,RESTORE` |
| F-LOD-06 | Details fold | missing | Built time and file count shown inline only; no details fold, quality/size, or per-profile LOD. | `crates/raul-gui/src/pages/lod.rs:existing_line` |

## TTW flow

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-TTW-01 | Entry points | differs | Entry via Home row "Set up…" and wizard "Set up TTW…"; no Mods row, Settings "Add…", or "Update TTW…". | `crates/raul-gui/src/pages/home.rs:RowId::Ttw (SET_UP)`<br>`crates/raul-gui/src/pages/setup.rs:665` |
| F-TTW-02 | Step 1 Fallout 3 | differs | Step reads Fallout 3 from Settings ("Change in Settings"); auto-find is for the package, not FO3; no folder picker; 4-step stepper exists. | `crates/raul-gui/src/pages/ttw.rs:sources` |
| F-TTW-03 | Step 2 TTW file | differs | "Open the download page", "Find it for me", Downloads watch, "Use this file"; no Browse…, no "Downloads › file" row. | `crates/raul-gui/src/pages/ttw.rs:package,find` |
| F-TTW-04 | Step 3 Install | differs | Space check blocks install, Cancel and staged phases exist; needs 40 GB not 25, % not minutes, no Free-up/Change-storage buttons. | `crates/raul-gui/src/pages/ttw.rs:space_line,cancel`<br>`crates/raul-ui-model/src/pages/ttw.rs:NEEDED_BYTES` |
| F-TTW-05 | Step 4 Done | differs | Done says "installed as one Mod, plugins enabled"; no per-profile wording, no pinning at top. | `crates/raul-gui/src/pages/ttw.rs:INSTALLED,DONE` |

## System

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-SYSTEM-01 | Check for updates | differs | Manual "Check for updates" button only; no auto-run on open, no setting, throttle, "Checked N ago" or palette command. | `crates/raul-gui/src/pages/versions.rs:CHECK_UPDATES` |
| F-SYSTEM-02 | Parts list | missing | Explicitly shows no Component versions; no six-part parts list. | `crates/raul-gui/src/pages/versions.rs (doc)` |
| F-SYSTEM-03 | Update | differs | Update via Download→Install→Restart with rollback UI; no Cancel and no "server didn't answer" banner. | `crates/raul-gui/src/pages/versions.rs (DOWNLOAD/INSTALL/RESTART/GO_BACK)` |
| F-SYSTEM-04 | Engine builds | missing | No engine-builds list or Use/Remove actions. | `crates/raul-gui/src/pages/versions.rs` |
| F-SYSTEM-05 | Masterlist | missing | Masterlist only mentioned as pinned during sort; no update section. | `crates/raul-gui/src/pages/versions.rs`<br>`crates/raul-gui/src/pages/load_order.rs:SORTING` |

## Doctor

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-DOCTOR-01 | Run checks | missing | `raul doctor` runs hidden in background; DoctorScreen model exists but no page renders it. | `crates/raul-gui/src/pages/home.rs:218`<br>`crates/raul-ui-model/src/doctor.rs:DoctorScreen` |
| F-DOCTOR-02 | Results | missing | Findings model exists unrouted; no groups, fix buttons or "Warns above 230". | `crates/raul-ui-model/src/doctor.rs:DoctorView` |
| F-DOCTOR-03 | Repair engine | missing | Engine is only verified, never repaired; no "Repair engine". | `crates/raul-gui/src/engine_check.rs:outcome` |

## Settings

| Id | Function | State | What differs or is missing | Where in the app |
|---|---|---|---|---|
| F-SETTINGS-01 | Game installations | differs | New Vegas (Required), Fallout 3 (Only for TTW), TTW installer rows exist as text fields saved on leave; no Browse…/Find buttons, no "Add". Since 7728d87 (#702) Settings also has a Nexus Mods section with the API key, which the demo does not show yet. | `crates/raul-gui/src/pages/settings.rs:form,path_field` |
| F-SETTINGS-02 | Toolbox storage | missing | Storage folder row only; no free space, cache or "Clean up…". | `crates/raul-gui/src/pages/settings.rs:SECTION_STORAGE` |
| F-SETTINGS-03 | Game settings for profile | missing | Settings page is paths only; no Look/Resolution/Upscaling/V-sync/Frame-rate settings. | `crates/raul-gui/src/pages/settings.rs:form` |
| F-SETTINGS-04 | Updates setting | missing | No update-check setting exists. | `crates/raul-gui/src/pages/settings.rs` |
| F-SETTINGS-05 | Appearance | missing | No High contrast, Larger text or Reduce motion options. | `crates/raul-gui/src/theme.rs` |
| F-SETTINGS-06 | Section folds | missing | Section labels are static; only LOD settings has an "Advanced settings" disclosure. | `crates/raul-gui/src/shell.rs:section_label` |

