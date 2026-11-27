# Shell notes (area `shell`)

Run: `node scripts/shell.js`. Last run: 0 console errors, 0 failed checks. Pictures: `screens/shell/01`–`23`. Measurements are CSS px in window coordinates at 1366x768 (window scale 1) unless stated. Source read: `../design/demo/src/style.css`, `overlays.js`, `core.js` (toast, render, fitWin), `actions.js` (keydown, click delegation, quit), `pages.js` (top bar, sidebar, footer, action bars, settings rows).

## Demo bugs found (ranked)
1. **Quit does not quit.** The close button (`quitAsk`) opens `Quit Raul’s Toolbox?`, but its `Quit` button is wired to `dlgClose`, so it only closes the dialog. Enter does the same.
2. **Minimise and Maximise are no-ops but are Tab stops** (`data-act="noop"`). Maximise does not change the window.
3. **The window never grows.** `#win` is fixed at 1280x720 and `fitWin` only scales down (max 1). At 1920x1080 it stays 1280x720 at scale 1, centred; content width stays 1064 px. "Maximised" pages do not fill the screen.
4. **Folder picker, first ArrowDown skips the first option.** With nothing selected (index -1), ArrowDown selects index 1.
5. **Ctrl K then Escape from the keyboard drops focus to `body`.** `U.ret` is taken from the active element's `data-k`, which is empty when nothing is focused. Opened by mouse, Escape returns focus to the Search button correctly.
6. **Settings hides its switches until a section is unfolded.** The `Appearance` section (Reduce motion, High contrast, Larger text) is folded by default; a keyboard user sees `Unfold Appearance` and must find it.
7. (code read, not exercised) `toast` Undo calls the global `undo()`, which undoes the latest change, not necessarily the change the toast names.
8. (code read) Toast timers keep running while a dialog hides the toasts, so a toast that expires under a dialog is lost without being seen.
9. (code read) Install dialog: Escape and scrim are ignored while `busy`, by design; not exercised here.

## Keyboard
- Tab order at the top of Home: `Minimise` (noop), `Maximise` (noop), `Close` (quit confirm), `New Vegas`, `Engine 0.24.0`, `Profile Beautiful`, `Search or go to… Ctrl K`, `Play`, then nav `Home`, `Mods`, `Load order`, `LOD`, `System`, `Doctor`, `Settings`, then Home content (`hrow:game` Fallout: New Vegas OK, `hrow:ttw`, `hrow:mods`, `hrow:load`, `hrow:far`, `Shortcuts` footer button).
- Settings: after the nav, stops are `Fold Game installations`, `Browse…`, `Find`, `Browse…`, `Find`, `Show in Mods`, `Unfold Toolbox storage`, `Profile`, `Unfold Game settings…`, `Unfold Appearance`, `Unfold Updates`, `Unfold Advanced`.
- Enter on a focused nav item opens the page; one Tab per nav item (verified for all 7 pages). Focus stays on the nav item.
- Ctrl K opens the palette with `INPUT[palIn]` focused. Typing filters; ArrowDown/ArrowUp move (wrap); Enter runs. Ctrl K toggles it closed. Escape closes it. Scrim click closes it.
- `?` opens the Shortcuts sheet only when no dialog is open, nothing is typed into, and setup is done. With the palette search open, `?` types `?` into the search field.
- Shortcuts sheet: focus lands on `Close`; Enter and Escape both close it.
- Dialogs: focus on open goes to the first of: input, selected option, folder option, option, `dlgok`, primary button. Name dialog focuses `INPUT[nameIn]`. Confirm dialog focuses its primary (`Quit`).
- Enter in a dialog submits the primary button, except on a plain button (native Enter). Name dialog Enter with an empty name does nothing (dialog stays with `New profile`); with a name, it submits and closes.
- Escape closes: confirm, palette, shortcuts sheet, menu (returns focus to its anchor), test-situations panel.
- Menu: opens focused on the first enabled item. ArrowDown/ArrowUp move (wrap), Home/End jump. Tab closes the menu. Click outside closes.
- Alt+ArrowDown on a focused mod moves it one place (order changed: true).
- Picker: ArrowDown/ArrowUp move between options (see bug 4).

