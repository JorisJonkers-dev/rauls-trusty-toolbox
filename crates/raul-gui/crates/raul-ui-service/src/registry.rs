// SPDX-License-Identifier: GPL-3.0-only
//! The registry that ties every GUI action to the `raul` command that does the same.
//!
//! The rule of the design (ADR-0003) is that the GUI cannot do anything the command line cannot.
//! [`Action`] lists the commands the GUI drives; [`coverage`] answers for every screen either with the action
//! behind it or with an explicit [`Coverage::Planned`] gap. Both matches are exhaustive, so a new screen or a new
//! action does not compile until it is placed, and the parity test in `tests/parity.rs` checks the table against
//! the real command line (`raul nvmod-help`) and against the palette.

use raul_ui_model::Screen;

/// A command of the real command line that the GUI drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    /// `raul setup`: the first run, base copy to a modded Instance. The GUI runs the setup job of `raul-core` as a
    /// streaming job in process ([`crate::direct::setup`]); the command line is its equivalent.
    Setup,
    /// `raul import-mo2`: a new Instance from an MO2 profile.
    ImportMo2,
    /// `raul install`: one Mod from an archive, with FOMOD answers.
    InstallMod,
    /// `raul rollback`: undo the last install, sort or mods change. The GUI runs it as a typed job in process
    /// ([`crate::direct`]); the command line stays as its equivalent.
    Rollback,
    /// `raul sort`: the load order.
    Sort,
    /// `raul ttw`: Tale of Two Wastelands.
    Ttw,
    /// `raul doctor`: the checks.
    Doctor,
    /// `raul verify`: confirm an install.
    Verify,
    /// `raul tools`: the pinned tools.
    Tools,
    /// `raul target`: what the Instance runs on.
    Target,
    /// `raul mods`: list, enable, disable and reorder the Mods of a profile.
    Mods,
    /// `raul conflicts`: the file-level conflicts of a profile (read only).
    Conflicts,
    /// `raul tree`: the virtual Data folder (read only).
    Tree,
    /// `raul plugins`: list, enable, disable, move and lock the plugins of a profile.
    Plugins,
    /// `raul config`: the game folders, the TTW package and the storage folder.
    Config,
    /// `raul install-nv-rs`: put the engine that ships with raul in an Instance (nothing is downloaded).
    InstallNvRs,
    /// `raul init`: an Instance skeleton for a base copy of the game.
    Init,
    /// `raul lod`: plan and generate distant land with lodwright.
    Lod,
    /// `raul run`: start the game on nv-rs, only through the memory-capped run guard.
    Run,
    /// `raul version`: the Toolbox version.
    Version,
    /// `raul update check`: look for a newer Toolbox release; downloads nothing.
    Update,
    /// `raul detect`: find New Vegas and Fallout 3 copies, or check one folder.
    Detect,
    /// `raul profile create`: the default profile of an Instance that has none. Setup makes it with its own
    /// steps; this action is the one place a screen asks for it when an Instance made another way has none.
    Profile,
}

impl Action {
    /// Every action.
    pub const ALL: [Action; 23] = [
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
        Action::Plugins,
        Action::Config,
        Action::InstallNvRs,
        Action::Init,
        Action::Lod,
        Action::Run,
        Action::Version,
        Action::Update,
        Action::Detect,
        Action::Profile,
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
            Self::Plugins => "plugins",
            Self::Config => "config",
            Self::InstallNvRs => "install-nv-rs",
            Self::Init => "init",
            Self::Lod => "lod",
            Self::Run => "run",
            Self::Version => "version",
            Self::Update => "update",
            Self::Detect => "detect",
            Self::Profile => "profile",
        }
    }

    /// Whether a run of it can be stopped from the GUI: the long ones that write.
    pub fn cancellable(self) -> bool {
        matches!(
            self,
            Self::Setup | Self::Ttw | Self::InstallMod | Self::InstallNvRs | Self::Lod
        )
    }

    /// Whether the command takes `--json`. `init` and `install-nv-rs` print text only.
    pub fn takes_json(self) -> bool {
        !matches!(self, Self::Init | Self::InstallNvRs | Self::Profile)
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
            Self::Plugins => "Manage the plugins",
            Self::Config => "Game folders",
            Self::InstallNvRs => "Install the engine",
            Self::Init => "Create the Instance",
            Self::Lod => "Distant land",
            Self::Run => "Play",
            Self::Version => "Versions",
            Self::Update => "Check for updates",
            Self::Detect => "Find the games",
            Self::Profile => "Make the default profile",
        }
    }

    /// The command line start, `raul <command>`.
    pub fn invocation(self) -> String {
        format!("raul {}", self.command())
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
        Screen::Plugins => Coverage::Cli(Action::Plugins),
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

/// Whether `help` (the output of `raul nvmod-help`, whose lines name the program `nvmod`) lists the action's
/// command. `raul` runs every nvmod command in process, so either program name counts.
pub fn help_lists(help: &str, action: Action) -> bool {
    help_block(help, &action.invocation()).is_some()
}

/// The `raul` command a command string such as `raul ttw --instance <dir>` starts, if it is one of the
/// registered actions.
pub fn action_of(command_line: &str) -> Option<Action> {
    let mut words = command_line.split_whitespace();
    if words.next()? != "raul" {
        return None;
    }
    let sub = words.next()?;
    Action::ALL.into_iter().find(|a| a.command() == sub)
}

/// One command line a page starts: the action, the words after it, and the flags the page passes. The parity
/// tests check each against the help of the real command line and run it against fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command {
    /// A stable name, unique in [`COMMANDS`], such as `sort-apply`; pages look a command up by it.
    pub id: &'static str,
    /// The command.
    pub action: Action,
    /// The words after the command word, such as `["get"]` for `raul config get`.
    pub words: &'static [&'static str],
    /// The flags a page may pass; each is in the command's help.
    pub flags: &'static [&'static str],
    /// Whether it writes: only one writing job runs at a time.
    pub writes: bool,
    /// The pull request that adds the command to `raul`, while it is not merged; the parity tests skip it and
    /// fail once the help lists it, so the mark is removed then.
    pub awaits: Option<&'static str>,
}

