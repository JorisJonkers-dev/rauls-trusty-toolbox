// SPDX-License-Identifier: GPL-3.0-only
//! Pure dialog rules from the redesign.
//!
//! Implements F-OVERLAY-03 (dialog frame), F-OVERLAY-04 (confirm dialogs) and
//! F-OVERLAY-05 (name dialog), plus SPEC section 7 (dialog focus and keyboard)
//! and section 1 rule 10 (Enter submits, Escape cancels).
//!
//! These functions are toolkit-free: they decide where focus goes and what `Enter`
//! and `Escape` do, so a GUI adapter can call them without owning the behaviour.

/// A focusable control inside a dialog.
///
/// The order of the variants matches the focus-start priority from SPEC section 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    /// A text field, such as the name dialog's input.
    Input,
    /// A pre-selected item, such as a highlighted row.
    SelectedItem,
    /// A selectable row in a folder picker.
    PickerItem,
    /// A dialog option, such as a `FOMOD` choice.
    Option,
    /// The `OK` button.
    Ok,
    /// A primary action button.
    Primary,
    /// A destructive action button.
    Danger,
    /// A plain button that keeps `Enter` when it has focus.
    Plain,
}

/// One focusable control with its enabled state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Control {
    /// What kind of control this is.
    pub kind: ControlKind,
    /// Whether the control can be focused and activated.
    pub enabled: bool,
}

/// Returns the index of the control that should receive focus when a dialog opens.
///
/// Focus starts on the first enabled control in this priority order: input,
/// selected item, picker item, option, OK, primary.
pub fn focus_start(controls: &[Control]) -> Option<usize> {
    const ORDER: &[ControlKind] = &[
        ControlKind::Input,
        ControlKind::SelectedItem,
        ControlKind::PickerItem,
        ControlKind::Option,
        ControlKind::Ok,
        ControlKind::Primary,
    ];

    for &kind in ORDER {
        if let Some(index) = controls
            .iter()
            .position(|control| control.enabled && control.kind == kind)
        {
            return Some(index);
        }
    }

    None
}

/// Moves focus to the next or previous enabled control, wrapping around.
///
/// When no control is focused yet, `back` selects the last enabled control and
/// forward selects the first.
pub fn tab(controls: &[Control], focused: Option<usize>, back: bool) -> Option<usize> {
    let enabled: Vec<usize> = controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| control.enabled.then_some(index))
        .collect();

    if enabled.is_empty() {
        return None;
    }

    let position = focused.and_then(|index| enabled.iter().position(|&i| i == index));

    let next = match position {
        Some(pos) if back => (pos + enabled.len() - 1) % enabled.len(),
        Some(pos) => (pos + 1) % enabled.len(),
        None if back => enabled.len() - 1,
        None => 0,
    };

    Some(enabled[next])
}

/// Returns the control that `Enter` should activate in a dialog.
///
/// If a plain button is focused, it receives `Enter`. Otherwise the last enabled
/// primary or danger button is the default. Returns `None` when there is no
/// suitable target.
pub fn enter_target(controls: &[Control], focused: Option<usize>) -> Option<usize> {
    if let Some(index) = focused
        && controls
            .get(index)
            .is_some_and(|control| control.enabled && control.kind == ControlKind::Plain)
    {
        return Some(index);
    }

    controls
        .iter()
        .enumerate()
        .rev()
        .find(|(_, control)| {
            control.enabled
                && (control.kind == ControlKind::Primary || control.kind == ControlKind::Danger)
        })
        .map(|(index, _)| index)
}

/// A modal layer that `Escape` can close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// A dropdown menu.
    Menu,
    /// The command palette.
    Palette,
    /// A dialog. `Escape` is ignored while `busy` is true.
    Dialog {
        /// True while the dialog's job runs (an install), so `Escape` leaves it open.
        busy: bool,
    },
    /// A drag in progress.
    Drag,
}

