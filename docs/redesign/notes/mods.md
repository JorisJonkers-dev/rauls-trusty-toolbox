# Mods page, Mod list tab: observed details (area `mods`)

Source: ../design/demo/src (pages.js PAGES.mods, modList, modDetail, recCard; overlays.js dialogs, install
dialog, toasts, palette; actions.js handlers; core.js change/undo/job/toast; style.css animations).
Behaviour confirmed by clicking in `scripts/mods.js` (Hardcore profile, 1366x768). Items marked
(code) were read from the source and not clicked.

## Page header and tabs
- Title "Mods"; subtitle "<on> of <total> on" (e.g. "7 of 15 on").
- Header chips: "Not tested yet" (warn) when the set is not stable; profile pill "Profile <name>".
- Tabs (segmented): "Mod list", "Test your setup", "Versions". Only "Mod list" is covered here.
- Sidebar item "Mods" sits under PLAY; badge = mods needing attention (see below).

## Untested-change card (recCard)
- Shown on Mods, Home and Load order when the mod setup is not stable and not hidden for this exact setup.
- Title "This mod setup hasn't been tested yet."; text "A 20-minute test checks this exact set and order."
- Buttons: "Not now" (ghost) hides it for this set; toast "Hidden until the mod setup changes" (no Undo).
  "Revert to last stable" (sec, undo icon) shows only when a passed test exists for an earlier set.
  "Run stability test" (primary, play icon).
- While a test runs: text "Stability test running · <n> %" and a "Show test" button instead.
- Orange dashed border (class rec).

## Search, filters, clear
- Search input placeholder "Search mods", type=search, sr-only label "Search mods". Updates as you type (no submit).
- Chips (segmented, label "Filter mods"): "All <n>", "On <n>", "Off <n>", "Attention <n>". Counts are the whole library.
- Empty result: "No mods match “<query>”. " followed by "Clear filters" (ghost, small). With no query and a
  filter: "No mods match." then "Clear filters".
- "Clear filters" sets query to empty and filter to All.
- While search or a filter is active, every group is forced open (groups show only their matches; a group with
  no matches is hidden).
- Matching: mod name or any plugin file name, case-insensitive. Highlight wraps the match (class hl).

## Built into the engine group
- Header: "BUILT INTO THE ENGINE" (uppercase by style), count "9" then an info icon with title
  "These ship inside the engine. Nothing to install.". Click the header to fold/unfold (default folded).
- Rows: lock icon, name, sub text, tag ("Built in" = ok green; "In progress" = info blue), two icon buttons:
  external-link ("Original mod: <name>") and code ("Our version of <name>").
- Those buttons only raise toasts (no page change): "<name>: the original mod’s page opened in your browser" and
  "<name>: our version’s source opened in your browser".
- Built-in rows: xNVSE (Script extender, Built in); JIP LN NVSE (Script functions and fixes, Built in);
  JohnnyGuitar NVSE (Script functions, In progress); ShowOff xNVSE (Script functions, In progress);
  kNVSE (Animation framework, In progress); lStewieAl's Tweaks (Engine tweaks and fixes, In progress);
  Mod Configuration Menu (In-game mod settings, Built in); UI Organizer (Menu layout for mods, Built in);
  TTW NVSE plugin (Script support for TTW, Built in).

## Tale of Two Wastelands row
- Row: pin icon, switch (aria-label "Tale of Two Wastelands on. Off: plain New Vegas, TTW stays installed." or
  "Tale of Two Wastelands off"), TTW icon, name, tag "Bundle" (pin icon), version "3.4", chevron.
- Switch toast: "TTW off" / "TTW on" with Undo (label + event "TTW turned off/on").
- Tag "Update available" (info) when an update is known. Details button: "Check for TTW update" (sec) or
  "Update TTW…" (primary, when an update is known; starts the TTW update flow).
- Details: "What it does" = "Fallout 3 and its DLC inside New Vegas"; "Plugins" = "<n> · always load first";
  "Off" = "Plain New Vegas. TTW stays installed.".
- Check: busy spinner for 1.5 s; toast "TTW 3.4 is the newest", or "TTW 3.4.1 is ready" when an update exists.
- Not added: subtitle "Fallout 3 inside New Vegas", tag "Not added" (plain), button "Add…" (primary, small);
  no switch. "Add…" opens the TTW add flow (page title "Add Tale of Two Wastelands").

