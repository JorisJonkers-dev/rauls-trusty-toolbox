// SPDX-License-Identifier: GPL-3.0-only
//! The shortcuts and the rebinding of them.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A key without its modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// A letter, digit or symbol; letters are stored in upper case.
    Char(char),
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Enter.
    Enter,
    /// Escape.
    Esc,
    /// Space bar.
    Space,
    /// A function key, 1 to 12.
    F(u8),
}

/// A key with modifiers: `Ctrl+K`, `Alt+Up`, `F1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Chord {
    /// Control (Command is out of scope).
    pub ctrl: bool,
    /// Alt.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// The key.
    pub key: Key,
}

impl Chord {
    /// A key with no modifier.
    pub const fn plain(key: Key) -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            key,
        }
    }

    /// A key with Ctrl.
    pub const fn ctrl(key: Key) -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: false,
            key,
        }
    }

    /// A key with Alt.
    pub const fn alt(key: Key) -> Self {
        Self {
            ctrl: false,
            alt: true,
            shift: false,
            key,
        }
    }

    /// Parses `Ctrl+Shift+K`, `Alt+Up`, `F1`, `/`, `Esc` (case of modifiers does not matter).
    ///
    /// # Errors
    /// The text that is not a key or a modifier.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut chord = Self::plain(Key::Space);
        let mut key = None;
        for part in text.split('+') {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => chord.ctrl = true,
                "alt" => chord.alt = true,
                "shift" => chord.shift = true,
                "up" => key = Some(Key::Up),
                "down" => key = Some(Key::Down),
                "enter" | "return" => key = Some(Key::Enter),
                "esc" | "escape" => key = Some(Key::Esc),
                "space" => key = Some(Key::Space),
                p if p.len() > 1
                    && p.starts_with('f')
                    && p[1..].chars().all(|c| c.is_ascii_digit()) =>
                {
                    let n: u8 = p[1..].parse().map_err(|_| part.to_owned())?;
                    if !(1..=12).contains(&n) {
                        return Err(part.to_owned());
                    }
                    key = Some(Key::F(n));
                }
                p if p.chars().count() == 1 => {
                    key = Some(Key::Char(
                        p.chars().next().unwrap_or(' ').to_ascii_uppercase(),
                    ));
                }
                _ => return Err(part.to_owned()),
            }
        }
        chord.key = key.ok_or_else(|| text.to_owned())?;
        Ok(chord)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        if self.shift {
            f.write_str("Shift+")?;
        }
        match self.key {
            Key::Char(c) => write!(f, "{c}"),
            Key::Up => f.write_str("Up"),
            Key::Down => f.write_str("Down"),
            Key::Enter => f.write_str("Enter"),
            Key::Esc => f.write_str("Esc"),
            Key::Space => f.write_str("Space"),
            Key::F(n) => write!(f, "F{n}"),
        }
    }
}

/// What a shortcut does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    /// Open the command palette.
    Palette,
    /// Go to the rail screen at this position (1 to 8).
    Rail(u8),
    /// Focus the filter box.
    Filter,
    /// Toggle the selected mod or plugin.
    Toggle,
    /// Move the selected mod or plugin up.
    MoveUp,
    /// Move the selected mod or plugin down.
    MoveDown,
    /// Open the detail of the selected row.
    Open,
    /// Close the topmost panel.
    Close,
    /// "Why is this here".
    Why,
    /// Undo, which asks first.
    Undo,
    /// Apply the reviewed plan.
    ApplyReviewed,
    /// The help overlay.
    Help,
}

impl Command {
    /// The name shown next to the shortcut.
    pub fn title(self) -> String {
        match self {
            Self::Palette => "Command palette".into(),
            Self::Rail(n) => format!("Go to rail screen {n}"),
            Self::Filter => "Filter".into(),
            Self::Toggle => "Enable or disable".into(),
            Self::MoveUp => "Move up".into(),
            Self::MoveDown => "Move down".into(),
            Self::Open => "Open detail".into(),
            Self::Close => "Close".into(),
            Self::Why => "Why is this here".into(),
            Self::Undo => "Undo".into(),
            Self::ApplyReviewed => "Apply the reviewed plan".into(),
            Self::Help => "Help".into(),
        }
    }
}

/// A shortcut table. Bindings are rebindable and no chord can mean two commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keymap {
    bindings: Vec<(Chord, Command)>,
}

impl Default for Keymap {
    /// The default bindings.
    fn default() -> Self {
        let mut b = vec![
            (Chord::ctrl(Key::Char('K')), Command::Palette),
            (Chord::plain(Key::Char('/')), Command::Filter),
            (Chord::ctrl(Key::Char('F')), Command::Filter),
            (Chord::plain(Key::Space), Command::Toggle),
            (Chord::alt(Key::Up), Command::MoveUp),
            (Chord::alt(Key::Down), Command::MoveDown),
            (Chord::plain(Key::Enter), Command::Open),
            (Chord::plain(Key::Esc), Command::Close),
            (Chord::plain(Key::Char('W')), Command::Why),
            (Chord::ctrl(Key::Char('Z')), Command::Undo),
            (Chord::ctrl(Key::Enter), Command::ApplyReviewed),
            (Chord::plain(Key::F(1)), Command::Help),
        ];
        for n in 1..=8u8 {
            b.push((
                Chord::ctrl(Key::Char(char::from(b'0' + n))),
                Command::Rail(n),
            ));
        }
        Self { bindings: b }
    }
}

