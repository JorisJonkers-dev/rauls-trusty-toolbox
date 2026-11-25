// SPDX-License-Identifier: GPL-3.0-only
//! The LOD settings of a Profile as a list of options: what `raul lod settings show|set|unset|reset|undo` and the
//! GUI's LOD settings view read and change. The file is `raul-lod.toml` in the Profile's folder.
//!
//! The file holds only what was set: a key that is absent takes the current default, so a better default reaches
//! every Profile that did not change it, and the request digest stays the one of a Profile with no file.
//! [`reset`] ("Restore defaults") removes the keys of LOD generation settings and never writes a default value; keys
//! that are not generation settings (`allow_in_repo`) stay. It keeps the file as it was in one `.previous` copy,
//! which [`undo`] puts back; the next [`set`] or [`unset`] drops that copy.
//!
//! Every option is a row of [`OPTIONS`]: its key, tier, kind, range and the catalogue strings that name and explain
//! it. Its default comes from [`default_value`], which asks the generator's own table of defaults.
//!
//! Every change is checked before it is saved, by the option's range and then by the generator's own
//! `Request::validate` on the request the settings give; a refused change leaves the file as it was.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use lodwright_contract::Request as LodwrightRequest;
use nvmod_instance::NvmodInstance;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use toml::Table;

use super::{SETTINGS_FILE, Settings};

/// The Contract name of the document `raul lod settings ... --json` prints.
pub const CONTRACT: &str = "raul-lod-settings";

/// The version of that document this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// The suffix of the copy [`reset`] keeps: `raul-lod.toml.previous`.
pub const PREVIOUS_SUFFIX: &str = ".previous";

/// The terrain levels, in the order the request lists them.
pub const LEVELS: [u32; 4] = [4, 8, 16, 32];

/// The message catalogue of the LOD settings: every string a view shows for them. Views show these as they are and
/// never build a sentence from parts; values and keys travel beside them as data.
pub mod catalogue {
    /// The detail line of every refusal: a refused change saves nothing.
    pub const NOTHING_CHANGED: &str = "Nothing changed.";
    /// The Instance cannot be opened.
    pub const INSTANCE_INVALID: &str = "The Instance cannot be opened.";
    /// The Profile does not exist.
    pub const PROFILE_MISSING: &str = "The Instance has no such Profile.";
    /// The file is not TOML, or cannot be read.
    pub const UNREADABLE: &str = "The LOD settings file of this Profile cannot be read.";
    /// A key the options do not have.
    pub const UNKNOWN_KEY: &str = "The LOD settings hold a setting this version does not know.";
    /// A value of the wrong kind, such as text for a number.
    pub const WRONG_TYPE: &str = "A LOD setting holds the wrong kind of value.";
    /// A value outside the option's range.
    pub const OUT_OF_RANGE: &str = "A LOD setting is outside the values it takes.";
    /// A texture size that is not a power of two.
    pub const NOT_A_POWER_OF_TWO: &str = "A texture size must be a power of two.";
    /// A record that is not eight hex digits.
    pub const NOT_A_RECORD: &str = "A record to leave out must be eight hex digits.";
    /// An empty name in a list.
    pub const EMPTY_NAME: &str = "A name in a LOD setting is empty.";
    /// The generator refused the settings for a reason the options do not check themselves.
    pub const REJECTED: &str = "The LOD generator does not accept these settings.";
    /// [`super::undo`] without a restore point.
    pub const NOTHING_TO_UNDO: &str = "There is no restore point to go back to.";
    /// The file could not be written.
    pub const WRITE_FAILED: &str = "The LOD settings could not be saved.";

    /// A world no plugin of the Profile defines.
    pub const UNKNOWN_WORLD: &str =
        "No active plugin of this Profile defines this world, so nothing is built for it.";
    /// The worlds could not be read from the plugins.
    pub const WORLDS_UNCHECKED: &str =
        "The worlds could not be checked against the plugins of this Profile.";
    /// An inclusion rule names a plugin that is not active.
    pub const PLUGIN_NOT_ACTIVE: &str =
        "This plugin is not active in this Profile, so building LOD stops with an error.";
    /// The inclusion rules leave no plugin for object LOD.
    pub const NO_OBJECT_LEFT: &str =
        "Every plugin in Only these plugins is also left out, so no object enters object LOD.";
    /// A LOD run of the Profile is going while the settings are read or changed.
    pub const RUN_IN_PROGRESS: &str = "A run is in progress. Changes apply to the next run.";

    /// Worlds.
    pub const WORLDS: &str = "Worlds";
    /// Worlds, explained.
    pub const WORLDS_HELP: &str = "The worlds to build, by editor id. Empty builds every world with land. A name that matches no world builds nothing.";
    /// Leave out plugins.
    pub const EXCLUDE_PLUGINS: &str = "Leave out plugins";
    /// Leave out plugins, explained.
    pub const EXCLUDE_PLUGINS_HELP: &str = "Objects from these plugins stay out of object LOD.";
    /// Only plugins.
    pub const ONLY_PLUGINS: &str = "Only these plugins";
    /// Only plugins, explained.
    pub const ONLY_PLUGINS_HELP: &str =
        "When not empty, only objects from these plugins enter object LOD.";
    /// Leave out records.
    pub const EXCLUDE_RECORDS: &str = "Leave out records";
    /// Leave out records, explained.
    pub const EXCLUDE_RECORDS_HELP: &str =
        "These records stay out of object LOD, each as eight hex digits.";
    /// Leave out models.
    pub const EXCLUDE_MODELS: &str = "Leave out models";
    /// Leave out models, explained.
    pub const EXCLUDE_MODELS_HELP: &str = "Models whose path under meshes matches one of these patterns stay out of object LOD; a star matches any run of characters. A broad pattern can leave out everything.";
    /// Terrain.
    pub const TERRAIN: &str = "Terrain";
    /// Terrain, explained.
    pub const TERRAIN_HELP: &str = "Build distant land.";
    /// Land textures.
    pub const LAND_TEXTURES: &str = "Land textures";
    /// Land textures, explained.
    pub const LAND_TEXTURES_HELP: &str =
        "Blend the land textures into distant land. Off uses plain colours, which is faster.";
    /// Water.
    pub const WATER: &str = "Water";
    /// Water, explained.
    pub const WATER_HELP: &str = "Build water surfaces into distant land.";
    /// Objects.
    pub const OBJECTS: &str = "Objects";
    /// Objects, explained.
    pub const OBJECTS_HELP: &str = "Build distant buildings and other large objects.";
    /// Trees.
    pub const TREES: &str = "Trees";
    /// Trees, explained.
    pub const TREES_HELP: &str = "Build distant trees.";
    /// Workers.
    pub const THREADS: &str = "Workers";
    /// Workers, explained.
    pub const THREADS_HELP: &str = "How many parts are built at once. Unset lets the machine decide. It never changes the result.";
    /// Memory budget.
    pub const MEMORY: &str = "Memory budget (MiB)";
    /// Memory budget, explained.
    pub const MEMORY_HELP: &str = "An estimate checked before the build starts: a build expected to need more is refused. Measured on Linux, best effort elsewhere.";
    /// Allow a git folder.
    pub const ALLOW_IN_REPO: &str = "Allow output in a git folder";
    /// Allow a git folder, explained.
    pub const ALLOW_IN_REPO_HELP: &str = "Lets the LOD be written when the Instance lies inside a git folder. Off by default, and Restore defaults leaves it as it is.";
    /// Quality.
    pub const QUALITY: &str = "Quality";
    /// Quality, explained.
    pub const QUALITY_HELP: &str = "How far distant land may stray from the real height, in half units. Lower is closer and heavier.";
    /// Vertex cap.
    pub const MAX_VERTICES: &str = "Vertex cap";
    /// Vertex cap, explained.
    pub const MAX_VERTICES_HELP: &str =
        "The most points one block of distant land may have. Too low leaves it coarse.";
    /// Diffuse size.
    pub const DIFFUSE_SIZE: &str = "Colour texture size";
    /// Diffuse size, explained.
    pub const DIFFUSE_SIZE_HELP: &str =
        "The edge of each block's colour texture in pixels. Larger costs disk space and memory.";
    /// Normal size.
    pub const NORMAL_SIZE: &str = "Shading texture size";
    /// Normal size, explained.
    pub const NORMAL_SIZE_HELP: &str =
        "The edge of each block's shading texture in pixels. Larger costs disk space and memory.";
    /// Specular in the alpha of the shading texture.
    pub const SPECULAR_ALPHA: &str = "Shine in the shading texture";
    /// Specular in alpha, explained.
    pub const SPECULAR_ALPHA_HELP: &str = "Stores how shiny each land texture is in the shading texture, so distant land can shine like the land near you. On by default.";
    /// Skirts at the borders of a level's blocks.
    pub const SKIRTS: &str = "Close the gaps between blocks";
    /// Skirts, explained.
    pub const SKIRTS_HELP: &str = "Hangs a narrow wall under the edge of every block, as the game's own blocks do, so no gap shows between neighbouring blocks. Off by default; it is new, so look at the result in the game.";
    /// Drop terrain faces under the water.
    pub const TERRAIN_DROP_UNDER_WATER: &str = "Leave out land faces deep under the water";
    /// Drop terrain faces under the water, explained.
    pub const TERRAIN_DROP_UNDER_WATER_HELP: &str = "Distant land loses the faces that lie at least this far below the water, in game units, which nobody sees through it. The files get smaller. Empty keeps every face.";
    /// How often a land texture repeats.
    pub const TEXTURE_SCALES: &str = "How often a land texture repeats";
    /// Texture scales, explained.
    pub const TEXTURE_SCALES_HELP: &str = "For a land texture, how many times it repeats across a quarter of a cell in the colour textures. Give the land texture's record id and the number, like 0012AB34=2, separated by commas. A texture that is not listed repeats once.";
    /// Walls at the shore of the water planes.
    pub const WATER_WALLS: &str = "Water walls";
    /// Water walls, explained.
    pub const WATER_WALLS_HELP: &str = "On by default. Writes a vertical wall along the edge of distant water, so no gap shows between the water and the land beside it.";
    /// Shared water vertices.
    pub const WATER_SHARED_VERTICES: &str = "Shared water corners (experimental)";
    /// Shared water vertices, explained.
    pub const WATER_SHARED_VERTICES_HELP: &str = "Experimental, off by default. Lets neighbouring water faces share their corners. It is new and not proven in the game: turn it on only to look at the result.";
    /// Water polygons.
    pub const WATER_POLYGONS: &str = "Water polygons (experimental)";
    /// Water polygons, explained.
    pub const WATER_POLYGONS_HELP: &str = "Experimental, off by default. Builds distant water from polygons that follow the water's shape. It is new and not proven in the game: turn it on only to look at the result.";
    /// Pack the generated LOD into one archive.
    pub const PACK: &str = "Pack the generated LOD into one archive";
    /// Packing, explained.
    pub const PACK_HELP: &str = "On by default. The next generate ends by writing the LOD into one archive (a .bsa file) and a small Plugin of the same name that registers it, in place of the loose files; the LOD Mod then holds the archive, the Plugin, an empty .override file and no loose files. The .override file lets the archive replace the same paths in earlier archives; on the original game that needs the JIP LN plugin, and without it turn packing off if the far view shows the older files. The Plugin takes one of the game's 255 plugin slots and stays on in the Load order, or the game does not open the archive and uses its own distant land; it needs the game's first Plugin (normally FalloutNV.esm) to be present. Turning this off applies at the next generate, which builds the loose files again.";
    /// Drop faces under the terrain.
    pub const DROP_UNDER_TERRAIN: &str = "Leave out object faces under the land";
    /// Drop faces under the terrain, explained.
    pub const DROP_UNDER_TERRAIN_HELP: &str = "Distant objects lose the faces that lie below the land, which nobody sees. The files get smaller. A face that was meant to show through the land goes too.";
    /// Drop faces under the water.
    pub const DROP_UNDER_WATER: &str = "Leave out object faces under the water";
    /// Drop faces under the water, explained.
    pub const DROP_UNDER_WATER_HELP: &str = "Distant objects lose the faces that lie below the water. The files get smaller. A face seen through clear water goes too.";
}

