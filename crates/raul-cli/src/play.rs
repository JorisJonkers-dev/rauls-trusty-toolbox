// SPDX-License-Identifier: GPL-3.0-only
//! `raul run` for a build that links the engine: the game is a child of raul's own binary in the play mode
//! (ADR-0016), not a separate engine program.
//!
//! nvmod plans and prepares the run as it always did (the Staging tree, the Instance lock, the log header) and hands
//! the start of the program to [`PlayLauncher`], which starts `raul --play-mode <DATA> [arguments]` instead of the
//! program nvmod found. Only the full `raul` calls [`enable`]; the CLI-only build keeps the program nvmod finds.

use nvmod_run::{LaunchSpec, Launcher, Running};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// The reserved first argument of the play mode.
pub const PLAY_MODE_ARG: &str = "--play-mode";

/// The environment variable that names the stop file (the engine's `fork_play::STOP_FILE_ENV`).
pub const STOP_FILE_ENV: &str = "NV_STOP_FILE";

/// The name of the stop file inside the run folder.
pub const STOP_FILE_NAME: &str = "play.stop";

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Makes `raul run` start the game in the play mode of this binary. The full `raul` calls it at start when it links
/// the engine.
pub fn enable() {
    ENABLED.store(true, Ordering::SeqCst);
}

/// Whether [`enable`] was called.
#[must_use]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

/// The stop file of a run whose folder is `run_dir`.
#[must_use]
pub fn stop_file(run_dir: &Path) -> PathBuf {
    run_dir.join(STOP_FILE_NAME)
}

/// Asks the running game to stop between two frames: the engine's watcher sees the file and ends the game with
/// status 143 after its exit hooks and the pipeline-cache flush.
///
/// # Errors
/// The file could not be created.
pub fn request_stop(run_dir: &Path) -> std::io::Result<()> {
    std::fs::write(stop_file(run_dir), b"stop")
}

/// Starts the programs of `raul run` through raul's own runner (`raul-runner`), so the Toolbox does not use the
/// launcher of nvmod-run: the game in the play mode (see [`PlayLauncher`]) and the person's other programs.
#[derive(Debug, Default, Clone, Copy)]
pub struct RaulLauncher;

struct Child(raul_runner::Started);

impl Running for Child {
    fn pid(&self) -> u32 {
        self.0.pid()
    }

    fn wait(self: Box<Self>) -> Result<i32, String> {
        self.0.wait()
    }
}

impl Launcher for RaulLauncher {
    fn start(&self, spec: &LaunchSpec, attached: bool) -> Result<Box<dyn Running>, String> {
        raul_runner::start_program(
            &spec.program,
            &spec.args,
            &spec.env,
            &spec.working_dir,
            attached,
            spec.log.as_deref(),
        )
        .map(|started| Box::new(Child(started)) as Box<dyn Running>)
    }
}

/// Starts raul's own binary in the play mode when nvmod asks for the viewer it was given; any other program is
/// started as nvmod asked.
pub struct PlayLauncher<'a> {
    exe: String,
    inner: &'a dyn Launcher,
}

impl<'a> PlayLauncher<'a> {
    /// A launcher for the binary at `exe`, starting through `inner`.
    #[must_use]
    pub fn new(exe: &Path, inner: &'a dyn Launcher) -> Self {
        Self {
            exe: exe.to_string_lossy().into_owned(),
            inner,
        }
    }
}

