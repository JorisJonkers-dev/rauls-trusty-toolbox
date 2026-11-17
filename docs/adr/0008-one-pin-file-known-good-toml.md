---
status: accepted, partly superseded by ADR-0016
date: 2026-11-16
---
# One pin file: `known-good.toml` decides every Component version

Superseded in part by [ADR-0016](0016-components-are-incorporated-libraries-called-in-process.md): `supply.toml` rows and generated wire Contract versions cover only the packages `raul` installs. Where they are published is [ADR-0017](0017-first-party-packages-on-ghcr-and-linked-crates-by-git-tag.md).

nvmod-rs's `tools.lock.toml` folds into the Toolbox. `known-good.toml` is the only place that decides which Component
versions belong together and which Contract versions each speaks; the Contract versions are generated from each
Component's `contracts --json` handshake, never typed by hand. `supply.toml`, generated and checked, holds for each
version a set names the per-platform asset URL, SHA-256, attestation identity and source tag. The submodule pointers
under `components/` are derived from the newest Known-good set, and a check fails when the pointers, `supply.toml`
and `known-good.toml` disagree.

A bump writes the new set as `candidate`; only `raul known-good check` and `raul e2e` promote it to `known-good`.

The Rolling channel does not break the rule: a newer engine build is offered only when every Contract version it
emits is one the current set's consumers accept, and which engine a player runs is that player's local state, not
a pin. This amends ADR-0005: the Toolbox's tool supply, not nvmod, verifies checksums and attestations.

## Consequences

- `raul known-good check` stops reading `tools.lock.toml` when the fold lands.
- Installers bundle the programs of the newest Known-good set, verified against `supply.toml`.

## Considered options

- Keep both files and cross-check them: rejected, two files that must agree are one file too many.
- One file with set membership and every artefact field: rejected, it mixes the release set with supply metadata.