## Focus look
- Button keyboard focus (`:focus-visible`): `solid 1px rgba(61, 214, 198, 0.75)`, outline-offset 1px, no box-shadow. Mouse focus: no ring (checked in `test/keys.js`).
- Text field (palette search): `outline: none`, `box-shadow: none`. Caret is the only sign.
- Row focus: a row's name button gets the outline (`solid 1px`); the row tint is `.r:focus-visible` / `.sech:focus-visible` / `.logl:focus-visible` / `.pal .pi:focus-visible` = `background: var(--sel)` (`#2A3240`), with outline none.
- Sidebar nav: `.nav:hover` = `--raise` background; `.nav.on` = `--raise2`; the active icon is `--accent`.

## Whole-row click targets
- Settings rows (`.r.click[data-act="gset"]`, width 992): clicking the label toggles the switch (false to true verified); clicking the empty right padding toggles it back (verified).
- Mods rows: right-side click on a mod row opens it (open rows 0 to 1).
- Home rows (`hrow:*`) are clickable whole rows (data-act `nav`).
- Hit area: `.hit` is 44x44 with -6 px margin.

## Toasts
- Container: `.toasts`, 440 px wide (x 420, right 860), bottom 40 px above the window bottom (window h 720, container bottom 680), centred (`left:50%`), 8 px gap.
- Toast: 52 px high (`min-height:52px`), padding 8/8/8/16, radius 14, background `--sel`, `pop` entrance 0.18 s.
- Order in the toast: text (flex 1), `Undo` (ghost, 32 px high, with undo icon), then a 32x32 `Dismiss` icon button on the right (x 820 to 852; toast right 858).
- Stacking: at most 2 visible; the third pushes out the oldest (verified with 3 quick toggles).
- Timing: undo toast 5000 ms (measured 4903 ms after its last appearance); plain toast 3500 ms (measured 3402 ms, toast created by `toast()` and rendered).
- Dismiss X removes the toast at once (verified).
- Undo: `Undone: NMC's Texture Pack on` appears after Undo; the mod state reverts (verified).
- Hidden while any dialog is open (`U.dlg ? [] : U.toasts`); back when it closes (verified: 2 toasts after closing).
- Toast copy seen: `NMC's Texture Pack off`, `Undo`, `Undone: NMC's Texture Pack on`. Plain toasts elsewhere use the copy the action passes.

## Dialogs and menus
- Dialog: `.dlg` 520 px wide (760 with `.wide`), radius 20, padding 20 px 22 px, `pop` 0.18 s on a `fade` 0.15 s scrim. Confirm dialog measured: x 380, y 287, w 520, h 146, right 900, bottom 433 (centred). Inside the window.
- Confirm buttons: `Cancel` (ghost) then `Quit` (primary, rightmost). Quit dialog body: `Running jobs stop. Your setup is saved.`
- Scrim: a click on the scrim closes it; a click inside the dialog body does not (verified).
- Shortcuts sheet rows: `Ctrl K` / `Search or go to a page`; `?` / `This list`; `Alt ↑ / Alt ↓` / `Move the focused mod, group or plugin`; `Space` / `Switch or tick the focused control`; `Enter` / `Open the focused row`; `Esc` / `Close a menu or dialog`. Button `Close`.
- Menu: `.menu` radius 14, padding 6, min width 220, `pop` 0.12 s, placed under its anchor with a 6 px gap (flips above when there is no room), clamped to 8 px inside the window. Profile menu measured x 448.5, y 94, w 220, h 320, right 668.5, bottom 414.
- Menu items: 38 px high, radius 10; disabled items keep their text colour (`--muted`).
- Palette: 600 px wide, top at 12vh, search field plus up to 9 rows of 42 px.

