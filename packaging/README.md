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

`icons/raul.png` is a placeholder gear until a designed icon exists.

## Signing

Signing runs only when its secrets exist; otherwise the release notes say the build is unsigned.

| Platform | Secrets |
| --- | --- |
| macOS | `APPLE_CERTIFICATE` (base64 .p12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` |
| Windows | SignPath Foundation (applied for; decision 8 of docs/PLAN.md), wired in when the application is accepted |
