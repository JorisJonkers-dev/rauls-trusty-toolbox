# Area: pages (Load order, LOD, TTW flow, System, Doctor, Settings)
# Source stories: redesign/stories/pages.md. Exact UI text in quotes.
# Background is the demo state after first-run setup: profile "Hardcore", TTW on but not added.

Feature: Load order
  Background:
    Given Raul's Toolbox is set up with the profile "Hardcore"
    And I am on the "Load order" page

  @US-PAGES-01
  Scenario: See the load order status and what needs fixing
    Then the header status reads "17 plugins · 17 active · 4 out of order"
    And the "Load order" sidebar badge is 5 with the title "4 out of order, untested change"
    And the sections "BASE GAME" and "YOUR PLUGINS" are shown
    And the row "WMX-DLCMerged.esp" is tagged "Out of order"

  @US-PAGES-02
  Scenario: Search plugins
    When I type "wmx" into "Search plugins"
    Then only plugins matching "wmx" by file or mod name are listed
    When I type "zzz" into "Search plugins"
    Then I see "No plugins match."
    When I clear "Search plugins"
    Then every plugin is listed again

  @US-PAGES-03
  Scenario: Show only out-of-order plugins
    When I choose the filter "Out of order 4"
    Then only the 4 out-of-order plugins are listed
    And the filter "All 7" is available again when I choose it

  @US-PAGES-04
  Scenario: Turn a plugin off and undo it
    When I click the checkbox "Tougher Traders.esp active"
    Then the checkbox reads "Tougher Traders.esp inactive"
    And a toast reads "Tougher Traders.esp inactive" with an "Undo" action
    When I click "Undo" in the action bar
    Then the toast reads "Undone: Tougher Traders.esp inactive"
    And "Tougher Traders.esp" is active again

  @US-PAGES-05
  Scenario: Preview a sort and cancel it
    When I click "Sort…"
    Then the action bar shows "Cancel" and "Apply 4 moves"
    And the card "The 4 moves" lists each moved plugin with "Up N" or "Down N"
    And I see "A restore point is made first, and Undo stays after."
    When I click "Cancel"
    Then the preview is gone and no toast is shown

  @US-PAGES-06
  Scenario: Apply a sort
    Given I have previewed the sort
    When I click "Apply 4 moves"
    Then a toast reads "Sorted: 4 moves"
    And the header status reads "17 plugins · 17 active · sorted"
    And the "Load order" sidebar badge is 1

  @US-PAGES-07
  Scenario: Move a plugin with Alt+Down
    When I focus the plugin "JSawyer Ultimate.esm"
    And I press "Alt+ArrowDown"
    Then "JSawyer Ultimate.esm" has moved one place down

  @US-PAGES-08
  Scenario: Switch profile from the profile pill
    When I click the "Profile" pill
    Then the menu "Profiles" lists the profiles with "Beautiful" marked "not tested" when untested
    And the menu offers "New profile…", "Duplicate “Hardcore”…", "Rename “Hardcore”…" and "Delete “Hardcore”…"
    When I choose "Beautiful"
    Then a toast reads "Switched to Beautiful: mods, load order, settings, LOD and tests"
    When I open the profile menu and press "Escape"
    Then the menu closes and the profile is still "Beautiful"

