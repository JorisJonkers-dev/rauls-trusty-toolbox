/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== mock data: mods, plugins, parts, places. Names are public mod names; nothing here comes from anyone's own files. ===== */
const BASE = ['FalloutNV.esm', 'DeadMoney.esm', 'HonestHearts.esm', 'OldWorldBlues.esm', 'LonesomeRoad.esm', 'GunRunnersArsenal.esm', 'ClassicPack.esm', 'MercenaryPack.esm', 'TribalPack.esm', 'CaravanPack.esm'];
const TTWP = ['Fallout3.esm', 'Anchorage.esm', 'ThePitt.esm', 'BrokenSteel.esm', 'PointLookout.esm', 'Zeta.esm', 'TaleOfTwoWastelands.esm'];

/* kind: plugin | dll | assets. Plugins: [file, rank (masterlist position), masters, masterlist note] */
const CAT = {
  yup: { name: 'YUP - Base Game and All DLC', ver: '16.0', kind: 'plugin', size: 38, plugins: [['YUP - Base Game + All DLC.esm', 10, [], '']], what: 'Fixes thousands of bugs in the base game and DLC.' },
  nvac: { name: 'NVAC - New Vegas Anti Crash', ver: '7.5.1', kind: 'dll', size: 1, plugins: [], what: 'Catches many crashes before they end the game.' },
  nvtf: { name: 'NVTF - New Vegas Tick Fix', ver: '6.4', kind: 'dll', size: 1, plugins: [], what: 'Smooths frame pacing and fixes timing bugs at high frame rates.' },
  lrf: { name: 'Lonesome Road Fixes', ver: '2.1', kind: 'plugin', size: 2, plugins: [['LonesomeRoadFixes.esp', 40, ['LonesomeRoad.esm'], 'Loads after LonesomeRoad.esm, as the masterlist asks.']], what: 'Quest and level fixes for Lonesome Road.' },
  fco: { name: 'Fallout Character Overhaul', ver: '3.0.5', kind: 'plugin', size: 412, plugins: [['Fallout Character Overhaul.esp', 70, [], '']], what: 'New faces, hair and eyes for every character.' },
  nsk: { name: 'Nevada Skies', ver: '2.6', kind: 'plugin', size: 96, plugins: [['Nevada Skies - Basic Edition.esp', 60, [], 'Load near the end: weather records win last.']], what: 'Weather, clouds, night skies and storms.' },
  nmc: { name: "NMC's Texture Pack", ver: '1.0', kind: 'assets', size: 2860, plugins: [], what: 'Sharper landscape and architecture textures.' },
  ojo: { name: 'Ojo Bueno Texture Pack', ver: '2.4', kind: 'assets', size: 1480, plugins: [], what: 'Higher-resolution clutter and town textures.' },
  ilo: { name: 'Interior Lighting Overhaul', ver: '9.2', kind: 'plugin', size: 22, plugins: [['Interior Lighting Overhaul - Core.esm', 20, [], ''], ['ILO - New Vegas.esp', 65, ['Interior Lighting Overhaul - Core.esm'], '']], what: 'Rebuilt light sources and shadows indoors.' },
  wrp: { name: 'Weapon Retexture Project', ver: '1.4', kind: 'plugin', size: 640, plugins: [['WeaponRetexture.esp', 55, [], '']], what: 'New models and textures for the base weapons.' },
  wfo: { name: 'Wasteland Flora Overhaul', ver: '1.2', kind: 'plugin', size: 210, plugins: [['WFO.esp', 62, [], 'LOD: rebuild after adding or removing this mod.']], what: 'Desert trees, shrubs and grass.' },
  jsue: { name: 'JSawyer Ultimate Edition', ver: '1.8', kind: 'plugin', size: 9, plugins: [['JSawyer Ultimate.esm', 22, [], '']], what: 'Rebalanced hardcore: level cap, needs, weapons and perks.' },
  wmx: { name: 'Weapon Mods Expanded', ver: '2.1', kind: 'plugin', size: 380, plugins: [['WMX.esm', 24, [], ''], ['WMX-DLCMerged.esp', 50, ['WMX.esm'], '']], what: 'New weapon mods for the base weapons and the DLC.' },
  tt: { name: 'Tougher Traders', ver: '1.3', kind: 'plugin', size: 1, plugins: [['Tougher Traders.esp', 58, [], '']], what: 'Merchants keep less caps and charge more.' },
  b42: { name: 'B42 Optics', ver: '1.3', kind: 'plugin', size: 54, plugins: [['B42 Optics.esp', 52, [], '']], what: '3D scopes with real magnification.' },
  tld: { name: 'The Living Desert', ver: '1.0', kind: 'plugin', size: 15, plugins: [['The Living Desert.esp', 56, [], '']], what: 'Adds wildlife and encounters to empty stretches.' },
  war: { name: 'Weapon Animation Replacers', ver: '3.2', kind: 'assets', size: 120, plugins: [], what: 'New first-person animations for many weapons.' }
};
/* patch plugins an install option can add */
const EXTRA_PLUGINS = { 'B42 Optics - WMX.esp': [54, ['WMX.esm'], 'Needs Weapon Mods Expanded.', 'b42'], 'B42 Optics - WRP.esp': [57, ['WeaponRetexture.esp'], 'Needs Weapon Retexture Project.', 'b42'], 'The Living Desert - JSUE.esp': [59, ['JSawyer Ultimate.esm'], 'Needs JSawyer Ultimate Edition.', 'tld'], 'Nevada Skies - Full Edition.esp': [60, [], 'Load near the end: weather records win last.', 'nsk'] };

