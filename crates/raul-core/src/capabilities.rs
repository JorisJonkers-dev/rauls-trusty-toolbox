// SPDX-License-Identifier: GPL-3.0-only
//! What the linked engine can do, written where the Doctor reads it (#398).
//!
//! nvmod's Doctor takes the coverage of an Instance for nv-rs from `<instance>/nv-rs/capabilities.json` and warns
//! `capabilities.missing` when the file is not there. The engine is linked into `raul` (ADR-0016), so no installed
//! package brings the file any more: the engine's own text (`fork_api::CAPABILITIES`) is handed to this module once at
//! start by the program that links the engine ([`provide`]), and written into an Instance when nv-rs is made its
//! target ([`write_provided`]). A build that does not link the engine provides nothing and writes nothing.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PROVIDED: OnceLock<&'static str> = OnceLock::new();

/// Hands over the capabilities text of the engine this program links. The first call counts; a later one is ignored.
pub fn provide(text: &'static str) {
    let _ = PROVIDED.set(text);
}

/// The folder nvmod's Doctor reads for nv-rs, inside the Instance.
fn folder(instance: &Path) -> PathBuf {
    instance.join("nv-rs")
}

/// The file the Doctor reads.
#[must_use]
pub fn file(instance: &Path) -> PathBuf {
    folder(instance).join("capabilities.json")
}

/// Writes the provided capabilities text into `instance`. Returns whether the engine provided any (`false` for a build
/// without the engine: nothing is written). The file is written whole or not at all, and left alone when it already
/// holds the text.
///
/// # Errors
/// The folder or the file could not be written.
pub fn write_provided(instance: &Path) -> std::io::Result<bool> {
    match PROVIDED.get() {
        Some(text) => write(instance, text).map(|()| true),
        None => Ok(false),
    }
}

/// Writes `text` as the capabilities file of `instance`, through a temporary file next to it.
///
/// # Errors
/// The folder or the file could not be written.
pub fn write(instance: &Path, text: &str) -> std::io::Result<()> {
    let target = file(instance);
    if std::fs::read_to_string(&target).is_ok_and(|have| have == text) {
        return Ok(());
    }
    std::fs::create_dir_all(folder(instance))?;
    let temp = target.with_extension("json.part");
    let mut out = std::fs::File::create(&temp)?;
    let written = out
        .write_all(text.as_bytes())
        .and_then(|()| out.sync_all())
        .and_then(|()| std::fs::rename(&temp, &target));
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;

    #[test]
    fn the_text_is_written_where_the_doctor_reads_it_and_replaced_whole() {
        let t = TempDir::new("capabilities");
        write(t.path(), "{\"a\":1}").unwrap();
        assert_eq!(
            std::fs::read_to_string(t.path().join("nv-rs/capabilities.json")).unwrap(),
            "{\"a\":1}"
        );
        write(t.path(), "{\"a\":2}").unwrap();
        assert_eq!(
            std::fs::read_to_string(file(t.path())).unwrap(),
            "{\"a\":2}"
        );
        let names: Vec<_> = std::fs::read_dir(t.path().join("nv-rs"))
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, ["capabilities.json"]);
    }

    #[test]
    fn nothing_is_written_when_no_engine_provided_a_text() {
        // No test in this binary provides one.
        let t = TempDir::new("capabilities-none");
        assert!(!write_provided(t.path()).unwrap());
        assert!(!t.path().join("nv-rs").exists());
    }

    #[test]
    fn a_folder_that_cannot_be_made_is_an_error() {
        let t = TempDir::new("capabilities-blocked");
        std::fs::write(t.path().join("nv-rs"), b"a file, not a folder").unwrap();
        assert!(write(t.path(), "{}").is_err());
    }
}
