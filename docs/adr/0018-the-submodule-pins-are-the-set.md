---
status: accepted
date: 2026-11-18
---
# The submodule pins and the lock files are the set of Component versions

raul pins each Component through its git submodule under `components/` and its Cargo git tag, the lock files fix the Rust crates, and CI tests that exact combination on every pull request, so a passing pipeline says the set works. A second file (`known-good.toml`) only repeated the pins and made every Component bump conflict with the next, so it is removed together with the `raul known-good` command. The version a person sees is `raul`'s own, read from the build. A Component bump changes the submodule pointer, the lock files and the one-line `components/<repo>.version` beside the pointer. That file is not a second set: the Component follower needs a tracked file to read the pinned release from, since it cannot read inside a submodule, `scripts/bump-component.sh` writes it, and CI fails when it differs from the release the pointer is at.

This supersedes ADR-0005 and ADR-0008.
