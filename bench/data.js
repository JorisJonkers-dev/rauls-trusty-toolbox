window.BENCHMARK_DATA = {
  "lastUpdate": 1794949296575,
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
      }
    ]
  }
}