## Sidebar, top bar, footer
- Top bar `.top`: x 8, y 42 (8 px margin), w 1264, h 56 visible (grid row 64 px). Chips 36 px high: `New Vegas`, `Engine 0.24.0`, `Profile Beautiful` (menu), search pill `Search or go to…` (min 240 px wide, 40 px high, `Ctrl K` kbd), `Play` (primary).
- Sidebar `.side`: 216 px wide, nav items 42 px high (all 7 measured 42), groups `PLAY` (Home, Mods, Load order, LOD) and `SYSTEM` (System, Doctor, Settings).
- Badges (`.bdg`, 22 px high, min 22 px wide, radius 11, text 0.75 rem bold; colours info blue, warn orange, err red):
  - At setup: `System` shows an icon-only badge `Update available` (blue, arrow icon). No other badge.
  - One mod switched off: `Load order` shows `1` (title `untested change`, orange), `LOD` shows `1` (title `LOD out of date`).
  - Code also defines: `Load order` missing masters (red), out of order; `Mods` needs attention (red or orange); `Doctor` failed (red) or warnings (orange) after a doctor run. Not reached in this probe (no doctor run, no attention issues).
- Footer `.foot`: 32 px high, `Ready` status with a dot; during a job it reads `<label> · N %` and the dot pulses; `Shortcuts` ghost button with `?` kbd at the right.

## Action bar (`.abar`)
- Position: absolute, right 20 px, bottom 14 px inside `#main`. Measured: bar x 964.9 to 1260 (w 295), y 626 to 674 (h 48); main y 98 to 688. Right gap 20 px, bottom gap 14 px. Radius 999 (pill), background `--raise2`.
- Buttons 36 px high, all equal (verified on Mods, Load order, LOD, Doctor).
- Primary rightmost: Mods `New group` (ghost) then `Install mod…` (primary, rightmost). Load order: `Sort…` (ghost). LOD: `Fine-tune` (ghost), `Rebuild LOD` (ghost). Doctor: `Run all checks` (primary). System: no bar.
- Clearance: `.mscroll` has 84 px bottom padding, so the last row can scroll clear of the bar (verified value; a row under the bar is visible in 14-abar-mods.png, the list scrolls underneath).

## Animations and transitions (style.css)
Keyframes (7):
| name | used by | duration / timing |
|---|---|---|
| `pgin` | `.pg` (every page), `.mdet` (expanded mod details) | 0.22 s ease-out (page), 0.18 s (details) |
| `spin` | `.spin`, `.btn.busy svg` | 1 s linear, infinite |
| `pulse` | `.foot .dot.busy` (1.4 s infinite); reduced-motion fallback for spinners (1.6 s) | opacity 1 to .35 |
| `sheen` | `.pb i` progress bar gloss | 1.4 s linear, infinite |
| `slide` | `.pb.ind i` indeterminate bar | 1.2 s ease-in-out, infinite |
| `fade` | `.scrim` | 0.15 s |
| `pop` | `.dlg` (0.18 s), `.menu` (0.12 s), `.toast` (0.18 s) | scale .97 to 1 plus fade |

Transitions (7 rules):
| selector | property, duration | use |
|---|---|---|
| `.fold` | transform .18 s | section fold chevron |
| `.btn` | background .15 s, transform .1 s | every button |
| `.sw` | background .15 s | switch track |
| `.sw::after` | left .15 s, background .15 s | switch knob |
| `.pb i` | width .35 s ease | progress bar fill |
| `.mrow` | background .15 s | mod row hover |
| `.chev` | transform .18 s | row chevron |

Reduce motion (verified by `emulateMedia` and the in-app switch):
- OS `prefers-reduced-motion: reduce`: all animation and transition durations become `.01 ms` (computed `1e-05s`); `.spin` and `.btn.busy svg` become `pulse 1.6s`; indeterminate bar held at `left:35%`.
- In-app `Reduce motion` (Settings, Appearance; `data-motion="0"`): same as above. Switching it off restores `data-motion="1"` and normal motion (verified: page entrance back to 0.22 s).
- Note: in-app `Reduce motion` is on when `S.g.motion` is false; the switch label is the same either way.

