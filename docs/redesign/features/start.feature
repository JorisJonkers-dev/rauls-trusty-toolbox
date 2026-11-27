# SPDX-License-Identifier: GPL-3.0-only
# Start area: setup wizard, Home, Play, top bar, Start setup again. Exact UI text from the clickable demo.

Feature: Start: setup wizard, Home and Play

  Background:
    Given the demo is open with its state cleared

  @US-START-01
  Scenario: First launch shows the welcome step
    Given setup has not been finished
    Then the header shows "Setup · step 1 of 8"
    And the page shows "Let's get New Vegas ready."
    And the step bar lists "Welcome", "Game", "Storage", "Engine", "TTW", "Profile", "LOD", "Done"
    And the only action is "Start"

  @US-START-02
  Scenario: Game folder is found automatically
    Given I press "Start"
    Then the Game row shows "Looking…" while searching
    And then the Game row shows "Found"
    And the footer line shows "Version 1.4.0.525 · all DLC found"
    And the "Next" button is enabled

  @US-START-02
  Scenario: Game folder is not found and a wrong folder is rejected
    Given the test situation "Game folder missing" is on
    When I press "Start"
    Then the banner reads "Fallout: New Vegas was not found"
    And the banner text reads "Pick the folder that holds FalloutNV.exe, usually in your Steam or GOG library."
    And the "Next" button is disabled
    When I press "Browse…" and pick "Downloads"
    And I press "Use this folder"
    Then the banner reads "That folder has no FalloutNV.exe"
    And the "Next" button is disabled
    When I press "Browse…" and pick "Steam library"
    And I press "Use this folder"
    Then the Game row shows "Found"

  @US-START-02
  Scenario: Cancel and Escape close the folder picker without a change
    Given I am on the Game step with the game not found
    When I press "Browse…"
    And I press "Cancel"
    Then the dialog "Pick the game folder" is closed
    When I press "Browse…"
    And I press Escape
    Then the dialog "Pick the game folder" is closed

  @US-START-03
  Scenario: Default storage folder has enough space
    Given I am on the Storage step
    Then the Storage row shows "D:\RaulsToolbox" and "142 GB free"
    And the footer line shows "Mods live here, never in the game folder."
    And no "Low on space" banner is shown

  @US-START-03
  Scenario: Low storage warns but does not block
    Given I am on the Storage step
    When I press "Browse…" and pick "USB stick"
    And I press "Use this folder"
    Then the banner reads "Low on space"
    And the banner text reads "You can go on; TTW and a High LOD may not fit."
    And the "Next" button is enabled

  @US-START-04
  Scenario: Engine installs with progress
    Given I am on the Engine step
    Then the Engine row reads "Not installed"
    When I press "Install engine"
    Then the progress card shows "Downloading"
    And "Next" is disabled
    When the install finishes
    Then the Engine row reads "Installed"
    And the "Next" button is enabled

  @US-START-04
  Scenario: Engine download fails and can be retried
    Given the test situation "Downloads fail" is on
    And I am on the Engine step
    When I press "Install engine"
    And the download reaches 40 %
    Then the Engine row reads "Download failed"
    And the banner reads "The download stopped"
    And the button "Try again" is shown
    When I turn "Downloads fail" off and press "Try again"
    Then the Engine row reads "Installed"

  @US-START-05
  Scenario: Add Tale of Two Wastelands during setup
    Given I am on the TTW step with Fallout 3 found
    Then "Next" is disabled
    When I press "Add TTW"
    Then the progress card shows "Checking the Fallout 3 files"
    When the install finishes
    Then the banner reads "TTW 3.4 added"
    And the banner text reads "It shows in Mods, pinned at the top."
    And the "Next" button is enabled

  @US-START-05
  Scenario: Skip TTW and undo the skip
    Given I am on the TTW step with Fallout 3 found
    When I press "Skip for now"
    Then the banner reads "Skipped"
    And the banner text reads "Mods shows TTW as Not added, with an Add button."
    When I press "Add TTW instead"
    Then the "Skip for now" button is shown again

  @US-START-05
  Scenario: Add TTW is disabled until Fallout 3 is found
    Given the test situation "Fallout 3 not found" is on
    And I am on the TTW step
    Then the Fallout 3 row reads "Not found"
    And the "Add TTW" button is disabled

  @US-START-06
  Scenario: Four starting profiles with Beautiful selected by default
    Given I am on the Profile step
    Then the profile cards are "Vanilla+", "Beautiful", "Hardcore" and "Empty"
    And "Beautiful" has aria-checked "true"
    And "Hardcore" shows "uses TTW"

  @US-START-07
  Scenario: Build the first LOD now
    Given I am on the LOD step with profile "Hardcore"
    Then the quality segment shows "Performance" selected
    When I press "Build now"
    Then the progress card shows "Reading your mods"
    When the build finishes
    Then the status reads "Built"

  @US-START-07
  Scenario: Leave the LOD for later and undo
    Given I am on the LOD step
    When I press "Later"
    Then the status reads "Later · shows as Not built"
    And the "Build now instead" button is shown
    And "Next" is enabled

  @US-START-08
  Scenario: Finish setup into Home
    Given I am on the Done step
    Then the page shows "You're set."
    And the summary rows include "Profile" and "LOD"
    When I press "Go to Home"
    Then the Home page is shown
    And the toast reads "Welcome in. Hardcore is your profile"

  @US-START-09
  Scenario: Reload keeps the wizard step
    Given I am on the Game step
    When I reload the page
    Then the header shows "Setup · step 2 of 8"

  @US-START-09
  Scenario: Reload during an engine install drops the running job
    Given I am on the Engine step and the install is running
    When I reload the page
    Then the header shows "Setup · step 4 of 8"
    And the Engine row reads "Not installed"

  @US-START-10
  Scenario: Not now hides the stability banner until the setup changes
    Given Home shows "This mod setup hasn't been tested yet."
    When I press "Not now"
    Then the banner is hidden
    When I reload the page
    Then the banner is still hidden

  @US-START-10
  Scenario: Run stability test from the banner
    Given Home shows the banner with "Run stability test"
    When I press "Run stability test"
    Then the Mods page is shown on the "Test your setup" view
    When I go to Home
    Then the banner reads "Stability test running · NN %"
    When the test passes
    Then the toast reads "Stability test passed. This setup is now Tested stable"
    And the banner is hidden

  @US-START-10
  Scenario: Revert to last stable asks first
    Given Home shows the banner with "Revert to last stable"
    When I press "Revert to last stable"
    Then the dialog starts with "Revert to the version from"
    And the dialog has buttons "Cancel" and "Revert"

  @US-START-11
  Scenario: Setup rows open their pages
    Given Home is shown for profile "Hardcore"
    Then the SETUP rows are "Fallout: New Vegas", "Tale of Two Wastelands", "Mods", "Load order" and "LOD"
    When I press the row "Tale of Two Wastelands"
    Then the Mods page is shown
    When I press the row "Load order"
    Then the Load order page is shown
    When I press the row "LOD"
    Then the LOD page is shown

  @US-START-12
  Scenario: Home after skipping TTW and LOD
    Given I skipped TTW, chose profile "Empty" and left LOD for later
    When I press "Go to Home"
    Then the status reads "Ready, with warnings"
    And the SETUP row "Tale of Two Wastelands" reads "Not added"
    And the SETUP row "LOD" reads "Not built"
    And Last played reads "Not yet"

  @US-START-13
  Scenario: Play checks eight things and waits on warnings
    Given Home is shown for profile "Hardcore"
    When I press "Play"
    Then the top button reads "Checking n/8" and is disabled
    And the page reads "Checking your setup"
    When the checks finish
    Then the card reads "Before you play: 4 warnings"
    And the card has buttons "Cancel" and "Play anyway"

  @US-START-13
  Scenario: Cancel stops the checks
    Given Play is checking
    When I press "Cancel"
    Then Home is shown normally
    And the page does not show "Checking your setup"

  @US-START-14
  Scenario: A damaged engine stops Play with a repair button
    Given the test situation "Engine files damaged" is on
    When I press "Play"
    Then the card reads "Play stopped: 1 check failed"
    And the row "Engine" shows "3 engine files changed"
    When I press "Repair engine"
    Then the toast reads "Engine repaired"

  @US-START-14
  Scenario: A missing game opens the folder picker from Play
    Given the test situation "Game folder missing" is on
    When I press "Play"
    Then the row "Fallout: New Vegas" shows "Game folder not found"
    When I press "Find the game…"
    Then the dialog "Pick the game folder" is shown

  @US-START-15
  Scenario: Play anyway, then Stop records playtime
    Given Play shows its warnings
    When I press "Play anyway"
    Then the top button reads "Stop · m"
    And the footer reads "Playing · Hardcore · m min"
    When I press "Stop · m"
    Then the toast starts with "Played"
    And Last played reads "Today · …"

  @US-START-16
  Scenario: Profile menu and switching from the top bar
    Given Home is shown
    When I press the "Profile" chip
    Then the menu has the heading "Profiles"
    And the menu offers "New profile…", "Duplicate", "Rename" and "Delete"
    When I press "Beautiful"
    Then the toast starts with "Switched to Beautiful"
    And Home shows "Tested stable"

  @US-START-16
  Scenario: Search field opens and closes
    Given Home is shown
    When I press "Search or go to…"
    Then the search dialog is shown
    When I press Escape
    Then the search dialog is closed

  @US-START-17
  Scenario: Start setup again needs confirmation
    Given setup is finished
    When I open Settings and expand "Advanced"
    And I press "Start setup again…"
    Then the dialog reads "Start setup again?"
    When I press "Cancel"
    Then the setup is unchanged
    When I press "Start setup again…" and confirm "Start setup again"
    Then the page shows "Let's get New Vegas ready."
