// SPDX-License-Identifier: GPL-3.0-only
//! The dialog layer: one modal dialog at a time over the shell (UI-RULES A2, T4, X1).
//!
//! A page opens a [`Dialog`] with an id of its own and reads the answer back with [`Dialogs::take_answer`] on a
//! later frame. Esc and the backdrop close a dialog with [`Answer::Closed`], unless it is marked as busy (files
//! are being copied); a confirm dialog states its cost and offers its action and Cancel.

use crate::catalogue::ErrorText;
use eframe::egui;

/// How a dialog was answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// The person took the dialog's action.
    Confirmed,
    /// The person closed it: Cancel, Close, Esc or the backdrop.
    Closed,
}

/// One dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dialog {
    /// The id its answer is filed under; chosen by the page that opens it.
    pub id: &'static str,
    /// Its title.
    pub title: String,
    /// Its text, one paragraph per entry.
    pub body: Vec<String>,
    /// The `raul` command the action runs. Kept for tests and callers; no longer drawn on screen.
    pub command: Option<String>,
    /// The label of its action; `None` for a dialog that only informs.
    pub action: Option<String>,
    /// Whether the action is destructive (drawn as a danger button).
    pub danger: bool,
    /// Whether the dialog must stay open (work that cannot be interrupted); Esc does nothing then.
    pub busy: bool,
    /// The error it shows, when it is one: what went wrong and its detail, drawn as selectable text with a "Copy
    /// details" button.
    pub error: Option<ErrorText>,
}

impl Dialog {
    /// Whether it must stay open until its action is taken: busy, and with an action to take. A busy dialog with no
    /// action can always be closed, so it can never trap the window.
    pub fn holds(&self) -> bool {
        self.busy && self.action.is_some()
    }

    /// A dialog that only informs, with a Close button.
    pub fn info(id: &'static str, title: impl Into<String>, body: Vec<String>) -> Dialog {
        Dialog {
            id,
            title: title.into(),
            body,
            command: None,
            action: None,
            danger: false,
            busy: false,
            error: None,
        }
    }

    /// A dialog that asks once before an action, stating its cost (UI-RULES T4).
    pub fn confirm(
        id: &'static str,
        title: impl Into<String>,
        cost: impl Into<String>,
        action: impl Into<String>,
    ) -> Dialog {
        Dialog {
            action: Some(action.into()),
            ..Dialog::info(id, title, vec![cost.into()])
        }
    }

    /// The same dialog carrying the command its action runs. The command is kept for tests and callers but is no
    /// longer drawn on screen.
    #[must_use]
    pub fn with_command(mut self, command: impl Into<String>) -> Dialog {
        self.command = Some(command.into());
        self
    }

    /// The same dialog with a destructive action.
    #[must_use]
    pub fn danger(mut self) -> Dialog {
        self.danger = true;
        self
    }

    /// The same dialog showing `error` as an error surface: its title and detail are selectable text and it has a
    /// "Copy details" button.
    #[must_use]
    pub fn with_error(mut self, error: ErrorText) -> Dialog {
        self.body = vec![error.detail.to_owned()];
        self.error = Some(error);
        self
    }
}

/// The open dialog and the answers not read yet.
#[derive(Debug, Default)]
pub struct Dialogs {
    open: Option<Dialog>,
    answers: Vec<(&'static str, Answer)>,
}

impl Dialogs {
    /// Opens `dialog`. A dialog that is already open is replaced and counts as closed, unless it is busy: then the
    /// new one is not opened and counts as closed itself.
    pub fn open(&mut self, dialog: Dialog) {
        if self.open.as_ref().is_some_and(Dialog::holds) {
            self.answers.push((dialog.id, Answer::Closed));
            return;
        }
        if let Some(old) = self.open.take() {
            self.answers.push((old.id, Answer::Closed));
        }
        self.open = Some(dialog);
    }

    /// The open dialog.
    pub fn current(&self) -> Option<&Dialog> {
        self.open.as_ref()
    }

    /// Whether a dialog is open.
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// The answer to the dialog `id`, once: a second call returns `None`.
    pub fn take_answer(&mut self, id: &str) -> Option<Answer> {
        let at = self.answers.iter().position(|(i, _)| *i == id)?;
        Some(self.answers.remove(at).1)
    }

    /// Answers the open dialog. A busy dialog with an action cannot be closed this way, only confirmed.
    pub fn answer(&mut self, answer: Answer) {
        if let Some(d) = &self.open
            && (answer == Answer::Confirmed || !d.holds())
        {
            self.answers.push((d.id, answer));
            self.open = None;
        }
    }

