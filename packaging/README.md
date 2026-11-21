# Packaging

Installers are built by `.github/workflows/release.yml` with
[cargo-packager](https://github.com/crabnebula-dev/cargo-packager); its configuration is the
`[package.metadata.packager]` table of `crates/raul-cli/Cargo.toml`.

| Platform | Artefacts |
| --- | --- |
| Linux x64 | `.AppImage`, `.deb`, `.tar.gz` |
| Windows x64 | NSIS `-setup.exe` (per user, no administrator rights), portable `.zip` |
| macOS 13+ | universal (arm64 and x86_64) `.app` in a `.dmg` |

Each package holds `raul` with nvmod, loot-rs, lodwright-rs, ttwright-rs and the engine (nv-rs) linked in (ADR-0016),
`THIRD-PARTY-NOTICES.md` and the licence texts. No Component program is in a package, and the smoke tests fail if one
is. The release notes list the parts with their versions and every file with its SHA-256
(`scripts/release-contents.sh`). No game file is ever included.

## No separate engine

The engine is linked into `raul` (feature `engine`, on every system, both macOS processors included), so no package
carries an engine program: no `components/nv-rs/` folder, no `nv-viewer`, no `FILES.sha256`. `raul run` starts the game
as `raul --play-mode`, a child process of raul's own binary (ADR-0016). The release smoke test fails when any package
holds an `nv-viewer` file or a `components/nv-rs` folder. Nothing is downloaded on first run.

The portable archives (`.tar.gz`, portable `.zip`) also hold `INSTALLED-FILES.sha256`: one checksum line for every
other file in the package (`scripts/installed-files.sh`, written by `release.yml` after the last file is in place).
`raul update` checks an unpacked update against it before it installs anything (ADR-0020), and the release smoke test
checks every portable package with the same script. The AppImage, the deb and the NSIS installer do not carry one.

The `.deb` is named `raul`, has a Maintainer and declares the libraries the window and the engine open at run time
(`[package.metadata.packager.deb]` in `crates/raul-cli/Cargo.toml`). `scripts/check-deb-install.sh` installs it on plain
Debian and Ubuntu images in the release workflow and starts `raul --version` and the window there. The AppImage and the
`.tar.gz` carry no system libraries; the README lists them. `scripts/check-release-assets.sh` fails a release or a dry run
that lacks an installer.

`icons/raul.png` is a placeholder gear until a designed icon exists.

## Signing

Signing runs only when its secrets exist; otherwise the release notes say the build is unsigned.

| Platform | Secrets |
| --- | --- |
| macOS | `APPLE_CERTIFICATE` (base64 .p12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` |
| Windows | SignPath Foundation (applied for; decision 8 of docs/PLAN.md), wired in when the application is accepted |
