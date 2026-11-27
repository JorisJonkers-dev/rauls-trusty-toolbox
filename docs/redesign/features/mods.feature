# SPDX-License-Identifier: GPL-3.0-only
# Feature: Mods page, Mod list tab (Raul's Toolbox redesign demo, area `mods`)
# Walked by redesign/scripts/mods.js on the Hardcore profile (7 of 15 mods on, TTW on, not tested).

Feature: Mods list

  Background:
    Given the setup is finished and the profile "Hardcore" is active
    And I am on the Mods page with the "Mod list" tab

  # US-MODS-01
  Scenario: Search finds a mod by name or plugin file
    When I type "zzz" in "Search mods"
    Then I see "No mods match “zzz”."
    And I see a "Clear filters" button
    When I type "nmc" in "Search mods"
    Then only "NMC's Texture Pack" is listed
    When I type "WFO.esp" in "Search mods"
    Then only "Wasteland Flora Overhaul" is listed

  # US-MODS-02
  Scenario: Filter chips show only the chosen kind of mod
    Then I see the chips "All 15", "On 7", "Off 8", "Attention 0"
    When I choose "Off 8"
    Then 8 mod rows are listed
    When I choose "Attention 0"
    Then I see "No mods match."
    When I choose "Clear filters"
    Then the filter is "All 15" and the search box is empty

  # US-MODS-03
  Scenario: Built into the engine group opens and links out
    Given the "Built into the engine" group is closed
    When I open "Built into the engine"
    Then I see 9 rows, among them "xNVSE" tagged "Built in" and "JohnnyGuitar NVSE" tagged "In progress"
    When I press the "Original mod: xNVSE" button
    Then I see "xNVSE: the original mod’s page opened in your browser"
    When I press the "Our version of xNVSE" button
    Then I see "xNVSE: our version’s source opened in your browser"

  # US-MODS-04
  Scenario: Mod details list the plugin files a mod brings
    When I open the details of "NMC's Texture Pack"
    Then I see "None · files only"
    When I open the details of "NVAC - New Vegas Anti Crash"
    Then I see "None · an engine plugin"
    When I open the details of "Lonesome Road Fixes"
    Then I see the plugin file "LonesomeRoadFixes.esp"
    And I do not see "Loads after LonesomeRoad.esm, as the masterlist asks."

  # US-MODS-05
  Scenario: Switch TTW off and check for an update
    When I switch "Tale of Two Wastelands" off
    Then I see the toast "TTW off" with an "Undo" button
    When I open the details of "Tale of Two Wastelands"
    Then I see "Off: Plain New Vegas. TTW stays installed."
    When I press "Check for TTW update"
    And I wait for the check to finish
    Then I see "TTW 3.4 is the newest"
    When an update is reported
    Then the TTW row shows "Update available"
    And the details offer "Update TTW…"

  # US-MODS-06
  Scenario: TTW that was never added offers Add
    Given TTW was skipped in setup
    Then the TTW row shows "Not added" and "Add…"
    And there is no TTW switch
    When I press "Add…"
    Then I see the page "Add Tale of Two Wastelands"

  # US-MODS-07
  Scenario: Create a group
    When I press "New group"
    Then the dialog "New group" has a "Create group" button that is disabled
    When I type "Gameplay" in "Name" and press "Create group"
    Then I see "A group with that name exists."
    When I type "Tests group" in "Name" and press Enter
    Then the dialog closes and the group "Tests group" is listed
    When I press "New group", type "Abc" and press "Cancel"
    Then no group "Abc" exists

  # US-MODS-08
  Scenario: Rename a group
    When I open the options of "FIXES" and choose "Rename…"
    And I type "Textures" in "Name" and press "Rename"
    Then I see "A group with that name exists."
    When I type "Core fixes" in "Name" and press "Rename"
    Then I see the group "Core fixes" and the toast "Renamed group to Core fixes"

  # US-MODS-09
  Scenario: Delete a group keeps its mods
    When I open the options of "GAMEPLAY" and choose "Delete group…"
    Then I see "Delete the group “Gameplay”?"
    And I see "Its mods move to Ungrouped. Nothing is uninstalled."
    When I press "Cancel"
    Then the group "GAMEPLAY" is still listed
    When I open the options of "GAMEPLAY", choose "Delete group…" and press "Delete group"
    Then the group "GAMEPLAY" is gone
    And "JSawyer Ultimate Edition" is in "Ungrouped"

  # US-MODS-10
  Scenario: Fold a group and switch all of its mods
    Given the "TEXTURES" group is folded
    When I open "TEXTURES"
    Then I see its 3 mods
    When I fold "TEXTURES"
    Then no mod rows of "TEXTURES" are listed
    When I switch the group "TEXTURES" on
    Then I see "Textures: all on"
    And the group switch reads on

  # US-MODS-11
  Scenario: Move a group up and down
    When I open the options of "FIXES"
    Then "Move up" is disabled
    When I choose "Move down"
    Then "FIXES" is the second group
    When I focus "FIXES" and press Alt+Down
    Then "FIXES" is the second group and "TEXTURES" is first

  # US-MODS-12
  Scenario: Drag a group header to reorder
    When I drag the grip of "FIXES" 120 px down
    Then I see the ghost "Moving group"
    When I release the mouse
    Then "FIXES" is placed after "TEXTURES"

  # US-MODS-13
  Scenario: Escape closes the group options menu
    When I open the options of "GAMEPLAY"
    And I press Escape
    Then the menu is closed

  # US-MODS-14
  Scenario: Switch a mod off and undo it from the toast
    When I switch "Tougher Traders" off
    Then I see the toast "Tougher Traders off" with an "Undo" button
    When I press the toast's "Undo"
    Then I see "Undone: Tougher Traders off"
    And "Tougher Traders" is on

  # US-MODS-15
  Scenario: Open and close a mod's details
    When I press the name "Weapon Retexture Project"
    Then I see "What it does" and "Installed" and "Group"
    When I press the name "Weapon Retexture Project" again
    Then I do not see "What it does"

  # US-MODS-16
  Scenario: Move a mod up with the button and with Alt+Down
    When I open the details of "Ojo Bueno Texture Pack" and press "Up"
    Then "Ojo Bueno Texture Pack" is one row higher
    When I focus the name "Ojo Bueno Texture Pack" and press Alt+Down
    Then it is one row lower again and keeps the focus

  # US-MODS-17
  Scenario: Drag a mod onto another group
    Given the "TEXTURES" group is open and the "LIGHT AND SKY" group is folded
    When I drag the grip of "Weapon Retexture Project" onto the "LIGHT AND SKY" header
    And I release the mouse
    Then "Weapon Retexture Project" is in "LIGHT AND SKY"

  # US-MODS-18
  Scenario: Choose a mod's group in its details
    When I open the details of "Fallout Character Overhaul"
    Then the "Group" list has 5 groups
    When I choose "Gameplay" in the "Group" list
    Then I see the toast "Moved Fallout Character Overhaul to Gameplay"

  # US-MODS-19
  Scenario: Remove a mod from all profiles
    When I open the details of "Tougher Traders" and press "Remove…"
    Then I see "Remove Tougher Traders from all profiles?"
    And I see "Mods are shared by every profile. Its plugins leave every load order. The archive stays in your downloads, and Undo brings it back."
    When I press "Cancel"
    Then "Tougher Traders" is still listed
    When I press "Remove…" and then "Remove from all"
    Then "Tougher Traders" is not listed
    And I see the toast "Removed Tougher Traders" with "Undo"
    When I press "Undo"
    Then "Tougher Traders" is listed again

  # US-MODS-20
  Scenario: A patch whose plugin needs a mod that is off
    When I start installing "Living Desert" and go to its review step
    Then I see the warning "B42 Optics - WRP.esp needs WeaponRetexture.esp"
    And I see "It won’t load until that mod is on."
    When I press "Install" and wait for the install to finish
    Then the "B42 Optics" row shows the red tag "Needs WeaponRetexture.esp"
    When I switch "Weapon Retexture Project" on
    Then the red tag on "B42 Optics" is gone

  # US-MODS-21
  Scenario: The Mods sidebar badge appears and clears
    Given no mod needs attention
    Then the Mods sidebar item has no badge
    When a mod needs attention
    Then the Mods sidebar item shows the badge "1" with the tooltip "1 mod need attention"
    When "Weapon Retexture Project" is switched on
    Then the Mods sidebar item has no badge

  # US-MODS-22
  Scenario: The untested change card can be hidden and comes back
    Then I see "This mod setup hasn't been tested yet." with "Not now", "Revert to last stable" and "Run stability test"
    When I press "Not now"
    Then I see "Hidden until the mod setup changes"
    And the card is gone
    When I switch "NMC's Texture Pack" on
    Then the card is back

  # US-MODS-23
  Scenario: The header Undo reverts the last change
    Given no change has been made yet
    Then there is no "Undo" button in the list header
    When I switch "Tougher Traders" off and press the header "Undo"
    Then I see "Undone: Tougher Traders off"
    When I switch "Tougher Traders" off and press the toast's X
    Then no toast is shown

  # US-MODS-24
  Scenario: An empty profile tells me to install a mod
    Given every mod has been removed from "Hardcore"
    Then I see "No mods in Hardcore yet"
    And I see "Install one from an archive. The engine already has the script extenders built in."
    When I press "Install mod…" in the empty state
    Then the install picker opens

  # US-MODS-25
  Scenario: A new profile is not empty (demo bug)
    When I press "New profile…" and enter "Fresh one"
    Then the dialog says "Starts empty. Use Duplicate to start from this one."
    When I open the Mods page
    Then I see "of 15 on" in the header
    And I do not see "No mods in Fresh one yet"

  # US-MODS-26
  Scenario: Install a mod with options and review it
    When I press "Install mod…"
    Then I see "Archives in your download folder. Pick one."
    And "Next" is disabled
    When I pick "The Living Desert-1.0.zip" and press "Next"
    Then I see "Page 1 of 2"
    When I choose "More" and press "Back"
    Then I see "Install a mod"
    When I pick the same archive, press "Next", choose "More", press "Next"
    Then I see "Page 2 of 2"
    When I press "Back" and "Next" twice
    Then I see the review "Install The Living Desert" with "Options" "Normal · JSawyer Ultimate patch"
    When I press "Install"
    Then I see "Installing The Living Desert" with a "Cancel" button
    And I see the toast "Installed The Living Desert" with "Undo"

  # US-MODS-27
  Scenario: A plain archive skips the options and Browse explains itself
    When I press "Install mod…"
    And I press "Browse…"
    Then the toast "Your file browser opened" waits until the picker closes
    When I press "Cancel"
    Then I see "Your file browser opened"
    When I press "Install mod…" and pick "Weapon Animation Replacers-3.2.zip" and press "Next"
    Then I see "Plugins go to the end of the load order. Sort puts them in place."
    When I press "Back"
    Then I see "Archives in your download folder. Pick one."

  # US-MODS-28
  Scenario: A split download shows the damaged archive message
    When I press "Install mod…", pick "Desert retexture (part 2 of 2).7z.002" and press "Next"
    Then I see "This archive is damaged"
    And I see "It looks like part of a split download. Download the whole file again, then install it."
    When I press "Pick another"
    Then the picker is shown

  # US-MODS-29
  Scenario: Cancel an install in progress
    When I start installing "Weapon Animation Replacers" and the progress is shown
    And I click outside the dialog and press Escape
    Then the dialog is still "Installing Weapon Animation Replacers"
    When I press "Cancel"
    Then I see "Install Weapon Animation Replacers" with "Install" (the review step)
    When I press Escape
    Then the dialog is closed

  # US-MODS-30
  Scenario: Ctrl K finds a mod and jumps to it
    When I press Ctrl K
    Then I see the field "Pages, actions, profiles, mods"
    When I type "wasteland"
    Then I see "Wasteland Flora Overhaul" labelled "Mod"
    When I press Enter
    Then the Mods list shows the row "Wasteland Flora Overhaul" with its details open
    When I press Ctrl K and type "zzzz"
    Then I see "Nothing found."
