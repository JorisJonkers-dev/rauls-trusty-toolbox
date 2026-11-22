// SPDX-License-Identifier: GPL-3.0-only
//! `raul scene` and `raul bench game`: the engine's picture, soak, sweep and bench tool, reached through the Profile.
//!
//! raul resolves what the bare tool wants by hand: the game folder (`raul config`), the Profile's Mods laid over it in
//! priority order, its active Plugins in Load order, the output folder under raul's data folder, the label and the
//! memory cap of the game start. The work itself is the engine's, behind [`SceneEngine`].
//!
//! The seam: the engine's library entry points do not exist yet, so [`CliTool`] runs the `nv-rs-scene` program (the
//! only implementation). When the entry points land, the full raul registers an in-process [`SceneEngine`] with
//! [`install`] (as it does for the play mode) and [`CliTool`] goes away; nothing else in this file changes.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use nvmod_instance::NvmodInstance;
use raul_core::config::{Config, Key};
use raul_core::context::Context;

/// The program the seam runs, and the variable that names another one.
pub const TOOL: &str = "nv-rs-scene";
/// The variable that names the tool program instead of `nv-rs-scene` beside raul or on the path.
pub const TOOL_ENV: &str = "RAUL_SCENE_TOOL";

const USAGE_LINE: &str = "scene takes `run`, `compare`, `soak`, `sweep` or `places`; `bench game` takes `--suite` or `compare`";

/// The place files raul carries: a few Mojave views by day and by night, with only coordinates and editor ids in them.
pub const BUILT_IN_PLACES: &[(&str, &str)] = &[
    ("mojave-day", include_str!("../scenes/mojave-day.toml")),
    ("mojave-night", include_str!("../scenes/mojave-night.toml")),
];

// ---------------------------------------------------------------------------------------------------------------
// What the engine is asked.

/// One start of the game, as raul has resolved it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Start {
    /// The program the game starts as (raul's own binary in the play mode, or a viewer the person names).
    pub viewer: PathBuf,
    /// The game folder.
    pub game: PathBuf,
    /// The place file.
    pub places: PathBuf,
    /// Where this start writes.
    pub out: PathBuf,
    /// The variant's name; empty for a start that has none (soak, sweep, bench).
    pub variant: String,
    /// Data folders laid over the game, lowest priority first.
    pub overlays: Vec<PathBuf>,
    /// The Plugins in Load order; empty: the engine takes the ones at the top of the first overlay.
    pub plugins: Vec<String>,
    /// `--only`: the places taken, by name.
    pub only: Vec<String>,
    /// `WxH`; empty: the engine's default.
    pub resolution: String,
    /// The loaded square of cells; 0: the game's own.
    pub grids: u32,
    /// Quick look: small pictures, short settle.
    pub quick: bool,
    /// Frames a soak holds each place; 0: the engine's default.
    pub frames: u32,
    /// The label of the start, shown by the system's monitor.
    pub label: String,
    /// The memory limit of one start (`10G`); empty: the engine's default share.
    pub mem: String,
    /// No memory cap (the person asked, or the capping program is not there).
    pub no_cap: bool,
    /// The lock file every game start of this machine takes (shared; a bench takes it exclusively).
    pub lock: Option<PathBuf>,
    /// A bench: the suite and the baselines.
    pub suite: String,
    /// `--baseline` name.
    pub baseline: Option<String>,
    /// `--save-baseline` name.
    pub save_baseline: Option<String>,
}

/// What raul asks of the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Pictures of the places, for one variant.
    Run(Start),
    /// The difference between two pictures or two folders of pictures.
    Compare {
        /// The first.
        a: PathBuf,
        /// The second.
        b: PathBuf,
        /// A second run of the first, to leave out what differs by itself.
        noise: Option<PathBuf>,
    },
    /// Hold every place for some frames and count stops and errors.
    Soak(Start),
    /// The stability sweep.
    Sweep(Start),
    /// The in-engine benchmark.
    Bench(Start),
    /// Two bench summaries side by side.
    BenchCompare {
        /// The first summary.
        a: PathBuf,
        /// The second.
        b: PathBuf,
    },
}