## Groups
- Group card header: grip (drag handle, title "Drag to move the group"), name in uppercase (button; click
  folds/unfolds), count, group switch, "…" icon button (title "<name> options", opens the group menu), fold
  chevron (title "Fold <name>" / "Unfold <name>").
- Group switch: aria-label "All mods in <name>". On when all mods on; "mixed" when some on (aria-checked="mixed");
  off when none. Toast "<name>: all on" / "<name>: all off" with Undo. Empty group: switch hidden.
- Group menu (popover, role menu, label "Group options"): "Rename…", "Move up", "Move down", separator,
  "Delete group…" (danger, title "Its mods move to Ungrouped"). Ungrouped has only "Move up" and "Move down".
  "Move up" disabled on the first group; "Move down" disabled on the last one.
  Escape closes the menu; so does a click outside (code).
- Empty group body: "Empty group. Drag mods here, or use Move to group in a mod’s details."
  Ungrouped empty body: "Mods without a group land here."
- Ungrouped (id ug) cannot be renamed or deleted; its menu has only "Move up" and "Move down" (code).
- Group create: New group dialog (see Dialogs). Group is inserted above Ungrouped and opened.
- Group delete: confirm "Delete the group “<name>”?", text "Its mods move to Ungrouped. Nothing is uninstalled.",
  buttons Cancel and "Delete group" (danger). Done: group removed, its mods move to Ungrouped, toast "Deleted group <name>"
  with Undo.
- Group move: toast "Moved group <name>" with Undo. Keyboard: focus the group name and Alt+Up / Alt+Down
  (verified Alt+Down).
- Drag group: pointer down on the grip, move 5 px or more to lift. Ghost (class dghost) with the group name and a
  "Moving group" tag follows the pointer; a drop line appears between groups. Release moves it.
  Auto-scroll of the list when the pointer nears its top or bottom edge (code).

## Mod rows
- Row: grip (title "Drag to move. Keyboard: Alt+Up or Alt+Down on the name."), order number (counts across
  all groups), switch (checkbox role, aria-label "<name> on|off"), name button (aria-expanded), tags at the
  right: "Needs <file>" (err red, when a missing master, shown as the attention reason), "Engine plugin"
  (DLL mods), "Files only" (asset mods), version, chevron.
