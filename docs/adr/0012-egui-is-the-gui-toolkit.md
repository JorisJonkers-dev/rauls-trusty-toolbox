---
status: accepted
date: 2026-11-16
---
# egui is the GUI toolkit; the look is readable dark with a light Fallout flavour

The GUI is built on egui. GPUI and the egui canary are dropped. This replaces the toolkit line of ADR-0003. The view
model stays toolkit-free (ADR-0007), so the toolkit is a rendering choice, not an architectural one.

The design that GPUI was chosen for leaned on glow, gradients and chamfered shapes. The settled design does not: it
is a neutral dark surface with hairlines, one teal accent, Raul's amber for the brand only, Inter bundled, hand-drawn
line icons, a dark theme plus a high-contrast variant, a sidebar of seven pages and full gamepad use
(`docs/REORG.md` section 11). For that design egui is enough. It is also stable and small, testable headless with
`egui_kittest`, and ttwright-rs already uses it. GPUI is about 560 crates on weekly breaking snapshots from one
maintainer.

## Consequences

- When the GUI moves (#44), only the toolkit-free shell and the view model move; the GPUI crates stay behind and are
  deleted with nvmod-rs's `gui/`.
- ADR-0004's reason for a nested GUI workspace (keeping GPUI's crates out of the main lock file) no longer holds.
  Whether `raul` keeps two workspaces or one with a `gui` feature is decided in #13 by measuring egui's lock-file
  footprint.
- The GUI smoke test and the page snapshots run headless in CI with no GPU.
- The UI rules (`docs/UI-RULES.md`) and the page fixtures are written for egui.

## Considered options

- Keep GPUI with egui as a fallback: rejected; it pays GPUI's supply-chain and churn costs for no remaining benefit.
- iced or Slint: not needed; egui covers the design and is already in the stack.
