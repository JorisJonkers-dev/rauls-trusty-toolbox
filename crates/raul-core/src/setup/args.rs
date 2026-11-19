// SPDX-License-Identifier: GPL-3.0-only
//! The words of `raul setup` as a typed [`Request`]: the options `nvmod setup` took, read the same way, so a command
//! line an old setup recorded (and a script) still reads. The words are the one place a command line is parsed; the
//! job itself only ever sees the [`Request`].

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{EngineOptions, Request, Ttw};

/// The options that take a value, as `--name value` or `--name=value`.
const OPTIONS: &[&str] = &[
    "base",
    "instance",
    "masterlist",
    "profile",
    "import-mo2",
    "mod",
    "answers",
    "fallout3",
    "package",
    "timeout-secs",
    "source",
    "cargo",
    "release-url",
    "asset",
    "repo",
    "gh",
    "curl",
    "retry-delay-ms",
    "catalog",
];

/// Options a setup recorded by `nvmod setup` may carry that no longer mean anything: they are read and ignored
/// (the extraction tool is gone, TTW runs in process with no installer, lock or store).
const IGNORED: &[&str] = &["extract-tool", "installer", "lock", "store"];

/// The switches.
const FLAGS: &[&str] = &[
    "json",
    "dry-run",
    "release",
    "skip-nv-rs",
    "sort",
    "copy",
    "keep-partial",
    "resume",
    "restart",
    "yes",
];

/// The longest `--timeout-secs` taken: a week.
const MAX_TIMEOUT_SECS: u64 = 7 * 24 * 60 * 60;

/// What the words say.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // One bool per switch of the command line.
pub struct Parsed {
    /// The request the options make. `instance`, `base` and `masterlist` are empty when the words leave them out.
    pub request: Request,
    /// Whether `--instance` was given.
    pub has_instance: bool,
    /// Whether `--base` was given.
    pub has_base: bool,
    /// Whether `--masterlist` was given.
    pub has_masterlist: bool,
    /// Whether any option or switch of a first run (anything but `--instance`, `--json`, `--yes`) was given.
    pub has_run_options: bool,
    /// `--json`.
    pub json: bool,
    /// `--dry-run`.
    pub dry_run: bool,
    /// `--keep-partial`.
    pub keep_partial: bool,
    /// `--resume`.
    pub resume: bool,
    /// `--restart`.
    pub restart: bool,
    /// `--yes`.
    pub yes: bool,
}

fn absolute(value: &str) -> PathBuf {
    std::path::absolute(value).unwrap_or_else(|_| Path::new(value).to_owned())
}