impl Launcher for PlayLauncher<'_> {
    fn start(&self, spec: &LaunchSpec, attached: bool) -> Result<Box<dyn Running>, String> {
        if spec.program != self.exe {
            return self.inner.start(spec, attached);
        }
        // Tale of Two Wastelands needs its values in the Profile's Fallout.ini; a failure is said, not fatal.
        match raul_core::game_ini::sync_run_folder(&spec.working_dir) {
            Ok(raul_core::game_ini::Outcome::Changed(said)) => {
                eprintln!(
                    "raul: run: game settings for this Profile: {}",
                    said.join(", ")
                );
            }
            Ok(_) => {}
            Err(why) => eprintln!("raul: run: the game settings could not be checked: {why}"),
        }
        let mut play = spec.clone();
        play.args.insert(0, PLAY_MODE_ARG.to_owned());
        // A file left from an earlier run would stop this one at once.
        let stop = stop_file(&spec.working_dir);
        match std::fs::remove_file(&stop) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("the old stop request could not be cleared: {e}")),
        }
        play.env.insert(
            STOP_FILE_ENV.to_owned(),
            stop.to_string_lossy().into_owned(),
        );
        self.inner.start(&play, attached)
    }
}

/// Whether `args` (the words after `run`) start the game natively: no Proton and no program of the person's own.
fn native_game(args: &[String]) -> bool {
    let value_flags = ["--instance", "--profile", "--proton", "--viewer"];
    let mut positional = None;
    let mut skip = false;
    for a in args.iter().take_while(|a| a.as_str() != "--") {
        if skip {
            skip = false;
        } else if value_flags.contains(&a.as_str()) {
            skip = true;
        } else if !a.starts_with("--") && positional.is_none() {
            positional = Some(a.as_str());
        }
    }
    matches!(positional, Some("game" | "nv-rs"))
        && !args.iter().any(|a| a == "--via-proton" || a == "--viewer")
}

/// The words after `run` with this binary named as the viewer, when the run is the native game.
#[must_use]
pub fn with_viewer(args: &[String], exe: &Path) -> Vec<String> {
    if !native_game(args) {
        return args.to_vec();
    }
    let mut out = vec!["--viewer".to_owned(), exe.to_string_lossy().into_owned()];
    out.extend(args.iter().cloned());
    out
}

/// What a plain person is told about how the game ended, by the exit code it left (the engine's documented
/// classes, `fork_play::ExitClass`). A code is only a hint: the game's own code can equal one of these, so the log has
/// the last word; `asked_stop` says the stop file was asked for.
#[must_use]
pub fn exit_words(code: i32, asked_stop: bool) -> Option<&'static str> {
    match code {
        0 => None,
        1 => Some("the game could not start; check the game folder"),
        2 => Some("the game did not understand how it was started"),
        3 => Some(
            "the game stopped because it ran into something it cannot do yet; the log says what",
        ),
        69 => Some("this computer refused to start the game; check the graphics driver"),
        143 if asked_stop => Some("the game was stopped as asked"),
        143 => Some("the game was stopped"),
        _ => Some("the game ended unexpectedly; the log says why"),
    }
}

/// `raul run --stop --instance <dir> [--profile <name>] [--json]`: asks a running game to stop between two frames.
/// Without a profile every run folder of the Instance is asked, since only one game runs at a time.
pub fn stop(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let Some(instance) = value("--instance") else {
        let _ = writeln!(
            err,
            "usage: raul run --stop --instance <dir> [--profile <name>] [--json]"
        );
        return crate::USAGE;
    };
    let runs = Path::new(&instance).join("run");
    let folders: Vec<PathBuf> = match value("--profile") {
        Some(profile) => vec![runs.join(profile)],
        None => std::fs::read_dir(&runs).map_or_else(
            |_| Vec::new(),
            |d| {
                d.filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect()
            },
        ),
    };
    let mut asked = Vec::new();
    for folder in folders.iter().filter(|f| f.is_dir()) {
        if let Err(e) = request_stop(folder) {
            let _ = writeln!(err, "raul: run: the stop could not be asked for: {e}");
            return crate::FAILED;
        }
        asked.push(folder.to_string_lossy().into_owned());
    }
    if args.iter().any(|a| a == "--json") {
        let _ = writeln!(out, "{}", serde_json::json!({ "stop_asked": asked }));
    } else if asked.is_empty() {
        let _ = writeln!(out, "no game was started from this Instance");
    } else {
        let _ = writeln!(out, "asked the game to stop");
    }
    crate::OK
}