/// The engine's scene tool. The one implementation today is [`CliTool`].
pub trait SceneEngine: Send + Sync {
    /// Runs one request, writing what the person should read to `out`/`err`; returns the exit code (0 fine, 1 the run
    /// ran and found a problem, anything else the request could not be run).
    ///
    /// # Errors
    /// Why the request could not be started at all.
    fn execute(
        &self,
        request: &Request,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<i32, String>;
}

static ENGINE: OnceLock<Box<dyn SceneEngine>> = OnceLock::new();

/// Registers the in-process engine. The first call wins; without one, [`CliTool`] is used.
pub fn install(engine: Box<dyn SceneEngine>) {
    let _ = ENGINE.set(engine);
}

// ---------------------------------------------------------------------------------------------------------------
// The seam: the command-line tool.

/// Runs `nv-rs-scene`. Replaced by an in-process engine when the library entry points exist.
#[derive(Debug, Clone, Default)]
pub struct CliTool {
    program: Option<PathBuf>,
}

impl CliTool {
    /// Runs the program at `program` instead of looking for it.
    #[must_use]
    pub fn at(program: impl Into<PathBuf>) -> Self {
        Self {
            program: Some(program.into()),
        }
    }

    fn program(&self) -> PathBuf {
        if let Some(p) = &self.program {
            return p.clone();
        }
        if let Some(p) = std::env::var_os(TOOL_ENV).filter(|v| !v.is_empty()) {
            return PathBuf::from(p);
        }
        let name = format!("{TOOL}{}", std::env::consts::EXE_SUFFIX);
        let beside = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.join(&name)))
            .filter(|p| p.is_file());
        beside.unwrap_or_else(|| PathBuf::from(name))
    }
}

/// The words after `nv-rs-scene` for a request. Pure, so a test can read them.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn tool_args(request: &Request) -> Vec<String> {
    fn push(v: &mut Vec<String>, flag: &str, value: impl Into<String>) {
        v.push(flag.to_owned());
        v.push(value.into());
    }
    fn display(p: &Path) -> String {
        p.display().to_string()
    }
    fn start_words(v: &mut Vec<String>, s: &Start) {
        push(v, "--game", display(&s.game));
        push(v, "--places", display(&s.places));
        push(v, "--out", display(&s.out));
        if s.grids > 0 {
            push(v, "--grids", s.grids.to_string());
        }
        if !s.plugins.is_empty() {
            push(v, "--plugins", s.plugins.join(","));
        }
        if !s.label.is_empty() {
            push(v, "--label", s.label.clone());
            push(v, "--who", "raul");
        }
        if s.no_cap {
            v.push("--no-cap".into());
        } else if !s.mem.is_empty() {
            push(v, "--mem", s.mem.clone());
        }
    }
    let mut v = Vec::new();
    match request {
        Request::Run(s) => {
            v.push("run".into());
            push(&mut v, "--viewer", display(&s.viewer));
            start_words(&mut v, s);
            let overlays: Vec<String> = s.overlays.iter().map(|p| display(p)).collect();
            if overlays.is_empty() {
                push(&mut v, "--variant", s.variant.clone());
            } else {
                push(
                    &mut v,
                    "--variant",
                    format!("{}={}", s.variant, overlays.join("+")),
                );
            }
            if !s.only.is_empty() {
                push(&mut v, "--only", s.only.join(","));
            }
            if !s.resolution.is_empty() {
                push(&mut v, "--resolution", s.resolution.clone());
            }
            if s.quick {
                v.push("--quick".into());
            }
            if let Some(lock) = &s.lock {
                push(&mut v, "--game-lock", display(lock));
            }
        }
        Request::Soak(s) | Request::Sweep(s) => {
            v.push(
                if matches!(request, Request::Soak(_)) {
                    "soak"
                } else {
                    "sweep"
                }
                .into(),
            );
            push(&mut v, "--viewer", display(&s.viewer));
            start_words(&mut v, s);
            if s.frames > 0 {
                push(&mut v, "--frames", s.frames.to_string());
            }
            if let Some(o) = s.overlays.first() {
                push(&mut v, "--overlay", display(o));
            }
            if let Some(lock) = &s.lock {
                push(&mut v, "--game-lock", display(lock));
            }
        }
        Request::Bench(s) => {
            v.push("bench".into());
            push(&mut v, "--viewer", display(&s.viewer));
            start_words(&mut v, s);
            push(&mut v, "--suite", s.suite.clone());
            if let Some(o) = s.overlays.first() {
                push(&mut v, "--overlay", display(o));
            }
            if let Some(lock) = &s.lock {
                push(&mut v, "--lock", display(lock));
            }
            if let Some(name) = &s.baseline {
                push(&mut v, "--baseline", name.clone());
            }
            if let Some(name) = &s.save_baseline {
                push(&mut v, "--save-baseline", name.clone());
            }
        }
        Request::Compare { a, b, noise } => {
            v.push("compare".into());
            if let Some(n) = noise {
                push(&mut v, "--noise", display(n));
            }
            v.push(display(a));
            v.push(display(b));
        }
        Request::BenchCompare { a, b } => {
            v.push("bench-compare".into());
            v.push(display(a));
            v.push(display(b));
        }
    }
    v
}

