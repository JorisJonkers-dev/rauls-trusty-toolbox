// SPDX-License-Identifier: GPL-3.0-only
//! `raul update`: check the Toolbox's GitHub Releases for a newer release, download and verify one, install it, go back
//! to the version before, and turn the GUI's check at start on or off (ADR-0005, ADR-0020). Nothing is fetched until a
//! command asks for it, and nothing identifying is sent. `check` only reads the release list; `download` is the only
//! command that downloads; `install` and `rollback` use what is already on the machine.

use std::io::Write;
use std::path::Path;

use raul_core::selfupdate::fetch::CurlFetch;
use raul_core::selfupdate::flow;
use raul_core::selfupdate::layout::{self, Layout, Os};
use raul_core::selfupdate::selfcheck;
use raul_core::update::{self, Settings};

/// The version this program is.
const RUNNING: &str = env!("CARGO_PKG_VERSION");

/// How this install is updated, from where the program is.
fn layout_here() -> Layout {
    let exe = std::env::current_exe().unwrap_or_default();
    let appimage = std::env::var_os("APPIMAGE").map(std::path::PathBuf::from);
    layout::detect(&exe, appimage.as_deref(), Os::current())
}

pub(crate) fn run(
    ctx: &raul_core::context::Context,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    // The commands that need no configuration folder.
    match words.as_slice() {
        ["status", rest @ ..] if only_json(rest) => {
            return status(rest.contains(&"--json"), out, err);
        }
        ["install", rest @ ..] if only_json(rest) => {
            return install(rest.contains(&"--json"), out, err);
        }
        ["rollback", rest @ ..] if only_json(rest) => {
            return rollback(rest.contains(&"--json"), out, err);
        }
        ["self-check", rest @ ..] => return self_check(rest, out, err),
        ["download", rest @ ..] => return download(rest, out, err),
        _ => {}
    }
    let Some(path) = ctx.settings_file() else {
        let _ = writeln!(
            err,
            "raul update: no configuration folder (HOME, XDG_CONFIG_HOME or APPDATA is not set)"
        );
        return crate::FOUND;
    };
    match words.as_slice() {
        ["check"] => check(false, out, err),
        ["check", "--json"] => check(true, out, err),
        ["auto"] => {
            let on = Settings::load(&path).check_for_updates;
            crate::write_or_fail(
                out,
                err,
                &format!("update check at start: {}\n", if on { "on" } else { "off" }),
            )
        }
        ["auto", value @ ("on" | "off")] => {
            // The file holds the app's other preferences too; they stay as they are.
            let settings = Settings {
                check_for_updates: *value == "on",
                ..Settings::load(&path)
            };
            match settings.save(&path) {
                Ok(()) => {
                    crate::write_or_fail(out, err, &format!("update check at start: {value}\n"))
                }
                Err(e) => {
                    let _ = writeln!(err, "raul update: {}: {e}", path.display());
                    crate::FOUND
                }
            }
        }
        _ => crate::usage(
            err,
            "update takes `check`, `status`, `download`, `install`, `rollback`, `self-check` or `auto [on|off]`",
        ),
    }
}

fn only_json(rest: &[&str]) -> bool {
    rest.iter().all(|w| *w == "--json")
}

/// Say `text` on standard output, or the JSON `value` with `--json`.
fn say(
    json: bool,
    value: &serde_json::Value,
    text: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    emit(json, Ok((value.clone(), text.to_owned())), out, err)
}

/// Why an update step did not go through: the line for standard error, the document `--json` also prints, and the
/// exit code.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    /// What the command prints on standard error.
    pub line: String,
    /// The document `--json` prints on standard output, when the command prints one.
    pub doc: Option<serde_json::Value>,
    /// The exit code the command ends with.
    pub code: u8,
}

/// What an update step ends with: its document and the plain sentence for the person, or why it did not go through.
pub type Step = Result<(serde_json::Value, String), Failure>;

/// Say the step's outcome: its sentence on standard output, or its document with `--json`; a failure on standard
/// error and, with `--json`, its document on standard output.
fn emit(json: bool, step: Step, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match step {
        Ok((doc, text)) => {
            let line = if json {
                format!("{doc}\n")
            } else {
                format!("{text}\n")
            };
            crate::write_or_fail(out, err, &line)
        }
        Err(f) => {
            let _ = writeln!(err, "{}", f.line);
            if json && let Some(doc) = &f.doc {
                let _ = writeln!(out, "{doc}");
            }
            f.code
        }
    }
}

/// A flow error as the step's failure.
fn failure(what: &str, e: &flow::Error) -> Failure {
    Failure {
        line: format!("raul update {what}: {e}"),
        doc: Some(serde_json::json!({ "ok": false, "error": e.to_string(), "detail": e.detail() })),
        code: match e {
            flow::Error::NotHere(_) | flow::Error::NothingStaged => crate::REFUSED,
            _ => crate::FAILED,
        },
    }
}