/// What kind of value an option holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// On or off.
    Switch,
    /// A whole number.
    Count,
    /// A number that may have a fraction.
    Number,
    /// A texture edge in pixels: a power of two.
    TextureSize,
    /// A list of names or patterns.
    Names,
    /// A list of record ids, eight hex digits each.
    Records,
    /// A table of record id (eight hex digits) to a number, written `id=number`.
    Scales,
}

/// One option of the LOD settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Descriptor {
    /// Its key in `raul-lod.toml`, with dots between tables: `output.trees`, `levels.4.quality`.
    pub key: &'static str,
    /// 1: shown at once; 2: advanced, behind the disclosure (per-level terrain, the git folder).
    pub tier: u8,
    /// What kind of value it holds.
    pub kind: Kind,
    /// The smallest value it takes, for numbers.
    pub min: Option<f64>,
    /// The largest value it takes, for numbers.
    pub max: Option<f64>,
    /// Whether it is a LOD generation setting, which Restore defaults removes.
    pub restores: bool,
    /// Its name, from the catalogue.
    pub label: &'static str,
    /// What it does, from the catalogue.
    pub help: &'static str,
}

const fn row(
    key: &'static str,
    tier: u8,
    kind: Kind,
    range: (Option<f64>, Option<f64>),
    label: &'static str,
    help: &'static str,
) -> Descriptor {
    Descriptor {
        key,
        tier,
        kind,
        min: range.0,
        max: range.1,
        restores: true,
        label,
        help,
    }
}

const NONE: (Option<f64>, Option<f64>) = (None, None);
const QUALITY: (Option<f64>, Option<f64>) = (Some(0.0), None);
const VERTICES: (Option<f64>, Option<f64>) = (Some(3.0), Some(32767.0));
const SIZE: (Option<f64>, Option<f64>) = (Some(4.0), Some(4096.0));

macro_rules! level_rows {
    ($($l:literal),*) => {
        [$(
            row(concat!("levels.", $l, ".quality"), 2, Kind::Number, QUALITY, catalogue::QUALITY, catalogue::QUALITY_HELP),
            row(concat!("levels.", $l, ".max_vertices"), 2, Kind::Count, VERTICES, catalogue::MAX_VERTICES, catalogue::MAX_VERTICES_HELP),
            row(concat!("levels.", $l, ".diffuse_size"), 2, Kind::TextureSize, SIZE, catalogue::DIFFUSE_SIZE, catalogue::DIFFUSE_SIZE_HELP),
            row(concat!("levels.", $l, ".normal_size"), 2, Kind::TextureSize, SIZE, catalogue::NORMAL_SIZE, catalogue::NORMAL_SIZE_HELP),
            row(concat!("levels.", $l, ".specular_alpha"), 3, Kind::Switch, NONE, catalogue::SPECULAR_ALPHA, catalogue::SPECULAR_ALPHA_HELP),
            row(concat!("levels.", $l, ".skirts"), 3, Kind::Switch, NONE, catalogue::SKIRTS, catalogue::SKIRTS_HELP),
        )*]
    };
}

const LEVEL_ROWS: [Descriptor; 24] = level_rows!("4", "8", "16", "32");

const BASE_ROWS: [Descriptor; 21] = [
    row(
        "worlds",
        1,
        Kind::Names,
        NONE,
        catalogue::WORLDS,
        catalogue::WORLDS_HELP,
    ),
    row(
        "output.terrain",
        1,
        Kind::Switch,
        NONE,
        catalogue::TERRAIN,
        catalogue::TERRAIN_HELP,
    ),
    row(
        "output.land_textures",
        1,
        Kind::Switch,
        NONE,
        catalogue::LAND_TEXTURES,
        catalogue::LAND_TEXTURES_HELP,
    ),
    row(
        "output.water",
        1,
        Kind::Switch,
        NONE,
        catalogue::WATER,
        catalogue::WATER_HELP,
    ),
    row(
        "output.objects",
        1,
        Kind::Switch,
        NONE,
        catalogue::OBJECTS,
        catalogue::OBJECTS_HELP,
    ),
    row(
        "output.trees",
        1,
        Kind::Switch,
        NONE,
        catalogue::TREES,
        catalogue::TREES_HELP,
    ),
    row(
        "inclusion.exclude_plugins",
        1,
        Kind::Names,
        NONE,
        catalogue::EXCLUDE_PLUGINS,
        catalogue::EXCLUDE_PLUGINS_HELP,
    ),
    row(
        "inclusion.only_plugins",
        1,
        Kind::Names,
        NONE,
        catalogue::ONLY_PLUGINS,
        catalogue::ONLY_PLUGINS_HELP,
    ),
    row(
        "inclusion.exclude_records",
        1,
        Kind::Records,
        NONE,
        catalogue::EXCLUDE_RECORDS,
        catalogue::EXCLUDE_RECORDS_HELP,
    ),
    row(
        "inclusion.exclude_models",
        1,
        Kind::Names,
        NONE,
        catalogue::EXCLUDE_MODELS,
        catalogue::EXCLUDE_MODELS_HELP,
    ),
    row(
        "threads",
        1,
        Kind::Count,
        (Some(1.0), Some(4_294_967_295.0)),
        catalogue::THREADS,
        catalogue::THREADS_HELP,
    ),
    row(
        "memory_mib",
        1,
        Kind::Count,
        (Some(1.0), None),
        catalogue::MEMORY,
        catalogue::MEMORY_HELP,
    ),
    Descriptor {
        restores: false,
        ..row(
            "allow_in_repo",
            2,
            Kind::Switch,
            NONE,
            catalogue::ALLOW_IN_REPO,
            catalogue::ALLOW_IN_REPO_HELP,
        )
    },
    row(
        "objects.drop_under_terrain",
        3,
        Kind::Switch,
        NONE,
        catalogue::DROP_UNDER_TERRAIN,
        catalogue::DROP_UNDER_TERRAIN_HELP,
    ),
    row(
        "objects.drop_under_water",
        3,
        Kind::Switch,
        NONE,
        catalogue::DROP_UNDER_WATER,
        catalogue::DROP_UNDER_WATER_HELP,
    ),
    row(
        "terrain.drop_under_water",
        3,
        Kind::Number,
        (Some(0.0), Some(100_000.0)),
        catalogue::TERRAIN_DROP_UNDER_WATER,
        catalogue::TERRAIN_DROP_UNDER_WATER_HELP,
    ),
    row(
        "terrain.water_walls",
        3,
        Kind::Switch,
        NONE,
        catalogue::WATER_WALLS,
        catalogue::WATER_WALLS_HELP,
    ),
    row(
        "terrain.water_shared_vertices",
        3,
        Kind::Switch,
        NONE,
        catalogue::WATER_SHARED_VERTICES,
        catalogue::WATER_SHARED_VERTICES_HELP,
    ),
    row(
        "terrain.water_polygons",
        3,
        Kind::Switch,
        NONE,
        catalogue::WATER_POLYGONS,
        catalogue::WATER_POLYGONS_HELP,
    ),
    row(
        "terrain.texture_scales",
        3,
        Kind::Scales,
        (Some(0.0625), Some(16.0)),
        catalogue::TEXTURE_SCALES,
        catalogue::TEXTURE_SCALES_HELP,
    ),
    row(
        "output.pack",
        3,
        Kind::Switch,
        NONE,
        catalogue::PACK,
        catalogue::PACK_HELP,
    ),
];

