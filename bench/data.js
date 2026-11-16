window.BENCHMARK_DATA = {
  "lastUpdate": 1794869596095,
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
      }
    ]
  }
}