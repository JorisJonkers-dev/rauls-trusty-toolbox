<!-- SPDX-License-Identifier: GPL-3.0-only -->
# Raul's Toolbox: the design standard

The clickable demo in [`demo/index.html`](demo/index.html) is the **golden standard for all design work on Raul's Toolbox**. The GUI is built to look and behave like it: layout, colours, wording, flows and keyboard handling. Where the app and the demo differ, the demo wins unless a later decision in this folder says otherwise. Change the design by changing the demo first (sources in [`demo/src/`](demo/src), built with `demo/build.sh`), then the app.

Open `demo/index.html` in a browser. It starts with the setup wizard and keeps its state in the browser, like a first start of the real app. Everything is clickable; data is mocked.

## Rules

- A 1280x720 desktop window. Sidebar: Play (Home, Mods, Load order, LOD) and System (System, Doctor, Settings), with badges on Load order, LOD, Mods and System when something needs doing.
- Cool blue-grey dark with a teal accent; warnings orange, errors red; Raul's amber only for the brand. High contrast mode; no light theme.
- Rounded blocks instead of lines. A card's header and its rows share the same insets and padding.
- Page actions sit bottom-right in a compact floating bar with equal-height buttons, the primary one rightmost. Headers hold title and status only.
- Show little text. No command lines or "Show command" in the UI. Say "LOD" (tooltip "Render distance"), never "Far view".
- Whole rows are click targets: a click folds a row open or flips its switch; nested buttons keep their own action.
- Enter submits forms and dialogs, Escape cancels. Text fields show only the caret when focused; other controls show one subtle indicator for keyboard focus only.
- Toasts have a fixed width with the close X on the right.
- Check again and Update live only on the System page; elsewhere a badge on the System nav.
- Location pictures: filter by world (New Vegas, plus Capital Wasteland when TTW is on), one side-scrolling row with "All N" to expand, search across both worlds; wheel zooms, drag pans.

## Checking the demo

`node demo/test/run.js` walks every flow at 1280x720, fails on console errors or failed checks, and rewrites the screenshots below. The other scripts in `demo/test/` check one area each (insets, keys, action bar, places, profiles, rows, versions, viewports). They need Node and the `playwright` package.

## The flows

### First start: the setup wizard

The toolbox opens on an eight-step wizard: find the game, choose storage, install the engine, add Tale of Two Wastelands, pick a profile, build LOD, done. Each step shows what was found, what went wrong (game not found, wrong folder, low space) and a busy state.

**1. Wizard 1 welcome**

![wizard 1 welcome](screenshots/01-wizard-1-welcome.png)

**2. Wizard 2 game finding**

![wizard 2 game finding](screenshots/02-wizard-2-game-finding.png)

**3. Wizard 2 game found**

![wizard 2 game found](screenshots/03-wizard-2-game-found.png)

**4. Scenario panel**

![scenario panel](screenshots/04-scenario-panel.png)

**5. Wizard 2 game notfound**

![wizard 2 game notfound](screenshots/05-wizard-2-game-notfound.png)

**6. Wizard 2 game wrongfolder**

![wizard 2 game wrongfolder](screenshots/06-wizard-2-game-wrongfolder.png)

**7. Wizard 3 storage**

![wizard 3 storage](screenshots/07-wizard-3-storage.png)

**8. Wizard 3 storage low**

![wizard 3 storage low](screenshots/08-wizard-3-storage-low.png)

**9. Wizard 4 engine**

![wizard 4 engine](screenshots/09-wizard-4-engine.png)

**10. Wizard 4 engine installing**

![wizard 4 engine installing](screenshots/10-wizard-4-engine-installing.png)

**11. Wizard 5 ttw**

![wizard 5 ttw](screenshots/11-wizard-5-ttw.png)

**12. Wizard 5 ttw installing**

![wizard 5 ttw installing](screenshots/12-wizard-5-ttw-installing.png)