/// Every option, in the order a view lists them: tier 1, then the advanced ones.
pub const OPTIONS: [Descriptor; 45] = {
    let mut all = [BASE_ROWS[0]; 45];
    let mut i = 0;
    while i < BASE_ROWS.len() {
        all[i] = BASE_ROWS[i];
        i += 1;
    }
    let mut j = 0;
    while j < LEVEL_ROWS.len() {
        all[i + j] = LEVEL_ROWS[j];
        j += 1;
    }
    all
};

/// The option `key` names.
#[must_use]
pub fn descriptor(key: &str) -> Option<&'static Descriptor> {
    OPTIONS.iter().find(|d| d.key == key)
}

/// The generator's own table of defaults, ranges and tiers (`lodwright_api::request_defaults`), read once.
fn generator() -> &'static lodwright_contract::Defaults {
    static TABLE: std::sync::OnceLock<lodwright_contract::Defaults> = std::sync::OnceLock::new();
    TABLE.get_or_init(lodwright_api::request_defaults)
}

/// Where an option sits in the generator's request: the dotted key its table uses.
fn generator_key(key: &str) -> Option<String> {
    Some(match key {
        "worlds"
        | "objects.drop_under_terrain"
        | "objects.drop_under_water"
        | "terrain.drop_under_water"
        | "terrain.water_walls"
        | "terrain.water_shared_vertices"
        | "terrain.water_polygons"
        | "terrain.texture_scales"
        | "output.pack"
        | "inclusion.exclude_plugins"
        | "inclusion.only_plugins"
        | "inclusion.exclude_records"
        | "inclusion.exclude_models" => key.to_owned(),
        "output.terrain" => "terrain.enabled".to_owned(),
        "output.land_textures" => "terrain.land_textures".to_owned(),
        "output.water" => "terrain.water".to_owned(),
        "output.objects" => "objects.enabled".to_owned(),
        "output.trees" => "trees.enabled".to_owned(),
        "threads" => "limits.threads".to_owned(),
        "memory_mib" => "limits.memory_mib".to_owned(),
        _ => {
            let rest = key.strip_prefix("levels.")?;
            let (_, field) = rest.split_once('.')?;
            format!("terrain.levels.{field}")
        }
    })
}

/// Whether the generator's table lists `d`. An option the pinned generator does not list is absent from every view
/// and cannot be set.
#[must_use]
pub fn listed_in(table: &lodwright_contract::Defaults, d: &Descriptor) -> bool {
    d.key == "allow_in_repo" || generator_key(d.key).is_some_and(|k| table.option(&k).is_some())
}

/// The release of the LOD generator whose defaults [`default_value`] reads.
#[must_use]
pub fn generator_version() -> &'static str {
    &generator().version
}

/// The value an option takes when the file does not set it. `null` means the generator decides (workers) or there is
/// none (memory budget).
///
/// This is the generator's own answer (`request_defaults`), so a better default reaches every Profile that did not
/// change it with the next bump of the generator; the Toolbox holds no copy. The one setting the generator does not
/// know, `allow_in_repo`, is off.
#[must_use]
pub fn default_value(key: &str) -> Option<Value> {
    let d = descriptor(key)?;
    if key == "allow_in_repo" {
        return Some(json!(false));
    }
    // raul's own default: the generator leaves packing off, raul packs.
    if key == "output.pack" {
        return Some(json!(true));
    }
    let info = generator().option(&generator_key(key)?)?;
    if let Some(rest) = key.strip_prefix("levels.") {
        let (level, _) = rest.split_once('.')?;
        let level: u32 = level.parse().ok()?;
        return info
            .per_level
            .iter()
            .find(|l| l.level == level)
            .map(|l| l.value.clone());
    }
    Some(match d.kind {
        Kind::Names | Kind::Records if info.default.is_null() => json!([]),
        Kind::Scales if info.default.is_null() => json!({}),
        _ => info.default.clone(),
    })
}

/// Why a reading or a change of the settings was refused. `what` is a catalogue string and `detail` is
/// [`catalogue::NOTHING_CHANGED`]; `key` and `reason` are data beside them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Refusal {
    /// The stable code callers match on.
    pub code: RefusalCode,
    /// The option or file key it is about, when there is one.
    pub key: Option<String>,
    /// What went wrong, from the catalogue.
    pub what: String,
    /// What was not changed, from the catalogue.
    pub detail: String,
    /// A further reason for people (a parser's or the generator's own words), never matched on.
    pub reason: Option<String>,
}

/// The stable codes of a [`Refusal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefusalCode {
    /// The Instance cannot be opened (exit code 2).
    InstanceInvalid,
    /// The Profile does not exist (exit code 2).
    ProfileMissing,
    /// The file cannot be read or is not TOML (exit code 2).
    Unreadable,
    /// A key that is not an option (exit code 2).
    UnknownKey,
    /// A value of the wrong kind (exit code 2).
    WrongType,
    /// A value outside its option's range (exit code 2).
    OutOfRange,
    /// A texture size that is not a power of two (exit code 2).
    NotAPowerOfTwo,
    /// A record id that is not eight hex digits (exit code 2).
    NotARecord,
    /// An empty name in a list (exit code 2).
    EmptyName,
    /// The generator's own check refused the settings (exit code 2).
    Rejected,
    /// Undo without a restore point (exit code 2).
    NothingToUndo,
    /// The file could not be written (exit code 4).
    WriteFailed,
}

impl RefusalCode {
    /// The catalogue string of this code.
    #[must_use]
    pub fn what(self) -> &'static str {
        match self {
            Self::InstanceInvalid => catalogue::INSTANCE_INVALID,
            Self::ProfileMissing => catalogue::PROFILE_MISSING,
            Self::Unreadable => catalogue::UNREADABLE,
            Self::UnknownKey => catalogue::UNKNOWN_KEY,
            Self::WrongType => catalogue::WRONG_TYPE,
            Self::OutOfRange => catalogue::OUT_OF_RANGE,
            Self::NotAPowerOfTwo => catalogue::NOT_A_POWER_OF_TWO,
            Self::NotARecord => catalogue::NOT_A_RECORD,
            Self::EmptyName => catalogue::EMPTY_NAME,
            Self::Rejected => catalogue::REJECTED,
            Self::NothingToUndo => catalogue::NOTHING_TO_UNDO,
            Self::WriteFailed => catalogue::WRITE_FAILED,
        }
    }
}

impl Refusal {
    fn new(code: RefusalCode, key: Option<&str>, reason: Option<String>) -> Refusal {
        Refusal {
            code,
            key: key.map(str::to_owned),
            what: code.what().to_owned(),
            detail: catalogue::NOTHING_CHANGED.to_owned(),
            reason,
        }
    }

    /// The refusal as one line for a terminal: what, the key and reason when there are, and the detail.
    #[must_use]
    pub fn line(&self) -> String {
        let mut line = self.what.clone();
        if let Some(k) = &self.key {
            let _ = write!(line, " ({k})");
        }
        if let Some(r) = &self.reason {
            let _ = write!(line, " {r}.");
        }
        line.push(' ');
        line.push_str(&self.detail);
        line
    }
}

/// Something the settings allow but that quietly builds less than asked: a warning, never a refusal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Warning {
    /// The stable code callers match on.
    pub code: WarningCode,
    /// The option it is about.
    pub key: String,
    /// The value it is about (a world or plugin name), when there is one.
    pub value: Option<String>,
    /// What it means, from the catalogue.
    pub what: String,
    /// A further reason for people, never matched on.
    pub reason: Option<String>,
}

/// The stable codes of a [`Warning`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WarningCode {
    /// A world in `worlds` that no active plugin defines.
    UnknownWorld,
    /// The plugins could not be read to check `worlds`.
    WorldsUnchecked,
    /// A plugin in an inclusion rule that is not active in the Profile.
    PluginNotActive,
    /// Every plugin in `only_plugins` is also in `exclude_plugins`: no object is left.
    NoObjectLeft,
}

/// The warnings of `settings` for a Profile whose active plugins are `plugins` (`None` when they cannot be read)
/// and whose plugins define `worlds` (`Err` with the reason when they cannot be read). Names match without regard to
/// case, as the generator matches them. An object enters object LOD when neither its reference's nor its base's
/// plugin is in `exclude_plugins` and, when `only_plugins` is not empty, one of them is in `only_plugins`.
#[must_use]
pub fn warnings(
    settings: &Settings,
    plugins: Option<&[String]>,
    worlds: Result<&[String], &str>,
) -> Vec<Warning> {
    let warn = |code: WarningCode, key: &str, value: Option<&str>, reason: Option<&str>| Warning {
        code,
        key: key.to_owned(),
        value: value.map(str::to_owned),
        what: match code {
            WarningCode::UnknownWorld => catalogue::UNKNOWN_WORLD,
            WarningCode::WorldsUnchecked => catalogue::WORLDS_UNCHECKED,
            WarningCode::PluginNotActive => catalogue::PLUGIN_NOT_ACTIVE,
            WarningCode::NoObjectLeft => catalogue::NO_OBJECT_LEFT,
        }
        .to_owned(),
        reason: reason.map(str::to_owned),
    };
    let same = |a: &String, b: &String| a.eq_ignore_ascii_case(b);
    let mut out = Vec::new();
    if !settings.worlds.is_empty() {
        match worlds {
            Ok(known) => {
                for w in &settings.worlds {
                    if !known.iter().any(|k| same(k, w)) {
                        out.push(warn(WarningCode::UnknownWorld, "worlds", Some(w), None));
                    }
                }
            }
            Err(why) => out.push(warn(
                WarningCode::WorldsUnchecked,
                "worlds",
                None,
                Some(why),
            )),
        }
    }
    let i = &settings.inclusion;
    if let Some(active) = plugins {
        for (key, list) in [
            ("inclusion.exclude_plugins", &i.exclude_plugins),
            ("inclusion.only_plugins", &i.only_plugins),
        ] {
            for p in list.iter().filter(|p| !active.iter().any(|a| same(a, p))) {
                out.push(warn(WarningCode::PluginNotActive, key, Some(p), None));
            }
        }
    }
    if !i.only_plugins.is_empty()
        && i.only_plugins
            .iter()
            .all(|o| i.exclude_plugins.iter().any(|e| same(e, o)))
    {
        out.push(warn(
            WarningCode::NoObjectLeft,
            "inclusion.only_plugins",
            None,
            None,
        ));
    }
    out
}

