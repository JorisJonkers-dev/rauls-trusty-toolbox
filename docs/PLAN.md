# Plan: Raul's tRusty Toolbox

> Note: the Known-good set described below was removed (ADR-0018); the submodule pins and the lock files are the set.

Status: **superseded in part by [REORG.md](REORG.md) (2026-11-16).** Where the two differ (who owns orchestration, tool
supply, the Runner and the view model; the order of work; the release order), REORG.md and ADRs 0007 to 0011 win.
From 2026-11-18 ADR-0019 decides who orchestrates: `raul` drives every flow and nvmod manages mods only; REORG.md
section 0 has the current order.
Original status: **Phase 0 reviewed; decisions recorded 2026-11-16** (section 9, and ADRs 0001 to 0006 in
[`docs/adr`](adr)). Terms follow [`GLOSSARY.md`](../GLOSSARY.md). Phase 1 starts after this plan is
merged, in the order of section 8.

Findings were read on 2026-11-16 from the component repositories (`origin` default branches and
release pages) and from the proposed ADRs and architecture review in `JorisJonkers-dev/nv-linux`
(`docs/adr/0003` to `0013`, `docs/architecture/estate-review.md`). Where a statement comes from a
document rather than from code, it says so.

## 1. Summary

- The toolbox is the **product umbrella**: one repository that pins the five components as
  submodules, owns the GUI, the `raul` command line, the known-good set and the release of
  installers for Linux, Windows and macOS.
- **Most of the orchestration already exists in nvmod-rs.** Its CLI already has `setup`, `ttw`,
  `sort`, `lod`, `install-nvrs`, `run`, `profile`, `conflicts`, `backup`, `saves`, `doctor` and a
  tool lock. `raul-core` should therefore be thin glue over nvmod's library crates, not a second
  implementation.