/* archives in the picker ("Downloads" in the Toolbox's download folder) */
const ARCH = [
  { file: 'B42 Optics-1.3.7z', mod: 'b42', size: 54, fomod: [
    { id: 'style', t: 'Scope style', kind: 'one', opts: [['3d', '3D scopes', 'You look through the real lens. Recommended.'], ['2d', 'Classic overlay', 'The flat scope picture from the base game.']] },
    { id: 'patch', t: 'Compatibility', kind: 'many', opts: [['wmx', 'Weapon Mods Expanded patch', 'Scopes on WMX weapons.'], ['wrp', 'Weapon Retexture Project patch', 'Scopes on the retextured weapons.']] }] },
  { file: 'The Living Desert-1.0.zip', mod: 'tld', size: 15, fomod: [
    { id: 'enc', t: 'Encounters', kind: 'one', opts: [['normal', 'Normal', 'About one new encounter per stretch of road.'], ['more', 'More', 'Busier roads. Harder early game.']] },
    { id: 'patch', t: 'Compatibility', kind: 'many', opts: [['jsue', 'JSawyer Ultimate patch', 'Uses JSUE’s creature stats.']] }] },
  { file: 'Nevada Skies-2.6.7z', mod: 'nsk', size: 96, fomod: [
    { id: 'ed', t: 'Edition', kind: 'one', opts: [['basic', 'Basic edition', 'Weather and skies only.'], ['full', 'Full edition', 'Adds storms with sound and the dust wall.']] }] },
  { file: 'Weapon Animation Replacers-3.2.zip', mod: 'war', size: 120 },
  { file: 'Tougher Traders-1.3.zip', mod: 'tt', size: 1 },
  { file: 'Interior Lighting Overhaul-9.2.7z', mod: 'ilo', size: 22 },
  { file: 'Wasteland Flora Overhaul-1.2.7z', mod: 'wfo', size: 210 },
  { file: 'Desert retexture (part 2 of 2).7z.002', mod: null, size: 840, broken: true }
];

/* built into the engine: no on/off, a state and two links */
const BUILTIN = [
  ['xNVSE', 'Built in', 'Script extender'], ['JIP LN NVSE', 'Built in', 'Script functions and fixes'], ['JohnnyGuitar NVSE', 'In progress', 'Script functions'],
  ['ShowOff xNVSE', 'In progress', 'Script functions'], ['kNVSE', 'In progress', 'Animation framework'], ["lStewieAl's Tweaks", 'In progress', 'Engine tweaks and fixes'],
  ['Mod Configuration Menu', 'Built in', 'In-game mod settings'], ['UI Organizer', 'Built in', 'Menu layout for mods'], ['TTW NVSE plugin', 'Built in', 'Script support for TTW']
];

