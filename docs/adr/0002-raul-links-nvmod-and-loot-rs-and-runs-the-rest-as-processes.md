---
status: accepted
date: 2026-11-16
---
# `raul` links nvmod and loot-rs; nv-rs, lodwright-rs and ttwright-rs run as processes

Amended by [ADR-0007](0007-orchestration-tool-supply-and-the-view-model-live-in-the-toolbox.md): `raul` links nvmod's
mod-management libraries; the orchestration that nvmod held, and the Runner, now live in the Toolbox.

`raul` depends on nvmod's library crates (and through them loot-rs) by path into the submodules, because nvmod
already holds the orchestration and a second copy here would drift. nv-rs, lodwright-rs and ttwright-rs are started
as separate programs through nvmod's Runner and speak versioned Contracts: their work is long-running over untrusted
input, a crash must not take the window down, and nv-rs's `[patch.crates-io]` and build profiles only apply at its
own workspace root, so linking it would silently change what gets built.

## Consequences

- The `raul` binary is GPL-3.0-only (nvmod and loot-rs are), so the repository is GPL-3.0-only.
- Shipping nv-rs (GPL-3.0-or-later) and lodwright-rs (MPL-2.0) beside it is an aggregate: each package carries every
  licence text and links each Component's exact source tag.

## Considered options

- Subprocess only (would allow a permissive licence): rejected, every screen would pay for parsing CLI output.
- Linking lodwright-rs and ttwright-rs too: rejected for crash isolation and resume of long installs.