impl Command {
    /// The command line start, such as `raul config get`.
    pub fn line(&self) -> String {
        let mut line = self.action.invocation();
        for w in self.words {
            line.push(' ');
            line.push_str(w);
        }
        line
    }

    /// The arguments of a job of this command: its words, then `rest` (values and flags).
    pub fn args(&self, rest: impl IntoIterator<Item = String>) -> Vec<String> {
        self.words
            .iter()
            .map(|w| (*w).to_owned())
            .chain(rest)
            .collect()
    }
}

/// Every command the pages of the minimum viable Toolbox start (#117).
pub const COMMANDS: &[Command] = &[
    Command {
        id: "verify",
        action: Action::Verify,
        words: &[],
        flags: &["--instance", "--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "version",
        action: Action::Version,
        words: &[],
        flags: &["--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "update-check",
        action: Action::Update,
        words: &["check"],
        flags: &["--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "detect",
        action: Action::Detect,
        words: &[],
        flags: &["--folder", "--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "config-list",
        action: Action::Config,
        words: &["list"],
        flags: &["--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "config-get",
        action: Action::Config,
        words: &["get"],
        flags: &["--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "config-set",
        action: Action::Config,
        words: &["set"],
        flags: &["--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "config-unset",
        action: Action::Config,
        words: &["unset"],
        flags: &["--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "install-nv-rs",
        action: Action::InstallNvRs,
        words: &[],
        flags: &["--instance"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "init",
        action: Action::Init,
        words: &[],
        flags: &["--base", "--instance"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "mods-list",
        action: Action::Mods,
        words: &["list"],
        flags: &["--instance", "--profile", "--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "mods-enable",
        action: Action::Mods,
        words: &["enable"],
        flags: &["--instance", "--profile", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "mods-disable",
        action: Action::Mods,
        words: &["disable"],
        flags: &["--instance", "--profile", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "mods-reorder",
        action: Action::Mods,
        words: &["reorder"],
        flags: &["--position", "--instance", "--profile", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "mods-remove",
        action: Action::Mods,
        words: &["remove"],
        flags: &["--instance", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "plugins-list",
        action: Action::Plugins,
        words: &["list"],
        flags: &["--instance", "--profile", "--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "sort-preview",
        action: Action::Sort,
        words: &[],
        flags: &[
            "--instance",
            "--profile",
            "--masterlist",
            "--fetch-masterlist",
            "--json",
        ],
        writes: false,
        awaits: None,
    },
    Command {
        id: "sort-apply",
        action: Action::Sort,
        words: &[],
        flags: &[
            "--instance",
            "--profile",
            "--masterlist",
            "--fetch-masterlist",
            "--apply",
            "--json",
        ],
        writes: true,
        awaits: None,
    },
    Command {
        id: "rollback",
        action: Action::Rollback,
        words: &[],
        flags: &["--instance", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "lod-plan",
        action: Action::Lod,
        words: &["plan"],
        flags: &[
            "--instance",
            "--profile",
            "--world",
            "--allow-in-repo",
            "--json",
        ],
        writes: false,
        awaits: None,
    },
    Command {
        id: "lod-generate",
        action: Action::Lod,
        words: &["generate"],
        flags: &[
            "--instance",
            "--profile",
            "--world",
            "--allow-in-repo",
            "--dry-run",
            "--json",
        ],
        writes: true,
        awaits: None,
    },
    Command {
        // The LOD settings view: the options of a Profile's raul-lod.toml (planned ticket C2).
        id: "lod-settings-show",
        action: Action::Lod,
        words: &["settings", "show"],
        flags: &["--instance", "--profile", "--json"],
        writes: false,
        awaits: None,
    },
    Command {
        // The LOD settings view: the options of a Profile's raul-lod.toml (planned ticket C2).
        id: "lod-settings-set",
        action: Action::Lod,
        words: &["settings", "set"],
        flags: &["--instance", "--profile", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        // The LOD settings view: the options of a Profile's raul-lod.toml (planned ticket C2).
        id: "lod-settings-unset",
        action: Action::Lod,
        words: &["settings", "unset"],
        flags: &["--instance", "--profile", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        // The LOD settings view: the options of a Profile's raul-lod.toml (planned ticket C2).
        id: "lod-settings-reset",
        action: Action::Lod,
        words: &["settings", "reset"],
        flags: &["--instance", "--profile", "--dry-run", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        // The LOD settings view: the options of a Profile's raul-lod.toml (planned ticket C2).
        id: "lod-settings-undo",
        action: Action::Lod,
        words: &["settings", "undo"],
        flags: &["--instance", "--profile", "--json"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "lod-mods",
        action: Action::Lod,
        words: &["mods"],
        flags: &["--instance", "--profile", "--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "run",
        action: Action::Run,
        // nvmod's `run <title|path|game|nv-rs>`: the target is the first value a page passes; Play passes `nv-rs`.
        words: &[],
        flags: &["--instance", "--dry-run", "--json"],
        // It stages the Profile before the launch and holds the Instance while it does; without --wait it returns
        // once the game has started.
        writes: true,
        awaits: None,
    },
    Command {
        id: "ttw",
        action: Action::Ttw,
        words: &[],
        flags: &[
            "--instance",
            "--profile",
            "--fallout3",
            "--newvegas",
            "--package",
            "--dry-run",
            "--resume",
            "--json",
        ],
        writes: true,
        awaits: None,
    },
    Command {
        // The pinned tools and the masterlist they name.
        id: "tools-list",
        action: Action::Tools,
        words: &["list"],
        flags: &["--json"],
        writes: false,
        awaits: None,
    },
    Command {
        id: "profile-create",
        action: Action::Profile,
        words: &["create"],
        flags: &["--name", "--instance"],
        writes: true,
        awaits: None,
    },
    Command {
        id: "doctor",
        action: Action::Doctor,
        words: &[],
        flags: &["--instance", "--profile", "--json"],
        writes: false,
        awaits: None,
    },
];

/// The help block of `line` (such as `raul config get`) in `help`: each usage line of it and the lines that continue
/// it, up to the next usage line. A line naming the program `nvmod` counts for `raul`, which runs nvmod's commands.
/// `None` when the help does not list it.
pub fn help_block(help: &str, line: &str) -> Option<String> {
    let rest = line.strip_prefix("raul ")?;
    // A usage line may name two forms (`raul --version | raul version [--json]`); either counts.
    let starts = |l: &str| {
        l.trim_start().split(" | ").any(|form| {
            ["raul ", "nvmod "].iter().any(|p| {
                form.strip_prefix(p).is_some_and(|r| {
                    r == rest
                        || r.starts_with(&format!("{rest} "))
                        || r.starts_with(&format!("{rest}|"))
                        || alternatives_match(r, rest)
                })
            })
        })
    };
    let lines: Vec<&str> = help.lines().collect();
    let mut blocks = Vec::new();
    for (at, first) in lines.iter().enumerate().filter(|(_, l)| starts(l)) {
        let mut block = (*first).to_owned();
        for l in &lines[at + 1..] {
            let t = l.trim_start();
            if t.starts_with("raul ") || t.starts_with("nvmod ") || t.is_empty() {
                break;
            }
            block.push('\n');
            block.push_str(l);
        }
        blocks.push(block);
    }
    (!blocks.is_empty()).then(|| blocks.join("\n"))
}

/// Whether a usage line such as `config list|get <key>|set <key> <path>` names the command `config get`.
fn alternatives_match(usage: &str, command: &str) -> bool {
    let mut words = command.split(' ');
    let (Some(first), Some(sub), None) = (words.next(), words.next(), words.next()) else {
        return false;
    };
    let Some(alts) = usage.strip_prefix(&format!("{first} ")) else {
        return false;
    };
    // A placeholder such as `<title|path|game|nv-rs>` lists the values a positional takes.
    alts.split_whitespace().next().is_some_and(|w| {
        w.trim_matches(['<', '>', '[', ']'])
            .split('|')
            .any(|a| a == sub)
    }) || alts
        .split('|')
        .any(|a| a.split_whitespace().next() == Some(sub))
}

/// The command `id` names in [`COMMANDS`].
pub fn command(id: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.id == id)
}
