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
        let mut play = spec.clone();
        play.args.insert(0, PLAY_MODE_ARG.to_owned());
        // A file left from an earlier run would stop this one at once.
        let stop = stop_file(&spec.working_dir);
        match std::fs::remove_file(&stop) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("the old stop request could not be cleared: {e}")),
        }
        play.env
            .insert(STOP_FILE_ENV.to_owned(), stop.to_string_lossy().into_owned());
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

/// `raul run ...` with the play mode: `rest` is the command line with `run` first.
pub fn run(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let Ok(exe) = std::env::current_exe() else {
        let _ = writeln!(err, "raul: run: this program's own path is not known");
        return crate::FAILED;
    };
    let args = with_viewer(&rest[1..], &exe);
    let launcher = PlayLauncher::new(&exe, &nvmod_run::ProcessLauncher);
    nvmod_cli::run_with_launcher(&args, out, err, &launcher)
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
