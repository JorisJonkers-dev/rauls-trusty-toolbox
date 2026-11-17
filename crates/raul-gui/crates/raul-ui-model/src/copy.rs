// SPDX-License-Identifier: GPL-3.0-only
//! What an error's "Copy details" button puts on the clipboard: plain, complete and self-contained, in one fixed
//! shape, so a report can be pasted into a message without the window.

use std::path::Path;

/// The shape of every copied error: what was being done, what failed, its detail lines, and the build it happened
/// in. A test checks that shape, so a change to it is a change to what people paste.
pub const COPY_SHAPE: &str = "What: <what was being done>\n\
Error: <title>\n\
Detail:\n\
<detail lines>\n\
raul: <version>\n\
System: <os> <arch>";

/// What the user's home folder is written as, so a copy carries no private path the user did not choose to keep.
pub const HOME: &str = "~";

/// The text "Copy details" puts on the clipboard: `doing` what was being done, `title` what failed, `detail` its
/// lines, and the build it happened in. Every occurrence of `home` in every line is written as [`HOME`].
pub fn copy_details(
    doing: &str,
    title: &str,
    detail: &str,
    version: &str,
    system: &str,
    home: Option<&Path>,
) -> String {
    let line = |label: &str, value: &str| format!("{label}: {}\n", without_home(value, home));
    let mut text = line("What", doing);
    text.push_str(&line("Error", title));
    text.push_str("Detail:\n");
    text.push_str(&without_home(detail, home));
    text.push('\n');
    text.push_str(&line("raul", version));
    text.push_str(&line("System", system));
    text
}

/// `text` with every occurrence of `home` written as [`HOME`].
fn without_home(text: &str, home: Option<&Path>) -> String {
    let Some(home) = home.map(Path::to_string_lossy).filter(|h| !h.is_empty()) else {
        return text.to_owned();
    };
    if text.contains(home.as_ref()) {
        text.replace(home.as_ref(), HOME)
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn home() -> PathBuf {
        PathBuf::from("/users/player")
    }

    #[test]
    fn the_copy_has_the_fixed_shape() {
        let text = copy_details(
            "Generating distant land",
            "The generator stopped",
            "The output folder is not writable.",
            "1.2.3",
            "linux x86_64",
            Some(&home()),
        );
        assert_eq!(
            text,
            "What: Generating distant land\n\
             Error: The generator stopped\n\
             Detail:\n\
             The output folder is not writable.\n\
             raul: 1.2.3\n\
             System: linux x86_64\n"
        );
        let headings: Vec<&str> = text.lines().map(|l| l.split(':').next().unwrap()).collect();
        assert_eq!(
            headings,
            [
                "What",
                "Error",
                "Detail",
                "The output folder is not writable.",
                "raul",
                "System"
            ]
        );
        assert!(COPY_SHAPE.starts_with("What: "));
        assert!(COPY_SHAPE.ends_with("System: <os> <arch>"));
    }

    #[test]
    fn the_home_folder_is_written_as_a_tilde_in_every_line() {
        let text = copy_details(
            "Reading /users/player/raul-storage",
            "Could not read /users/player/.config/raul/config.toml",
            "Nothing changed. /users/player/Downloads is not a game folder.",
            "1.2.3",
            "linux x86_64",
            Some(&home()),
        );
        assert!(!text.contains("/users/player"), "{text}");
        assert_eq!(text.matches(HOME).count(), 3, "{text}");
        assert!(text.contains("What: Reading ~/raul-storage\n"));
        assert!(text.contains("Error: Could not read ~/.config/raul/config.toml\n"));
        assert!(text.contains("Nothing changed. ~/Downloads is not a game folder.\n"));
    }

    #[test]
    fn a_home_that_is_not_in_the_text_changes_nothing() {
        let text = copy_details("Play", "No game", "Set it.", "1.2.3", "linux x86_64", None);
        assert_eq!(
            text,
            "What: Play\nError: No game\nDetail:\nSet it.\nraul: 1.2.3\nSystem: linux x86_64\n"
        );
        let same = copy_details(
            "Play",
            "No game",
            "Set it.",
            "1.2.3",
            "linux x86_64",
            Some(&home()),
        );
        assert_eq!(same, text);
    }

    #[test]
    fn a_multi_line_detail_keeps_its_lines() {
        let text = copy_details(
            "Sorting the load order",
            "The sort was refused",
            "Nothing changed.\nThe masterlist is out of date.\nRun the Doctor.",
            "1.2.3",
            "windows x86_64",
            None,
        );
        let detail: Vec<&str> = text
            .lines()
            .skip_while(|l| *l != "Detail:")
            .skip(1)
            .take_while(|l| !l.starts_with("raul: "))
            .collect();
        assert_eq!(
            detail,
            [
                "Nothing changed.",
                "The masterlist is out of date.",
                "Run the Doctor."
            ]
        );
        assert!(text.ends_with("raul: 1.2.3\nSystem: windows x86_64\n"));
    }

    #[test]
    fn an_empty_detail_line_still_has_its_heading() {
        let text = copy_details("Play", "No game", "", "1.2.3", "linux x86_64", None);
        assert_eq!(
            text,
            "What: Play\nError: No game\nDetail:\n\nraul: 1.2.3\nSystem: linux x86_64\n"
        );
        assert!(text.lines().any(|l| l == "Detail:"));
    }
}
