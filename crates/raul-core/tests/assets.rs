// SPDX-License-Identifier: GPL-3.0-only
//! ADR-0014: only our own and openly licensed assets ship. Every non-code asset in the repository is listed in
//! `packaging/NOTICE-assets.toml` with its source and licence, and no game-style file or path is in it (also not as
//! a fixture). It walks the working tree (no git needed). Runs with the workspace tests in pull-request CI.

use raul_testkit::{covers, ids};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Extensions of files that are art, fonts, audio or documents shown to people: not code, so they need a listing.
const ASSET_EXTENSIONS: [&str; 22] = [
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "icns", "svg", "ttf", "otf", "woff",
    "woff2", "html", "wav", "ogg", "mp3", "flac", "mp4", "webm", "pdf", "ani",
];

/// Formats and paths of the game's and its mods' own files, which never enter the repository.
const GAME_EXTENSIONS: [&str; 9] = [
    "dds", "tex", "fnt", "swf", "bsa", "ba2", "nif", "esm", "esp",
];
const GAME_FOLDERS: [&str; 6] = ["meshes", "textures", "interface", "sound", "music", "video"];

/// Folders whose images our own tests make (rendered screenshots of the window): they are output of the code, not
/// assets, and are not listed one by one.
const GENERATED: [&str; 1] = ["crates/raul-gui/tests/snapshots/"];

fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .map_or_else(String::new, |e| e.to_string_lossy().to_lowercase())
}

/// What is wrong with `files` (paths from the repository root) given the `listed` assets; empty when nothing is.
fn problems(files: &[String], listed: &BTreeSet<String>) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let ext = extension(file);
        if GAME_EXTENSIONS.contains(&ext.as_str()) {
            found.push(format!("{file}: a game-style format (.{ext})"));
        }
        if let Some(folder) = file
            .split('/')
            .rev()
            .skip(1)
            .find(|part| GAME_FOLDERS.contains(&part.to_lowercase().as_str()))
        {
            found.push(format!("{file}: a game-style path ({folder}/)"));
        }
        if ASSET_EXTENSIONS.contains(&ext.as_str())
            && !GENERATED.iter().any(|g| file.starts_with(g))
            && !listed.contains(file)
        {
            found.push(format!(
                "{file}: not listed in packaging/NOTICE-assets.toml"
            ));
        }
    }
    for path in listed {
        if !files.contains(path) {
            found.push(format!(
                "{path}: listed in packaging/NOTICE-assets.toml but not in the repository"
            ));
        }
    }
    found
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Every file of the working tree, from the root, except what is not the repository's own: git's folder, build
/// output, installed packages and the submodules (other repositories with their own checks).
fn tracked() -> Vec<String> {
    fn walk(dir: &Path, root: &Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                if !matches!(
                    name.as_str(),
                    ".git" | "target" | "node_modules" | "components"
                ) {
                    walk(&path, root, found);
                }
            } else {
                let relative = path.strip_prefix(root).unwrap();
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let root = root();
    let mut found = Vec::new();
    walk(&root, &root, &mut found);
    found.sort();
    found
}

fn listed() -> BTreeSet<String> {
    let text = std::fs::read_to_string(root().join("packaging/NOTICE-assets.toml")).unwrap();
    let doc: toml::Table = text.parse().unwrap();
    doc["asset"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["path"].as_str().unwrap().to_owned())
        .collect()
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

#[test]
fn the_repository_has_no_unlisted_asset_and_no_game_style_file() {
    covers(&[ids::guarantees::build::NO_GAME_FILES]);
    let found = problems(&tracked(), &listed());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn an_unlisted_asset_fails_and_a_listed_one_passes() {
    let listed: BTreeSet<String> = ["art/logo.png".to_owned()].into();
    assert_eq!(
        problems(&strings(&["art/logo.png", "src/main.rs"]), &listed),
        Vec::<String>::new()
    );
    let found = problems(&strings(&["art/logo.png", "art/other.PNG"]), &listed);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("art/other.PNG: not listed"));
}

#[test]
fn game_style_formats_and_paths_fail_even_when_listed() {
    let listed: BTreeSet<String> = ["fixtures/x.dds".to_owned()].into();
    let found = problems(
        &strings(&["fixtures/x.dds", "fixtures/Textures/a.json", "a.bsa"]),
        &listed,
    );
    assert_eq!(found.len(), 3, "{found:?}");
}

#[test]
fn a_listed_asset_that_is_gone_fails_and_generated_screenshots_need_no_listing() {
    let listed: BTreeSet<String> = ["gone.png".to_owned()].into();
    let found = problems(
        &strings(&["crates/raul-gui/tests/snapshots/home/home.png"]),
        &listed,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("but not in the repository"));
}