/// Returns the index in `open` of the layer `Escape` should close.
///
/// `Escape` closes the first open layer in the order menu, palette, dialog (skipped
/// while busy), drag. Returns `None` when nothing should close.
pub fn escape_closes(open: &[Layer]) -> Option<usize> {
    let order = [
        Layer::Menu,
        Layer::Palette,
        Layer::Dialog { busy: false },
        Layer::Drag,
    ];

    for target in order {
        if let Some(index) = open.iter().position(|&layer| {
            matches!(
                (layer, target),
                (Layer::Menu, Layer::Menu)
                    | (Layer::Palette, Layer::Palette)
                    | (Layer::Dialog { .. }, Layer::Dialog { .. })
                    | (Layer::Drag, Layer::Drag)
            )
        }) {
            if matches!(target, Layer::Dialog { .. })
                && matches!(open[index], Layer::Dialog { busy: true })
            {
                continue;
            }
            return Some(index);
        }
    }

    None
}

/// Maximum length of a profile or group name in characters.
pub const NAME_MAX: usize = 40;

/// Why a proposed name is not acceptable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// The trimmed name is empty.
    Empty,
    /// The name is longer than [`NAME_MAX`] characters.
    TooLong,
    /// Another profile or group already uses this name (case-insensitive).
    Duplicate,
}

/// What the name dialog names: the duplicate error says which.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Named {
    /// A profile.
    Profile,
    /// A group of mods.
    Group,
}

impl NameError {
    /// The inline error under the name field, as the demo shows it. Only a duplicate has one: an empty name keeps
    /// OK disabled, and the field takes no more than [`NAME_MAX`] characters.
    #[must_use]
    pub fn text(&self, named: Named) -> Option<&'static str> {
        match (self, named) {
            (Self::Duplicate, Named::Profile) => Some("A profile with that name exists."),
            (Self::Duplicate, Named::Group) => Some("A group with that name exists."),
            (Self::Empty | Self::TooLong, _) => None,
        }
    }
}