impl SceneEngine for CliTool {
    fn execute(
        &self,
        request: &Request,
        _out: &mut dyn Write,
        _err: &mut dyn Write,
    ) -> Result<i32, String> {
        let program = self.program();
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        // Attached: the tool's own progress lines go straight to the person's terminal.
        let started = raul_runner::start_program(
            &program.to_string_lossy(),
            &tool_args(request),
            &BTreeMap::new(),
            &cwd,
            true,
            None,
        )
        .map_err(|why| {
            format!(
                "{why}; the engine's picture tool `{TOOL}` is not installed here (put it beside raul or on the path, or \
                 name it in {TOOL_ENV})"
            )
        })?;
        started.wait()
    }
}

fn execute(
    request: &Request,
    tool: Option<&PathBuf>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<i32, String> {
    if let Some(engine) = ENGINE.get() {
        return engine.execute(request, out, err);
    }
    let cli = tool.map_or_else(CliTool::default, CliTool::at);
    cli.execute(request, out, err)
}

// ---------------------------------------------------------------------------------------------------------------
// The Profile as a variant.

/// What a variant draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The Profile as it is.
    Profile,
    /// The Profile without its generated LOD Mod and the stub Plugin that Mod carries.
    ProfileWithoutLod,
    /// The game's own data, no Mods.
    Game,
    /// A data folder laid over the game by hand.
    Folder(PathBuf),
}

impl Source {
    /// Reads `profile`, `profile-without-lod`, `game` or a folder.
    ///
    /// # Errors
    /// The word is none of those and not a folder.
    pub fn parse(word: &str) -> Result<Source, String> {
        match word {
            "profile" => Ok(Source::Profile),
            "profile-without-lod" => Ok(Source::ProfileWithoutLod),
            "game" => Ok(Source::Game),
            path if Path::new(path).is_dir() => Ok(Source::Folder(PathBuf::from(path))),
            other => Err(format!(
                "`{other}` is not a variant source: use profile, profile-without-lod, game, or a folder that exists"
            )),
        }
    }
}

/// A named variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// Names its folder of pictures: letters, digits, `-` and `_`.
    pub name: String,
    /// What it draws.
    pub source: Source,
}

/// A variant resolved to folders and Plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The variant's name.
    pub name: String,
    /// Mod folders, lowest priority first.
    pub overlays: Vec<PathBuf>,
    /// Active Plugins in Load order.
    pub plugins: Vec<String>,
}

/// The names of the Profile's generated LOD Mods.
fn lod_mod_names(instance: &NvmodInstance, profile: &str) -> Vec<String> {
    raul_drivers::lod::generated_mods(instance, profile)
        .mods
        .into_iter()
        .map(|m| m.name)
        .collect()
}

/// Whether the Profile lists a generated LOD Mod that is enabled.
#[must_use]
pub fn has_lod_mod(instance: &NvmodInstance, profile: &str) -> bool {
    raul_drivers::lod::generated_mods(instance, profile)
        .mods
        .iter()
        .any(|m| m.enabled)
}

