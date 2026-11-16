window.BENCHMARK_DATA = {
  "lastUpdate": 1794832590345,
  "repoUrl": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox",
  "entries": {
    "raul bench": [
      {
        "commit": {
          "author": {
            "email": "74975850+ExtraToast@users.noreply.github.com",
            "name": "Joris Wouter Jonkers",
            "username": "ExtraToast"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e47156caf416c2bc631fa087d41635746e6e4e6e",
          "message": "feat(bench): add raul bench with a synthetic suite, comparison and CI history (#8)\n\n* docs(plan): phase 0 findings, architecture and migration plan\n\nAdds docs/PLAN.md for review before any code lands, and pins Rust 1.100.0 with a 7-day registry cooldown.\n\n* docs: record the plan decisions, ADRs 0001 to 0006, glossary and licence\n\nRecords the owner's decisions of 2026-11-16 in docs/PLAN.md, adds ADRs 0001 to 0006, moves the product glossary here (with Component meaning a pinned project, Engine extension and Release test), adds the GPL-3.0-only LICENSE and the order agreed for work in other repositories.\n\n* docs(glossary): drop a self-referencing avoid entry\n\n* feat: pin the five Components and add the raul command line\n\nAdds the Components as submodules at the versions of the first Known-good set, the umbrella workspace with raul-core and raul-cli, known-good.toml with its check, and the submodule workflow in CONTRIBUTING.md. raul runs every nvmod command in process. loot-rs is pinned at v0.2.0 because that is what nvmod v0.5.0 links.\n\n* feat(detect): find game copies on Steam, GOG and Epic\n\nAdds raul-core::detect and raul detect: Steam libraries (native, Steam Deck, Flatpak), GOG and Heroic folders, Epic manifests, or one picked folder. It reports the game, storefront (nvmod's markers), executable version against the supported 1.4.0.525, and missing DLC masters, including Fallout 3 for TTW.\n\n* ci: three-OS build, deny, audit, submodule check and Component bump workflow\n\nPipeline Complete gains the submodule check, cargo deny and cargo audit, nextest, doc tests and an MSRV check, and gates a Windows and macOS arm64 build. A Component release can dispatch component-bump.yml, which moves one submodule, adds a Known-good set through raul known-good bump and opens a pull request with auto-merge.\n\n* ci: build macOS on macos-latest\n\nThe macos-14 runner configuration is not available to this repository.\n\n* feat(bench): add raul bench with a synthetic suite and report comparison\n\nMeasures loot-rs sorting of 60, 300 and 1000 synthetic plugins, reading their headers, and installing an 8 MiB synthetic zip through nvmod. LOD and in-engine frame time are reported as skipped with the reason. raul bench compare marks cases slower than a threshold as warnings.\n\n* ci(bench): compare pull requests labelled bench and keep a history on main\n\nAdds the Bench workflow, docs/BENCHMARKS.md and the comparison test fix.",
          "timestamp": "2026-11-16T11:55:20Z",
          "tree_id": "a590a548b93d8ba2a5fb2d8f60bd127fcc3f1d0e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e47156caf416c2bc631fa087d41635746e6e4e6e"
        },
        "date": 1794830248909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 4.116296999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 54.965045999999994,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 531.8287459999999,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.658909,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.532178,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "74975850+ExtraToast@users.noreply.github.com",
            "name": "Joris Wouter Jonkers",
            "username": "ExtraToast"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "19783c49e1b92325209337ce908ce2f4d81dc4dd",
          "message": "feat(release): installers for Linux, Windows and macOS, and an opt-in update check (#9)\n\n* docs(plan): phase 0 findings, architecture and migration plan\n\nAdds docs/PLAN.md for review before any code lands, and pins Rust 1.100.0 with a 7-day registry cooldown.\n\n* docs: record the plan decisions, ADRs 0001 to 0006, glossary and licence\n\nRecords the owner's decisions of 2026-11-16 in docs/PLAN.md, adds ADRs 0001 to 0006, moves the product glossary here (with Component meaning a pinned project, Engine extension and Release test), adds the GPL-3.0-only LICENSE and the order agreed for work in other repositories.\n\n* docs(glossary): drop a self-referencing avoid entry\n\n* feat: pin the five Components and add the raul command line\n\nAdds the Components as submodules at the versions of the first Known-good set, the umbrella workspace with raul-core and raul-cli, known-good.toml with its check, and the submodule workflow in CONTRIBUTING.md. raul runs every nvmod command in process. loot-rs is pinned at v0.2.0 because that is what nvmod v0.5.0 links.\n\n* feat(detect): find game copies on Steam, GOG and Epic\n\nAdds raul-core::detect and raul detect: Steam libraries (native, Steam Deck, Flatpak), GOG and Heroic folders, Epic manifests, or one picked folder. It reports the game, storefront (nvmod's markers), executable version against the supported 1.4.0.525, and missing DLC masters, including Fallout 3 for TTW.\n\n* ci: three-OS build, deny, audit, submodule check and Component bump workflow\n\nPipeline Complete gains the submodule check, cargo deny and cargo audit, nextest, doc tests and an MSRV check, and gates a Windows and macOS arm64 build. A Component release can dispatch component-bump.yml, which moves one submodule, adds a Known-good set through raul known-good bump and opens a pull request with auto-merge.\n\n* ci: build macOS on macos-latest\n\nThe macos-14 runner configuration is not available to this repository.\n\n* feat(bench): add raul bench with a synthetic suite and report comparison\n\nMeasures loot-rs sorting of 60, 300 and 1000 synthetic plugins, reading their headers, and installing an 8 MiB synthetic zip through nvmod. LOD and in-engine frame time are reported as skipped with the reason. raul bench compare marks cases slower than a threshold as warnings.\n\n* ci(bench): compare pull requests labelled bench and keep a history on main\n\nAdds the Bench workflow, docs/BENCHMARKS.md and the comparison test fix.\n\n* feat(release): installers for Linux, Windows and macOS, and an opt-in update check\n\nThe Release workflow builds raul and the bundled Components per platform, packages an AppImage, deb, tar.gz, NSIS setup, portable zip and a universal macOS dmg with cargo-packager, smoke-tests each, and publishes them with SHA256SUMS, a CycloneDX SBOM and build provenance. Signing runs only when its secrets exist; the release notes say otherwise. raul update checks GitHub Releases on request and remembers the opt-in for the GUI.\n\n* fix(release): let the version step fetch git dependencies\n\n* fix(bench): skip a sort case that fails instead of timing it, and write history only from main\n\n* fix: address review findings in the update check and release workflow\n\nThe update setting is merged into settings.toml instead of replacing it, an unparsable file is never overwritten, a missing configuration folder is an error rather than a relative path, and development versions compare by their release numbers. Packages get plain file names so SHA256SUMS still matches after upload, every installer carries the licence, the publish job stamps the tag version before writing the SBOM, and the release notes read known-good.toml without a debug build.\n\n* fix(release): check the nv-rs Windows build against SHA256SUMS-windows\n\n* chore(deps): move ttwright-rs to main with the macOS build fix\n\nttwright-rs #4 makes it compile on macOS; the release build bundles it there.\n\n* ci: fetch crates through the estate packages proxy in every cargo job\n\nThe repository hygiene guard refuses cargo install and cargo build outside the proxy. Release, CI and bench jobs now set up actions/setup-package-proxy right after checkout.",
          "timestamp": "2026-11-16T13:34:35+01:00",
          "tree_id": "8232791aed4fdffdf743cc067b94532ec6fe97bf",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/19783c49e1b92325209337ce908ce2f4d81dc4dd"
        },
        "date": 1794832589716,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 3.5304830000000003,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 44.197862,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 405.622239,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.7281249999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.023672999999999,
            "unit": "ms"
          }
        ]
      }
    ]
  }
}