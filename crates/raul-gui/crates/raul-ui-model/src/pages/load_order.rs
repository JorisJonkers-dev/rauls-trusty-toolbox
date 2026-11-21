// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the Load order page: the plugin list of the Profile, the sort preview with the moves
//! it makes, Apply, and Undo.
//!
//! The page drives it with the answers of `raul plugins list`, `raul sort` (a preview, then `--apply`) and
//! `raul rollback`. The moves are computed here from the current and the proposed order: a plugin moves when its
//! place is not the same in both, the same count `raul sort` prints ("15 of 18 position(s) would change"), so the
//! Move column, the header and the Apply button agree with it.

use super::settings::Command;
use std::collections::HashMap;

/// `raul plugins list --instance <dir>`: the Profile's plugins in Load order.
pub fn list_command(instance: &str) -> Command {
    Command::read(
        "plugins-list",
        2,
        &["plugins", "list", "--instance", instance],
    )
}

/// `raul sort --instance <dir> --fetch-masterlist`: the proposed order, with the pinned masterlist; nothing is
/// written.
pub fn preview_command(instance: &str) -> Command {
    Command::read(
        "sort-preview",
        1,
        &["sort", "--instance", instance, "--fetch-masterlist"],
    )
}

/// `raul sort --instance <dir> --fetch-masterlist --apply`: writes the proposed order; `raul rollback` undoes it.
pub fn apply_command(instance: &str) -> Command {
    Command::write(
        "sort-apply",
        1,
        &[
            "sort",
            "--instance",
            instance,
            "--fetch-masterlist",
            "--apply",
        ],
    )
}

/// `raul rollback --instance <dir>`: undoes the last change of the Instance, here the applied sort.
pub fn undo_command(instance: &str) -> Command {
    Command::write("rollback", 1, &["rollback", "--instance", instance])
}

/// What a masterlist message or a sorter finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageKind {
    /// For information.
    Note,
    /// Something to look at.
    Warning,
    /// A problem.
    Error,
}

impl MessageKind {
    /// The kind of a masterlist message, from its `kind` word (`say`, `warn`, `error`).
    pub fn parse(word: &str) -> MessageKind {
        match word.to_ascii_lowercase().as_str() {
            "warn" | "warning" => MessageKind::Warning,
            "error" => MessageKind::Error,
            _ => MessageKind::Note,
        }
    }
}

/// A message about one plugin or the whole list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// What it is.
    pub kind: MessageKind,
    /// The text, as the masterlist or the sorter wrote it.
    pub text: String,
}

/// One plugin of the Load order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plugin {
    /// The file name.
    pub name: String,
    /// Active in the Profile.
    pub active: bool,
    /// The masterlist group, when the sort named one.
    pub group: Option<String>,
    /// The masterlist messages and the sorter's findings.
    pub messages: Vec<Message>,
}

impl Plugin {
    /// An active plugin with no group and no messages.
    pub fn named(name: impl Into<String>) -> Plugin {
        Plugin {
            name: name.into(),
            active: true,
            ..Plugin::default()
        }
    }

    /// The most serious of its messages.
    pub fn worst(&self) -> Option<MessageKind> {
        self.messages.iter().map(|m| m.kind).max()
    }
}

/// A plugin the sort moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    /// The plugin.
    pub name: String,
    /// Its place now, 1 first.
    pub from: usize,
    /// Its place after the sort.
    pub to: usize,
}

impl Move {
    /// Whether it moves towards the top (loads earlier).
    pub fn up(&self) -> bool {
        self.to < self.from
    }
}

/// The plugins that move between `current` and `proposed`: every plugin of both whose place differs, with the place
/// it has now and the one it gets. A plugin only in `proposed` is not a move.
pub fn moves(current: &[Plugin], proposed: &[Plugin]) -> Vec<Move> {
    let at: HashMap<&str, usize> = current
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.as_str(), i))
        .collect();
    proposed
        .iter()
        .enumerate()
        .filter_map(|(to, p)| {
            at.get(p.name.as_str())
                .filter(|from| **from != to)
                .map(|from| Move {
                    name: p.name.clone(),
                    from: from + 1,
                    to: to + 1,
                })
        })
        .collect()
}

