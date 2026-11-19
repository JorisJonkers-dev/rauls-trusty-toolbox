# Packaging

Installers are built by `.github/workflows/release.yml` with
[cargo-packager](https://github.com/crabnebula-dev/cargo-packager); its configuration is the
`[package.metadata.packager]` table of `crates/raul-cli/Cargo.toml`.

| Platform | Artefacts |
| --- | --- |
| Linux x64 | `.AppImage`, `.deb`, `.tar.gz` |
| Windows x64 | NSIS `-setup.exe` (per user, no administrator rights), portable `.zip` |
| macOS 13+ | universal (arm64 and x86_64) `.app` in a `.dmg` |

Each package holds `raul` with nvmod, loot-rs, lodwright-rs and ttwright-rs linked in (ADR-0016), the engine's
packaged nv-rs build under `components/` where one exists for that platform (until it is linked too),
`THIRD-PARTY-NOTICES.md` and the licence texts. No Component program other than nv-rs is in a package, and the
smoke tests fail if one is. The release notes list the parts with their versions and every file with its SHA-256
(`scripts/release-contents.sh`). No game file is ever included.

## The packaged engine

On Linux and Windows the engine's nv-rs build ships under `components/nv-rs/` with `FILES.sha256`, the checksum list of
its files, which `release.yml` writes after checking the archive against nv-rs's own `SHA256SUMS`. The setup wizard's
Engine step and `raul install-nv-rs` look for that folder next to the running `raul` (beside it for the tarball, the
portable zip and the NSIS install, under `lib/` for the deb and the AppImage, under `Resources/` in the macOS bundle), check
every file against the list, copy the viewer into the Instance and run its check. **Nothing is downloaded**: a copy
without the folder (macOS, which has no engine build yet, or a development build) stops with an instruction to
install raul again. `--source <dir>` still builds an nv-rs workspace, for developers.

What stays until the play mode lands (#260, nv-rs#353, engine link T2 #315): the engine is still a separate program in
the package and in the Instance (`nv-rs/bin/nv-viewer`), because the game is still started as that program. When raul
starts the game from its own binary in a play mode, the packaged folder, `FILES.sha256`, the Engine step and
`raul install-nv-rs` are deleted (see the note at the top of `crates/raul-core/src/setup/engine.rs`).
The follow-up to check once a release is built: that each package lays `components/nv-rs/` where the search above
looks (the release smoke test fails when `FILES.sha256` is missing from a package, not when it is in an unexpected place).

The portable archives (`.tar.gz`, portable `.zip`) also hold `INSTALLED-FILES.sha256`: one checksum line for every
other file in the package (`scripts/installed-files.sh`, written by `release.yml` after the last file is in place).
`raul update` checks an unpacked update against it before it installs anything (ADR-0020), and the release smoke test
checks every portable package with the same script. The AppImage, the deb and the NSIS installer do not carry one.

`icons/raul.png` is a placeholder gear until a designed icon exists.

## Signing

Signing runs only when its secrets exist; otherwise the release notes say the build is unsigned.

| Platform | Secrets |
| --- | --- |
| macOS | `APPLE_CERTIFICATE` (base64 .p12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` |
| Windows | SignPath Foundation (applied for; decision 8 of docs/PLAN.md), wired in when the application is accepted |
