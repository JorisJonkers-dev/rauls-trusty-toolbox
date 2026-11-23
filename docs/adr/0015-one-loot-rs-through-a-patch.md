---
status: accepted
date: 2026-11-16
---
# One loot-rs: the Toolbox patches nvmod's loot-rs dependency to its submodule

Amended by [ADR-0018](0018-the-submodule-pins-are-the-set.md): `known-good.toml` and `raul known-good check` are gone; the loot-rs submodule pin and the lock file are what fix the one loot-rs, and the test on the submodule's tag stays.

nvmod depends on loot-rs by git tag, and the Toolbox pins loot-rs as a submodule. The Toolbox's root `Cargo.toml`
patches the git source to `components/loot-rs`, so one loot-rs is built and the pin in `known-good.toml` is the one
that runs. A test fails when the submodule's tag and the tag nvmod names differ. loot-rs itself is not changed; it
remains the owner's own track.

## Consequences

- A loot-rs bump is one submodule move plus nvmod's tag, checked together by `raul known-good check`.
- `raul-bench` uses the same loot-rs as `raul`.
