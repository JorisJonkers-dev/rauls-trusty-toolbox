// SPDX-License-Identifier: GPL-3.0-only
//! Exit classes and the error shape every screen shows.

use serde::{Deserialize, Serialize};

/// What an `nvmod` exit code means. The numbers are the ones of `nvmod-cli` (`lib.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitClass {
    /// 0: success.
    Ok,
    /// 1: the command ran and found problems it was asked to fail on.
    ProblemsFound,
    /// 2: refused before anything changed.
    Refused,
    /// 3: a verification failed; nothing was installed.
    VerificationFailed,
    /// 4: a step failed and was rolled back.
    StepFailedRolledBack,
    /// 5: the command line was not understood; for the GUI this is a bug in the GUI.
    Usage,
    /// Any other code, or a process that did not exit normally.
    Internal,
}

impl ExitClass {
    /// Classifies a process exit code.
    pub fn from_code(code: i64) -> Self {
        match code {
            0 => Self::Ok,
            1 => Self::ProblemsFound,
            2 => Self::Refused,
            3 => Self::VerificationFailed,
            4 => Self::StepFailedRolledBack,
            5 => Self::Usage,
            _ => Self::Internal,
        }
    }

    /// The exit code of the class, `None` for [`ExitClass::Internal`].
    pub fn code(self) -> Option<u8> {
        match self {
            Self::Ok => Some(0),
            Self::ProblemsFound => Some(1),
            Self::Refused => Some(2),
            Self::VerificationFailed => Some(3),
            Self::StepFailedRolledBack => Some(4),
            Self::Usage => Some(5),
            Self::Internal => None,
        }
    }

    /// A refusal or a failed verification is a result shown as a card on its screen, not a toast.
    pub fn is_result_card(self) -> bool {
        matches!(self, Self::Refused | Self::VerificationFailed)
    }

    /// The class in words, for screen readers and the copyable summary.
    pub fn words(self) -> &'static str {
        match self {
            Self::Ok => "succeeded",
            Self::ProblemsFound => "found problems",
            Self::Refused => "refused; nothing was changed",
            Self::VerificationFailed => "verification failed; nothing was installed",
            Self::StepFailedRolledBack => "a step failed and was rolled back",
            Self::Usage => "the command line was not understood",
            Self::Internal => "failed unexpectedly",
        }
    }
}

/// Whether trying again can help.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Retry {
    /// Retrying as is can work.
    Yes,
    /// It cannot.
    No,
    /// After the user fixes something, described by the hint.
    AfterFix(String),
}

/// An error as a screen shows it. The text comes from the report; the GUI composes no error prose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewError {
    /// The cause.
    pub what: String,
    /// What changed or did not change.
    pub effect: String,
    /// The class of the exit code.
    pub exit: ExitClass,
    /// The command that reproduces it.
    pub repro: Option<String>,
    /// Whether to offer a retry.
    pub retry: Retry,
}

impl ViewError {
    /// An error of `exit` class with the cause and the effect.
    pub fn new(exit: ExitClass, what: impl Into<String>, effect: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            effect: effect.into(),
            exit,
            repro: None,
            retry: Retry::No,
        }
    }
}

/// The states every screen defines (document 02): empty, loading, ready, partial and failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Load<T> {
    /// Nothing to show; the hint says what to do next.
    Empty(String),
    /// Data is on its way.
    Loading,
    /// The data.
    Ready(T),
    /// Some data and the parts that failed.
    Partial {
        /// What could be read.
        value: T,
        /// What could not.
        failed: Vec<ViewError>,
    },
    /// Nothing could be read.
    Failed(ViewError),
}

impl<T> Load<T> {
    /// The data when there is any, also when part of it failed.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Ready(v) | Self::Partial { value: v, .. } => Some(v),
            _ => None,
        }
    }

    /// True while data is on its way.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// The errors to show, none for the states without any.
    pub fn errors(&self) -> Vec<&ViewError> {
        match self {
            Self::Partial { failed, .. } => failed.iter().collect(),
            Self::Failed(e) => vec![e],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for code in 0..=5u8 {
            assert_eq!(ExitClass::from_code(i64::from(code)).code(), Some(code));
        }
        assert_eq!(ExitClass::from_code(9), ExitClass::Internal);
        assert_eq!(ExitClass::from_code(-1).code(), None);
    }

    #[test]
    fn refusals_are_cards() {
        assert!(ExitClass::Refused.is_result_card());
        assert!(ExitClass::VerificationFailed.is_result_card());
        assert!(!ExitClass::StepFailedRolledBack.is_result_card());
    }

    #[test]
    fn every_class_has_words() {
        for c in [
            ExitClass::Ok,
            ExitClass::ProblemsFound,
            ExitClass::Refused,
            ExitClass::VerificationFailed,
            ExitClass::StepFailedRolledBack,
            ExitClass::Usage,
            ExitClass::Internal,
        ] {
            assert!(!c.words().is_empty());
        }
    }

    #[test]
    fn load_states() {
        let ready: Load<u8> = Load::Ready(1);
        assert_eq!(ready.value(), Some(&1));
        assert!(ready.errors().is_empty());
        let err = ViewError::new(ExitClass::Refused, "why", "Nothing was changed");
        let partial = Load::Partial {
            value: 2u8,
            failed: vec![err.clone()],
        };
        assert_eq!(partial.value(), Some(&2));
        assert_eq!(partial.errors().len(), 1);
        let failed: Load<u8> = Load::Failed(err);
        assert!(failed.value().is_none());
        assert_eq!(failed.errors().len(), 1);
        assert!(Load::<u8>::Loading.is_loading());
        assert!(!Load::<u8>::Empty("run it".into()).is_loading());
    }
}