/// Resolves `variant` against the Profile.
///
/// # Errors
/// The Profile's lists cannot be read, or a Mod folder is missing.
pub fn resolve(
    instance: &NvmodInstance,
    profile: &str,
    variant: &Variant,
) -> Result<Resolved, String> {
    let name = variant.name.clone();
    let without_lod = match &variant.source {
        Source::Game => {
            return Ok(Resolved {
                name,
                overlays: Vec::new(),
                plugins: Vec::new(),
            });
        }
        Source::Folder(f) => {
            return Ok(Resolved {
                name,
                overlays: vec![f.clone()],
                plugins: Vec::new(),
            });
        }
        Source::Profile => false,
        Source::ProfileWithoutLod => true,
    };
    let dir = instance.profile_dir(profile);
    let text = std::fs::read(dir.join("modlist.txt"))
        .map_err(|e| format!("{}: {e}", dir.join("modlist.txt").display()))?;
    let list = nvmod_formats::Modlist::parse(&nvmod_formats::text::decode(&text));
    let skip = if without_lod {
        lod_mod_names(instance, profile)
    } else {
        Vec::new()
    };
    let mut overlays = Vec::new();
    // The first line wins a conflict, so the lowest priority goes down first.
    for e in list.entries.iter().rev() {
        if e.state != nvmod_formats::ModState::Enabled
            || e.kind != nvmod_formats::EntryKind::Mod
            || skip.iter().any(|s| s == &e.name)
        {
            continue;
        }
        let folder = instance.mod_dir(&e.name);
        if !folder.is_dir() {
            return Err(format!(
                "the Mod `{}` is enabled but has no folder at {}",
                e.name,
                folder.display()
            ));
        }
        overlays.push(folder);
    }
    let plugins = nvmod_loadorder::read_slots(&dir)
        .map_err(|e| format!("the Load order of `{profile}` cannot be read: {e:?}"))?
        .into_iter()
        .filter(|s| s.active)
        .map(|s| s.name)
        .filter(|n| !(without_lod && n.eq_ignore_ascii_case(raul_drivers::lod::PACKED_PLUGIN)))
        .collect();
    Ok(Resolved {
        name,
        overlays,
        plugins,
    })
}

// ---------------------------------------------------------------------------------------------------------------
// The command line.

#[derive(Default)]
struct Flags {
    instance: Option<PathBuf>,
    profile: Option<String>,
    game: Option<PathBuf>,
    places: Option<String>,
    variants: Vec<(String, String)>,
    only: Vec<String>,
    out: Option<PathBuf>,
    resolution: String,
    grids: u32,
    quick: bool,
    frames: u32,
    viewer: Option<PathBuf>,
    tool: Option<PathBuf>,
    lock: Option<PathBuf>,
    mem: String,
    no_cap: bool,
    suite: Option<String>,
    baseline: Option<String>,
    save_baseline: Option<String>,
    noise: Option<PathBuf>,
    rest: Vec<String>,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut f = Flags::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{a} needs a value"))
        };
        match a.as_str() {
            "--instance" => f.instance = Some(PathBuf::from(value()?)),
            "--profile" => f.profile = Some(value()?),
            "--game" => f.game = Some(PathBuf::from(value()?)),
            "--places" => f.places = Some(value()?),
            "--variant" => {
                let v = value()?;
                let (n, s) = v.split_once('=').ok_or_else(|| {
                    format!("--variant needs NAME=SOURCE (profile, profile-without-lod, game or a folder), not {v}")
                })?;
                f.variants.push((n.to_owned(), s.to_owned()));
            }
            "--only" => f.only = value()?.split(',').map(str::to_owned).collect(),
            "--out" => f.out = Some(PathBuf::from(value()?)),
            "--resolution" => f.resolution = value()?,
            "--grids" => {
                f.grids = value()?
                    .parse()
                    .ok()
                    .filter(|n: &u32| n % 2 == 1 && (3..=21).contains(n))
                    .ok_or("--grids is an odd number from 3 to 21")?;
            }
            "--quick" => f.quick = true,
            "--frames" => {
                f.frames = value()?
                    .parse()
                    .map_err(|_| "--frames needs a number".to_owned())?;
            }
            "--viewer" => f.viewer = Some(PathBuf::from(value()?)),
            "--tool" => f.tool = Some(PathBuf::from(value()?)),
            "--game-lock" => f.lock = Some(PathBuf::from(value()?)),
            "--mem" => f.mem = value()?,
            "--no-cap" => f.no_cap = true,
            "--suite" => f.suite = Some(value()?),
            "--baseline" => f.baseline = Some(value()?),
            "--save-baseline" => f.save_baseline = Some(value()?),
            "--noise" => f.noise = Some(PathBuf::from(value()?)),
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other => f.rest.push(other.to_owned()),
        }
    }
    for (name, _) in &f.variants {
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!(
                "the variant name `{name}` may hold letters, digits, - and _ only"
            ));
        }
    }
    Ok(f)
}