    /// Draws the open dialog, if any, over everything else.
    pub fn show(&mut self, ctx: &egui::Context, tokens: &crate::theme::Tokens) {
        let Some(dialog) = self.open.clone() else {
            return;
        };
        let frame = egui::Frame::new()
            .fill(tokens.raise.color())
            .stroke(egui::Stroke::new(1.0, tokens.line.color()))
            .corner_radius(12)
            .inner_margin(egui::Margin::symmetric(22, 20));
        let mut chosen = None;
        let response = egui::Modal::new(egui::Id::unique(("dialog", dialog.id)))
            .frame(frame)
            .backdrop_color(egui::Color32::from_black_alpha(184))
            .show(ctx, |ui| {
                ui.set_max_width(560.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&dialog.title)
                            .text_style(crate::theme::title_style())
                            .size(crate::theme::base(ctx) * 1.1),
                    )
                    .wrap()
                    .selectable(true),
                );
                ui.add_space(6.0);
                if let Some(error) = &dialog.error {
                    // The dialog's own paragraphs are the detail; the shared widget adds the copy button.
                    let detail = dialog.body.join("\n");
                    crate::shell::error_widget(ui, tokens, error.detail, error.what, &detail);
                } else {
                    for paragraph in &dialog.body {
                        ui.add(egui::Label::new(paragraph).wrap().selectable(true));
                    }
                }
                ui.add_space(12.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(action) = &dialog.action {
                        let button = if dialog.danger {
                            crate::shell::danger_button(tokens, action)
                        } else {
                            crate::shell::primary_button(tokens, action)
                        };
                        if ui.add(button).clicked() {
                            chosen = Some(Answer::Confirmed);
                        }
                        if !dialog.busy && ui.button(crate::catalogue::CANCEL).clicked() {
                            chosen = Some(Answer::Closed);
                        }
                    } else if ui.button(crate::catalogue::CLOSE).clicked() {
                        chosen = Some(Answer::Closed);
                    }
                });
            });
        if chosen.is_none() && response.should_close() {
            chosen = Some(Answer::Closed);
        }
        if let Some(answer) = chosen {
            self.answer(answer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_is_read_once() {
        let mut d = Dialogs::default();
        d.open(Dialog::confirm("remove", "Remove?", "It costs.", "Remove"));
        assert!(d.is_open());
        d.answer(Answer::Confirmed);
        assert!(!d.is_open());
        assert_eq!(d.take_answer("remove"), Some(Answer::Confirmed));
        assert_eq!(d.take_answer("remove"), None);
    }

    #[test]
    fn a_replaced_dialog_counts_as_closed() {
        let mut d = Dialogs::default();
        d.open(Dialog::info("a", "A", vec![]));
        d.open(Dialog::info("b", "B", vec![]));
        assert_eq!(d.take_answer("a"), Some(Answer::Closed));
        assert_eq!(d.current().map(|x| x.id), Some("b"));
    }

    #[test]
    fn a_busy_dialog_stays_open_until_confirmed() {
        let mut d = Dialogs::default();
        let mut busy = Dialog::confirm("copy", "Copying", "Files are being copied.", "Hide");
        busy.busy = true;
        d.open(busy);
        d.answer(Answer::Closed);
        assert!(d.is_open());
        d.answer(Answer::Confirmed);
        assert_eq!(d.take_answer("copy"), Some(Answer::Confirmed));
    }

    #[test]
    fn a_busy_dialog_with_no_action_can_still_be_closed() {
        let mut d = Dialogs::default();
        let mut busy = Dialog::info("wait", "Wait", vec![]);
        busy.busy = true;
        d.open(busy);
        d.answer(Answer::Closed);
        assert!(!d.is_open());
    }

    #[test]
    fn a_busy_dialog_is_not_replaced() {
        let mut d = Dialogs::default();
        let mut busy = Dialog::confirm("copy", "Copying", "Files are being copied.", "Hide");
        busy.busy = true;
        d.open(busy);
        d.open(Dialog::info("about", "About", vec![]));
        assert_eq!(d.current().map(|x| x.id), Some("copy"));
        assert_eq!(d.take_answer("about"), Some(Answer::Closed));
        assert_eq!(d.take_answer("copy"), None);
    }

    #[test]
    fn an_error_dialog_shows_the_error_and_its_detail() {
        let e = crate::catalogue::PLAY_NO_GAME;
        let d = Dialog::confirm("play", "Blocked", "gone", "Fix").with_error(e);
        assert_eq!(d.error, Some(e));
        assert_eq!(d.body, [e.detail]);
        assert!(Dialog::info("about", "About", vec![]).error.is_none());
    }

    #[test]
    fn builders_set_the_command_and_the_danger_flag() {
        let d = Dialog::confirm("x", "T", "C", "Go")
            .with_command("raul rollback")
            .danger();
        assert_eq!(d.command.as_deref(), Some("raul rollback"));
        assert!(d.danger);
    }
}