/// One option with the value a Profile gives it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OptionState {
    /// Its key.
    pub key: String,
    /// Its tier: 1 or 2.
    pub tier: u8,
    /// What kind of value it holds.
    pub kind: Kind,
    /// The value it takes: the file's when the file sets it, else the default.
    pub value: Value,
    /// The value it takes when the file does not set it; `null` when the generator decides.
    pub default: Value,
    /// The file sets it, even to its default value.
    pub overridden: bool,
    /// Restore defaults removes it.
    pub restores: bool,
    /// The smallest value it takes, for numbers.
    pub min: Option<f64>,
    /// The largest value it takes, for numbers.
    pub max: Option<f64>,
}

/// What a change did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Change {
    /// The verb: `set`, `unset`, `reset` or `undo`.
    pub action: ChangeAction,
    /// The keys whose value changed (for `reset`, the keys it removes), in the order of [`OPTIONS`].
    pub keys: Vec<String>,
    /// Whether the file changed. `false` for a dry run and for a change that changes nothing.
    pub changed: bool,
    /// A dry run: what `reset` would remove; nothing was written.
    pub dry_run: bool,
    /// The settings file no longer exists after the change (nothing was left in it).
    pub file_removed: bool,
    /// The restore point after the change: the `.previous` copy, or `null` when there is none.
    pub restore_point: Option<PathBuf>,
}

/// The verbs that change the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeAction {
    /// One option set.
    Set,
    /// One key removed.
    Unset,
    /// Restore defaults.
    Reset,
    /// Back to the restore point.
    Undo,
}

/// The document `raul lod settings ... --json` prints (Contract `raul-lod-settings`, version 1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SettingsDoc {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-lod-settings`.
    #[schemars(extend("const" = "raul-lod-settings"))]
    pub contract: String,
    /// The Instance.
    pub instance: PathBuf,
    /// The Profile, or empty when it could not be resolved.
    pub profile: String,
    /// The settings file, or `null` when the Profile could not be resolved.
    pub file: Option<PathBuf>,
    /// Whether the file exists.
    pub exists: bool,
    /// Every option with its value after the command (unchanged after a refusal); empty when the settings cannot be
    /// read.
    pub options: Vec<OptionState>,
    /// The restore point Undo goes back to, or `null`.
    pub restore_point: Option<PathBuf>,
    /// What the settings allow but would quietly build nothing or fail on: unknown worlds, inactive plugins in the
    /// inclusion rules, rules that leave no object. Never a refusal.
    pub warnings: Vec<Warning>,
    /// A LOD run of the Profile is going: it uses the settings it started with, and a change applies to the next run.
    #[serde(default)]
    pub run_in_progress: bool,
    /// What the command changed; `null` for `show` and for a refusal.
    pub change: Option<Change>,
    /// Set when the command was refused; nothing changed then.
    pub error: Option<Refusal>,
}

/// The JSON Schema of [`SettingsDoc`], as `contracts/raul-lod-settings.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::schema_text(&schemars::schema_for!(SettingsDoc))
}

/// The settings of one Profile: where they live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSettings {
    /// The Profile.
    pub profile: String,
    /// `raul-lod.toml` in the Profile's folder.
    pub file: PathBuf,
}

impl ProfileSettings {
    /// The settings of `profile` (the Instance's only or selected one when `None`).
    ///
    /// # Errors
    ///
    /// The Profile does not exist.
    pub fn of(instance: &NvmodInstance, profile: Option<&str>) -> Result<ProfileSettings, Refusal> {
        let profile = instance
            .resolve_profile(profile)
            .map_err(|e| Refusal::new(RefusalCode::ProfileMissing, None, Some(e.to_string())))?;
        let file = instance.profile_dir(&profile).join(SETTINGS_FILE);
        Ok(ProfileSettings { profile, file })
    }

    /// The restore point: `raul-lod.toml.previous` beside the file.
    #[must_use]
    pub fn previous(&self) -> PathBuf {
        let mut name = self.file.as_os_str().to_owned();
        name.push(PREVIOUS_SUFFIX);
        PathBuf::from(name)
    }

    fn restore_point(&self) -> Option<PathBuf> {
        let p = self.previous();
        p.is_file().then_some(p)
    }
}

/// Read `file` as a table: empty when it does not exist.
fn read_table(file: &Path) -> Result<(Table, bool), Refusal> {
    match std::fs::read_to_string(file) {
        Ok(text) => text.parse::<Table>().map(|t| (t, true)).map_err(|e| {
            Refusal::new(
                RefusalCode::Unreadable,
                None,
                Some(one_line(&e.to_string())),
            )
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok((Table::new(), false)),
        Err(e) => Err(Refusal::new(
            RefusalCode::Unreadable,
            None,
            Some(e.to_string()),
        )),
    }
}

fn one_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every key the table sets, as dotted paths, in the file's order. A table that holds nothing counts as a key.
fn leaves(table: &Table, prefix: &str, out: &mut Vec<(String, toml::Value)>) {
    for (k, v) in table {
        let path = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        match v {
            toml::Value::Table(t) if descriptor(&path).is_none() && !t.is_empty() => {
                leaves(t, &path, out);
            }
            _ => out.push((path, v.clone())),
        }
    }
}

fn get<'t>(table: &'t Table, key: &str) -> Option<&'t toml::Value> {
    let mut parts = key.split('.');
    let mut cur = table.get(parts.next()?)?;
    for p in parts {
        cur = cur.as_table()?.get(p)?;
    }
    Some(cur)
}

fn put(table: &mut Table, key: &str, value: toml::Value) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut cur = table;
    for p in &parts[..parts.len() - 1] {
        let entry = cur
            .entry((*p).to_owned())
            .or_insert_with(|| toml::Value::Table(Table::new()));
        if !entry.is_table() {
            *entry = toml::Value::Table(Table::new());
        }
        cur = entry.as_table_mut().expect("a table was just put here");
    }
    cur.insert(parts[parts.len() - 1].to_owned(), value);
}

/// Remove `key`; then remove each table above it that is left empty. Whether it was there.
fn remove(table: &mut Table, key: &str) -> bool {
    match key.split_once('.') {
        None => table.remove(key).is_some(),
        Some((head, rest)) => {
            let Some(toml::Value::Table(inner)) = table.get_mut(head) else {
                return false;
            };
            let removed = remove(inner, rest);
            if removed && inner.is_empty() {
                table.remove(head);
            }
            removed
        }
    }
}

fn number(v: &toml::Value) -> Option<f64> {
    match v {
        #[allow(clippy::cast_precision_loss)] // file values are far below 2^52
        toml::Value::Integer(i) => Some(*i as f64),
        toml::Value::Float(f) => Some(*f),
        _ => None,
    }
}