/// `raul scene ...`: `args` are the words after `scene`.
pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let Some(verb) = args.first().map(String::as_str) else {
        return crate::usage(err, USAGE_LINE);
    };
    let flags = match parse(&args[1..]) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    match verb {
        "places" => places(&flags, out, err),
        "compare" => compare(&flags, out, err),
        "run" => run_variants(ctx, &flags, out, err),
        "soak" | "sweep" => one_start(ctx, verb, &flags, out, err),
        _ => crate::usage(err, USAGE_LINE),
    }
}

/// `raul bench game ...`: `args` are the words after `game`.
pub(crate) fn bench_game(
    ctx: &Context,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let flags = match parse(args) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    if flags.rest.first().map(String::as_str) == Some("compare") {
        let [_, a, b] = flags.rest.as_slice() else {
            return crate::usage(err, "bench game compare needs <a.json> <b.json>");
        };
        return go(
            &Request::BenchCompare {
                a: a.into(),
                b: b.into(),
            },
            flags.tool.as_ref(),
            out,
            err,
        );
    }
    if flags.suite.is_none() {
        return crate::usage(err, "bench game needs --suite quick|mojave|soak|capital");
    }
    one_start(ctx, "bench", &flags, out, err)
}

/// Runs a request and turns its result into the exit code, saying plainly what went wrong.
fn go(request: &Request, tool: Option<&PathBuf>, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let result = execute(request, tool, out, err);
    finish(err, result)
}

fn refused(err: &mut dyn Write, verb: &str, why: &str) -> u8 {
    let _ = writeln!(err, "raul scene {verb}: refused: {why}");
    crate::REFUSED
}

fn finish(err: &mut dyn Write, result: Result<i32, String>) -> u8 {
    match result {
        Ok(0) => crate::OK,
        Ok(1) => crate::FOUND,
        Ok(code) => {
            let _ = writeln!(err, "raul scene: the tool ended with status {code}");
            crate::FAILED
        }
        Err(why) => {
            let _ = writeln!(err, "raul scene: {why}");
            crate::FAILED
        }
    }
}

/// Writes the built-in place files into `--out` (or the current folder).
fn places(flags: &Flags, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let dir = flags.out.clone().unwrap_or_else(|| PathBuf::from("."));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return refused(err, "places", &format!("{}: {e}", dir.display()));
    }
    for (name, text) in BUILT_IN_PLACES {
        let file = dir.join(format!("{name}.toml"));
        if file.exists() {
            let _ = writeln!(out, "{} is there already; left as it is", file.display());
            continue;
        }
        if let Err(e) = std::fs::write(&file, text) {
            return refused(err, "places", &format!("{}: {e}", file.display()));
        }
        let _ = writeln!(out, "wrote {}", file.display());
    }
    crate::OK
}

fn compare(flags: &Flags, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let [a, b] = flags.rest.as_slice() else {
        return crate::usage(
            err,
            "scene compare needs <a> <b> (two pictures or two folders)",
        );
    };
    let code = go(
        &Request::Compare {
            a: a.into(),
            b: b.into(),
            noise: flags.noise.clone(),
        },
        flags.tool.as_ref(),
        out,
        err,
    );
    let _ = writeln!(
        out,
        "A number alone does not pass a view: open both pictures before you say they are equal."
    );
    code
}

/// The place file: a file that exists, or the name of a built-in one (written into `run_dir`).
fn places_file(word: &str, run_dir: &Path) -> Result<PathBuf, String> {
    if Path::new(word).is_file() {
        return std::path::absolute(word).map_err(|e| format!("{word}: {e}"));
    }
    if let Some((name, text)) = BUILT_IN_PLACES.iter().find(|(n, _)| *n == word) {
        std::fs::create_dir_all(run_dir).map_err(|e| format!("{}: {e}", run_dir.display()))?;
        let file = run_dir.join(format!("{name}.toml"));
        std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
        return Ok(file);
    }
    let known: Vec<&str> = BUILT_IN_PLACES.iter().map(|(n, _)| *n).collect();
    Err(format!(
        "no place file `{word}` (a file, or one that raul carries: {})",
        known.join(", ")
    ))
}

