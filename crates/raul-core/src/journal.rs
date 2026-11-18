// SPDX-License-Identifier: GPL-3.0-only
//! The setup journal: the Contract `raul-journal` (ADR-0009), an on-disk document the setup job writes into the
//! Instance after each finished step, so a setup that was interrupted resumes at the first unfinished step,
//! restarts, or is removed from `raul`.
//!
//! The file is `<instance>/.raul/setup-journal.json`. It is written only after a step finished, so nothing is
//! created in the Instance folder before the first step made it. It has an integer `schema_version`; a file of an
//! older version is migrated when it is opened, after a copy of it was kept next to it; a file of a newer version
//! is refused and left as it is. Every write goes to a temporary file that is renamed over the real one, so an
//! interrupted write leaves the old file whole.
//!
//! An Instance that holds an unfinished `nvmod setup` (`.nvmod/setup.json`) and no journal is taken over once: the
//! steps that setup finished become the journal's, a copy of its file is kept in `.raul/`, and `nvmod`'s own file
//! is not touched. The copy also marks the takeover as done, so removing the journal does not take it over again.
//!
//! The journal has one writer at a time, the setup job, as the configuration file has. The Toolbox has no
//! Instance lock yet, so the journal does not claim one.

use std::fmt;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::setup::{self, CancelToken, Event, Outcome, Planned, Progress, Request, Step};

/// The Contract name of the journal file.
pub const CONTRACT: &str = "raul-journal";

/// The version of the journal this build writes.
pub const SCHEMA_VERSION: u32 = 1;

/// The folder inside the Instance that holds `raul`'s own files.
pub const FOLDER: &str = ".raul";

/// The journal's file name inside [`FOLDER`].
pub const FILE_NAME: &str = "setup-journal.json";

/// The copy of `nvmod`'s setup state a takeover keeps in [`FOLDER`].
pub const NVMOD_COPY: &str = "nvmod-setup.json.bak";

/// What an `nvmod setup` run left, as `nvmod` names it: the file `.nvmod/setup.json` inside the Instance.
const NVMOD_STATE: [&str; 2] = [".nvmod", "setup.json"];

/// The name of the setup state a journal was taken over from.
const NVMOD_SOURCE: &str = "nvmod-setup";

/// Where a journal came from, when it did not start in `raul`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TakenOver {
    /// What it was taken over from: `nvmod-setup`.
    pub from: String,
    /// The version of the program that wrote the old state, as that state names it.
    pub version: String,
}

/// The document in the journal file (Contract `raul-journal`, version 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct JournalDoc {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-journal`.
    #[schemars(extend("const" = "raul-journal"))]
    pub contract: String,
    /// The version of `raul` that last wrote the file.
    pub raul_version: String,
    /// Set when the journal was made from an unfinished `nvmod setup` and not by `raul` itself.
    #[serde(default)]
    pub taken_over: Option<TakenOver>,
    /// The ids of the steps of the setup, in order, as the plan named them when the run began (see `Step::id`).
    #[serde(default)]
    pub plan: Vec<String>,
    /// The ids of the steps that finished, in the order they finished.
    pub finished: Vec<String>,
    /// Whether every step of the plan finished.
    #[serde(default)]
    pub complete: bool,
}

/// The JSON Schema of [`JournalDoc`], as `contracts/raul-journal.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::contracts::schema_text(&schemars::schema_for!(JournalDoc))
}

/// One step as the journal lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    /// The step's id.
    pub id: String,
    /// Whether it finished.
    pub finished: bool,
}

/// The class of an [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The file has a `schema_version` newer than this build reads; nothing was written.
    NewerSchema,
    /// The file cannot be read or is not a journal; nothing was written.
    Unreadable,
    /// The file or a copy could not be written; the old file is as it was.
    WriteFailed,
    /// The state of an unfinished `nvmod setup` cannot be taken over; it was left as it is.
    TakeoverRefused,
}

