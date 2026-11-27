# SPDX-License-Identifier: GPL-3.0-only
# Shell behaviour of Raul's Toolbox demo (area: shell). One scenario per story in stories/shell.md.
# Window: Raul's Toolbox, 1280x720 design size. Setup finished, profile "Beautiful".

Feature: Shell keyboard, focus, dialogs, toasts and window behaviour

  @US-SHELL-01
  Scenario: Walk every page by keyboard
    Given the app is open with setup finished
    When I press Tab until the "Home" nav item is focused
    And I press Enter
    Then the page heading area is "Home"
    When I press Tab until the "Mods" nav item is focused
    And I press Enter
    Then the page heading area is "Mods"
    When I press Tab until the "Load order" nav item is focused
    And I press Enter
    Then the page heading area is "Load order"
    When I press Tab until the "LOD" nav item is focused
    And I press Enter
    Then the page heading area is "LOD"
    When I press Tab until the "System" nav item is focused
    And I press Enter
    Then the page heading area is "System"
    When I press Tab until the "Doctor" nav item is focused
    And I press Enter
    Then the page heading area is "Doctor"
    When I press Tab until the "Settings" nav item is focused
    And I press Enter
    Then the page heading area is "Settings"

  @US-SHELL-02
  Scenario: Search or go to with Ctrl K
    Given the app is open with setup finished
    When I press Ctrl K
    Then a dialog labelled "Search or go to" is open
    And the search field with placeholder "Pages, actions, profiles, mods" is focused
    When I type "mod"
    Then the list shows "Mods Page", "Install mod… Action" and "Weapon Mods Expanded Mod"
    When I press ArrowDown and then Enter
    Then the dialog "Install a mod" is open
    And the search dialog is closed
    When I press Ctrl K and then Escape
    Then no search dialog is open

  @US-SHELL-03
  Scenario: Shortcuts sheet
    Given the app is open with setup finished
    When I press "?" with nothing focused
    Then a dialog titled "Shortcuts" is open
    And it lists "Ctrl K", "?", "Alt ↑ / Alt ↓", "Space", "Enter" and "Esc"
    And the focused button is "Close"
    When I press Enter
    Then no dialog is open
    When I click the footer button "Shortcuts"
    And I press Escape
    Then no dialog is open

  @US-SHELL-04
  Scenario: Quit confirm from the window close button
    Given the app is open with setup finished
    When I click the window button "Close"
    Then a dialog titled "Quit Raul’s Toolbox?" is open
    And it says "Running jobs stop. Your setup is saved."
    And the buttons are "Cancel" and "Quit"
    And the focused button is "Quit"
    When I press Escape
    Then no dialog is open
    When I click the window button "Close" and then press Enter
    Then no dialog is open
    And the window is still open

  @US-SHELL-05
  Scenario: Dialogs sit inside the window and close on scrim click
    Given the app is open with setup finished
    When I click the window button "Close"
    Then the dialog is inside the window
    When I click the window corner outside the dialog
    Then no dialog is open
    When I open "New profile" from the Profile menu
    Then the name field is focused
    When I press Enter with the name empty
    Then the dialog "New profile" is still open
    When I type "Scratch test" and press Enter
    Then no dialog is open
    And a profile named "Scratch test" exists

  @US-SHELL-06
  Scenario: Profile menu with arrows, End, Escape and Tab
    Given the app is open with setup finished
    When I click "Profile"
    Then a menu is open inside the window
    And the first item "Vanilla+" is focused
    When I press ArrowDown
    Then "Beautiful" is focused
    When I press End
    Then "Delete “Beautiful”…" is focused
    When I press Escape
    Then no menu is open
    And the "Profile" button is focused
    When I click "Profile" and then press Tab
    Then no menu is open

  @US-SHELL-07
  Scenario: Toast with Undo, stacking and timing
    Given the app is open with setup finished on the Mods page
    When I switch off "NMC's Texture Pack"
    Then a toast reads "NMC's Texture Pack off" with buttons "Undo" and "Dismiss"
    And the toast container is 440 px wide and 40 px above the window bottom
    When I click "Undo"
    Then the toast reads "Undone: NMC's Texture Pack on"
    When I switch off three mods in a row
    Then at most 2 toasts are shown
    And an undo toast disappears after about 5 seconds
    When I click "Dismiss" on a toast
    Then that toast is gone at once
    When I switch off a mod and then open the quit confirm
    Then no toast is shown while the dialog is open

  @US-SHELL-08
  Scenario: Whole-row click on a settings row
    Given the app is open on Settings with "Appearance" unfolded
    When I click the label "Reduce motion"
    Then the "Reduce motion" switch is on
    When I click the empty right side of the "Reduce motion" row
    Then the "Reduce motion" switch is off

  @US-SHELL-09
  Scenario: Action bar placement and primary position
    Given the app is open on Mods
    Then the action bar shows "New group" and then "Install mod…"
    And "Install mod…" is the rightmost button
    And both buttons are 36 px high
    And the action bar is inside the content area 20 px from the right and 14 px from the bottom

  @US-SHELL-10
  Scenario: Reduce motion from the OS and from the in-app switch
    Given the OS asks for reduced motion
    Then the page entrance animation lasts about 0 seconds
    And spinners pulse instead of spinning
    Given the app is open on Settings
    When I turn on the "Reduce motion" switch
    Then the page entrance animation lasts about 0 seconds
    When I turn off the "Reduce motion" switch
    Then motion is restored

  @US-SHELL-11
  Scenario: High contrast and Larger text
    Given the app is open on Settings with "Appearance" unfolded
    When I turn on "High contrast"
    Then the background is "#000000" and the text is "#FFFFFF"
    When I turn off "High contrast" and turn on "Larger text"
    Then the body text is 15.5 px
    And every page has no horizontal scroll

  @US-SHELL-12
  Scenario: Window size does not grow when the viewport is 1920x1080
    Given the browser viewport is 1920x1080
    Then the window is 1280x720 at scale 1
    When I click "Maximise"
    Then the window stays 1280x720

  @US-SHELL-13
  Scenario: Card headers and rows share the same inset
    Given the app is open on Home
    Then the card header and each row are 12 px from both card edges

  @US-SHELL-14
  Scenario: Sidebar badges follow the state
    Given the app is open with setup finished
    Then only "System" shows an "Update available" icon badge
    When I switch off one mod
    Then "Load order" shows a badge "1" titled "untested change"
    And "LOD" shows a badge "1" titled "LOD out of date"

  @US-SHELL-15
  Scenario: Arrow keys in the folder picker and Alt+Down moves a mod
    Given the app is open on Settings
    When I click "Browse…" for the game folder
    And I press ArrowDown
    Then the second folder is selected
    When I press Escape
    Given the app is open on Mods with the first mod focused
    When I press Alt+ArrowDown
    Then the first mod moves one place down
