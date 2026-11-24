// SPDX-License-Identifier: GPL-3.0-only
//! `raul scene` and `raul bench game`: the engine's picture, soak, sweep and bench tool, reached through the Profile.
//!
//! raul resolves what the bare tool wants by hand: the game folder (`raul config`), the Profile's Mods laid over it in
//! priority order, its active Plugins in Load order, the output folder under raul's data folder, the label and the
//! memory cap of the game start. The work itself is the engine's scene library (`fork_scene::tool`), linked into raul
//! and called in this process; only the game is a child process, and that is raul's own binary in the play mode (see
//! [`SceneLauncher`]).

use std::io::Write;
use std::path::{Path, PathBuf};

use fork_scene::tool;
use nvmod_instance::NvmodInstance;
use raul_core::config::{Config, Key};
use raul_core::context::Context;

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
    /// The viewer is raul's own binary: it is started with the play-mode word first.
    pub play_mode: bool,
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
    /// Where the card's memory is read for the start check; `None`: the engine's default (`/sys/class/drm`).
    pub gpu_sysfs: Option<PathBuf>,
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

/// Runs one request in this process, through the engine's scene library, and writes what the person should read to
/// `out`. Returns the exit code (0 fine, 1 the run ran and found a problem).
///
/// # Errors
/// Why the request could not be carried out at all.
fn execute(request: &Request, out: &mut dyn Write, err: &mut dyn Write) -> Result<i32, String> {
    // What the library says while it works (the memory cap, a start's memory, a hang) goes to the person at once.
    let say = |line: &str| {
        let _ = writeln!(std::io::stderr(), "{line}");
    };
    let ok = |ok: bool| i32::from(!ok);
    match request {
        Request::Run(s) => {
            let launcher = SceneLauncher::new(s);
            let mut req = tool::RunRequest::new(lib_start(s));
            req.variants = vec![tool::Variant {
                name: s.variant.clone(),
                overlays: s.overlays.clone(),
                env: Vec::new(),
            }];
            req.jobs = 1;
            if !s.resolution.is_empty() {
                req.resolution.clone_from(&s.resolution);
            }
            req.quick = s.quick;
            let r = tool::run(&req, &launcher, &say)?;
            let _ = write!(out, "{}", r.summary());
            Ok(ok(r.ok))
        }
        Request::Compare { a, b, noise } => {
            let r = tool::compare(&tool::CompareRequest {
                a: a.clone(),
                b: b.clone(),
                noise: noise.clone(),
                ..tool::CompareRequest::default()
            })?;
            let _ = writeln!(out, "{}", r.summary());
            Ok(ok(r.ok))
        }
        Request::Soak(s) => {
            let launcher = SceneLauncher::new(s);
            let r = tool::soak(&soak_request(s), &launcher, &say)?;
            let _ = write!(out, "{}", r.summary());
            Ok(ok(r.ok))
        }
        Request::Sweep(s) => {
            let launcher = SceneLauncher::new(s);
            let req = tool::SweepRequest {
                soak: sweep_soak(s),
            };
            let r = tool::sweep(&req, &launcher, &say)?;
            let _ = writeln!(out, "{}", r.markdown);
            let _ = writeln!(out, "sweep report: {}", r.report.display());
            Ok(ok(r.ok))
        }
        Request::Bench(s) => {
            let Some(lock) = s.lock.clone() else {
                return Err(
                    "a bench needs the game lock file (--game-lock, or a data folder for raul)"
                        .into(),
                );
            };
            let launcher = SceneLauncher::new(s);
            let mut req = tool::BenchRequest::new(lib_start(s), lock);
            req.soak = soak_request(s);
            req.soak.realtime = true;
            req.soak.exclusive = true;
            req.soak.passes = Some(5);
            req.suite = Some(s.suite.clone()).filter(|n| !n.is_empty());
            req.baseline.clone_from(&s.baseline);
            req.save_baseline.clone_from(&s.save_baseline);
            let r = tool::bench(&req, &launcher, &say)?;
            let _ = writeln!(out, "{}", r.markdown);
            let _ = writeln!(out, "summary: {}", r.summary.display());
            Ok(ok(r.ok))
        }
        Request::BenchCompare { a, b } => {
            let r = tool::bench_compare(&tool::BenchCompareRequest {
                a: a.clone(),
                b: b.clone(),
            })?;
            let _ = writeln!(out, "{}", r.summary());
            Ok(0)
        }
    }
    .inspect(|_| {
        let _ = err.flush();
    })
}

/// The library's start for what raul resolved.
fn lib_start(s: &Start) -> tool::Start {
    let mut start = tool::Start::new(
        s.viewer.clone(),
        s.game.clone(),
        s.places.clone(),
        s.out.clone(),
    );
    start.plugins.clone_from(&s.plugins);
    start.only.clone_from(&s.only);
    if s.grids > 0 {
        start
            .ini
            .push(("uGridsToLoad".to_owned(), s.grids.to_string()));
    }
    start.game_lock.clone_from(&s.lock);
    if let Some(root) = &s.gpu_sysfs {
        start.gpu_sysfs.clone_from(root);
    }
    start
}

