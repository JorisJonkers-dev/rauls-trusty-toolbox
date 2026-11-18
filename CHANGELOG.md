# Changelog

## [0.4.0](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/compare/v0.3.0...v0.4.0) (2026-11-18)


### Features

* **drivers:** install TTW through ttwright linked into raul ([#270](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/270)) ([d193bb0](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d193bb07c887fe23f6ba949f536e71089ee89236))
* **drivers:** run ttwright's install in process ([#264](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/264)) ([02ffd22](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/02ffd228fc375dae15f97a5edf24c53f3ebf2ffd))
* **gui:** add InProcessExec, an Exec that runs raul in this process ([#268](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/268)) ([46add02](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/46add0211442f008c1fa79729581c1cb3e1dd8b8))
* **gui:** check a TTW package by name, size and contents ([#223](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/223)) ([dca2e32](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/dca2e32ebf2849a021e140471561fa1d526cd5c6))
* **gui:** copy the details of every error ([#189](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/189)) ([51e787f](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/51e787f8248f5a0ff0ffcc2c9614f92a6282804e))
* **gui:** find the newest valid TTW package over several folders ([#228](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/228)) ([acc63e1](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/acc63e16a9f24d0e273071cc02ba075f0b662634))
* **gui:** no command lines on Versions, Home and dialogs ([#200](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/200)) ([7b99b8c](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7b99b8c017e731e0b44969beb7979cbcff496919))
* **gui:** open the TTW download page and find the package for me ([#187](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/187)) ([da82eea](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/da82eeaf1c7a38000963235bb768e59e845119f0))
* **gui:** search Downloads and the Desktop and show the size of the package found ([#238](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/238)) ([0daedc1](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/0daedc1b73f1a3d56075d968d9c1a14616547857))
* **gui:** search the folders Settings points at for the TTW package ([#241](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/241)) ([7399eb5](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7399eb55925a2174614d072eba684406d24a77fd))
* **gui:** show no command lines in the setup wizard ([#220](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/220)) ([5280df2](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5280df257ef159f6bb031bdd5ffb2e881bd797ca))
* **gui:** show no command lines in the TTW flow ([#219](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/219)) ([f4fa06a](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/f4fa06a471fa1141a5a45a8d10675b4f19832bbc))
* **gui:** show no command lines on the Load order page ([#218](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/218)) ([625bf7f](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/625bf7fd2084c29d8ca6d3c324be0c11579de9c2))
* **gui:** show no command lines on the LOD page ([#217](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/217)) ([116109e](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/116109e3040e16c99ed60300708d23392c16aa28))
* **gui:** tell when a download is complete ([#224](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/224)) ([ba7ef93](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ba7ef9341d31caeedae4405539d3c2750e0c1ea3))
* **gui:** unpack a TTW package that arrives in an archive ([#242](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/242)) ([7f27796](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7f277966e02b21e29dbfe98c0054fe448445155b))
* **gui:** watch Downloads for the TTW package once the user agrees ([#240](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/240)) ([3986768](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/398676874caf293652ae83685d5aa13d58633a59))


### Bug Fixes

* **ci:** give the Components follower's commands the cargo registry token ([#182](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/182)) ([56d2719](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/56d27193f0328acd0cc5349afc196110117a8a67))
* **ci:** read the cargo registry token from the job environment ([#184](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/184)) ([b05d8e2](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/b05d8e21c4abd323d07055c90cb87731acc474ed))
* **ci:** run the Component follower on Node 24 ([#158](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/158)) ([c0eaa30](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/c0eaa30e309570ed4b90621a8a108276a5a48aae))
* **deps:** pin rust crate ctrlc to =3.5.2 ([#243](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/243)) ([6b77a96](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/6b77a9656f71c4489ffbb19f0e81bb0f8568a744))
* **gui:** end a chained job as its second command ended ([#222](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/222)) ([edffb36](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/edffb368f375ab9caaaa00447bca6cb1d8028612))
* **gui:** fit the Home page to the smallest window ([#229](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/229)) ([e4548b0](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e4548b08fb25dd29ca02dce922f2be3cc8e443c6))
* **gui:** fit the Load order page to the smallest window ([#232](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/232)) ([da85974](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/da85974c671b0c8417fd9815cda5f384fc6e8d3b))
* **gui:** fit the LOD page to the smallest window ([#231](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/231)) ([030560c](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/030560cfe02005c79a232a77e2a510d24848d4b6))
* **gui:** fit the Settings page to the smallest window ([#233](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/233)) ([e7a57b7](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e7a57b772b0d4924742b2e342e00a86a630ec653))
* **gui:** fit the Setup wizard page to the smallest window ([#237](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/237)) ([d9e9cb2](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d9e9cb29fcd7a9c047cb1f1397d675024a663b67))
* **gui:** fit the TTW flow page to the smallest window ([#234](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/234)) ([8a1390e](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/8a1390e2dc863456e880f05b02f525ff69acb332))
* **gui:** fit the Versions page to the smallest window ([#230](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/230)) ([4dbcce1](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/4dbcce1997ab9c9c36bdaa680d62bf2dad880397))
* **gui:** say ready to play only when the game can start ([#186](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/186)) ([e8acb88](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/e8acb88cd8ba4be38f7a01d3f8cba267b42ef4c5))
* **gui:** search Downloads once without a home folder ([#192](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/192)) ([d412a15](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d412a152916d93b592777c0ae6f731c95abbe722))
* **gui:** set up the default profile and keep the word out of the UI ([#190](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/190)) ([d0f0297](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/d0f0297f65da163fc0bfb4002f518f3fa89a1fe0))
* **gui:** show one version and drop known-good from the UI ([#188](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/188)) ([66275e3](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/66275e383f60bf1ac8af4eadf226624af9a7eb04))
* **gui:** use the found TTW package only after it is saved ([#221](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/221)) ([3ffdd65](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/3ffdd65076d01fa13d7eb63987778bfadeae65ef))

## [0.3.0](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/compare/v0.2.0...v0.3.0) (2026-11-17)


### Features

* **gui:** Load order page with sort preview, apply and undo ([#150](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/150)) ([1795a59](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/1795a59650e98e1b8cd8679e5fcdd19e85a9b69f))
* **gui:** LOD page with plan, blocks, generate and stop ([#151](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/151)) ([ef32e33](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/ef32e335aa533ce4f86e94cb01c7117b09da3a43))
* **gui:** read raul ttw's event stream and result in the service ([#141](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/141)) ([48788dc](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/48788dc6eba374ba43b7c6b5643fd5174518628d))
* **gui:** register verify, version and detect, and give run its target ([#138](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/138)) ([74fff85](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/74fff852a42f7276cd234bdabe3096eb4f1d8620))
* **gui:** Settings, Versions and Home pages ([#143](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/143)) ([296335e](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/296335e0dd3e09467fb17d5ae2fd722919d8206d))
* **gui:** share the Instance with every page, read TTW reports from job output, register tools list ([#148](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/148)) ([a37324b](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/a37324b66ce762e2b926d31c1dc5e8c0817486b5))
* **gui:** the first-run setup wizard ([#144](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/144)) ([5119d1a](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/5119d1af78db58461cf86d1fc509025e078b4e7d))
* **gui:** TTW flow page with a dry-run review, install, cancel and resume ([#153](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/153)) ([7ecde77](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/7ecde773a3daf54016aef5807027d765b4bf0e26))
* **lod:** raul lod generate runs lodwright with the request and keeps its output as one Mod ([#127](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/127)) ([dfd987a](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/dfd987a9f335afde48d98232ea05196523a84b28))


### Bug Fixes

* **gui:** register nvmod's raul run and start the game from Play with it ([#149](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/149)) ([94c78d5](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/commit/94c78d5c08967093e27a6102542498e6c2e04093))

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