/// Where the page is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// No Instance is known yet: nothing can be listed.
    #[default]
    NoInstance,
    /// The plugin list is being read.
    Loading,
    /// The plugin list is shown.
    Listed,
    /// The sort preview is being computed.
    Sorting,
    /// The preview is shown; nothing has changed.
    Preview,
    /// The sorted order is being written.
    Applying,
    /// The sorted order was written; Undo restores the one before.
    Applied,
    /// The applied sort is being undone.
    Undoing,
}

/// Why the last step did not work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The catalogue id of the error the page shows.
    pub id: &'static str,
    /// The cause, in the command's words.
    pub cause: String,
    /// The command that reproduces it.
    pub command: Option<String>,
}

/// The state of the Load order page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// Where it is.
    pub phase: Phase,
    /// The Instance the list was read from.
    pub instance: Option<String>,
    /// The Profile, once a command named it.
    pub profile: Option<String>,
    /// The order the Profile has now.
    pub current: Vec<Plugin>,
    /// The order the sort proposes, while a preview is shown.
    pub proposed: Vec<Plugin>,
    /// Messages about the list as a whole, and the plugins the Profile names that are not installed.
    pub notes: Vec<Message>,
    /// The last problem, shown until the next step starts.
    pub problem: Option<Problem>,
    /// The journal of the applied sort: Undo is offered while it is set.
    pub journal: Option<String>,
    /// The moves of the preview shown, computed once when it arrived; read them with [`State::moves`].
    pub preview_moves: Vec<Move>,
    /// How many errors of the load order the Doctor reports for the list read (a master after the plugin that needs
    /// it); `None` until the Doctor answered. See [`State::unsorted`].
    pub order_errors: Option<usize>,
}

impl State {
    /// Where the page rests when nothing runs: the applied sort while it can be undone, the list otherwise.
    fn resting(&self) -> Phase {
        if self.journal.is_some() {
            Phase::Applied
        } else {
            Phase::Listed
        }
    }

    /// The list is read (again).
    pub fn loading(&mut self) {
        self.phase = Phase::Loading;
        self.problem = None;
    }

    /// The plugin list arrived.
    pub fn listed(&mut self, profile: String, plugins: Vec<Plugin>) {
        self.profile = Some(profile);
        let mut plugins = plugins;
        // After an applied sort the list is read again, and `raul plugins list` carries no group or messages: the
        // ones of the sort stay on their plugins until the sort is undone.
        if self.journal.is_some() {
            let known: HashMap<&str, &Plugin> =
                self.current.iter().map(|p| (p.name.as_str(), p)).collect();
            for p in &mut plugins {
                if let Some(old) = known.get(p.name.as_str()) {
                    p.group.clone_from(&old.group);
                    p.messages.clone_from(&old.messages);
                }
            }
        }
        self.current = plugins;
        self.proposed.clear();
        self.phase = self.resting();
    }

    /// The sort preview was started.
    pub fn sorting(&mut self) {
        self.phase = Phase::Sorting;
        self.problem = None;
    }

    /// The preview arrived.
    pub fn previewed(&mut self, proposed: Vec<Plugin>, notes: Vec<Message>) {
        // The proposed rows carry the group and the messages; the current rows take them too.
        let by_name: HashMap<&str, &Plugin> =
            proposed.iter().map(|q| (q.name.as_str(), q)).collect();
        for p in &mut self.current {
            let q = by_name.get(p.name.as_str());
            p.group = q.and_then(|q| q.group.clone());
            p.messages = q.map_or_else(Vec::new, |q| q.messages.clone());
        }
        self.preview_moves = moves(&self.current, &proposed);
        self.proposed = proposed;
        self.notes = notes;
        self.phase = Phase::Preview;
    }

    /// The moves of the preview shown; none outside a preview.
    pub fn moves(&self) -> &[Move] {
        if matches!(self.phase, Phase::Preview | Phase::Applying) {
            &self.preview_moves
        } else {
            &[]
        }
    }

    /// Cancel on the preview: back to the list; nothing changed.
    pub fn cancel_preview(&mut self) {
        if self.phase == Phase::Preview {
            self.proposed.clear();
            self.phase = self.resting();
        }
    }

