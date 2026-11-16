---
status: accepted
date: 2026-11-16
---
# Producers own their Contracts; the Toolbox keeps the registry

A Component defines each schema it emits as `contracts/<name>.v<N>.json`, versions it by an integer
`schema_version`, publishes it as a release asset and answers `<tool> contracts --json` (whose envelope has its own
`schema_version`). The producer's published schema is authoritative. The Toolbox keeps the registry: which
Contracts exist, which versions each Known-good set speaks, and a snapshot of each such version, fetched at bump
time; a bump fails when a snapshot differs from the pinned release.

A consumer knows a newest version N, accepts N and N-1, and refuses N+1 with a plain reason. A snapshot is removed
when no supported set names it. Adding an optional field is not a new version; removing, renaming or retyping is.

Wire Contracts (what a program prints) and on-disk Contracts (what a program keeps: Instance, Profile, the setup
journal, settings) are separate classes. On-disk Contracts also define migration on open, a backup before migrating
and refusal of a newer version. A Contract with no registry row cannot be part of a Known-good set.

This replaces the compatibility matrix in the private workshop and nvmod-rs's vendored `contracts/` snapshots.

## Consequences

- The registry lists, among others, the engine's release manifest, capabilities, run options, run summary and bench
  summary; lodwright's plan, request, result, manifest, verify and events; loot-rs's sort and check; ttwright's
  result and events; nvmod's Instance and Profile schemas and mod update rows; and the Toolbox's own `raul-events`,
  `raul-plan`, `raul-journal`, `raul-doctor`, `raul-detect` and `raul-updates`.
