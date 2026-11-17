// SPDX-License-Identifier: GPL-3.0-only
//! Screens and the left rail (document 02).

use serde::{Deserialize, Serialize};

/// Every screen of the inventory in document 02.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Screen {
    /// Home: the install spine and health.
    Home,
    /// First run wizard.
    FirstRun,
    /// Import MO2 and its report.
    ImportMo2,
    /// The mod list.
    Mods,
    /// Install plan.
    Install,
    /// FOMOD questions.
    Fomod,
    /// Plugins and load order.
    Plugins,
    /// Sort preview and diff.
    SortDiff,
    /// Dependency and conflict graph.
    Graph,
    /// TTW install.
    Ttw,
    /// Doctor.
    Doctor,
    /// Verify.
    Verify,
    /// Runtimes: nv-rs and vanilla.
    Runtimes,
    /// History and rollback.
    History,
    /// Jobs and progress.
    Jobs,
    /// Real data status (owner builds only).
    Oracle,
}

/// The rail, in order: `Ctrl+1` to `Ctrl+8` select by position.
pub const RAIL: [Screen; 8] = [
    Screen::Home,
    Screen::Mods,
    Screen::Plugins,
    Screen::Graph,
    Screen::Doctor,
    Screen::Runtimes,
    Screen::History,
    Screen::Oracle,
];

impl Screen {
    /// Every screen.
    pub const ALL: [Screen; 16] = [
        Screen::Home,
        Screen::FirstRun,
        Screen::ImportMo2,
        Screen::Mods,
        Screen::Install,
        Screen::Fomod,
        Screen::Plugins,
        Screen::SortDiff,
        Screen::Graph,
        Screen::Ttw,
        Screen::Doctor,
        Screen::Verify,
        Screen::Runtimes,
        Screen::History,
        Screen::Jobs,
        Screen::Oracle,
    ];

    /// The title shown in the rail and the palette.
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::FirstRun => "First run",
            Self::ImportMo2 => "Import MO2",
            Self::Mods => "Mods",
            Self::Install => "Install",
            Self::Fomod => "FOMOD",
            Self::Plugins => "Plugins",
            Self::SortDiff => "Sort preview",
            Self::Graph => "Graph",
            Self::Ttw => "TTW",
            Self::Doctor => "Doctor",
            Self::Verify => "Verify",
            Self::Runtimes => "Runtimes",
            Self::History => "History",
            Self::Jobs => "Jobs",
            Self::Oracle => "Real data",
        }
    }

    /// The `raul` command behind the screen.
    pub fn cli(self) -> &'static str {
        match self {
            Self::Home => "raul status --json",
            Self::FirstRun => "raul init --plan",
            Self::ImportMo2 => "raul import mo2 --plan",
            Self::Mods => "raul mods list --json",
            Self::Install => "raul install --plan",
            Self::Fomod => "raul fomod",
            Self::Plugins => "raul plugins list --json",
            Self::SortDiff => "raul sort --plan --explain",
            Self::Graph => "raul graph --json",
            Self::Ttw => "raul ttw install --plan",
            Self::Doctor => "raul doctor --json",
            Self::Verify => "raul verify --json",
            Self::Runtimes => "raul runtime list",
            Self::History => "raul history",
            Self::Jobs => "raul jobs --follow --json",
            Self::Oracle => "raul oracle status --json",
        }
    }

    /// Whether the screen can change anything (after a plan and an Apply).
    pub fn mutates(self) -> bool {
        matches!(
            self,
            Self::FirstRun
                | Self::ImportMo2
                | Self::Mods
                | Self::Install
                | Self::SortDiff
                | Self::Ttw
                | Self::Runtimes
                | Self::History
        )
    }

    /// The position in the rail, if the screen is one of its items.
    pub fn rail_index(self) -> Option<usize> {
        RAIL.iter().position(|s| *s == self)
    }

    /// Whether this screen exists in a public build (the Real data screen is `private-oracle` only).
    pub fn in_public_build(self) -> bool {
        self != Self::Oracle
    }
}

/// Where the user is: a screen and a navigation history for Back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    current: Screen,
    back: Vec<Screen>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            current: Screen::Home,
            back: Vec::new(),
        }
    }
}

impl Route {
    /// The screen shown.
    pub fn current(&self) -> Screen {
        self.current
    }

    /// Goes to `screen`; going to the screen already shown changes nothing.
    pub fn navigate(&mut self, screen: Screen) {
        if screen != self.current {
            self.back.push(self.current);
            self.current = screen;
        }
    }

    /// `Ctrl+n` (1-based): the rail item at that position. False when there is none.
    pub fn navigate_rail(&mut self, n: usize) -> bool {
        match n.checked_sub(1).and_then(|i| RAIL.get(i)) {
            Some(s) => {
                self.navigate(*s);
                true
            }
            None => false,
        }
    }

    /// Back to the previous screen. False on the first screen.
    pub fn go_back(&mut self) -> bool {
        match self.back.pop() {
            Some(s) => {
                self.current = s;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_has_eight_items_in_order() {
        assert_eq!(RAIL[0], Screen::Home);
        assert_eq!(RAIL[7], Screen::Oracle);
        assert_eq!(Screen::Doctor.rail_index(), Some(4));
        assert_eq!(Screen::Fomod.rail_index(), None);
    }

    #[test]
    fn navigation_and_back() {
        let mut r = Route::default();
        assert!(!r.go_back());
        r.navigate(Screen::Mods);
        r.navigate(Screen::Mods);
        r.navigate(Screen::Install);
        assert_eq!(r.current(), Screen::Install);
        assert!(r.go_back());
        assert_eq!(r.current(), Screen::Mods);
        assert!(r.go_back());
        assert!(!r.go_back());
        assert_eq!(r.current(), Screen::Home);
    }

    #[test]
    fn rail_shortcuts() {
        let mut r = Route::default();
        assert!(r.navigate_rail(3));
        assert_eq!(r.current(), Screen::Plugins);
        assert!(!r.navigate_rail(0));
        assert!(!r.navigate_rail(9));
        assert_eq!(r.current(), Screen::Plugins);
    }

    #[test]
    fn inventory_is_consistent() {
        assert_eq!(Screen::ALL.len(), 16);
        for s in Screen::ALL {
            assert!(!s.title().is_empty());
            assert!(s.cli().starts_with("raul "));
        }
        assert!(Screen::Install.mutates());
        assert!(!Screen::Doctor.mutates());
        assert!(!Screen::Oracle.in_public_build());
        assert!(Screen::Home.in_public_build());
    }
}