/// Read the words after `raul setup`.
///
/// # Errors
/// What is wrong with them, in plain words: an option that is not known, a value that is missing or not a number, or
/// a word that is no option.
#[allow(clippy::too_many_lines)] // one arm per option of the command line
pub fn parse(words: &[String]) -> Result<Parsed, String> {
    let mut request = Request {
        base: PathBuf::new(),
        instance: PathBuf::new(),
        masterlist: PathBuf::new(),
        profile: None,
        import_mo2: None,
        copy: false,
        engine: true,
        engine_options: EngineOptions::default(),
        ttw: None,
        mods: Vec::new(),
        answers: None,
        sort: false,
        catalog: None,
        done: Vec::new(),
    };
    let mut parsed = Parsed {
        request: request.clone(),
        has_instance: false,
        has_base: false,
        has_masterlist: false,
        has_run_options: false,
        json: false,
        dry_run: false,
        keep_partial: false,
        resume: false,
        restart: false,
        yes: false,
    };
    let mut fallout3: Option<PathBuf> = None;
    let mut package: Option<PathBuf> = None;
    let mut timeout: Option<Duration> = None;
    let mut rest = words.iter();
    while let Some(word) = rest.next() {
        let Some(name) = word.strip_prefix("--") else {
            return Err(format!("unexpected word {word}"));
        };
        let (name, inline) = match name.split_once('=') {
            Some((n, v)) => (n, Some(v.to_owned())),
            None => (name, None),
        };
        if FLAGS.contains(&name) && inline.is_none() {
            match name {
                "json" => parsed.json = true,
                "dry-run" => parsed.dry_run = true,
                "keep-partial" => parsed.keep_partial = true,
                "resume" => parsed.resume = true,
                "restart" => parsed.restart = true,
                "yes" => parsed.yes = true,
                "release" => request.engine_options.release = true,
                "skip-nv-rs" => request.engine = false,
                "sort" => request.sort = true,
                "copy" => request.copy = true,
                _ => {}
            }
            if !matches!(name, "json" | "yes" | "resume" | "restart") {
                parsed.has_run_options = true;
            }
            continue;
        }
        if !OPTIONS.contains(&name) && !IGNORED.contains(&name) {
            return Err(format!("unknown option --{name}"));
        }
        let value = match inline {
            Some(v) => v,
            None => rest
                .next()
                .cloned()
                .ok_or_else(|| format!("--{name} needs a value"))?,
        };
        if name != "instance" {
            parsed.has_run_options = true;
        }
        match name {
            "base" => {
                request.base = absolute(&value);
                parsed.has_base = true;
            }
            "instance" => {
                request.instance = absolute(&value);
                parsed.has_instance = true;
            }
            "masterlist" => {
                request.masterlist = absolute(&value);
                parsed.has_masterlist = true;
            }
            "profile" => request.profile = Some(value),
            "import-mo2" => request.import_mo2 = Some(absolute(&value)),
            "mod" => request.mods.push(absolute(&value)),
            "answers" => request.answers = Some(absolute(&value)),
            "fallout3" => fallout3 = Some(absolute(&value)),
            "package" => package = Some(absolute(&value)),
            "timeout-secs" => {
                let secs = value
                    .parse::<u64>()
                    .ok()
                    .filter(|s| (1..=MAX_TIMEOUT_SECS).contains(s))
                    .ok_or_else(|| {
                        format!(
                            "--timeout-secs needs a number of seconds from 1 to {MAX_TIMEOUT_SECS}"
                        )
                    })?;
                timeout = Some(Duration::from_secs(secs));
            }
            "source" => request.engine_options.source = Some(absolute(&value)),
            "cargo" => request.engine_options.cargo = Some(absolute(&value)),
            "release-url" => request.engine_options.release_url = value,
            "asset" => request.engine_options.asset = value,
            "repo" => request.engine_options.repo = value,
            "gh" => request.engine_options.gh = PathBuf::from(value),
            "curl" => request.engine_options.curl = PathBuf::from(value),
            "retry-delay-ms" => {
                request.engine_options.retry_delay_ms = value
                    .parse()
                    .map_err(|_| "--retry-delay-ms needs a number of milliseconds".to_owned())?;
            }
            "catalog" => request.catalog = Some(absolute(&value)),
            // An option an old setup recorded: read, and nothing is done with it.
            _ => {}
        }
    }
    if let Some(fallout3) = fallout3 {
        request.ttw = Some(Ttw {
            fallout3,
            newvegas: None,
            package,
            resume: false,
            timeout,
        });
    } else if package.is_some() || timeout.is_some() {
        return Err(
            "--package and --timeout-secs are TTW options and need --fallout3 <dir>".to_owned(),
        );
    }
    parsed.request = request;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    /// The absolute form of `path` on this platform (`/i` is `D:\i` on Windows).
    fn abs(path: &str) -> PathBuf {
        std::path::absolute(path).unwrap()
    }

    #[test]
    fn the_options_of_a_setup_make_a_typed_request() {
        let p = parse(&words(
            "--base /b --instance /i --masterlist /m.yaml --profile P --mod /a/One.zip --mod /a/Two.zip \
             --fallout3 /f3 --package /p.mpi --timeout-secs 60 --source /nv --release --skip-nv-rs --sort --copy --dry-run --json",
        ))
        .unwrap();
        let r = &p.request;
        assert_eq!(r.base, abs("/b"));
        assert_eq!(r.instance, abs("/i"));
        assert_eq!(r.profile.as_deref(), Some("P"));
        assert_eq!(r.mods.len(), 2);
        let ttw = r.ttw.as_ref().unwrap();
        assert_eq!(ttw.fallout3, abs("/f3"));
        assert_eq!(ttw.package, Some(abs("/p.mpi")));
        assert_eq!(ttw.timeout, Some(Duration::from_secs(60)));
        assert!(!r.engine && r.sort && r.copy && r.engine_options.release);
        assert_eq!(r.engine_options.source, Some(abs("/nv")));
        assert!(p.json && p.dry_run && !p.keep_partial && p.has_run_options);
    }

    #[test]
    fn the_engine_is_on_unless_it_is_skipped_as_in_nvmod_setup() {
        assert!(parse(&words("--instance /i")).unwrap().request.engine);
    }

    #[test]
    fn a_resume_and_a_restart_are_read_with_their_instance() {
        let p = parse(&words("--resume --instance /i")).unwrap();
        assert!(p.resume && p.has_instance && !p.has_run_options);
        let p = parse(&words("--restart --instance=/i --yes")).unwrap();
        assert!(p.restart && p.yes && !p.has_run_options);
    }

    #[test]
    fn options_an_old_setup_recorded_are_read_and_ignored() {
        let p = parse(&words(
            "--instance /i --extract-tool /x --installer /y --lock /l --store /s",
        ))
        .unwrap();
        assert_eq!(p.request.instance, abs("/i"));
    }

    #[test]
    fn what_cannot_be_read_is_said_in_plain_words() {
        for (line, why) in [
            ("--nope", "unknown option --nope"),
            ("--base", "--base needs a value"),
            ("stray", "unexpected word stray"),
            ("--timeout-secs x --fallout3 /f", "--timeout-secs needs"),
            ("--package /p", "need --fallout3"),
            ("--retry-delay-ms x", "--retry-delay-ms needs"),
        ] {
            let e = parse(&words(line)).unwrap_err();
            assert!(e.contains(why), "{line}: {e}");
        }
    }
}
