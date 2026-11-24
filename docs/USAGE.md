# Using raul from the command line

Every page of raul's window has a command, so raul also works in a terminal or a script:

```sh
raul detect                                   # find your game copies
raul init --base "<game folder>" --instance ~/raul/fnv
raul profile create --instance ~/raul/fnv
raul install <mod archive> --instance ~/raul/fnv
raul sort --instance ~/raul/fnv --fetch-masterlist --apply
raul ttw --instance ~/raul/fnv                 # Tale of Two Wastelands, if you want it
raul target nv-rs --instance ~/raul/fnv        # run on the engine built into raul; nothing to install
raul lod generate --instance ~/raul/fnv
raul addons list --instance ~/raul/fnv        # raul's add-ons for the mods you have, and where each stands
raul run nv-rs --instance ~/raul/fnv
```

`raul help` lists the Toolbox's own commands, `raul nvmod-help` every nvmod command; add `--json` for scripts.

## Add-ons

An add-on is a small Mod raul makes on your computer for a mod you installed, from numbers and from your own copy of
that mod's plugin; nothing of the game or the mod is downloaded or shipped. The first one is **Raul Small Lamp Glow**
for Street Lights TTW: it swaps the wide glow discs over the street lamps for a small, soft halo per bulb, and keeps
the lamps' light and beams.

- `raul setup` adds the add-ons at its end, and `raul run` and `raul scene` check them before the game starts. Each
  change prints one line, such as `raul add-on Raul Small Lamp Glow: added the Mod ...`. A check that goes wrong
  prints a warning, and the game starts anyway.
- The add-on's Mod goes right above the mod it is for, and its plugin right after that mod's plugin. The Mods page
  tags it "Made by raul".
- Turn it off on the Mods page (or `raul mods disable "Raul Small Lamp Glow"`) and raul leaves it off; remove it and
  raul does not add it back. A Profile without the mod's plugin is never touched.
- `raul addons apply` does the same by hand; `--dry-run` only says what it would change.

The documents a command prints with `--json`, and its exit codes, are in [contracts/README.md](../contracts/README.md).
A tour of the window, page by page, is in [TOUR.md](TOUR.md).