**13. Wizard 5 ttw added**

![wizard 5 ttw added](screenshots/13-wizard-5-ttw-added.png)

**14. Wizard 6 profile**

![wizard 6 profile](screenshots/14-wizard-6-profile.png)

**15. Wizard 7 farview**

![wizard 7 farview](screenshots/15-wizard-7-farview.png)

**16. Wizard 7 farview building**

![wizard 7 farview building](screenshots/16-wizard-7-farview-building.png)

**17. Wizard 8 done**

![wizard 8 done](screenshots/17-wizard-8-done.png)

### Home and playing

Home is one column: a short setup list (New Vegas OK, TTW Installed, Mods OK, Load order OK, LOD OK; Engine only when it can be updated), the live profile, one playtime total and Last played. Play always rechecks the setup first.

**18. Home after setup**

![home after setup](screenshots/18-home-after-setup.png)

**19. Profile menu**

![profile menu](screenshots/19-profile-menu.png)

**20. Home beautiful**

![home beautiful](screenshots/20-home-beautiful.png)

**21. Play checking**

![play checking](screenshots/21-play-checking.png)

**22. Play result beautiful**

![play result beautiful](screenshots/22-play-result-beautiful.png)

**23. Play running**

![play running](screenshots/23-play-running.png)

**24. Home after play**

![home after play](screenshots/24-home-after-play.png)

**25. Play failed**

![play failed](screenshots/25-play-failed.png)

### LOD (render distance)

Build, rebuild after changes, quality, fine-tuning and details.

**26. Far never**

![far never](screenshots/26-far-never.png)

**27. Far building**

![far building](screenshots/27-far-building.png)

**28. Far done**

![far done](screenshots/28-far-done.png)

**29. Far stale quality**

![far stale quality](screenshots/29-far-stale-quality.png)

**30. Far finetune**

![far finetune](screenshots/30-far-finetune.png)

**31. Far details**

![far details](screenshots/31-far-details.png)

### Mods

Shared mod library with custom groups that fold and drag, rows that fold open on click, the built-in engine group (Built in or In progress, with links), install with options and review, and remove with confirm.

**32. Mods list**

![mods list](screenshots/32-mods-list.png)

**33. Mods builtin**

![mods builtin](screenshots/33-mods-builtin.png)

**34. Mods row open**

![mods row open](screenshots/34-mods-row-open.png)

**35. Mods untested card**

![mods untested card](screenshots/35-mods-untested-card.png)

**36. Mods search**

![mods search](screenshots/36-mods-search.png)

**37. Mods group menu**

![mods group menu](screenshots/37-mods-group-menu.png)

**38. Mods new group**

![mods new group](screenshots/38-mods-new-group.png)

**39. Mods drag**

![mods drag](screenshots/39-mods-drag.png)

**40. Mods after drag**

![mods after drag](screenshots/40-mods-after-drag.png)

**41. Install pick**

![install pick](screenshots/41-install-pick.png)

**42. Install options 1**

![install options 1](screenshots/42-install-options-1.png)

**43. Install options 2**

![install options 2](screenshots/43-install-options-2.png)

**44. Install review**

![install review](screenshots/44-install-review.png)

**45. Install busy**

![install busy](screenshots/45-install-busy.png)

**46. Install done**

![install done](screenshots/46-install-done.png)

**47. Install broken**

![install broken](screenshots/47-install-broken.png)

**48. Mods remove confirm**

![mods remove confirm](screenshots/48-mods-remove-confirm.png)

### Test your setup

On the Mods page: stability test, benchmark with history, location pictures with zoom and pan, and a filtered log with warnings and errors. A stability test is recommended whenever the exact mod set and order has not passed before.

**49. Test tab**

![test tab](screenshots/49-test-tab.png)

**50. Test stability running**

![test stability running](screenshots/50-test-stability-running.png)

**51. Test stability passed**

![test stability passed](screenshots/51-test-stability-passed.png)