impl ErrorKind {
    /// A stable code: `newer_schema`, `unreadable`, `write_failed` or `takeover_refused`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NewerSchema => "newer_schema",
            Self::Unreadable => "unreadable",
            Self::WriteFailed => "write_failed",
            Self::TakeoverRefused => "takeover_refused",
        }
    }
}

/// Why the journal could not be used. It never holds a command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The class.
    pub kind: ErrorKind,
    /// What went wrong, in plain words.
    pub summary: String,
}

impl Error {
    fn new(kind: ErrorKind, summary: String) -> Self {
        Self { kind, summary }
    }

    /// The error as one block of plain text to copy into a bug report: `raul <version>`, the code and the summary,
    /// with the home directory written as `~`.
    #[must_use]
    pub fn copy_text(&self) -> String {
        let summary = std::env::var("HOME").ok().map_or_else(
            || self.summary.clone(),
            |home| setup::tilde(&self.summary, &home),
        );
        format!(
            "raul {}\njournal\nerror: {}\n{summary}",
            env!("CARGO_PKG_VERSION"),
            self.kind.code()
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary)
    }
}

impl std::error::Error for Error {}

fn write_failed(path: &Path, e: &dyn fmt::Display) -> Error {
    Error::new(ErrorKind::WriteFailed, format!("{}: {e}", path.display()))
}

/// The journal of one Instance's setup.
#[derive(Debug, Clone)]
pub struct Journal {
    instance: PathBuf,
    doc: JournalDoc,
    /// Whether the file is on disk.
    on_disk: bool,
}

impl Journal {
    /// The journal file of the Instance at `instance`.
    #[must_use]
    pub fn file(instance: &Path) -> PathBuf {
        instance.join(FOLDER).join(FILE_NAME)
    }

    fn blank() -> JournalDoc {
        JournalDoc {
            schema_version: SCHEMA_VERSION,
            contract: CONTRACT.to_owned(),
            raul_version: env!("CARGO_PKG_VERSION").to_owned(),
            taken_over: None,
            plan: Vec::new(),
            finished: Vec::new(),
            complete: false,
        }
    }

