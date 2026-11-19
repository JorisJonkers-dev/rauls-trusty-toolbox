// SPDX-License-Identifier: GPL-3.0-only
//! The engine build that ships next to `raul` (ADR-0005, ADR-0016): finding it, checking it against the checksum list
//! the release wrote beside it, and putting it in an Instance. Nothing here fetches anything: an engine that is not
//! shipped is an error with plain instructions, never a download (#179).
//!
//! **This file goes away with the play mode (#260)**, together with `engine.rs`.

use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use nvmod_instance::{NvmodInstance, find_ci};
use raul_runner::{Call, Runner, sha256_hex};

use super::ErrorKind;

/// The checksum list the release writes in the packaged engine's folder, in `sha256sum` format: one line per file.
pub const MANIFEST: &str = "FILES.sha256";

/// Folders that may hold `components/nv-rs`, relative to the folder of the running program: next to it (tarball,
/// portable zip, NSIS), in a `lib/raul*` folder one level up (deb and `AppImage`, which name it after the product), or
/// in the `Resources` of a macOS bundle (the program being in `Contents/MacOS`). Folders of other packages are not
/// looked at.
fn candidate_roots(exe_dir: &Path) -> Vec<PathBuf> {
    let mut roots = vec![exe_dir.to_path_buf()];
    if let Some(up) = exe_dir.parent() {
        if exe_dir.file_name().is_some_and(|n| n == "MacOS") {
            roots.push(up.join("Resources"));
        }
        if let Ok(entries) = std::fs::read_dir(up.join("lib")) {
            let mut dirs: Vec<PathBuf> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .starts_with("raul")
                })
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            dirs.sort();
            roots.extend(dirs);
        }
    }
    roots
}

/// The folder of the packaged engine build for the program at `exe`: the first `components/nv-rs` beside it that holds a
/// [`MANIFEST`]. `None` when this copy of raul ships none (macOS, a development build).
#[must_use]
pub fn locate(exe: &Path) -> Option<PathBuf> {
    let exe_dir = exe.parent()?;
    candidate_roots(exe_dir)
        .into_iter()
        .map(|root| root.join("components").join("nv-rs"))
        .find(|dir| dir.join(MANIFEST).is_file())
}

/// Why the packaged engine could not be used: the step's error kind and a plain summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// How the step fails: refused before anything was written, unverified, or failed and rolled back.
    pub kind: ErrorKind,
    /// What to tell the person.
    pub summary: String,
}

fn problem(kind: ErrorKind, summary: impl Into<String>) -> Problem {
    Problem {
        kind,
        summary: summary.into(),
    }
}

/// The problem when this copy of raul ships no engine: what to do, without a command or a tool name.
#[must_use]
pub fn missing() -> Problem {
    problem(
        ErrorKind::Refused,
        "This copy of raul does not come with the game engine. Download raul from its release page and install raul again; \
         the engine is part of the same download.",
    )
}

fn viewer_name() -> String {
    format!("nv-viewer{}", std::env::consts::EXE_SUFFIX)
}

fn unverified(why: impl std::fmt::Display) -> Problem {
    problem(
        ErrorKind::Unverified,
        format!("The game engine that came with raul is damaged ({why}). Install raul again."),
    )
}

/// The engine's checksum list as (hash, relative path) pairs.
fn parse(list: &str) -> Result<Vec<(String, PathBuf)>, Problem> {
    let mut out = Vec::new();
    for line in list.lines().filter(|l| !l.trim().is_empty()) {
        let Some((hash, rest)) = line.split_once(char::is_whitespace) else {
            return Err(unverified("the checksum list is malformed"));
        };
        let name = rest.trim_start().trim_start_matches('*');
        let relative = Path::new(name);
        let hex = hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit());
        let inside = !name.is_empty()
            && relative.is_relative()
            && relative
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
        if !hex || !inside {
            return Err(unverified("the checksum list is malformed"));
        }
        out.push((hash.to_ascii_lowercase(), relative.to_path_buf()));
    }
    Ok(out)
}

/// A file of the packaged engine whose checksum matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// Where it is.
    pub path: PathBuf,
    /// Its SHA-256 as the list gives it.
    pub sha256: String,
}

/// What [`verify`] found: the viewer and, when the list names one, the capabilities file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    /// The engine's program.
    pub viewer: Checked,
    /// The file that says which extender commands the engine implements, when the list names it.
    pub capabilities: Option<Checked>,
}