## High contrast and Larger text
Colour variables (verified via `getPropertyValue` on `:root`):
| var | default | High contrast (`data-hc="1"`) |
|---|---|---|
| `--bg` | #101215 | #000 |
| `--raise` | #181C22 | #000 |
| `--raise2` | #222831 | #000 |
| `--sel` | #2A3240 | #111 |
| `--hover` | #29303B | #151515 |
| `--line` | #2F353F | #B8B8B8 |
| `--ctl` | #7A848F | #fff |
| `--text` | #E6E8EB | #fff |
| `--muted` | #A3ACB7 | #E0E0E0 |
| `--accent` | #3DD6C6 | #5CF2E2 |
| `--on-accent` | #0B1513 | (unchanged) |
| `--brand` | #F59E3B | (unchanged) |
| `--ok` | #4CC38A | #6FE8A6 |
| `--warn` | #FF9A4D | #FFB066 |
| `--err` | #FF7A7A | #FF8A8A |
| `--info` | #6EA8FE | #8DBBFF |
| `--sunk` | #0C0E10 | #000 |
| `--choice` | #B79CFF | (unchanged) |
Other tokens: `--font` Inter, system-ui; `--mono` ui-monospace; `--fs` 14 px (Larger text 15.5 px); body background `#07080A` (outside the window); window `--bg`.
- High contrast adds 1 px `--line` borders to `.r`, `.card`, `div.sec`, `.banner`, `.btn`, `.tag`, `.abar`, `.mrow`, `.stile`, `.top`, `.menu`, `.dlg`.
- Larger text: body 15.5 px (verified). No horizontal overflow on Home, Mods, Load order, LOD, System, Doctor, Settings (`.mscroll` scrollWidth 1064 = clientWidth 1064; top bar not overflowing).

## Window size
- `#win` is 1280x720 fixed, scaled by `--ws = min(1, (w-32)/1280, (h-32)/720)`, floor 0.3, centred with `transform`.
- 1366x768: scale 1. 1920x1080: scale 1 (window stays 1280x720, not maximised; desk shows around it). 1280x720: scale 0.9556 (window 1223.1x688).
- Content width (`.mscroll`) is 1064 px at every size checked.
- Maximise: no-op (window stayed 1223.1x688 at viewport 1280x720 before and after the click).

## Alignment (px, from getBoundingClientRect)
- Home: card `div.sec` padding 12 px; header `.sech` left 12 / right 12 from card edge; each `.r` row left 12 / right 12. Header and rows share the inset.
- Mods: first three cards: header `.sech` 12/12, `.rows` 12/12. Same inset.
- Sidebar nav items 42 px high, 8 px inset from sidebar edge (`.side` padding 12 px 8 px).
- Action bar: 20 px from the content's right edge and 14 px from its bottom.
- Toast: 440 wide, 40 px from the window bottom.
- Dialog: centred in window (confirm: left 380, right 900 in a 1280 window: 380 left, 380 right).

## Keyboard and Enter/Escape table (checked)
| control | Enter | Escape | Scrim click |
|---|---|---|---|
| Quit confirm | closes (does not quit) | cancels | closes |
| Shortcuts sheet | Close | closes | closes |
| Name dialog | submits (blocked when empty) | not tested here | closes |
| Palette | runs highlighted item | closes (focus to Search button if opened by mouse; to body if opened by keyboard) | closes |
| Profile menu | item action | closes, focus to anchor | closes on outside click |
| Folder picker | option click | closes | not tested |

## Not covered here (already in other scripts or out of scope)
- Install dialog steps, folder dialog Enter, vRevert confirm: see `../design/demo/test/keys.js` (Enter submits every dialog) and `test/run.js`; not re-run in this probe.
- Wizard overlays (owned by another area).
