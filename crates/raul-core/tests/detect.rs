// SPDX-License-Identifier: GPL-3.0-only
//! Finding the player's game copies: Steam libraries, GOG, Epic and a folder the player picks.

use std::fmt::Write as _;
use std::path::Path;

use nvmod_testkit::{BaseBuilder, PeBuilder, TempDir, write_file};
use raul_core::detect::{Game, Platform, SearchRoots, detect, inspect};

const NV_DLC: [&str; 9] = [
    "DeadMoney.esm",
    "HonestHearts.esm",
    "OldWorldBlues.esm",
    "LonesomeRoad.esm",
    "GunRunnersArsenal.esm",
    "CaravanPack.esm",
    "ClassicPack.esm",
    "MercenaryPack.esm",
    "TribalPack.esm",
];

fn new_vegas(dir: &Path, builder: BaseBuilder) {
    builder.exe_version([1, 4, 0, 525]).write(dir);
    for dlc in NV_DLC {
        write_file(dir, &format!("Data/{dlc}"), b"synthetic");
    }
}

fn steam_libraries(home: &Path, libraries: &[&Path]) {
    let mut vdf = String::from("\"libraryfolders\"\n{\n");
    for (i, lib) in libraries.iter().enumerate() {
        let _ = write!(
            vdf,
            "\t\"{i}\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n",
            lib.display()
        );
    }
    vdf.push_str("}\n");
    write_file(
        home,
        ".local/share/Steam/steamapps/libraryfolders.vdf",
        vdf.as_bytes(),
    );
}

#[test]
fn a_new_vegas_copy_in_a_second_steam_library_is_found() {
    let home = TempDir::new("raul-home");
    let lib = TempDir::new("raul-lib");
    steam_libraries(
        home.path(),
        &[&home.path().join(".local/share/Steam"), lib.path()],
    );
    let game = lib.path().join("steamapps/common/Fallout New Vegas");
    new_vegas(&game, BaseBuilder::steam());
    let found = detect(&SearchRoots::new(Platform::Linux, home.path()));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].path, game);
    assert_eq!(found[0].game, Game::NewVegas);
    assert_eq!(found[0].storefront, "Steam");
}

#[test]
fn the_executable_version_of_a_copy_is_read_and_judged_supported() {
    let dir = TempDir::new("raul-nv");
    new_vegas(dir.path(), BaseBuilder::steam());
    let copy = inspect(dir.path()).unwrap();
    assert_eq!(copy.version.as_deref(), Some("1.4.0.525"));
    assert_eq!(copy.supported_version, Some(true));
}

#[test]
fn an_older_executable_version_is_judged_unsupported() {
    let dir = TempDir::new("raul-old");
    BaseBuilder::steam()
        .exe_version([1, 2, 0, 314])
        .write(dir.path());
    assert_eq!(inspect(dir.path()).unwrap().supported_version, Some(false));
}

#[test]
fn missing_dlc_are_listed() {
    let dir = TempDir::new("raul-nodlc");
    BaseBuilder::gog()
        .exe_version([1, 4, 0, 525])
        .write(dir.path());
    write_file(dir.path(), "Data/DeadMoney.esm", b"synthetic");
    let copy = inspect(dir.path()).unwrap();
    assert!(copy.missing_dlc.contains(&"HonestHearts.esm".to_string()));
    assert!(!copy.missing_dlc.contains(&"DeadMoney.esm".to_string()));
}

#[test]
fn a_folder_without_a_game_executable_is_not_a_game_copy() {
    let dir = TempDir::new("raul-empty");
    write_file(dir.path(), "Data/FalloutNV.esm", b"synthetic");
    assert!(inspect(dir.path()).is_none());
}

#[test]
fn fallout_3_goty_is_found_for_tale_of_two_wastelands() {
    let home = TempDir::new("raul-fo3home");
    let steam = home.path().join(".local/share/Steam");
    steam_libraries(home.path(), &[&steam]);
    let fo3 = steam.join("steamapps/common/Fallout 3 goty");
    write_file(&fo3, "Fallout3.exe", &PeBuilder::exe().build());
    write_file(&fo3, "Data/Fallout3.esm", b"synthetic");
    let found = detect(&SearchRoots::new(Platform::Linux, home.path()));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].game, Game::Fallout3);
    assert!(
        found[0]
            .missing_dlc
            .contains(&"BrokenSteel.esm".to_string())
    );
}

#[test]
fn an_epic_manifest_points_at_its_install_location() {
    let home = TempDir::new("raul-winhome");
    let data = TempDir::new("raul-programdata");
    let game = TempDir::new("raul-epicgame");
    new_vegas(game.path(), BaseBuilder::epic());
    let manifest =
        serde_json::json!({ "DisplayName": "Fallout: New Vegas", "InstallLocation": game.path() });
    write_file(
        data.path(),
        "Epic/EpicGamesLauncher/Data/Manifests/abc.item",
        manifest.to_string().as_bytes(),
    );
    let mut roots = SearchRoots::new(Platform::Windows, home.path());
    roots.program_data = Some(data.path().to_path_buf());
    let found = detect(&roots);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].storefront, "Epic");
}

#[test]
fn a_copy_reachable_from_two_roots_is_reported_once() {
    let home = TempDir::new("raul-dup");
    let steam = home.path().join(".local/share/Steam");
    steam_libraries(home.path(), &[&steam, &steam]);
    new_vegas(
        &steam.join("steamapps/common/Fallout New Vegas"),
        BaseBuilder::steam(),
    );
    assert_eq!(
        detect(&SearchRoots::new(Platform::Linux, home.path())).len(),
        1
    );
}