/// Check every file the packaged engine's [`MANIFEST`] lists against its checksum. A listed file must be a regular
/// file (a link is refused), and the list must name exactly one viewer.
///
/// # Errors
/// [`ErrorKind::Unverified`] when the list is unreadable or malformed, a listed file is missing, a link or differs, or
/// the list names no viewer or more than one.
pub fn verify(dir: &Path) -> Result<Verified, Problem> {
    let list = std::fs::read_to_string(dir.join(MANIFEST))
        .map_err(|e| unverified(format!("the checksum list cannot be read: {e}")))?;
    let (mut viewer, mut capabilities) = (None, None);
    for (hash, relative) in parse(&list)? {
        let file = dir.join(&relative);
        let plain = std::fs::symlink_metadata(&file).is_ok_and(|m| m.file_type().is_file());
        if !plain {
            return Err(unverified("a listed file is missing or is a link"));
        }
        let bytes = std::fs::read(&file).map_err(|_| unverified("a listed file cannot be read"))?;
        if sha256_hex(&bytes) != hash {
            return Err(unverified("the checksum of a file does not match"));
        }
        let name = relative
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        let checked = Checked {
            path: file,
            sha256: hash,
        };
        if name.as_deref() == Some(viewer_name().as_str()) {
            if viewer.replace(checked).is_some() {
                return Err(unverified("it lists more than one program"));
            }
        } else if name.as_deref() == Some(CAPABILITIES) && relative.components().count() == 1 {
            capabilities = Some(checked);
        }
    }
    let viewer = viewer.ok_or_else(|| unverified("it holds no program"))?;
    Ok(Verified {
        viewer,
        capabilities,
    })
}

const STAGING: &str = ".nv-rs-staging";
const FINAL: &str = "nv-rs";
const CAPABILITIES: &str = "capabilities.json";
const RECORD: &str = "install.json";
/// How long the installation check may take (it reads the whole Data folder) and how much of its output is kept.
const CHECK_TIMEOUT: Duration = Duration::from_mins(30);
const CHECK_OUTPUT_CAP: usize = 16 * 1024 * 1024;

fn io(path: &Path, e: &std::io::Error) -> Problem {
    problem(ErrorKind::Failed, format!("{}: {e}", path.display()))
}

fn stopped() -> Problem {
    problem(ErrorKind::Failed, "The install was stopped.")
}