    /// Open the journal of the Instance at `instance`. A file of an older version is migrated, after a copy of it
    /// is kept next to it. With no file, an unfinished `nvmod setup` in the Instance is taken over (once); with
    /// neither, the journal is new and empty and is written when the first step finishes.
    ///
    /// # Errors
    /// [`ErrorKind::NewerSchema`] for a file of a newer version, [`ErrorKind::Unreadable`] for a file that is not a
    /// journal, [`ErrorKind::TakeoverRefused`] for an `nvmod` state that cannot be read, and
    /// [`ErrorKind::WriteFailed`] when a copy or the migrated file cannot be written. Nothing is changed then.
    pub fn open(instance: &Path) -> Result<Self, Error> {
        let file = Self::file(instance);
        match std::fs::read_to_string(&file) {
            Ok(text) => {
                let (doc, migrated) = parse(&file, &text)?;
                let mut journal = Self {
                    instance: instance.to_path_buf(),
                    doc,
                    on_disk: true,
                };
                if migrated {
                    keep_copy(&file, &format!("{FILE_NAME}.v{}.bak", migrated_from(&text)))?;
                    journal.save()?;
                }
                Ok(journal)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::take_over(instance),
            Err(e) => Err(Error::new(
                ErrorKind::Unreadable,
                format!("{}: {e}", file.display()),
            )),
        }
    }

    fn take_over(instance: &Path) -> Result<Self, Error> {
        let fresh = Self {
            instance: instance.to_path_buf(),
            doc: Self::blank(),
            on_disk: false,
        };
        let copy = instance.join(FOLDER).join(NVMOD_COPY);
        if copy.exists() {
            return Ok(fresh);
        }
        let state = NVMOD_STATE
            .iter()
            .fold(instance.to_path_buf(), |p, s| p.join(s));
        let text = match std::fs::read_to_string(&state) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(fresh),
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::TakeoverRefused,
                    format!("{}: {e}; it was left as it is", state.display()),
                ));
            }
        };
        let Some(old) = nvmod_state(&state, &text)? else {
            return Ok(fresh);
        };
        let mut journal = fresh;
        journal.doc.taken_over = Some(TakenOver {
            from: NVMOD_SOURCE.to_owned(),
            version: old.version,
        });
        journal.doc.plan = old.plan;
        journal.doc.finished = old.finished;
        write_atomic(&copy, text.as_bytes())?;
        if let Err(e) = journal.save() {
            let _ = std::fs::remove_file(&copy);
            return Err(e);
        }
        Ok(journal)
    }

    /// The document as it is now.
    #[must_use]
    pub fn doc(&self) -> &JournalDoc {
        &self.doc
    }

    /// The ids of the finished steps, in the order they finished.
    #[must_use]
    pub fn finished(&self) -> &[String] {
        &self.doc.finished
    }

    /// Every step the journal knows, in the plan's order, with whether it finished. A finished step the plan does
    /// not name comes last.
    #[must_use]
    pub fn list(&self) -> Vec<Listed> {
        let mut listed: Vec<Listed> = self
            .doc
            .plan
            .iter()
            .map(|id| Listed {
                id: id.clone(),
                finished: self.doc.finished.contains(id),
            })
            .collect();
        for id in &self.doc.finished {
            if !self.doc.plan.contains(id) {
                listed.push(Listed {
                    id: id.clone(),
                    finished: true,
                });
            }
        }
        listed
    }

    /// Remember the plan of the run in the document. It is written with the next step that finishes.
    pub fn set_plan(&mut self, planned: &[Planned]) {
        self.doc.plan = planned.iter().map(|p| p.step.id()).collect();
    }

    /// Record that `step` finished, and write the file.
    ///
    /// # Errors
    /// [`ErrorKind::WriteFailed`]; the file on disk is as it was.
    pub fn record_finished(&mut self, step: &Step) -> Result<(), Error> {
        let id = step.id();
        if !self.doc.finished.contains(&id) {
            self.doc.finished.push(id);
        }
        self.doc.complete = false;
        self.save()
    }

    /// Record that every step of the plan finished, and write the file.
    ///
    /// # Errors
    /// [`ErrorKind::WriteFailed`]; the file on disk is as it was.
    pub fn record_complete(&mut self) -> Result<(), Error> {
        self.doc.complete = true;
        self.save()
    }

    /// The steps of `request` that already finished: the set to pass as [`Request::done`]. A Mod step is matched
    /// by the file name of its archive, which is what its id holds. The steps the request already lists as done
    /// are part of it.
    #[must_use]
    pub fn resume(&self, request: &Request) -> Vec<Step> {
        let mut done = request.done.clone();
        for planned in setup::plan(request) {
            if self.doc.finished.contains(&planned.step.id()) && !done.contains(&planned.step) {
                done.push(planned.step);
            }
        }
        done
    }

    /// The first step of `request` that is neither finished nor skipped, or `None` when nothing is left.
    #[must_use]
    pub fn next_step(&self, request: &Request) -> Option<Step> {
        let resumed = Request {
            done: self.resume(request),
            ..request.clone()
        };
        setup::plan(&resumed)
            .into_iter()
            .find(|p| p.skip.is_none())
            .map(|p| p.step)
    }

    /// Forget every finished step, so a run starts from the first. The file stays, so the Instance is not taken
    /// over again. Making the Instance ready for a new run (nvmod's `init` refuses a folder that holds one) is the
    /// caller's to do.
    ///
    /// # Errors
    /// [`ErrorKind::WriteFailed`]; the file on disk is as it was.
    pub fn restart(&mut self) -> Result<(), Error> {
        self.doc.finished.clear();
        self.doc.plan.clear();
        self.doc.complete = false;
        if self.on_disk {
            self.save()?;
        }
        Ok(())
    }

    /// Delete the journal file. The copies kept next to it (of a migrated file or of an `nvmod` state) stay, and a
    /// copy of an `nvmod` state keeps that state from being taken over again.
    ///
    /// # Errors
    /// [`ErrorKind::WriteFailed`] when the file cannot be deleted.
    pub fn remove(self) -> Result<(), Error> {
        let file = Self::file(&self.instance);
        match std::fs::remove_file(&file) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(write_failed(&file, &e)),
        }
    }

    fn save(&mut self) -> Result<(), Error> {
        env!("CARGO_PKG_VERSION").clone_into(&mut self.doc.raul_version);
        let mut text = serde_json::to_string_pretty(&self.doc)
            .map_err(|e| Error::new(ErrorKind::WriteFailed, e.to_string()))?;
        text.push('\n');
        write_atomic(&Self::file(&self.instance), text.as_bytes())?;
        self.on_disk = true;
        Ok(())
    }
}

