# Mods: user stories (area `mods`)

Source: the clickable demo (index.html built from src/). Every story was walked by `scripts/mods.js`
(162 checks, all passing on the last run). Screens are in `redesign/screens/mods/`. Exact texts and
rules are in `redesign/notes/mods.md`; Gherkin is in `redesign/features/mods.feature`.

Profile used: Hardcore (7 of 15 mods on, untested, TTW on). Where a story needs a mod that is off or on,
it says so.

### US-MODS-01 Search the mod list
As a player, I want to type part of a mod's name or plugin file, so that I find it fast in a long list.
Acceptance:
- Typing in "Search mods" narrows the list at once; the match is on the mod name or any plugin file name, ignoring case.
- A search with no hit shows "No mods match “zzz”." and a "Clear filters" button.
- Clearing the box shows all groups again.
Screens: 01-search-no-match, 02-search-match-by-name, 03-search-match-by-plugin
Demo handlers: IN.q, A.clearFilt

### US-MODS-02 Filter the mod list
As a player, I want to show only the on, off or attention mods, so that I can check one kind at a time.
Acceptance:
- Chips read "All 15", "On 7", "Off 8", "Attention 0" (counts update with the profile).
- "Off" shows only off mods; "Attention" with zero hits shows "No mods match." (no quoted search text).
- "Clear filters" resets the filter to All and clears the search.
- Search and filter combine (Nevada + On shows "No mods match “Nevada”.").
Screens: 04-filter-off, 05-filter-attention-empty
Demo handlers: A.filt, A.clearFilt

### US-MODS-03 See the built-in engine group
As a player, I want to see which script extenders and fixes ship inside the engine, so that I do not install them twice.
Acceptance:
- "Built into the engine" group is closed at first, with the count 9 and an info icon ("These ship inside the engine. Nothing to install.").
- Opening it lists 9 rows with tags "Built in" (green) or "In progress" (blue).
- Each row has two icon links: "Original mod: <name>" and "Our version of <name>". They show toasts "<name>: the original mod’s page opened in your browser" and "<name>: our version’s source opened in your browser".
- The search box filters these rows too.
Screens: 06-builtin-open
Demo handlers: A.sec (builtin), A.openLink

### US-MODS-04 See a mod's bundled plugins
As a player, I want to see which plugin files a mod brings, so that I know what goes into the load order.
Acceptance:
- Opening a mod's details lists its plugin files in monospace; a mod with no plugins says "None · files only" (asset mods) or "None · an engine plugin" (DLL mods).
- A mod with two plugins lists both.
- The plugin note stored in the data (e.g. "Loads after LonesomeRoad.esm, as the masterlist asks.") is not shown anywhere on this page (see notes).
Screens: 07-bundled-plugins-details
Demo handlers: A.rowOpen

### US-MODS-05 Switch Tale of Two Wastelands on or off and check for an update
As a player, I want to turn TTW on or off for this profile and check for a new TTW, so that I keep the Fallout 3 content under control.
Acceptance:
- The TTW row shows a switch, the tag "Bundle", and the version "3.4".
- Switching off gives toast "TTW off" with Undo; details say "Off: Plain New Vegas. TTW stays installed."
- "Check for TTW update" shows a spinner for about 1.5 s, then the toast "TTW 3.4 is the newest" (or "TTW 3.4.1 is ready" when an update exists).
- With an update available (scenario panel "TTW update") the row shows the tag "Update available" and the button becomes "Update TTW…".
Screens: 08-ttw-off, 09-ttw-checked, 10-ttw-update-available
Demo handlers: A.ttwToggle, A.ttwCheck, A.scenTtwUpd

### US-MODS-06 Add Tale of Two Wastelands when it is not added yet
As a player who skipped TTW in setup, I want to add it from the Mods list, so that I can use it later.
Acceptance:
- Row shows the name, subtitle "Fallout 3 inside New Vegas", tag "Not added", and "Add…" (primary).
- There is no switch while TTW is not added.
- "Add…" opens the Add Tale of Two Wastelands flow.
Screens: 47-ttw-not-added
Demo handlers: A.ttwStart