/// The soak the request describes: the frames, and the one data folder laid over the game.
fn soak_request(s: &Start) -> tool::SoakRequest {
    let mut req = tool::SoakRequest::new(lib_start(s));
    if s.frames > 0 {
        req.frames = s.frames;
    }
    req.overlay = s.overlays.first().cloned();
    req
}

/// The soak of a sweep: the sweep's own defaults (300 frames, a shorter timeout, a stall limit) with the same changes.
fn sweep_soak(s: &Start) -> tool::SoakRequest {
    let mut req = tool::SweepRequest::new(lib_start(s)).soak;
    if s.frames > 0 {
        req.frames = s.frames;
    }
    req.overlay = s.overlays.first().cloned();
    req
}

// ---------------------------------------------------------------------------------------------------------------
// The game child: raul's own binary in the play mode.

/// Starts the game for the engine's tool: `raul --play-mode <folder>` (the engine is linked into raul, ADR-0016), or
/// the program the person named with `--viewer`, through raul's runner, with the run's label and memory cap.
struct SceneLauncher {
    cap: tool::Cap,
    /// The viewer is raul itself in the play mode: its first argument is the play-mode word.
    play_mode: bool,
}

impl SceneLauncher {
    fn new(s: &Start) -> SceneLauncher {
        SceneLauncher {
            cap: tool::Cap {
                label: s.label.clone(),
                who: "raul".to_owned(),
                mem: s.mem.clone(),
                off: s.no_cap,
            },
            play_mode: s.play_mode,
        }
    }
}

/// Whether a program of this name is in a folder of the path.
fn on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file()))
}

impl tool::Launcher for SceneLauncher {
    fn describe(&self, concurrent: usize) -> String {
        self.cap.describe(&[], concurrent)
    }