/// The version number a file names, for the name of the copy kept before a migration.
fn migrated_from(text: &str) -> u64 {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("schema_version").and_then(serde_json::Value::as_u64))
        .unwrap_or(0)
}

/// The journal `text` holds, and whether it was of an older version and was migrated.
fn parse(file: &Path, text: &str) -> Result<(JournalDoc, bool), Error> {
    let unreadable = |why: String| {
        Error::new(
            ErrorKind::Unreadable,
            format!("{}: {why}; the file was left as it is", file.display()),
        )
    };
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| unreadable(e.to_string()))?;
    let version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| unreadable("it has no schema_version".to_owned()))?;
    match version {
        v if v > u64::from(SCHEMA_VERSION) => Err(Error::new(
            ErrorKind::NewerSchema,
            format!(
                "{} has schema_version {v}; this raul reads {SCHEMA_VERSION}. Update raul; the file was left as it is",
                file.display()
            ),
        )),
        0 => Ok((migrate_v0(&value).map_err(unreadable)?, true)),
        _ => {
            let doc: JournalDoc =
                serde_json::from_value(value).map_err(|e| unreadable(e.to_string()))?;
            if doc.contract != CONTRACT {
                return Err(unreadable(format!(
                    "it is a {} document, not a {CONTRACT} one",
                    doc.contract
                )));
            }
            Ok((doc, false))
        }
    }
}

/// Version 0, the shape a draft of the journal had: `{"schema_version": 0, "steps": [<ids of finished steps>]}`,
/// with no contract name, no plan and no completion flag.
fn migrate_v0(value: &serde_json::Value) -> Result<JournalDoc, String> {
    let steps = value
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "a version 0 journal has no steps list".to_owned())?;
    let finished = steps
        .iter()
        .map(|s| {
            s.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "a step of a version 0 journal is not a text".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(JournalDoc {
        finished,
        ..Journal::blank()
    })
}

/// What a takeover reads from `nvmod`'s setup state.
struct NvmodState {
    version: String,
    plan: Vec<String>,
    finished: Vec<String>,
}

/// The steps an unfinished `nvmod setup` state holds, or `None` when that setup finished (nothing to resume).
fn nvmod_state(file: &Path, text: &str) -> Result<Option<NvmodState>, Error> {
    #[derive(Deserialize)]
    struct Entry {
        step: String,
    }
    #[derive(Deserialize)]
    struct State {
        #[serde(default)]
        nvmod_version: String,
        #[serde(default)]
        plan: Vec<Entry>,
        #[serde(default)]
        completed: Vec<Entry>,
        #[serde(default)]
        complete: bool,
    }
    let refused = |why: String| {
        Error::new(
            ErrorKind::TakeoverRefused,
            format!("{}: {why}; it was left as it is", file.display()),
        )
    };
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| refused(e.to_string()))?;
    match value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(1) => {}
        Some(v) if v > 1 => {
            return Err(refused(format!(
                "it has schema_version {v}, newer than this raul reads"
            )));
        }
        _ => {
            return Err(refused(
                "it is not a setup state this raul reads".to_owned(),
            ));
        }
    }
    let state: State = serde_json::from_value(value).map_err(|e| refused(e.to_string()))?;
    if state.complete {
        return Ok(None);
    }
    // nvmod calls the engine step `nv-rs`; every other id is the one raul uses.
    let id = |e: Entry| {
        if e.step == "nv-rs" {
            "engine".to_owned()
        } else {
            e.step
        }
    };
    Ok(Some(NvmodState {
        version: state.nvmod_version,
        plan: state.plan.into_iter().map(id).collect(),
        finished: state.completed.into_iter().map(id).collect(),
    }))
}