/// Whether anything, a link included, stands at `path`.
fn present(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// Put the packaged engine at `packaged` into the Instance at `instance`, as the engine install library does for a
/// build it fetched: verify, stage (hashing the copy, not the original), move into place, run the installation check,
/// write the record. On failure the Instance is as it was; a leftover `nv-rs` folder without a record (a removal that
/// failed, as can happen on Windows) is an unfinished install and is cleared before the next try. `stop` is asked
/// before each stage; the check itself cannot be stopped once it runs.
///
/// # Errors
/// [`ErrorKind::Refused`] when the folder is no Instance, holds an engine already, or a path in it is a link;
/// [`ErrorKind::Unverified`] when the engine does not match its checksums; [`ErrorKind::Failed`] when copying or the
/// check failed, or `stop` said so.
pub fn install(instance: &Path, packaged: &Path, stop: &dyn Fn() -> bool) -> Result<(), Problem> {
    let found = NvmodInstance::open(instance)
        .map_err(|e| problem(ErrorKind::Refused, format!("not an Instance: {e}")))?;
    let root = std::fs::canonicalize(&found.root)
        .map(|p| nvmod_core::plain_path(&p))
        .map_err(|e| io(&found.root, &e))?;
    let (target, staging) = (root.join(FINAL), root.join(STAGING));
    for path in [&target, &staging] {
        if is_link(path) {
            return Err(problem(
                ErrorKind::Refused,
                "A link stands where the engine goes; it was left alone.",
            ));
        }
    }
    if target.join(RECORD).is_file() || (present(&target) && !target.is_dir()) {
        return Err(problem(
            ErrorKind::Refused,
            format!(
                "The engine is already installed in {}; it is not overwritten.",
                root.display()
            ),
        ));
    }
    if present(&target) {
        std::fs::remove_dir_all(&target).map_err(|e| io(&target, &e))?;
    }
    let verified = verify(packaged)?;
    let _ = std::fs::remove_dir_all(&staging);
    let result = if stop() {
        Err(stopped())
    } else {
        stage(&verified, &staging, stop).and_then(|()| {
            if stop() {
                Err(stopped())
            } else {
                commit(&found, &root, &staging)
            }
        })
    };
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    result
}

/// Copy `from` to `to` and check the copy, not the original, against `sha256`: what was checked is what is installed.
fn copy_checked(from: &Checked, to: &Path) -> Result<(), Problem> {
    std::fs::copy(&from.path, to).map_err(|e| io(to, &e))?;
    let bytes = std::fs::read(to).map_err(|e| io(to, &e))?;
    if sha256_hex(&bytes) == from.sha256 {
        Ok(())
    } else {
        Err(unverified("a file changed while it was copied"))
    }
}

fn stage(verified: &Verified, staging: &Path, stop: &dyn Fn() -> bool) -> Result<(), Problem> {
    let bin = staging.join("bin");
    std::fs::create_dir_all(&bin).map_err(|e| io(&bin, &e))?;
    copy_checked(&verified.viewer, &bin.join(viewer_name()))?;
    if stop() {
        return Err(stopped());
    }
    if let Some(capabilities) = &verified.capabilities {
        copy_checked(capabilities, &staging.join(CAPABILITIES))?;
    }
    Ok(())
}

fn commit(found: &NvmodInstance, root: &Path, staging: &Path) -> Result<(), Problem> {
    let target = root.join(FINAL);
    std::fs::rename(staging, &target).map_err(|e| io(&target, &e))?;
    let finish = || -> Result<(), Problem> {
        let binary = target.join("bin").join(viewer_name());
        let data =
            find_ci(&found.config.base, "Data").unwrap_or_else(|| found.config.base.join("Data"));
        let arguments = vec![data.display().to_string(), "--check".to_owned()];
        check(&binary, &arguments)?;
        let bytes = std::fs::read(&binary).map_err(|e| io(&binary, &e))?;
        // The record keeps the shape nvmod's verify reads, which names the program.
        let record = serde_json::json!({
            "method": "packaged",
            "sha256": sha256_hex(&bytes),
            "check": {"ran": true, "command": "nv-viewer", "arguments": arguments, "exit_code": 0},
        });
        let text = serde_json::to_string_pretty(&record)
            .map_err(|e| problem(ErrorKind::Failed, e.to_string()))?;
        let file = target.join(RECORD);
        std::fs::write(&file, text + "\n").map_err(|e| io(&file, &e))
    };
    finish().inspect_err(|_| {
        // Where this fails (Windows, with the program still open), the folder has no record and the next try clears it.
        let _ = std::fs::remove_dir_all(&target);
    })
}

/// Run the installation check. The first line of a failure says what happened; what the check printed follows.
fn check(binary: &Path, arguments: &[String]) -> Result<(), Problem> {
    let runner = Runner {
        timeout: CHECK_TIMEOUT,
        max_output: CHECK_OUTPUT_CAP,
        max_stderr: CHECK_OUTPUT_CAP,
        ..Runner::default()
    };
    let out = runner
        .run_any(&Call::new("nv-viewer", binary, arguments))
        .map_err(|_| {
            problem(
                ErrorKind::Failed,
                "The engine's check could not run: it did not start, or it ran for too long.",
            )
        })?;
    if out.exit_code == 0 {
        return Ok(());
    }
    Err(problem(
        ErrorKind::Failed,
        format!(
            "The engine's check did not pass (exit status {}).\n{}{}",
            out.exit_code,
            String::from_utf8_lossy(&out.stdout).trim_end(),
            out.stderr.trim_end()
        ),
    ))
}

/// What `raul install-nv-rs` reports: how the engine came to be in the Instance and where its program is.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Installed {
    /// `packaged` (the build that ships with raul) or `source` (built by a developer).
    pub method: &'static str,
    /// The installed program.
    pub binary: PathBuf,
}

fn job_error(p: Problem) -> nvmod_api::JobError {
    let kind = match p.kind {
        ErrorKind::Refused => nvmod_api::ErrorKind::Refused,
        ErrorKind::Unverified => nvmod_api::ErrorKind::Corrupt,
        _ => nvmod_api::ErrorKind::Failed,
    };
    nvmod_api::JobError::new(kind, "install_nvrs.install", p.summary)
}