fn status_step() -> Step {
    let s = flow::status(&layout_here()).map_err(|e| failure("status", &e))?;
    let text = if let Some(reason) = &s.reason {
        reason.clone()
    } else {
        let mut parts = vec![format!("raul {RUNNING}")];
        if let Some(v) = &s.staged {
            parts.push(format!("version {v} is downloaded and ready to install"));
        }
        if let Some(v) = &s.previous {
            parts.push(format!("version {v} is kept to go back to"));
        }
        parts.join("; ")
    };
    Ok((serde_json::json!({ "current": RUNNING, "status": s }), text))
}

fn status(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    emit(json, status_step(), out, err)
}

/// `download [--tag <tag>] [--allow-older] [--json]`: the newest release unless a tag is named.
fn download(rest: &[&str], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (mut json, mut allow_older, mut tag) = (false, false, None);
    let mut words = rest.iter();
    while let Some(w) = words.next() {
        match *w {
            "--json" => json = true,
            "--allow-older" => allow_older = true,
            "--tag" => match words.next() {
                Some(t) => tag = Some((*t).to_owned()),
                None => return crate::usage(err, "--tag needs a release tag"),
            },
            other => return crate::usage(err, &format!("unknown option {other}")),
        }
    }
    emit(json, download_step(tag, allow_older), out, err)
}

fn download_step(tag: Option<String>, allow_older: bool) -> Step {
    let layout = layout_here();
    // Say at once that this install cannot be updated, before anything is fetched.
    if let Layout::NotHere(reason) = &layout {
        return Err(failure("download", &flow::Error::NotHere(reason.plain())));
    }
    let tag = match tag {
        Some(t) => t,
        None => match update::check(&update::runner(), Path::new("curl"), RUNNING) {
            Ok(Some(r)) => format!("v{}", r.version),
            Ok(None) => {
                return Ok((
                    serde_json::json!({ "ok": true, "newer": null, "current": RUNNING }),
                    format!("raul {RUNNING} is the newest release"),
                ));
            }
            Err(e) => {
                return Err(Failure {
                    line: format!("raul update download: {e}"),
                    doc: Some(serde_json::json!({ "ok": false, "error": e, "detail": e })),
                    code: crate::FOUND,
                });
            }
        },
    };
    let s = flow::download(&CurlFetch::default(), &layout, &tag, RUNNING, allow_older)
        .map_err(|e| failure("download", &e))?;
    Ok((
        serde_json::json!({ "ok": true, "staged": s }),
        format!(
            "version {} is downloaded and checked; it is ready to install",
            s.to
        ),
    ))
}

fn install_step() -> Step {
    let s = flow::install(&layout_here(), &|program, version| {
        selfcheck::run_child(program, version)
    })
    .map_err(|e| failure("install", &e))?;
    let text = format!("version {} is installed; start raul again to use it", s.to);
    Ok((serde_json::json!({ "ok": true, "installed": s }), text))
}

fn install(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    emit(json, install_step(), out, err)
}

fn rollback_step() -> Step {
    let j = flow::rollback(&layout_here()).map_err(|e| failure("rollback", &e))?;
    let text = format!(
        "went back to version {}; start raul again to use it",
        j.from
    );
    Ok((serde_json::json!({ "ok": true, "back_to": j.from }), text))
}

fn rollback(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    emit(json, rollback_step(), out, err)
}

/// The update step `verb` (`status`, `check`, `download`, `install`, `rollback`) with the words after it, as the
/// window's direct job runs it: the document `--json` prints, or why it did not go through. `None` is a usage error.
#[must_use]
pub fn answer(verb: &str, rest: &[String]) -> Option<Step> {
    let words: Vec<&str> = rest
        .iter()
        .map(String::as_str)
        .filter(|w| *w != "--json")
        .collect();
    match (verb, words.as_slice()) {
        ("status", []) => Some(status_step()),
        ("check", []) => Some(check_step()),
        ("install", []) => Some(install_step()),
        ("rollback", []) => Some(rollback_step()),
        ("download", flags) => {
            let (mut allow_older, mut tag) = (false, None);
            let mut it = flags.iter();
            while let Some(w) = it.next() {
                match *w {
                    "--allow-older" => allow_older = true,
                    "--tag" => tag = Some((*it.next()?).to_owned()),
                    _ => return None,
                }
            }
            Some(download_step(tag, allow_older))
        }
        _ => None,
    }
}

/// `self-check [--expect <version>] [--json]`: what the new version answers when the old one asks it to check itself.
fn self_check(rest: &[&str], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (mut json, mut expect) = (false, None);
    let mut words = rest.iter();
    while let Some(w) = words.next() {
        match *w {
            "--json" => json = true,
            "--expect" => match words.next() {
                Some(v) => expect = Some((*v).to_owned()),
                None => return crate::usage(err, "--expect needs a version"),
            },
            other => return crate::usage(err, &format!("unknown option {other}")),
        }
    }
    let exe = std::env::current_exe().unwrap_or_default();
    let layout = layout_here();
    let root = exe.parent().map_or_default(Path::to_path_buf);
    let report = selfcheck::run_in_process(
        &root,
        RUNNING,
        expect.as_deref(),
        matches!(layout, Layout::Folder { .. }),
    );
    let text = if report.ok {
        format!("raul {RUNNING} is working")
    } else {
        report.problems.join(" ")
    };
    let code = if report.ok { crate::OK } else { crate::FAILED };
    let doc = serde_json::to_value(&report).unwrap_or_default();
    say(json, &doc, &text, out, err);
    code
}