- **The GUI moves from nvmod-rs to the toolbox** (`nvmod-rs/gui/`, the GPUI binary plus the egui
  canary and the toolkit-free shell). This answers the owner's remark that some of nvmod-rs
  belongs here, and it is the separation ADR-0004 named as the trigger for a separate home ("when
  the GUI needs releases that nvmod does not").
- **nv-rs and lodwright-rs are not linked.** nvmod already talks to both over versioned process
  contracts (`nvmod-tools/src/lodwright.rs`, `capabilities.rs`). The toolbox builds their
  binaries from the pinned submodules and ships them beside `raul`. Linking nv-rs would also drop
  its `[patch.crates-io]` for `wgpu-hal` and its build profiles, which Cargo only honours at the
  workspace root (section 4.3).
- **Licence: GPL-3.0-only**, because `raul` links nvmod-rs and loot-rs (section 3, ADR-0002).
- **macOS is a native target**, at the owner's instruction (2026-11-16): Linux, Windows and macOS
  all native, no compatibility layers where avoidable. This supersedes the "macOS is not
  supported" line of ADR-0009 and needs work in nv-rs and in the GUI toolkit choice.
- **A fifth component, `ttwright-rs`**, was created on 2026-11-16 as this organisation's rewrite
  of [SulfurNitride/TTW_Linux_Installer](https://github.com/SulfurNitride/TTW_Linux_Installer)
  (Rust, GPL-3.0-only). Upstream history is kept, an `upstream` branch mirrors upstream `main`, and
  a sync workflow opens an issue listing commits to port (its `FORK.md`). It replaces the
  third-party TTW installer that nvmod's `tools.lock.toml` could not pin (no binary, checksum or
  attestation upstream).
- **Toolchain: Rust 1.100.0** (released 2026-11-09, taken after its 7-day cooldown) with Cargo's
  `[registry] global-min-publish-age = "7 days"` in `.cargo/config.toml`, stable since Cargo 1.100.
  The toolbox and ttwright-rs use it from their first commit; the four older components are on
  1.99.1 and get the bump as their own pull requests.
- Several requests in the brief conflict with proposed nv-linux ADRs. Section 5 lists each one with
  the decision taken; none is overridden silently.

## 2. Findings per component

| | nv-rs | nvmod-rs | loot-rs | lodwright-rs |
|---|---|---|---|---|
| Repository | `JorisJonkers-dev/nv-rs`, fork of `slaterain/nv-rs` (not a GitHub fork-network member) | `JorisJonkers-dev/nvmod-rs` | `JorisJonkers-dev/loot-rs` | `JorisJonkers-dev/lodwright-rs` |
| Default branch | `fork` (`main` mirrors upstream) | `main` | `main` | `main` |
| Latest release | `v0.4.0` (2026-11-16) | `v0.5.0` (2026-11-15) | `v0.5.0` (2026-11-15) | `v0.2.0` (2026-11-15) |
| Licence | GPL-3.0-or-later (upstream's) + `LEGAL.md`, `FORK.md` | GPL-3.0-only | GPL-3.0-only, `NOTICE.md` (MIT `plugin.rs` from fnvorder) | MPL-2.0, no "Incompatible With Secondary Licenses" notice found in `crates/` |
| Edition / resolver | 2021 / 2 | 2024 / 3 | 2024 (single crate) | 2024 / 3 |
| `rust-version` | `1.75` (upstream's declared value) | `1.99` | `1.99` | `1.99` |
| Toolchain pin | `rust-toolchain.toml` 1.99.1 | `rust-toolchain.toml` 1.99.1 | `mise.toml` 1.99.1 (no `rust-toolchain.toml`) | `rust-toolchain.toml` 1.99.1 |
| Layout | 41 crates (engine, d3d9-wgpu, esm, bsa, host, play-patch, viewer, …), `vendor/` excluded, `[patch.crates-io] wgpu-hal`, custom `play` profile | 26 crates + nested workspaces `gui/` and `fuzz/` | one crate (`src/`) + `fuzz/` | 9 crates + `fuzz/` |
| GUI | none (engine window only) | `gui/`: `nvmod-gui-gpui` (`gpui-pre =0.3.11`, `gpui-kit =0.9.0`), `nvmod-gui-egui` canary (`eframe =0.37.0`), `nvmod-gui-shell`; toolkit-free `nvmod-ui-model`, `nvmod-ui-service` in the main workspace | none | none |
| Release assets | Windows setup `.exe`, portable `.zip`, Linux `-dev.tar.gz`, `SHA256SUMS`, capabilities and release manifest JSON | release-please; `release-assets.yml` | release-please; consumed as a git tag dependency | `lodwright-x86_64-unknown-linux-gnu`, `SHA256SUMS`, plan/result schemas |
| Platforms today | Windows preview; Linux dev build; macOS none | Linux-first; Windows paths handled | portable | Linux binary only |
| CI | `fork-*.yml` (ci, build, fuzz, package-windows, release, security, sync), one required check | `pipeline.yml` (Pipeline Complete), `gui.yml`, fuzz, security | `ci.yml`, fuzz, mutants, masterlist, advisories | `pipeline.yml`, fuzz, security |
| Benchmarks | none (`criterion`/`divan` absent everywhere) | none | none | none |

**ttwright-rs** (`JorisJonkers-dev/ttwright-rs`, default branch `main`, mirror branch
`upstream`): one crate `ttw_installer` 0.2.0, edition 2021, GPL-3.0-only, binaries `mpi_installer`
(CLI, clap) and `mpi_installer_gui` (eframe 0.33), modules for BSA building and extraction (`ba2`),
MPI index and extraction, xdelta patching (`oxidelta`), audio re-encoding (symphonia, vorbis_rs,
mp3lame), game detection and file verification. On Rust 1.100.0 it passes `cargo fmt --check`,
`clippy --all-targets -D warnings` and 19 unit tests (2026-11-16). Building needs `make`, a C
compiler and `pkg-config` for `mp3lame-sys`. Upstream releases Linux and Windows only.

Other facts that shape the plan:

- **nvmod links loot-rs** by git tag (`loot-rs = { git = …, tag = "v0.5.0" }`) and records it in
  `tools.lock.toml` as `kind = "linked"`. Everything else in the lock is fetched or third-party.
- **nv-rs fork rules.** Only the paths in `.fork/overlay-paths` may differ from upstream, and
  `scripts/fork/verify-sync.sh` checks it. The toolbox must never ask nv-rs to change an
  upstream file (for example its `rust-version`) except as a pull request to upstream.
- **nv-linux already has the umbrella machinery:** submodules under `projects/`,
  `compat/matrix.toml` (known-good sets with contract versions), `compat/release_gate.py`,
  `e2e/run.py` and `release-gate.yml`. The brief's `compat.toml` is the same idea; here it is `known-good.toml`.
- **The current nv-linux submodule pointers are stale** (nv-rs at `v0.1.0~7`, lodwright at
  `v0.1.0~3`), so the first toolbox pin is a fresh known-good set, not a copy of nv-linux's.

## 3. Licences

| Linked into `raul` | Licence | Effect |
|---|---|---|
| nvmod-rs crates | GPL-3.0-only | `raul` must be GPL-3.0-only |
| loot-rs (through nvmod) | GPL-3.0-only | same |
| GPUI / gpui-kit, egui | Apache-2.0 / MIT OR Apache-2.0 | compatible one way with GPL-3.0 |

| Shipped beside `raul`, not linked | Licence | Effect |
|---|---|---|
| nv-rs binaries | GPL-3.0-or-later | separate program in an aggregate; source offer and licence text in the package |
| lodwright binary | GPL-3.0-only (MPL-2.0 up to 0.2.x, ADR-0002) | separate program in an aggregate; source offer and licence text in the package |

Rules the plan follows (from ADR-0008, proposed, plus the brief):

1. **Recommended licence for the toolbox: GPL-3.0-only.** A permissive licence is possible only if
   `raul` links none of nvmod or loot-rs and drives everything as subprocesses; that is a real
   option (section 9, question 1) but costs the shared view-model crates.
2. Keep nv-rs's `LEGAL.md` disclaimer and `FORK.md` credit; the README's "Origins" section credits
   slaterain/nv-rs and LOOT. Every release carries a generated third-party notices file
   (`cargo about` or `cargo deny list`) and links the exact source tag of every GPL component.
3. Never redistributed: the LOOT masterlist (fetched at run time, CC0 data), MPI packages (TTW data),
   xNVSE and JIP LN DLLs, any game file.
4. Fonts and icons are checked before shipping (OFL or permissive); Lucide is ISC.
5. Not legal advice. The MPL-2.0 aggregate reading and the secondary-licence point are taken from
   ADR-0008, which marks them unverified.

## 4. Proposed architecture

### 4.1 Layout

```
rauls-trusty-toolbox/
├── components/            git submodules, pinned to release tags
│   ├── nv-rs/             branch "fork"
│   ├── nvmod-rs/
│   ├── lodwright-rs/
│   ├── loot-rs/
│   └── ttwright-rs/
├── crates/
│   ├── raul-core/         thin orchestration over nvmod's library crates: game detection, profiles, pipeline
│   ├── raul-cli/          CLI-only `raul`: every GUI action, `--json` everywhere
│   ├── raul-gui/          nested workspace, own Cargo.lock: the GUI (moved from nvmod-rs/gui) and the
│   │                      release `raul` = same CLI + GUI (ADR-0004)
│   └── raul-bench/        `raul bench`: suite runner and JSON + Markdown report
├── packaging/             cargo-packager config, icons, .desktop, Info.plist, NSIS/WiX
├── known-good.toml        the chain of Known-good sets (schema of nv-linux compat/matrix.toml)
├── rust-toolchain.toml    1.100.0 (+ .cargo/config.toml: 7-day registry cooldown)
└── Cargo.toml             workspace of crates/* (raul-gui excluded, see 4.4)
```

### 4.2 What links what

```
raul-gui ──(Service: reads in process)──► raul-core ──► nvmod-* crates ──► loot-rs
raul-gui ──(Service: writes as `raul` subprocess, progress events)──► raul-cli
raul-cli ───────────────────────────────► raul-core
raul-core ──(process + versioned contract)──► lodwright, nv-rs, ttwright
```

- `raul-core` depends on nvmod's crates (`nvmod-core`, `nvmod-instance`, `nvmod-loadorder`,
  `nvmod-tools`, `nvmod-ui-model`, `nvmod-ui-service`, …) by path into `components/nvmod-rs`.
- The GUI keeps nvmod's `Service` seam with a hybrid implementation (ADR-0003): reads in
  process through `raul-core`; mod installs, TTW, LOD and applying a sort as a `raul` subprocess,
  so a crash or a hostile mod archive in an install cannot take down the window.
- ttwright runs as a process like lodwright, through nvmod's existing TTW adapter
  (`nvmod-tools/src/ttw.rs`): a TTW install runs for a long time over untrusted archives, so crash
  isolation and resume belong at the process boundary. The rewrite adds a versioned result
  contract (`ttw-installer-result`, already named in nv-linux `compat/matrix.toml`) and progress
  events on stdout.
- If `raul-core` turns out to be only re-exports, it is dropped and `raul-cli` wraps `nvmod-cli`.
  Do not grow a second copy of nvmod's logic here.

### 4.3 Workspace mechanics (to verify in Phase 1)

- Root `Cargo.toml`: `members = ["crates/*"]`, `exclude = ["components", "crates/raul-gui"]`.
  Each component keeps its own `[workspace]`; Cargo finds the nearest workspace root for a path
  dependency, so the nvmod crates resolve `workspace = true` against
  `components/nvmod-rs/Cargo.toml`. Phase 1 confirms `cargo build` gives no "multiple workspace
  roots" error.
- **One loot-rs.** nvmod depends on loot-rs by git tag. The toolbox adds
  `[patch."https://github.com/JorisJonkers-dev/loot-rs"] loot-rs = { path = "components/loot-rs" }`
  so the submodule pin and nvmod's pin are the same source. A test fails if
  `components/loot-rs`'s tag differs from nvmod's `tools.lock.toml` entry.
- **`[patch]` and profiles only apply at the root.** nv-rs's `wgpu-hal` patch and `play` profile
  would be silently lost if nv-rs were a path dependency. Another reason nv-rs is built in its own
  workspace (`cargo build --manifest-path components/nv-rs/Cargo.toml --release --locked`) with
  its own lock file, not linked.
- **Lock files.** The brief asks for one shared `Cargo.lock`. That holds for the umbrella
  workspace (`raul-core`, `raul-cli`, `raul-bench` and the nvmod and loot-rs crates they pull in).
  nv-rs, lodwright and the GUI keep their own lock files, because each is its own workspace. The
  release is still reproducible: every lock file is pinned by the submodule SHA and built with
  `--locked`.

### 4.4 GUI

Decided (ADR-0003): GPUI, the framework already chosen and measured in nv-linux's
`docs/research/gui-toolkit-spike.md`, with the egui canary still painting the same view model.
The trade-offs that were weighed:

| | GPUI (current) | egui (current canary) | Slint / iced |
|---|---|---|---|
| macOS native | yes (Metal; Zed's home platform) | yes | yes |
| Windows native | yes (recent) | yes | yes |
| Linux | Wayland/X11 + Vulkan | any GL/wgpu | any |
| Headless smoke test | hard: needs a GPU or lavapipe under xvfb | easy: `egui_kittest` | Slint has a testing backend |
| Supply chain | ~560 crates, weekly breaking `gpui-pre` snapshots, one maintainer | small, stable | medium |
| Look | best fit for the Mojave Dusk design | plainer | good |

The GUI smoke test runs on Linux with Mesa lavapipe under xvfb; if lavapipe cannot start GPUI, the smoke test runs the egui
canary and the GPUI build is a compile-only check. The GUI stays a nested workspace with its own
lock file and `cargo deny` allow-list, as in nvmod-rs today. All on-screen text is written in
Raul's voice; every error also carries a plain detail line.

### 4.5 Native on all three platforms

Decided (ADR-0006). Baselines: macOS 13+ universal, Windows 10 22H2 or 11 x64, Linux glibc 2.35+
on Wayland and X11. Until nv-rs runs natively on a platform, Launch is disabled there with a plain
message; setup, mods, sorting and LOD still work.

- **Linux:** nv-rs native (Vulkan through wgpu), never Proton; Proton prefixes remain a
  detection source for the game files only (Steam Deck included).
- **Windows:** native.
- **macOS:** nv-rs has no macOS build today. Its renderer is `d3d9-wgpu`, so Metal through wgpu
  is the natural path; the work is in nv-rs (platform layer, windowing, audio, file paths) and
  goes there as fork pull requests, offered upstream where upstream wants them. Universal
  binaries need both `aarch64-apple-darwin` and `x86_64-apple-darwin` builds joined with `lipo`.
- Game detection on macOS covers Steam (`~/Library/Application Support/Steam`) and GOG; the game
  itself is a Windows game, so the user points at Windows game files copied or downloaded through
  Steam's or GOG's own tools. No CrossOver or Wine.

## 5. Conflicts with the proposed nv-linux ADRs

| Brief asks | ADR says (proposed) | Decided (2026-11-16) |
|---|---|---|
| Umbrella owns the product release (`raul`) | ADR-0007: the only user-facing release is nvmod | Superseded by ADR-0001: the user-facing release is `raul`; nvmod stays a released Component. The Rolling channel amendment to ADR-0007 (nvmod-rs `docs/DECISIONS.md`) is kept |
| GUI in the umbrella | ADR-0004: GUI is `nvmod-rs/gui/` | Moved (ADR-0003) |
| Components as path dependencies | ADR-0003/0005: tools meet over contracts and pinned releases, one runner for subprocesses | Link only nvmod and loot-rs; run nv-rs, lodwright-rs and ttwright-rs as processes (ADR-0002) |
| Ship nv-rs and lodwright in the installer | ADR-0009: tools are never bundled, fetched by lock | Bundle the Known-good set; opt-in fetch of newer known-good sets between releases (ADR-0005) |
| macOS universal DMG | ADR-0009: macOS not supported | Superseded by ADR-0006 |
| `compat.toml` in the umbrella | `compat/matrix.toml` in nv-linux | `known-good.toml` here; `compat/`, `e2e/` and `release-gate.yml` move here; nv-linux stays the workshop (ADR-0001) |
| Shared crate for duplicated parsing | ADR-0004: no shared format crate yet; golden vectors by copy | Agree with ADR-0004 for now (section 6) |
| Built-in update check | ADR-0010: pull-only, explicit, no identifier | Follow ADR-0010: opt-in, GitHub Releases API, no identifier sent |

The superseding decisions are ADRs 0001 to 0006 here; a pull request to nv-linux marks its 0004,
0007 and 0009 superseded (coordinated with the nv-linux ticket move, section 8).

## 6. Overlap and dependency alignment

| Concern | Where it exists | Plan |
|---|---|---|
| Plugin header (TES4) reading | `loot-rs/src/plugin.rs` (MIT, from fnvorder), `lodwright-plugin-meta`, nv-rs `crates/esm`, `nvmod-formats/plugins.rs`, nv-linux `tools/fnvorder` | No shared crate yet (ADR-0004). Phase 4 adds the golden vectors to a check here. If extracted later: a leaf crate, MIT OR Apache-2.0, no code from loot-rs or nv-rs, in its own repository |
| BSA reading | nv-rs `crates/bsa`, `nvmod-bsa`, `lodwright-assets/bsa.rs` | same |
| Executable version | `nvmod-pe`, `loot-rs/src/exe_version.rs` | same |
| Edition | nv-rs 2021, rest 2024 | leave nv-rs (upstream's) |
| `rust-version` | nv-rs `1.75`, rest `1.99` | the toolbox declares `1.100`; nv-rs's value is upstream's and its real minimum is higher. Offer upstream an accurate `rust-version`; do not patch it here |
| Toolchain | 1.99.1 in the four older components (loot-rs via `mise.toml` only); 1.100.0 in the toolbox and ttwright-rs | bump each component to `rust-toolchain.toml` 1.100.0 plus `.cargo/config.toml` `global-min-publish-age = "7 days"` by its own pull request. nv-rs's `rust-toolchain.toml` is upstream's file, so there the bump is an overlay change only if `.fork/overlay-paths` allows it, otherwise an upstream pull request |
| BSA writing and reading in ttwright | `ttwright-rs/src/services/archive/bsa` (`ba2` crate) | a sixth BSA implementation; ttwright keeps `ba2` (published, maintained) and the others are compared against it in the golden-vector check |
| Duplicate crate versions | unknown until one lock exists | Phase 1 runs `cargo tree -d` and `cargo deny check bans` and reports |

## 7. CI, sync, benchmarks and releases (Phases 3 to 5, summarised)

- **Required check stays `Pipeline Complete`** (estate rule). The OS matrix runs as further jobs
  gated from it with `needs:` and a `success` check, so the ruleset never changes.
- **PR CI** on ubuntu, windows and macos-14 (arm64): `cargo fmt --check`, `clippy -D warnings`,
  `cargo nextest`, doc tests, with `Swatinem/rust-cache`. Plus `cargo deny`, `cargo audit`, an MSRV
  job (1.100) and the submodule check: every pointer must be reachable from its repository's
  default branch (`fork` for nv-rs) or be a tag.
- **Ten-minute target.** Achievable for the umbrella workspace with a warm cache. **Not achievable
  if nv-rs is built on every pull request**: nv-rs is 41 crates and its release build is several
  minutes per OS even when warm. So nv-rs and lodwright are built only when their pointer
  changes, in submodule-bump pull requests and in releases; ordinary pull requests use the release
  binaries named by `known-good.toml`.
- **Component sync.** Recommended: each component's release workflow sends a `repository_dispatch`
  to the toolbox (token from the estate release GitHub App), and a workflow here opens a pull
  request bumping that submodule to the new tag and adding a candidate set to `known-good.toml`; full
  CI and the end-to-end run decide whether it becomes known-good. A `workflow_dispatch` input does
  the same by hand. Renovate's git-submodules manager is not chosen: it follows branch heads, not
  release tags, and estate crons fire unreliably. One pull request moves one component (ADR-0007
  chain rule).
- **Versioning:** release-please, as every component already uses; not release-plz.
- **Benchmarks:** none exist. Add `divan` micro-benchmarks in each component (plugin header
  parse, BSA index read, sort, LOD steps) as component pull requests; `raul bench` runs the suite
  on synthetic fixtures built by `nvmod-testkit` and `lodwright-testkit` (small, medium, TTW-sized
  load orders) and writes JSON + Markdown. Frame-time capture along a fixed camera path depends on
  nv-rs gaining a scripted camera and a frame-time log (nv-rs has frame-rate code in
  `crates/d3d9-wgpu/src/frame_rate.rs`, no capture yet); that is an nv-rs proposal. CI runs benches
  on pull requests labelled `bench` and posts a comment; history through
  `github-action-benchmark` on `gh-pages` (free). Regressions warn, never fail, on shared runners.
- **Releases:** `cargo-packager` for AppImage, `.tar.gz`, `.deb`, NSIS `.exe`, portable `.zip`
  and the universal `.app` in a `.dmg`. `SHA256SUMS`, CycloneDX SBOM (`cargo cyclonedx`),
  `actions/attest-build-provenance`. Signing only when the secrets exist; otherwise unsigned, and
  the release notes say so; Windows signing is applied for through SignPath Foundation, Apple
  signing is decided later. Release notes list the component versions from `known-good.toml`. Each
  artefact is smoke-tested (`raul --version`, GUI start under xvfb on Linux).

## 8. Migration plan

Each step is one pull request with one concern. Steps in a component repository are marked.

**Phase 1: structure**
1. Add the five submodules (nv-rs on `fork`) at their latest release tags,
   and `CONTRIBUTING.md`'s submodule workflow.
2. Add the root `Cargo.toml` with an empty `raul-cli` printing `--version` plus the component
   versions; prove no "multiple workspace roots" error; add the loot-rs `[patch]`.
3. Add `known-good.toml` with the first known-good set and its check (ported from nv-linux `compat/`).
4. *(nvmod-rs, loot-rs, lodwright-rs, nv-rs)* Rust 1.100.0 and the 7-day registry cooldown, one
   pull request each.
5. *(ttwright-rs)* Restructure into a workspace (`ttwright-core`, `ttwright-cli`, `ttwright-contract`,
   `ttwright-testkit`; the eframe GUI is dropped in favour of `raul-gui`), add the result contract,
   fuzz targets for the MPI and BSA readers, the three-OS matrix, and attested release assets.
   Then nvmod's `tools.lock.toml` pins it.

**Phase 2: GUI and CLI**
6. `raul-core` over nvmod crates: detection, profiles; `raul-cli` subcommands mapping to nvmod's.
7. *(nvmod-rs)* Expose what `raul-core` needs as library API where only the CLI has it today.
8. Move `nvmod-rs/gui` into `crates/raul-gui` with history (`git subtree split` in nvmod-rs, then
   added here); *(nvmod-rs)* remove `gui/` afterwards. Rebrand the binary `raul-gui`.
9. The first-run wizard copy in Raul's voice; detection, TTW, mods, load order, LOD, launch screens.

**Phase 3: CI and sync**
10. OS matrix, deny, audit, MSRV, submodule check.
11. Bump workflow here; *(each component)* dispatch on release.

**Phase 4: benchmarks**
12. *(each component)* `divan` benches.
13. `raul-bench` and the bench workflow.

**Phase 5: releases**
14. cargo-packager config and the release workflow, unsigned.
15. Signing when secrets exist.
16. Opt-in update check.

**Phase 6: docs**
17. `docs/ARCHITECTURE.md`, `CODE_OF_CONDUCT.md`, final README with screenshots.

**In parallel, in nv-rs:** native macOS (Metal), native Linux completion, scripted camera and
frame-time log. These are the long poles; everything above works without them except the macOS
launch step and in-engine benchmarks.

### Order across repositories (agreed with the nv-rs program coordinator, 2026-11-16)

- **nvmod-rs** (step 7, the `gui/` move in step 8, the toolchain in step 4): only after the nvmod-rs
  0.6.0 Release test passes and nvmod-rs #65 is merged; nvmod-rs `main` is frozen for structural changes
  from this side until then.
- **nv-rs and lodwright-rs** (toolchain, `divan` benches): only after their 0.5.0 and 0.3.0 Release
  tests. In nv-rs, every change to an upstream-owned file needs an allowed-divergence entry and a
  `DIVERGENCE.md` row, or Verify sync fails; the default branch is `fork`.
- **nv-linux moves** (`compat/`, `e2e/`, release gate, glossary pointer, superseding ADRs): agreed
  with the coordinator first, because its ticket-move pull request rewrites the same documents.
- **loot-rs** is owned by the owner's own agent: the toolchain step and the `[patch]` to
  `components/loot-rs` wait for the owner.
- No benchmark runs from this side while the coordinator holds the nv-rs bench lock.
- Owner decisions carried over: the Nexus milestone comes before the Windows GUI job; the Engine
  extension template repository is `JorisJonkers-dev/nv-rs-extension-template` (manifests at
  `Data/NVRS/Extensions/<id>/nvext.toml`); the coverage gate is 70% of
  changed lines on fork-owned crates.

### Policy from nv-linux ADR 0014, points 17 to 33 (2026-11-16)

The engineering policy of the stack is kept in nv-linux `docs/adr/0014-fork-positioning-and-engineering-policy.md`.
The points that bear on the Toolbox:

| Point | What the Toolbox follows |
| --- | --- |
| 17 | Component releases pass a local Release test before their release PR merges (nv-rs#195, nvmod-rs#70, lodwright-rs#82); a bump here waits for that release |
| 18 | nv-rs blocks merges below 70% coverage of changed lines in fork-owned crates (nv-rs#194) |
| 21 to 23 | Shader packs: one reviewed seam line in nv-rs, vkd3d-shader loaded at run time; the shader-hash profile never leaves the owner's machine |
| 24 to 27 | nv-rs performance work (draw thread, render thread A/B, prepared bench saves, fog and reflections) stays in nv-rs; `raul bench` frame-time capture waits for it |
| 28 | The Nexus milestone comes before the Windows GUI job (nv-linux #328), which moves here with the GUI |
| 29, 30 | "Engine extension" for native code in nv-rs, template `nv-rs-extension-template`, manifests at `Data/NVRS/Extensions/<id>/nvext.toml`; opcode blocks are fork-only |
| 31 | Discord presence is nv-rs's, opt-in, through the games' existing applications (nv-rs#187) |
| 32 | The hardware checklist (nv-linux #302) is the owner's to run |
| 33 | "Component" means a pinned project; the nv-rs CI split unit is a "CI unit" (as in `GLOSSARY.md`) |

The lodwright result and event Contracts gained `up_to_date` per block (lodwright-rs #84, after v0.2.0); the GUI's
LOD progress reads it once a set with that release is known-good. Extender readiness shown in the GUI comes from
nvmod's readiness report (nv-linux #265) and opcode scan (#97).

## 9. Decisions

Settled with the owner on 2026-11-16. The ones that are hard to reverse are ADRs.

| # | Decision | Record |
|---|---|---|
| 1 | `raul` links nvmod and loot-rs; nv-rs, lodwright-rs, ttwright-rs run as processes | ADR-0002 |
| 2 | The GUI moves from nvmod-rs into the Toolbox; nvmod-rs keeps `nvmod-ui-model` and `nvmod-ui-service` | ADR-0003 |
| 3 | `compat/`, `e2e/` and the release gate move here; nv-linux stays the private workshop | ADR-0001 |
| 4 | Installers bundle the Known-good set | ADR-0005 |
| 5 | Licence GPL-3.0-only | `LICENSE`, ADR-0002 |
| 6 | GPUI with the egui canary; Linux smoke test under xvfb + lavapipe, egui as fallback | ADR-0003 |
| 7 | Hybrid service seam: reads in process, risky writes as a `raul` subprocess | ADR-0003 |
| 8 | Unsigned for now with a clear note; apply to SignPath Foundation; Apple later | ADR-0006 |
| 9 | Decisions and glossary live in the Toolbox; nv-linux 0004, 0007, 0009 marked superseded | ADR-0001 |
| 10 | Newer sets between releases: opt-in, known-good only, verified; nv-rs Rolling channel kept | ADR-0005 |
| 11 | Launch only where nv-rs is native; no Proton, Wine or CrossOver | ADR-0006 |
| 12 | Benchmark history: `github-action-benchmark` on `gh-pages`, warnings only | section 7 |
| 13 | One `raul` binary: no arguments opens the GUI, a command runs the CLI | ADR-0004 |
| 14 | The glossary moves here and is canonical; "Component" means one of the five pinned projects | `GLOSSARY.md` |
| 15 | Releases on demand; only the latest is supported | ADR-0001 |
| 16 | Raul's voice across the whole GUI, plain detail under every error; CLI and logs plain | ADR-0003 |
| 17 | macOS 13+ universal, Windows 10 22H2/11 x64, Linux glibc 2.35+ (Wayland and X11) | ADR-0006 |
| 18 | CLI-only `raul` from the main workspace for CI, benches and headless use | ADR-0004 |
| 19 | The set file is `known-good.toml` | `GLOSSARY.md` |
| 20 | Two workspaces build the same `raul`; a test keeps their commands equal | ADR-0004 |

Still open: whether nvmod-rs `docs/DECISIONS.md`'s 2026-11-16 row "the GUI shells out to the nvmod
CLI with `--json`" is amended to point at ADR-0003 once the GUI has moved (an nvmod-rs pull request,
after 0.6.0).