/// What every start of one command shares, resolved.
struct Setup {
    instance: Option<(NvmodInstance, String)>,
    start: Start,
    run_dir: PathBuf,
}

fn setup(ctx: &Context, verb: &str, f: &Flags) -> Result<Setup, String> {
    let instance_root = f.instance.clone().or_else(|| {
        ctx.config_file()
            .and_then(|c| Config::load(&c).ok())
            .and_then(|c| c.get(Key::Instance))
    });
    let instance = match instance_root {
        Some(root) => {
            let i = NvmodInstance::open_read_only(&root)
                .map_err(|e| format!("{}: {e}", root.display()))?;
            let p = i
                .resolve_profile(f.profile.as_deref())
                .map_err(|e| e.to_string())?;
            Some((i, p))
        }
        None if f.profile.is_some() => {
            return Err(
                "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`"
                    .into(),
            );
        }
        None => None,
    };
    let game = f
        .game
        .clone()
        .or_else(|| {
            ctx.config_file()
                .and_then(|c| Config::load(&c).ok())
                .and_then(|c| c.get(Key::GameNewVegas))
        })
        .ok_or("no game folder: pass --game <dir> or run `raul config set game.newvegas <dir>`")?;
    let stamp = {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        raul_drivers::lod::utc_minute(secs)
            .replace(' ', "T")
            .replace(':', "")
            .replace("UTC", "Z")
            .replace("TT", "T")
    };
    let who = instance
        .as_ref()
        .map_or("game".to_owned(), |(_, p)| p.clone());
    let run_dir = match &f.out {
        Some(o) => std::path::absolute(o).map_err(|e| format!("{}: {e}", o.display()))?,
        None => raul_core::data::folder()
            .ok_or("raul has no data folder here (set RAUL_DATA_DIR to an absolute folder, or pass --out)")?
            .join("scene")
            .join(&who)
            .join(&stamp),
    };
    let places = match &f.places {
        Some(w) => places_file(w, &run_dir)?,
        None if verb == "bench" => PathBuf::new(),
        None => {
            return Err(
                "--places <file> is needed (a place file, or mojave-day / mojave-night)".into(),
            );
        }
    };
    // The game child is raul's own binary in the play mode, through a one-line launcher the tool can start as a viewer.
    let viewer = match &f.viewer {
        Some(v) => v.clone(),
        None if crate::play::enabled() => launcher(&run_dir)?,
        None => {
            return Err("this raul has no engine in it: pass --viewer <program> naming a game viewer, or use the full raul".into());
        }
    };
    let lock = f
        .lock
        .clone()
        .or_else(|| raul_core::data::folder().map(|d| d.join("scene").join("game.lock")));
    let (no_cap, mem) = if f.no_cap || !gamerun_present() {
        (true, f.mem.clone())
    } else {
        (false, f.mem.clone())
    };
    Ok(Setup {
        instance,
        run_dir: run_dir.clone(),
        start: Start {
            viewer,
            game,
            places,
            out: run_dir,
            only: f.only.clone(),
            resolution: f.resolution.clone(),
            grids: f.grids,
            quick: f.quick,
            frames: f.frames,
            label: format!("raul-scene-{verb}-{who}"),
            mem,
            no_cap,
            lock,
            suite: f.suite.clone().unwrap_or_default(),
            baseline: f.baseline.clone(),
            save_baseline: f.save_baseline.clone(),
            ..Start::default()
        },
    })
}

/// Whether the memory-capping program of this machine is there; without it a start has no cap, and says so.
fn gamerun_present() -> bool {
    std::env::var_os("HOME")
        .map(|h| Path::new(&h).join(".local/bin/gamerun"))
        .is_some_and(|p| p.is_file())
}