/// Check one value against its option.
fn check_value(d: &Descriptor, v: &toml::Value) -> Result<(), Refusal> {
    let refuse = |code| Err(Refusal::new(code, Some(d.key), None));
    match d.kind {
        Kind::Switch => {
            if !v.is_bool() {
                return refuse(RefusalCode::WrongType);
            }
        }
        Kind::Count | Kind::TextureSize | Kind::Number => {
            let whole = matches!(v, toml::Value::Integer(_));
            let Some(n) = number(v).filter(|_| whole || d.kind == Kind::Number) else {
                return refuse(RefusalCode::WrongType);
            };
            if !n.is_finite() || d.min.is_some_and(|m| n < m) || d.max.is_some_and(|m| n > m) {
                return refuse(RefusalCode::OutOfRange);
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            // in range, checked above
            if d.kind == Kind::TextureSize && !(n as u64).is_power_of_two() {
                return refuse(RefusalCode::NotAPowerOfTwo);
            }
        }
        Kind::Scales => {
            let Some(entries) = v.as_table() else {
                return refuse(RefusalCode::WrongType);
            };
            for (id, scale) in entries {
                if id.len() != 8 || u32::from_str_radix(id, 16).is_err() {
                    return refuse(RefusalCode::NotARecord);
                }
                let Some(n) = number(scale) else {
                    return refuse(RefusalCode::WrongType);
                };
                if !n.is_finite() || d.min.is_some_and(|m| n < m) || d.max.is_some_and(|m| n > m) {
                    return refuse(RefusalCode::OutOfRange);
                }
            }
        }
        Kind::Names | Kind::Records => {
            let Some(items) = v.as_array() else {
                return refuse(RefusalCode::WrongType);
            };
            for item in items {
                let Some(s) = item.as_str() else {
                    return refuse(RefusalCode::WrongType);
                };
                if s.trim().is_empty() {
                    return refuse(RefusalCode::EmptyName);
                }
                if d.kind == Kind::Records && (s.len() != 8 || u32::from_str_radix(s, 16).is_err())
                {
                    return refuse(RefusalCode::NotARecord);
                }
            }
        }
    }
    Ok(())
}

/// Check a whole table: every key an option, every value in range, then the generator's own check on the request
/// the settings give. Returns the typed settings.
///
/// # Errors
///
/// The first key or value that is refused.
pub fn check_table(table: &Table) -> Result<Settings, Refusal> {
    let mut all = Vec::new();
    leaves(table, "", &mut all);
    for (key, value) in &all {
        let Some(d) = descriptor(key).filter(|d| listed_in(generator(), d)) else {
            return Err(Refusal::new(RefusalCode::UnknownKey, Some(key), None));
        };
        check_value(d, value)?;
    }
    let settings: Settings = table.clone().try_into().map_err(|e: toml::de::Error| {
        Refusal::new(RefusalCode::WrongType, None, Some(one_line(&e.to_string())))
    })?;
    let mut request = LodwrightRequest::new("data", &["check.esm"], "out");
    let parts = super::request_parts(&settings);
    request.worlds.clone_from(&settings.worlds);
    request.terrain = serde_json::from_value(parts["terrain"].clone())
        .map_err(|e| Refusal::new(RefusalCode::Rejected, None, Some(e.to_string())))?;
    request.objects = serde_json::from_value(parts["objects"].clone())
        .map_err(|e| Refusal::new(RefusalCode::Rejected, None, Some(e.to_string())))?;
    request.inclusion = serde_json::from_value(parts["inclusion"].clone())
        .map_err(|e| Refusal::new(RefusalCode::Rejected, None, Some(e.to_string())))?;
    request.limits = serde_json::from_value(parts["limits"].clone())
        .map_err(|e| Refusal::new(RefusalCode::Rejected, None, Some(e.to_string())))?;
    request
        .validate()
        .map_err(|why| Refusal::new(RefusalCode::Rejected, None, Some(why)))?;
    Ok(settings)
}

fn to_json(v: &toml::Value) -> Value {
    match v {
        toml::Value::String(s) => json!(s),
        toml::Value::Integer(i) => json!(i),
        toml::Value::Float(f) => json!(f),
        toml::Value::Boolean(b) => json!(b),
        toml::Value::Array(a) => Value::Array(a.iter().map(to_json).collect()),
        toml::Value::Table(t) => {
            Value::Object(t.iter().map(|(k, v)| (k.clone(), to_json(v))).collect())
        }
        toml::Value::Datetime(d) => json!(d.to_string()),
    }
}

fn to_toml(d: &Descriptor, v: &Value) -> Result<toml::Value, Refusal> {
    let wrong = || Refusal::new(RefusalCode::WrongType, Some(d.key), None);
    Ok(match d.kind {
        Kind::Switch => toml::Value::Boolean(v.as_bool().ok_or_else(wrong)?),
        Kind::Count | Kind::TextureSize => {
            let n = v.as_u64().ok_or_else(|| {
                if v.is_number() {
                    Refusal::new(RefusalCode::OutOfRange, Some(d.key), None)
                } else {
                    wrong()
                }
            })?;
            toml::Value::Integer(
                i64::try_from(n)
                    .map_err(|_| Refusal::new(RefusalCode::OutOfRange, Some(d.key), None))?,
            )
        }
        Kind::Number => toml::Value::Float(v.as_f64().ok_or_else(wrong)?),
        Kind::Scales => {
            let mut table = Table::new();
            for (id, scale) in v.as_object().ok_or_else(wrong)? {
                table.insert(
                    id.clone(),
                    toml::Value::Float(scale.as_f64().ok_or_else(wrong)?),
                );
            }
            toml::Value::Table(table)
        }
        Kind::Names | Kind::Records => toml::Value::Array(
            v.as_array()
                .ok_or_else(wrong)?
                .iter()
                .map(|s| {
                    s.as_str()
                        .map(|s| toml::Value::String(s.to_owned()))
                        .ok_or_else(wrong)
                })
                .collect::<Result<_, _>>()?,
        ),
    })
}

/// The value of `key` from command-line words: `true`/`false`, a number, or the names of a list (none for an
/// empty list).
///
/// # Errors
///
/// The key is not an option, or the words are not a value of its kind.
pub fn parse_value(key: &str, words: &[String]) -> Result<Value, Refusal> {
    let d =
        descriptor(key).ok_or_else(|| Refusal::new(RefusalCode::UnknownKey, Some(key), None))?;
    let wrong = || Refusal::new(RefusalCode::WrongType, Some(d.key), None);
    let one = || match words {
        [w] => Ok(w.as_str()),
        _ => Err(wrong()),
    };
    match d.kind {
        Kind::Switch => match one()? {
            "true" | "on" => Ok(json!(true)),
            "false" | "off" => Ok(json!(false)),
            _ => Err(wrong()),
        },
        Kind::Count | Kind::TextureSize => {
            one()?.parse::<u64>().map(|n| json!(n)).map_err(|_| wrong())
        }
        Kind::Number => one()?
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(|n| json!(n))
            .ok_or_else(wrong),
        Kind::Names | Kind::Records => Ok(json!(words)),
        Kind::Scales => {
            let mut entries = serde_json::Map::new();
            for word in words {
                let (id, scale) = word.split_once('=').ok_or_else(wrong)?;
                let scale: f64 = scale
                    .trim()
                    .parse()
                    .ok()
                    .filter(|n: &f64| n.is_finite())
                    .ok_or_else(wrong)?;
                entries.insert(id.trim().to_ascii_uppercase(), json!(scale));
            }
            Ok(Value::Object(entries))
        }
    }
}

/// Every option with the value `table` gives it.
fn states(table: &Table) -> Vec<OptionState> {
    OPTIONS
        .iter()
        .filter(|d| listed_in(generator(), d))
        .map(|d| {
            let default = default_value(d.key).unwrap_or(Value::Null);
            let set = get(table, d.key);
            OptionState {
                key: d.key.to_owned(),
                tier: d.tier,
                kind: d.kind,
                value: set.map_or_else(|| default.clone(), to_json),
                default,
                overridden: set.is_some(),
                restores: d.restores,
                min: d.min,
                max: d.max,
            }
        })
        .collect()
}

/// What a Profile's settings are now.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    /// Whether the file exists.
    pub exists: bool,
    /// Every option with its value.
    pub options: Vec<OptionState>,
    /// The restore point, or `None`.
    pub restore_point: Option<PathBuf>,
}

/// The settings of `at`, checked.
///
/// # Errors
///
/// The file cannot be read, or holds a key or value that is refused.
pub fn show(at: &ProfileSettings) -> Result<View, Refusal> {
    let (table, exists) = read_table(&at.file)?;
    check_table(&table)?;
    Ok(View {
        exists,
        options: states(&table),
        restore_point: at.restore_point(),
    })
}

/// Write `table` to `file` through a file beside it, or remove the file when the table is empty. Whether it was
/// removed.
fn save(file: &Path, table: &Table) -> Result<bool, Refusal> {
    let failed =
        |e: std::io::Error| Refusal::new(RefusalCode::WriteFailed, None, Some(e.to_string()));
    if table.is_empty() {
        match std::fs::remove_file(file) {
            Ok(()) => return Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(true),
            Err(e) => return Err(failed(e)),
        }
    }
    let text = toml::to_string(table)
        .map_err(|e| Refusal::new(RefusalCode::WriteFailed, None, Some(e.to_string())))?;
    write_atomically(file, text.as_bytes()).map_err(failed)?;
    Ok(false)
}

fn write_atomically(file: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, file).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// Drop the restore point: a change after Restore defaults ends its Undo.
fn drop_restore_point(at: &ProfileSettings) -> Result<(), Refusal> {
    match std::fs::remove_file(at.previous()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Refusal::new(
            RefusalCode::WriteFailed,
            None,
            Some(e.to_string()),
        )),
    }
}

fn change(action: ChangeAction, keys: Vec<String>, changed: bool) -> Change {
    Change {
        action,
        keys,
        changed,
        dry_run: false,
        file_removed: false,
        restore_point: None,
    }
}

/// Set option `key` to `value` (JSON: a bool, a number or a list of strings). Setting the default value writes it:
/// it then counts as changed, and Restore defaults removes it.
///
/// # Errors
///
/// The key is not an option, the value is refused, or the file cannot be read or written. Nothing changed then.
pub fn set(at: &ProfileSettings, key: &str, value: &Value) -> Result<Change, Refusal> {
    let d =
        descriptor(key).ok_or_else(|| Refusal::new(RefusalCode::UnknownKey, Some(key), None))?;
    let (mut table, _) = read_table(&at.file)?;
    let value = to_toml(d, value)?;
    check_value(d, &value)?;
    let before = table.clone();
    put(&mut table, key, value);
    check_table(&table)?;
    let changed = table != before;
    if changed {
        save(&at.file, &table)?;
        drop_restore_point(at)?;
    }
    let mut c = change(ChangeAction::Set, vec![key.to_owned()], changed);
    c.restore_point = at.restore_point();
    Ok(c)
}

/// Remove `key` from the file, so the option takes its default again. Any key the file holds can be removed, also
/// one this version does not know. The file goes when nothing is left in it.
///
/// # Errors
///
/// The file cannot be read, what is left is refused, or the file cannot be written. Nothing changed then.
pub fn unset(at: &ProfileSettings, key: &str) -> Result<Change, Refusal> {
    let (mut table, exists) = read_table(&at.file)?;
    if !remove(&mut table, key) {
        if descriptor(key).is_none() {
            return Err(Refusal::new(RefusalCode::UnknownKey, Some(key), None));
        }
        let mut c = change(ChangeAction::Unset, vec![key.to_owned()], false);
        c.restore_point = at.restore_point();
        c.file_removed = !exists;
        return Ok(c);
    }
    check_table(&table)?;
    let removed = save(&at.file, &table)?;
    drop_restore_point(at)?;
    let mut c = change(ChangeAction::Unset, vec![key.to_owned()], true);
    c.file_removed = removed;
    Ok(c)
}