/* example profiles: groups [name, open, [mod ids]], ungrouped mods, TTW on, out-of-order plugins (index swaps) */
/* one shared mod library and its groups; each example profile switches on its own subset */
const LIB = [['Fixes', ['yup', 'nvac', 'nvtf', 'lrf']], ['Textures', ['nmc', 'ojo', 'wrp']], ['Light and sky', ['ilo', 'nsk']], ['Gameplay', ['jsue', 'wmx', 'b42', 'tt']], ['Ungrouped', ['fco', 'wfo']]];
const SEED = [
  { id: 'vplus', name: 'Vanilla+', blurb: 'The base game with fixes and a few light touches.', on: ['yup', 'nvac', 'nvtf', 'lrf', 'fco', 'nsk'], open: ['Fixes'], ttw: false, q: 'balanced', stable: true, swap: [],
    gfx: { res: '1920 × 1080', cap: '60', up: 'Off', look: 'Original' }, bench: [61, 63, 62, 64, 65] },
  { id: 'beaut', name: 'Beautiful', blurb: 'Textures, light and weather turned up.', on: ['yup', 'nvac', 'nvtf', 'nmc', 'ojo', 'wrp', 'ilo', 'nsk', 'fco', 'wfo'], open: ['Textures', 'Light and sky', 'Ungrouped'], ttw: false, q: 'high', stable: true, swap: [],
    gfx: { res: '2560 × 1440', cap: '120', up: 'FSR', look: 'Original' }, bench: [68, 70, 66, 71, 74] },
  { id: 'hard', name: 'Hardcore', blurb: 'Both wastelands, tougher rules, fewer caps.', on: ['yup', 'nvac', 'nvtf', 'jsue', 'wmx', 'b42', 'tt'], open: ['Gameplay'], ttw: true, q: 'performance', stable: false, swap: [[1, 3], [5, 6]],
    gfx: { res: '1920 × 1080', cap: '90', up: 'Off', look: 'Original' }, bench: [58, 57, 59] }
];

const QUAL = { performance: ['Performance', 0.9, 8, 'Smaller textures, nearer distance, sparse trees.'], balanced: ['Balanced', 1.9, 15, 'Matches the game’s own LOD.'], high: ['High', 3.4, 26, 'Sharper textures, farther, denser trees.'] };

/* preview places: [id, name, area, kind]; kinds pick the drawn landmark. The Capital Wasteland needs TTW on. */
const AREAS = ['South', 'Vegas', 'East and the river', 'North and west', 'Capital Wasteland'];
const PLACES = [
  ['gs', 'Goodsprings', 'South', 'town'], ['primm', 'Primm', 'South', 'coaster'], ['nipton', 'Nipton', 'South', 'town'], ['novac', 'Novac', 'South', 'dino'], ['outpost', 'Mojave Outpost', 'South', 'base'],
  ['sloan', 'Sloan', 'South', 'quarry'], ['quarry', 'Quarry Junction', 'South', 'quarry'], ['searchlight', 'Searchlight', 'South', 'town'], ['jean', 'Jean Sky Diving', 'South', 'town'],
  ['strip', 'The Strip', 'Vegas', 'city'], ['freeside', 'Freeside', 'Vegas', 'ruins'], ['westside', 'Westside', 'Vegas', 'town'], ['northvegas', 'North Vegas', 'Vegas', 'ruins'], ['mccarran', 'Camp McCarran', 'Vegas', 'base'], ['tops', 'The Tops', 'Vegas', 'city'],
  ['dam', 'Hoover Dam', 'East and the river', 'dam'], ['fort', 'The Fort', 'East and the river', 'base'], ['boulder', 'Boulder City', 'East and the river', 'town'], ['mead', 'Lake Mead', 'East and the river', 'water'], ['colorado', 'Colorado River', 'East and the river', 'water'], ['cottonwood', 'Cottonwood Cove', 'East and the river', 'water'], ['lasvegasblvd', 'Bitter Springs', 'East and the river', 'base'],
  ['red', 'Red Rock Canyon', 'North and west', 'rocks'], ['jacobstown', 'Jacobstown', 'North and west', 'lodge'], ['blackmtn', 'Black Mountain', 'North and west', 'mountain'], ['helios', 'Helios One', 'North and west', 'solar'], ['v22', 'Vault 22', 'North and west', 'vault'], ['bonnie', 'Bonnie Springs', 'North and west', 'town'], ['ranch', 'NCR Sharecropper Farms', 'North and west', 'farm'],
  ['meg', 'Megaton', 'Capital Wasteland', 'dome'], ['rivet', 'Rivet City', 'Capital Wasteland', 'ship'], ['tenpenny', 'Tenpenny Tower', 'Capital Wasteland', 'tower'], ['citadel', 'The Citadel', 'Capital Wasteland', 'fortress'], ['monument', 'Washington Monument', 'Capital Wasteland', 'monument'],
  ['dupont', 'Dupont Circle', 'Capital Wasteland', 'ruinsdc'], ['georgetown', 'Georgetown', 'Capital Wasteland', 'ruinsdc'], ['arefu', 'Arefu', 'Capital Wasteland', 'bridge'], ['paradise', 'Paradise Falls', 'Capital Wasteland', 'town'], ['bigtown', 'Big Town', 'Capital Wasteland', 'town'],
  ['canterbury', 'Canterbury Commons', 'Capital Wasteland', 'town'], ['underworld', 'Underworld', 'Capital Wasteland', 'ruinsdc'], ['jefferson', 'Jefferson Memorial', 'Capital Wasteland', 'memorial'], ['v101', 'Vault 101', 'Capital Wasteland', 'vault'], ['gnr', 'Galaxy News Radio', 'Capital Wasteland', 'ruinsdc']
];

