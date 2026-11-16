---
status: accepted
date: 2026-11-16
---
# The Toolbox repository is the product umbrella; nv-linux stays the workshop

The Toolbox pins the five Components as submodules, owns the GUI, the `raul` program, `known-good.toml`, the
end-to-end run and the release of installers. `compat/`, `e2e/` and the release gate move here from nv-linux, which
stays the owner's private workshop (MO2 state, harness, research). Product decisions and the canonical glossary live
in this repository from now on, so a public reader can see them; nv-linux keeps its ADR 0001 to 0013 as history, and
its 0004 (GUI home), 0007 (the user-facing release is nvmod) and 0009 (packaging) are marked superseded by the ADRs
here. The amendment to nv-linux ADR-0007 for a Rolling channel of nv-rs builds (nvmod-rs `docs/DECISIONS.md`,
2026-11-16) is kept unchanged.

## Consequences

- The only user-facing release is the Toolbox's `vX.Y.Z`; Component releases are inputs to a Known-good set.
- Releases happen on demand; only the newest is supported, and the previous one gets security and contract fixes only.
- Moves out of nv-linux are coordinated with the session running the nv-linux ticket move; whoever lands second rebases.