/// Restore defaults: remove every LOD generation setting from the file, also those set to their default value, and
/// keep the rest (`allow_in_repo`). The file as it was is kept as the restore point first, replacing an earlier one;
/// the file goes when nothing is left in it. With `dry_run`, only say which keys it would remove.
///
/// # Errors
///
/// The file cannot be read, holds a key or value that is refused, or cannot be written. Nothing changed then.
pub fn reset(at: &ProfileSettings, dry_run: bool) -> Result<Change, Refusal> {
    let (mut table, exists) = read_table(&at.file)?;
    check_table(&table)?;
    let mut all = Vec::new();
    leaves(&table, "", &mut all);
    let mut keys: Vec<String> = all
        .into_iter()
        .map(|(k, _)| k)
        .filter(|k| descriptor(k).is_some_and(|d| d.restores))
        .collect();
    keys.sort_by_key(|k| OPTIONS.iter().position(|d| d.key == k));
    let mut c = change(ChangeAction::Reset, keys.clone(), false);
    c.dry_run = dry_run;
    if dry_run || keys.is_empty() {
        c.restore_point = at.restore_point();
        c.file_removed = !exists;
        return Ok(c);
    }
    for k in &keys {
        remove(&mut table, k);
    }
    let before = std::fs::read(&at.file)
        .map_err(|e| Refusal::new(RefusalCode::Unreadable, None, Some(e.to_string())))?;
    write_atomically(&at.previous(), &before)
        .map_err(|e| Refusal::new(RefusalCode::WriteFailed, None, Some(e.to_string())))?;
    c.file_removed = save(&at.file, &table)?;
    c.changed = true;
    c.restore_point = Some(at.previous());
    Ok(c)
}

/// Undo the last Restore defaults: put the restore point back as the file and drop it (one level).
///
/// # Errors
///
/// There is no restore point, or it cannot be put back. Nothing changed then.
pub fn undo(at: &ProfileSettings) -> Result<Change, Refusal> {
    let previous = at.previous();
    let bytes = match std::fs::read(&previous) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(Refusal::new(RefusalCode::NothingToUndo, None, None));
        }
        Err(e) => {
            return Err(Refusal::new(
                RefusalCode::Unreadable,
                None,
                Some(e.to_string()),
            ));
        }
    };
    let (now, _) = read_table(&at.file).unwrap_or_default();
    let then: Table = String::from_utf8_lossy(&bytes).parse().unwrap_or_default();
    let mut keys: Vec<String> = OPTIONS
        .iter()
        .filter(|d| get(&now, d.key) != get(&then, d.key))
        .map(|d| d.key.to_owned())
        .collect();
    keys.sort_by_key(|k| OPTIONS.iter().position(|d| d.key == k));
    write_atomically(&at.file, &bytes)
        .map_err(|e| Refusal::new(RefusalCode::WriteFailed, None, Some(e.to_string())))?;
    drop_restore_point(at)?;
    Ok(change(ChangeAction::Undo, keys, true))
}

/// The document of a command over `at`: the settings after `outcome`, or the refusal, and `warnings` (see
/// [`super::settings_warnings`]).
#[must_use]
pub fn document(
    instance: &Path,
    at: Option<&ProfileSettings>,
    outcome: Result<Option<Change>, Refusal>,
    warnings: Vec<Warning>,
) -> SettingsDoc {
    let mut doc = SettingsDoc {
        schema_version: SCHEMA_VERSION,
        contract: CONTRACT.to_owned(),
        instance: instance.to_path_buf(),
        profile: at.map_or_else(String::new, |a| a.profile.clone()),
        file: at.map(|a| a.file.clone()),
        exists: at.is_some_and(|a| a.file.is_file()),
        options: Vec::new(),
        restore_point: at.and_then(ProfileSettings::restore_point),
        warnings: Vec::new(),
        run_in_progress: at.is_some_and(|a| super::RunLock::in_progress(instance, &a.profile)),
        change: None,
        error: None,
    };
    match outcome {
        Ok(change) => doc.change = change,
        Err(refusal) => doc.error = Some(refusal),
    }
    doc.warnings = warnings;
    if let Some(a) = at
        && let Ok(view) = show(a)
    {
        doc.options = view.options;
    }
    doc
}