/// Copy `file` to `name` next to it without replacing a copy that is already there (`name`, else `name.1`, ...).
fn keep_copy(file: &Path, name: &str) -> Result<(), Error> {
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    let bytes = std::fs::read(file).map_err(|e| write_failed(file, &e))?;
    for n in 0.. {
        let target = if n == 0 {
            dir.join(name)
        } else {
            dir.join(format!("{name}.{n}"))
        };
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
        {
            Ok(mut f) => {
                return std::io::Write::write_all(&mut f, &bytes)
                    .and_then(|()| f.sync_all())
                    .map_err(|e| write_failed(&target, &e));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(write_failed(&target, &e)),
        }
    }
    unreachable!("the counter does not end")
}

/// Write `bytes` to `file` through a temporary file next to it (created new, synced, then renamed over it).
fn write_atomic(file: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| write_failed(dir, &e))?;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let mut temporary = file.to_path_buf().into_os_string();
    temporary.push(format!(".{}-{nanos}.new", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written = (|| {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        std::io::Write::write_all(&mut f, bytes)?;
        f.sync_all()?;
        std::fs::rename(&temporary, file)
    })();
    written.map_err(|e| {
        let _ = std::fs::remove_file(&temporary);
        write_failed(file, &e)
    })
}

/// Why a journaled setup stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The journal could not be opened or written.
    Journal(Error),
    /// A step failed.
    Setup(setup::Error),
}

impl RunError {
    /// The error as one block of plain text to copy into a bug report.
    #[must_use]
    pub fn copy_text(&self) -> String {
        match self {
            Self::Journal(e) => e.copy_text(),
            Self::Setup(e) => e.copy_text(),
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Journal(e) => e.fmt(f),
            Self::Setup(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RunError {}

/// Records each finished step in the journal; a write that fails stops the run before the next step.
struct Recording<'a> {
    journal: &'a mut Journal,
    inner: &'a mut dyn Progress,
    cancel: CancelToken,
    failed: Option<Error>,
}

impl Progress for Recording<'_> {
    fn report(&mut self, event: &Event) {
        match event {
            Event::Planned { steps } => self.journal.set_plan(steps),
            Event::Finished { step, .. } if self.failed.is_none() => {
                if let Err(e) = self.journal.record_finished(step) {
                    self.failed = Some(e);
                    self.cancel.cancel();
                }
            }
            _ => {}
        }
        self.inner.report(event);
    }
}

/// [`setup::run`] with the Instance's journal: the steps the journal lists as finished are skipped, each step that
/// finishes is recorded before the next one starts, and a run that ends done marks the journal complete. An
/// unfinished `nvmod setup` in the Instance is taken over first (see the module).
///
/// # Errors
/// [`RunError::Journal`] when the journal cannot be opened or a write to it fails (the run stops before the next
/// step), [`RunError::Setup`] when a step fails.
pub fn run(
    request: &Request,
    progress: &mut dyn Progress,
    cancel: &CancelToken,
) -> Result<Outcome, RunError> {
    let mut journal = Journal::open(&request.instance).map_err(RunError::Journal)?;
    let resumed = Request {
        done: journal.resume(request),
        ..request.clone()
    };
    let mut recording = Recording {
        journal: &mut journal,
        inner: progress,
        cancel: cancel.clone(),
        failed: None,
    };
    let result = setup::run(&resumed, &mut recording, cancel);
    if let Some(e) = recording.failed.take() {
        return Err(RunError::Journal(e));
    }
    let outcome = result.map_err(RunError::Setup)?;
    if matches!(outcome, Outcome::Done(_)) {
        journal.record_complete().map_err(RunError::Journal)?;
    }
    Ok(outcome)
}