### US-MODS-07 Create a group
As a player, I want to make my own group, so that mods sit together by purpose.
Acceptance:
- "New group" opens the dialog "New group" with field "Name" (max 40 characters). "Create group" is disabled while empty.
- A name already used (any case) shows "A group with that name exists." under the field and does not create.
- A valid name creates the group above Ungrouped and opens it.
- Cancel closes without creating.
Screens: 11-new-group-empty, 12-new-group-duplicate, 13-new-group-created
Demo handlers: A.newGroup, A.nameOk, IN.dlgName

### US-MODS-08 Rename a group
As a player, I want to rename a group, so that its name says what it holds.
Acceptance:
- Group options menu item "Rename…" opens "Rename group" with the current name; button "Rename".
- A taken name is refused with the same duplicate message.
- Done: toast "Renamed group to Core fixes" and the header shows the new name.
- Ungrouped has no Rename.
Screens: 14-group-menu, 15-group-renamed
Demo handlers: A.grpMenu, A.grpRename, A.nameOk

### US-MODS-09 Delete a group
As a player, I want to delete a group I no longer need, without losing any mod.
Acceptance:
- Menu item "Delete group…" (danger) opens the confirm "Delete the group “Gameplay”?" with "Its mods move to Ungrouped. Nothing is uninstalled." and buttons Cancel and "Delete group".
- Cancel keeps the group.
- Delete removes the group card; its mods move to Ungrouped (checked in the profile); toast "Deleted group Gameplay" with Undo.
- Ungrouped has no Delete.
Screens: 16-group-delete-confirm, 17-group-deleted
Demo handlers: A.grpDelete, A.doDeleteGroup

### US-MODS-10 Fold a group and switch all its mods
As a player, I want to fold a group away and turn all its mods on or off at once.
Acceptance:
- Clicking the group header (or the chevron) unfolds or folds the group; folded groups show no rows.
- The group switch reads on when all its mods are on, mixed (aria-checked="mixed") when some are on, off when none are on. Toast "Textures: all on" / "Textures: all off".
- Only the Gameplay group is open by default in the Hardcore profile.
Screens: 18-group-unfolded, 19-group-all-on
Demo handlers: A.grpFold, A.grpToggle

### US-MODS-11 Move a group up and down
As a player, I want to move a group up or down the list, from the menu or the keyboard.
Acceptance:
- Menu "Move up" is disabled on the first group; "Move down" is disabled on the last group.
- Move down reorders immediately; toast "Moved group …" with Undo.
- Focus on a group name and Alt+Down moves that group one place down (checked: Fixes moved below Textures).
Screens: 20-group-moved-down
Demo handlers: A.grpMenu, A.grpMove

### US-MODS-12 Reorder groups by dragging
As a player, I want to drag a group header to a new place.
Acceptance:
- Pressing and moving a group grip more than 5 px lifts a ghost labelled "Moving group" and shows a drop line.
- Releasing below another group places it after that group.
Screens: 21-group-drag-ghost
Demo handlers: pointer drag (data-grip="group")

### US-MODS-13 Close the group options menu
As a player, I want the group options menu to close when I change my mind.
Acceptance:
- The menu opens from the "…" button of a group.
- Escape closes it and returns focus to the button.
- A click outside closes it too (from code; not clicked in the walk).
Demo handlers: A.grpMenu, Escape handler

### US-MODS-14 Switch a mod on or off
As a player, I want to switch a mod on or off from its row, and to undo that at once.
Acceptance:
- The row switch is a checkbox labelled "<name> on" or "<name> off".
- Toggling shows toast "<name> off" (or on) with an Undo button for about 5 s.
- Undo shows "Undone: <label>" and restores the state.
- A switched-off mod's name is grey (from code and style).
Screens: 22-mod-off-toast
Demo handlers: A.modToggle, A.undo

