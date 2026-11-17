# GUI rules

The rules every screen of the Toolbox GUI follows. MUST rules block a release; SHOULD rules need a written reason to
break. Each rule names how it is checked where a test, a lint or a snapshot review can check it. The reference is
the interactive mockup [`design/mockup.html`](design/mockup.html); the decisions behind these rules are
[ADR-0012](adr/0012-egui-is-the-gui-toolkit.md) to [ADR-0014](adr/0014-only-own-and-open-assets-ship.md) and
[`REORG.md`](REORG.md) sections 11 and 12.

Priority order: readability and ease of use first, then accessibility, then a light Fallout flavour. An all-amber,
scanline, Pip-Boy look was tried and rejected because it read like a 1990s website.

## 1. Pages and where data lives

- **P1 MUST:** the sidebar has two groups and seven pages: **Play** (Home, Mods, Load order, LOD) and **System**
  (Versions, Doctor, Settings). Home is the start page. Keys 1 to 7 open them in this order. Below 860 px the
  sidebar becomes a wrapping top bar. *Check: navigation test over the page registry.*
- **P2 MUST:** first run is a wizard on the same shell as the app (top bar, step rail, page header, footer with Back
  and Next): New Vegas folder, TTW (optional, asks for the Fallout 3 folder), Engine, Quick check, Ready. It can be
  skipped and run again from Settings, and writes nothing to a game folder. *Check: wizard state-machine tests.*
- **P3 MUST:** each kind of data has one home page; other pages may show it read-only with a link to that home.

  | Data | Home page | Shown read-only on |
  | --- | --- | --- |
  | Game installations (New Vegas, Fallout 3, TTW output) | Settings, Game installations | top bar (New Vegas), Home |
  | Toolbox storage (mods, downloads, engine builds, LOD output, logs) | Settings, Toolbox storage | LOD (output path) |
  | Engine builds, channel, active build | Versions, Engine | top bar, Home |
  | Component versions and the masterlist | Versions, Components | Load order (masterlist), LOD (generator) |
  | Mods, mod updates, conflicts, profiles | Mods | Home chips, Versions (count and link) |
  | TTW bundle: status, enable, set up, reinstall, remove | Mods, TTW band and pinned row | Home, Doctor |
  | Plugin order | Load order | Doctor |
  | Health checks | Doctor | Home chips |
  | App behaviour, appearance, notices | Settings | none |

- **P4 MUST:** TTW is set up in a four-step flow started from Mods: Sources (Fallout 3 folder, New Vegas folder,
  TTW installer), Output (target folder, space), Review (checks, time, outcome) and Install (stages; can be hidden
  and keeps running). Afterwards TTW is a pinned bundle row in Mods that can be enabled or disabled. Both source
  games stay untouched.
- **P5 MUST:** Settings never holds versions and Versions never holds paths. Doctor fixes nothing in place except
  trivial configuration; every other fix opens the owning page with the right control focused.

## 2. Page anatomy

- **A1 MUST:** every page has the same header: its icon, title, a one-line state ("3 of 6 blocks out of date") and
  the page tools on the right. At most one primary button per page. *Check: snapshot review.*
- **A2 MUST:** content is separated by hairlines and spacing on one continuous surface; only dialogs float.
- **A3 SHOULD:** list pages have two panes, list then detail, split by a vertical hairline; below 860 px the detail
  stacks under the list.
- **A4 MUST:** section labels are small uppercase text with the amber bracket tick.
- **A5 MUST:** nothing scrolls sideways at 402 px; long paths truncate with an ellipsis and show in full in a
  tooltip or the detail pane. *Check: narrow-width snapshot of every page.*

## 3. Global elements and attention

- **G1 MUST:** the top bar holds the brand ("Raul's Toolbox"), the setup strip (New Vegas folder, engine build with
  an update marker, profile; each one click to its home page), "Search or run" (Ctrl K) and Play.
- **G2 MUST:** one attention model, computed once, feeds three places: sidebar chips (count and icon per page), the
  Home chip row (each chip opens the page that fixes it) and Doctor rows. *Check: view-model test.*
- **G3 MUST:** Play is blocked only for blocking problems. It stays focusable, pressing it explains why, and the
  reason is shown under it with a Fix button. The same holds for any disabled Next: a line says what is missing.
- **G4 SHOULD:** transient results go to one polite status line in the footer; blocking problems are shown where
  they occur, never only in the footer.
- **G5 MUST:** Raul speaks at most one short line, only in calm places: the wizard, the Home launch panel and
  all-clear states. Never in errors, settings or dense lists.
- **G6 MUST:** every action shows its equivalent `raul` command, which reproduces it in a terminal.

## 4. Colour and themes

- **C1 MUST:** the dark theme uses these tokens. Every text colour is at least 4.5:1 on every surface and control
  borders at least 3:1. *Check: a unit test computes the ratios from the token table.*

  | Token | Hex | Role |
  | --- | --- | --- |
  | bg | `#101215` | canvas |
  | raise | `#171A1F` | dialogs, inputs |
  | sel | `#1F2530` | selected row, active navigation |
  | line | `#2F353F` | hairlines |
  | ctl | `#606973` | control borders |
  | text | `#E6E8EB` | text |
  | muted | `#98A1AC` | secondary text |
  | accent | `#3DD6C6` | primary buttons, active marker, focus ring, links |
  | brand | `#F59E3B` | logo, Raul's name, section ticks; never a status |
  | ok | `#4CC38A` | OK |
  | warn | `#F2A93B` | warning |
  | err | `#F26B6B` | problem |
  | info | `#6EA8FE` | updates, running work |

