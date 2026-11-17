// SPDX-License-Identifier: GPL-3.0-only
//! The state of the Doctor screen: the run, the findings, the severity filter and the selected row.

use crate::error::{Load, ViewError};
use serde::{Deserialize, Serialize};

/// How serious a finding is. The order is the order of the filter: `Info` is the least serious.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// A note.
    Info,
    /// Something to look at.
    Warning,
    /// Something that must be fixed.
    Error,
}

impl Severity {
    /// Reads the word the command line prints (`info`, `warning`, `error`). An unknown word is shown as an
    /// error, so a new severity is never hidden by a filter.
    pub fn parse(word: &str) -> Self {
        match word {
            "info" => Self::Info,
            "warning" => Self::Warning,
            _ => Self::Error,
        }
    }

    /// The word for a screen reader and the copyable summary.
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "note",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// One finding as the screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorFinding {
    /// Stable identifier, `<area>.<what>`.
    pub code: String,
    /// How serious it is.
    pub severity: Severity,
    /// What it is about.
    pub subject: String,
    /// What was found.
    pub message: String,
}

/// What a Doctor run found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorView {
    /// The game folder the Instance runs on.
    pub base: String,
    /// The findings, in the order the checks ran.
    pub findings: Vec<DoctorFinding>,
}

/// The one-word verdict of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Nothing was found.
    Healthy,
    /// Only notes.
    Notes,
    /// Warnings, no errors.
    Warnings,
    /// At least one error.
    Errors,
}

impl DoctorView {
    /// Counts of notes, warnings and errors.
    pub fn counts(&self) -> (usize, usize, usize) {
        let n = |s| self.findings.iter().filter(|f| f.severity == s).count();
        (n(Severity::Info), n(Severity::Warning), n(Severity::Error))
    }

    /// The verdict.
    pub fn verdict(&self) -> Verdict {
        match self.findings.iter().map(|f| f.severity).max() {
            None => Verdict::Healthy,
            Some(Severity::Info) => Verdict::Notes,
            Some(Severity::Warning) => Verdict::Warnings,
            Some(Severity::Error) => Verdict::Errors,
        }
    }
}

/// The refusals of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorError {
    /// The row is not in the filtered list.
    NoSuchRow(usize),
}

/// The Doctor screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorScreen {
    load: Load<DoctorView>,
    min: Severity,
    selected: Option<usize>,
}

impl Default for DoctorScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorScreen {
    /// A screen that has not run yet.
    pub fn new() -> Self {
        Self {
            load: Load::Empty("Run the Doctor to check this Instance".to_owned()),
            min: Severity::Info,
            selected: None,
        }
    }

    /// The load state.
    pub fn load(&self) -> &Load<DoctorView> {
        &self.load
    }

    /// True while a run is going.
    pub fn is_running(&self) -> bool {
        self.load.is_loading()
    }

    /// The Run button was pressed. Refused (`false`) while a run is going.
    pub fn begin_run(&mut self) -> bool {
        if self.is_running() {
            return false;
        }
        self.load = Load::Loading;
        self.selected = None;
        true
    }

    /// The run produced a report.
    pub fn finish_ok(&mut self, view: DoctorView) {
        self.load = Load::Ready(view);
        self.selected = None;
    }

    /// The run failed. The screen shows the failure; a report from an earlier run stays on screen next to it.
    /// `previous` is that report, if the caller kept one.
    pub fn finish_err(&mut self, error: ViewError, previous: Option<DoctorView>) {
        self.load = match previous {
            Some(value) => Load::Partial {
                value,
                failed: vec![error],
            },
            None => Load::Failed(error),
        };
        self.selected = None;
    }

    /// Shows only findings at or above `min`. The selection is dropped, since the rows move.
    pub fn set_filter(&mut self, min: Severity) {
        self.min = min;
        self.selected = None;
    }

    /// The lowest severity shown.
    pub fn filter(&self) -> Severity {
        self.min
    }

    /// The findings shown: at or above the filter, the most serious first and in check order within a severity.
    pub fn visible(&self) -> Vec<&DoctorFinding> {
        let mut rows: Vec<&DoctorFinding> = self.load.value().map_or_default(|v| {
            v.findings
                .iter()
                .filter(|f| f.severity >= self.min)
                .collect()
        });
        rows.sort_by_key(|f| std::cmp::Reverse(f.severity));
        rows
    }

    /// Selects a row of [`DoctorScreen::visible`].
    ///
    /// # Errors
    /// [`DoctorError::NoSuchRow`] when there is no such row.
    pub fn select(&mut self, row: usize) -> Result<(), DoctorError> {
        if row >= self.visible().len() {
            return Err(DoctorError::NoSuchRow(row));
        }
        self.selected = Some(row);
        Ok(())
    }

    /// The selected finding.
    pub fn selected(&self) -> Option<&DoctorFinding> {
        self.visible().get(self.selected?).copied()
    }

    /// What to show in place of the list when there are no rows, `None` when there are rows.
    pub fn empty_hint(&self) -> Option<String> {
        match &self.load {
            Load::Empty(hint) => Some(hint.clone()),
            Load::Ready(v) | Load::Partial { value: v, .. } if self.visible().is_empty() => {
                Some(if v.findings.is_empty() {
                    "The Doctor found nothing".to_owned()
                } else {
                    "Nothing at this severity; lower the filter to see more".to_owned()
                })
            }
            _ => None,
        }
    }

