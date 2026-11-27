# SPDX-License-Identifier: GPL-3.0-only
Feature: Mods test tab, Versions and profiles (raul demo)
  Background:
    Given Raul's Toolbox setup is finished
    And the profile "Beautiful" is active on the Mods page

  @US-TESTS-01
  Scenario: Run a stability test and see it pass
    Given the Test your setup tab is open
    Then I see "This exact mod set and order has no passed test yet." or "This exact mod set and order passed a stability test."
    When I click "Run stability test"
    Then I see "Loading cells · 0 of 40" and a "Cancel" button
    And the recommendation banner is hidden
    When the run completes
    Then I see the toast "Stability test passed. This setup is now Tested stable"
    And a row "Passed" with "40 cells" appears in the stability history

  @US-TESTS-02
  Scenario: Cancel a running stability test
    Given a stability test is running
    When I click "Cancel"
    Then I see the toast "Stopped"
    And no stability run is added

  @US-TESTS-03
  Scenario: A stability test crashes
    Given the stability scenario is set to fail
    When I click "Run stability test"
    And the run reaches its failure point
    Then I see the toast "Stability test crashed in Freeside. See the log"
    And a row "Crashed" appears with "in Freeside after 11 min"
    And the log filter is "Errors"

  @US-TESTS-04
  Scenario: Forget a known stable setup
    Given "Known stable setups" is open with one entry
    When I click "Forget" on the entry
    Then I see the toast "Forgotten"
    And "Known stable setups · 0" is shown
    And "None yet. A passed test adds the setup here." is shown

  @US-TESTS-05
  Scenario: Turning a mod off makes the setup untested
    Given the setup is tested stable
    When I turn off a mod
    And I open the Test your setup tab
    Then I see "This mod setup hasn't been tested yet."
    And I see "A 20-minute test checks this exact set and order."
    And I see the buttons "Not now", "Run stability test"

  @US-TESTS-06
  Scenario: Run and cancel a benchmark
    When I click "Run benchmark"
    Then I see "Running the benchmark route"
    When I click "Cancel"
    Then I see the toast "Stopped"
    When I click "Run benchmark" and wait for the run to finish
    Then I see the toast "Benchmark done: 74 fps average"
    And the tiles "Average", "1 % low", "Frame time" are shown

  @US-TESTS-07
  Scenario: Take pictures and compare side by side
    When I click "Take pictures"
    Then I see "Taking pictures · 1 of 29 places"
    When the run completes
    Then I see the toast "New pictures taken"
    And two pictures are shown with captions "A" and "B"
    When I choose "Single"
    Then one picture is shown with the caption "Latest"

  @US-TESTS-08
  Scenario: Search places across worlds
    When I type "freeside" in "Search places"
    Then I see "1 match"
    And the place "Freeside" is shown
    When I click "Fewer" or "All N"
    Then the place list toggles between the strip and all places

  @US-TESTS-09
  Scenario: Zoom and reset a picture
    When I scroll the mouse wheel up over a picture
    Then the zoom is 1.25x
    When I double-click the picture
    Then the zoom is 1x and the pan is centred
    When I click "8x"
    Then the "8x" chip is pressed

  @US-TESTS-10
  Scenario: Filter the log
    When I click "Full log"
    And I choose "Errors" in the "Log filter"
    Then only error lines are shown
    When I type "zzz-none" in "Search the log"
    Then I see "Nothing matches."

  @US-TESTS-11
  Scenario: Revert to an older version
    Given I am on the Versions tab
    When I click "Revert…" on an older version
    Then I see the dialog "Revert to the version from" and the text "Other versions stay in the list."
    When I press Escape
    Then no version changes
    When I click "Revert…" and then "Revert"
    Then I see the toast "Now on the version from"
    And every version is still listed

  @US-TESTS-12
  Scenario: Revert to last stable from the banner
    Given the setup is untested and an older version passed
    When I click "Revert to last stable"
    Then I see the dialog "Revert to the version from"
    When I press Escape
    Then no version changes

  @US-TESTS-13
  Scenario: Home banner runs the test on Mods
    Given the setup is untested
    When I open Home and click "Run stability test"
    Then the Mods page is open on "Test your setup"
    And "Stability test running" is shown on the banner while it runs

  @US-TESTS-14
  Scenario: Create a profile with a name in use
    When I open the "PROFILES" menu and click "New profile…"
    And I type "Beautiful" in "Name"
    And I click "Create"
    Then I see "A profile with that name exists."
    When I type "Test new" in "Name" and click "Create"
    Then I see the toast "Created Test new"

  @US-TESTS-15
  Scenario: Duplicate a profile
    When I open the "PROFILES" menu and click "Duplicate “Beautiful”…"
    And I type "Beautiful copy" in "Name" and click "Duplicate"
    Then I see the toast "Duplicated as Beautiful copy"
    And the mods are copied and the stability history is empty

  @US-TESTS-16
  Scenario: Rename a profile
    When I open the "PROFILES" menu and click "Rename “Beautiful”…"
    And I type "Test renamed" in "Name" and click "Rename"
    Then I see the toast "Renamed"

  @US-TESTS-17
  Scenario: Delete a profile
    When I open the "PROFILES" menu and click "Delete “Beautiful copy”…"
    Then I see the text "Its mod choices, load order, settings and test history go."
    When I click "Delete profile"
    Then I see the toast "Deleted Beautiful copy. Now on Vanilla+"

  @US-TESTS-18
  Scenario: Switch profile from the Load order page
    Given I am on the Load order page
    When I open the profile menu from the page header and click "Switch to Beautiful"
    Then I see the toast "Switched to Beautiful: mods, load order, settings, LOD and tests"
    And the Load order page is still open

  @US-TESTS-19
  Scenario: Installed mods are shared between profiles
    Given the profiles "Vanilla+" and "Beautiful" exist
    Then both profiles list the same installed mods
