---
status: accepted
date: 2026-11-27
---
# Mod lists are TOML files of references; Beautiful ships as one flat list

A **Mod list** is a file that names Mods and where to get them, so one command can show what an Instance lacks and, later,
fetch and install it into a Profile. It holds references only (Nexus ids, file ids, names, sizes, hashes): no archive
and no mod file is in it or ships with it ([ADR-0014](0014-only-own-and-open-assets-ship.md)).

Decided by the owner on 2026-11-27. The research behind it is the "Presets research 2026-11-27" report (format,
Nexus rules, estimate); this ADR keeps what was decided and drops the rest.

## The name

The research called it a Preset. GLOSSARY.md says Profile avoids that word, and a Preset also reads as a graphics or
ReShade setting (#564, #570). The term is **Mod list**; GLOSSARY.md gets the entry. It avoids "collection" (Nexus's
word for a list that Nexus hosts), "modpack" (implies archives travel with it) and "preset". MO2's `modlist.txt` is
a Profile's own file and stays that; the two are written "Mod list" and "`modlist.txt`".

A Mod list builds a Profile; it is not one. Applying a list leaves an ordinary Profile that the person owns and edits.

## The format

TOML, schema name `raul-modlist/1`, file extension `.toml`. TOML because the workspace already reads it everywhere
(`toml` is a pinned dependency, `known-good.toml` and the Contracts use it), and the YAML crates are either archived
or new to `cargo deny` and the registry cooldown. One file is one self-contained list; the format has **no `extends`
and no layers** in version 1.

```toml
schema = "raul-modlist/1"
id = "beautiful"
name = "Beautiful"
description = "..."
game = "fnv"
target = "ttw"                  # vanilla | ttw
plugins = "loot"                # loot-rs sorts the plugins; per-mod `plugins` are the names the Mod brings

[[groups]]                      # optional, for display and for choosing what to leave out
id = "lighting"
title = "Lighting"

[[mods]]                        # array order = priority order, lowest first: a later Mod wins a file conflict
id = "3d-rain"                  # unique slug
name = "3D Rain"                # the Mod folder name in the Instance
group = "weather"
enabled = true                  # false = listed, staged, switched off; `why_off` says why
version = "2.0"
plugins = ["CC - 3D Rain.esp"]
source = { type = "nexus", game = "newvegas", mod = 12345, file = 678, file_name = "...", size = 1234 }
```

Source types, all in version 1 because the Beautiful list needs them:

| `type` | Fields | Meaning |
|---|---|---|
| `nexus` | `game`, `mod`, `file` (optional), `file_name`, `size`, `md5`, `uploaded` | A Nexus file. `file` may be absent when only the upload time is known; the plan then links the mod's files page |
| `url` | `url`, `sha256`, `file_name` | A non-Nexus host (ModDB, GitHub). `sha256` is required |
| `manual` | `note`, `file_name`, `url` (all optional) | A file the person must find or already has; raul only lists it |
| `tool` | `tool`, `inputs` | Built by a Component from the person's own files (Tale of Two Wastelands) |
| `generated` | `tool` | Built last by a Component (the generated LOD, [ADR-0021](0021-one-generated-lod-mod-per-profile.md)) |
| `addon` | `id` | One of the add-ons raul carries (`raul addons`), made on the person's machine |

The parser checks the schema name, unique ids, that every `group` exists, that a `nexus` source has a mod id, that a
`url` source has a `sha256`, and that a `disabled` Mod says why. A newer major schema is refused with a plain line.

### Why no layering yet

The research proposed `extends` so that `beautiful` would be `full-300` plus a layer, and `mega-800` a layer on that.
Only Beautiful ships now, and the big lists are not installed end to end yet. Layering code with one list to test it on
is speculative, and a person who picks a list should get the whole look from one file. If a second list needs a shared
base, version 2 of the schema adds `extends` and the merge rules written up in the research (add, move, disable, id
unique across the chain); version 1 files stay valid.

## Beautiful is flattened, not shipped with a base

Beautiful was `extends: full-300` (312 Mods) plus 18 Mods, one move and two removals. The first shipped list is that
result written out as **one flat list**, plus the two Lucky 38 Mods below: 332 Mods, 330 of them enabled. Shipping the base beside it
was the other option. Flat wins because: the person asked for one thing ("get the full 3c look"), a second list is a
second file to keep in step and a second place a Mod can be missing, and the cost of flat is only a longer file that
is generated once. The owner's own `full-300` is a profile, not something we promise.

The flatten rules were applied once, by a script, and checked: Mod order is `full-300`'s with
`nuclear-trees-vanilla` moved after `s3-mojave-flora-project` and the added Mods after the Mod they named; `terrain-lod`
and `object-lod` are `enabled = false` ("replaced by the generated LOD"); the pinned plugin order of the research is
replaced by `plugins = "loot"` because raul links loot-rs and a pinned order of 142 names would go stale with every
Mod update.

### Lucky 38

The owner asked on 2026-11-27 for Lucky 38 interior overhaul(s) and Lucky 38 Exterior Remastered in the list; this
replaces the 2026-11-23 ban on the exterior. The research files and the three collection manifests give:

| Mod | Nexus mod | File | Plugin |
|---|---|---|---|
| The Mogul Mausoleum, a Lucky 38 Overhaul (interior) | 66622 | 1000059826 | `Lucky38Overhaul.esp` |
| Lucky 38 Exterior Remastered | 81005 | 1000108857 | `Lucky38ExtRemasterNew.esp` |

Left out on purpose: The Final Resting Place (74141, deleted on Nexus), Ultra-Luxe Overhaul Dark Side Edition (91786, the
Ultra-Luxe casino, not the Lucky 38) and the small Lucky 38 add-ons (posters, suite upgrades, light fixes), which can be
added later as plain entries. Whether the interior and exterior mods collide with the Strip lighting mods already in the
list has not been tested on nv-rs.

## Where lists live

`lists/<id>.toml` in this repository, one file per list, embedded in the `raul` binary so a list works offline and with
no download. `raul` also reads a list from any path the person gives it, which is how third-party and own lists work
(#554 said "no list ships with raul"; that note is superseded for Beautiful by the owner's decision of 2026-11-27).
The parser and the plan live in a new crate `raul-modlist`, with no dependency on a Component, so it builds and tests
quickly; the install into a Profile is `raul`'s and calls nvmod's install jobs ([ADR-0019](0019-raul-orchestrates-nvmod-manages-mods.md)).

Third-party Mod names appear in `lists/beautiful.toml`. They are references, as in any Vortex collection or Wabbajack
list; nothing of the Mods themselves is copied.

## Nexus key and download

Another piece of work owns the key (storage, validation, the client). What the plan and the installer rely on:

- **Key.** OS keyring by default; `NEXUS_API_KEY` wins when set; a 0600 file only after the person agrees. The key never
  enters a list, a log or a command line. `GET /v1/users/validate.json` with an `apikey` header; every request sends
  `Application-Name` and `Application-Version`; the hourly and daily limits are read from the response headers.
- **Premium** accounts may call `download_link.json` and get signed, short-lived links, never stored.
- **Free** accounts go through the website: the "Mod Manager Download" button gives an `nxm://` link whose `key` and
  `expires` let the app ask for the link (how Vortex and MO2 work; to be confirmed against a real link).
- **Manual** is the floor and is built first: the plan lists the Nexus page of each missing file
  (`https://www.nexusmods.com/<game>/mods/<mod>?tab=files&file_id=<file>`) or the other link, the person downloads,
  and raul imports the file from the downloads folder by name, then size, then md5.
- **Registration.** Nexus allows personal keys for personal use only; a public release needs an app registration.
  Every call is user-initiated and budget-aware (about three calls per Mod); raul never uploads, mirrors or
  scrapes pages; downloads go to the person's own machine.
- **The seam.** The plan asks a `FileResolver` trait for a file id and a link when the list has none. Today's
  implementation resolves nothing and the plan falls back to the files page; the Nexus client plugs in later without
  a change to the plan or the CLI.

## Order of work

1. This ADR, and the issues for the work below.
2. `raul-modlist`: schema, parser, validation, `lists/beautiful.toml`, tests on synthetic fixtures.
3. The install plan (present, missing, links, order) and `raul modlist show|plan`.
4. Next, tracked in issues: opening links and the Downloads watcher, install into a Profile in list order, FOMOD
   answers, premium download, the `nxm` handler, the GUI.

## Open questions, left open on purpose

- **Lighting in the big lists.** The 800-mod list has two lighting stacks (Beautiful's, and Lumen); which one wins is not
  decided.
- **Clutter.** The same, for the clutter mods that collide.
- **The hand-made cuts.** Eight Beautiful Mods have no archive the list can point at: `T NMC TTW 3b no roads`,
  `T NMC NV Pt2 no roads`, `T Nutscapes land`, `W Natural Waters` (cuts of a larger Mod, no recipe), the two re-packed S3
  Mods (`S3 Strip Lighting Overhaul`, `S3 Accurate NASA Stars`), and `T NMC NV Pt1` and `Pt3` (archive names, no Nexus
  id). They are `manual` sources with a note until the owner says whether they become a recipe (a copy of a subset of a
  source Mod), a download somewhere, or are replaced. Nine more `manual` Mods come from the base set (Stash Organizer,
  LODIFY, the eddoursul.win checklists, Vanilla UI Plus on ModDB and others) and have a name and often a link.
- **INI tweaks.** The research format carried a few `Fallout.ini` keys and per-Mod tweaks; version 1 leaves them out.
- **Nexus registration.** Whether and when raul registers as a Nexus app; until then the key is for the person's own use.
- **The big lists.** `full-300` and `mega-800` are not shipped and need layering (version 2).

## Considered options

- YAML with `extends` (the research draft): rejected for version 1, see the format and layering sections.
- Ship Beautiful as a layer plus the base list: rejected, two files for one wish.
- Keep the word Preset: rejected, GLOSSARY and the ReShade-preset tickets already use it for something else.

## Tickets

Under #554: #678 crate and Beautiful list, #679 plan and `raul modlist`, #680 links and Downloads watcher, #681 install into
a Profile, #682 FOMOD answers, #683 export, #684 lock, #685 premium download, #686 `nxm` handler, #687 non-Nexus sources and
phases, #688 recipe for the hand-made cuts, #689 7z and rar tools, #690 GUI, #691 layering for the big lists, #692 Nexus
registration.
