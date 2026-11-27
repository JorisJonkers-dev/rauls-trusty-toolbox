<!-- SPDX-License-Identifier: GPL-3.0-only -->
# Getting started

## What you need

- Your own copy of Fallout: New Vegas (Steam, GOG or Epic).
- Your own copy of Fallout 3 and the installer file from the TTW site, but only if you want Tale of Two Wastelands.
- About 25 GB of free space for Tale of Two Wastelands, and a few GB without it.

## Download

Open the [latest release](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases/latest) and take the file
for your computer.

| System | File | Notes |
| --- | --- | --- |
| Linux (x64, Steam Deck) | `.AppImage`, `.deb` or `.tar.gz` | Make the AppImage runnable (`chmod +x`) and start it. Or install the `.deb` with `sudo apt install ./raul-*.deb`. |
| Windows 10 22H2 or 11 (x64) | `-setup.exe` or `-portable.zip` | Installs for your user only. It is not signed yet, so SmartScreen may warn you. |
| macOS 13 or newer (Apple silicon and Intel) | `.dmg` | Not signed yet. Allow it under System Settings, Privacy and Security. |

## Check the download

The release has a file called `SHA256SUMS`. Compare your download with it:

- Linux: `sha256sum -c SHA256SUMS --ignore-missing`, in the download folder.
- macOS: `shasum -a 256 <file>`, then compare with the matching line.
- Windows (PowerShell): `Get-FileHash <file>`, then compare with the matching line.

The [install guide](../INSTALL.md) also shows how to check where the file was built.

## Linux libraries

The `.deb` brings the libraries the window needs. The AppImage and the `.tar.gz` do not. On Debian or Ubuntu, install
them first: the list is in the [install guide](../INSTALL.md#which-file-and-how-to-check-it).

## First start

raul opens a short setup. It looks for New Vegas in your Steam, GOG and Epic libraries, asks if you want Tale of Two
Wastelands, makes a folder for your mods and profiles, and runs a quick check. Nothing is changed in your game folder.
See [Using the Toolbox](using-the-toolbox.md) for what each page does afterwards.

## Build it yourself

```sh
git clone --recursive https://github.com/JorisJonkers-dev/rauls-trusty-toolbox.git
cd rauls-trusty-toolbox
cargo run -p raul-cli -- --version
```

The install guide has the details: [Installing raul](../INSTALL.md#from-source).

## Updates

raul checks for a new release only when you ask. An update keeps the version before it, so you can go back.

## Legal

raul is an independent project. It is not affiliated with or endorsed by Bethesda Softworks, ZeniMax Media or
Obsidian Entertainment. It is free software under the GPL-3.0-only licence.
