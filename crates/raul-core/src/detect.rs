// SPDX-License-Identifier: GPL-3.0-only
//! Finding the player's own game copies: Steam libraries (including Steam Deck and the Flatpak Steam), GOG and Epic
//! install folders, and a folder the player picks. Only reads; nothing is changed. The storefront is decided by
//! nvmod's marker files (`nvmod_instance::detect_storefront`), the version by the executable's version resource.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use nvmod_instance::{detect_storefront, find_ci};
use serde::Serialize;

/// The game a copy is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Game {
    /// Fallout: New Vegas.
    NewVegas,
    /// Fallout 3 (needed for TTW).
    Fallout3,
}

/// The operating system whose usual install locations are searched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux, including Steam Deck.
    Linux,
    /// Windows.
    Windows,
    /// macOS.
    MacOs,
}

/// Where to look.
#[derive(Debug, Clone)]
pub struct SearchRoots {
    /// Which platform's usual folders to try.
    pub platform: Platform,
    /// The person's home folder.
    pub home: PathBuf,
    /// Windows `Program Files (x86)`.
    pub program_files_x86: Option<PathBuf>,
    /// Windows `ProgramData` (Epic's manifests).
    pub program_data: Option<PathBuf>,
    /// Further folders that may hold a game copy directly, or a `steamapps` library.
    pub extra: Vec<PathBuf>,
}

/// One game copy found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameCopy {
    /// Which game.
    pub game: Game,
    /// The copy's folder (the one holding the executable).
    pub path: PathBuf,
    /// The storefront nvmod recognises from marker files: `Steam`, `GOG`, `Epic` or `unknown`.
    pub storefront: String,
    /// The executable's file version, when it has a version resource.
    pub version: Option<String>,
    /// Whether that version is the one the Toolbox supports; `None` when it is not known.
    pub supported_version: Option<bool>,
    /// The DLC master files the copy lacks.
    pub missing_dlc: Vec<String>,
}

const NV_EXE: &str = "FalloutNV.exe";
const FO3_EXE: &str = "Fallout3.exe";
/// The only New Vegas executable version supported (the current Steam and GOG release).
const NV_SUPPORTED: &str = "1.4.0.525";
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
const FO3_DLC: [&str; 5] = [
    "Anchorage.esm",
    "ThePitt.esm",
    "BrokenSteel.esm",
    "PointLookout.esm",
    "Zeta.esm",
];
const STEAM_FOLDERS: [&str; 3] = ["Fallout New Vegas", "Fallout 3 goty", "Fallout 3"];
const GOG_FOLDERS: [&str; 3] = ["Fallout New Vegas", "Fallout 3 GOTY", "Fallout 3"];

impl SearchRoots {
    /// Roots for `platform` under `home`, with no Windows system folders and no extras.
    #[must_use]
    pub fn new(platform: Platform, home: &Path) -> SearchRoots {
        SearchRoots {
            platform,
            home: home.to_path_buf(),
            program_files_x86: None,
            program_data: None,
            extra: Vec::new(),
        }
    }

    /// Roots for the machine this runs on, from the environment.
    #[must_use]
    pub fn for_this_machine() -> SearchRoots {
        let env = |k| std::env::var_os(k).map(PathBuf::from);
        let platform = if cfg!(windows) {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        };
        let home = env("HOME")
            .or_else(|| env("USERPROFILE"))
            .unwrap_or_default();
        let mut roots = SearchRoots::new(platform, &home);
        roots.program_files_x86 = env("ProgramFiles(x86)");
        roots.program_data = env("ProgramData");
        roots
    }

    fn steam_roots(&self) -> Vec<PathBuf> {
        let h = &self.home;
        match self.platform {
            Platform::Linux => vec![
                h.join(".local/share/Steam"),
                h.join(".steam/steam"),
                h.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            ],
            Platform::MacOs => vec![h.join("Library/Application Support/Steam")],
            Platform::Windows => self
                .program_files_x86
                .iter()
                .map(|p| p.join("Steam"))
                .collect(),
        }
    }

    fn plain_folders(&self) -> Vec<PathBuf> {
        let mut parents = vec![
            self.home.join("Games"),
            self.home.join("Games/Heroic"),
            self.home.join("GOG Games"),
        ];
        if let Some(pf) = &self.program_files_x86 {
            parents.push(pf.join("GOG Galaxy/Games"));
        }
        if self.platform == Platform::Windows {
            parents.push(PathBuf::from(r"C:\GOG Games"));
        }
        let mut out: Vec<PathBuf> = parents
            .iter()
            .flat_map(|p| GOG_FOLDERS.iter().map(move |f| p.join(f)))
            .collect();
        out.extend(self.extra.iter().cloned());
        out
    }
}

/// Every game copy found under `roots`, each folder once, in a stable order.
#[must_use]
pub fn detect(roots: &SearchRoots) -> Vec<GameCopy> {
    let mut libraries: Vec<PathBuf> = roots.steam_roots();
    for root in roots.steam_roots() {
        libraries.extend(steam_libraries(&root));
    }
    libraries.extend(roots.extra.iter().cloned());
    let mut candidates: Vec<PathBuf> = libraries
        .iter()
        .flat_map(|lib| {
            STEAM_FOLDERS
                .iter()
                .map(move |n| lib.join("steamapps/common").join(n))
        })
        .collect();
    candidates.extend(roots.plain_folders());
    if let Some(data) = &roots.program_data {
        candidates.extend(epic_locations(
            &data.join("Epic/EpicGamesLauncher/Data/Manifests"),
        ));
    }
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter_map(|c| {
            let key = std::fs::canonicalize(&c).ok()?;
            if seen.insert(key) { inspect(&c) } else { None }
        })
        .collect()
}

/// The game copy in `dir`, when `dir` holds a game executable.
#[must_use]
pub fn inspect(dir: &Path) -> Option<GameCopy> {
    let (game, exe, dlc) = match find_ci(dir, NV_EXE) {
        Some(exe) => (Game::NewVegas, exe, &NV_DLC[..]),
        None => (Game::Fallout3, find_ci(dir, FO3_EXE)?, &FO3_DLC[..]),
    };
    let version = nvmod_pe::inspect_file(&exe)
        .ok()
        .and_then(|r| r.file_version)
        .map(|v| v.to_string());
    let supported_version = match game {
        Game::NewVegas => version.as_deref().map(|v| v == NV_SUPPORTED),
        Game::Fallout3 => None,
    };
    let data = find_ci(dir, "Data");
    let missing_dlc = dlc
        .iter()
        .filter(|f| data.as_deref().and_then(|d| find_ci(d, f)).is_none())
        .map(|f| (*f).to_string())
        .collect();
    Some(GameCopy {
        game,
        path: dir.to_path_buf(),
        storefront: detect_storefront(dir).label().to_string(),
        version,
        supported_version,
        missing_dlc,
    })
}

/// The library folders in a Steam root's `steamapps/libraryfolders.vdf`.
fn steam_libraries(root: &Path) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let mut quoted = line.split('"').skip(1).step_by(2);
            if quoted.next()? == "path" {
                quoted.next()
            } else {
                None
            }
        })
        .map(|p| PathBuf::from(p.replace(r"\\", r"\")))
        .collect()
}

/// The `InstallLocation` of every Epic manifest in `dir`.
fn epic_locations(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "item"))
        .filter_map(|e| std::fs::read(e.path()).ok())
        .filter_map(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .filter_map(|v| v["InstallLocation"].as_str().map(PathBuf::from))
        .collect()
}