/// Why a rebind was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindError {
    /// The chord already means another command.
    Taken(Command),
    /// `Esc` always closes, so it cannot be moved to another command.
    EscIsReserved,
}

impl Keymap {
    /// All bindings.
    pub fn bindings(&self) -> &[(Chord, Command)] {
        &self.bindings
    }

    /// The command a chord triggers.
    pub fn lookup(&self, chord: Chord) -> Option<Command> {
        self.bindings
            .iter()
            .find(|(c, _)| *c == chord)
            .map(|(_, cmd)| *cmd)
    }

    /// The chords of a command, for the help overlay and the palette.
    pub fn chords_for(&self, command: Command) -> Vec<Chord> {
        self.bindings
            .iter()
            .filter(|(_, c)| *c == command)
            .map(|(ch, _)| *ch)
            .collect()
    }

    /// Binds `chord` to `command`, replacing the command's own earlier chord(s) of the same index only when the
    /// chord is free.
    ///
    /// # Errors
    /// [`BindError::Taken`] when another command uses the chord; [`BindError::EscIsReserved`] for `Esc`.
    pub fn bind(&mut self, chord: Chord, command: Command) -> Result<(), BindError> {
        if chord == Chord::plain(Key::Esc) && command != Command::Close {
            return Err(BindError::EscIsReserved);
        }
        match self.lookup(chord) {
            Some(other) if other != command => Err(BindError::Taken(other)),
            Some(_) => Ok(()),
            None => {
                self.bindings.push((chord, command));
                Ok(())
            }
        }
    }

    /// Removes a chord.
    pub fn unbind(&mut self, chord: Chord) -> bool {
        let before = self.bindings.len();
        self.bindings.retain(|(c, _)| *c != chord);
        self.bindings.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_document_05() {
        let k = Keymap::default();
        assert_eq!(
            k.lookup(Chord::ctrl(Key::Char('K'))),
            Some(Command::Palette)
        );
        assert_eq!(
            k.lookup(Chord::plain(Key::Char('/'))),
            Some(Command::Filter)
        );
        assert_eq!(k.lookup(Chord::ctrl(Key::Char('F'))), Some(Command::Filter));
        assert_eq!(k.lookup(Chord::alt(Key::Up)), Some(Command::MoveUp));
        assert_eq!(
            k.lookup(Chord::ctrl(Key::Enter)),
            Some(Command::ApplyReviewed)
        );
        assert_eq!(k.lookup(Chord::plain(Key::F(1))), Some(Command::Help));
        assert_eq!(
            k.lookup(Chord::ctrl(Key::Char('8'))),
            Some(Command::Rail(8))
        );
        assert_eq!(k.lookup(Chord::ctrl(Key::Char('9'))), None);
        assert_eq!(k.chords_for(Command::Filter).len(), 2);
    }

    #[test]
    fn no_chord_means_two_commands() {
        let k = Keymap::default();
        for (i, (a, _)) in k.bindings().iter().enumerate() {
            assert!(k.bindings().iter().skip(i + 1).all(|(b, _)| a != b));
        }
    }

    #[test]
    fn parse_and_display_round_trip() {
        for text in [
            "Ctrl+K",
            "Alt+Up",
            "F1",
            "/",
            "Esc",
            "Ctrl+Shift+Z",
            "Space",
            "Alt+Down",
            "Enter",
            "Ctrl+1",
        ] {
            let c = Chord::parse(text).unwrap();
            assert_eq!(c.to_string(), text);
        }
        assert_eq!(Chord::parse("ctrl+k").unwrap(), Chord::ctrl(Key::Char('K')));
        assert!(Chord::parse("Ctrl+").is_err());
        assert!(Chord::parse("Ctrl+Nope").is_err());
        assert!(Chord::parse("F13").is_err());
        assert!(Chord::parse("Fx").is_err());
        assert!(Chord::parse("Ctrl").is_err());
    }

    #[test]
    fn rebinding_refuses_conflicts() {
        let mut k = Keymap::default();
        let g = Chord::ctrl(Key::Char('G'));
        k.bind(g, Command::Why).unwrap();
        assert_eq!(k.lookup(g), Some(Command::Why));
        assert_eq!(
            k.bind(Chord::ctrl(Key::Char('K')), Command::Help),
            Err(BindError::Taken(Command::Palette))
        );
        assert_eq!(
            k.bind(Chord::plain(Key::Esc), Command::Help),
            Err(BindError::EscIsReserved)
        );
        assert!(k.bind(Chord::plain(Key::Esc), Command::Close).is_ok());
        assert!(k.unbind(g));
        assert!(!k.unbind(g));
    }

    #[test]
    fn every_command_has_a_title() {
        for (_, c) in Keymap::default().bindings() {
            assert!(!c.title().is_empty());
        }
    }
}