Feature: LOD
  Background:
    Given Raul's Toolbox is set up with the profile "Hardcore"
    And I am on the "LOD" page

  @US-PAGES-09
  Scenario: See whether LOD is built and current
    Then the status reads "Up to date" with "Matches your 7 enabled mods."
    And there is no LOD sidebar badge
    When I switch to the profile "Beautiful" and open LOD
    Then the status reads "Not built" with "Far away looks flat until you build it."
    And the button "Build LOD" is shown
    And the LOD sidebar badge is shown

  @US-PAGES-10
  Scenario: Build LOD with progress
    Given the profile "Beautiful" has no LOD
    When I click "Build LOD"
    Then the status shows "Building · NN %" and "Reading your mods", "Terrain", "Objects", "Trees" or "Packing" with "step N of 5"
    And the action bar shows "Cancel…"
    When the build finishes
    Then a toast reads "LOD for Beautiful is ready"
    And the status reads "Up to date"

  @US-PAGES-11
  Scenario: Stop a LOD build after confirming
    Given a LOD build is running
    When I click "Cancel…"
    Then a dialog titled "Stop the LOD build?" shows "The LOD you had stays in place."
    When I click "Cancel" in the dialog
    Then the build is still running
    When I click "Cancel…" and then "Stop build"
    Then a toast reads "Build stopped"

  @US-PAGES-12
  Scenario: LOD goes out of date after a quality or mod change
    Given LOD is up to date
    When I choose the quality "High"
    Then the status reads "Out of date" with "Quality changed since the last build."
    And the LOD sidebar badge reads "LOD out of date"
    When I change a mod that is on
    And I open LOD
    Then the status reads "Out of date" with "Mods changed since the last build."

  @US-PAGES-13
  Scenario: Choose a quality preset
    When I choose the quality "Performance"
    Then the help text reads "Smaller textures, nearer distance, sparse trees."
    When I choose the quality "Balanced"
    Then the help text reads "Matches the game’s own LOD."
    When I choose the quality "High"
    Then the help text reads "Sharper textures, farther, denser trees."

  @US-PAGES-14
  Scenario: Fine-tune each control and reset
    When I click "Fine-tune"
    Then the page "LOD settings" is shown with "Texture size", "Distance", "Tree density" and "Far desert plants"
    When I choose texture size "1024" and tree density "Sparse"
    And I set "Distance" to 4
    Then the distance label reads "Far"
    And LOD shows "Custom" when I return with "Done"
    When I click "Restore defaults"
    Then "Restore defaults" is disabled

  @US-PAGES-15
  Scenario: Open LOD details
    When I click the "Details" header
    Then I see "Profile", "Built", "Quality", "Size", "Worlds", "Mods in it"
    And "Worlds" reads "Mojave Wasteland and the DLC worlds"

Feature: TTW flow
  Background:
    Given Raul's Toolbox is set up with the profile "Hardcore" and Tale of Two Wastelands is not added
    And I am on the "Mods" page

  @US-PAGES-16
  Scenario: Start TTW from Mods and see Fallout 3 found
    Then the Mods card shows "Tale of Two Wastelands" with "Not added" and "Add…"
    When I click "Add…"
    Then the page "Add Tale of Two Wastelands" shows "Step 1 of 4"
    And the step "Fallout 3" reads "Found"
    And "Next: TTW file" is enabled

  @US-PAGES-17
  Scenario: Fallout 3 not found, then Find and Browse
    Given Fallout 3 is not found
    When I click "Add…"
    Then "Fallout 3" reads "Not found" with "No Fallout 3 folder found"
    And "Next: TTW file" is disabled
    When I click "Find"
    Then a toast reads "Fallout 3 not found. Try Browse"
    When I click "Browse…" and choose a folder that is not Fallout 3 and click "Use this folder"
    Then a toast reads "No Fallout 3 in that folder"
    When I click "Browse…" and choose the Steam library and click "Use this folder"
    Then a toast reads "Fallout 3 folder saved"
    And "Next: TTW file" is enabled

  @US-PAGES-18
  Scenario: TTW installer file and download page
    Given I am on step "TTW file"
    Then I see "TTW installer file" with "Downloads › TTW_3.4.0.mpi"
    When I click "Get the file"
    Then a toast reads "The TTW download page opened in your browser"
    When I click "Browse…"
    Then a toast reads "Your file browser opened"
    When I click "Back"
    Then the page shows "Step 1 of 4"

  @US-PAGES-19
  Scenario: Install is blocked on low disk space
    Given I am on step "Install" and free space is under 20 GB
    Then I see "25 GB, only 11 GB free"
    And I see "Not enough space"
    And "Install" is disabled
    And the buttons "Free up space…" and "Change storage" are shown

  @US-PAGES-20
  Scenario: Install TTW with progress, cancel and finish
    Given I am on step "Install"
    When I click "Install"
    Then the progress card shows a stage and "NN % · about N min left"
    When I click "Cancel"
    Then a toast reads "Stopped"
    When I click "Install" and wait for it to finish
    Then the banner reads "Tale of Two Wastelands 3.4 is in your mods"
    And a toast reads "Tale of Two Wastelands added"
    When I click "Done"
    Then I am on the "Mods" page

