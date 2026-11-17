// SPDX-License-Identifier: GPL-3.0-only
//! The pages of the minimum viable Toolbox and their toolkit-free state (`docs/UI-RULES.md` P1).
//!
//! The sidebar has two groups: **Play** (Home, Load order, LOD) and **System** (Versions, Settings). Keys 1 to 5
//! open them in this order. The setup wizard and the TTW flow are [`Flow`]s: they run on the same shell but are
//! not sidebar pages.
//!
//! Each page and flow has a module of its own here. The shell owns this file; a page's module is its owner's to
//! fill, so adding a page's state never needs an edit outside it.

pub mod home;
pub mod load_order;
pub mod lod;
pub mod settings;
pub mod setup;
pub mod ttw;
pub mod versions;

/// A sidebar group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    /// Home, Load order and LOD.
    Play,
    /// Versions and Settings.
    System,
}

/// A page of the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    /// The start page: the launch panel and what needs attention.
    Home,
    /// The plugin order: sort with a preview, apply and undo.
    LoadOrder,
    /// Distant land: generate it and see which blocks are up to date.
    Lod,
    /// The engine release and the Component versions.
    Versions,
    /// The paths the Toolbox uses.
    Settings,
}

impl PageId {
    /// Every page, in sidebar order.
    pub const ALL: [PageId; 5] = [
        PageId::Home,
        PageId::LoadOrder,
        PageId::Lod,
        PageId::Versions,
        PageId::Settings,
    ];

    /// The group it is listed in.
    pub fn group(self) -> Group {
        match self {
            PageId::Home | PageId::LoadOrder | PageId::Lod => Group::Play,
            PageId::Versions | PageId::Settings => Group::System,
        }
    }

    /// The number key that opens it, 1 to 5.
    pub fn key(self) -> u8 {
        match self {
            PageId::Home => 1,
            PageId::LoadOrder => 2,
            PageId::Lod => 3,
            PageId::Versions => 4,
            PageId::Settings => 5,
        }
    }

    /// The page a number key opens.
    pub fn from_key(key: u8) -> Option<PageId> {
        PageId::ALL.into_iter().find(|p| p.key() == key)
    }

    /// A stable name for logs, tests and snapshot files.
    pub fn slug(self) -> &'static str {
        match self {
            PageId::Home => "home",
            PageId::LoadOrder => "load-order",
            PageId::Lod => "lod",
            PageId::Versions => "versions",
            PageId::Settings => "settings",
        }
    }
}

/// A flow that runs on the shell without being a sidebar page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flow {
    /// The first-run wizard (UI-RULES P2).
    Setup,
    /// Setting up Tale of Two Wastelands (UI-RULES P4).
    Ttw,
}

impl Flow {
    /// Every flow.
    pub const ALL: [Flow; 2] = [Flow::Setup, Flow::Ttw];

    /// A stable name for logs, tests and snapshot files.
    pub fn slug(self) -> &'static str {
        match self {
            Flow::Setup => "setup",
            Flow::Ttw => "ttw",
        }
    }
}

/// What the shell shows in its content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum View {
    /// A sidebar page.
    Page(PageId),
    /// A flow.
    Flow(Flow),
}

impl View {
    /// A stable name for logs, tests and snapshot files.
    pub fn slug(self) -> &'static str {
        match self {
            View::Page(p) => p.slug(),
            View::Flow(f) => f.slug(),
        }
    }
}

/// How urgent a sidebar chip is (UI-RULES G2, C3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Running work or an update.
    Info,
    /// Something to look at.
    Warning,
    /// Something that blocks.
    Problem,
}

/// What a page says needs attention: shown as a chip next to it in the sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attention {
    /// How urgent it is.
    pub severity: Severity,
    /// How many things.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_open_the_pages_in_sidebar_order() {
        for (i, page) in PageId::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(page.key()), i + 1);
            assert_eq!(PageId::from_key(page.key()), Some(page));
        }
        assert_eq!(PageId::from_key(0), None);
        assert_eq!(PageId::from_key(6), None);
    }

    #[test]
    fn play_comes_before_system() {
        let groups: Vec<Group> = PageId::ALL.into_iter().map(PageId::group).collect();
        assert_eq!(
            groups,
            [
                Group::Play,
                Group::Play,
                Group::Play,
                Group::System,
                Group::System
            ]
        );
    }

    #[test]
    fn every_view_has_its_own_slug() {
        let mut slugs: Vec<&str> = PageId::ALL
            .into_iter()
            .map(|p| View::Page(p).slug())
            .chain(Flow::ALL.into_iter().map(|f| View::Flow(f).slug()))
            .collect();
        slugs.sort_unstable();
        let before = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), before);
    }

    #[test]
    fn a_problem_outranks_a_warning() {
        assert!(Severity::Problem > Severity::Warning && Severity::Warning > Severity::Info);
    }
}
