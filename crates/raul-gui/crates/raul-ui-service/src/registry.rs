// SPDX-License-Identifier: GPL-3.0-only
//! The registry that ties every GUI action to the `nvmod` command that does the same.
//!
//! The rule of the design (document 04, section 8) is that the GUI cannot do anything the command line cannot.
//! [`Action`] lists the commands the GUI drives; [`coverage`] answers for every screen either with the action
//! behind it or with an explicit [`Coverage::Planned`] gap. Both matches are exhaustive, so a new screen or a new
//! action does not compile until it is placed, and the parity test in `tests/parity.rs` checks the table against
//! the real command line (`nvmod --help`) and against the palette.

use nvmod_ui_model::Screen;

/// A command of the real command line that the GUI drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    /// `nvmod setup`: the first run, base copy to a modded Instance.
    Setup,
    /// `nvmod import-mo2`: a new Instance from an MO2 profile.
    ImportMo2,
    /// `nvmod install`: one Mod from an archive, with FOMOD answers.
    InstallMod,
    /// `nvmod rollback`: undo the last install, sort or mods change.
    Rollback,
    /// `nvmod sort`: the load order.
    Sort,
    /// `nvmod ttw`: Tale of Two Wastelands.
    Ttw,
    /// `nvmod doctor`: the checks.
    Doctor,
    /// `nvmod verify`: confirm an install.
    Verify,
    /// `nvmod tools`: the pinned tools.
    Tools,
    /// `nvmod target`: what the Instance runs on.
    Target,
    /// `nvmod mods`: list, enable, disable and reorder the Mods of a profile.
    Mods,
    /// `nvmod conflicts`: the file-level conflicts of a profile (read only).
    Conflicts,
    /// `nvmod tree`: the virtual Data folder (read only).
    Tree,
}

impl Action {
    /// Every action.
    pub const ALL: [Action; 13] = [
        Action::Setup,
        Action::ImportMo2,
        Action::InstallMod,
        Action::Rollback,
        Action::Sort,
        Action::Ttw,
        Action::Doctor,
        Action::Verify,
        Action::Tools,
        Action::Target,
        Action::Mods,
        Action::Conflicts,
        Action::Tree,
    ];

    /// The subcommand word of the command line.
    pub fn command(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::ImportMo2 => "import-mo2",
            Self::InstallMod => "install",
            Self::Rollback => "rollback",
            Self::Sort => "sort",
            Self::Ttw => "ttw",
            Self::Doctor => "doctor",
            Self::Verify => "verify",
            Self::Tools => "tools",
            Self::Target => "target",
            Self::Mods => "mods",
            Self::Conflicts => "conflicts",
            Self::Tree => "tree",
        }
    }

    /// Whether a run of it can be stopped from the GUI: the long ones that write, `setup`, `ttw` and `install`.
    pub fn cancellable(self) -> bool {
        matches!(self, Self::Setup | Self::Ttw | Self::InstallMod)
    }

    /// What the GUI calls it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Setup => "First run",
            Self::ImportMo2 => "Import from MO2",
            Self::InstallMod => "Install a Mod",
            Self::Rollback => "Undo the last change",
            Self::Sort => "Sort the load order",
            Self::Ttw => "Install Tale of Two Wastelands",
            Self::Doctor => "Run the Doctor",
            Self::Verify => "Verify the install",
            Self::Tools => "Pinned tools",
            Self::Target => "Choose the runtime",
            Self::Mods => "Manage the Mods",
            Self::Conflicts => "Show the file conflicts",
            Self::Tree => "Explore the virtual Data folder",
        }
    }

    /// The command line start, `nvmod <command>`.
    pub fn invocation(self) -> String {
        format!("nvmod {}", self.command())
    }
}

/// How a screen is covered by the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// The command line does what the screen does.
    Cli(Action),
    /// The command line has no equivalent yet; the note says what is missing. A screen listed here must not offer
    /// an action that changes anything.
    Planned(&'static str),
}

/// The command line behind a screen.
pub fn coverage(screen: Screen) -> Coverage {
    match screen {
        Screen::Home => Coverage::Planned("a status command that summarises the Instance"),
        Screen::FirstRun => Coverage::Cli(Action::Setup),
        Screen::ImportMo2 => Coverage::Cli(Action::ImportMo2),
        Screen::Mods => Coverage::Cli(Action::Mods),
        Screen::Install | Screen::Fomod => Coverage::Cli(Action::InstallMod),
        Screen::Plugins => Coverage::Planned("a command that lists the plugins and their masters"),
        Screen::SortDiff => Coverage::Cli(Action::Sort),
        Screen::Graph => {
            Coverage::Planned("a command that prints the dependency and conflict graph")
        }
        Screen::Ttw => Coverage::Cli(Action::Ttw),
        Screen::Doctor => Coverage::Cli(Action::Doctor),
        Screen::Verify => Coverage::Cli(Action::Verify),
        Screen::Runtimes => Coverage::Cli(Action::Target),
        Screen::History => Coverage::Cli(Action::Rollback),
        Screen::Jobs => Coverage::Planned("a command that follows the jobs of other processes"),
        Screen::Oracle => Coverage::Planned("the private real-data status, owner builds only"),
    }
}

/// The screens that have no command line equivalent yet, in the order of the inventory.
pub fn planned_screens() -> Vec<Screen> {
    Screen::ALL
        .into_iter()
        .filter(|s| matches!(coverage(*s), Coverage::Planned(_)))
        .collect()
}

/// Whether `help` (the output of `nvmod --help`) lists the action's command.
pub fn help_lists(help: &str, action: Action) -> bool {
    let start = action.invocation();
    help.lines().any(|l| {
        let l = l.trim_start();
        l == start || l.starts_with(&format!("{start} "))
    })
}

/// The `nvmod` command a command string such as `nvmod ttw --instance <dir>` starts, if it is one of the
/// registered actions.
pub fn action_of(command_line: &str) -> Option<Action> {
    let mut words = command_line.split_whitespace();
    if words.next()? != "nvmod" {
        return None;
    }
    let sub = words.next()?;
    Action::ALL.into_iter().find(|a| a.command() == sub)
}
