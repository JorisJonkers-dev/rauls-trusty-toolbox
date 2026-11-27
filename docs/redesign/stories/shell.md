# Shell: cross-cutting behaviour (area `shell`)

Evidence: `scripts/shell.js` (run: 0 console errors, 0 failed checks), pictures in `screens/shell/`, measurements in `notes/shell.md`.

### US-SHELL-01 Walk every page by keyboard
As a player, I want to reach every page from the keyboard alone, so that I can use the app without a mouse.
Acceptance:
- From the top of the window, Tab reaches `Minimise`, `Maximise`, `Close`, `New Vegas`, `Engine`, `Profile`, `Search or go to…`, `Play`, then the nav items `Home`, `Mods`, `Load order`, `LOD`, `System`, `Doctor`, `Settings`.
- Each nav item is one Tab stop; Enter on a focused nav item opens that page (main label `Home`, `Mods`, `Load order`, `LOD`, `System`, `Doctor`, `Settings`).
- After Enter the focus stays on the nav item.
Screens: `screens/shell/01-walk-home.png` … `07-walk-settings.png`
Demo handlers: A.nav, A.noop (Minimise, Maximise)

### US-SHELL-02 Search or go to with Ctrl K
As a player, I want Ctrl K to open a search that jumps to pages, actions and mods, so that I do not hunt through the sidebar.
Acceptance:
- Ctrl K opens a dialog labelled `Search or go to` with the search field focused; placeholder `Pages, actions, profiles, mods`.
- Typing `mod` lists `Mods Page`, `Install mod… Action`, `Weapon Mods Expanded Mod`.
- ArrowDown then Enter runs the highlighted item and closes the palette.
- Ctrl K again closes it. Escape closes it. A click on the window scrim closes it.
- Escape returns focus to the Search button when it was opened by mouse.
Screens: `screens/shell/08-palette-mod.png`
Demo handlers: A.palette, A.palGo, A.scrim

### US-SHELL-03 Shortcuts sheet
As a player, I want a sheet of shortcuts, so that I can learn the keys.
Acceptance:
- `?` (not while typing, no dialog open, setup finished) or the footer `Shortcuts` button opens a dialog titled `Shortcuts`.
- Rows: `Ctrl K` Search or go to a page; `?` This list; `Alt ↑ / Alt ↓` Move the focused mod, group or plugin; `Space` Switch or tick the focused control; `Enter` Open the focused row; `Esc` Close a menu or dialog.
- Focus lands on `Close`. Enter and Escape both close it.
- With the search field focused, `?` types into the field and does not open the sheet.
Screens: `screens/shell/09-shortcuts-sheet.png`
Demo handlers: A.shortcuts, A.dlgClose

### US-SHELL-04 Quit confirm from the window close button
As a player, I want the window close button to ask before quitting, so that a running job is not stopped by accident.
Acceptance:
- Close (top-right of the window) opens a dialog `Quit Raul’s Toolbox?` with text `Running jobs stop. Your setup is saved.`, buttons `Cancel` and `Quit`.
- Focus lands on `Quit`. Escape, `Cancel`, and a scrim click close it.
- Enter and clicking `Quit` both close the dialog. The window stays open (the demo cannot quit; see notes: the Quit button only closes the dialog).
Screens: `screens/shell/10-quit-confirm.png`
Demo handlers: A.quitAsk, A.dlgClose

### US-SHELL-05 Dialogs: placement, focus, Escape, Enter, scrim
As a player, I want every dialog to sit inside the window, take focus, and close the same way, so that I always know what Enter and Escape will do.
Acceptance:
- The confirm dialog is 520 px wide and sits inside the window (measured x 380, y 287, right 900, bottom 433 in a 1280x720 window).
- A click on the dialog body does not close it; a click on the scrim does.
- Name dialog (`New profile`): focus on the input; Enter with an empty name does nothing; Enter with `Scratch test` creates the profile.
Screens: `screens/shell/10-quit-confirm.png`, `screens/shell/12-name-dialog.png`
Demo handlers: A.scrim, A.nameOk, A.profNew

### US-SHELL-06 Menus: arrows, Escape, Tab
As a player, I want the profile menu to work from the keyboard, so that I can switch profiles without a mouse.
Acceptance:
- Opening `Profile` (top bar) places the menu inside the window (measured x 448.5, y 94, 220 px wide) and focuses the first item `Vanilla+`.
- ArrowDown moves to `Beautiful`; End moves to `Delete “Beautiful”…`.
- Escape closes the menu and focus returns to the `Profile` button. Tab closes it too.
Screens: `screens/shell/11-profile-menu.png`
Demo handlers: A.profMenu, A.profSwitch