    fn launch(&self, spec: &tool::StartSpec, log: std::fs::File) -> Result<tool::Launched, String> {
        use std::ffi::OsString;
        let mut words: Vec<OsString> = Vec::new();
        let mut scope_prefix = None;
        // The lock and the cap are programs of the Unix desktop the engine runs on; elsewhere the start has neither.
        if cfg!(unix) {
            // macOS ships no `flock`: a start there takes no lock.
            if let Some(lock) = spec.lock.as_ref().filter(|_| on_path("flock")) {
                words.push("flock".into());
                words.push(if lock.exclusive { "-x" } else { "-s" }.into());
                words.push((&lock.path).into());
            }
            let cap = self
                .cap
                .words(&spec.kind, &spec.name, &[], spec.concurrent)?;
            scope_prefix = cap.get(1).map(|label| format!("nvgame-{label}-"));
            words.extend(cap.into_iter().map(OsString::from));
        }
        words.push((&spec.viewer).into());
        if self.play_mode {
            words.push(crate::play::PLAY_MODE_ARG.into());
        }
        words.push((&spec.game_dir).into());
        let child = raul_runner::start_logged(&words, &spec.env, &spec.game_dir, log)?;
        Ok(tool::Launched {
            child,
            scope_prefix,
        })
    }
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
    lock: Option<PathBuf>,
    gpu_sysfs: Option<PathBuf>,
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
            "--game-lock" => f.lock = Some(PathBuf::from(value()?)),
            "--gpu-sysfs" => f.gpu_sysfs = Some(PathBuf::from(value()?)),
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
fn go(request: &Request, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let result = execute(request, out, err);
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

/// Where one run writes: `--out`, or a folder named by the Profile and the second under raul's data folder.
fn run_folder(f: &Flags, who: &str) -> Result<PathBuf, String> {
    let stamp = {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let minute = raul_drivers::lod::utc_minute(secs)
            .replace(" UTC", "")
            .replace(' ', "T")
            .replace(':', "");
        format!("{minute}{:02}Z", secs % 60)
    };
    let run_dir = match &f.out {
        Some(o) => std::path::absolute(o).map_err(|e| format!("{}: {e}", o.display()))?,
        None => raul_core::data::folder()
            .ok_or("raul has no data folder here (set RAUL_DATA_DIR to an absolute folder, or pass --out)")?
            .join("scene")
            .join(who)
            .join(&stamp),
    };
    // Two runs in the same second get folders of their own.
    let run_dir = if f.out.is_none() && run_dir.exists() {
        (2..100)
            .map(|n| PathBuf::from(format!("{}-{n}", run_dir.display())))
            .find(|p| !p.exists())
            .ok_or("raul has run too many scenes in this second")?
    } else {
        run_dir
    };
    Ok(run_dir)
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
    // The game an Instance plays is its base copy, as for `raul run`; the setting is only for runs without one.
    let game = f
        .game
        .clone()
        .or_else(|| instance.as_ref().map(|(i, _)| i.config.base.clone()))
        .or_else(|| {
            ctx.config_file()
                .and_then(|c| Config::load(&c).ok())
                .and_then(|c| c.get(Key::GameNewVegas))
        })
        .ok_or("no game folder: pass --game <dir> or --instance <dir>, or run `raul config set game.newvegas <dir>`")?;
    let who = instance
        .as_ref()
        .map_or("game".to_owned(), |(_, p)| p.clone());
    let run_dir = run_folder(f, &who)?;
    let places = match &f.places {
        Some(w) => places_file(w, &run_dir)?,
        None if verb == "bench" => PathBuf::new(),
        None => {
            return Err(
                "--places <file> is needed (a place file, or mojave-day / mojave-night)".into(),
            );
        }
    };
    // The game child is raul's own binary in the play mode, unless the person names a program.
    let (viewer, play_mode) = match &f.viewer {
        Some(v) => (v.clone(), false),
        None if crate::play::enabled() => {
            (std::env::current_exe().map_err(|e| e.to_string())?, true)
        }
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
            play_mode,
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
            gpu_sysfs: f.gpu_sysfs.clone(),
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
        let _ = writeln!(out, "raul scene run: {} ...", r.name);
        let c = go(&Request::Run(start), out, err);
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
    go(&request, out, err)
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
    fn the_library_start_carries_the_plugins_in_load_order_the_grids_and_the_lock() {
        let mut s = start();
        s.grids = 7;
        s.only = vec!["goodsprings".into()];
        let l = lib_start(&s);
        assert_eq!(l.plugins, ["A.esm", "B.esp"]);
        assert_eq!(l.only, ["goodsprings"]);
        assert_eq!(l.ini, [("uGridsToLoad".to_owned(), "7".to_owned())]);
        assert_eq!(l.game_lock, Some(PathBuf::from("/l")));
        assert_eq!(
            (l.viewer, l.game, l.places, l.out),
            ("/v".into(), "/game".into(), "/p.toml".into(), "/out".into())
        );
    }

    #[test]
    fn a_soak_takes_the_first_data_folder_and_the_frames_and_a_sweep_keeps_its_own_defaults() {
        let mut s = start();
        s.frames = 40;
        let soak = soak_request(&s);
        assert_eq!((soak.frames, soak.overlay), (40, Some("/m1".into())));
        s.frames = 0;
        assert_eq!(soak_request(&s).frames, 600);
        assert_eq!(sweep_soak(&s).frames, 300);
    }

    #[test]
    fn the_cap_follows_the_label_the_memory_limit_and_no_cap() {
        let mut s = start();
        s.mem = "10G".into();
        let l = SceneLauncher::new(&s);
        assert_eq!(l.cap.label, "raul-scene-run-P-a");
        assert_eq!(l.cap.who, "raul");
        assert!(tool::Launcher::describe(&l, 1).contains("10G"));
        s.no_cap = true;
        assert!(tool::Launcher::describe(&SceneLauncher::new(&s), 1).contains("OFF"));
    }

    #[cfg(unix)]
    #[test]
    fn the_launcher_starts_the_play_mode_of_the_viewer_in_the_game_folder_with_the_environment() {
        use std::os::unix::fs::PermissionsExt;
        let dir = nvmod_testkit::TempDir::new("scene-launcher");
        let viewer = dir.path().join("viewer.sh");
        std::fs::write(&viewer, "#!/bin/sh\necho \"$@\" \"$MARK\" $(pwd)\n").unwrap();
        std::fs::set_permissions(&viewer, std::fs::Permissions::from_mode(0o755)).unwrap();
        let game = dir.path().join("game");
        std::fs::create_dir_all(&game).unwrap();
        let mut s = start();
        s.no_cap = true;
        s.play_mode = true;
        let spec = tool::StartSpec {
            kind: "run".into(),
            name: "a".into(),
            viewer,
            game_dir: game.clone(),
            env: vec![("MARK".into(), "marked".into())],
            concurrent: 1,
            lock: None,
        };
        let log = dir.path().join("log");
        let mut launched = tool::Launcher::launch(
            &SceneLauncher::new(&s),
            &spec,
            std::fs::File::create(&log).unwrap(),
        )
        .unwrap();
        assert!(launched.child.wait().unwrap().success());
        let line = std::fs::read_to_string(&log).unwrap();
        // The child's working folder is printed by `pwd`, which resolves links (macOS keeps temp folders under /var,
        // a link to /private/var).
        let real = game.canonicalize().unwrap();
        assert_eq!(
            line.trim(),
            format!("--play-mode {} marked {}", game.display(), real.display())
        );
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