Feature: System
  Background:
    Given Raul's Toolbox is set up with the profile "Hardcore"
    And automatic update checks are on

  @US-PAGES-21
  Scenario: Check for updates when System opens
    When I open "System"
    Then the header reads "Checking for updates…" with an indeterminate bar
    When the check finishes
    Then the header reads "Update available" followed by "Checked just now"
    And the "System" sidebar badge shows an up arrow

  @US-PAGES-22
  Scenario: See what an update changes, per part
    Given an update is available
    Then the banner reads "Updates Raul's Toolbox 0.5.0 → 0.6.0, including nvmod 0.18.0, loot-rs 0.10.0, ttwright 0.6.0"
    And the part "nvmod" shows "0.18.0 · will be updated"
    When I click the part "Raul's Toolbox"
    Then I see the version "0.6.0" with the tag "With the update" and the tag "Feature"

  @US-PAGES-23
  Scenario: Update Raul's Toolbox with progress
    Given an update is available
    When I click "Update"
    Then the banner reads "Updating to 0.6.0 · NN %" with a stage
    And the action bar shows "Cancel"
    When the update finishes
    Then a toast reads "Updated to Raul's Toolbox 0.6.0"
    And the banner reads "Updated to 0.6.0"

  @US-PAGES-24
  Scenario: Update fails, then retry works
    Given the download fails
    When I click "Update"
    Then the alert reads "The download stopped" with "The server didn't answer. Your current version is untouched."
    And the action bar shows "Try again"
    When the download works and I click "Try again"
    Then the banner reads "Updated to 0.6.0"

  @US-PAGES-25
  Scenario: Use or remove an engine build, with confirm
    When I open "Engine builds"
    And I click "Use this build…" for "Engine 0.23.0"
    Then the dialog "Use engine 0.23.0?" reads "The game runs on 0.23.0 from the next start. You can switch back here."
    When I click "Use 0.23.0"
    Then a toast reads "Engine 0.23.0 in use"
    When I click "Remove…" for "Engine 0.23.0"
    Then the dialog "Remove engine 0.23.0?" reads "You can’t switch back to it after this."
    When I click "Remove"
    Then a toast reads "Engine 0.23.0 removed"

  @US-PAGES-26
  Scenario: Update the masterlist
    When I click "Update masterlist"
    Then the header reads "Updating…" and a progress bar is shown
    When the update finishes
    Then a toast reads "Masterlist updated: 4 new messages"

Feature: Doctor
  Background:
    Given Raul's Toolbox is set up with the profile "Hardcore"

  @US-PAGES-27
  Scenario: Run the checks and read the results
    Given I am on "Doctor" and nothing has run
    Then the header reads "Not run yet" and the button is "Run all checks"
    When I click "Run all checks"
    Then the header reads "Checking N of 12" and the button is "Cancel"
    When the checks finish
    Then the header shows "2 warnings" and "Ran just now · Hardcore"
    And the button reads "Run all checks again"
    When I click the group "Game and storage"
    Then its rows are hidden

  @US-PAGES-28
  Scenario: Repair a damaged engine from Doctor
    Given the engine files are damaged
    When I run all checks on Doctor
    Then the row "Engine" reads "Damaged" with "Repair engine"
    When I click "Repair engine"
    Then a toast reads "Engine repaired"
    And "Engine" reads "0.24.0 · OK"

Feature: Settings
  Background:
    Given Raul's Toolbox is set up with the profile "Hardcore"
    And I am on the "Settings" page

  @US-PAGES-29
  Scenario: Game settings for this profile
    When I choose "2560 × 1440" for "Resolution"
    Then a toast reads "Saved for Hardcore"
    When I choose "144" for "Frame rate limit"
    And I choose "FSR" for "Upscaling"
    And I choose "Modern" for "Look"
    And I switch "V-sync" on
    Then the changes are kept for "Hardcore"

  @US-PAGES-30
  Scenario: Storage folder and download cache
    Then "Storage folder" shows "Mods, LOD and backups live here" with "412 GB free"
    When I click "Clean up…" under "Download cache"
    Then the dialog "Clean up the download cache?" offers "Clean up"
    When I click "Clean up"
    Then a toast reads "Freed 3.2 GB"

  @US-PAGES-31
  Scenario: Appearance switches apply at once
    When I switch on "Reduce motion"
    And I switch on "High contrast"
    And I switch on "Larger text"
    Then each switch is on

  @US-PAGES-32
  Scenario: Updates setting, TTW row and start setup again
    Then "Check for updates when System opens" says "At most once every 5 minutes."
    And the "Tale of Two Wastelands" row reads "Not added" with "Add…"
    When I click "Add…" on that row
    Then the TTW flow opens at "Step 1 of 4"
    When I click "Start setup again…"
    Then the dialog "Start setup again?" is shown
    When I click "Cancel"
    Then nothing is reset