/// The `raul install-nv-rs` job, for the command line and the GUI's direct job: put the engine in the Instance with
/// `source` (a developer's workspace, built with `cargo`) or else the packaged build at `packaged`, else the one that
/// ships next to the running program. It never fetches anything. Hold the Instance lock around it.
///
/// # Errors
/// As [`install`], and `Refused` when this copy of raul ships no engine ([`missing`]).
pub fn job(
    instance: &Path,
    source: Option<&Path>,
    cargo: Option<&Path>,
    packaged: Option<&Path>,
    progress: &mut dyn nvmod_api::Progress,
    cancel: &nvmod_api::CancelToken,
) -> Result<nvmod_api::Outcome<Installed>, nvmod_api::JobError> {
    if cancel.is_cancelled() {
        return Ok(nvmod_api::Outcome::Cancelled);
    }
    progress.report(&nvmod_api::Event::step(
        "install-nv-rs",
        "install_nvrs.install.installing",
    ));
    if let Some(source) = source {
        let cargo = cargo
            .map(Path::to_path_buf)
            .or_else(|| std::env::var_os("NVMOD_CARGO").map(PathBuf::from));
        if nvmod_install::resolve_cargo(cargo.as_deref()).is_none() {
            return Err(job_error(problem(
                ErrorKind::Refused,
                "Building the engine from source needs a Rust toolchain, and none was found.",
            )));
        }
        let request: nvmod_api::install_nvrs::InstallRequest = serde_json::from_value(
            serde_json::json!({"instance": instance, "source": source, "cargo": cargo, "release": false}),
        )
        .map_err(|e| job_error(problem(ErrorKind::Failed, e.to_string())))?;
        return Ok(
            match nvmod_api::install_nvrs::install(&request, progress, cancel)? {
                nvmod_api::Outcome::Done(done) => nvmod_api::Outcome::Done(Installed {
                    method: "source",
                    binary: done.binary,
                }),
                nvmod_api::Outcome::Cancelled => nvmod_api::Outcome::Cancelled,
            },
        );
    }
    let dir = packaged
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_exe().ok().and_then(|exe| locate(&exe)))
        .filter(|dir| dir.join(MANIFEST).is_file())
        .ok_or_else(|| job_error(missing()))?;
    install(instance, &dir, &|| cancel.is_cancelled()).map_err(job_error)?;
    let binary = instance.join(FINAL).join("bin").join(viewer_name());
    Ok(nvmod_api::Outcome::Done(Installed {
        method: "packaged",
        binary,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{
        FAKE_CHECK_REFUSED, TempDir, fake_failing_release_binary, fake_release_binary,
        nv_viewer_name, write_executable, write_file,
    };

    /// One build of the fake engine program for the whole test run: on Windows each call compiles a new stand-in, whose
    /// bytes differ from the last, so a test that packages one and compares with another would never match.
    fn viewer_bytes() -> Vec<u8> {
        static BYTES: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        BYTES.get_or_init(fake_release_binary).clone()
    }

    fn shipped(root: &Path) {
        write_file(root, "components/nv-rs/FILES.sha256", b"");
    }

    /// A packaged engine folder whose list names `files` (relative path, bytes) with the right checksums.
    fn engine(root: &Path, files: &[(&str, Vec<u8>)]) -> PathBuf {
        let dir = root.join("components/nv-rs");
        let mut list = String::new();
        for (name, bytes) in files {
            if name.ends_with(&nv_viewer_name()) {
                let bin = dir.join("bin");
                std::fs::create_dir_all(&bin).expect("bin");
                write_executable(&bin, &nv_viewer_name(), bytes);
            } else {
                write_file(&dir, name, bytes);
            }
            list.push_str(&sha256_hex(bytes));
            list.push_str("  ");
            list.push_str(name);
            list.push('\n');
        }
        write_file(&dir, MANIFEST, list.as_bytes());
        dir
    }

    fn viewer() -> String {
        format!("bin/{}", nv_viewer_name())
    }

    fn problem(r: Result<Verified, Problem>) -> Problem {
        match r {
            Err(p) => p,
            Ok(v) => panic!("expected a problem, got {v:?}"),
        }
    }

    #[test]
    fn the_engine_next_to_the_program_is_found() {
        let t = TempDir::new("packaged-locate-next");
        shipped(t.path());
        let found = locate(&t.path().join("raul"));
        assert_eq!(found, Some(t.path().join("components/nv-rs")));
    }

    #[test]
    fn the_engine_in_a_lib_folder_of_a_deb_layout_is_found() {
        let t = TempDir::new("packaged-locate-deb");
        write_file(t.path(), "usr/bin/raul", b"x");
        shipped(&t.path().join("usr/lib/raul"));
        let found = locate(&t.path().join("usr/bin/raul"));
        assert_eq!(found, Some(t.path().join("usr/lib/raul/components/nv-rs")));
    }

    #[test]
    fn the_engine_in_the_resources_of_a_bundle_is_found() {
        let t = TempDir::new("packaged-locate-mac");
        write_file(t.path(), "Contents/MacOS/raul", b"x");
        shipped(&t.path().join("Contents/Resources"));
        let found = locate(&t.path().join("Contents/MacOS/raul"));
        assert_eq!(
            found,
            Some(t.path().join("Contents/Resources/components/nv-rs"))
        );
    }

    #[test]
    fn a_folder_without_a_checksum_list_is_not_an_engine() {
        let t = TempDir::new("packaged-locate-none");
        write_file(t.path(), "components/nv-rs/bin/nv-viewer", b"x");
        assert_eq!(locate(&t.path().join("raul")), None);
    }

    #[test]
    fn a_listed_engine_that_matches_its_checksums_verifies_and_names_its_viewer() {
        let t = TempDir::new("packaged-verify-ok");
        let dir = engine(
            t.path(),
            &[(&viewer(), viewer_bytes()), ("README", b"hi".to_vec())],
        );
        assert_eq!(
            verify(&dir).expect("verified").viewer.path,
            dir.join(viewer())
        );
    }

    #[test]
    fn a_changed_file_is_unverified() {
        let t = TempDir::new("packaged-verify-changed");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        write_file(&dir, &viewer(), b"tampered");
        let p = problem(verify(&dir));
        assert_eq!(p.kind, ErrorKind::Unverified);
        assert!(p.summary.contains("checksum"), "{}", p.summary);
    }

    #[test]
    fn a_missing_file_is_unverified() {
        let t = TempDir::new("packaged-verify-missing");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        std::fs::remove_file(dir.join(viewer())).expect("remove");
        assert_eq!(problem(verify(&dir)).kind, ErrorKind::Unverified);
    }

    #[test]
    fn a_list_that_names_no_viewer_is_unverified() {
        let t = TempDir::new("packaged-verify-no-viewer");
        let dir = engine(t.path(), &[("README", b"hi".to_vec())]);
        let p = problem(verify(&dir));
        assert_eq!(p.kind, ErrorKind::Unverified);
        assert!(!p.summary.contains("nv-viewer"), "{}", p.summary);
    }

    #[test]
    fn a_list_that_leaves_its_folder_or_is_malformed_is_unverified() {
        for line in [
            format!("{}  ../outside\n", sha256_hex(b"x")),
            format!("{}  /etc/passwd\n", sha256_hex(b"x")),
            "not-a-hash  bin/nv-viewer\n".to_owned(),
            "nospace\n".to_owned(),
        ] {
            let t = TempDir::new("packaged-verify-bad-list");
            let dir = t.path().join("components/nv-rs");
            write_file(&dir, MANIFEST, line.as_bytes());
            assert_eq!(problem(verify(&dir)).kind, ErrorKind::Unverified, "{line}");
        }
    }

    #[test]
    fn the_binary_mode_marker_of_sha256sum_is_accepted() {
        let t = TempDir::new("packaged-verify-star");
        let dir = t.path().join("components/nv-rs");
        let name = viewer();
        write_file(&dir, &name, &viewer_bytes());
        let line = format!("{} *{name}\n", sha256_hex(&viewer_bytes()));
        write_file(&dir, MANIFEST, line.as_bytes());
        assert!(verify(&dir).is_ok());
    }

    fn instance(t: &TempDir) -> PathBuf {
        let base = t.path().join("base");
        nvmod_testkit::BaseBuilder::steam().write(&base);
        let root = t.path().join("fnv");
        let out = nvmod_api::init::create(
            &nvmod_api::init::CreateRequest {
                instance: root.clone(),
                base,
            },
            &mut nvmod_api::NoProgress,
            &nvmod_api::CancelToken::new(),
        );
        assert!(out.is_ok(), "{out:?}");
        root
    }

    #[test]
    fn a_verified_engine_is_installed_into_the_instance_and_checked() {
        let t = TempDir::new("packaged-install");
        let dir = engine(
            t.path(),
            &[
                (&viewer(), viewer_bytes()),
                ("capabilities.json", b"{}".to_vec()),
            ],
        );
        let root = instance(&t);
        install(&root, &dir, &|| false).expect("installed");
        let target = root.join("nv-rs");
        assert_eq!(
            std::fs::read(target.join("bin").join(nv_viewer_name())).expect("viewer"),
            viewer_bytes()
        );
        assert!(target.join("capabilities.json").is_file());
        let record = std::fs::read_to_string(target.join("install.json")).expect("record");
        assert!(record.contains("\"method\": \"packaged\""), "{record}");
        assert!(!root.join(".nv-rs-staging").exists());
    }

    #[test]
    fn a_failing_check_is_a_failure_and_leaves_nothing_behind() {
        let t = TempDir::new("packaged-install-check");
        let dir = engine(t.path(), &[(&viewer(), fake_failing_release_binary())]);
        let root = instance(&t);
        let p = install(&root, &dir, &|| false).expect_err("the check fails");
        assert_eq!(p.kind, ErrorKind::Failed);
        assert!(p.summary.contains(FAKE_CHECK_REFUSED), "{}", p.summary);
        assert!(!root.join("nv-rs").exists());
        assert!(!root.join(".nv-rs-staging").exists());
    }

    #[test]
    fn an_unverified_engine_installs_nothing() {
        let t = TempDir::new("packaged-install-unverified");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        write_file(&dir, &viewer(), b"tampered");
        let root = instance(&t);
        let p = install(&root, &dir, &|| false).expect_err("unverified");
        assert_eq!(p.kind, ErrorKind::Unverified);
        assert!(!root.join("nv-rs").exists());
        assert!(!root.join(".nv-rs-staging").exists());
    }

    #[test]
    fn an_engine_already_in_the_instance_is_refused_and_left_alone() {
        let t = TempDir::new("packaged-install-twice");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        let root = instance(&t);
        install(&root, &dir, &|| false).expect("first");
        let p = install(&root, &dir, &|| false).expect_err("second");
        assert_eq!(p.kind, ErrorKind::Refused);
        assert!(p.summary.contains("already installed"), "{}", p.summary);
        assert!(root.join("nv-rs/bin").join(nv_viewer_name()).is_file());
    }

    #[test]
    fn a_missing_engine_says_what_to_do_in_plain_words() {
        let p = missing();
        assert_eq!(p.kind, ErrorKind::Refused);
        assert!(p.summary.contains("install raul again"), "{}", p.summary);
        for tool in ["curl", "gh ", "cargo", "--release", "install-nv-rs"] {
            assert!(!p.summary.contains(tool), "{tool}: {}", p.summary);
        }
    }

    fn run_job(
        root: &Path,
        packaged: Option<&Path>,
    ) -> Result<nvmod_api::Outcome<Installed>, nvmod_api::JobError> {
        job(
            root,
            None,
            None,
            packaged,
            &mut nvmod_api::NoProgress,
            &nvmod_api::CancelToken::new(),
        )
    }

    #[test]
    fn the_job_installs_the_packaged_engine_and_never_fetches() {
        let t = TempDir::new("packaged-job");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        let root = instance(&t);
        let done = run_job(&root, Some(&dir))
            .expect("job")
            .done()
            .expect("done");
        assert_eq!(done.method, "packaged");
        assert!(done.binary.is_file());
    }

    #[test]
    fn the_job_maps_each_problem_to_its_kind() {
        let t = TempDir::new("packaged-job-errors");
        let root = instance(&t);
        let none = run_job(&root, Some(&t.path().join("nope"))).expect_err("missing");
        assert_eq!(none.kind, nvmod_api::ErrorKind::Refused);
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        write_file(&dir, &viewer(), b"tampered");
        let bad = run_job(&root, Some(&dir)).expect_err("damaged");
        assert_eq!(bad.kind, nvmod_api::ErrorKind::Corrupt);
        let dir = engine(t.path(), &[(&viewer(), fake_failing_release_binary())]);
        let failed = run_job(&root, Some(&dir)).expect_err("check");
        assert_eq!(failed.kind, nvmod_api::ErrorKind::Failed);
    }

    #[test]
    fn a_source_build_without_a_toolchain_is_refused_and_does_not_fetch() {
        let t = TempDir::new("packaged-job-nocargo");
        let root = instance(&t);
        let e = job(
            &root,
            Some(t.path()),
            Some(&t.path().join("no-such-cargo")),
            None,
            &mut nvmod_api::NoProgress,
            &nvmod_api::CancelToken::new(),
        )
        .expect_err("no toolchain");
        assert_eq!(e.kind, nvmod_api::ErrorKind::Refused);
        assert!(!root.join("nv-rs").exists());
    }

    #[test]
    fn another_packages_lib_folder_is_not_searched() {
        let t = TempDir::new("packaged-locate-other");
        write_file(t.path(), "usr/bin/raul", b"x");
        shipped(&t.path().join("usr/lib/other-package"));
        assert_eq!(locate(&t.path().join("usr/bin/raul")), None);
        shipped(&t.path().join("usr/lib/Raul's tRusty Toolbox"));
        assert_eq!(
            locate(&t.path().join("usr/bin/raul")),
            Some(
                t.path()
                    .join("usr/lib/Raul's tRusty Toolbox/components/nv-rs")
            )
        );
    }

    #[test]
    fn resources_are_searched_only_for_a_program_in_a_bundle() {
        let t = TempDir::new("packaged-locate-resources");
        write_file(t.path(), "bin/raul", b"x");
        shipped(&t.path().join("Resources"));
        assert_eq!(locate(&t.path().join("bin/raul")), None);
    }

    #[cfg(unix)]
    #[test]
    fn a_listed_file_that_is_a_link_is_unverified() {
        let t = TempDir::new("packaged-verify-link");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        let real = t.path().join("real-viewer");
        std::fs::rename(dir.join(viewer()), &real).unwrap();
        std::os::unix::fs::symlink(&real, dir.join(viewer())).unwrap();
        assert_eq!(problem(verify(&dir)).kind, ErrorKind::Unverified);
    }

    #[test]
    fn a_list_with_two_programs_is_unverified() {
        let t = TempDir::new("packaged-verify-two");
        let other = format!("other/{}", nv_viewer_name());
        let dir = engine(
            t.path(),
            &[(&viewer(), viewer_bytes()), (&other, viewer_bytes())],
        );
        assert_eq!(problem(verify(&dir)).kind, ErrorKind::Unverified);
    }

    #[test]
    fn a_capabilities_file_not_on_the_list_is_not_installed() {
        let t = TempDir::new("packaged-caps");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        write_file(&dir, "capabilities.json", b"{}");
        let root = instance(&t);
        install(&root, &dir, &|| false).expect("installed");
        assert!(!root.join("nv-rs/capabilities.json").exists());
    }

    #[test]
    fn a_stop_asked_for_installs_nothing() {
        let t = TempDir::new("packaged-stop");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        let root = instance(&t);
        assert!(install(&root, &dir, &|| true).is_err());
        assert!(!root.join("nv-rs").exists() && !root.join(".nv-rs-staging").exists());
    }

    #[test]
    fn an_unfinished_leftover_without_a_record_is_cleared_and_a_retry_works() {
        let t = TempDir::new("packaged-leftover");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        let root = instance(&t);
        write_file(&root, "nv-rs/bin/half", b"half");
        install(&root, &dir, &|| false).expect("retry");
        assert!(root.join("nv-rs/install.json").is_file());
        assert!(!root.join("nv-rs/bin/half").exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_where_the_engine_goes_is_refused_and_not_followed() {
        let t = TempDir::new("packaged-link-target");
        let dir = engine(t.path(), &[(&viewer(), viewer_bytes())]);
        let root = instance(&t);
        let elsewhere = t.path().join("elsewhere");
        write_file(&elsewhere, "precious", b"keep");
        std::os::unix::fs::symlink(&elsewhere, root.join(".nv-rs-staging")).unwrap();
        let p = install(&root, &dir, &|| false).expect_err("refused");
        assert_eq!(p.kind, ErrorKind::Refused);
        assert!(elsewhere.join("precious").is_file());
    }

    #[test]
    fn a_failed_check_reads_as_a_headline_then_what_the_check_printed() {
        let t = TempDir::new("packaged-headline");
        let dir = engine(t.path(), &[(&viewer(), fake_failing_release_binary())]);
        let root = instance(&t);
        let p = install(&root, &dir, &|| false).expect_err("fails");
        let (head, rest) = p.summary.split_once('\n').expect("two parts");
        assert!(
            head.contains("did not pass") && !head.contains("nv-viewer"),
            "{head}"
        );
        assert!(rest.contains(FAKE_CHECK_REFUSED), "{rest}");
    }
}