/// A launcher the tool starts as the viewer: it runs raul's own binary in the play mode with the folder the tool gives.
fn launcher(run_dir: &Path) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(run_dir).map_err(|e| format!("{}: {e}", run_dir.display()))?;
    let (file, text) = if cfg!(windows) {
        (
            run_dir.join("raul-play.cmd"),
            format!("@\"{}\" --play-mode %*\r\n", exe.display()),
        )
    } else {
        (
            run_dir.join("raul-play.sh"),
            format!("#!/bin/sh\nexec '{}' --play-mode \"$@\"\n", exe.display()),
        )
    };
    std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }
    Ok(file)
}

#[allow(clippy::too_many_lines)]
fn run_variants(ctx: &Context, f: &Flags, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let s = match setup(ctx, "run", f) {
        Ok(s) => s,
        Err(why) => return refused(err, "run", &why),
    };
    // The variants: those given, else the Profile's pair (with and without its LOD Mod) or the Profile alone.
    let mut variants = Vec::new();
    for (name, word) in &f.variants {
        match Source::parse(word) {
            Ok(source) => variants.push(Variant {
                name: name.clone(),
                source,
            }),
            Err(why) => return refused(err, "run", &why),
        }
    }
    if variants.is_empty() {
        match &s.instance {
            Some((i, p)) if has_lod_mod(i, p) => {
                variants.push(Variant {
                    name: "with-lod".into(),
                    source: Source::Profile,
                });
                variants.push(Variant {
                    name: "without-lod".into(),
                    source: Source::ProfileWithoutLod,
                });
            }
            Some(_) => variants.push(Variant {
                name: "profile".into(),
                source: Source::Profile,
            }),
            None => variants.push(Variant {
                name: "game".into(),
                source: Source::Game,
            }),
        }
    }
    let _ = writeln!(
        out,
        "raul scene run: {} variant(s), pictures under {}",
        variants.len(),
        s.run_dir.display()
    );
    if s.start.no_cap {
        let _ = writeln!(
            out,
            "raul scene run: no memory cap on the game start (--no-cap, or the capping program is not installed)"
        );
    }
    let mut code = crate::OK;
    let mut done: Vec<String> = Vec::new();
    for v in &variants {
        let resolved = match (&s.instance, &v.source) {
            (Some((i, p)), _) => resolve(i, p, v),
            (None, Source::Profile | Source::ProfileWithoutLod) => {
                Err("this variant needs a Profile: pass --instance <dir>".to_owned())
            }
            (None, _) => Ok(resolve_without(v)),
        };
        let r = match resolved {
            Ok(r) => r,
            Err(why) => return refused(err, "run", &why),
        };
        let mut start = s.start.clone();
        start.variant.clone_from(&r.name);
        start.overlays = r.overlays;
        start.plugins = r.plugins;
        start.label = format!("{}-{}", start.label, r.name);
        let _ = writeln!(out, "raul scene run: {} ...", r.name);
        let c = go(&Request::Run(start), f.tool.as_ref(), out, err);
        if c != crate::OK {
            let _ = writeln!(
                err,
                "raul scene run: variant {} did not finish; its folder is {}",
                r.name,
                s.run_dir.join(&r.name).display()
            );
            code = c;
            break;
        }
        done.push(r.name);
    }
    if code == crate::OK && done.len() > 1 {
        let first = s.run_dir.join(&done[0]).join("pictures");
        for name in &done[1..] {
            let _ = writeln!(out, "--- {name} against {}", done[0]);
            let c = go(
                &Request::Compare {
                    a: first.clone(),
                    b: s.run_dir.join(name).join("pictures"),
                    noise: None,
                },
                f.tool.as_ref(),
                out,
                err,
            );
            if c != crate::OK {
                code = c;
            }
        }
        let _ = writeln!(
            out,
            "Open both pictures of each name before you say they are equal or different."
        );
    }
    code
}

fn resolve_without(v: &Variant) -> Resolved {
    Resolved {
        name: v.name.clone(),
        overlays: match &v.source {
            Source::Folder(f) => vec![f.clone()],
            _ => Vec::new(),
        },
        plugins: Vec::new(),
    }
}