/// `raul run ...` for a build without the engine, or a program of the person's own: nvmod plans and prepares the run,
/// raul's runner starts the program. `args` is the command line with `run` first; `log` writes the run log as every
/// nvmod command does.
pub fn run_plain(args: &[String], log: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    if !log {
        return nvmod_cli::run_with_launcher(&args[1..], out, err, &RaulLauncher);
    }
    match nvmod_cli::begin_run_log(args) {
        Ok((rest, session)) => {
            let code = nvmod_cli::run_with_launcher(&rest[1..], out, err, &RaulLauncher);
            session.finish(code);
            code
        }
        Err(why) => {
            let _ = writeln!(err, "nvmod: {why}");
            crate::USAGE
        }
    }
}

/// `raul run ...` with the play mode: `rest` is the command line with `run` first.
pub fn run(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let Ok(exe) = std::env::current_exe() else {
        let _ = writeln!(err, "raul: run: this program's own path is not known");
        return crate::FAILED;
    };
    let args = with_viewer(&rest[1..], &exe);
    let launcher = PlayLauncher::new(&exe, &RaulLauncher);
    let code = nvmod_cli::run_with_launcher(&args, out, err, &launcher);
    if args.iter().any(|a| a == "--wait") {
        let asked = args
            .iter()
            .position(|a| a == "--instance")
            .and_then(|i| args.get(i + 1))
            .is_some_and(|i| {
                std::fs::read_dir(Path::new(i).join("run")).is_ok_and(|d| {
                    d.filter_map(Result::ok)
                        .any(|e| stop_file(&e.path()).exists())
                })
            });
        if let Some(words) = exit_words(i32::from(code), asked) {
            let _ = writeln!(err, "raul: run: {words}");
        }
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    struct Idle;
    impl Running for Idle {
        fn pid(&self) -> u32 {
            0
        }
        fn wait(self: Box<Self>) -> Result<i32, String> {
            Ok(0)
        }
    }

    #[derive(Default)]
    struct Recording(Mutex<Vec<LaunchSpec>>);
    impl Launcher for Recording {
        fn start(&self, spec: &LaunchSpec, _attached: bool) -> Result<Box<dyn Running>, String> {
            self.0.lock().unwrap().push(spec.clone());
            Ok(Box::new(Idle))
        }
    }

    fn spec(program: &str, dir: &Path) -> LaunchSpec {
        LaunchSpec {
            program: program.to_owned(),
            args: words("/run/folder -x"),
            env: std::collections::BTreeMap::new(),
            working_dir: dir.to_path_buf(),
            log: None,
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("raul-play-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_viewer_is_started_as_this_binary_in_the_play_mode_with_a_stop_file() {
        let dir = scratch("viewer");
        let inner = Recording::default();
        PlayLauncher::new(Path::new("/opt/raul"), &inner)
            .start(&spec("/opt/raul", &dir), false)
            .unwrap();
        let started = inner.0.lock().unwrap();
        assert_eq!(started[0].program, "/opt/raul");
        assert_eq!(started[0].args, words("--play-mode /run/folder -x"));
        assert_eq!(
            started[0].env[STOP_FILE_ENV],
            stop_file(&dir).to_string_lossy()
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_profile_with_tale_of_two_wastelands_gets_its_game_settings_before_the_game_starts() {
        let root = scratch("ttw-ini");
        let run = root.join("run/Default");
        std::fs::create_dir_all(&run).unwrap();
        std::fs::create_dir_all(root.join("profiles/Default")).unwrap();
        std::fs::write(
            root.join("profiles/Default/plugins.txt"),
            "*TaleOfTwoWastelands.esm\n",
        )
        .unwrap();
        std::fs::write(
            run.join("Fallout_default.ini"),
            "[General]\nSCharGenQuest=00102037\n",
        )
        .unwrap();
        let inner = Recording::default();
        PlayLauncher::new(Path::new("/opt/raul"), &inner)
            .start(&spec("/opt/raul", &run), false)
            .unwrap();
        let ini = nvmod_run::my_games(&run.join("userdata")).join("Fallout.ini");
        assert!(
            std::fs::read_to_string(ini)
                .unwrap()
                .contains("SCharGenQuest=001FFFF8")
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn another_program_is_started_as_asked() {
        let dir = scratch("other");
        let inner = Recording::default();
        PlayLauncher::new(Path::new("/opt/raul"), &inner)
            .start(&spec("/usr/bin/tool", &dir), false)
            .unwrap();
        let started = inner.0.lock().unwrap();
        assert_eq!(started[0].args, words("/run/folder -x"));
        assert!(started[0].env.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_old_stop_request_is_cleared_at_start_and_a_new_one_is_a_file() {
        let dir = scratch("stop");
        request_stop(&dir).unwrap();
        assert!(stop_file(&dir).is_file());
        let inner = Recording::default();
        PlayLauncher::new(Path::new("/opt/raul"), &inner)
            .start(&spec("/opt/raul", &dir), true)
            .unwrap();
        assert!(!stop_file(&dir).exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn raul_s_own_launcher_starts_a_program_waits_for_it_and_logs_its_output() {
        let dir = scratch("launcher");
        let log = dir.join("run.log");
        let me = std::env::current_exe().unwrap();
        let spec = LaunchSpec {
            program: me.to_string_lossy().into_owned(),
            args: words("--list"),
            env: std::collections::BTreeMap::new(),
            working_dir: dir.clone(),
            log: Some(log.clone()),
        };
        let running = RaulLauncher.start(&spec, false).unwrap();
        assert!(running.pid() > 0);
        assert_eq!(running.wait(), Ok(0));
        assert!(std::fs::read_to_string(&log).unwrap().contains("tests"));
        let missing = LaunchSpec {
            program: dir.join("nothing").to_string_lossy().into_owned(),
            ..spec
        };
        assert!(RaulLauncher.start(&missing, true).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn exit_codes_have_plain_words_and_a_finished_game_has_none() {
        assert_eq!(exit_words(0, false), None);
        for code in [1, 2, 3, 69, 143, 77] {
            let words = exit_words(code, false).unwrap();
            assert!(
                !words.contains("exit") && !words.contains("code"),
                "{words}"
            );
        }
        assert_ne!(exit_words(143, true), exit_words(143, false));
    }

    #[test]
    fn stop_asks_every_run_folder_or_the_named_one() {
        let inst = scratch("stopcmd");
        for p in ["Default", "Other"] {
            std::fs::create_dir_all(inst.join("run").join(p)).unwrap();
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let i = inst.to_string_lossy().into_owned();
        let args = words(&format!("--stop --instance {i} --profile Other --json"));
        assert_eq!(stop(&args, &mut out, &mut err), crate::OK);
        assert!(stop_file(&inst.join("run/Other")).is_file());
        assert!(!stop_file(&inst.join("run/Default")).exists());
        let args = words(&format!("--stop --instance {i}"));
        assert_eq!(stop(&args, &mut out, &mut err), crate::OK);
        assert!(stop_file(&inst.join("run/Default")).is_file());
        assert_eq!(stop(&words("--stop"), &mut out, &mut err), crate::USAGE);
        std::fs::remove_dir_all(&inst).unwrap();
    }

    #[test]
    fn only_the_native_game_names_this_binary_as_the_viewer() {
        let exe = Path::new("/opt/raul");
        assert_eq!(
            with_viewer(&words("game --instance /i"), exe),
            words("--viewer /opt/raul game --instance /i")
        );
        assert_eq!(
            with_viewer(&words("--instance /i nv-rs"), exe)[..2],
            words("--viewer /opt/raul")[..]
        );
        for same in [
            "game --instance /i --via-proton",
            "game --instance /i --viewer /x/nv-viewer",
            "LOOT --instance /i",
            "/some/tool --instance /i",
            "--instance game",
        ] {
            assert_eq!(with_viewer(&words(same), exe), words(same), "{same}");
        }
    }
}