    /// Apply was started.
    pub fn applying(&mut self) {
        self.phase = Phase::Applying;
        self.problem = None;
    }

    /// The sort was written. `journal` is the one `raul rollback` undoes, `None` when nothing had to change.
    pub fn applied(&mut self, journal: Option<String>) {
        if !self.proposed.is_empty() {
            self.current = std::mem::take(&mut self.proposed);
        }
        self.journal = journal;
        self.phase = self.resting();
    }

    /// Undo was started.
    pub fn undoing(&mut self) {
        self.phase = Phase::Undoing;
        self.problem = None;
    }

    /// The undo went through; the list is read again.
    pub fn undone(&mut self) {
        self.journal = None;
        self.phase = Phase::Loading;
    }

    /// The step that ran failed: back to where it started, with the problem shown.
    pub fn failed(&mut self, problem: Problem) {
        self.phase = match self.phase {
            Phase::Applying => Phase::Preview,
            Phase::Loading | Phase::Sorting | Phase::Undoing => self.resting(),
            other => other,
        };
        self.problem = Some(problem);
    }

    /// Whether a command of this page runs.
    pub fn busy(&self) -> bool {
        matches!(
            self.phase,
            Phase::Loading | Phase::Sorting | Phase::Applying | Phase::Undoing
        )
    }

    /// How many errors of the load order the list has, when it rests as it was read and the Doctor found some: the
    /// page warns that the list is not in a working order until a sort puts it right.
    pub fn unsorted(&self) -> Option<usize> {
        (self.phase == Phase::Listed)
            .then_some(self.order_errors)
            .flatten()
            .filter(|n| *n > 0)
    }

    /// How many plugins are active.
    pub fn active(&self) -> usize {
        self.current.iter().filter(|p| p.active).count()
    }