/// A soak, a sweep or a bench: one start over the Profile as it is (or the game's own data without an Instance).
fn one_start(ctx: &Context, verb: &str, f: &Flags, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let s = match setup(ctx, verb, f) {
        Ok(s) => s,
        Err(why) => return refused(err, verb, &why),
    };
    let mut start = s.start.clone();
    if let Some((i, p)) = &s.instance {
        match resolve(
            i,
            p,
            &Variant {
                name: "profile".into(),
                source: Source::Profile,
            },
        ) {
            Ok(r) => {
                start.overlays = r.overlays;
                start.plugins = r.plugins;
            }
            Err(why) => return refused(err, verb, &why),
        }
    }
    // The tool takes one data folder for a soak, a sweep or a bench: the Profile's Staging tree is that folder.
    if let Some((i, p)) = &s.instance {
        match nvmod_loadorder::staging_data(i, p) {
            Ok(d) => start.overlays = vec![d],
            Err(e) => return refused(err, verb, &format!("{e:?}")),
        }
    }
    let request = match verb {
        "soak" => Request::Soak(start),
        "sweep" => Request::Sweep(start),
        _ => Request::Bench(start),
    };
    let _ = writeln!(
        out,
        "raul scene {verb}: results under {}",
        s.run_dir.display()
    );
    go(&request, f.tool.as_ref(), out, err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start() -> Start {
        Start {
            viewer: "/v".into(),
            game: "/game".into(),
            places: "/p.toml".into(),
            out: "/out".into(),
            variant: "a".into(),
            overlays: vec!["/m1".into(), "/m2".into()],
            plugins: vec!["A.esm".into(), "B.esp".into()],
            label: "raul-scene-run-P-a".into(),
            lock: Some("/l".into()),
            ..Start::default()
        }
    }

    #[test]
    fn a_run_names_its_overlays_in_one_variant_and_its_plugins_in_load_order() {
        let a = tool_args(&Request::Run(start())).join(" ");
        assert!(
            a.starts_with("run --viewer /v --game /game --places /p.toml --out /out"),
            "{a}"
        );
        assert!(a.contains("--plugins A.esm,B.esp"), "{a}");
        assert!(a.contains("--variant a=/m1+/m2"), "{a}");
        assert!(a.contains("--label raul-scene-run-P-a --who raul"), "{a}");
        assert!(a.contains("--game-lock /l"), "{a}");
        assert!(!a.contains("--no-cap"), "{a}");
    }

    #[test]
    fn a_variant_without_overlays_is_the_game_alone_and_no_cap_is_passed_on() {
        let mut s = start();
        s.overlays.clear();
        s.no_cap = true;
        let a = tool_args(&Request::Run(s)).join(" ");
        assert!(a.contains("--variant a "), "{a}");
        assert!(a.contains("--no-cap"), "{a}");
    }

    #[test]
    fn a_bench_takes_the_lock_exclusively_by_name_and_the_baselines() {
        let mut s = start();
        s.suite = "quick".into();
        s.baseline = Some("mine".into());
        let a = tool_args(&Request::Bench(s)).join(" ");
        assert!(a.starts_with("bench "), "{a}");
        assert!(
            a.contains("--suite quick") && a.contains("--lock /l") && a.contains("--baseline mine"),
            "{a}"
        );
        assert!(!a.contains("--game-lock"), "{a}");
    }

    #[test]
    fn compare_puts_the_noise_run_before_the_two_folders() {
        let a = tool_args(&Request::Compare {
            a: "/a".into(),
            b: "/b".into(),
            noise: Some("/n".into()),
        });
        assert_eq!(a, ["compare", "--noise", "/n", "/a", "/b"]);
    }

    #[test]
    fn sources_are_the_three_words_or_a_folder_that_exists() {
        assert_eq!(Source::parse("game"), Ok(Source::Game));
        assert_eq!(Source::parse("profile"), Ok(Source::Profile));
        assert!(Source::parse("/no/such/folder").is_err());
        assert!(matches!(Source::parse("."), Ok(Source::Folder(_))));
    }

    #[test]
    fn the_built_in_place_files_hold_only_places_and_the_night_one_sets_the_hour() {
        for (name, text) in BUILT_IN_PLACES {
            assert!(
                text.starts_with("# SPDX-License-Identifier: GPL-3.0-only"),
                "{name}"
            );
            assert_eq!(text.matches("[[place]]").count(), 4, "{name}");
            assert_eq!(
                text.matches("hour = 22.5").count(),
                if name.ends_with("night") { 4 } else { 0 },
                "{name}"
            );
        }
    }
}