/// Validates a profile or group name.
///
/// `name` is trimmed. An empty trimmed name returns [`NameError::Empty`], more
/// than [`NAME_MAX`] characters returns [`NameError::TooLong`], and a name that
/// equals an existing one ignoring case (other than `current`) returns
/// [`NameError::Duplicate`]. On success the trimmed name is returned.
///
/// # Errors
/// The [`NameError`] of the first rule the name breaks.
pub fn validate_name(
    name: &str,
    existing: &[&str],
    current: Option<&str>,
) -> Result<String, NameError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(NameError::Empty);
    }

    if trimmed.chars().count() > NAME_MAX {
        return Err(NameError::TooLong);
    }

    let trimmed_lower = trimmed.to_lowercase();

    for &other in existing {
        if other.to_lowercase() == trimmed_lower && current != Some(other) {
            return Err(NameError::Duplicate);
        }
    }

    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn control(kind: ControlKind, enabled: bool) -> Control {
        Control { kind, enabled }
    }

    #[test]
    fn focus_start_prefers_input_over_primary() {
        let controls = [
            control(ControlKind::Primary, true),
            control(ControlKind::Input, true),
        ];
        assert_eq!(focus_start(&controls), Some(1));
    }

    #[test]
    fn focus_start_skips_disabled_controls() {
        let controls = [
            control(ControlKind::Input, false),
            control(ControlKind::Primary, true),
        ];
        assert_eq!(focus_start(&controls), Some(1));
    }

    #[test]
    fn focus_start_picks_selected_item_when_there_is_no_input() {
        let controls = [
            control(ControlKind::Primary, true),
            control(ControlKind::SelectedItem, true),
        ];
        assert_eq!(focus_start(&controls), Some(1));
    }

    #[test]
    fn tab_wraps_forward_over_enabled_controls() {
        let controls = [
            control(ControlKind::Input, true),
            control(ControlKind::Primary, false),
            control(ControlKind::Plain, true),
        ];
        assert_eq!(tab(&controls, Some(0), false), Some(2));
        assert_eq!(tab(&controls, Some(2), false), Some(0));
    }

    #[test]
    fn tab_wraps_backward_over_enabled_controls() {
        let controls = [
            control(ControlKind::Input, true),
            control(ControlKind::Primary, false),
            control(ControlKind::Plain, true),
        ];
        assert_eq!(tab(&controls, Some(2), true), Some(0));
        assert_eq!(tab(&controls, Some(0), true), Some(2));
    }

    #[test]
    fn enter_goes_to_last_primary_or_danger() {
        let controls = [
            control(ControlKind::Primary, true),
            control(ControlKind::Danger, true),
            control(ControlKind::Primary, true),
        ];
        assert_eq!(enter_target(&controls, None), Some(2));

        let with_disabled_danger = [
            control(ControlKind::Primary, true),
            control(ControlKind::Danger, false),
        ];
        assert_eq!(enter_target(&with_disabled_danger, None), Some(0));
    }

    #[test]
    fn focused_plain_button_takes_enter() {
        let controls = [
            control(ControlKind::Plain, true),
            control(ControlKind::Primary, true),
        ];
        assert_eq!(enter_target(&controls, Some(0)), Some(0));
        assert_eq!(enter_target(&controls, Some(1)), Some(1));
    }

    #[test]
    fn escape_closes_menu_before_dialog() {
        let open = [Layer::Dialog { busy: false }, Layer::Menu];
        assert_eq!(escape_closes(&open), Some(1));
    }

    #[test]
    fn escape_skips_busy_dialog() {
        let open = [Layer::Palette, Layer::Dialog { busy: true }, Layer::Drag];
        assert_eq!(escape_closes(&open), Some(0));
    }

    #[test]
    fn only_a_duplicate_shows_an_error_text() {
        assert_eq!(NameError::Empty.text(Named::Profile), None);
        assert_eq!(NameError::TooLong.text(Named::Group), None);
        assert_eq!(
            NameError::Duplicate.text(Named::Profile),
            Some("A profile with that name exists.")
        );
        assert_eq!(
            NameError::Duplicate.text(Named::Group),
            Some("A group with that name exists.")
        );
    }

    #[test]
    fn validate_name_trims() {
        assert_eq!(validate_name("  Foo  ", &[], None), Ok("Foo".to_owned()));
    }

    #[test]
    fn validate_name_rejects_empty() {
        assert_eq!(validate_name("   ", &[], None), Err(NameError::Empty));
    }

    #[test]
    fn validate_name_accepts_exactly_40_chars() {
        let name = "a".repeat(40);
        assert_eq!(validate_name(&name, &[], None), Ok(name));
    }

    #[test]
    fn validate_name_rejects_41_chars() {
        let name = "a".repeat(41);
        assert_eq!(validate_name(&name, &[], None), Err(NameError::TooLong));
    }

    #[test]
    #[allow(clippy::non_ascii_literal)]
    fn validate_name_counts_multibyte_name_by_chars() {
        let ok = "é".repeat(40);
        assert_eq!(ok.chars().count(), 40);
        assert_eq!(validate_name(&ok, &[], None), Ok(ok));

        let too_long = "é".repeat(41);
        assert_eq!(too_long.chars().count(), 41);
        assert_eq!(validate_name(&too_long, &[], None), Err(NameError::TooLong));
    }

    #[test]
    fn validate_name_rejects_duplicate_in_another_case() {
        assert_eq!(
            validate_name("Foo", &["foo"], None),
            Err(NameError::Duplicate)
        );
    }

    #[test]
    fn validate_name_accepts_renaming_to_its_own_name() {
        assert_eq!(
            validate_name("Foo", &["Foo"], Some("Foo")),
            Ok("Foo".to_owned())
        );
    }
}
