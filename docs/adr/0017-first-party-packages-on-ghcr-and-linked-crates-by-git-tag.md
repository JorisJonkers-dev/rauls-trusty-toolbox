---
status: accepted
date: 2026-11-17
---
# First-party packages are published on ghcr.io; linked crates stay git tag dependencies

Amended by [ADR-0018](0018-the-submodule-pins-are-the-set.md): the "Yank" rule below names a `known-good.toml` `withdrawn` entry; that file is removed, so a withdrawal can no longer be an entry there. How a withdrawal is recorded is not decided by this note.

Programs are published as OCI artifacts on ghcr.io. Libraries that `raul` links are not published to any registry:
`raul` keeps consuming them as Cargo git dependencies pinned by tag, as it does for loot-rs today (ADR-0015).

GitHub Packages has no Cargo registry, and Cargo cannot read crates from an OCI registry, so one home cannot serve
both kinds. ghcr.io needs no new service and takes each repository's own token. A Cargo git dependency needs no registry.

## Programs

- **What**: the nv-rs engine package for each target, for people who run the engine without `raul`, and the CLI of
  nvmod, lodwright-rs, ttwright-rs and loot-rs for standalone use. `raul` reads none of them (amended 2026-11-20: the
  engine program and its pin in `raul` are removed; the engine is linked, ADR-0016).
- **Name and version**: `ghcr.io/jorisjonkers-dev/<repo>:<version>-<target>`, named after the repository (for example
  `ghcr.io/jorisjonkers-dev/ttwright-rs`; amended 2026-11-18, the first releases used `<repo>-<program>`), where `<version>` is the
  Component's release tag and `<target>` a Rust target triple. One version per Component release, no other tags. Each
  artifact is one archive layer plus the licence texts and a link to the source tag.
- **Pin**: `supply.toml` records the manifest digest (`sha256:...`) and the archive SHA-256 per target. The trust
  anchor is not in that file: the expected signer identity (the Component's own release workflow on a `v*` tag of its
  own repository) is a constant in `raul`'s source, one per Component, so a changed `supply.toml` cannot widen who
  may sign. `raul` fetches by digest over the OCI distribution API with an anonymous pull token, checks the SHA-256,
  then fetches the Sigstore attestation bundle through the registry's referrers listing and verifies it in process:
  the Fulcio certificate chain, the Rekor inclusion proof and the signer identity. It refuses on any failure. The
  digest decides; a tag is only a label. No `oras` and no token are needed on a developer machine. CI reads the same
  way.
- **Publish**: the Component's release workflow, with the job's `GITHUB_TOKEN` (`packages: write`, `id-token: write`,
  `attestations: write`). No personal access token and no stored secret exist. The build attests its provenance with
  `actions/attest-build-provenance`, pushed to the registry. Each package is linked to its repository and set public
  once, by an organisation owner, because ghcr.io creates new packages private. That step happens before a package's
  first release is announced. From then on the release workflow ends with an anonymous pull of what it pushed, so a
  package that became private again fails the release instead of a developer's machine. The first push of a new
  package is exempt from that pull and is checked by hand.
- **Yank**: a published tag and digest are never moved or deleted. A bad release is withdrawn: `known-good.toml`
  gets a `withdrawn` entry with a reason, `raul` refuses to install a withdrawn digest, and a fixed patch release
  follows. An installed older `raul` learns of the entry only through its next release or the opt-in update check
  (ADR-0005), which carries the withdrawn list; a signed list fetched on every start is not part of this decision. Deleting is for legal cause only.

## Linked crates

- **Which**: the library crates of nvmod-rs, lodwright-rs and ttwright-rs that `raul` links, and loot-rs. Each carries
  `publish = false`, so none reaches crates.io by accident.
- **How**: `git` plus `tag`, resolved to the submodule through the root `[patch]` (ADR-0015). Cargo.lock holds the
  commit, and every build uses `--locked`, so the commit, not the tag, is what compiles. The existing check that the submodule pointer equals the tag stays, and each Component repository protects
  `v*` tags from being moved or deleted.
- **Read path**: CI and developers clone from github.com. Cargo's `global-min-publish-age` applies to registries, not
  to git, so the tag pin and the Renovate bump pull request are the control.

## Consequences

- No new service, secret or registry account. The cost is that `raul` compiles the libraries; speed comes from CI
  caches keyed on Cargo.lock, not from prebuilt libraries. Revisit if a clean build proves too slow.
- A tag is mutable in git unless protected; the protection and the lockfile commit are what make the pin firm.
- Yank by tombstone means an old binary can still be downloaded by digest; only `raul` refuses it.
- A Component needs a release workflow with attestation before the first Known-good set can name its digest.

## Considered options

- A Cargo registry we run or rent, read through the estate's `packages.jorisjonkers.dev` proxy: rejected, it adds a
  service, a publish token and an index to run for a few crates. Revisit if the Components gain outside users.
- crates.io: rejected, names are global and releases cannot be removed, for crates that exist to be linked here.
- Prebuilt library archives (rlib) as OCI artifacts: rejected, an rlib only fits one compiler version, target and
  feature set, and Cargo cannot consume it without build-script tricks.
- Programs as GitHub release assets: rejected, ghcr.io gives digest addressing and registry-stored attestations.