    /// The counts in words, "1 error, 2 warnings, 0 notes", for the header and a screen reader.
    pub fn summary_line(&self) -> Option<String> {
        let (i, w, e) = self.load.value()?.counts();
        Some(format!(
            "{e} {}, {w} {}, {i} {}",
            plural(e, "error", "errors"),
            plural(w, "warning", "warnings"),
            plural(i, "note", "notes")
        ))
    }

    /// The text of Copy report: the summary and one line per finding, nothing else.
    pub fn report_text(&self) -> Option<String> {
        let view = self.load.value()?;
        let mut lines = vec![format!("Doctor: {}", self.summary_line()?)];
        let mut all: Vec<&DoctorFinding> = view.findings.iter().collect();
        all.sort_by_key(|f| std::cmp::Reverse(f.severity));
        for f in all {
            lines.push(format!(
                "{} {} {}: {}",
                f.severity.label(),
                f.code,
                f.subject,
                f.message
            ));
        }
        Some(lines.join("\n"))
    }
}

fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExitClass;

    fn finding(code: &str, severity: Severity) -> DoctorFinding {
        DoctorFinding {
            code: code.to_owned(),
            severity,
            subject: "S".to_owned(),
            message: format!("{code} happened"),
        }
    }

    fn view() -> DoctorView {
        DoctorView {
            base: "/base".to_owned(),
            findings: vec![
                finding("a.note", Severity::Info),
                finding("b.err", Severity::Error),
                finding("c.warn", Severity::Warning),
                finding("d.err", Severity::Error),
            ],
        }
    }

    fn err() -> ViewError {
        ViewError::new(ExitClass::Refused, "not an Instance", "Nothing was changed")
    }

    #[test]
    fn a_new_screen_asks_for_a_run_and_a_run_cannot_start_twice() {
        let mut s = DoctorScreen::new();
        assert!(s.empty_hint().unwrap().contains("Run the Doctor"));
        assert!(s.begin_run());
        assert!(s.is_running());
        assert!(!s.begin_run());
        s.finish_ok(view());
        assert!(!s.is_running());
        assert!(s.begin_run());
    }

    #[test]
    fn rows_are_most_serious_first_and_the_filter_hides_the_rest() {
        let mut s = DoctorScreen::new();
        s.finish_ok(view());
        let codes = |s: &DoctorScreen| -> Vec<String> {
            s.visible().iter().map(|f| f.code.clone()).collect()
        };
        assert_eq!(codes(&s), ["b.err", "d.err", "c.warn", "a.note"]);
        s.set_filter(Severity::Warning);
        assert_eq!(codes(&s), ["b.err", "d.err", "c.warn"]);
        s.set_filter(Severity::Error);
        assert_eq!(codes(&s), ["b.err", "d.err"]);
        assert_eq!(s.filter(), Severity::Error);
    }

    #[test]
    fn selection_follows_the_visible_rows_and_is_dropped_when_they_move() {
        let mut s = DoctorScreen::new();
        s.finish_ok(view());
        s.select(2).unwrap();
        assert_eq!(s.selected().unwrap().code, "c.warn");
        assert_eq!(s.select(9), Err(DoctorError::NoSuchRow(9)));
        s.set_filter(Severity::Warning);
        assert!(s.selected().is_none());
    }

    #[test]
    fn counts_verdict_and_summary_are_in_words() {
        let mut s = DoctorScreen::new();
        assert!(s.summary_line().is_none());
        s.finish_ok(view());
        assert_eq!(s.summary_line().unwrap(), "2 errors, 1 warning, 1 note");
        assert_eq!(view().verdict(), Verdict::Errors);
        let clean = DoctorView {
            base: String::new(),
            findings: Vec::new(),
        };
        assert_eq!(clean.verdict(), Verdict::Healthy);
        s.finish_ok(clean);
        assert_eq!(s.empty_hint().unwrap(), "The Doctor found nothing");
        let notes = DoctorView {
            base: String::new(),
            findings: vec![finding("n", Severity::Info)],
        };
        assert_eq!(notes.verdict(), Verdict::Notes);
        s.finish_ok(notes);
        s.set_filter(Severity::Error);
        assert!(s.empty_hint().unwrap().contains("lower the filter"));
    }

    #[test]
    fn a_failed_run_keeps_the_earlier_report_beside_the_error() {
        let mut s = DoctorScreen::new();
        s.finish_err(err(), None);
        assert!(matches!(s.load(), Load::Failed(_)));
        assert!(s.visible().is_empty());
        s.finish_err(err(), Some(view()));
        assert_eq!(s.load().errors().len(), 1);
        assert_eq!(s.visible().len(), 4);
    }

    #[test]
    fn the_report_text_lists_every_finding_whatever_the_filter() {
        let mut s = DoctorScreen::new();
        assert!(s.report_text().is_none());
        s.finish_ok(view());
        s.set_filter(Severity::Error);
        let text = s.report_text().unwrap();
        assert!(text.starts_with("Doctor: 2 errors, 1 warning, 1 note\nerror b.err S:"));
        assert_eq!(text.lines().count(), 5);
        assert_eq!(Severity::parse("surprise"), Severity::Error);
        assert_eq!(Severity::parse("info"), Severity::Info);
    }
}
