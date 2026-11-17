# Changelog

## [0.2.0](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/compare/v0.1.0...v0.2.0) (2026-11-17)


### Features

* **cli:** add nvmod mods list, enable, disable and reorder (JorisJonkers-dev/nvmod-rs[#33](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/33)) ([5d53b06](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5d53b066fcd6956fbd57ed23f98430ed5bf35dbe))
* **config:** publish the raul-config v1 Contract with its schema and fixtures ([#119](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/119)) ([33029e7](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/33029e7a5c3c06f9048840b544f241125fb34908))
* **config:** raul config list, get, set and unset for the paths the GUI needs ([#120](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/120)) ([ef3ef97](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ef3ef976fd381d6b4ee4377faa5a695d0a2128fc))
* **conflicts:** add nvmod conflicts and nvmod tree (JorisJonkers-dev/nvmod-rs[#42](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/42)) ([5a3c7d5](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5a3c7d5f17edb1592e0a62b07c79edf5aba129d6))
* **gui:** give the view model and service seam their own workspace and run raul through the Runner ([ee8726a](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ee8726ab4610af95f6424e2eedda25f77ee7b2f8))
* **gui:** open the egui shell from raul with no arguments ([#136](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/136)) ([c5105cd](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c5105cde5cb5a8599e5707334264fd044b84a39b))
* **lod:** build lodwright's request from the Profile and show it with raul lod plan ([#123](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/123)) ([9f936ce](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/9f936ce5ef202226af1d0e68d6df54edf241513d))
* **lod:** publish the raul-lod-result v1 Contract with fixtures ([#126](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/126)) ([92320af](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/92320af83ab66ad125f4269d161d85213f5d9608))
* **mods:** add separators and categories (mods separator, mods category set, mods list --category) (JorisJonkers-dev/nvmod-rs[#61](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/61)) ([050098c](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/050098c4829810b92583c2e1b8ab97f71a1be72c))
* **plugins:** add nvmod plugins commands and sort --fetch-masterlist (JorisJonkers-dev/nvmod-rs[#49](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/49)) ([fe2d5e1](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/fe2d5e19ee3f764d10f4fe22b2ad31ad5c6050f2))
* **runner:** add a streaming mode for long jobs ([41f72f7](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/41f72f7f1fad0e9437468cabff884623ef265640))
* **setup:** keep the imported MO2 load order and make the sort opt-in (JorisJonkers-dev/nvmod-rs[#57](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/57)) ([904146d](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/904146dfe1e85c9fb0338a20d7ee98cf0643e6d7))
* **ttw:** publish the raul-events and raul-ttw-result v1 Contracts with fixtures ([#121](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/121)) ([5d8b1a4](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5d8b1a4f0186e4f0c9316563f601f3f94d7eb209))
* **ttw:** raul ttw installs TTW through ttwright-rs and adds it to the Instance ([#122](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/122)) ([15dd1ca](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/15dd1ca4789af6803800d2b968e2ccdc64450ad8))
* **ui-model:** add nvmod-ui-model, the toolkit-free view-model of the GUI (JorisJonkers-dev/nvmod-rs[#31](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/31)) ([cde4d6b](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/cde4d6be22c0680ab5308600f0a4ea1de54fc005))
* **ui-service:** add nvmod-ui-service, the service seam of the GUI (JorisJonkers-dev/nvmod-rs[#32](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/32)) ([5b999cd](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5b999cd38dbb84196b6a949906c4f841bbf573bf))
* **ui:** apply Mods edits, typed setup request and job cancel (JorisJonkers-dev/nvmod-rs[#35](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/35)) ([1715542](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1715542ad3f79aa6e1a7377471eb5437d4a92050))
* **ui:** typed calls for sort, verify, target, tools and rollback, job streaming, Doctor and Mods state (JorisJonkers-dev/nvmod-rs[#34](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/34)) ([dcf89ff](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/dcf89ff23465ba6ff8c257067f5cbb5a79331414))


### Bug Fixes

* **runner:** count line endings against the cap and keep the caller's variables ([71ac701](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/71ac70134533c317f9adb102638b4a6be9e72df9))

## 0.1.0 (2026-11-16)


### Features

* **bench:** add raul bench with a synthetic suite, comparison and CI history ([#8](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/8)) ([e47156c](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e47156caf416c2bc631fa087d41635746e6e4e6e))
* **detect:** find game copies on Steam, GOG and Epic ([#4](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/4)) ([42b8c65](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/42b8c65ed8f7361ec138cc075aea964de3a69b4b))
* pin the five Components and add the raul command line ([#3](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/3)) ([f482364](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f482364f3d211da1ccf93db8789e5677fcc865bb))
* **release:** installers for Linux, Windows and macOS, and an opt-in update check ([#9](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/9)) ([19783c4](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/19783c49e1b92325209337ce908ce2f4d81dc4dd))
* **runner:** the Runner moves in from nvmod-rs as `raul-runner`, with its history ([#105](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/105))
* **runner:** make raul-runner the only Toolbox crate that starts a process ([69111c4](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/69111c42321937435c5bd84a754e003e8fcab451))
* **update:** run the update check through raul-runner and compare versions as semver ([35d5072](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/35d50723be28d55438160ebd7a576490c8d0ddff)), closes [#48](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/48)


### Bug Fixes

* **release:** make the macOS and Windows smoke tests finish ([#106](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/106)) ([649f273](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/649f27395e1c9a7d28911c9978768db7970ef3c3))
* **release:** make the Windows and macOS smoke tests run unattended ([#28](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/28)) ([bee895e](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/bee895e0682b26bc6d1bed4f261791d831dd0973))
* **release:** start releases at 0.1.0 ([#7](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/7)) ([db8c256](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/db8c25688b4126d7a19950dc0769c2f374d1383d))