fn check_step() -> Step {
    let current = env!("CARGO_PKG_VERSION");
    // `curl` is found on the `PATH`; the Runner passes only that, the proxy and certificate variables, and limits.
    match update::check(&update::runner(), Path::new("curl"), current) {
        Ok(newer) => {
            let text = match &newer {
                Some(r) => format!("raul {} is out (you run {current}): {}", r.version, r.url),
                None => format!("raul {current} is the newest release"),
            };
            Ok((
                serde_json::json!({ "current": current, "newer": newer }),
                text,
            ))
        }
        Err(e) => Err(Failure {
            line: format!("raul update check: {e}"),
            doc: None,
            code: crate::FOUND,
        }),
    }
}

fn check(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    emit(json, check_step(), out, err)
}

/// See [`crate::settle_update_at_start`].
pub(crate) fn settle_at_start(args: &[String], err: &mut dyn Write) -> Option<u8> {
    // The update commands judge the update themselves: the self-check must not confirm it early.
    if args.first().map(String::as_str) == Some("update") {
        return None;
    }
    let layout = layout_here();
    let root = layout.root()?;
    if !root
        .join(raul_core::selfupdate::swap::WORK)
        .join("journal.json")
        .is_file()
    {
        return None;
    }
    let exe = std::env::current_exe().unwrap_or_default();
    let dir = exe.parent().map_or_default(Path::to_path_buf);
    let folder = matches!(layout, Layout::Folder { .. });
    let check = || {
        let report = selfcheck::run_in_process(&dir, RUNNING, None, folder);
        match report.problems.into_iter().next() {
            None => Ok(()),
            Some(first) => Err(first),
        }
    };
    settle_outcome(
        raul_core::selfupdate::install::settle_at_start(root, &check),
        err,
    )
}

/// What the start does with the result of settling an update: stop when the old version was put back or when the
/// install could not be put in order (a half-swapped install is not run), carry on when there was nothing to settle,
/// the update is confirmed, or another program holds the lock because it is settling the same update.
fn settle_outcome(
    result: Result<raul_core::selfupdate::install::Startup, raul_core::selfupdate::swap::Error>,
    err: &mut dyn Write,
) -> Option<u8> {
    use raul_core::selfupdate::install::Startup;
    use raul_core::selfupdate::swap::Error as SwapError;
    match result {
        Ok(Startup::RolledBack(why)) => {
            let _ = writeln!(
                err,
                "The update did not work, so raul went back to the old version. {why} Start raul again."
            );
            Some(crate::FAILED)
        }
        Ok(Startup::Recovered) => {
            let _ = writeln!(
                err,
                "An update that was interrupted was undone. Start raul again."
            );
            Some(crate::FAILED)
        }
        Ok(Startup::Nothing | Startup::Confirmed) | Err(SwapError::Locked) => None,
        Err(e) => {
            let _ = writeln!(
                err,
                "raul could not put its last update in order, so it did not start: {e}"
            );
            Some(crate::FAILED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_core::selfupdate::install::Startup;
    use raul_core::selfupdate::swap::Error as SwapError;

    fn outcome(result: Result<Startup, SwapError>) -> (Option<u8>, String) {
        let mut err = Vec::new();
        let code = settle_outcome(result, &mut err);
        (code, String::from_utf8(err).unwrap())
    }

    #[test]
    fn nothing_to_settle_a_confirmed_update_and_a_lock_held_by_another_let_the_program_run() {
        for result in [
            Ok(Startup::Nothing),
            Ok(Startup::Confirmed),
            Err(SwapError::Locked),
        ] {
            assert_eq!(outcome(result), (None, String::new()));
        }
    }

    #[test]
    fn a_rollback_a_recovery_and_an_install_that_could_not_be_put_in_order_stop_the_program() {
        let (code, said) = outcome(Ok(Startup::RolledBack("A file is changed.".into())));
        assert_eq!(code, Some(crate::FAILED));
        assert!(
            said.contains("went back to the old version") && said.contains("A file is changed."),
            "{said}"
        );
        let (code, said) = outcome(Ok(Startup::Recovered));
        assert_eq!(code, Some(crate::FAILED));
        assert!(said.contains("undone"), "{said}");
        let (code, said) = outcome(Err(SwapError::Io("a file could not be moved".into())));
        assert_eq!(
            code,
            Some(crate::FAILED),
            "a half-swapped install is not run"
        );
        assert!(said.contains("did not start"), "{said}");
    }
}