- Switched-off names are muted grey (style).
- Switch toggles on click; toast "<name> on|off" with Undo (5 s).
- Name click or row click opens the details (rowOpen). Details fold on the next click.
- Details (indented, fades in .18s): "What it does", "Plugins" (monospace file names, one per line; or
  "None · files only" for asset mods; or "None · an engine plugin" for DLL mods), "Options" (only for mods
  installed with options; labels joined " · "), "Size" (MB, or GB from 1000 MB), "Installed" (date), "Group"
  (select of all groups, id mg-<mod>), buttons "Up" (title "Move up (Alt+Up)"), "Down" (title "Move down
  (Alt+Down)"), "Remove…" (danger, small).
- Group select change moves the mod to the end of that group; toast "Moved <mod> to <group>" with Undo.
- Up/Down move one place in the whole list and may change its group to the neighbour's group (code). No toast
  for Up/Down (toast: false); Undo still records it. Focus stays on the moved mod.
- Plugins of a mod are also listed under it when they came from an install option (e.g. B42 Optics - WRP.esp
  is listed under B42 Optics after the install that added it).

## Drag between groups
- Grip drag of a mod: ghost "Moving" tag; the target group card is the one under the pointer; a drop line appears
  at the computed position inside the group. Release: toast "Moved <mod> to <group>" (code) and state change.
- The target group stays as it was (folded groups stay folded after a drop).

## Remove a mod
- "Remove…" confirm: title "Remove <name> from all profiles?"; text "Mods are shared by every profile. Its plugins
  leave every load order. The archive stays in your downloads, and Undo brings it back."; buttons Cancel and
  "Remove from all" (danger). Done: row removed, its plugins removed from every load order, toast
  "Removed <name>" with Undo (restores mod and plugins).

## Missing masters and attention
- A plugin is active when its mod is on; a missing master is a master file that is not among the active files.
- Row tag "Needs <master file>" (err red) on the mod that owns the plugin (for patch plugins, the owning mod is
  the one named in the plugin table, e.g. B42 Optics for "B42 Optics - WRP.esp").
- Install review warning: "<plugin> needs <master>" + "It won’t load until that mod is on." (warn box).
- Sidebar badge on Mods: count of mods with attention (err colour when any master is missing). Tooltip
  "<n> mod need attention" (grammar: "1 mod need attention"). Badge disappears when the count is 0.
- Filter "Attention <n>" lists those mods.
- Load order sidebar badge also counts missing masters (other area).

## Undo
- Header button "Undo" (ghost, undo icon) appears in the list action bar once a change exists. Title "Undo: <label>".
  Click reverts the last change in this profile: toast "Undone: <label>". Up to 30 steps are kept.
- Toasts show "Undo" for changes (5 s). Plain toasts last 3.5 s. At most two toasts are kept; the oldest is dropped.
- Toast X (title "Dismiss") removes one toast.
- Toasts are not drawn while a dialog is open (they wait and show after it closes). Seen with "Browse…".

## Empty profile
- Banner (info): "No mods in <profile> yet" / "Install one from an archive. The engine already has the script
  extenders built in." with "Install mod…" (primary). Shown when the profile has no mods and no filter is active.

## Dialogs
- Shared shape: title (h2), body, buttons right-aligned; scrim (dark, 0.55) closes the dialog on a click outside
  it. Escape closes it. Both are blocked only during an install's busy step.
- New group / Rename group: field label "Name", max 40 characters, autofocus. "Create group" / "Rename"
  (primary) disabled while empty. Duplicate name (any case): error text under the field "A group with that name
  exists.". Enter in the field submits (verified). Cancel closes.
- Remove mod, delete group: confirm dialogs (see above). Confirm buttons are danger-styled.

## Install a mod
- Entry: "Install mod…" in the action bar (primary) and in the empty-state banner.
- Step 1, picker (wide): title "Install a mod"; text "Archives in your download folder. Pick one."
  - Rows: archive icon, file name, size ("54 MB" or "1.2 GB") and "· has install options" when it has options,
    tag "Installed" (ok) and aria-disabled for mods already installed (cannot be picked, opacity 0.55).
  - Buttons: "Browse…" (ghost, folder icon, toast "Your file browser opened"), "Cancel", "Next" (primary,
    disabled until a row is picked).
  - Arrow keys move between rows (roving focus, code).
- Step 2, options (one page per option group): title "<mod name> · install options"; text "Page <n> of <total> ·
  <group title>"; radio list (one-of groups, round radio) or checkbox list (many-of groups); each option has a
  bold label and a description; "You have this mod" tag on patch options whose mod is already installed.
  Defaults: one-of groups pick the first option; many-of groups pre-tick the options whose mod is installed.
  A preview picture with a caption sits under the list. Buttons "Back" (ghost) and "Next" (primary).
- Step 3, review: title "Install <mod name>"; rows Version, Plugins (monospace list; "None · files only" when none),
  Options (labels joined " · " or "Defaults"), Group (select, default = first group that is not Ungrouped).
  Missing-master warning (see above). Text "Plugins go to the end of the load order. Sort puts them in place."
  Buttons "Back" and "Install" (primary, download icon).
  Known demo bug: a patch option of The Living Desert (JSawyer patch) lists "B42 Optics - WRP.esp" instead of
  "The Living Desert - JSUE.esp" (the install code adds a fixed name per patch id; see notes below).
- Step 4, busy: title "Installing <mod name>"; phase text "Unpacking", then "Checking files", then "Adding plugins"
  (by progress); a progress bar; button "Cancel" (ghost). Duration about 2.6 s. Footer shows "Installing · <n> %".
  Cancel returns to the review step (no change made). The scrim and Escape do nothing during this step.
- Done: dialog closes; the mod is added (on, in the chosen group, group opened), its plugins appended; the list
  focuses the new mod row and opens its details (code); toast "Installed <name>" with Undo.
- Error (broken archive): title "This archive is damaged"; banner "<file> can’t be opened" + "It looks like part of a
  split download. Download the whole file again, then install it."; buttons "Pick another" (back to the picker)
  and "Close" (primary).
- Plain archive (no options): Next goes straight to review; Back on review returns to the picker.
- Back on options page 1 returns to the picker; Back on page 2 returns to page 1.

## Command palette (Ctrl K)
- Ctrl K opens (only after setup). Dialog with search field placeholder "Pages, actions, profiles, mods"
  (aria-label "Search or go to"). Field has role combobox.
- Results: up to 9 rows, each with an icon, name, and a kind label on the right: "Page", "Action", "Profile", "Mod".
  Mod entries are the library mods: they match by name only (typing "wfo" finds nothing; "wasteland" finds
  Wasteland Flora Overhaul). Empty result: "Nothing found.".
- Up/Down move the highlight; Enter runs it; click runs it; Escape or a click outside closes (focus back to the
  control that opened it).
- A mod result (findMod) opens the Mods list, clears search and filters, opens that mod's details and focuses it.

## Mods sidebar badge
- Badge appears only while at least one mod needs attention; tooltip "<n> mod need attention".
- Verified: appears after the install with the missing master, disappears when the master mod is turned on.

## Animations (style.css)
- Page change: fade + 6 px rise, .22 s (pgin).
- Dialog: scale .97 to 1 + fade, .18 s (pop); scrim fade .15 s.
- Menus: pop .12 s. Toasts: pop .18 s in; removed at timer end (no exit animation).
- Switch knob slides and recolours .15 s; group fold chevron rotates .18 s.
- Mod details fade in .18 s (pgin). Row background transitions .15 s on hover.
- Progress bar width .35 s; indeterminate slide 1.2 s loop. Busy button and "Checking" icon spin 1 s.
- The row being dragged keeps its place with the class dragsrc (code).
- Reduce-motion setting shortens durations to .01 ms (style.css media query and data-motion="0").

## Keyboard and focus
- Enter on a focused role row activates it (keydown handler dispatches click). Space on a focused control too.
- Alt+Up / Alt+Down on a focused mod name, group name or plugin name moves it one place (not while a sort is on).
- Escape order: menu first, then palette, then dialog (not while busy), then test situations panel, then drag.
- "?" opens the Shortcuts dialog (not while typing or in a dialog).
- Focus is restored after move, toggle and group edits (U.focus keys like "mod:<id>", "grp:<id>").
- Tab order follows DOM order; the list action bar sits last.

## Alignment and layout
- Mod rows: grip, number, switch, name, then tags and version right-aligned; the chevron is the last item.
- Group switch and "…" and fold are right-aligned in the group header, aligned with the chevron column.
- The action bar (Undo, New group, Install mod…) floats bottom right over the list (fixed); it covers the
  bottom-right corner of the list at 1366x768 (seen in the screenshots). The status bar ("Ready") is bottom left.
- Toasts are centred at the bottom (width 440 px), above the status bar.

## Observed demo bugs and oddities (do not copy into the Toolbox)
1. Install patch for The Living Desert (JSawyer patch): the review lists "B42 Optics - WRP.esp" instead of
   "The Living Desert - JSUE.esp". Cause: instPlugins() adds "B42 Optics - WMX.esp" for the 'wmx' patch id and
   "B42 Optics - WRP.esp" for every other patch id, whatever the mod. The file then needs WeaponRetexture.esp.
2. New profile says "Starts empty. Use Duplicate to start from this one." but copies the shared 15-mod library
   (header "x of 15 on"). It is not empty.
3. Plugin notes (fourth field of a plugin entry, e.g. "Loads after LonesomeRoad.esm, as the masterlist asks.",
   "Load near the end: weather records win last.", "LOD: rebuild after adding or removing this mod.") are stored
   but never shown in mod details.
4. Badge tooltip grammar: "1 mod need attention" (singular verb).
5. Palette matches mod names only, not file or folder ids; typing "wfo" finds nothing.
6. Toasts are hidden while any dialog is open and show only after it closes (seen with "Browse…"); the message
   looks lost while the picker is open.
7. Install review pre-selects the patch options of mods already installed (JSawyer patch on The Living Desert
   when JSawyer is installed). Clicking it unticks it; this is by design, but the review does not say why.
