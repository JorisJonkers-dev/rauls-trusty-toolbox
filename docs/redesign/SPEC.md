# Raul's Toolbox: visual and interaction spec

This is the look and feel the app must match. The demo is the reference: every value below is read from
`../design/demo/src/style.css` (CSS), `../design/demo/src/core.js`, `pages.js`, `overlays.js` and `actions.js` (JS), or measured by
`scripts/shell.js` (see `notes/shell.md`). "Where" gives the file and selector or function to copy from. Sizes are CSS
pixels at window scale 1. Font sizes written in `rem` in the CSS are given in px at a 16 px root (the demo does not set
the root size). Function ids refer to `FUNCTIONALITY.md`.

## 1. Product rules

Distilled from the owner's decisions. They apply to every ticket.

1. Look: readable cool blue-grey dark, teal accent, orange for warnings, red for errors, amber only for the brand mark. No light theme. No warm gold. No all-amber terminal look.
2. Rounded blocks instead of lines and dividers. Cards, rows and bars are filled shapes with radii. Hairlines only as a faint line colour on timelines and in high contrast.
3. Little text. Do not flood a page. One status phrase per header, short row text, details fold out on demand.
4. No command lines, no "Show command", no other CLI pieces in the UI.
5. Blocks grow to the page width. No maximum widths on cards, banners or bars. Only the wizard keeps a centred column (900 px).
6. A card header and its rows share the same left and right inset (12 px from the card edge).
7. Page actions (submit, update, install, rebuild, next, save, OK, Cancel) live in a floating bar at the bottom right, primary rightmost, equal height, compact. The header holds title and status only (plus the profile pill and stable chip, which are indicators with a menu).
8. The action bar floats over the content; no strip is reserved. The content has bottom padding so its last row scrolls clear.
9. Whole rows are click targets: a click folds the row open or flips its switch. Nested buttons keep their own action.
10. Enter submits forms and dialogs (primary action). Escape cancels.
11. Focus: text fields show only the caret. Other controls show one thin indicator on keyboard focus only. Never a double ring or outline on mouse focus.
12. Mods page: a click on a row folds its details out downwards inside the list. No split pane. The only on/off control is the one in the row.
13. Toasts have a fixed width, are centred at the bottom, and have the close X on the right.
14. The UI says "LOD" with "Render distance" as a tooltip. Never "Far view".
15. Check again and Update live on System only (plus the play-check's own "Check again" on Home). Elsewhere a badge on the System item.
16. Profiles are full combinations (mods, order, settings, LOD). They share installed mods and custom groups. They are switchable from the top bar and the Mods, Load order and LOD pages.
17. Recommend a stability test whenever the exact mod set and order is not a known-stable combination.
18. Home is one single column. One playtime total. The live profile is an indicator only.
19. The update button says "Update"; parts say "will be updated". Version text does not appear in the button.
20. Progress and loading icons spin; busy buttons spin their icon.
21. Location pictures: filter by world only (New Vegas, and Capital Wasteland when TTW is on); one scrolling row of places with an "All N" toggle; the wheel zooms, drag pans.
22. Comparison pictures sit side by side with labels, never stacked.
23. The app is built with egui (ADR-0012), bundles the Inter font, and says "Raul's Toolbox".
24. Text in the UI is plain and short. No jargon in headers; mod and plugin names are shown as they are.

## 2. Colour tokens

Where: `style.css` `:root` and `[data-hc="1"]`. In egui: one `Palette` struct, swapped by the High contrast switch.

| Token | Default | High contrast | Use |
|---|---|---|---|
| `--bg` | `#101215` | `#000000` | window and page background |
| `--raise` | `#181C22` | `#000000` | cards, top bar, banners, dialogs |
| `--raise2` | `#222831` | `#000000` | rows inside cards, chips, action bar, icon tiles, menus |
| `--sel` | `#2A3240` | `#111111` | selected or focused row tint, toast fill, open mod row |
| `--hover` | `#29303B` | `#151515` | hover on rows and buttons |
| `--line` | `#2F353F` | `#B8B8B8` | switch border, timeline, dashed empty box, menu separator |
| `--ctl` | `#7A848F` | `#FFFFFF` | checkbox border, grip, switch off knob variants |
| `--text` | `#E6E8EB` | `#FFFFFF` | main text |
| `--muted` | `#A3ACB7` | `#E0E0E0` | secondary text, ghost buttons, icons |
| `--accent` | `#3DD6C6` | `#5CF2E2` | primary button, active icon, section bar, progress, selection ring |
| `--on-accent` | `#0B1513` | same | text on accent |
| `--brand` | `#F59E3B` | same | brand mark and name only |
| `--ok` | `#4CC38A` | `#6FE8A6` | success |
| `--warn` | `#FF9A4D` | `#FFB066` | warnings, untested-change dashed border |
| `--err` | `#FF7A7A` | `#FF8A8A` | errors, danger |
| `--info` | `#6EA8FE` | `#8DBBFF` | information, update badge |
| `--sunk` | `#0C0E10` | `#000000` | input fields, switch track, log box |
| `--choice` | `#B79CFF` | same | choice marker colour |

Extra literal values: desk behind the window `#07080A` (demo only); title bar `#0B0D10`; primary hover `#5DE3D5`;
primary inner shade `inset 0 -3px 0 rgba(0,0,0,.18)`; tag fills are the status colour at 14 % alpha
(ok `rgba(76,195,138,.14)`, warn `rgba(255,154,77,.14)`, err `rgba(255,122,122,.14)`, info `rgba(110,168,254,.14)`,
accent `rgba(61,214,198,.14)`); text highlight `rgba(61,214,198,.25)`; scrim `rgba(0,0,0,.55)`; close button hover `#C4403A`.
High contrast also adds a 1 px `--line` border to rows, cards, sections, banners, buttons, tags, action bar, mod rows,
stat tiles, top bar, menus and dialogs (`[data-hc="1"] .r, .card, ...`).

## 3. Type

Font `Inter` then system sans; mono `ui-monospace` for versions, keys, log times. Base size `--fs` 14 px (15.5 px with Larger text, `[data-fs="l"]`). Line height 1.45.

| Role | px | Weight | Where |
|---|---|---|---|
| Page title `h1` | 24.8 | 750, tracking -0.01em | `.phdr h1` |
| Wizard title | 30.4 | 700 | `.wiz h1` |
| Wizard lead | 16.8 | 400 | `.wiz .lead` |
| Dialog title | 19.2 | 700 | `.dlg h2` |
| Stat value | 23.2 | 750 | `.stile .sv` |
| Section header text | 13.1, uppercase, tracking .06em | 700 | `.sech .t` |
| Sidebar group label | 11.8, uppercase, tracking .08em | 700 | `.navh` |
| Body | 14 | 400 or 500 | `body` |
| Row sub text | 13.8 | 400 | `.r .sub` |
| Banner sub | 14.7 | 400 | `.banner .bs` |
| Tag | 12.8 | 600 | `.tag` |
| Badge | 12 | 700 | `.bdg` |
| Footer | 13.8 | 400 | `.foot` |
| Kbd hint | 12.5 mono | 400 | `.kbd` |
| Title bar | 13.4 | 600 | `.tbar` |

## 4. Spacing, radii and component sizes

Where: `style.css`. Insets are padding, gaps are between siblings.

| Piece | Size | Selector |
|---|---|---|
| Window | 1280x720 design size, radius 12 (the app uses the real window) | `.win` |
| Title bar | 34 high; buttons 40x28, radius 7 | `.tbar`, `.wbtn` |
| Grid | rows 64 / 1fr / 32; columns 216 / 1fr | `#app` |
| Top bar | 56 visible, margin 8 8 0, radius 14, padding 0 10 0 16, gap 6 | `.top` |
| Top chip | 36 high, padding 0 12, radius 10 | `.tchip` |
| Search pill | 40 high, min 240 wide, padding 0 10 0 14, radius pill | `.srchbtn` |
| Sidebar | 216 wide, padding 12 8 | `.side` |
| Nav item | 42 high, padding 0 12, gap 12, radius 12 | `.nav` |
| Nav group label | padding 14 10 6 | `.navh` |
| Badge | 22 high, min 22 wide, radius 11, padding 0 6 | `.bdg` |
| Content area | padding 18 24 84 (bottom 84 clears the action bar); 1064 wide at design size | `.mscroll` |
| Page header | min 48 high, gap 14, margin-bottom 16; icon tile 48x48 radius 14 | `.phdr` |
| Profile pill | 36 high, padding 0 10 0 12, radius pill | `.ppill` |
| Card | radius 16, padding 14 16, margin-bottom 12 | `.card` |
| Section block | radius 16, padding 12, margin-bottom 12 | `div.sec` |
| Section header | min 48 high, padding 8 8 8 14, margin-bottom 6, radius 12; accent bar 6x16 radius 3 before the title | `.sech` |
| Fold button | 32x32 circle, chevron rotates 90 degrees | `.fold` |
| Row | min 48 high, padding 6 14, gap 12, radius 12, fill `--raise2`; rows gap 6 | `.r`, `.rows` |
| Mod row details | padding 4 16 14 96, two columns | `.mdet` |
| Button | 40 high, padding 0 18, radius pill, gap 8, weight 600 | `.btn` |
| Small button | 32 high, padding 0 12, 14 px text | `.btn.sm` |
| Icon button | 36x36 | `.btn.icon` |
| Segmented control | pill, padding 3, buttons 34 high, padding 0 16 | `.segc` |
| Tag | 26 high, radius 13, padding 0 11 | `.tag` |
| Switch | 42x24, knob 18, track radius 12, knob left 3 (off), 19 (on), 11 (mixed) | `.sw` |
| Checkbox | 22x22, radius 7, border 1.5 | `.cb` |
| Input | 40 high, radius 12, fill `--sunk`, padding 0 12 | `.inp` |
| Search field | 40 high, radius pill, fill `--raise`, min 240 | `.srch` |
| Progress bar | 8 high, radius 4 | `.pb` |
| Banner | radius 16, padding 14 16, gap 14; icon disc 44 | `.banner` |
| Stat tile | radius 12, padding 10 14, four columns gap 8 | `.stile` |
| Stepper | radius 16, padding 14 18; number disc 34 | `.stp` |
| Action bar | 48 high pill, padding 6, gap 6; buttons 36 high, padding 0 16; right 20, bottom 14 of the content area; shadow `0 10px 30px rgba(0,0,0,.5)` | `.abar` |
| Dialog | 520 wide (760 wide variant), radius 20, padding 20 22, shadow `0 20px 60px rgba(0,0,0,.6)`; button row right-aligned gap 10, margin-top 12 | `.dlg`, `.cca` |
| Menu | min 220 wide, radius 14, padding 6; item 38 high radius 10 | `.menu`, `.mi` |
| Toast | 440 wide, min 52 high, radius 14, padding 8 8 8 16, gap 8, bottom 40 | `.toasts`, `.toast` |
| Palette | 600 wide, padding 12, rows 42 high, up to 9 rows, top at 12 % of the window | `.pal`, `.pi` |
| Footer | 32 high, padding 0 16, dot 8x8 | `.foot` |
| Hit area | 44x44 around small controls, margin -6 | `.hit` |
| Profile cards (wizard) | two columns gap 10, radius 16, padding 14 16 | `.pcard` |
| Picture pair | two columns gap 10; frame radius 12; view 16:9 radius 8 | `.pics`, `.pic` |
| Place buttons | 30 high | `.places .btn.sm` |

## 5. Alignment rules (measured)

From `notes/shell.md` at 1366x768, scale 1.

- Card header and rows: header left and right inset 12; each row left 12 and right 12 from the card edge. Verified on Home and Mods (`../design/demo/test/insets.js`).
- Sidebar items are inset 8 from the sidebar edge and are 42 high (all seven).
- Top bar: x 8, y 42 in the page (8 px margin), width 1264, chips 36 high, search 40 high.
- Action bar: right gap 20, bottom gap 14; measured x 964.9 to 1260 (295 wide), y 626 to 674 (48 high). Buttons all 36 high (Mods, Load order, LOD, Doctor).
- Content bottom padding 84 so the last row clears the bar.
- Toasts: 440 wide, centred (x 420 to 860 in a 1280 window), 40 above the window bottom, 8 gap between toasts. The X is 32x32, 6 px inside the right edge.
- Dialog: centred; confirm dialog measured x 380, y 287, w 520, h 146.
- Menu: 6 px below its anchor; flips above if it does not fit; clamped 8 px inside the window. Profile menu measured x 448.5, y 94, w 220.
- Mod row order: grip, number, checkbox, name, tags and version right-aligned, chevron last. Group header: tri-state switch, options button and fold right-aligned in the chevron column.
- Load order numbers are right-aligned in a fixed 26 px column; tags at the far right.
- Settings rows: label left (min 200), controls in a fixed-width end column; path field 340.
- Header: status chip next to the title; profile pill at the right of the header row on every page that has one.
- Cards on a page fill the full width of the content column (rule 5).

## 6. Animations and transitions

Where: `style.css` `@keyframes` and `transition`. Reduce motion: see the last column.

| Name | What moves | Duration, easing | Used by | Reduce motion |
|---|---|---|---|---|
| `pgin` | opacity 0 to 1, rise 6 px | 0.22 s ease-out | every page (`.pg`) | 0.01 ms |
| `pgin` | same | 0.18 s ease-out | opened mod details (`.mdet`) | 0.01 ms |
| `fade` | scrim opacity | 0.15 s | `.scrim` | 0.01 ms |
| `pop` | scale .97 to 1 and fade | 0.18 s ease-out | dialog, toast | 0.01 ms |
| `pop` | same | 0.12 s ease-out | menu | 0.01 ms |
| `spin` | rotate 360 | 1 s linear, infinite | `.spin`, busy button icon | becomes `pulse` 1.6 s ease-in-out |
| `pulse` | opacity 1 to .35 | 1.4 s, infinite | busy dot in the footer | 1.6 s pulse (kept) |
| `sheen` | gloss moves across the fill | 1.4 s linear, infinite | progress fill `.pb i` | 0.01 ms |
| `slide` | 30 % bar travels | 1.2 s ease-in-out, infinite | indeterminate bar `.pb.ind` | bar parked at left 35 % |
| width | progress fill width | 0.35 s ease | `.pb i` | 0.01 ms |
| transform | fold chevron rotate 90 | 0.18 s | `.fold`, `.chev` | 0.01 ms |
| background | button, mod row | 0.15 s | `.btn`, `.mrow` | 0.01 ms |
| transform | button press 1 px down | 0.1 s | `.btn:active` | 0.01 ms |
| background, left | switch track and knob | 0.15 s | `.sw`, `.sw::after` | 0.01 ms |

Other motion: toasts leave without an exit animation (removed at the end of their timer); the dragged ghost is tilted -1.2 degrees with a shadow; the picture zoom has no easing (wheel steps apply at once). Reduce motion is on when the OS asks (`prefers-reduced-motion`) or when the in-app switch is on (`data-motion="0"`). Spinners never stop completely: they become a slow pulse so a busy state stays visible.

## 7. Focus and keyboard

Where: `actions.js` `keydown` handler, `style.css` focus rules, `core.js` `U.ret`/`U.focus`.

- Tab order: title buttons, top bar (New Vegas, Engine, Profile, Search, Play), sidebar (one stop per item), page content, footer Shortcuts.
- Keyboard focus: 1 px outline, `rgba(61,214,198,.75)`, offset 1. Mouse focus: nothing. Text fields: caret only. Rows, section headers, log lines and palette items: background `--sel`, no outline.
- Enter: opens the focused row or runs the focused control. In a dialog Enter runs the last primary or danger button, except when a plain button is focused (that button gets the Enter).
- Space: switches or ticks the focused control.
- Escape order: menu, palette, dialog (not while an install runs), test panel (demo only), drag.
- Alt+↑ / Alt+↓: move the focused mod, group or plugin one place (not while a sort preview is shown). Focus returns to the moved item.
- Menus: first enabled item focused; ↑ ↓ wrap; Home and End; Tab closes; Escape closes and focuses the anchor.
- Palette: Ctrl K toggles; ↑ ↓ wrap; Enter runs; "?" typed in the field is text.
- "?" opens Shortcuts only when not typing, no dialog is open and setup is finished.
- Folder picker: ↑ ↓ move the selection (the demo skips the first item on the first press, a bug).
- Dialog focus start: first of input, selected item, picker item, option, OK, primary button. Tab wraps inside. Focus returns to the opener on close.
- After a toggle, move or group edit the focus stays on the same item.

## 8. Behaviour rules worth copying exactly

- Toasts: at most 2; plain 3.5 s; with Undo 5 s; hidden while a dialog is open; X dismisses at once. Where: `core.js` `toast`, `overlays.js` `overlays`.
- Undo: stack per profile, 30 deep, cleared by a revert. Where: `core.js` `change`, `undo`.
- Versions: a new version per change; edits within 30 s join the latest. Where: `core.js` `recordVersion`.
- Badges are pure functions of state. Where: `pages.js` `sideHTML`, `docSummary`; `core.js` `checks`.
- Window: the app uses the real window. `fitWin` (scale to fit) is demo only.
- Dismissal of the untested-change banner is stored per setup signature. Where: `core.js` `sig`, `actions.js` `recDismiss`.

## 9. How to read the demo for a control

1. Find the handler in `../design/demo/src/actions.js` (`A.<name>`).
2. Find the markup with `data-act="<name>"` in `pages.js`, `overlays.js` or `wizard.js`.
3. Find the class in `style.css`.
4. Open the matching story in `stories/` for the exact text and `screens/` for the picture.
