---
status: accepted
date: 2026-11-19
---
# One generated LOD Mod per Profile

The LOD that `raul lod generate` builds is kept as **one Mod per Profile**, holding every world of that Profile. A
Profile's LOD settings (`raul-lod.toml` in the Profile's folder) and the LOD built from them stay together: two
Profiles with different plugins or settings never share or overwrite each other's LOD.

Decided by the owner on 2026-11-19 (decisions G1, D8, D9 and D11 of the LOD plan, recorded in #352).

## The rules

- **Name.** `Generated LOD (<Profile>)`: one constant prefix and the Profile's name. The name is built in one place
  (`raul_drivers::lod::mod_name`); the GUI shows the name the run reports and keeps no copy of its own. The name
  carries no tool name (UI-RULES: no command or tool names in plain view).
- **Marker.** The Mod's `meta.ini` carries an MO2-compatible section `[raulGeneratedLod]` with `generated=true`,
  `profile`, `requestDigest` (lodwright's digest of the request it was built from) and `lodwrightVersion`. Every
  other key of an existing `meta.ini` is kept, so a category or notes the person set survive a regenerate. Code
  recognises generated LOD by the marker, not by the folder name. A Mod with the generated name but without the
  marker is not ours: a generate refuses to replace it and names it.
- **Enabled in its own Profile only.** A generate adds or replaces the Mod in the Profile it runs for. Other
  Profiles never list it.
- **What a person's choices mean.** Disabling the Mod suppresses it: the next generate replaces its files and keeps
  it disabled. Deleting it means the next generate makes it again, on top and enabled. Moving it keeps its place: a
  regenerate puts the new copy where the old one was. A Mod made for the first time goes on top, enabled.
- **Folders.** lodwright keeps its output and manifest per Profile, in `<instance>/.raul/lod/<Profile>/out`, so
  switching Profiles does not make the other Profile's next run rebuild from nothing. Until that change lands the
  folder is `<instance>/.raul/lod/out`, shared by the Profiles; that is correct (lodwright rebuilds whenever the
  request differs) but slower when two Profiles generate in turn.
- **Staleness** (#66) is tracked per Profile: the request digest in the marker against the request the Profile gives
  now.
- **Profile rename, copy and delete.** A rename renames the Mod with it (and its marker's `profile`). A copy copies
  `raul-lod.toml` with the Profile but not the Mod: the copy's first generate makes its own. Deleting a Profile
  removes its LOD Mod only after the person confirms it in the same dialog.
- **Out of scope.** One Mod per world. A Profile's worlds stay in one Mod.

## Migration of the shared Mod

Before this decision every Profile shared one Mod, `LOD (lodwright)`. It still counts as generated LOD. The next
generate of a Profile that lists it gives that Profile's own Mod its place and enabled state, then removes the old
Mod, or only disables it in that Profile while another Profile still lists it (that Profile migrates on its own next
run). A `… previous` backup an interrupted run left is recovered under either name.

## Consequences

- The Mods page (#97) and the generated LOD row (#341) find these Mods through `raul lod mods` and the marker, and
  can give them their own treatment without knowing their names.
- Update checks and conflict warnings leave generated LOD out (nvmod-rs#197), using the marker.
- A Profile's LOD settings view (#338) edits the settings of exactly the Profile whose Mod it describes.

## Considered options

- One Mod for every Profile (the state before): rejected. Profiles with different plugins overwrite each other's
  LOD, and a Mod enabled in one Profile shows LOD built for another.
- One Mod per world: rejected for now. A Tale of Two Wastelands load order has dozens of worlds, and the Mods list
  would fill with them.
- `LOD (lodwright) <Profile>`: rejected for the shown name, since it puts a tool name in plain view.