**52. Test bench running**

![test bench running](screenshots/52-test-bench-running.png)

**53. Test bench done**

![test bench done](screenshots/53-test-bench-done.png)

**54. Test pictures zoom**

![test pictures zoom](screenshots/54-test-pictures-zoom.png)

**55. Test log**

![test log](screenshots/55-test-log.png)

**56. Test stability crashed**

![test stability crashed](screenshots/56-test-stability-crashed.png)

### Profiles and load order

Profiles share installed mods and groups; each has its own enabled set, order, settings, LOD, tests and versions. Load order sorts with a preview and flags missing masters, also on Home and in sidebar badges.

**57. Mods hardcore shared library**

![mods hardcore shared library](screenshots/57-mods-hardcore-shared-library.png)

**58. Load hardcore**

![load hardcore](screenshots/58-load-hardcore.png)

**59. Load sort preview**

![load sort preview](screenshots/59-load-sort-preview.png)

**60. Load sorted**

![load sorted](screenshots/60-load-sorted.png)

**61. Load missing master**

![load missing master](screenshots/61-load-missing-master.png)

**62. Home missing master**

![home missing master](screenshots/62-home-missing-master.png)

**63. Profile duplicate**

![profile duplicate](screenshots/63-profile-duplicate.png)

**64. Mods empty profile**

![mods empty profile](screenshots/64-mods-empty-profile.png)

**65. Mods ttw not added**

![mods ttw not added](screenshots/65-mods-ttw-not-added.png)

### Tale of Two Wastelands

TTW is a mod: added from Mods in four steps, and it can be turned on or off per profile.

**66. Ttw 1 notfound**

![ttw 1 notfound](screenshots/66-ttw-1-notfound.png)

**67. Ttw 1 found**

![ttw 1 found](screenshots/67-ttw-1-found.png)

**68. Ttw 2**

![ttw 2](screenshots/68-ttw-2.png)

**69. Ttw 3**

![ttw 3](screenshots/69-ttw-3.png)

**70. Ttw 3 installing**

![ttw 3 installing](screenshots/70-ttw-3-installing.png)

**71. Ttw 4 done**

![ttw 4 done](screenshots/71-ttw-4-done.png)

### System

Checks for updates on open. One Update button; every part that changes says "will be updated", with a change list.

**72. System checking**

![system checking](screenshots/72-system-checking.png)

**73. System**

![system](screenshots/73-system.png)

**74. System changes**

![system changes](screenshots/74-system-changes.png)

**75. System update failed**

![system update failed](screenshots/75-system-update-failed.png)

**76. System updating**

![system updating](screenshots/76-system-updating.png)

**77. System updated**

![system updated](screenshots/77-system-updated.png)

**78. System engines**

![system engines](screenshots/78-system-engines.png)

### Doctor, settings and the rest

Doctor checks, settings, high contrast, command palette, shortcuts, reset to the wizard, a skipped-setup home and reduced motion.

**79. Doctor not run**

![doctor not run](screenshots/79-doctor-not-run.png)

**80. Doctor running**

![doctor running](screenshots/80-doctor-running.png)

**81. Doctor results**

![doctor results](screenshots/81-doctor-results.png)

**82. Settings**

![settings](screenshots/82-settings.png)

**83. Settings storage profile**

![settings storage profile](screenshots/83-settings-storage-profile.png)

**84. Settings high contrast**

![settings high contrast](screenshots/84-settings-high-contrast.png)

**85. Palette**

![palette](screenshots/85-palette.png)

**86. Shortcuts**

![shortcuts](screenshots/86-shortcuts.png)

**87. Reset to wizard**

![reset to wizard](screenshots/87-reset-to-wizard.png)

**88. Home skipped ttw later far**

![home skipped ttw later far](screenshots/88-home-skipped-ttw-later-far.png)

**89. Far building reduced motion**

![far building reduced motion](screenshots/89-far-building-reduced-motion.png)