/* System page: parts and their change lists */
const PARTS = [
  { id: 'toolbox', n: "Raul's Toolbox", role: 'This app', icon: 'app', cur: '0.5.0', nw: '0.6.0', v: [
    ['0.6.0', '21 Nov', [['feat', 'Install a mod from an archive, with its install options'], ['feat', 'Undo the last change on the Mods page'], ['feat', 'Tells when the LOD is out of date'], ['feat', 'Profiles: switch the whole setup at once'], ['feat', 'Checks free space before the TTW install']]],
    ['0.5.0', '19 Nov', [['feat', 'Setup installs the engine without a separate program'], ['feat', 'LOD built in the app']]]] },
  { id: 'engine', n: 'Engine', role: 'Runs the game', icon: 'bolt', cur: '0.24.0', nw: '0.24.0', v: [
    ['0.24.0', '24 Nov', [['feat', 'Balanced fill light indoors and out'], ['feat', 'A subtle moon highlight'], ['fix', 'A dead light no longer stops the game'], ['fix', 'Some textures no longer draw black']]],
    ['0.23.0', '23 Nov', [['feat', 'Graphics overlay in the game, on F10'], ['feat', 'Reads BC4, BC5 and BC7 textures'], ['fix', 'New Game no longer hangs after a graphics card change']]]] },
  { id: 'lodwright', n: 'lodwright', role: 'Builds the LOD', icon: 'lod', cur: '0.17.0', nw: '0.17.0', v: [
    ['0.17.0', '24 Nov', [['feat', 'Far shapes for models without an LOD model'], ['feat', 'Error report per level']]],
    ['0.16.3', '23 Nov', [['fix', 'Marks the packed LOD so it wins over other files']]]] },
  { id: 'nvmod', n: 'nvmod', role: 'Manages mods', icon: 'mods', cur: '0.17.1', nw: '0.18.0', v: [
    ['0.18.0', '24 Nov', [['feat', 'Stage a profile into another folder']]],
    ['0.17.1', '24 Nov', [['fix', 'A generated LOD wins over every other mod’s files']]]] },
  { id: 'loot', n: 'loot-rs', role: 'Sorts the load order', icon: 'load', cur: '0.9.0', nw: '0.10.0', v: [
    ['0.10.0', '25 Nov', [['feat', 'Reads masterlist messages for more plugins'], ['fix', 'Faster sort for big load orders']]],
    ['0.9.0', '21 Nov', [['feat', 'Shows why a plugin moved']]]] },
  { id: 'ttwright', n: 'ttwright', role: 'Builds Tale of Two Wastelands', icon: 'ttw', cur: '0.5.0', nw: '0.6.0', v: [
    ['0.6.0', '25 Nov', [['feat', 'Pick where the Fallout 3 files come from'], ['fix', 'Checks free space before it starts']]],
    ['0.5.0', '20 Nov', [['feat', 'Faster audio conversion'], ['fix', 'Never writes outside the destination']]]] }
];

/* Settings › game folder picker (generic locations) */
const FOLDERS = {
  fnv: [['Steam library', 'D:\\SteamLibrary\\steamapps\\common\\Fallout New Vegas', true], ['GOG Games', 'C:\\GOG Games\\Fallout New Vegas', true], ['Downloads', 'C:\\Users\\Player\\Downloads', false]],
  fo3: [['Steam library', 'D:\\SteamLibrary\\steamapps\\common\\Fallout 3 goty', true], ['GOG Games', 'C:\\GOG Games\\Fallout 3 GOTY', true], ['Documents', 'C:\\Users\\Player\\Documents', false]],
  store: [['Fast drive', 'E:\\RaulsToolbox', true, 412], ['Game drive', 'D:\\RaulsToolbox', true, 142], ['USB stick', 'F:\\RaulsToolbox', true, 11]]
};
