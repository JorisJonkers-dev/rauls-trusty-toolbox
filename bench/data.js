window.BENCHMARK_DATA = {
  "lastUpdate": 1795185474063,
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
          "id": "6e0ceac694fa431f3bdeb938510da7a4e3e98506",
          "message": "docs: architecture, code of conduct and install instructions (#10)\n\n* docs(plan): phase 0 findings, architecture and migration plan\n\nAdds docs/PLAN.md for review before any code lands, and pins Rust 1.100.0 with a 7-day registry cooldown.\n\n* docs: record the plan decisions, ADRs 0001 to 0006, glossary and licence\n\nRecords the owner's decisions of 2026-11-16 in docs/PLAN.md, adds ADRs 0001 to 0006, moves the product glossary here (with Component meaning a pinned project, Engine extension and Release test), adds the GPL-3.0-only LICENSE and the order agreed for work in other repositories.\n\n* docs(glossary): drop a self-referencing avoid entry\n\n* feat: pin the five Components and add the raul command line\n\nAdds the Components as submodules at the versions of the first Known-good set, the umbrella workspace with raul-core and raul-cli, known-good.toml with its check, and the submodule workflow in CONTRIBUTING.md. raul runs every nvmod command in process. loot-rs is pinned at v0.2.0 because that is what nvmod v0.5.0 links.\n\n* feat(detect): find game copies on Steam, GOG and Epic\n\nAdds raul-core::detect and raul detect: Steam libraries (native, Steam Deck, Flatpak), GOG and Heroic folders, Epic manifests, or one picked folder. It reports the game, storefront (nvmod's markers), executable version against the supported 1.4.0.525, and missing DLC masters, including Fallout 3 for TTW.\n\n* ci: three-OS build, deny, audit, submodule check and Component bump workflow\n\nPipeline Complete gains the submodule check, cargo deny and cargo audit, nextest, doc tests and an MSRV check, and gates a Windows and macOS arm64 build. A Component release can dispatch component-bump.yml, which moves one submodule, adds a Known-good set through raul known-good bump and opens a pull request with auto-merge.\n\n* ci: build macOS on macos-latest\n\nThe macos-14 runner configuration is not available to this repository.\n\n* feat(bench): add raul bench with a synthetic suite and report comparison\n\nMeasures loot-rs sorting of 60, 300 and 1000 synthetic plugins, reading their headers, and installing an 8 MiB synthetic zip through nvmod. LOD and in-engine frame time are reported as skipped with the reason. raul bench compare marks cases slower than a threshold as warnings.\n\n* ci(bench): compare pull requests labelled bench and keep a history on main\n\nAdds the Bench workflow, docs/BENCHMARKS.md and the comparison test fix.\n\n* feat(release): installers for Linux, Windows and macOS, and an opt-in update check\n\nThe Release workflow builds raul and the bundled Components per platform, packages an AppImage, deb, tar.gz, NSIS setup, portable zip and a universal macOS dmg with cargo-packager, smoke-tests each, and publishes them with SHA256SUMS, a CycloneDX SBOM and build provenance. Signing runs only when its secrets exist; the release notes say otherwise. raul update checks GitHub Releases on request and remembers the opt-in for the GUI.\n\n* docs: architecture, code of conduct and install instructions\n\nAdds docs/ARCHITECTURE.md (pieces, data flow from game copy to launch, versions, workspaces), CODE_OF_CONDUCT.md (Contributor Covenant 2.1) and install, from-source and quickstart sections in the README.\n\n* docs(plan): mirror nv-linux ADR 0014 points 17 to 33\n\n* docs(bench): real game measurements go through the nv-rs bench lock",
          "timestamp": "2026-11-16T12:40:58Z",
          "tree_id": "175f4f992267b8c422a363310cc65129dc62953f",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/6e0ceac694fa431f3bdeb938510da7a4e3e98506"
        },
        "date": 1794832949889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 4.137766,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 55.552517,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 529.250052,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.930466,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.428833,
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
          "id": "bee895e0682b26bc6d1bed4f261791d831dd0973",
          "message": "fix(release): make the Windows and macOS smoke tests run unattended (#28)\n\n* fix(release): make the Windows and macOS smoke tests run unattended\n\nGit Bash on Windows has no /dev/stderr, and the dmg's licence agreement made hdiutil attach wait for an answer.\n\n* fix(release): set the pager through env for hdiutil",
          "timestamp": "2026-11-16T12:48:48Z",
          "tree_id": "d94df8ee5ee14e1f7e052c64af6c33abfb9f2298",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/bee895e0682b26bc6d1bed4f261791d831dd0973"
        },
        "date": 1794833407998,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.6020400000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.412291999999997,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 253.07869,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.408798,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.901101,
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
          "id": "b9c393eebe1ebd5937ab915b7065892806e15b6a",
          "message": "docs: reorganise the stack around the Toolbox (plan and ADRs 0007 to 0011) (#29)\n\n* docs: reorganise the stack around the Toolbox (plan and ADRs 0007 to 0011)\n\nThe Toolbox now owns orchestration, tool supply, the Runner, the view model and\nthe GUI; nvmod-rs keeps mod management. docs/REORG.md records the division,\nevery ticket's disposition, the milestones with their gates, the order of the\nmoves and the open questions. ADRs 0007 to 0011 record the decisions; ADRs\n0002, 0003 and 0005 point to them.\n\n* docs: record the owner's UI and plan decisions (ADR-0012: egui)\n\nSettle the UI direction against the mockup (readable dark, teal accent, Inter,\nsidebar of seven pages, gamepad from the first GUI release, light voice),\nchoose egui over GPUI, remove moved nvmod commands at once, tighten the M3\nframe-time gate to 6.9 and 11.1 ms, and list the other decisions taken.\n\n* docs: list the epics under each milestone",
          "timestamp": "2026-11-16T17:11:42+01:00",
          "tree_id": "3b4330e5eca7f84e3a6df466f40800fabe8b46ba",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/b9c393eebe1ebd5937ab915b7065892806e15b6a"
        },
        "date": 1794845594497,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.6111140000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.541303,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 255.16560900000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.402518,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.223546,
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
          "id": "7b7935c3fa596dc6a8974ebb1158dcaa77f24701",
          "message": "docs: GUI rules, the interactive mockup and ADRs 0013 to 0015 (#73)\n\nAdd docs/UI-RULES.md and docs/design/mockup.html for the settled GUI design\n(readable dark, sidebar of seven pages, gamepad, light voice), ADR-0013 for\nthe look and navigation, ADR-0014 for the asset policy and the opt-in install\nskin, ADR-0015 for one loot-rs through a patch, an asset notice file and\nthree glossary terms.",
          "timestamp": "2026-11-16T17:17:51+01:00",
          "tree_id": "767c28a90a1cd3b18e68aacd2086f473fd5fc12a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7b7935c3fa596dc6a8974ebb1158dcaa77f24701"
        },
        "date": 1794845958082,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.60795,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.484762,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 253.77519900000001,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.427042,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.918813999999999,
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
          "id": "9275a2de97497ffc93391a31e0638857a98ace07",
          "message": "docs(adr): lodwright-rs moves to GPL-3.0-only from its next release (#104)",
          "timestamp": "2026-11-16T17:52:37+01:00",
          "tree_id": "81a42cd06e6b29fecd7fbb3b6d266e94870ac390",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/9275a2de97497ffc93391a31e0638857a98ace07"
        },
        "date": 1794848035474,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.601858,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.494054000000002,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 253.94218600000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.403058,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.047341,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "info@jorisjonkers.dev",
            "name": "J.W. Jonkers",
            "username": "ExtraToast"
          },
          "committer": {
            "email": "74975850+ExtraToast@users.noreply.github.com",
            "name": "Joris Wouter Jonkers",
            "username": "ExtraToast"
          },
          "distinct": true,
          "id": "35d50723be28d55438160ebd7a576490c8d0ddff",
          "message": "feat(update): run the update check through raul-runner and compare versions as semver\n\nraul update check starts curl through the Runner with a scrubbed environment (plus proxy and certificate variables), a timeout and an output cap, HTTPS only and without the person's .curlrc. Versions compare by semver precedence: a pre-release comes before its release, build metadata is ignored, and a tag with a pre-release part is never offered. raul-cli no longer links nvmod-tools for this.\n\nCloses #48",
          "timestamp": "2026-11-16T18:00:46+01:00",
          "tree_id": "520233a69f908907210b790701cf21ff276f9334",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/35d50723be28d55438160ebd7a576490c8d0ddff"
        },
        "date": 1794848531620,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 4.573745,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 57.502814,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 525.431045,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.879741,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.156897,
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
          "id": "649f27395e1c9a7d28911c9978768db7970ef3c3",
          "message": "fix(release): make the macOS and Windows smoke tests finish (#106)\n\n* fix(release): keep the macOS smoke test from failing on a broken pipe\n\nThe dmg licence is accepted by piping yes into hdiutil attach. When hdiutil\nstops reading, yes ends with a broken pipe, and under pipefail that failed the\nstep. yes is now fed through process substitution, and the lipo checks read\ncaptured output instead of piping into grep -q.\n\n* fix(release): check the Windows installer's contents and time-limit the silent install\n\nThe silent NSIS install hung in CI until the job's 90-minute limit. The\nsmoke test now unpacks the installer with 7z and checks the raul.exe inside,\nthen tries the silent install under a 180-second limit; a hang is a warning\nthat names what was checked instead of a stuck job.",
          "timestamp": "2026-11-16T20:05:14+01:00",
          "tree_id": "df47c97cc952c321ed39f5a348c0d21895e3ce94",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/649f27395e1c9a7d28911c9978768db7970ef3c3"
        },
        "date": 1794855984333,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 3.540682,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 44.261408,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 408.398958,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.73888,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.946287,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d0d3a0af9448f0e6ea339f1844e0932b8cebc01b",
          "message": "chore(main): release 0.1.0 (#5)\n\n* chore(main): release 0.1.0\n\n* chore: list the Runner move as one change in the 0.1.0 changelog\n\n---------\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>\nCo-authored-by: J.W. Jonkers <info@jorisjonkers.dev>",
          "timestamp": "2026-11-16T20:11:36+01:00",
          "tree_id": "106b36820f93892ef563aef03dabff1377b7194b",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d0d3a0af9448f0e6ea339f1844e0932b8cebc01b"
        },
        "date": 1794856367844,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.617221,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.594880999999997,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 254.867746,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.4586009999999998,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.780435,
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
          "id": "e3b8f10eb5ffadabc9963bcf36eef49fe15865ab",
          "message": "ci: gate 70% of changed lines on tests, as the Components do (#109)\n\nPipeline Complete runs the workspace tests once under cargo llvm-cov (nextest) and then\ntools/coverage-gate, which fails a change when fewer than 70% of the executable lines it adds or\nedits in the raul crates ran in a test. The tool is the one ttwright-rs uses (Rust, no\ndependencies, its own workspace); docs/COVERAGE.md describes it.\n\nRefs #19.",
          "timestamp": "2026-11-16T22:30:00+01:00",
          "tree_id": "53f3780e90040084855be7b846b4ebf0418b8e6c",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e3b8f10eb5ffadabc9963bcf36eef49fe15865ab"
        },
        "date": 1794864689423,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.6079350000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.519617,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 253.928784,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.411826,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.40585,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "169e4f83fdc4e929af21de9996ca8863c30362fa",
          "message": "chore(deps): bump ttwright-rs to v0.3.0 (#110)\n\n* chore(deps): bump ttwright-rs to v0.3.0\n\n* test(cli): read the expected set from known-good.toml\n\nThe version tests named the first set and its versions, so every Component\nbump broke them. They now take the latest known-good set from the file the\nbinary is built from.\n\n---------\n\nCo-authored-by: rauls-trusty-toolbox bump <noreply@github.com>\nCo-authored-by: J.W. Jonkers <info@jorisjonkers.dev>",
          "timestamp": "2026-11-16T22:36:23Z",
          "tree_id": "4dc1700d17a67d9c172e1881082aae7e43a8bfa7",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/169e4f83fdc4e929af21de9996ca8863c30362fa"
        },
        "date": 1794868668069,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.641251,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 24.724477,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 254.543659,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.40923,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 7.110106,
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
          "id": "8f5a572b85e8922ec5f585eedb32d0a94928cf6c",
          "message": "chore(deps): bump nvmod to v0.6.0, with the loot-rs v0.5.0 it links (#118)\n\nnvmod v0.6.0 links loot-rs v0.5.0, so the new set moves both (ADR-0015): the two\nsubmodules, the workspace loot-rs tag and Cargo.lock.",
          "timestamp": "2026-11-16T23:51:57+01:00",
          "tree_id": "160bcced7fc6276b35b410f83cbed473990a9f44",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8f5a572b85e8922ec5f585eedb32d0a94928cf6c"
        },
        "date": 1794869595509,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.336387,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.504489000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.347903,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.46687,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.841707000000001,
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
          "id": "33029e7a5c3c06f9048840b544f241125fb34908",
          "message": "feat(config): publish the raul-config v1 Contract with its schema and fixtures (#119)\n\n* feat(config): publish the raul-config v1 Contract with its schema and fixtures\n\n* test(config): fixtures for a missing configuration folder and an unknown key\n\n* fix(config): list every printed field as required and pin the list order in a test\n\n* test(config): neutral paths in the fixtures\n\n* fix(config): print every field without serde defaults and document the usage exit",
          "timestamp": "2026-11-17T06:54:47+01:00",
          "tree_id": "46b7da5d2c0fec47735e8d53c09b451827e1c18d",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/33029e7a5c3c06f9048840b544f241125fb34908"
        },
        "date": 1794894951869,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.05243,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.628124,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.449838999999997,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.495692,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.938031,
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
          "id": "552c447550d5aee96e354edf7e831988af85ff76",
          "message": "docs: cite only this repository's ADR in the update check (#131)\n\nCloses #129.",
          "timestamp": "2026-11-17T07:01:08+01:00",
          "tree_id": "7712e8f75bd395f218ab694f75e7f30991748c60",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/552c447550d5aee96e354edf7e831988af85ff76"
        },
        "date": 1794895342002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0822939999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.649832,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.386472,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.4885689999999998,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.060243,
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
          "id": "ef3ef976fd381d6b4ee4377faa5a695d0a2128fc",
          "message": "feat(config): raul config list, get, set and unset for the paths the GUI needs (#120)\n\n* feat(config): raul config list, get, set and unset for the paths the GUI needs\n\n* fix(config): write through a new synced temporary file and name a section that is not a table\n\n* test(config): compare the stored path with the file, not its spelling",
          "timestamp": "2026-11-17T07:04:28+01:00",
          "tree_id": "9580ff212f416bd1f5e7b231b950a1862462bea0",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ef3ef976fd381d6b4ee4377faa5a695d0a2128fc"
        },
        "date": 1794895536315,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.385396,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.629417,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.291404,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.820397,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.314639,
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
          "id": "5d8b1a4f0186e4f0c9316563f601f3f94d7eb209",
          "message": "feat(ttw): publish the raul-events and raul-ttw-result v1 Contracts with fixtures (#121)",
          "timestamp": "2026-11-17T07:11:09+01:00",
          "tree_id": "a4c7de6153bfab3fadef69d3cd818e613af55119",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5d8b1a4f0186e4f0c9316563f601f3f94d7eb209"
        },
        "date": 1794895936853,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.118968,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.661103,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.236993,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.4969549999999998,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.084885,
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
          "id": "58a51fae6604168bff227002b074f69f89eb6f42",
          "message": "ci(release): build ttwright-cli and bundle the ttwright program (#125)\n\n* ci(release): build ttwright-cli and bundle the ttwright program\n\n* ci(release): check the bundled ttwright in the AppImage and the Windows installer too\n\n* ci(release): spell the missing-ttwright check as an if",
          "timestamp": "2026-11-17T07:14:12+01:00",
          "tree_id": "319412452f43d968ae756bd0b14b3d835ce14f50",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/58a51fae6604168bff227002b074f69f89eb6f42"
        },
        "date": 1794896127804,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.387025,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.604043,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.789124,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.768078999999999,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.447308999999999,
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
          "id": "15dd1ca4789af6803800d2b968e2ccdc64450ad8",
          "message": "feat(ttw): raul ttw installs TTW through ttwright-rs and adds it to the Instance (#122)\n\n* feat(ttw): raul ttw installs TTW through ttwright-rs and adds it to the Instance\n\n* test(ttw): bundled program layouts, the event tail and the result version check; absolute paths\n\n* fix(ttw): a stop asked for while ttwright finishes adds nothing\n\n* fix(ttw): never follow a link when clearing or counting the work folder, and refuse a zero timeout\n\n* fix(ttw): cap --timeout-secs at a week",
          "timestamp": "2026-11-17T07:19:08+01:00",
          "tree_id": "fabbd41ec63add757c7959596a1dc1f76d02c005",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/15dd1ca4789af6803800d2b968e2ccdc64450ad8"
        },
        "date": 1794896413947,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.106593,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.670967,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.591188000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.511391,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.983109000000001,
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
          "id": "9f936ce5ef202226af1d0e68d6df54edf241513d",
          "message": "feat(lod): build lodwright's request from the Profile and show it with raul lod plan (#123)\n\n* feat(lod): build lodwright's request from the Profile and show it with raul lod plan\n\n* fix(lod): refuse an output folder in the data folder or a git working tree, and say plugin for one\n\n* test(lod): a synthetic world name in the fixture\n\n* style(lod): rustfmt\n\n* test(lod): compare the output folder as a path, not as text",
          "timestamp": "2026-11-17T07:25:55+01:00",
          "tree_id": "06653547476b46018a7501366d428f6aad89cbb6",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/9f936ce5ef202226af1d0e68d6df54edf241513d"
        },
        "date": 1794896836252,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.110498,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.776403,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 22.031852,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.536754,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 7.059385,
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
          "id": "92320af83ab66ad125f4269d161d85213f5d9608",
          "message": "feat(lod): publish the raul-lod-result v1 Contract with fixtures (#126)",
          "timestamp": "2026-11-17T07:33:10+01:00",
          "tree_id": "e61f884331682fcdda32a259d5abc90e056cfa5f",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/92320af83ab66ad125f4269d161d85213f5d9608"
        },
        "date": 1794897332786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.063302,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.623278,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.557454,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.495345,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.00194,
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
          "id": "c5105cde5cb5a8599e5707334264fd044b84a39b",
          "message": "feat(gui): open the egui shell from raul with no arguments (#136)\n\n* feat(gui): open the egui shell from raul with no arguments\n\n* test(gui): run the release raul against raul_cli::run and cover the shell helpers\n\n* fix(gui): refuse a second writer before the first reports, and end a job whose handling panics\n\n* fix(gui): follow main: drop the end-to-end case of the old TTW command and allow the toolkit fonts\n\nraul ttw is now the Toolbox command that drives ttwright, so the service test\nthat ran the earlier installer contract through it no longer applies; the TTW\nflow is rewired to the new command with its page. cargo deny allows the\nlicences of the fonts egui bundles and reports unmaintained crates only when\nthe GUI depends on them directly.\n\n* fix(gui): keep a busy dialog open only while it has an action, read keys after the frame, and prove the smoke frame",
          "timestamp": "2026-11-17T07:44:41+01:00",
          "tree_id": "d3a18f721c31db2c122135855204719d59945724",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c5105cde5cb5a8599e5707334264fd044b84a39b"
        },
        "date": 1794897960152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0964200000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.61377,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.376735,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.5036509999999998,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.891584,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f167a6d073f4319e86d957d74c906aa8c5a9877b",
          "message": "chore(main): release 0.2.0 (#130)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-17T08:01:21+01:00",
          "tree_id": "2499fae540af166a3ddcabac7d9df0e1ac8a1829",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f167a6d073f4319e86d957d74c906aa8c5a9877b"
        },
        "date": 1794898956819,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.094678,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.667992,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.328611000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.514993,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.109001999999999,
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
          "id": "74fff852a42f7276cd234bdabe3096eb4f1d8620",
          "message": "feat(gui): register verify, version and detect, and give run its target (#138)",
          "timestamp": "2026-11-17T08:18:07+01:00",
          "tree_id": "bc15d1c0c33bcaf412fe36cb97badf54e28928db",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/74fff852a42f7276cd234bdabe3096eb4f1d8620"
        },
        "date": 1794899966374,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.089958,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.614712,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.769095,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.506265,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.337442,
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
          "id": "da41bcd11aaa2f0e85c3efe55e551483a72c1a5f",
          "message": "chore(deps): bump lodwright-rs to v0.3.0 (#142)\n\nlodwright-rs v0.3.0 adds `lodwright generate` and publishes the lodwright-request and lodwright-result v1\nschemas as release assets. The new Known-good set 2026-11-d moves only lodwright-rs and names the three\nContracts it speaks.\n\nThe snapshots in contracts/snapshots/lodwright-rs/ are already byte for byte the v0.3.0 release assets; a new\ntest keeps every snapshot equal to the schema its pinned Component publishes. lodwright-rs is GPL-3.0-only\nfrom this release, so the release notices and the plan's licence table say so.",
          "timestamp": "2026-11-17T08:28:36+01:00",
          "tree_id": "6646a93a1eda4e3136b7a8065ce082f3a6963f98",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/da41bcd11aaa2f0e85c3efe55e551483a72c1a5f"
        },
        "date": 1794900588276,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.073517,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.617177,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.198960999999997,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.503272,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.008821999999999,
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
          "id": "296335e0dd3e09467fb17d5ae2fd722919d8206d",
          "message": "feat(gui): Settings, Versions and Home pages (#143)\n\n* feat(gui): Settings, Versions and Home pages\n\nSettings holds the paths and saves each through raul config set when its\nfield is left. Versions shows the active engine build and installs the\nnv-rs release after a confirm. Home shows readiness, the attention chips\nand the launch panel, and starts the game with raul run nv-rs. Every\naction shows its raul command and starts through the command registry.\n\n* fix(gui): pass the nv-rs target through the run entry, and test the engine install\n\n* fix(gui): drop a stale Play, show Home again after a launch, and say when a path is cleared\n\nA Play that waited for the reads is dropped when the person opens another\npage meanwhile. Home returns to readiness after raul run has started the\ngame. A failed engine check reads as not known, and the top bar stops\nshowing an engine. Emptying a path reports that it was cleared. The\nversion list is read once, and a page does not start a read while one\nruns.",
          "timestamp": "2026-11-17T08:51:52+01:00",
          "tree_id": "38ca915309547cad69fd75418ce7b3ed77fc4782",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/296335e0dd3e09467fb17d5ae2fd722919d8206d"
        },
        "date": 1794901990808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.4698669999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.383589000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.967669,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.011681,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 8.835089,
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
          "id": "dfd987a9f335afde48d98232ea05196523a84b28",
          "message": "feat(lod): raul lod generate runs lodwright with the request and keeps its output as one Mod (#127)\n\n* feat(lod): raul lod generate runs lodwright with the request and keeps its output as one Mod\n\n* fix(lod): keep the new Mod with a warning when the old one cannot be removed, and cap the timeout\n\n* fix(lod): clear a stale backup Mod first, name a failed rename-back, and report a stop during the replace as cancelled\n\n* test(lod): run raul lod generate against the real lodwright in CI\n\nThe new test builds a synthetic world with lodwright's own test kit, makes it the game master of an Instance,\nand runs `raul lod generate` twice with the real lodwright program: the first run leaves one enabled Mod with\nlodwright's files, the second answers up_to_date and changes nothing. It sits behind the real-lodwright feature,\nso a plain cargo test needs no lodwright.\n\nCI downloads the Linux release asset of the lodwright-rs version in the shipped Known-good set, checks it\nagainst that release's SHA256SUMS, lints the test and runs it.\n\n* fix(lod): put back a backup an interrupted run left alone, and list lod generate in the help\n\nA backup Mod with no LOD Mod beside it is the only copy: an earlier run stopped between moving the old Mod\nout of the way and adding the new one. It used to be removed first, so a later run whose install failed lost\nthe Mod. It is now renamed back first; a backup beside a LOD Mod is still removed.\n\nA stale events file that cannot be removed now fails the run instead of being read as this run's progress.\n`raul help` and the usage line of `raul lod` name `lod generate` and its flags.\n\n* fix(gui): lod generate is in raul help now, so its registry entry no longer waits for #127\n\n* test(gui): pin lod generate's refusal on the fixture, which has no lodwright program",
          "timestamp": "2026-11-17T09:18:03+01:00",
          "tree_id": "6bbe1e1b0e3118e58339e0886f6e180d43b2481e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/dfd987a9f335afde48d98232ea05196523a84b28"
        },
        "date": 1794903549364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.652225,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.51176,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.362569,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.853951,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.287939,
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
          "id": "5119d1af78db58461cf86d1fc509025e078b4e7d",
          "message": "feat(gui): the first-run setup wizard (#144)\n\n* feat(gui): the first-run setup wizard\n\nFive steps on the shell, as in the design mockup: the New Vegas and\nstorage folders, TTW (optional, with the Fallout 3 folder), the engine,\na quick read-only check and Ready. Paths save through raul config set,\nthe engine goes in with raul init and raul install-nv-rs --release, and\nthe check runs raul doctor. Nothing in a game folder changes; the wizard\ncan be skipped and run again from Settings.\n\n* test(gui): drive the setup wizard from its first step to Home\n\n* test(gui): refresh the wizard snapshots now that Find can run\n\n* fix(gui): Raul does not call the setup clean when the checks did not run\n\n* fix(gui): keep the wizard's late answers and Find from overwriting what the person did\n\nFind fills a folder only while nothing is typed, and an answer for a step\nthe person left is dropped. Back waits for a running install or save. A\nfailed read or check offers Try again, an unknown finding severity is a\nnote rather than OK, and a disabled Install says why.",
          "timestamp": "2026-11-17T09:22:27+01:00",
          "tree_id": "bd60adb60f3f514ab84ad87cbb1737a2654c0460",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5119d1af78db58461cf86d1fc509025e078b4e7d"
        },
        "date": 1794903818084,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0761180000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.673916,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.549555,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.512552,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.990805,
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
          "id": "a37324b66ce762e2b926d31c1dc5e8c0817486b5",
          "message": "feat(gui): share the Instance with every page, read TTW reports from job output, register tools list (#148)",
          "timestamp": "2026-11-17T09:23:35+01:00",
          "tree_id": "0eda3d2a13a7313fe9f4ebce279bf76b87aba727",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a37324b66ce762e2b926d31c1dc5e8c0817486b5"
        },
        "date": 1794903882786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.9482849999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.675700999999998,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.725712,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.56556,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.327195000000001,
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
          "id": "1795a59650e98e1b8cd8679e5fcdd19e85a9b69f",
          "message": "feat(gui): Load order page with sort preview, apply and undo (#150)",
          "timestamp": "2026-11-17T09:30:48+01:00",
          "tree_id": "b1732b00cfe35c7d307d20fedd8ef87f7e777105",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1795a59650e98e1b8cd8679e5fcdd19e85a9b69f"
        },
        "date": 1794904307675,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.643351,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.5443999999999996,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.278103,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.875114,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.953879,
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
          "id": "94c78d5c08967093e27a6102542498e6c2e04093",
          "message": "fix(gui): register nvmod's raul run and start the game from Play with it (#149)\n\n* fix(gui): register nvmod's raul run and start the game from Play with it\n\nThe run entry waited for a run guard command that is no longer planned.\nraul runs nvmod's run command in process, whose target is its first\nvalue: Play passes nv-rs. The entry takes the flags of that usage and\nwrites, since a run stages the Profile first.\n\n* fix(gui): list only the run flags a page passes\n\n* test(gui): give the registry fixture a home of its own, so a run finds no engine of the test machine\n\n* fix(gui): merge the registry test's identical exit-code arms",
          "timestamp": "2026-11-17T09:51:20+01:00",
          "tree_id": "d42d95641a3b23eb860e2ab8c9a38bb198775845",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/94c78d5c08967093e27a6102542498e6c2e04093"
        },
        "date": 1794905550904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.090157,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.6480690000000005,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.565614,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.504675,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.103669,
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
          "id": "1929615fabb364ac95fb949b267dd77a24e7b529",
          "message": "chore(deps): bump nvmod to v0.6.2 (#152)",
          "timestamp": "2026-11-17T10:00:16+01:00",
          "tree_id": "1da94ba04bb79533fa1d29c3637e1629c08a5dd3",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1929615fabb364ac95fb949b267dd77a24e7b529"
        },
        "date": 1794906092511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.358101,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.423306,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.958287,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.7765439999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.202264999999999,
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
          "id": "ef32e335aa533ce4f86e94cb01c7117b09da3a43",
          "message": "feat(gui): LOD page with plan, blocks, generate and stop (#151)\n\n* feat(gui): LOD page with plan, blocks, generate and stop\n\nThe LOD page reads `raul lod plan` and shows what a run reads and writes,\none block per world with its status and progress, and the generator\nversion and output folder read only. Generate asks once, stating the cost,\nthen runs `raul lod generate`, which keeps the output as one Mod. The run\nkeeps going when the page is left, and Stop ends it with the Instance as\nit was.\n\nSnapshot fixtures cover the no-Instance, loading, planned, generating,\ngenerated, up-to-date, error, plan-error, long-text and large-list states.\nAn end-to-end test drives the page through the release `raul` on a\nsynthetic Instance with a stand-in generator: generate, a second run that\nfinds the output current, Cancel in the dialog, and Stop.\n\nCloses #99\n\n* test(gui): accept the header and the status line both naming the running state\n\n* test(gui): stop the fake LOD run only after it has started",
          "timestamp": "2026-11-17T10:06:26+01:00",
          "tree_id": "b1a3374bdcc20c23ba5f0f47264a08850c4b29f7",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ef32e335aa533ce4f86e94cb01c7117b09da3a43"
        },
        "date": 1794906465199,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.375177,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.528933,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.013323,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.759662,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.999078,
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
          "id": "7ecde773a3daf54016aef5807027d765b4bf0e26",
          "message": "feat(gui): TTW flow page with a dry-run review, install, cancel and resume (#153)\n\n* feat(gui): TTW flow page with a dry-run review, install, cancel and resume\n\nThe TTW flow walks the four steps of UI-RULES P4. Sources shows the\nFallout 3 folder, the New Vegas folder and the TTW package from\n`raul config list`, read only with a link to Settings, and says what is\nmissing. Output names the Instance. Review runs `raul ttw --dry-run` and\nshows its checks, what happens and the install command. Install runs\n`raul ttw` as a job with a percentage, keeps going when the page is\nleft, and can be cancelled. A stopped run offers Resume (`--resume`), a\nrefused one Try again.\n\nTtwWizard no longer asks whether an installer is pinned: it needs the\nFallout 3 folder and the TTW package, which is what `raul ttw` needs.\n\nChecked with model tests for the flow, snapshot fixtures for every state\nand end-to-end tests through the release raul with a fake installer:\ninstall, refusal and try again, resume, and cancel.\n\n* fix(gui): keep the TTW flow when its sources did not change\n\nShowing the flow again reads the sources once more and starts over only\nwhen they changed. On a resumed review the one button says Resume, and\nan unknown install phase reads as Working.",
          "timestamp": "2026-11-17T10:39:59+01:00",
          "tree_id": "0ab3cb548b4b703480cc343070ecf4b1f8384f23",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7ecde773a3daf54016aef5807027d765b4bf0e26"
        },
        "date": 1794908467707,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.102484,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.694496,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.681784,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.504637,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.999243,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9f73539aa6f136a1029556a5a404702bff074db9",
          "message": "chore(main): release 0.3.0 (#140)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-17T10:45:16+01:00",
          "tree_id": "699fe3a951a62b2de2ea83bd402c85d5b02a0e84",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/9f73539aa6f136a1029556a5a404702bff074db9"
        },
        "date": 1794908793791,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.097714,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.595472,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.292384000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.488893,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.743056,
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
          "id": "265874a2fa66b8b4e29718697e2f536f6ce76e87",
          "message": "ci: follow Component releases with a self-hosted Renovate run (#154)\n\nA scheduled Renovate run finds each new release of nv-rs, nvmod, lodwright-rs and ttwright-rs and opens one pull request that moves the submodule, adds a Known-good set and refreshes both lock files. nvmod moves with the loot-rs it links. The Component repositories send nothing. Replaces the dispatch-driven Component bump workflow.",
          "timestamp": "2026-11-17T10:49:24+01:00",
          "tree_id": "14cc29893cdd64cc87a237ff062cfe5486c128eb",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/265874a2fa66b8b4e29718697e2f536f6ce76e87"
        },
        "date": 1794909042330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.094277,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.632538,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.331916,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.498664,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.3169319999999995,
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
          "id": "c0eaa30e309570ed4b90621a8a108276a5a48aae",
          "message": "fix(ci): run the Component follower on Node 24 (#158)",
          "timestamp": "2026-11-17T10:56:30+01:00",
          "tree_id": "38e326aa66482f4f3e5bb8faadfa4c7253b29c8f",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c0eaa30e309570ed4b90621a8a108276a5a48aae"
        },
        "date": 1794909465597,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.091774,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.6535709999999995,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.564595,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.5354539999999999,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.531339,
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
          "id": "36a8f8f5297d4ce9870eca3934439ac0439076b3",
          "message": "ci: log the Component follower at debug level (#163)",
          "timestamp": "2026-11-17T11:06:44+01:00",
          "tree_id": "becfb6dd040d59e672412bc5074faa18036b91ad",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/36a8f8f5297d4ce9870eca3934439ac0439076b3"
        },
        "date": 1794910084425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.102108,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.627328,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.736673000000003,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.5254649999999998,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.820186,
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
          "id": "93ad249cbeae6c1d2b9ff3bbddd5771ace8c8903",
          "message": "ci(release): use a prebuilt cargo-cyclonedx and cache cargo-packager (#165)",
          "timestamp": "2026-11-17T11:16:52+01:00",
          "tree_id": "b26a1ac35c8fa4a37e051a71ee8f5f416ae43b36",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/93ad249cbeae6c1d2b9ff3bbddd5771ace8c8903"
        },
        "date": 1794910693720,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.514004,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.534904,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.857954,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.191888,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.963465000000001,
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
          "id": "56d27193f0328acd0cc5349afc196110117a8a67",
          "message": "fix(ci): give the Components follower's commands the cargo registry token (#182)\n\nThe token the package proxy step sets is not visible to commands Renovate starts, so the bump command could not fetch crates ('no token found for jorisjonkers-packages'). Pass only that one variable.",
          "timestamp": "2026-11-17T11:37:43+01:00",
          "tree_id": "b1eac3e48127c857e38b43d4bc02154e9e1f3b2f",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/56d27193f0328acd0cc5349afc196110117a8a67"
        },
        "date": 1794911938371,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.080328,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.621842999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.304859999999998,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.488261,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.800451,
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
          "id": "7ede4bcb28458f5ff32edfba20348a5647f8d8d0",
          "message": "docs(adr): components are incorporated; where packages are published (#183)",
          "timestamp": "2026-11-17T12:01:40+01:00",
          "tree_id": "c4cd7a09e529dd78b2657007c67ed05d8506e144",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7ede4bcb28458f5ff32edfba20348a5647f8d8d0"
        },
        "date": 1794913379193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.526224,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.288581,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.704197,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.141251,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.172638,
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
          "id": "b05d8e21c4abd323d07055c90cb87731acc474ed",
          "message": "fix(ci): read the cargo registry token from the job environment (#184)\n\nThe follower's bump command now sees the token but the registry answers 401: the proxy step writes the token in the form cargo expects, and the secret is not. Pass the variable that step set.",
          "timestamp": "2026-11-17T12:06:53+01:00",
          "tree_id": "229fd475a34a0faee7d0ef2374e05344b27e0d5a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/b05d8e21c4abd323d07055c90cb87731acc474ed"
        },
        "date": 1794913675089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.926613,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.663411,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.663492,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.715333,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.09866,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1e0b4a37d44130e6ec331e0281cc031a393fb890",
          "message": "chore(deps): bump nv-rs to v0.5.0 (#185)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-17T11:14:12Z",
          "tree_id": "52a061e34e8be7454021c02408f0e8d85303fc86",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1e0b4a37d44130e6ec331e0281cc031a393fb890"
        },
        "date": 1794914125293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.4233959999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.337844,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.126925,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.041991,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 8.648776999999999,
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
          "id": "e8acb88cd8ba4be38f7a01d3f8cba267b42ef4c5",
          "message": "fix(gui): say ready to play only when the game can start (#186)\n\n* wip: truthful ready to play\n\n* wip: readiness gates Play, settles reads without settings, explains the missing step\n\n* fix(gui): answer the Play press on either dialog answer, name a step once, settle reads without an Instance",
          "timestamp": "2026-11-17T13:32:52+01:00",
          "tree_id": "aedf2f9a1a0f55a790a3cc3bd7463a6606bb2988",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e8acb88cd8ba4be38f7a01d3f8cba267b42ef4c5"
        },
        "date": 1794918848879,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.1194499999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.684779,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.810964,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.548578,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.894111,
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
          "id": "da82eeaf1c7a38000963235bb768e59e845119f0",
          "message": "feat(gui): open the TTW download page and find the package for me (#187)",
          "timestamp": "2026-11-17T18:42:23+01:00",
          "tree_id": "9baa0ce5dd0cd0d856c9077120dcef4ee11f1a4b",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/da82eeaf1c7a38000963235bb768e59e845119f0"
        },
        "date": 1794937423976,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.946116,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.661296,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.424344,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.7122079999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.741294,
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
          "id": "66275e383f60bf1ac8af4eadf226624af9a7eb04",
          "message": "fix(gui): show one version and drop known-good from the UI (#188)\n\n* fix(gui): show one version and take the known-good set out of the UI\n\n* test(gui): leave the network answer of the update check out of the registry test\n\n* fix(gui): follow main's profile and update seams after the merge",
          "timestamp": "2026-11-17T18:55:08+01:00",
          "tree_id": "9d3ac96624bc20389eb7d48a183061eb91afeb3c",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/66275e383f60bf1ac8af4eadf226624af9a7eb04"
        },
        "date": 1794938176147,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.959583,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.730478,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.529195,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.6329839999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.001873,
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
          "id": "51e787f8248f5a0ff0ffcc2c9614f92a6282804e",
          "message": "feat(gui): copy the details of every error (#189)\n\n* feat(gui): copy the details of every error\n\n* wip\n\n* fix(gui): clear the failed status once the settings read again",
          "timestamp": "2026-11-17T19:01:35+01:00",
          "tree_id": "8da8b7d821c8d1ecd497814ab1fdd88f43f22a61",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/51e787f8248f5a0ff0ffcc2c9614f92a6282804e"
        },
        "date": 1794938567608,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.092964,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.625882000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.567262,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.49378,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.85272,
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
          "id": "d412a152916d93b592777c0ae6f731c95abbe722",
          "message": "fix(gui): search Downloads once without a home folder (#192)\n\n* fix(gui): search the Downloads folder once when there is no home folder\n\n* test(gui): wait for the update answer on screen instead of a fixed number of frames",
          "timestamp": "2026-11-17T19:19:15+01:00",
          "tree_id": "9c563a45f7dfb62340af871f7ff571bd867353b7",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d412a152916d93b592777c0ae6f731c95abbe722"
        },
        "date": 1794939629019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.085582,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.628972,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.468883,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.482489,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.12256,
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
          "id": "7b99b8c017e731e0b44969beb7979cbcff496919",
          "message": "feat(gui): no command lines on Versions, Home and dialogs (#200)\n\n* feat(gui): show no command lines on Versions, Home, Settings and dialogs\n\n* fix(gui): say the engine is installed without its tool name",
          "timestamp": "2026-11-17T20:07:15+01:00",
          "tree_id": "1e1743457f2e6e594385b5debae6bb7688975eb1",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7b99b8c017e731e0b44969beb7979cbcff496919"
        },
        "date": 1794942517249,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.089311,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.631049,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.759366,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.500499,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.62078,
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
          "id": "116109e3040e16c99ed60300708d23392c16aa28",
          "message": "feat(gui): show no command lines on the LOD page (#217)",
          "timestamp": "2026-11-17T20:33:41+01:00",
          "tree_id": "9623928f0af04f2b8f47444115a7cf8c899bf47e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/116109e3040e16c99ed60300708d23392c16aa28"
        },
        "date": 1794944096436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.9519069999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.725857999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.200641,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.74914,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.400864,
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
          "id": "625bf7fd2084c29d8ca6d3c324be0c11579de9c2",
          "message": "feat(gui): show no command lines on the Load order page (#218)",
          "timestamp": "2026-11-17T20:58:45+01:00",
          "tree_id": "f62ce93c54b2d5d8cfa98e13021f824b2c99fdd7",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/625bf7fd2084c29d8ca6d3c324be0c11579de9c2"
        },
        "date": 1794945601979,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.1594680000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.898625,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 22.309359,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.516558,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 7.77938,
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
          "id": "f4fa06a471fa1141a5a45a8d10675b4f19832bbc",
          "message": "feat(gui): show no command lines in the TTW flow (#219)",
          "timestamp": "2026-11-17T21:25:14+01:00",
          "tree_id": "b52d4afb92507e03617d578485a5f481bbdafb02",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f4fa06a471fa1141a5a45a8d10675b4f19832bbc"
        },
        "date": 1794947178610,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.7029720000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.5465,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.974896,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.90436,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.074744,
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
          "id": "5280df257ef159f6bb031bdd5ffb2e881bd797ca",
          "message": "feat(gui): show no command lines in the setup wizard (#220)",
          "timestamp": "2026-11-17T21:43:34+01:00",
          "tree_id": "3f78449dd4a8ad5e41b0e668dd09edbab66ddee8",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5280df257ef159f6bb031bdd5ffb2e881bd797ca"
        },
        "date": 1794948283594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.6452630000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.281821,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.483462,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.771782,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.522285,
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
          "id": "3ffdd65076d01fa13d7eb63987778bfadeae65ef",
          "message": "fix(gui): use the found TTW package only after it is saved (#221)",
          "timestamp": "2026-11-17T22:00:27+01:00",
          "tree_id": "a25ab3ef195d74e8eda8e7f21cca05b2169aa136",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/3ffdd65076d01fa13d7eb63987778bfadeae65ef"
        },
        "date": 1794949295796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.95028,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.642768,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.40583,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.6163289999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.364134,
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
          "id": "edffb368f375ab9caaaa00447bca6cb1d8028612",
          "message": "fix(gui): end a chained job as its second command ended (#222)",
          "timestamp": "2026-11-17T22:27:03+01:00",
          "tree_id": "29e7ae8cfbd58b5b04dd0fd9728bcea1320cb4c6",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/edffb368f375ab9caaaa00447bca6cb1d8028612"
        },
        "date": 1794950895305,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.100611,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.631698,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.416744,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.52001,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.809026,
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
          "id": "dca2e32ebf2849a021e140471561fa1d526cd5c6",
          "message": "feat(gui): check a TTW package by name, size and contents (#223)",
          "timestamp": "2026-11-17T22:31:28+01:00",
          "tree_id": "f0033e999d770137178a3315b41b0b958aa7464a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/dca2e32ebf2849a021e140471561fa1d526cd5c6"
        },
        "date": 1794951165946,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.105099,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.643643,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.296305,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.536082,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.9700370000000005,
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
          "id": "ba7ef9341d31caeedae4405539d3c2750e0c1ea3",
          "message": "feat(gui): tell when a download is complete (#224)",
          "timestamp": "2026-11-17T22:52:26+01:00",
          "tree_id": "145557f2581ca8b32fa38ffa33e19483ec68d332",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ba7ef9341d31caeedae4405539d3c2750e0c1ea3"
        },
        "date": 1794952419084,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.9649590000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.785438,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.450134000000006,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.765145,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.396513,
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
          "id": "0d3065ecd0cacedcbd5efa104372981aa1fb2687",
          "message": "test(gui): fail when a page shows a command line (#225)",
          "timestamp": "2026-11-17T23:08:17+01:00",
          "tree_id": "c1e33b37746cf9b8bb72e2dced285afa4c8713b5",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/0d3065ecd0cacedcbd5efa104372981aa1fb2687"
        },
        "date": 1794953376497,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.3545939999999996,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.544429,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.83983,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.873528,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.092968,
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
          "id": "26763cc8d61d5626ce4b9f8642494828cd601281",
          "message": "test(gui): check that every page fits its window and document the smallest size (#226)",
          "timestamp": "2026-11-17T23:19:14+01:00",
          "tree_id": "9aaa447290844f292ebbf430ca82768faa3e63df",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/26763cc8d61d5626ce4b9f8642494828cd601281"
        },
        "date": 1794954038102,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.394464,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 5.969921,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 27.915257,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.0565100000000003,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 8.972268,
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
          "id": "acc63e16a9f24d0e273071cc02ba075f0b662634",
          "message": "feat(gui): find the newest valid TTW package over several folders (#228)",
          "timestamp": "2026-11-17T23:27:59+01:00",
          "tree_id": "5bddaf2f92892402357dbacb6b8bbf0cfdf73228",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/acc63e16a9f24d0e273071cc02ba075f0b662634"
        },
        "date": 1794954559857,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0801260000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.632425,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.628729,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.496547,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.028468999999999,
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
          "id": "e4548b08fb25dd29ca02dce922f2be3cc8e443c6",
          "message": "fix(gui): fit the Home page to the smallest window (#229)",
          "timestamp": "2026-11-17T23:40:45+01:00",
          "tree_id": "079ddbed73b9d89fd9463306b46f52af5f13fec9",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e4548b08fb25dd29ca02dce922f2be3cc8e443c6"
        },
        "date": 1794955325716,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0968600000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.597708,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.29882,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.502834,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.944703,
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
          "id": "4dbcce1997ab9c9c36bdaa680d62bf2dad880397",
          "message": "fix(gui): fit the Versions page to the smallest window (#230)",
          "timestamp": "2026-11-17T23:57:14+01:00",
          "tree_id": "640492fdac62adf80ea788c0c23cfb31d4eab779",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/4dbcce1997ab9c9c36bdaa680d62bf2dad880397"
        },
        "date": 1794956293868,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.58133,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.036002000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.395508,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.680968,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.201825,
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
          "id": "030560cfe02005c79a232a77e2a510d24848d4b6",
          "message": "fix(gui): fit the LOD page to the smallest window (#231)",
          "timestamp": "2026-11-18T00:50:13+01:00",
          "tree_id": "047a1ddfd82fdafdc99c0d274c3f57098a8883f9",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/030560cfe02005c79a232a77e2a510d24848d4b6"
        },
        "date": 1794959482651,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.945755,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.659475,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.371735,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.715536,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.424106,
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
          "id": "da85974c671b0c8417fd9815cda5f384fc6e8d3b",
          "message": "fix(gui): fit the Load order page to the smallest window (#232)\n\n* fix(gui): fit the Load order page to the smallest window\n\n* chore: format the known list after the merge",
          "timestamp": "2026-11-18T00:57:10+01:00",
          "tree_id": "b4ca896d4a8999ef8481a925d2a6ae96555c54d1",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/da85974c671b0c8417fd9815cda5f384fc6e8d3b"
        },
        "date": 1794959904070,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.1256140000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.702123,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.871173000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.497751,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 7.755013,
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
          "id": "e7a57b772b0d4924742b2e342e00a86a630ec653",
          "message": "fix(gui): fit the Settings page to the smallest window (#233)",
          "timestamp": "2026-11-18T01:10:05+01:00",
          "tree_id": "b5d8765f6bb72869e3e6f35b2d838556f6f83717",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e7a57b772b0d4924742b2e342e00a86a630ec653"
        },
        "date": 1794960686386,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.334867,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.459902,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.602594,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.782526,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.073091999999999,
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
          "id": "8a1390e2dc863456e880f05b02f525ff69acb332",
          "message": "fix(gui): fit the TTW flow page to the smallest window (#234)\n\n* fix(gui): fit the TTW flow page to the smallest window\n\n* chore: snapshots after the merge",
          "timestamp": "2026-11-18T01:27:32+01:00",
          "tree_id": "0ba2a47edea9cc22e048c9016eb5261bef6260cb",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8a1390e2dc863456e880f05b02f525ff69acb332"
        },
        "date": 1794961734834,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.12757,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.700626000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.710922,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.5077109999999998,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.311735,
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
          "id": "e4b5123777ef6f950da933dc1431e058deb85c76",
          "message": "test(gui): see the page change after a fixed number of frames (#236)",
          "timestamp": "2026-11-18T01:43:37+01:00",
          "tree_id": "196d308559f969c91706ea55c469dd869734e105",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e4b5123777ef6f950da933dc1431e058deb85c76"
        },
        "date": 1794962685614,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.73093,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.966167999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 33.397973,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.101528,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.614391,
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
          "id": "d9e9cb29fcd7a9c047cb1f1397d675024a663b67",
          "message": "fix(gui): fit the Setup wizard page to the smallest window (#237)",
          "timestamp": "2026-11-18T01:50:58+01:00",
          "tree_id": "5a1b3d5d742074ec168f358ba2084f37564e61c5",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d9e9cb29fcd7a9c047cb1f1397d675024a663b67"
        },
        "date": 1794963133489,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.949484,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.729189,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.018418999999994,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.6343369999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.428954,
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
          "id": "398676874caf293652ae83685d5aa13d58633a59",
          "message": "feat(gui): watch Downloads for the TTW package once the user agrees (#240)",
          "timestamp": "2026-11-18T02:05:16+01:00",
          "tree_id": "de0c5eed1f67256b7a4d2826587c3c6c8bdcf2af",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/398676874caf293652ae83685d5aa13d58633a59"
        },
        "date": 1794963991958,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.3807,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.49745,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.407424000000006,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.5857600000000005,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.920178,
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
          "id": "7399eb55925a2174614d072eba684406d24a77fd",
          "message": "feat(gui): search the folders Settings points at for the TTW package (#241)",
          "timestamp": "2026-11-18T02:19:41+01:00",
          "tree_id": "43e4d93c9fc39330c2ef5e64a5fb6889f4dd6d2b",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7399eb55925a2174614d072eba684406d24a77fd"
        },
        "date": 1794964853665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.1253199999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.711663,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.874036999999998,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.505237,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.1057760000000005,
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
          "id": "7f277966e02b21e29dbfe98c0054fe448445155b",
          "message": "feat(gui): unpack a TTW package that arrives in an archive (#242)\n\n* feat(gui): unpack a TTW package that arrives in an archive\n\n* fix(gui): keep watching when a settled package does not check out yet",
          "timestamp": "2026-11-18T02:33:04+01:00",
          "tree_id": "a1e1312263968fbce4a389f7bbc54ba54b5d93fc",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7f277966e02b21e29dbfe98c0054fe448445155b"
        },
        "date": 1794965656682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.128998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.661592,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.583175,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.516512,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.119715,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6b77a9656f71c4489ffbb19f0e81bb0f8568a744",
          "message": "fix(deps): pin rust crate ctrlc to =3.5.2 (#243)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T06:04:50Z",
          "tree_id": "9a868f3c4725190bba0f87efe27123fd52600b95",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/6b77a9656f71c4489ffbb19f0e81bb0f8568a744"
        },
        "date": 1794981955312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.5679299999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.028236000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.209008,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.7536739999999997,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.921479,
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
          "id": "08397846d1b7950343bcd27caeb8b0ff57c2d42d",
          "message": "docs(adr): name each package after its repository (#244)",
          "timestamp": "2026-11-18T07:13:17+01:00",
          "tree_id": "da3b5e0e32e484826d248629966d20728ec93769",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/08397846d1b7950343bcd27caeb8b0ff57c2d42d"
        },
        "date": 1794982477493,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.3366979999999997,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.346349,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 44.918118,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.728509,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.346463,
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
          "id": "b51f6b8abc56d19d25227dd096e85d24ed978d55",
          "message": "build: Rust 1.100 and a 7-day crate cooldown (#245)\n\nRenovate now waits seven days before proposing a crates.io release, as Cargo already does for resolution (.cargo/config.toml). The organisation's own git dependencies keep their existing rule. The toolchain and rust-version were already 1.100.",
          "timestamp": "2026-11-18T07:28:15+01:00",
          "tree_id": "b350aa60c2f1e1d488ebb08f9b6e854511f83ba2",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/b51f6b8abc56d19d25227dd096e85d24ed978d55"
        },
        "date": 1794983381161,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.495581,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.259196,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.564504,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.1035950000000003,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.674062000000001,
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
          "id": "fa16ee176d58f09ab6052913a4713c5d74baa6c1",
          "message": "ci: build without the package proxy (#246)\n\nBuilds fetch crates from crates.io directly. The estate packages proxy is no longer used, so every setup-package-proxy step is removed from the workflows, and the PACKAGES_PROXY_TOKEN secret is no longer passed to the repository hygiene guard.\n\nThe 7-day cooldown in .cargo/config.toml and the Renovate minimumReleaseAge for crates are the protection that the proxy's minimum age used to give.\n\nThe hygiene guard's package-proxy-guard input is removed too, so the shared guard falls back to its default report mode. It now reports installs outside the proxy without failing the job.",
          "timestamp": "2026-11-18T07:38:15+01:00",
          "tree_id": "1d84a3e7fcc327f6542116497a44ca457d5d128c",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/fa16ee176d58f09ab6052913a4713c5d74baa6c1"
        },
        "date": 1794984004424,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.097152,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.637251,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.754373,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.49499,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.745789,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a9cc69bedf052bc839bb96e39495a73d65529443",
          "message": "chore(deps): bump lodwright-rs to v0.4.0 (#247)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T07:46:54+01:00",
          "tree_id": "efeda49e98ca4790da46b8feec6d264cd92251d9",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a9cc69bedf052bc839bb96e39495a73d65529443"
        },
        "date": 1794984491897,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.083642,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.675396,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.397464000000003,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.495524,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.127673,
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
          "id": "1d727158e3f2153ed7512f600b9a8a1033c5debd",
          "message": "chore(deps): bump the Components to their new releases (#252)\n\n* chore(deps): bump nvmod to v0.7.0\n\n* chore(deps): bump ttwright-rs to v0.4.0\n\n---------\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T08:31:27+01:00",
          "tree_id": "d122f08354b8edf12b0c5e61760deb22eefdbcbb",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1d727158e3f2153ed7512f600b9a8a1033c5debd"
        },
        "date": 1794987168303,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.3249999999999997,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.552387,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.888645,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.650543,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.557649000000001,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d4f9cfe1877a63120252f8ec50f539a858a05d09",
          "message": "chore(deps): bump nv-rs to v0.6.0 (#248)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>\nCo-authored-by: J.W. Jonkers <info@jorisjonkers.dev>",
          "timestamp": "2026-11-18T07:49:28Z",
          "tree_id": "3c2e973b90b6f4b68f595d408a4b5b225a47756a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d4f9cfe1877a63120252f8ec50f539a858a05d09"
        },
        "date": 1794988233443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.727696,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.679674,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 31.347696000000003,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.042682,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.002932999999999,
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
          "id": "a38ae6f76b4dfbed418f20b68513b35e39bfe8c1",
          "message": "docs(adr): the game run is the one child process, started from raul's own binary (#261)",
          "timestamp": "2026-11-18T09:11:23+01:00",
          "tree_id": "9748cd28a6ffa315c9d4db8dea480a37794055f3",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a38ae6f76b4dfbed418f20b68513b35e39bfe8c1"
        },
        "date": 1794989570843,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.411546,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.677956,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 47.628471,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.921435,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.761033000000001,
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
          "id": "0416fa826ec49337a2c928b55e5b288f71cb93ed",
          "message": "refactor: remove the code that reads the Known-good set (#257)\n\n* refactor: remove the code that reads the Known-good set\n\n* ci: follow the Components without the Known-good set\n\n* test: cover the text and the usage error of raul version",
          "timestamp": "2026-11-18T09:14:44+01:00",
          "tree_id": "e0803a0e1b657763c4e7976217bf63f5fe494fbd",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/0416fa826ec49337a2c928b55e5b288f71cb93ed"
        },
        "date": 1794989766847,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.088436,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.678947999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.337836999999997,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.496837,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.99292,
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
          "id": "8099a86a059490982fe476bdd34e4d7dd5a22399",
          "message": "docs: the submodule pins are the set of Component versions (#262)\n\n* refactor: remove the code that reads the Known-good set\n\n* ci: follow the Components without the Known-good set\n\n* docs: the submodule pins and the lock files are the set of Component versions",
          "timestamp": "2026-11-18T09:21:59+01:00",
          "tree_id": "a02d420531f3523adcc43c5dedf0203d0a721a2a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8099a86a059490982fe476bdd34e4d7dd5a22399"
        },
        "date": 1794990199892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.1375300000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.7002440000000005,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.885516,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.502532,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.1487,
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
          "id": "02ffd228fc375dae15f97a5edf24c53f3ebf2ffd",
          "message": "feat(drivers): run ttwright's install in process (#264)\n\n* feat(drivers): run ttwright's install in process\n\n* ci: build in a short folder on Windows, where the path limit stops a native dependency\n\n* fix(deps): keep cc at 1.2.62, which builds directxtex on Windows\n\n* build(deny): allow the licences ttwright-core's dependencies use\n\n* build(deny): allow LGPL-3.0 under the id the LAME bindings declare",
          "timestamp": "2026-11-18T10:27:21+01:00",
          "tree_id": "2b3a55b0bb8219c880720d38cd7a6170a9bc6d14",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/02ffd228fc375dae15f97a5edf24c53f3ebf2ffd"
        },
        "date": 1794994228428,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.307829,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.472627,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.469404,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.08869,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.538307,
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
          "id": "46add0211442f008c1fa79729581c1cb3e1dd8b8",
          "message": "feat(gui): add InProcessExec, an Exec that runs raul in this process (#268)\n\n* feat(gui): add InProcessExec, an Exec that runs raul in this process\n\n* fix(gui): reset the cancel flag on every path and keep streamed output when a run is stopped\n\n* build(gui): lock ttwright-core's dependencies, with cc kept at 1.2.62\n\n* build(deny): allow the licences ttwright-core's dependencies use in the GUI workspace",
          "timestamp": "2026-11-18T10:52:24+01:00",
          "tree_id": "138e45ad946650415cc6cf78f2bf6bc2133ae9f9",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/46add0211442f008c1fa79729581c1cb3e1dd8b8"
        },
        "date": 1794995630925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.447154,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 11.193641000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.957001,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 6.533551,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.54221,
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
          "id": "292c68557341fc5fef9e1c45c3497b2ce49d136a",
          "message": "docs(adr): raul orchestrates; nvmod manages mods; no standalone nvmod setup or GUI (#277)\n\nRecord the owner decision of 2026-11-18 as ADR-0019. It supersedes the\nconflicting parts of ADR-0002, ADR-0003, ADR-0016's nvmod row and REORG\nsection 12 decision 5. REORG gains a status section: M1 is complete and\nM2 runs now in the new order.",
          "timestamp": "2026-11-18T11:02:05+01:00",
          "tree_id": "c18c0087b8ee2799a17faff86c509beb3ece884e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/292c68557341fc5fef9e1c45c3497b2ce49d136a"
        },
        "date": 1794996206821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.8771470000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.624416,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.935140999999994,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.522418999999999,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 18.68691,
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
          "id": "d193bb07c887fe23f6ba949f536e71089ee89236",
          "message": "feat(drivers): install TTW through ttwright linked into raul (#270)",
          "timestamp": "2026-11-18T11:03:28+01:00",
          "tree_id": "5b6f57d3c2bf296a9aa46e5f5864d41a80c6618b",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d193bb07c887fe23f6ba949f536e71089ee89236"
        },
        "date": 1794996302542,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.449313,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.426318,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.857257,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.1475359999999997,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.806828999999999,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8809928fc452b8ef6937c692def5012b2c2fa15a",
          "message": "chore(main): release 0.4.0 (#164)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T12:12:27+01:00",
          "tree_id": "758017e142a51c9e88528e6ba7e859b7dc96c316",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8809928fc452b8ef6937c692def5012b2c2fa15a"
        },
        "date": 1795000419699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.904277,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.695319,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.58082,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.136593,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.918796,
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
          "id": "66c5350f34ee492ac78082064d03a22020bd1b8f",
          "message": "build: move to nvmod v0.8.0, nv-rs v0.7.0 and lodwright-rs v0.4.1 (#279)",
          "timestamp": "2026-11-18T12:29:32+01:00",
          "tree_id": "e62cf251fcb61acf5f75e6519459a9866b544058",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/66c5350f34ee492ac78082064d03a22020bd1b8f"
        },
        "date": 1795001445491,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.6687500000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.656325000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 31.048927,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.257897,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.296244,
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
          "id": "adb9498e479547d0cd2976a6c2f33cdca5ffc8ca",
          "message": "build: move to nvmod v0.8.1, ttwright-rs v0.4.2 and lodwright-rs v0.4.2 (#281)",
          "timestamp": "2026-11-18T13:24:30+01:00",
          "tree_id": "64c8ba003f1edd1a9ac794547acdb93ed43ee07c",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/adb9498e479547d0cd2976a6c2f33cdca5ffc8ca"
        },
        "date": 1795004761414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.281183,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.459090000000002,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.617309000000006,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.1106620000000005,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.457913,
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
          "id": "ab64d4bd339637f46e7fa91f629b291583c2b2ac",
          "message": "refactor(runner): extend the process guard to every crate and tool (#282)\n\nThe guard that keeps std::process::Command in raul-runner now scans the tests and build scripts of every Toolbox crate, the GUI workspace's member crates and the developer tools. The few files that start the built raul binary or run git are allow-listed, each with a one-line reason, and an entry that no longer starts a process fails the test.",
          "timestamp": "2026-11-18T18:46:36+01:00",
          "tree_id": "b3642fb70646de1283ca549369b8c3be0aa08a66",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ab64d4bd339637f46e7fa91f629b291583c2b2ac"
        },
        "date": 1795024083757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.303295,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.473905,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.720265000000005,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.087737,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.057746999999999,
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
          "id": "6a32f9574e6947fad24c47cee1494aa0a97be04c",
          "message": "refactor: drop the ttwright program path (#283)\n\n* refactor: drop the ttwright program path (round 1)\n\n* test: drive raul ttw through the in-process install\n\n* refactor: keep the ttwright event-line reader to the tail test and format\n\n* test(gui): drive the TTW flow through the in-process install\n\n* docs: say that ttwright runs in process, and make the cancel tests deterministic\n\nttwright runs in raul's own process since the program path was removed, so the descriptions of raul ttw and its result Contract now say so. The error codes nothing produces any more are marked as kept for older result documents; no code, field or schema version changed.\n\nThe cancel tests wait for the first asset ttwright writes, polled every 5 ms, instead of the third file, so the stop lands early in a 4000-asset install that takes seconds. Unused TTWRIGHT_RESULT_VERSIONS and a duplicate rebinding are removed, and a test pins that --ttwright is refused as an unknown option.\n\n* docs: say that ttwright is linked and lodwright still runs as a program\n\n* fix(drivers): stop the watcher thread when the in-process install panics\n\n* test: tighten the ttw cancel assertions and say a timeout ends as cancelled",
          "timestamp": "2026-11-18T19:45:01+01:00",
          "tree_id": "ff737c654ae5311f1eaa40736813c486e0890bce",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/6a32f9574e6947fad24c47cee1494aa0a97be04c"
        },
        "date": 1795027583699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.170147,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 9.608764,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 38.145443,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.286476,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.332208,
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
          "id": "1047abe506acbf8390bd8661d877ea0e9f06c5e5",
          "message": "build: move to nvmod v0.9.0, ttwright-rs v0.4.3 and loot-rs v0.6.2 (#284)\n\nnvmod v0.9.0 carries the doctor, conflicts and tree jobs and links loot-rs v0.6.2, so loot-rs moves with it. ttwright-rs v0.4.3 holds cc below 1.9 for Windows builds and no longer uses chrono, so the advisory exception for it leaves deny.toml.",
          "timestamp": "2026-11-18T20:34:13+01:00",
          "tree_id": "d5f1fc4a5b01010bfb55b5083e53b88beeee7e2b",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1047abe506acbf8390bd8661d877ea0e9f06c5e5"
        },
        "date": 1795030670963,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.318979,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.531749999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.091611,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 7.1382959999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.491869,
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
          "id": "fe96385068501c9e3357ced31b8d279e76c3ef15",
          "message": "feat(drivers): generate LOD through lodwright in process (#285)\n\n* feat(drivers): generate LOD through lodwright's library API in process\n\nraul calls lodwright-api from its own process instead of starting the\nlodwright program. Progress from the library becomes raul events, and a\nstop request or the timeout cancels the call through its cancel token.\nA watcher thread ends with the call, also when the call panics, through\na drop guard. The Profile's request is passed as a typed request, so no\nrequest file is written.\n\nraul lod generate loses --lodwright, the bundled program lookup and the\nprogram's version check. The TTW event-tail helpers that only the\nprogram path used are removed with their test.\n\n* test: run raul lod generate against the linked lodwright\n\nThe LOD tests used a fake lodwright shell script. They now generate a\nsynthetic world with lodwright's test kit: success, an up-to-date second\nrun, a replaced Mod, refusals reported by lodwright, a dry run and bad\ninputs, and that nvmod never runs. Cancel and resume run in process; a\nstop request is tested in its own binary because the stop flag is\nprocess wide. The GUI's LOD flow runs the same way, without a fake\nprogram, and its registry expectation for LOD generation follows the\nlinked generation. The real-program test is removed with its feature.\n\n* docs(contracts): describe lodwright as called in process\n\nDescriptions and the contract README only; the schema's shape, codes and\nversion are unchanged. The CI step that ran the real lodwright program\nis removed with the feature it used.\n\n* fix(drivers): keep the symlink tests unix-only and allow the synthetic-height casts\n\n* build(gui): allow esplugin's GPL-3.0 licence for the test-only lodwright-testkit\n\n* test(gui): size the stop test's world for a coverage-instrumented runner\n\n* build: allow esplugin's GPL-3.0 licence for the test-only lodwright-testkit",
          "timestamp": "2026-11-18T21:31:45+01:00",
          "tree_id": "81535c3f06f4b91e348b9f47abd96a1ef4f7e47e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/fe96385068501c9e3357ced31b8d279e76c3ef15"
        },
        "date": 1795034136427,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.25265,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.357417,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.532825,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.013914000000001,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.300099,
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
          "id": "ac540fba3164d54db4db552d47f85469892a37b2",
          "message": "feat(core): setup job that plans and runs nvmod's mod jobs (#288)\n\n* feat(core): add the setup job that plans and runs nvmod's mod jobs in order\n\n* fix(core): clear the stop bridge on unwind and document the engine boundary\n\n* build(gui): record the new raul-core dependencies in the lock file",
          "timestamp": "2026-11-18T22:02:14+01:00",
          "tree_id": "3d3579d505b1732b34bcf0363e00b78fff916765",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ac540fba3164d54db4db552d47f85469892a37b2"
        },
        "date": 1795035819243,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.914342,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.763074,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.063001,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.094475999999999,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.21866,
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
          "id": "c8505b2716b3e529c658c76a85e17fc2c1848cf7",
          "message": "fix(ci): let the Component follower see the pinned releases (#290)\n\n* fix(ci): give the Component follower a tracked pin file to read\n\nRenovate lists files with git and never sees inside a submodule, so the managers that read components/<name>/.release-please-manifest.json found 0 package files. The pinned release is now also written in components/<name>.version, which bump-component.sh updates and check-submodules.sh compares with the submodule.\n\n* fix(ci): fail when a followed Component has no pin file, and write the pin safely\n\n* docs(ci): say that the submodule check needs jq",
          "timestamp": "2026-11-18T22:20:27+01:00",
          "tree_id": "a84e9048911e7243b39386e70a0d0bf1dc5b7b56",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c8505b2716b3e529c658c76a85e17fc2c1848cf7"
        },
        "date": 1795036908989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.9286159999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.758327,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.383465,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.159988,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.126009,
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
          "id": "c860246f7f70d46f95f346779d2f896d653311ab",
          "message": "feat(gui): run Undo as a direct job under the Instance lock (#291)\n\n* feat(gui): run Undo as a direct job under the Instance lock\n\n* fix(gui): refuse an Undo with no Instance and say when the lock is held by this program\n\n* test(gui): the lock is held while a direct job runs",
          "timestamp": "2026-11-18T22:25:56+01:00",
          "tree_id": "4f106e5d4ba7471f68473c16afa20abaadbc0593",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c860246f7f70d46f95f346779d2f896d653311ab"
        },
        "date": 1795037250168,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.2871129999999997,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.440227,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.813072000000005,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.144948,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.825656,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "87da4f54a26eda83cac0398c02ef4c30bd11817a",
          "message": "chore(deps): bump nv-rs to 0.8.0 (#293)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T21:27:28Z",
          "tree_id": "20905560c32fd66b6cfc4515257143a786f6ebeb",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/87da4f54a26eda83cac0398c02ef4c30bd11817a"
        },
        "date": 1795037348818,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.297557,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.52569,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.584058,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.080365,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.139863,
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
          "id": "a1fbb350d4507b174ff2066cf1b91430f3068b07",
          "message": "feat(core): setup resume journal taking over nvmod setups (#292)\n\n* feat(core): setup resume journal that takes over an unfinished nvmod setup\n\n* fix(core): journal takeover order, plan check and complete flag",
          "timestamp": "2026-11-18T22:38:38+01:00",
          "tree_id": "ca16ae5cdca381d371a8aa4d894e77960f93e4d7",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a1fbb350d4507b174ff2066cf1b91430f3068b07"
        },
        "date": 1795038145767,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.4257300000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.099768,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.445417,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.075771,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 8.877555000000001,
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
          "id": "b70f398ad9fadaf4329c2cedc228a78d95d00b3b",
          "message": "feat(gui): run the read-only actions as direct jobs (#295)\n\n* feat(gui): run conflicts and tree as direct jobs\n\n* feat(gui): run the Doctor as a direct job\n\n* refactor(gui): key direct jobs by the registry command id\n\n* feat(gui): run verify as a direct job\n\n* feat(gui): run the plugin list as a direct job\n\n* test(gui): satisfy clippy in the direct read tests\n\n* test(gui): drop an unused import\n\n* fix(gui): keep the usage error of a bad severity and test the flags of the read jobs\n\n* build: keep the nv-rs pin of main\n\n* fix(gui): a failed read is a refusal, not a rolled-back step",
          "timestamp": "2026-11-18T23:11:50+01:00",
          "tree_id": "89d47c0cfd3bf996d9d7d5cd4b4f4a95e18b4c97",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/b70f398ad9fadaf4329c2cedc228a78d95d00b3b"
        },
        "date": 1795039999663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.6696739999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.610892,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.62425,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.260105,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.110782,
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
          "id": "e26c5347568406f152fa8aade8242d25061eb982",
          "message": "feat(core): engine step installs nv-rs in process (#297)\n\n* feat(core): engine step installs nv-rs in process through the install library\n\n* fix(core): stop the engine install on cancel and read NVMOD_CARGO\n\n* test(core): run the installed fake engine through the runner",
          "timestamp": "2026-11-18T23:37:27+01:00",
          "tree_id": "052f307f0ee51648426acf92caee3b7c83953e27",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e26c5347568406f152fa8aade8242d25061eb982"
        },
        "date": 1795041546216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.282171,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.476999000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.103402,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.1650849999999995,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.568117,
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
          "id": "f3bc9de88d68c8e6bf9df60f7d9a1f66deaddb25",
          "message": "feat(deps): follow nvmod and drop the extract tool (#299)\n\n* feat(deps): follow nvmod in reading archives in process and drop the extract tool\n\n* fix(gui): stop offering rar archives and an outside tool for TTW packages\n\n* test(core): an old setup request with extract_tool still reads",
          "timestamp": "2026-11-19T00:04:13+01:00",
          "tree_id": "35ff23a49a54ee4f6266a2ef2bcc187af5577c56",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f3bc9de88d68c8e6bf9df60f7d9a1f66deaddb25"
        },
        "date": 1795043162854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.422276,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.553239,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 31.726676,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.1661319999999997,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.687206999999999,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c8ccceed2d197de04f23ed993e33a001a2197210",
          "message": "chore(deps): bump nvmod to 0.10.0 (#301)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T23:37:16Z",
          "tree_id": "b2591dc7876f40a0cbd0bc5b25ce057265e92492",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c8ccceed2d197de04f23ed993e33a001a2197210"
        },
        "date": 1795045129722,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.446852,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 11.235261,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.931656000000004,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 6.372193,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.238541,
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
          "id": "173d464eda5245cb3db6894ea338dbe4291e15cd",
          "message": "feat(gui): run Mods change, target and profile creation in process (#300)\n\n* feat(gui): run the Mods change, target and profile creation as direct jobs\n\n* test(gui): cover a Mods move, an unknown Mod and a locked profile creation\n\n* test(gui): the missing-profile retry runs on a real Instance\n\n* test(gui): make the unmakable-profile test independent of the host\n\n* test(gui): poll for the ready page instead of a fixed wait in the wizard test\n\n* test(gui): keep clicking Next until the Ready page is up in the wizard test\n\n* test(gui): accept both ends of a Stop that races the end of the LOD run",
          "timestamp": "2026-11-19T00:47:20+01:00",
          "tree_id": "e09db0224a917de98aa1b0bee734a06bb6b5559e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/173d464eda5245cb3db6894ea338dbe4291e15cd"
        },
        "date": 1795045728163,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.896553,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.683105,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.770269,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.082661,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.108104,
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
          "id": "010a3c199d2cc53ac6628820697f074d6c0b52b7",
          "message": "feat(setup): raul setup and the wizard run the setup job (#302)\n\n* wip: setup args and journal request\n\n* feat(cli): raul setup runs the setup job in process with a resume journal\n\n* test(cli): satisfy clippy in the setup replay\n\n* feat(gui): the setup wizard runs the setup job in process\n\n* fix(core): restart takes the lock, resume keeps the Instance, journal keeps no program paths\n\n* fix(core): name the interrupted step on a cancel and fix the engine doc\n\n* test(core): compare setup paths in their absolute form on every platform",
          "timestamp": "2026-11-19T01:18:22+01:00",
          "tree_id": "e9088629517b80b1496b40af1dc705107519538f",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/010a3c199d2cc53ac6628820697f074d6c0b52b7"
        },
        "date": 1795047581261,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.9007539999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.698020000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.482800000000005,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.0923560000000005,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.250403,
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
          "id": "0777c714079ae9367171d132acd1ceb243ccb6d7",
          "message": "feat(gui): run init, install-nv-rs, install and import-mo2 as direct jobs (#305)\n\n* feat(gui): run init, install-nv-rs, install and import-mo2 as direct jobs\n\n* fix(gui): take the automatic snapshot around an install and a Mods change, and keep the engine install tests offline\n\n* fix(gui): refuse init and install-nv-rs requests the command line would call a usage error",
          "timestamp": "2026-11-19T01:30:09+01:00",
          "tree_id": "16447e3c2bee71636543c11d91fb3d3c0ff59d23",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/0777c714079ae9367171d132acd1ceb243ccb6d7"
        },
        "date": 1795048312048,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.545105,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.910728,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 32.91751,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.297529,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.171930999999999,
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
          "id": "2b39b948f45be22eb9435bd41c542ca4415e0736",
          "message": "ci: allow the engine's licences narrowly in cargo-deny (#306)\n\n* ci: allow the engine's licences narrowly in cargo-deny\n\n* ci: widen the GPL scope check and use a portable temp file",
          "timestamp": "2026-11-19T01:34:11+01:00",
          "tree_id": "80f1c9fbe603cf293c3cfac9950d10bcb2914219",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/2b39b948f45be22eb9435bd41c542ca4415e0736"
        },
        "date": 1795048548379,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.374297,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.059314,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 27.956165000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.052508,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.171859,
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
          "id": "a4e477f001369ed2ba43abc75e01b106f67615a8",
          "message": "feat(gui): link the engine's crates behind the feature engine (#308)\n\n* feat(gui): link the engine's crates behind the feature engine\n\n* fix(gui): allow the engine's licences in the GUI deny file and re-export the linked crates",
          "timestamp": "2026-11-19T03:19:51+01:00",
          "tree_id": "61951cb9f511c9c35e28644162c1c7857191c0c4",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a4e477f001369ed2ba43abc75e01b106f67615a8"
        },
        "date": 1795054888352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.411386,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.132518,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.083530999999997,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.1386209999999997,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.806505999999999,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c9e7d8837bd592e4a11db5b2b8b71fef36361342",
          "message": "chore(deps): bump nv-rs to 0.9.0 (#312)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-19T06:37:17Z",
          "tree_id": "a93de31abbef556c46f29fb9cdd87c55b44f13a6",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c9e7d8837bd592e4a11db5b2b8b71fef36361342"
        },
        "date": 1795070324777,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.918258,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.767158,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.838558,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.163443,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.939849,
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
          "id": "fe6193c52e94b02b6662443ffb08c237eff8c8d7",
          "message": "ci: start the Component follower when a Component releases (#314)",
          "timestamp": "2026-11-19T08:30:19+01:00",
          "tree_id": "2a1663c11f9f56abfa7190d2bcde675c09bab797",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/fe6193c52e94b02b6662443ffb08c237eff8c8d7"
        },
        "date": 1795073531689,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.356137,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.89071,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 48.450638,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.118677,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.594256000000001,
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
          "id": "d39c3415a23f1398c21a488747f0f214e947f4ff",
          "message": "feat(release): installers carry no lodwright or ttwright program; release notes list contents with checksums (#313)\n\nThe installers no longer build, bundle or smoke-test the lodwright and ttwright programs, which are linked into raul. The\nsmoke tests now fail if a package holds one. The release notes get a Contents section listing each part with its version\nand every installer file with its SHA-256, read from SHA256SUMS.",
          "timestamp": "2026-11-19T10:24:59+01:00",
          "tree_id": "cf39307c42fe2658c398f25a2b4bdd7ff67c592f",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d39c3415a23f1398c21a488747f0f214e947f4ff"
        },
        "date": 1795080400377,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.451759,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 11.207937,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 47.045556000000005,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 6.516231,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.387013999999999,
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
          "id": "9f3a203e360b863ec4c242e4aa6f3edfdd2e5df5",
          "message": "chore(deps): update minor and patch dependencies, with the lock files regenerated (#318)\n\n* chore(deps): update minor and patch dependencies\n\n* fix(deps): regenerate Cargo.lock and adapt to jsonschema 0.61\n\njsonschema 0.61 turned ValidationError::instance_path into a method.\n\n* fix(deps): refresh the GUI lock for jsonschema 0.61\n\n* chore(deps): jsonschema 0.61.2\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-19T11:09:18+01:00",
          "tree_id": "25479a52404cf94cb94bb44cf8f4b4cd3aeed8a7",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/9f3a203e360b863ec4c242e4aa6f3edfdd2e5df5"
        },
        "date": 1795083176550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.902352,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.647692000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.250087,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.1031260000000005,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.651845,
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
          "id": "108aabfdce0c646974305862647bb8f39fa69a87",
          "message": "ci: let Renovate clone the Component submodules so Cargo.lock can be refreshed (#322)",
          "timestamp": "2026-11-19T11:19:21+01:00",
          "tree_id": "a9d8681bb09d9015634da077f963dfb6d6897cc3",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/108aabfdce0c646974305862647bb8f39fa69a87"
        },
        "date": 1795083849716,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.276612,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.387894,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.388444,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.126179,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.361222999999999,
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
          "id": "77a3ba23bce6c4952c5077a468f01650a07722f1",
          "message": "feat(gui): sort the Load order by calling nvmod's sort job directly (#269) (#321)\n\n* feat(gui): sort the Load order by calling nvmod's sort job directly (#269)\n\n* test(gui): the sort apply takes its snapshot, usage text is checked, keep-order stays a command line (#269)",
          "timestamp": "2026-11-19T11:42:39+01:00",
          "tree_id": "e703d21228794104175de76a8b3d8c1c1de0c8ff",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/77a3ba23bce6c4952c5077a468f01650a07722f1"
        },
        "date": 1795085062040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.3910529999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.38502,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 31.053779,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.183937,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.735816,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3bddc0a9b81a9f739e0edefbf00af999a09e4820",
          "message": "chore(main): release 0.5.0 (#286)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-19T12:00:09+01:00",
          "tree_id": "92170c04adaab6bc990ea852b644246fc71458ab",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/3bddc0a9b81a9f739e0edefbf00af999a09e4820"
        },
        "date": 1795086125562,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.340542,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.696202,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 47.742481000000005,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.487912000000001,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.498415,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a5c1f0dcac3dc25fdf9f5a3bbc196f6df313b394",
          "message": "chore(deps): bump nvmod to 0.11.0 (#326)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-19T11:11:03Z",
          "tree_id": "e330644d38dee1d799a2775b7c4845a5b46d0f5a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a5c1f0dcac3dc25fdf9f5a3bbc196f6df313b394"
        },
        "date": 1795086876114,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.8967420000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.645819000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.51861,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.122645,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.426268,
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
          "id": "08b8387a6af7e99b8e3532fa543dd9e75460a0d2",
          "message": "feat(update): journalled swap and rollback of an install unit (#325)\n\n* feat(update): journalled swap and rollback of an install unit\n\nThe portable core of the update flow (ADR-0020): stage the new files beside the install, swap them in with a journal\nwritten first, keep the previous version, undo an interrupted swap at start, and roll back a finished one. Synthetic\nfolders only; nothing is downloaded and no program is started.\n\nPart of #179\n\n* fix(update): lock, link refusal, durable renames and a kept rollback target in the swap\n\nPart of #179",
          "timestamp": "2026-11-19T12:21:55+01:00",
          "tree_id": "e8e273c8cbbbf555c589d6420bb3ce4cde74a581",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/08b8387a6af7e99b8e3532fa543dd9e75460a0d2"
        },
        "date": 1795087388806,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.558371,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.223475,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.963658000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 3.972512,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.560992,
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
          "id": "6e68927948d2d2d0135c0c15a69e661289f31ff4",
          "message": "ci(renovate): leave loot-rs to the nvmod bump (#328)\n\nThe shared preset's first-party Cargo git rule makes the hosted Renovate app bump the loot-rs tag in Cargo.toml on its own. Here loot-rs moves with nvmod (ADR-0015): scripts/bump-component.sh takes the tag nvmod links. A lone bump builds a second loot-rs beside nvmod's, leaves components/loot-rs behind and does not refresh crates/raul-gui/Cargo.lock, as in #327.",
          "timestamp": "2026-11-19T12:33:12+01:00",
          "tree_id": "27e081176850ba2997a60fb789c6d31ffe2ce8c9",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/6e68927948d2d2d0135c0c15a69e661289f31ff4"
        },
        "date": 1795088097359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.3764560000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.168882,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.532229,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.086026,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.262022,
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
          "id": "f4b2f7fcb3ba05f623f9242ac075422475a56d13",
          "message": "fix(engine): plain words for the engine check, a bounded run, and tests that pin both refusals (#331)\n\n* fix(engine): plain words for the engine check, a bounded run, and tests that pin both refusals (#315)\n\n* test(engine): a neutral sample path in the engine check test (#315)",
          "timestamp": "2026-11-19T13:00:24+01:00",
          "tree_id": "c615aa142b4b65d69bad4f595fce63da4db87f37",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f4b2f7fcb3ba05f623f9242ac075422475a56d13"
        },
        "date": 1795089746677,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.371324,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.863904,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 49.194802,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.591232,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 14.267464,
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
          "id": "bcb4485fd2044e87704dc492ddf2dbc03043afff",
          "message": "feat(lod): one marked LOD Mod per Profile that keeps its place and state (#357)\n\n* feat(lod): one marked LOD Mod per Profile that keeps its place and state\n\nThe Mod raul lod generate keeps is now Generated LOD (<Profile>), with a\n[raulGeneratedLod] marker in its meta.ini (Profile, request digest and\nlodwright version; every other key kept). A new run keeps the Mod's\nplace and enabled state, so a disabled LOD Mod stays disabled; a new or\nremoved Mod goes on top, enabled. The old shared LOD (lodwright) Mod is\nrecognised and hands its place and state to the Profile's own Mod. A\nMod of the new name without the marker is refused. raul lod mods lists\nthe generated LOD Mods of a Profile (Contract raul-lod-mods v1).\n\n* fix(lod): clippy, the LOD page wording and snapshots, meta.ini from an interrupted copy",
          "timestamp": "2026-11-19T12:28:13Z",
          "tree_id": "3dfbc75e7ac0f195e3feaa64d330f8d0fb0d023e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/bcb4485fd2044e87704dc492ddf2dbc03043afff"
        },
        "date": 1795091618826,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.2947699999999998,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.486304,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.728958,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.140084,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.256576,
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
          "id": "a88904f2fe6d6d05c724ab5b682bdb16b0940d80",
          "message": "feat(lod): LOD settings per option, per-level terrain overrides and raul lod settings (#358)\n\n* feat(lod): allow the output folder inside a git working tree on request\n\nThe LOD output folder is refused when it lies inside a git working tree, by\nraul's plan and again by lodwright. Either `allow_in_repo = true` in the\nProfile's raul-lod.toml or `--allow-in-repo` on `raul lod plan` or\n`raul lod generate` now allows it: raul skips its own check and tells\nlodwright through GenerateRequest::allow_in_repo. The flag applies to that\nrun only and writes nothing to the settings. The refusal names both ways in\nplain words. An output folder inside the data folder stays refused.\n\nRefs #344, part of #38\n\n* feat(lod): LOD settings per option, per-level terrain overrides and raul lod settings\n\nA Profile's raul-lod.toml becomes a list of options (key, tier, kind,\nrange, default and catalogue text) in raul_drivers::lod::settings, the\nfunctions the LOD settings view will call:\n\n- show, set, unset: one option at a time. Each change is checked against\n  the option's range and then by lodwright's own Request::validate on the\n  request the settings give; a refused change writes nothing and says\n  what went wrong and \"Nothing changed.\" from the catalogue.\n- reset (Restore defaults): removes every LOD generation key, also ones\n  set to their default value, keeps allow_in_repo, never writes default\n  values, prunes the file and deletes it when nothing is left. The file\n  as it was is kept as raul-lod.toml.previous.\n- undo: puts the .previous copy back (one level); the next set or unset\n  ends it.\n\n[levels.4] to [levels.32] set quality, vertex cap and texture sizes per\nterrain level. Without them the request keeps an empty levels list, so\nthe digest of an unchanged Profile stays the same; with any of them it\nlists all four levels, the rest at their defaults. The defaults come\nfrom one function, the Toolbox's copy of lodwright's today.\n\nraul lod settings show|set|unset|reset|undo prints the new\nraul-lod-settings v1 document with --json; the schema, fixtures and the\ncommand-parity registry entries are added. raul lod plan refuses an\nunknown key with the same plain words.\n\nRefs #337, part of #38\n\n* fix(lod): portable settings fixtures and the registry test's settings arguments\n\nThe fixture test wrote the Instance as <instance> by text replacement,\nwhich missed the escaped backslashes of Windows paths; it now rewrites the\nJSON values and turns the separators after the Instance into slashes. The\nGUI registry test passes an option to lod settings set and unset.",
          "timestamp": "2026-11-19T12:59:37Z",
          "tree_id": "057a97f3d9cc9af9e88fc967f5a518ecd6dbbb5c",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a88904f2fe6d6d05c724ab5b682bdb16b0940d80"
        },
        "date": 1795093449533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.4021050000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.1346609999999995,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.256726,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.079039,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 8.980772,
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
          "id": "b5459bf1d857350e0da0dbc4eaba9281049ecba1",
          "message": "feat(lod): a work folder per Profile, and fixes to the generated LOD Mod (#363)\n\nlodwright's output and manifest move to <instance>/.raul/lod/<Profile>/out\n(ADR-0021); the first generate takes the shared folder over. A folder with\nthe backup name that is not generated LOD is refused instead of removed or\nrenamed; the old shared Mod's meta.ini is also read from its backup; a\nmarker needs its Profile; the note is written only for a new Mod; the\nraul-lod-mods position starts at 1 in the schema; the README says what\nraul lod mods refuses.\n\nRefs #334, part of #38",
          "timestamp": "2026-11-19T14:38:00+01:00",
          "tree_id": "012b1247929a100b3e7bc280a29aa85d1b68fd4e",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/b5459bf1d857350e0da0dbc4eaba9281049ecba1"
        },
        "date": 1795095587033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0471840000000001,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.732968,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.747607,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.5012459999999999,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.200411,
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
          "id": "37d6001d61826b748dee6cfc1905ebdde02d4516",
          "message": "feat(gui): the Mods page with a generated LOD Mod treatment (#361)\n\n* feat(lod): one marked LOD Mod per Profile that keeps its place and state\n\nThe Mod raul lod generate keeps is now Generated LOD (<Profile>), with a\n[raulGeneratedLod] marker in its meta.ini (Profile, request digest and\nlodwright version; every other key kept). A new run keeps the Mod's\nplace and enabled state, so a disabled LOD Mod stays disabled; a new or\nremoved Mod goes on top, enabled. The old shared LOD (lodwright) Mod is\nrecognised and hands its place and state to the Profile's own Mod. A\nMod of the new name without the marker is refused. raul lod mods lists\nthe generated LOD Mods of a Profile (Contract raul-lod-mods v1).\n\n* fix(lod): clippy, the LOD page wording and snapshots, meta.ini from an interrupted copy\n\n* feat(gui): the Mods page with a generated LOD Mod treatment (work in progress)\n\nNot built or tested yet: page, model, registry commands mods-list,\nmods-enable, mods-disable, mods-reorder, mods-remove and lod-mods, the\nsidebar entry (keys 1 to 6) and an end-to-end test.\n\n* fix(gui): the Mods page passes clippy, the registry test drives its commands, and its snapshots\n\nModEntry's flags are what raul mods list reports, so the bool lint is\nallowed there. The registry test's fixture gets one Mod for the Mods\npage's enable, disable, reorder and remove commands. Snapshots: the ten\nMods page states, and every other page with the new sidebar entry.",
          "timestamp": "2026-11-19T13:42:54Z",
          "tree_id": "52298b3b63d49eb17a3e1610e0a24631916616c4",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/37d6001d61826b748dee6cfc1905ebdde02d4516"
        },
        "date": 1795095866263,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.91414,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.744546000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.121872,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.981143,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.011625,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a0286586c6bdbb98a3032f493c7ca3cebbca1c1b",
          "message": "chore(deps): bump nvmod to 0.12.0 (#366)\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-19T14:09:34Z",
          "tree_id": "4ec9e3194969bf7081341b36422abb2f72bc37b1",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a0286586c6bdbb98a3032f493c7ca3cebbca1c1b"
        },
        "date": 1795097579265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.627526,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.4401340000000005,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.138937000000002,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.064398000000001,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.479481,
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
          "id": "c2fc2591f78ace14b3e3929e6f67cffdf20be1f8",
          "message": "feat(lod): one LOD run per Profile at a time; settings edits apply to the next run (#367)\n\nA generate holds an operating-system lock on .raul/lod/<Profile>/run.lock\nwhile it runs; the system releases it when the process ends, also when it\nis killed. A second generate of the same Profile is refused with the new\ncode run_in_progress (exit code 2) and starts nothing. A run reads the\nsettings once, when it builds its request, so raul lod settings stays\nusable during a run: it saves the change, says that it applies to the next\nrun, and reports run_in_progress in its document.\n\nRefs #343, part of #38",
          "timestamp": "2026-11-19T15:35:08+01:00",
          "tree_id": "146268613b021d6fe222a9fe934f8fe5143c538a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c2fc2591f78ace14b3e3929e6f67cffdf20be1f8"
        },
        "date": 1795099206942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.304757,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.780892,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.148027,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.065666,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.989712,
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
          "id": "13b23e5e74e1b3f5e700303af4c1d7065390158a",
          "message": "feat(gui): the LOD settings view with its warning, advanced settings and Restore defaults (#369)\n\nA LOD settings view reached from the LOD page (own route, no sidebar\nentry): an amber warning at the top, the settings that choose what is\nbuilt, saved when a field is left through raul lod settings set or unset,\nthe per-level terrain settings and Allow output in a git folder behind a\ndisclosure closed on every visit and a one-time I understand kept in the\napp's own preferences, and Restore defaults that asks once and keeps a\nrestore point that Undo restore puts back.\n\nraul update auto now keeps the other keys of settings.toml.\n\nRefs #338, #339, #340, part of #38",
          "timestamp": "2026-11-19T15:59:09+01:00",
          "tree_id": "2ce168e51715c30c80da2c10362c324079cafdc8",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/13b23e5e74e1b3f5e700303af4c1d7065390158a"
        },
        "date": 1795100468663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.378669,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.792821,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 47.555582,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.108983,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.936405,
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
          "id": "d9ec0cfa7cf08c476e5de2bf24bec510c5e43205",
          "message": "test(gui): finish moving the GUI fixtures to the ProcessExec seam (#368)\n\n* test(gui): point the program at a config and state folder and a masterlist file through ProcessExec, not wrapper scripts\n\nPart of #267\n\n* test(gui): the three other GUI fixtures use the ProcessExec seam, and a given environment value is pinned to win\n\nPart of #267\n\n* chore: align the lodwright-rs pin and lock files with main",
          "timestamp": "2026-11-19T15:09:23Z",
          "tree_id": "89e6da2e8e6a742d334f4d200aa7a69b19f17ac9",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d9ec0cfa7cf08c476e5de2bf24bec510c5e43205"
        },
        "date": 1795101056558,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.8933769999999999,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.681343,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 36.430557,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.977543,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.702065000000001,
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
          "id": "8d915fff7e584da23989e5335963a542c3931c54",
          "message": "feat(lod): progress for object and tree LOD, and plain words for each step of a run (#370)\n\nlodwright 0.5.0 reports finished object and tree blocks with done and\ntotal. They become raul-events progress lines of the phase objects or\ntrees, with the world as the message; terrain keeps world and level. The\nLOD page names each step in plain words instead of showing the phase\nkey, and shows a step it does not know as Working, never as an error.\n\nRefs #351, part of #38",
          "timestamp": "2026-11-19T15:11:45Z",
          "tree_id": "da593443e20fe227ba74677dd5c80f81307c3092",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8d915fff7e584da23989e5335963a542c3931c54"
        },
        "date": 1795101202364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.072529,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.617467,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 21.290584000000003,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.48933,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 5.907433999999999,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "290797fb3e9e112860f77b02f117dd0248984c8f",
          "message": "fix(deps): pin dependencies (#371)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-20T04:17:08Z",
          "tree_id": "667e3dff36ce088f244ab7b7182e883255d86b03",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/290797fb3e9e112860f77b02f117dd0248984c8f"
        },
        "date": 1795148515515,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.327248,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.629608000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.92657,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.120617,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 15.490950999999999,
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
          "id": "90575efdd74a8e8fa8657537aaaaeb45edfab217",
          "message": "feat(release): the engine in the macOS package (#360)\n\n* feat(release): the engine in the macOS package\n\nraul is built with the feature engine on macOS too, for both\nprocessors, and the macOS package carries nv-rs's universal build\nunder the bundle's Resources/components/nv-rs with its FILES.sha256,\nchecked against nv-rs's SHA256SUMS-macos first. The checksum and file\nlisting steps use what macOS has (no find -printf, shasum for\nsha256sum). The macOS smoke test checks the engine folder in the\nbundle and in the tarball and that the viewer is universal. The setup\nmessage no longer says a Mac has no engine.\n\nRefs #353\n\n* ci(release): build the two macOS halves of raul in parallel\n\nraul with the engine was built for arm64 and then x86_64 in the macOS\npackage job. A macos-raul job now builds each processor on its own\nrunner (x86_64 cross-compiled on Apple silicon) with its own\nrust-cache, and the package job joins the halves with lipo.",
          "timestamp": "2026-11-20T12:32:02+01:00",
          "tree_id": "9657c4c4aa95efe6a98adb647b5ab7169a5930d4",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/90575efdd74a8e8fa8657537aaaaeb45edfab217"
        },
        "date": 1795174426009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.0604900000000002,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 4.704556999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 22.135416000000003,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 1.5189489999999999,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 6.52057,
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
          "id": "a9d8662be956af65e5a43ecee28f8257f6cbd426",
          "message": "feat(gui): list the Mods by calling nvmod's mods job directly (#269) (#379)\n\nThe Mods page read its list through the command line in process. It now calls\nnvmod_api::mods::list with a typed request, and the document it answers with is\nthe one the command prints, checked against it for the plain and the\nprofile-and-category forms.",
          "timestamp": "2026-11-20T12:11:52Z",
          "tree_id": "6d964cc640f3f0f71cc0ee7d65e8609b8afa258a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a9d8662be956af65e5a43ecee28f8257f6cbd426"
        },
        "date": 1795176814630,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.918491,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 8.72873,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 37.109363,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.229364,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 36.051014,
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
          "id": "f0e7549b3242099791250f34bf6bdb4069a0068e",
          "message": "feat(lod): record what a LOD run was built from and tell when it is out of date (#66) (#382)\n\nA successful run keeps a fingerprint in the Profile's work folder: the request digest, a digest of the settings and worlds, and for each active plugin its size, a digest of its first and last megabyte and whether it has a top-level world group. raul lod mods --json now gives the Profile's own LOD Mod a stale field: current, may (a reorder, or a plugin without a world turned on, off or changed), must (any settings change, Restore defaults, Undo restore, or a plugin with a world turned on, off or changed) or unknown, with a one-line reason. A Mod without plugins and an inactive plugin do not count. A new run clears it. The packing switch does not exist yet, so it is not in the fingerprint; it joins the settings digest with the BSA work.",
          "timestamp": "2026-11-20T12:26:36Z",
          "tree_id": "5c4999d9bdc917de0fd47e6de229fedb681eca6d",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f0e7549b3242099791250f34bf6bdb4069a0068e"
        },
        "date": 1795177694746,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.691201,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.684533,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 31.606983999999997,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.2775240000000005,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 11.178645,
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com",
            "name": "jorisjonkers-dev-agents[bot]",
            "username": "jorisjonkers-dev-agents[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "17d4fac77fc3040fcf32ac392811ddd769af1b3f",
          "message": "chore(deps): bump lodwright-rs to 0.6.0 (#381)\n\n* chore(deps): bump lodwright-rs to 0.6.0\n\n* test(contracts): take the lodwright 0.6.0 request and result schemas as the snapshots\n\n---------\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>\nCo-authored-by: J.W. Jonkers <info@jorisjonkers.dev>",
          "timestamp": "2026-11-20T12:35:00Z",
          "tree_id": "4c502893577694691a1bc0f0c5f486a6aa366cd2",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/17d4fac77fc3040fcf32ac392811ddd769af1b3f"
        },
        "date": 1795178296607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.535386,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.027731,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 29.879521999999998,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.279568,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.306636,
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
          "id": "e86498d5e27d013acb0aee69f8b4fab011b59866",
          "message": "docs(adr): name what the installers cannot bundle and the guided step for each (#383)\n\n* docs(adr): name what the installers cannot bundle and the guided step for each (#169)\n\n* docs(adr): match the step names to the setup",
          "timestamp": "2026-11-20T12:42:31Z",
          "tree_id": "2977163e8aae833580ddb03d00e9b94f5d1a1069",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e86498d5e27d013acb0aee69f8b4fab011b59866"
        },
        "date": 1795178662008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.290623,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.380398000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.450522,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.114794,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.03805,
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
          "id": "2fd8776939d82a48517126ce5e233213091073fd",
          "message": "feat(cli): raul run starts raul itself in the play mode when the build links the engine (#396)\n\n* feat(gui): raul's own binary runs the game when started with --play-mode (#260)\n\n* chore(deps): bump nvmod to 0.13.0\n\n* style(gui): fmt and a clippy-clean refusal test (#260)\n\n* wip: play launcher\n\n* feat(cli): raul run starts raul itself in the play mode when the build links the engine (#377)\n\n* fix(deps): refresh both lock files for ttwright-rs 0.4.4 and the nvmod-run edge of raul-cli\n\n---------\n\nCo-authored-by: jorisjonkers-dev-agents[bot] <290081373+jorisjonkers-dev-agents[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-20T13:14:32Z",
          "tree_id": "c3b6b2af0d5ec3423892d621c717841b67822e02",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/2fd8776939d82a48517126ce5e233213091073fd"
        },
        "date": 1795180733861,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.556602,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.140518999999999,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 28.382008,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.008578,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 9.335532,
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
          "id": "434b71051be570a0de38d7b273323d5e0db18624",
          "message": "feat(gui): check the free space before the TTW install, with raul space (#385)\n\n* feat(gui): check the free space before the TTW install, with raul space (#155)\n\n* fix(core): drop the verbatim prefix of a Windows path before matching it to a drive (#155)",
          "timestamp": "2026-11-20T13:26:12Z",
          "tree_id": "e632cd89ad03f87b8bdb57536bf8a7427177ee5a",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/434b71051be570a0de38d7b273323d5e0db18624"
        },
        "date": 1795181367584,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.646132,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.589417,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.592301,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.140975,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.601098,
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
          "id": "bacf1d5114512442a69d4c7445046ce67dc5ddc1",
          "message": "test(gui): drive the TTW flow, Cancel and Resume with the keyboard only (#393)\n\n* fix(gui): apply a source re-read that finishes during a TTW job when the job ends (#156)\n\n* test(gui): drive the TTW flow, Cancel and Resume with the keyboard only (#157)",
          "timestamp": "2026-11-20T13:30:11Z",
          "tree_id": "cce7b4eebdc3ba505949c4045dab8e71a5ad3c46",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/bacf1d5114512442a69d4c7445046ce67dc5ddc1"
        },
        "date": 1795181529362,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.282639,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.449065000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.977648,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.311509,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.088154,
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
          "id": "6cd9c6286e96b1d6fe2ba6e382904b5437080029",
          "message": "test(gui): prove no GUI action starts a program, by counting what the runner starts (#390)\n\n* test(gui): count the programs the runner starts and prove no GUI action starts one (#168)\n\nraul-runner counts every program it starts (Runner and start_program). A GUI test runs the doctor, conflicts, tree, plugins, mods list and change, sort preview and apply, LOD plan and mods, a TTW install and an undo through the real jobs and fails when the count moves, which a scan of the sources cannot show.\n\n* test(gui): count the stop signals too and assert what each action ends with (#168)\n\nsignal_group starts kill or taskkill when a stopped run is signalled, so it counts as a started program. The new test asserted only that each job had an output; it now asserts exit code 0 where the fixture decides it and a code at all elsewhere, and its comments say what the TTW and undo cases cover.",
          "timestamp": "2026-11-20T13:39:41Z",
          "tree_id": "a937cba051f655a4a11e53c8d9606cf873d3f413",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/6cd9c6286e96b1d6fe2ba6e382904b5437080029"
        },
        "date": 1795182267651,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.3309960000000003,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.547142000000001,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 46.63953,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.240551000000001,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 13.136132,
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
          "id": "8c30b9d6c86905465d4157d0d98b1758969fc7cf",
          "message": "feat(gui): Stop for a running game and plain words for how it ended (#407)\n\n* feat(gui): a Stop button for the running game through the play mode's stop file, and plain words for how a game ended (#260)\n\n* fix(cli): clippy in the stop command, and the stop command in the registry fixture test (#260)",
          "timestamp": "2026-11-20T13:49:17Z",
          "tree_id": "a1bb25fa53f753b9efc52fd6d2c6d46b3d1feeeb",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8c30b9d6c86905465d4157d0d98b1758969fc7cf"
        },
        "date": 1795182668016,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 2.238598,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 10.290741,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 45.216356,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 5.026023,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 12.911079,
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
          "id": "3ccb98a7e16a802f68920e7917d1be9370c74545",
          "message": "feat(gui): run LOD generation and the TTW install as typed jobs with their own cancel (#402)\n\n* feat(gui): run the LOD generation and the TTW install as typed jobs with their own cancel (#168)\n\nThe Toolbox's LOD and TTW drivers take a Stop (the process-wide request, or one run's own), and raul lod generate and raul ttw are thin wrappers over TtwJob and LodJob. The window's ttw and lod-generate commands start those as streaming direct jobs in process: no command line is parsed, no Ctrl+C handler is installed in the window, the job's cancel token stops the run at its next safe point and the job never sets the process-wide request. The setup job's TTW step uses the setup's own token the same way. Tests drive both runs through the job table: progress events, cancel, the Instance lock.\n\n* test(cli): cover the typed LOD and TTW job streams and the line sink (#168)",
          "timestamp": "2026-11-20T14:26:37Z",
          "tree_id": "69fd12f77f98fa4764d759ed39cead505737da0d",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/3ccb98a7e16a802f68920e7917d1be9370c74545"
        },
        "date": 1795184884884,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.663347,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 7.752591,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 30.765431,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 4.256436,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.166579,
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
          "id": "52ca49e8083c34578d1eff2219e54cb03eed0078",
          "message": "test(gui): cover the Stop command, its button and how a failed Stop is shown (#411)\n\nThe changed-line gate failed on main after #407: the stop command in the Home model and the Stop job and button in the page ran in no test.",
          "timestamp": "2026-11-20T14:35:53Z",
          "tree_id": "d2c33f3d3f071ec033e3cbd3bc6195d1c7d829b0",
          "url": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/52ca49e8083c34578d1eff2219e54cb03eed0078"
        },
        "date": 1795185472839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sort/small",
            "value": 1.487258,
            "unit": "ms"
          },
          {
            "name": "sort/medium",
            "value": 6.759035,
            "unit": "ms"
          },
          {
            "name": "sort/ttw-sized",
            "value": 32.125780999999996,
            "unit": "ms"
          },
          {
            "name": "plugin-headers/ttw-sized",
            "value": 2.268952,
            "unit": "ms"
          },
          {
            "name": "install/zip",
            "value": 10.739902,
            "unit": "ms"
          }
        ]
      }
    ]
  }
}