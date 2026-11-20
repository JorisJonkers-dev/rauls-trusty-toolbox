---
status: accepted
date: 2026-11-16
---
# The Toolbox repository is the product umbrella; the private workshop stays private

The Toolbox pins the five Components as submodules, owns the GUI, the `raul` program, `known-good.toml`, the
end-to-end run and the release of installers. `compat/`, `e2e/` and the release gate move here from the owner's private
workshop, which stays private (MO2 state, harness, research). Product decisions and the canonical glossary live
in this repository from now on, so a public reader can see them; the workshop keeps its earlier decisions as history, and
those on the GUI home, on the user-facing release being nvmod and on packaging are marked superseded by the ADRs
here. The amendment to the earlier decision on the user-facing release for a Rolling channel of nv-rs builds (nvmod-rs `docs/DECISIONS.md`,
2026-11-16) is kept unchanged.

## Consequences

- The only user-facing release is the Toolbox's `vX.Y.Z`; Component releases are inputs to a Known-good set.
- Releases happen on demand; only the newest is supported, and the previous one gets security and contract fixes only.
- Moves out of the workshop are coordinated with the work that moves its tickets; whoever lands second rebases.
