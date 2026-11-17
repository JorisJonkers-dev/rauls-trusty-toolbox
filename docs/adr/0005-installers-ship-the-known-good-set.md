---
status: accepted, partly superseded by ADR-0016
date: 2026-11-16
---
# Installers ship the Known-good set; later updates are opt-in

Superseded in part by [ADR-0016](0016-components-are-incorporated-libraries-called-in-process.md): installers ship the engine package, not lodwright-rs and ttwright-rs programs, which are linked.

Amended by [ADR-0008](0008-one-pin-file-known-good-toml.md) and [ADR-0011](0011-the-rolling-engine-channel.md): the
Toolbox's tool supply, not nvmod, verifies checksums and attestations, and the Rolling channel is the Toolbox's.

Each Toolbox installer contains the nv-rs, lodwright-rs and ttwright-rs binaries of the Known-good set it was built
from, so a fresh install works offline. Between Toolbox releases the GUI may offer a newer set that the Toolbox
published as known-good; the person chooses to fetch it, and nvmod verifies its checksum and attestation. nv-rs alone
may also move ahead of the set on the opt-in Rolling channel (nvmod-rs `docs/DECISIONS.md`, 2026-11-16): compatibility
check, side-by-side install, one-click return. Nothing updates without the person asking, and update checks carry no
identifier. This supersedes nv-linux ADR-0009's "tools are never bundled".
