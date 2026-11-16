window.BENCHMARK_DATA = {
  "lastUpdate": 1794830249646,
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
      }
    ]
  }
}