    /// How many plugins have a warning or an error.
    pub fn flagged(&self) -> usize {
        self.current
            .iter()
            .filter(|p| p.worst() >= Some(MessageKind::Warning))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(names: &[&str]) -> Vec<Plugin> {
        names.iter().map(|n| Plugin::named(*n)).collect()
    }

    fn problem() -> Problem {
        Problem {
            id: "x",
            cause: "it broke".into(),
            command: None,
        }
    }

    #[test]
    fn a_plugin_moved_up_moves_every_plugin_it_passes_down_one_place() {
        let now = list(&["A.esm", "B.esp", "C.esp", "D.esp", "E.esp"]);
        let then = list(&["A.esm", "E.esp", "B.esp", "C.esp", "D.esp"]);
        let m = moves(&now, &then);
        let places: Vec<(&str, usize, usize)> =
            m.iter().map(|m| (m.name.as_str(), m.from, m.to)).collect();
        assert_eq!(
            places,
            [
                ("E.esp", 5, 2),
                ("B.esp", 2, 3),
                ("C.esp", 3, 4),
                ("D.esp", 4, 5)
            ]
        );
        assert!(m[0].up());
        assert!(!m[1].up());
    }

    #[test]
    fn the_same_order_has_no_moves_and_a_plugin_that_stays_is_not_one() {
        let now = list(&["A", "B", "C"]);
        assert!(moves(&now, &now).is_empty());
        let m = moves(&now, &list(&["A", "C", "B"]));
        assert_eq!(m.len(), 2, "A stays, C and B change place");
        assert!(m.iter().all(|m| m.name != "A" && m.from != m.to));
        assert!(moves(&[], &now).is_empty(), "new plugins are not moves");
        assert_eq!(
            moves(&now, &list(&["C", "B", "A"])).len(),
            2,
            "B keeps the middle"
        );
    }

    #[test]
    fn the_count_is_the_number_of_plugins_whose_place_changes() {
        // The shape of the report: 18 plugins, 15 change place.
        let now: Vec<Plugin> = (1..=18)
            .map(|i| Plugin::named(format!("P{i:02}")))
            .collect();
        let mut then = now.clone();
        then.rotate_left(1);
        then.swap(0, 17);
        let changed = then
            .iter()
            .enumerate()
            .filter(|(i, p)| now[*i].name != p.name)
            .count();
        assert_eq!(moves(&now, &then).len(), changed);
    }

    #[test]
    fn a_list_with_load_order_errors_is_unsorted_until_it_is_previewed_or_applied() {
        let mut s = State::default();
        s.listed("Default".to_owned(), list(&["A.esp", "B.esp"]));
        assert_eq!(s.unsorted(), None, "the Doctor has not answered");
        s.order_errors = Some(0);
        assert_eq!(s.unsorted(), None, "no error");
        s.order_errors = Some(21);
        assert_eq!(s.unsorted(), Some(21));
        s.sorting();
        assert_eq!(s.unsorted(), None, "not while a sort runs");
        s.previewed(list(&["B.esp", "A.esp"]), Vec::new());
        assert_eq!(s.unsorted(), None, "the preview has its own note");
        s.applying();
        s.applied(Some("j".to_owned()));
        assert_eq!(
            s.unsorted(),
            None,
            "an applied sort is not shown as unsorted"
        );
    }

    #[test]
    fn preview_apply_and_undo_walk_the_phases() {
        let mut s = State::default();
        assert_eq!(s.phase, Phase::NoInstance);
        s.loading();
        assert!(s.busy());
        s.listed("Default".into(), list(&["A", "B", "C"]));
        assert_eq!((s.phase, s.active()), (Phase::Listed, 3));
        assert!(s.moves().is_empty());
        s.sorting();
        let mut proposed = list(&["A", "C", "B"]);
        proposed[1].messages.push(Message {
            kind: MessageKind::parse("warn"),
            text: "check this".into(),
        });
        s.previewed(proposed, Vec::new());
        assert_eq!(s.moves().len(), 2, "C and B change place");
        assert_eq!(s.flagged(), 1, "the current rows took the messages");
        s.applying();
        s.applied(Some("j1".into()));
        assert_eq!(s.phase, Phase::Applied);
        assert_eq!(s.current[1].name, "C");
        assert!(s.moves().is_empty());
        s.loading();
        s.listed("Default".into(), list(&["A", "C", "B"]));
        assert_eq!(s.phase, Phase::Applied, "Undo stays offered after a reload");
        assert_eq!(
            s.flagged(),
            1,
            "the messages of the sort stay visible after Apply and a reload"
        );
        s.undoing();
        s.undone();
        assert_eq!((s.phase, s.journal.clone()), (Phase::Loading, None));
        s.listed("Default".into(), list(&["A", "B", "C"]));
        assert_eq!(s.phase, Phase::Listed);
    }

    #[test]
    fn a_failure_goes_back_to_where_the_step_started() {
        let mut s = State::default();
        s.loading();
        s.failed(problem());
        assert_eq!(s.phase, Phase::Listed);
        s.sorting();
        s.previewed(list(&["A"]), Vec::new());
        s.applying();
        s.failed(problem());
        assert_eq!(s.phase, Phase::Preview, "the preview stays for another try");
        s.cancel_preview();
        assert_eq!(s.phase, Phase::Listed);
        s.journal = Some("j".into());
        s.undoing();
        s.failed(problem());
        assert_eq!(s.phase, Phase::Applied, "Undo stays offered");
        s.sorting();
        s.failed(problem());
        assert_eq!(s.phase, Phase::Applied);
        assert!(s.problem.is_some());
        s.loading();
        assert!(s.problem.is_none());
    }

    #[test]
    fn every_command_names_the_instance_and_only_apply_and_undo_write() {
        let l = list_command("/i");
        assert_eq!(l.shown(), "raul plugins list --instance /i");
        assert_eq!(l.rest(), ["--instance", "/i"]);
        assert!(!l.writes && !preview_command("/i").writes);
        assert!(apply_command("/i").writes && undo_command("/i").writes);
        assert_eq!(
            apply_command("/a b").shown(),
            "raul sort --instance '/a b' --fetch-masterlist --apply"
        );
    }

    #[test]
    fn applying_with_nothing_to_change_offers_no_undo() {
        let mut s = State::default();
        s.listed("Default".into(), list(&["A"]));
        s.phase = Phase::Preview;
        s.applying();
        s.applied(None);
        assert_eq!(s.phase, Phase::Listed);
        assert_eq!(MessageKind::parse("say"), MessageKind::Note);
        assert_eq!(MessageKind::parse("ERROR"), MessageKind::Error);
    }
}