/// The refusal for an Instance that cannot be opened.
#[must_use]
pub fn instance_invalid(reason: String) -> Refusal {
    Refusal::new(RefusalCode::InstanceInvalid, None, Some(reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;
    use raul_testkit::{covers, ids};

    fn at(dir: &TempDir) -> ProfileSettings {
        ProfileSettings {
            profile: "Default".to_owned(),
            file: dir.path().join(SETTINGS_FILE),
        }
    }

    fn table(text: &str) -> Table {
        text.parse().unwrap()
    }

    #[test]
    fn every_option_has_a_default_and_catalogue_text() {
        covers(&[ids::lod::settings::DEFAULTS]);
        for d in &OPTIONS {
            assert!(default_value(d.key).is_some(), "{}", d.key);
            assert!(!d.label.is_empty() && d.help.ends_with('.'), "{}", d.key);
            assert!(
                !d.help.to_lowercase().contains("lodwright") && !d.help.contains("raul "),
                "{}: no tool names or commands in view text",
                d.key
            );
        }
        let mut keys: Vec<&str> = OPTIONS.iter().map(|d| d.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), OPTIONS.len(), "a key is listed twice");
    }

    /// The Toolbox holds no copy of a default or a range: each option has a row in the generator's table, and the
    /// range it checks is the generator's.
    #[test]
    fn every_option_is_in_the_generators_table_with_its_range() {
        for d in &OPTIONS {
            if d.key == "allow_in_repo" {
                continue;
            }
            let key = generator_key(d.key).unwrap();
            let info = generator().option(&key).unwrap_or_else(|| panic!("{key}"));
            if info.minimum.is_some() {
                assert_eq!(d.min, info.minimum, "{}", d.key);
            }
            if info.maximum.is_some() {
                assert_eq!(d.max, info.maximum, "{}", d.key);
            }
            assert!(default_value(d.key).is_some(), "{}", d.key);
        }
        assert_eq!(default_value("levels.16.quality"), Some(json!(20.0)));
        assert_eq!(default_value("levels.4.max_vertices"), Some(json!(2000)));
        assert_eq!(default_value("output.water"), Some(json!(true)));
        assert_eq!(default_value("threads"), Some(Value::Null));
        assert_ne!(generator_version(), "");
    }

    #[test]
    fn the_defaults_are_the_generators_level_defaults() {
        covers(&[ids::lod::settings::DEFAULTS]);
        // lodwright's TerrainSettings::default(): levels 4, 8, 16, 32 with qualities 10, 15, 20, 25, 2000 vertices
        // and 256 pixel textures. The request the defaults give is accepted as it is.
        let mut request = LodwrightRequest::new("data", &["a.esm"], "out");
        request.terrain.levels = super::super::levels_of(&Settings::default(), true);
        assert_eq!(request.terrain.levels.len(), 4);
        request.validate().unwrap();
        let qualities: Vec<f32> = request.terrain.levels.iter().map(|l| l.quality).collect();
        assert_eq!(qualities, [10.0, 15.0, 20.0, 25.0]);
        assert!(
            request
                .terrain
                .levels
                .iter()
                .all(|l| l.max_vertices == 2000 && l.diffuse_size == 256 && l.normal_size == 256)
        );
    }

    #[test]
    fn a_key_that_is_not_an_option_is_refused_by_name() {
        covers(&[ids::lod::settings::UNKNOWN_KEY]);
        let r = check_table(&table("colour = \"blue\"\n")).unwrap_err();
        assert_eq!(r.code, RefusalCode::UnknownKey);
        assert_eq!(r.key.as_deref(), Some("colour"));
        assert_eq!(r.what, catalogue::UNKNOWN_KEY);
        assert_eq!(r.detail, catalogue::NOTHING_CHANGED);
        let r = check_table(&table("[levels.5]\nquality = 1\n")).unwrap_err();
        assert_eq!(r.key.as_deref(), Some("levels.5.quality"));
        let r = check_table(&table("[output]\nsky = true\n")).unwrap_err();
        assert_eq!(r.key.as_deref(), Some("output.sky"));
    }

    /// Each range the options check is one the generator's own check refuses too.
    #[test]
    fn the_ranges_mirror_the_generators_own_check() {
        covers(&[ids::lod::settings::RANGES]);
        for (key, bad) in [
            ("levels.4.max_vertices", json!(2)),
            ("levels.8.max_vertices", json!(32768)),
            ("levels.16.quality", json!(-1.0)),
            ("levels.32.diffuse_size", json!(2)),
            ("levels.32.normal_size", json!(8192)),
            ("levels.4.diffuse_size", json!(300)),
            ("inclusion.exclude_records", json!(["0001"])),
            ("threads", json!(0)),
        ] {
            let d = descriptor(key).unwrap();
            let v = to_toml(d, &bad).unwrap();
            assert!(
                check_value(d, &v).is_err(),
                "{key} = {bad} passed the range"
            );
            // The same value straight into the generator's request, past the option's own check.
            let mut t = Table::new();
            put(&mut t, key, v);
            let settings: Settings = t.try_into().unwrap();
            let mut request = LodwrightRequest::new("data", &["a.esm"], "out");
            let parts = super::super::request_parts(&settings);
            request.terrain = serde_json::from_value(parts["terrain"].clone()).unwrap();
            request.inclusion = serde_json::from_value(parts["inclusion"].clone()).unwrap();
            request.limits = serde_json::from_value(parts["limits"].clone()).unwrap();
            assert!(
                request.validate().is_err(),
                "the generator took {key} = {bad}"
            );
        }
    }

    #[test]
    fn a_set_value_reads_back_and_the_file_holds_only_it() {
        covers(&[ids::lod::settings::SET_READS_BACK]);
        let dir = TempDir::new("raul-lodset-set");
        let at = at(&dir);
        let c = set(&at, "levels.8.quality", &json!(12.5)).unwrap();
        assert!(c.changed);
        let view = show(&at).unwrap();
        let q = view
            .options
            .iter()
            .find(|o| o.key == "levels.8.quality")
            .unwrap();
        assert_eq!((q.value.clone(), q.overridden), (json!(12.5), true));
        let t = toml::from_str::<Table>(&std::fs::read_to_string(&at.file).unwrap()).unwrap();
        let mut keys = Vec::new();
        leaves(&t, "", &mut keys);
        assert_eq!(keys.len(), 1, "{t}");
        let s = Settings::load_file(&at.file).unwrap();
        assert_eq!(s.levels.l8.as_ref().and_then(|l| l.quality), Some(12.5));
    }

    #[test]
    fn a_refused_value_leaves_the_file_as_it_was() {
        covers(&[ids::lod::settings::REFUSED_VALUE]);
        let dir = TempDir::new("raul-lodset-refuse");
        let at = at(&dir);
        std::fs::write(&at.file, "threads = 2\n").unwrap();
        let r = set(&at, "levels.4.diffuse_size", &json!(100)).unwrap_err();
        assert_eq!(r.code, RefusalCode::NotAPowerOfTwo);
        let r = set(&at, "output.trees", &json!("no")).unwrap_err();
        assert_eq!(r.code, RefusalCode::WrongType);
        assert_eq!(std::fs::read_to_string(&at.file).unwrap(), "threads = 2\n");
    }

    #[test]
    fn restore_defaults_removes_generation_keys_also_at_their_default_and_keeps_the_git_folder_switch()
     {
        let dir = TempDir::new("raul-lodset-reset");
        let at = at(&dir);
        let text = "allow_in_repo = true\nthreads = 2\n\n[output]\nterrain = true\n\n[levels.4]\nquality = 10\n";
        std::fs::write(&at.file, text).unwrap();
        let c = reset(&at, false).unwrap();
        assert_eq!(
            c.keys,
            ["output.terrain", "threads", "levels.4.quality"].map(str::to_owned)
        );
        assert!(c.changed && !c.file_removed);
        assert_eq!(
            std::fs::read_to_string(&at.file).unwrap(),
            "allow_in_repo = true\n"
        );
        assert_eq!(std::fs::read_to_string(at.previous()).unwrap(), text);
    }

    #[test]
    fn restore_defaults_deletes_the_file_when_nothing_is_left_and_undo_puts_it_back() {
        covers(&[ids::lod::settings::RESTORE_AND_UNDO]);
        let dir = TempDir::new("raul-lodset-undo");
        let at = at(&dir);
        let text = "worlds = [\"A\"]\n";
        std::fs::write(&at.file, text).unwrap();
        let c = reset(&at, false).unwrap();
        assert!(c.file_removed && !at.file.exists());
        let c = undo(&at).unwrap();
        assert_eq!(c.keys, ["worlds"]);
        assert_eq!(std::fs::read_to_string(&at.file).unwrap(), text);
        assert!(
            !at.previous().exists(),
            "one level: the restore point is used up"
        );
        assert_eq!(undo(&at).unwrap_err().code, RefusalCode::NothingToUndo);
    }

    #[test]
    fn a_dry_run_of_restore_defaults_names_the_keys_and_writes_nothing() {
        let dir = TempDir::new("raul-lodset-dry");
        let at = at(&dir);
        std::fs::write(&at.file, "memory_mib = 512\n").unwrap();
        let c = reset(&at, true).unwrap();
        assert_eq!(
            (c.keys.clone(), c.changed),
            (vec!["memory_mib".to_owned()], false)
        );
        assert!(!at.previous().exists());
        assert_eq!(
            std::fs::read_to_string(&at.file).unwrap(),
            "memory_mib = 512\n"
        );
    }

    #[test]
    fn a_change_after_restore_defaults_ends_its_undo() {
        let dir = TempDir::new("raul-lodset-drop");
        let at = at(&dir);
        std::fs::write(&at.file, "threads = 3\n").unwrap();
        reset(&at, false).unwrap();
        assert!(at.previous().exists());
        set(&at, "output.water", &json!(false)).unwrap();
        assert!(!at.previous().exists());
        assert_eq!(undo(&at).unwrap_err().code, RefusalCode::NothingToUndo);
    }

    #[test]
    fn a_key_this_version_does_not_know_can_be_removed() {
        let dir = TempDir::new("raul-lodset-typo");
        let at = at(&dir);
        std::fs::write(&at.file, "thread = 2\nthreads = 2\n").unwrap();
        assert_eq!(show(&at).unwrap_err().code, RefusalCode::UnknownKey);
        unset(&at, "thread").unwrap();
        assert_eq!(std::fs::read_to_string(&at.file).unwrap(), "threads = 2\n");
        assert_eq!(
            unset(&at, "nothing.here").unwrap_err().code,
            RefusalCode::UnknownKey
        );
    }

    /// A tier 3 option is a row only when the generator's table lists it, and setting then restoring leaves the
    /// request the one of a Profile with no file.
    #[test]
    fn a_tier_three_row_needs_the_generator_to_list_it_and_reset_leaves_the_digest() {
        let d = descriptor("objects.drop_under_water").unwrap();
        assert_eq!(d.tier, 3);
        assert!(listed_in(generator(), d));
        let mut older = generator().clone();
        older
            .options
            .retain(|o| o.key != "objects.drop_under_water");
        assert!(!listed_in(&older, d));
        assert!(listed_in(
            &older,
            descriptor("objects.drop_under_terrain").unwrap()
        ));
        assert!(listed_in(&older, descriptor("allow_in_repo").unwrap()));

        let dir = TempDir::new("raul-lodset-tier3");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        set(&at, "objects.drop_under_terrain", &json!(true)).unwrap();
        let on = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        assert_eq!(on["objects"]["drop_under_terrain"], json!(true));
        assert_ne!(on, none);
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
        assert!(none["objects"].get("drop_under_terrain").is_none());
    }

    /// The skirts row of each terrain level is a tier 3 switch that is on, reaches the request of its own level only
    /// when turned off, and leaves the digest of a Profile with no file when restored.
    #[test]
    fn the_skirts_row_is_per_level_on_by_default_and_restores_the_request() {
        for level in LEVELS {
            let key = format!("levels.{level}.skirts");
            let d = descriptor(&key).unwrap();
            assert_eq!((d.tier, d.kind), (3, Kind::Switch));
            assert!(listed_in(generator(), d), "{key}");
            assert_eq!(default_value(&key), Some(json!(true)), "{key}");
        }
        let dir = TempDir::new("raul-lodset-skirts");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        set(&at, "levels.8.skirts", &json!(false)).unwrap();
        let off = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        let on_at: Vec<(u64, bool)> = off["terrain"]["levels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| {
                (
                    l["level"].as_u64().unwrap(),
                    l["skirts"].as_bool().unwrap_or(true),
                )
            })
            .collect();
        assert_eq!(on_at, [(4, true), (8, false), (16, true), (32, true)]);
        assert_eq!(
            set(&at, "levels.4.skirts", &json!(1)).unwrap_err().code,
            RefusalCode::WrongType
        );
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
    }

    /// Water walls are on in the generator: the request names them only when they are turned off, and a restore
    /// takes that out again.
    #[test]
    fn water_walls_are_on_by_default_and_only_turning_them_off_reaches_the_request() {
        let dir = TempDir::new("raul-lodset-walls");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        let d = descriptor("terrain.water_walls").unwrap();
        assert_eq!((d.tier, d.kind), (3, Kind::Switch));
        assert!(listed_in(generator(), d));
        assert_eq!(default_value("terrain.water_walls"), Some(json!(true)));
        assert!(Settings::default().terrain.water_walls);
        assert!(none["terrain"].get("water_walls").is_none());
        set(&at, "terrain.water_walls", &json!(false)).unwrap();
        let off = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        assert_eq!(off["terrain"]["water_walls"], json!(false));
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
    }

    /// The two experimental water switches are tier 3, off by default, written to the request only when set and
    /// gone from it again after a restore.
    #[test]
    fn the_experimental_water_rows_are_off_by_default_and_restore_the_request() {
        let dir = TempDir::new("raul-lodset-water");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        for key in ["water_shared_vertices", "water_polygons"] {
            let key = format!("terrain.{key}");
            let d = descriptor(&key).unwrap();
            assert_eq!((d.tier, d.kind), (3, Kind::Switch), "{key}");
            assert!(d.help.contains("Experimental, off by default"), "{key}");
            assert!(listed_in(generator(), d), "{key}");
            assert_eq!(default_value(&key), Some(json!(false)), "{key}");
            let bare = key.trim_start_matches("terrain.");
            assert!(none["terrain"].get(bare).is_none(), "{key}");
            set(&at, &key, &json!(true)).unwrap();
            let on = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
            assert_eq!(on["terrain"][bare], json!(true), "{key}");
            assert_eq!(
                set(&at, &key, &json!(1)).unwrap_err().code,
                RefusalCode::WrongType
            );
            reset(&at, false).unwrap();
        }
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
    }

    /// The pack row is tier 3 and on by default (raul's own default; the generator's is off): the request names it
    /// until it is turned off, and a restore puts the default back; a generator that does not list it hides it.
    #[test]
    fn the_pack_row_is_on_by_default_and_turning_it_off_leaves_it_out_of_the_request() {
        let dir = TempDir::new("raul-lodset-pack");
        let at = at(&dir);
        let on = super::super::request_parts(&Settings::default());
        let d = descriptor("output.pack").unwrap();
        assert_eq!((d.tier, d.kind), (3, Kind::Switch));
        assert!(!d.label.contains("experimental") && !d.help.contains("Experimental"));
        assert!(listed_in(generator(), d));
        assert_eq!(default_value("output.pack"), Some(json!(true)));
        assert_eq!(on["output"], json!({ "pack": true }));
        set(&at, "output.pack", &json!(false)).unwrap();
        let off = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        assert!(
            off.get("output").is_none(),
            "off is the generator's own default"
        );
        assert_eq!(
            set(&at, "output.pack", &json!("yes")).unwrap_err().code,
            RefusalCode::WrongType
        );
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, on, "a restore packs again");
        // A settings file written before the row existed packs too.
        let old = Settings::default();
        assert!(old.output.pack);
        let mut without = generator().clone();
        without.options.retain(|o| o.key != "output.pack");
        assert!(!listed_in(&without, d), "an older generator hides the row");
    }

    /// The depth under water is a tier 3 number with no default: absent from the request until set, in range when set.
    #[test]
    fn the_terrain_water_depth_row_is_absent_until_set_and_in_range() {
        let d = descriptor("terrain.drop_under_water").unwrap();
        assert_eq!((d.tier, d.kind), (3, Kind::Number));
        assert!(listed_in(generator(), d));
        assert_eq!(default_value("terrain.drop_under_water"), Some(Value::Null));
        let dir = TempDir::new("raul-lodset-depth");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        assert!(none["terrain"].get("drop_under_water").is_none());
        set(&at, "terrain.drop_under_water", &json!(64.5)).unwrap();
        let on = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        assert_eq!(on["terrain"]["drop_under_water"], json!(64.5));
        for bad in [json!(-1), json!(100_001)] {
            assert_eq!(
                set(&at, "terrain.drop_under_water", &bad).unwrap_err().code,
                RefusalCode::OutOfRange
            );
        }
        assert_eq!(
            set(&at, "terrain.drop_under_water", &json!("deep"))
                .unwrap_err()
                .code,
            RefusalCode::WrongType
        );
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
    }

    /// The texture scales row is a tier 3 table of record id to number, empty by default, written to the request only
    /// when set, with each id and each number checked.
    #[test]
    fn the_texture_scales_row_takes_record_ids_with_numbers_in_range() {
        let d = descriptor("terrain.texture_scales").unwrap();
        assert_eq!((d.tier, d.kind), (3, Kind::Scales));
        assert!(listed_in(generator(), d));
        assert_eq!(default_value("terrain.texture_scales"), Some(json!({})));
        let words = |w: &[&str]| w.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let value = parse_value(
            "terrain.texture_scales",
            &words(&["0012ab34=2", "0012AB35=0.5"]),
        )
        .unwrap();
        assert_eq!(value, json!({"0012AB34": 2.0, "0012AB35": 0.5}));
        for bad in [&["oops"][..], &["0012AB34=x"], &["0012AB34"]] {
            assert_eq!(
                parse_value("terrain.texture_scales", &words(bad))
                    .unwrap_err()
                    .code,
                RefusalCode::WrongType,
                "{bad:?}"
            );
        }
        let dir = TempDir::new("raul-lodset-scales");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        assert!(none["terrain"].get("texture_scales").is_none());
        set(&at, "terrain.texture_scales", &value).unwrap();
        let on = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        assert_eq!(
            on["terrain"]["texture_scales"],
            json!({"0012AB34": 2.0, "0012AB35": 0.5})
        );
        assert_eq!(
            set(&at, "terrain.texture_scales", &json!({"oak": 2}))
                .unwrap_err()
                .code,
            RefusalCode::NotARecord
        );
        for scale in [0.01, 17.0] {
            assert_eq!(
                set(&at, "terrain.texture_scales", &json!({"0012AB34": scale}))
                    .unwrap_err()
                    .code,
                RefusalCode::OutOfRange
            );
        }
        assert_eq!(
            set(&at, "terrain.texture_scales", &json!({"0012AB34": "big"}))
                .unwrap_err()
                .code,
            RefusalCode::WrongType
        );
        assert_eq!(
            set(&at, "terrain.texture_scales", &json!([1]))
                .unwrap_err()
                .code,
            RefusalCode::WrongType
        );
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
    }

    /// The specular row of each terrain level is a tier 3 switch that is off, reaches the request of its own level
    /// only when set, and leaves the digest of a Profile with no file when restored.
    #[test]
    fn the_specular_in_alpha_row_is_per_level_off_by_default_and_restores_the_request() {
        for level in LEVELS {
            let key = format!("levels.{level}.specular_alpha");
            let d = descriptor(&key).unwrap();
            assert_eq!((d.tier, d.kind), (3, Kind::Switch));
            assert!(listed_in(generator(), d), "{key}");
            assert_eq!(default_value(&key), Some(json!(false)), "{key}");
        }
        let dir = TempDir::new("raul-lodset-specular");
        let at = at(&dir);
        let none = super::super::request_parts(&Settings::default());
        assert_eq!(none["terrain"]["levels"], json!([]));
        set(&at, "levels.16.specular_alpha", &json!(true)).unwrap();
        assert_eq!(
            parse_value("levels.16.specular_alpha", &["on".to_owned()]).unwrap(),
            json!(true)
        );
        let on = super::super::request_parts(&Settings::load_file(&at.file).unwrap());
        let levels = on["terrain"]["levels"].as_array().unwrap();
        let on_at: Vec<(u64, bool)> = levels
            .iter()
            .map(|l| {
                (
                    l["level"].as_u64().unwrap(),
                    l["specular_alpha"].as_bool().unwrap_or(false),
                )
            })
            .collect();
        assert_eq!(on_at, [(4, false), (8, false), (16, true), (32, false)]);
        assert_eq!(
            set(&at, "levels.8.specular_alpha", &json!(1))
                .unwrap_err()
                .code,
            RefusalCode::WrongType
        );
        reset(&at, false).unwrap();
        let back = super::super::request_parts(&Settings::load_file(&at.file).unwrap_or_default());
        assert_eq!(back, none);
    }

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn a_world_no_plugin_defines_is_a_warning_and_case_does_not_matter() {
        let s = Settings {
            worlds: names(&["wastelandnv", "Nowhere"]),
            ..Settings::default()
        };
        let w = warnings(&s, None, Ok(&names(&["WastelandNV"])));
        assert_eq!(w.len(), 1, "{w:?}");
        assert_eq!(
            (w[0].code, w[0].value.as_deref()),
            (WarningCode::UnknownWorld, Some("Nowhere"))
        );
        assert_eq!(w[0].what, catalogue::UNKNOWN_WORLD);
        let w = warnings(&s, None, Err("unreadable"));
        assert_eq!(w[0].code, WarningCode::WorldsUnchecked);
        assert_eq!(warnings(&Settings::default(), None, Err("unreadable")), []);
    }

    #[test]
    fn inclusion_rules_that_name_an_inactive_plugin_or_leave_no_object_are_warnings() {
        let mut s = Settings::default();
        s.inclusion.only_plugins = names(&["A.esp"]);
        s.inclusion.exclude_plugins = names(&["a.ESP", "Gone.esp"]);
        let w = warnings(&s, Some(&names(&["Base.esm", "A.esp"])), Ok(&[]));
        let codes: Vec<(WarningCode, Option<&str>)> =
            w.iter().map(|w| (w.code, w.value.as_deref())).collect();
        assert_eq!(
            codes,
            [
                (WarningCode::PluginNotActive, Some("Gone.esp")),
                (WarningCode::NoObjectLeft, None)
            ]
        );
        s.inclusion.only_plugins.push("Base.esm".to_owned());
        assert_eq!(warnings(&s, None, Ok(&[])), [], "Base.esm is left");
    }

    #[test]
    fn setting_an_option_to_its_default_writes_it_and_lists_it_as_set() {
        let dir = TempDir::new("raul-lodset-explicit");
        let at = at(&dir);
        assert!(set(&at, "output.water", &json!(true)).unwrap().changed);
        let view = show(&at).unwrap();
        let o = view
            .options
            .iter()
            .find(|o| o.key == "output.water")
            .unwrap();
        assert!(o.overridden && o.value == o.default);
        assert_eq!(reset(&at, true).unwrap().keys, ["output.water"]);
    }

    #[test]
    fn command_line_words_become_values_of_the_options_kind() {
        let w = |s: &[&str]| s.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            parse_value("output.trees", &w(&["off"])).unwrap(),
            json!(false)
        );
        assert_eq!(parse_value("threads", &w(&["4"])).unwrap(), json!(4));
        assert_eq!(
            parse_value("levels.4.quality", &w(&["7.5"])).unwrap(),
            json!(7.5)
        );
        assert_eq!(
            parse_value("worlds", &w(&["A", "B"])).unwrap(),
            json!(["A", "B"])
        );
        assert_eq!(parse_value("worlds", &[]).unwrap(), json!([]));
        assert_eq!(
            parse_value("threads", &w(&["four"])).unwrap_err().code,
            RefusalCode::WrongType
        );
        assert_eq!(
            parse_value("nope", &[]).unwrap_err().code,
            RefusalCode::UnknownKey
        );
    }
}