### US-SHELL-07 Toasts: width, position, Undo, stacking, timing
As a player, I want short confirmations at the bottom of the window that I can undo or dismiss, so that a mistake costs one click.
Acceptance:
- Switching a mod off on Mods shows a toast `NMC's Texture Pack off` with `Undo` then a dismiss button labelled `Dismiss`.
- The toast container is 440 px wide and 40 px above the window bottom; the dismiss button is 32x32 at the right.
- Undo restores the mod and shows `Undone: NMC's Texture Pack on`.
- A third toast pushes out the oldest (two visible at most).
- Undo toasts stay about 5 s (measured 4.9 s); plain toasts about 3.5 s (measured 3.4 s).
- While a dialog is open the toasts are hidden, and they come back when it closes.
Screens: `screens/shell/13-toast-undo.png`
Demo handlers: A.modToggle, A.undo, A.toastX

### US-SHELL-08 Whole-row click targets
As a player, I want a click anywhere on a row to act, so that I do not have to aim at the small switch.
Acceptance:
- On Settings (with Appearance unfolded), clicking the label `Reduce motion` toggles its switch (off to on).
- Clicking the empty right padding of the same row toggles it back.
- On Mods, a click on the right side of a mod row opens it (open rows 0 to 1).
Demo handlers: A.gset, A.rowOpen

### US-SHELL-09 Floating action bar
As a player, I want the page action bar at the bottom right, so that the main action is always in the same place.
Acceptance:
- Mods: `New group` (ghost) then `Install mod…` (primary, rightmost); both 36 px high.
- Load order: `Sort…`. LOD: `Fine-tune`, `Rebuild LOD`. Doctor: `Run all checks` (primary).
- The bar is 48 px high, 20 px from the right and 14 px from the bottom of the content area.
- Content scrolls clear of it: the scroll area has 84 px bottom padding.
Screens: `screens/shell/14-abar-mods.png`, `15-abar-load.png`, `16-abar-far.png`, `17-abar-doctor.png`
Demo handlers: abar buttons on Mods, Load order, LOD, Doctor

### US-SHELL-10 Reduce motion (OS setting and in-app switch)
As a player with motion sensitivity, I want motion switched off, so that the app does not move.
Acceptance:
- With the OS preference `reduce`, page entrance is about 0 s (was 0.22 s) and spinners become a `pulse 1.6s` instead of `spin 1s`.
- The in-app `Reduce motion` switch (Settings, Appearance) sets the page to motion off: the page entrance becomes about 0 s; switching it off restores motion.
Screens: `screens/shell/18-settings-reduce-motion-on.png`
Demo handlers: A.gset (motion)

### US-SHELL-11 High contrast and Larger text
As a player with low vision, I want high contrast and bigger text, so that I can read the app.
Acceptance:
- `High contrast` on: background #000, text #fff, outlines #B8B8B8 on every block, accent #5CF2E2.
- `Larger text` on: body 15.5 px (was 14 px). No horizontal overflow on Home, Mods, Load order, LOD, System, Doctor or Settings (scroll width 1064 = client width 1064).
Screens: `screens/shell/19-settings-high-contrast.png`, `20-larger-text-mods.png`
Demo handlers: A.gset (hc, big)

### US-SHELL-12 Window size and maximised
As a player with a big screen, I want the window to use the space, so that pages are not cramped.
Acceptance:
- The window is a fixed 1280x720 design size and only scales down (never up). At 1920x1080 it stays 1280x720 at scale 1, centred, and pages do not grow (content width 1064).
- At 1280x720 it scales to 0.956 (1223x688).
- `Maximise` does nothing (window size unchanged).
Screens: `screens/shell/21-viewport-1920.png`
Demo handlers: A.noop (Maximise), fitWin

### US-SHELL-13 Alignment of card headers and rows
As a player, I want card headers and rows to share the same inset, so that the cards read as one column.
Acceptance:
- Home: header and each row sit 12 px from both card edges (card padding 12 px).
- Mods: header and rows sit 12 px from both card edges.
Demo handlers: none (layout)

### US-SHELL-14 Sidebar badges
As a player, I want a count on the pages that need attention, so that I know where to look.
Acceptance:
- At setup only `System` has a badge: an icon badge `Update available`.
- After switching one mod off: `Load order` shows `1` with title `untested change`; `LOD` shows `1` with title `LOD out of date`.
- Nav items are 42 px high.
Screens: none (badge text captured in notes)
Demo handlers: A.modToggle

### US-SHELL-15 Arrow keys in the folder picker and Alt+Up/Down moves
As a player, I want arrow keys to move through lists and Alt+Up/Down to move a mod, so that I can reorder from the keyboard.
Acceptance:
- Folder picker (Settings, `Browse…` for the game folder): with nothing selected, one ArrowDown selects index 1 (the second folder), not index 0 (measured -1 to 1). Flagged in notes as a demo bug.
- Alt+ArrowDown on the focused first mod moves it one place (order changes).
Demo handlers: A.pickFolder, A.moveModBy (keydown Alt+Arrow)