- **C2 MUST:** the accent is never amber, so warnings stay unambiguous; amber is the brand's only.
- **C3 MUST:** status is always colour, an icon and a word (OK, Warning, Problem, Update, Out of date, Not set).
- **C4 MUST NOT:** full-screen scanlines, glow, vignettes, monochrome themes or CRT effects.
- **C5 MUST:** there are two themes, dark and high contrast. High contrast uses pure white text, a black canvas,
  1 px borders on every control and no tinted fills, and passes 7:1 for text. There is no light theme.

## 5. Type, spacing, controls

- **T1 MUST:** Inter (OFL) is bundled for all interface text; the system monospace is used only for paths,
  versions and plugin names. 14 px base; page title 1.4 rem, section label 0.74 rem uppercase, body 1 rem,
  secondary 0.9 rem.
- **T2 SHOULD:** spacing on a 4 px grid (4, 8, 12, 16, 24, 32); list rows 40 to 44 px.
- **T3 MUST:** four button kinds: primary (accent fill), secondary (outline), ghost (text) and danger (red text and
  border). Labels are one or two words ("Install…", "Activate", "Roll back", "Apply 4 moves"); an ellipsis means a
  multi-step flow follows.
- **T4 MUST:** a destructive or long action (remove a mod, a build or TTW; rebuild all LOD) asks once, in a dialog
  that states the cost. Reversible actions (enable, sort with Undo, update with Roll back) do not ask.
- **T5 MUST:** everything that changes files follows plan, review, apply: the review renders a `raul-plan` document,
  names its restore point and says whether it can be undone.
- **T6 SHOULD:** settings save when a field is left; each save reports "<field> saved." and its marker updates.
- **T7 MUST:** filtered lists say so ("Showing 6 of 8") and offer Clear.

## 6. Icons

- **I1 MUST:** line icons on a 24 px grid, 1.6 px stroke, round caps, drawn in the current colour; 18 px in
  navigation, 28 px in page headers.
- **I2 SHOULD:** Fallout-flavoured but generic shapes: vault door (Home), crate (Mods), holotape stack (Load
  order), desert and sun (LOD), radio mast (Versions), syringe (Doctor), tools (Settings), two overlapping worlds
  (TTW). Never game art, characters, logos or the game's typefaces.
- **I3 MUST:** icons next to text are hidden from screen readers; icon-only controls have a label.
- **I4 MUST:** the app icon is Raul's amber wrench crossing a cog, flat and readable at 16 px.

## 7. Input: keyboard and gamepad

- **K1 MUST:** everything works by keyboard. Tab moves between controls; Up and Down move in lists; Space enables or
  disables; Enter moves to the selected item's actions; Esc closes dialogs (except while files are copying, which
  is explained); Ctrl K searches commands and mods; ? lists keys.
- **K2 MUST:** focus never lands on a destructive action by itself; after a re-render it returns to the same logical
  control or to the page heading.
- **K3 MUST:** long jobs (LOD, TTW, updates) show progress with a percentage, keep running when the page is left, and
  update in place.
- **K4 MUST:** every page and dialog works with a gamepad from the first GUI release: the D-pad moves focus, A
  activates, B goes back or closes, the shoulder buttons switch pages, Start opens the palette. The layout works at
  1280x800 (Steam Deck). *Check: an input-replay test drives each page by gamepad events.*

## 8. Accessibility

- **X1 MUST:** a skip link; table headers; the current page marked for assistive technology; modal dialogs with a
  title and a focus trap; progress with its value as text; a visible 2 px accent focus ring for keyboard focus.
- **X2 MUST:** Reduce motion follows the system and removes transitions; text size 100 to 200% never causes overlap
  at 402 px.
- **X3 SHOULD:** forced-colours mode keeps icons and chips visible.

## 9. Words

- **W1 MUST:** all text comes from one message catalogue; no string is assembled from fragments, and uppercase is a
  style, not how a string is stored. *Check: a lint finds literals in view code.*
- **W2 MUST:** labels and buttons are short. A sentence appears only where a state needs explaining, and then a plain
  one ("Points to build 0.13.0, not the active 0.14.2.").
- **W3 MUST:** an error says what went wrong, what was not changed when that is true ("Nothing changed."), and has
  a plain detail line (ADR-0003).
- **W4 MUST:** no real mod names, real user paths or game text in shipped strings, tests or screenshots.

## 10. Testing

- **S1 MUST:** every page has fixtures for its empty, loading, partial, error, read-only, offline, long-text and
  large-list states, rendered headless in CI (egui, no GPU) and compared with reviewed snapshots.
- **S2 MUST:** the GUI performs nothing the CLI cannot: the command-parity registry covers every action.

## 11. Assets and the install skin

- **L1 MUST:** the repository and releases contain only our own or openly licensed assets (GPL-3.0-only, CC0,
  CC-BY, CC-BY-SA, OFL, MIT, ISC, Apache-2.0). Each is listed in `packaging/NOTICE-assets.toml` with source, SPDX id
  and attribution, and shown in Settings, About and notices. *Check: CI fails on an unlisted asset.*
- **L2 MUST:** no game files and no mod files in the repository, installers, fixtures or support bundles. No mod's
  interface art ships and mod authors are not asked (ADR-0014). *Check: CI fails on game-style paths and formats
  (`.dds`, `.tex`, `.fnt`, `.swf`, `.bsa`).*
- **L3 SHOULD:** the install skin, "Game interface art" in Settings, is opt-in and off by default. It reads icons and
  fonts from the player's own install at run time through a Component's archive reader, caches them in the user's
  data folder, redistributes nothing, and falls back to the bundled set when anything is missing, changed or
  unreadable; the Doctor says so. It replaces icons and fonts only, never colours, layout or contrast. Screenshots in
  support bundles use the bundled set.
- **L4 MUST:** bundled fonts are OFL; the game's own typefaces are never bundled.
