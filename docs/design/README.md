# Design

- [`mockup.html`](mockup.html): an interactive, self-contained mockup of the GUI. Open it in a browser. It starts in
  the first-run wizard ("Skip setup" opens the app); Ctrl K opens the command palette; `#home`, `#mods`, `#order`,
  `#lod`, `#versions`, `#doctor` and `#prefs` open a page directly. Every name, path and number in it is invented,
  and it contains no game or mod files.
- [`../UI-RULES.md`](../UI-RULES.md): the rules every screen follows.
- Decisions: [ADR-0012](../adr/0012-egui-is-the-gui-toolkit.md) (egui),
  [ADR-0013](../adr/0013-the-gui-is-readable-first-with-a-light-fallout-flavour.md) (look and navigation),
  [ADR-0014](../adr/0014-only-own-and-open-assets-ship.md) (assets and the install skin), and
  [`REORG.md`](../REORG.md) sections 11 and 12.

The mockup is a design reference, not product code: the GUI is built on egui from the toolkit-free view model, and
the mockup's pages, states and words are what its fixtures and snapshots must match.