### US-MODS-15 Open a mod's details
As a player, I want to open one mod to see what it does, its files, options, size and install date.
Acceptance:
- Clicking the name or the row opens the details; the chevron turns and the name button says aria-expanded.
- Details: "What it does", "Plugins", "Options" (only when installed with options), "Size", "Installed", "Group", and buttons "Up", "Down", "Remove…".
- Clicking the name again folds the details.
Screens: 23-mod-details-open
Demo handlers: A.rowOpen, A.modMove, A.modRemove

### US-MODS-16 Move a mod up or down
As a player, I want to move a mod up or down in the load order of the list, so that I can sort it without dragging.
Acceptance:
- "Up" and "Down" in details move the mod one row (it can change group to the neighbour's group).
- Alt+Up / Alt+Down on the focused name does the same, and focus stays on the mod.
Screens: 24-mod-moved-up
Demo handlers: A.modMove, Alt+arrow handler

### US-MODS-17 Drag a mod into another group
As a player, I want to drag a mod onto a group header to move it there.
Acceptance:
- Dragging the grip of a mod lifts a ghost "Moving" and shows a drop line over the target group.
- Dropping on the target group puts the mod in that group (checked: Light and sky).
Screens: 25-mod-drag-ghost, 26-mod-dropped-in-group
Demo handlers: pointer drag (data-grip="mod"), dropMod

### US-MODS-18 Choose a mod's group in its details
As a player, I want to pick a mod's group from a list in its details.
Acceptance:
- The "Group" select lists all groups in profile order.
- Choosing one moves the mod to the end of that group; toast "Moved Fallout Character Overhaul to Gameplay" with Undo.
Screens: 27-mod-group-select
Demo handlers: IN.modGroup

### US-MODS-19 Remove a mod from all profiles
As a player, I want to remove a mod for good, with a clear warning and a way back.
Acceptance:
- "Remove…" opens "Remove <name> from all profiles?" with the text "Mods are shared by every profile. Its plugins leave every load order. The archive stays in your downloads, and Undo brings it back." and buttons Cancel and "Remove from all".
- Cancel keeps the mod.
- Remove takes the row out; toast "Removed Tougher Traders" with Undo brings it back.
Screens: 28-remove-confirm, 29-mod-removed
Demo handlers: A.modRemove, A.doRemoveMod

### US-MODS-20 See a missing master
As a player, I want to know when a mod's plugin needs another mod that is off, before I install it and while I play.
Acceptance:
- Install review shows a warning banner "<plugin> needs <master>" and "It won’t load until that mod is on."
- After install the mod row shows a red tag "Needs <master>".
- Turning the master mod on removes the tag.
Screens: 30-install-review-missing-master, 31-installed-with-missing-master, 32-missing-master
Demo handlers: A.instNext, A.instGo, A.modToggle

### US-MODS-21 See the Mods badge appear and clear
As a player, I want the sidebar to tell me how many mods need attention, and stop telling me once they are fixed.
Acceptance:
- With no attention needed, the Mods sidebar item has no badge.
- After the missing master case above, a badge appears on Mods with the count "1" (tooltip "1 mod need attention").
- Turning the master on removes the badge.
Screens: 45-badge-appears, 46-badge-cleared
Demo handlers: A.modToggle, nav badges

### US-MODS-22 Respond to an untested change
As a player, I want to be told when the mod setup has not been tested, and to act on that or hide the message.
Acceptance:
- Card title "This mod setup hasn't been tested yet." with "A 20-minute test checks this exact set and order." and buttons "Not now", "Revert to last stable" (when a stable setup exists), and "Run stability test".
- "Not now" hides it with toast "Hidden until the mod setup changes".
- Changing a mod brings the card back.
Screens: 33-untested-card
Demo handlers: A.recDismiss, A.stabGo

### US-MODS-23 Undo a change
As a player, I want to undo my last change from the list header or from the toast.
Acceptance:
- The "Undo" button appears in the list header after the first change, with title "Undo: <label>". It reverts the last change with toast "Undone: <label>".
- The toast's Undo lasts about 5 s; the toast X dismisses it.
- Before any change there is no Undo button.
Screens: 22-mod-off-toast
Demo handlers: A.undo, A.toastX

### US-MODS-24 Empty profile
As a player with no mods in a profile, I want a clear starting point.
Acceptance:
- With every mod removed, the list shows "No mods in Hardcore yet", "Install one from an archive. The engine already has the script extenders built in." and an "Install mod…" button.
- That button opens the install picker.
Screens: 34-empty-profile
Demo handlers: A.modRemove, A.install

### US-MODS-25 New profile contents
As a player, I want a new profile to start empty, as the dialog says.
Acceptance:
- The New profile dialog says "Starts empty. Use Duplicate to start from this one."
- Observed: the new profile lists the 15 library mods (header "x of 15 on"), not an empty list. See notes; this is a demo bug.
Screens: 48-new-profile-not-empty
Demo handlers: A.profNew

### US-MODS-26 Install a mod with install options
As a player, I want to step through a mod's install options and review them before installing.
Acceptance:
- Picker: "Archives in your download folder. Pick one." Next is disabled until a pick. Installed archives cannot be picked (aria-disabled, tag "Installed").
- Options pages show "Page 1 of 2 · <group>", radio choices for one-of groups and checkboxes for many-of groups, and a preview with a caption.
- Back and Next move between pages; Back on page 1 returns to the picker.
- Review: "Install <name>", Version, Plugins, Options, Group select (default first group), then "Install".
- Progress: "Installing <name>", phases "Unpacking", "Checking files", "Adding plugins", a bar, and Cancel.
- Done: toast "Installed <name>" with Undo; the new row opens.
Screens: 35-install-pick, 36-install-options-1, 37-install-options-2, 38-install-review, 39-install-progress, 40-install-done
Demo handlers: A.install, A.archSel, A.instNext, A.instBack, A.optPick, A.instGo

### US-MODS-27 Install a plain archive, browse, or cancel the picker
As a player, I want an archive without options to go straight to review, and to close the picker.
Acceptance:
- An archive with no options goes straight to review ("Plugins go to the end of the load order. Sort puts them in place.").
- Back on review of a plain archive returns to the picker.
- "Browse…" shows toast "Your file browser opened"; the toast is queued while the picker is open and appears after it closes.
- Cancel closes the picker (tested); Escape does too (from code).
Demo handlers: A.install, A.instNext, A.openLink, A.dlgClose

### US-MODS-28 See a broken archive
As a player with a split download, I want a clear message and a way to pick another file.
Acceptance:
- Title "This archive is damaged"; banner "<file> can’t be opened" and "It looks like part of a split download. Download the whole file again, then install it."
- "Pick another" returns to the picker; "Close" closes.
Screens: 41-broken-archive
Demo handlers: A.instNext (broken), A.instBack ('pick')

### US-MODS-29 Cancel an install in progress
As a player, I want to stop an install I changed my mind about.
Acceptance:
- While "Installing …" runs, the scrim click and Escape do not close the dialog.
- Cancel stops the job and returns to the review step; the mod is not added.
- Escape then closes the review step.
Screens: 42-install-cancelled-review
Demo handlers: A.instCancel, A.scrim

### US-MODS-30 Find a mod with Ctrl K and jump to it
As a player, I want to press Ctrl K, type a mod's name, and land on that mod.
Acceptance:
- Ctrl K opens "Search or go to…" (placeholder "Pages, actions, profiles, mods").
- Typing "wasteland" lists "Wasteland Flora Overhaul" with the kind label "Mod".
- Enter opens the Mods list with search and filters cleared, the mod's row open.
- "Nothing found." for no match. Note: matching is on the name, not the file id (see notes).
Screens: 43-palette-mod, 44-palette-jumped
Demo handlers: A.palette, A.palGo, A.findMod
