// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of installing a Mod from an archive on the Mods page: the archive, the plan that asks
//! the FOMOD's questions when there are any, the install, and how it ended.
//!
//! The first step is a plan (`raul install <archive> --dry-run`): nothing is written. An archive with no questions
//! goes straight to the install. One with questions shows them, and the install runs with the answers (a file that
//! maps each question to the options chosen). A stop or a refusal leaves the mod list as it was.

use super::settings::Command;
use crate::wizard::{FomodQuestion, FomodWizard};

/// `raul install <archive> --instance <dir> [--answers <file>] [--dry-run]`.
pub fn install_command(
    instance: &str,
    archive: &str,
    answers: Option<&str>,
    dry_run: bool,
) -> Command {
    let mut words = vec!["install", archive, "--instance", instance];
    if let Some(file) = answers {
        words.extend(["--answers", file]);
    }
    if dry_run {
        words.push("--dry-run");
    }
    Command::write("install", 1, &words)
}

/// The archive extensions the downloads list offers.
pub const ARCHIVE_EXTENSIONS: [&str; 4] = ["zip", "7z", "rar", "fomod"];

/// Whether a file name looks like a Mod archive.
pub fn is_archive(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        ARCHIVE_EXTENSIONS
            .iter()
            .any(|e| ext.eq_ignore_ascii_case(e))
    })
}

/// Where the install is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Nothing is open: only the Install button shows.
    #[default]
    Closed,
    /// The archive is being chosen.
    Choosing,
    /// The plan runs.
    Planning,
    /// The archive asks questions.
    Asking,
    /// The install runs.
    Installing,
    /// A stop was asked; the job ends at its next safe point.
    Stopping,
    /// The Mod is installed.
    Done,
    /// The install did not happen.
    Failed,
}

/// What an installed Mod is, for the line that says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The Mod's folder name.
    pub name: String,
    /// How many files it has.
    pub files: usize,
    /// The plugins it activated.
    pub plugins: Vec<String>,
}

/// The state of the install.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// Where it is.
    pub phase: Phase,
    /// The archive path as typed or picked.
    pub archive: String,
    /// The archives found in the downloads folder.
    pub found: Vec<String>,
    /// The FOMOD questions and the answers so far.
    pub questions: Option<FomodWizard>,
    /// The Mod that was installed.
    pub installed: Option<Installed>,
    /// Why it failed: the command's last words.
    pub problem: Option<String>,
}

impl State {
    /// The chooser opens.
    pub fn open(&mut self, found: Vec<String>) {
        *self = State {
            phase: Phase::Choosing,
            archive: std::mem::take(&mut self.archive),
            found,
            ..State::default()
        };
    }

    /// The install is put away.
    pub fn close(&mut self) {
        *self = State::default();
    }

    /// Picks `path` as the archive.
    pub fn pick(&mut self, path: &str) {
        path.clone_into(&mut self.archive);
    }

    /// Whether Install may start: an archive is named and nothing runs.
    pub fn can_start(&self) -> bool {
        matches!(self.phase, Phase::Choosing | Phase::Failed) && !self.archive.trim().is_empty()
    }

    /// The archive, without the spaces around it.
    pub fn archive_path(&self) -> &str {
        self.archive.trim()
    }

    /// The plan starts.
    pub fn planning(&mut self) {
        self.phase = Phase::Planning;
        self.questions = None;
        self.installed = None;
        self.problem = None;
    }

    /// The plan, or an install, found questions to ask.
    pub fn asking(&mut self, questions: Vec<FomodQuestion>) {
        self.phase = Phase::Asking;
        self.questions = Some(FomodWizard::new(questions));
    }

    /// The install starts.
    pub fn installing(&mut self) {
        self.phase = Phase::Installing;
        self.problem = None;
    }

    /// Stop was asked.
    pub fn stopping(&mut self) {
        if matches!(self.phase, Phase::Planning | Phase::Installing) {
            self.phase = Phase::Stopping;
        }
    }

    /// The Mod is installed.
    pub fn done(&mut self, installed: Installed) {
        self.phase = Phase::Done;
        self.questions = None;
        self.installed = Some(installed);
    }

    /// The install did not happen, or stopped; the mod list is as it was.
    pub fn failed(&mut self, why: String) {
        self.phase = Phase::Failed;
        self.questions = None;
        self.problem = Some(why);
    }

    /// Whether a command of the install runs.
    pub fn busy(&self) -> bool {
        matches!(
            self.phase,
            Phase::Planning | Phase::Installing | Phase::Stopping
        )
    }

    /// Whether every question has an answer, so the install can start.
    pub fn answered(&self) -> bool {
        self.questions
            .as_ref()
            .is_some_and(FomodWizard::is_complete)
    }

    /// The answers file's text, when there are questions.
    pub fn answers_text(&self) -> Option<String> {
        self.questions
            .as_ref()
            .map(|q| q.answers_json().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::{FomodOption, GroupRule};

    fn question() -> FomodQuestion {
        FomodQuestion {
            id: "Main/Look".into(),
            step: "Main".into(),
            group: "Look".into(),
            rule: GroupRule::SelectExactlyOne,
            min: 1,
            max: 1,
            options: vec![FomodOption {
                name: "Plain".into(),
                description: String::new(),
                kind: None,
            }],
        }
    }

    #[test]
    fn the_command_names_the_archive_first_and_the_flags_after() {
        let c = install_command("/i", "/d/a b.zip", Some("/t/answers.json"), true);
        assert_eq!(
            c.words,
            [
                "install",
                "/d/a b.zip",
                "--instance",
                "/i",
                "--answers",
                "/t/answers.json",
                "--dry-run"
            ]
        );
        assert_eq!(c.rest()[0], "/d/a b.zip");
        assert!(c.writes);
    }

    #[test]
    fn archives_are_told_by_their_extension() {
        assert!(is_archive("Mod.ZIP"));
        assert!(is_archive("a.7z"));
        assert!(!is_archive("notes.txt"));
        assert!(!is_archive("zip"));
    }

    #[test]
    fn the_steps_follow_each_other_and_a_failure_keeps_the_archive() {
        let mut s = State::default();
        assert!(!s.can_start());
        s.open(vec![]);
        s.pick(" /d/a.zip ");
        assert!(s.can_start());
        assert_eq!(s.archive_path(), "/d/a.zip");
        s.planning();
        assert!(s.busy() && !s.can_start());
        s.asking(vec![question()]);
        assert_eq!(s.phase, Phase::Asking);
        assert!(!s.answered());
        s.questions
            .as_mut()
            .unwrap()
            .answer("Main/Look", &["Plain"])
            .unwrap();
        assert!(s.answered());
        assert!(s.answers_text().unwrap().contains("Plain"));
        s.installing();
        s.stopping();
        assert_eq!(s.phase, Phase::Stopping);
        s.failed("the command was stopped".into());
        assert!(s.can_start());
        assert_eq!(s.archive_path(), "/d/a.zip");
        assert!(s.questions.is_none());
    }

    #[test]
    fn opening_again_keeps_the_archive_and_clears_the_rest() {
        let mut s = State::default();
        s.open(vec![]);
        s.pick("/d/a.zip");
        s.failed("no".into());
        s.open(vec!["b.zip".into()]);
        assert_eq!(s.archive, "/d/a.zip");
        assert!(s.problem.is_none());
        assert_eq!(s.found, ["b.zip"]);
    }
}
