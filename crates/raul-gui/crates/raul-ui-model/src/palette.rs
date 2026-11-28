// SPDX-License-Identifier: GPL-3.0-only
//! The command palette (F-OVERLAY-08): one registry of the pages and the actions, plus the mods the
//! caller passes in; the query picks the rows, the arrow keys wrap the highlight, Enter runs the pick
//! and a query that matches nothing shows [`NOTHING_FOUND`] (redesign SPEC 7).

use crate::keymap::{Chord, Command, Keymap};
use crate::route::{RAIL, Screen};
use serde::{Deserialize, Serialize};

/// A page the palette opens (F-SHELL-04): the sidebar's seven, and the Real data page of a
/// `private-oracle` build after them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Page {
    /// The start page: the launch panel and what needs attention.
    Home,
    /// The Mods of the profile: enable, order and remove them.
    Mods,
    /// The plugin order, with the sort and its preview.
    LoadOrder,
    /// Distant land: build it and see which blocks are up to date.
    Lod,
    /// The Toolbox's own version and the update check, and the engine release.
    System,
    /// The checks of the setup.
    Doctor,
    /// The paths the Toolbox uses.
    Settings,
    /// The Real data status, owner builds only.
    RealData,
}

impl Page {
    /// Every page, in sidebar order; [`Page::RealData`] comes after the sidebar's seven and is
    /// `private-oracle` only.
    pub const ALL: [Page; 8] = [
        Page::Home,
        Page::Mods,
        Page::LoadOrder,
        Page::Lod,
        Page::System,
        Page::Doctor,
        Page::Settings,
        Page::RealData,
    ];

    /// The page's title in the sidebar and the palette.
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Mods => "Mods",
            Self::LoadOrder => "Load order",
            Self::Lod => "LOD",
            Self::System => "System",
            Self::Doctor => "Doctor",
            Self::Settings => "Settings",
            Self::RealData => "Real data",
        }
    }

    /// The screen of the earlier inventory the page continues, when there is one: the command-parity
    /// registry (ADR-0003) covers a page through it. `Lod`, `System` and `Settings` are pages the
    /// inventory does not have.
    pub fn screen(self) -> Option<Screen> {
        match self {
            Self::Home => Some(Screen::Home),
            Self::Mods => Some(Screen::Mods),
            Self::LoadOrder => Some(Screen::Plugins),
            Self::Doctor => Some(Screen::Doctor),
            Self::RealData => Some(Screen::Oracle),
            Self::Lod | Self::System | Self::Settings => None,
        }
    }
}

/// An action of the app the palette runs: the action items of F-OVERLAY-08.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Check the setup, then start the game (F-PLAY-01).
    Play,
    /// Open the Install a mod dialog (F-INSTALL-01).
    InstallMod,
    /// Sort the load order and show the preview (F-LOAD-05).
    SortLoadOrder,
    /// Build the LOD (F-LOD-02).
    BuildLod,
    /// Open Test your setup and run a stability test (F-TEST-01).
    RunStabilityTest,
    /// Open Test your setup and run the benchmark (F-TEST-03).
    RunBenchmark,
    /// Open System and check for updates (F-SYSTEM-01).
    CheckForUpdates,
    /// Open the New profile dialog (F-PROF-02).
    NewProfile,
}

impl Action {
    /// Every action, in the palette's order.
    pub const ALL: [Action; 8] = [
        Action::Play,
        Action::InstallMod,
        Action::SortLoadOrder,
        Action::BuildLod,
        Action::RunStabilityTest,
        Action::RunBenchmark,
        Action::CheckForUpdates,
        Action::NewProfile,
    ];

    /// The title the palette row shows.
    pub fn title(self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::InstallMod => "Install mod…",
            Self::SortLoadOrder => "Sort the load order",
            Self::BuildLod => "Build the LOD",
            Self::RunStabilityTest => "Run a stability test",
            Self::RunBenchmark => "Run the benchmark",
            Self::CheckForUpdates => "Check for updates",
            Self::NewProfile => "New profile…",
        }
    }
}

/// The kind tag a palette row shows next to its title; F-OVERLAY-08 names the tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// A page of the sidebar.
    Page,
    /// An action of the app.
    Action,
    /// A switch to a profile, the rows the adapter builds from the app's live profiles.
    Profile,
    /// A mod of the library.
    Mod,
}

impl Kind {
    /// The tag text the row shows.
    pub fn label(self) -> &'static str {
        match self {
            Self::Page => "Page",
            Self::Action => "Action",
            Self::Profile => "Profile",
            Self::Mod => "Mod",
        }
    }
}

/// What running an entry does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Do {
    /// Open a page of the sidebar.
    Page(Page),
    /// Run one of the app's actions.
    Action(Action),
    /// Jump to the Mods page and open this mod's details, with the filters cleared (F-OVERLAY-08); the
    /// string is the mod's name.
    OpenMod(String),
}

/// One row of the palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The label the row shows.
    pub title: String,
    /// The kind tag the row shows after it.
    pub kind: Kind,
    /// The shortcut, when one is bound.
    pub shortcut: Option<Chord>,
    /// What running the entry does.
    pub action: Do,
    /// Texts the search also matches, besides the title: a mod's short id and plugin file names.
    pub search: Vec<String>,
}

impl Entry {
    /// The entry's match for the query, when its letters appear in order: the gap of the title, or of
    /// one of the search texts carrying [`SEARCH_PENALTY`], the smaller the better; `None` when nothing
    /// matches.
    fn gap(&self, query: &str) -> Option<usize> {
        let title = subsequence(query, &self.title);
        let search = self
            .search
            .iter()
            .filter_map(|t| subsequence(query, t).map(|g| g + SEARCH_PENALTY))
            .min();
        title.into_iter().chain(search).min()
    }
}

/// Whether the letters of `query` appear in `text` in order, ignoring case, and how far apart they lie:
/// the sum of the byte offsets between the finds, which ranks a tighter match first.
fn subsequence(query: &str, text: &str) -> Option<usize> {
    let text = text.to_lowercase();
    let mut pos = 0;
    let mut gap = 0;
    for q in query.to_lowercase().chars().filter(|c| !c.is_whitespace()) {
        let found = text[pos..].find(q)?;
        gap += found;
        pos += found + q.len_utf8();
    }
    Some(gap)
}

/// The handicap a match on a search text carries over a title match, so a close title match ranks first.
const SEARCH_PENALTY: usize = 100;

/// The chord the keymap binds to `screen`'s position on the rail of the earlier inventory, when the
/// screen is on the rail.
fn rail_chord(keymap: &Keymap, screen: Screen) -> Option<Chord> {
    let n = u8::try_from(screen.rail_index()? + 1).ok()?;
    keymap.chords_for(Command::Rail(n)).first().copied()
}

/// The registry of F-OVERLAY-08's fixed entries, in the palette's order: the seven pages of the sidebar
/// (and the Real data page of a `private-oracle` build), then the app's actions. The profiles and mods
/// of the live library are the caller's: profiles come from the app's state, mods from
/// [`mod_entries`]. `keymap` holds the chords; a page whose screen sits on the rail of the earlier
/// inventory carries its chord.
pub fn entries(keymap: &Keymap, private: bool) -> Vec<Entry> {
    let mut v = Vec::new();
    for p in Page::ALL {
        if p == Page::RealData && !private {
            continue;
        }
        let shortcut = p.screen().and_then(|s| rail_chord(keymap, s));
        v.push(Entry {
            title: p.title().to_owned(),
            kind: Kind::Page,
            shortcut,
            action: Do::Page(p),
            search: Vec::new(),
        });
    }
    for a in Action::ALL {
        v.push(Entry {
            title: a.title().to_owned(),
            kind: Kind::Action,
            shortcut: None,
            action: Do::Action(a),
            search: Vec::new(),
        });
    }
    v
}

/// A mod as the palette lists it: the name the row shows, and the short id and the plugin file names
/// the search also matches (DEMO-BUGS mods 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModRef {
    /// The mod's name.
    pub name: String,
    /// The mod's short id.
    pub short_id: String,
    /// The mod's plugin file names.
    pub plugins: Vec<String>,
}

/// One Mod entry per mod, F-OVERLAY-08's "every mod": the name the row shows, and running it jumps to
/// the Mods page and opens the mod's details. The short id and the plugin file names go into the
/// entry's search texts, so the query finds them too.
pub fn mod_entries(mods: &[ModRef]) -> Vec<Entry> {
    mods.iter()
        .map(|m| {
            let mut search = vec![m.short_id.clone()];
            search.extend(m.plugins.iter().cloned());
            Entry {
                title: m.name.clone(),
                kind: Kind::Mod,
                shortcut: None,
                action: Do::OpenMod(m.name.clone()),
                search,
            }
        })
        .collect()
}

/// The most rows the palette shows (redesign SPEC 4: "up to 9 rows").
pub const MAX_ROWS: usize = 9;

/// The text the palette shows in place of the list when nothing matches (F-OVERLAY-08).
pub const NOTHING_FOUND: &str = "Nothing found.";

/// The palette: the entries the query searches, the query typed and the highlighted row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
    /// The entries searched, in a stable order.
    all: Vec<Entry>,
    /// The query typed.
    query: String,
    /// The highlighted row.
    selected: usize,
    /// Whether it is shown.
    open: bool,
}

impl Palette {
    /// A closed palette over `all`, the entries the query searches.
    pub fn new(all: Vec<Entry>) -> Self {
        Self {
            all,
            query: String::new(),
            selected: 0,
            open: false,
        }
    }

    /// Whether it is shown.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Ctrl+K: opens with an empty query.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
    }

    /// Esc.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Ctrl+K again: closes an open palette, opens a closed one.
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// The query typed.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Sets the query and highlights the first match.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.selected = 0;
    }

    /// The matches, best first: a smaller gap between the matched letters ranks higher and a tie keeps
    /// the order of the table; a search text carries [`SEARCH_PENALTY`]. At most [`MAX_ROWS`] rows show.
    pub fn results(&self) -> Vec<&Entry> {
        let mut scored: Vec<(usize, usize, &Entry)> = self
            .all
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.gap(&self.query).map(|g| (g, i, e)))
            .collect();
        scored.sort_by_key(|(g, i, _)| (*g, *i));
        scored.truncate(MAX_ROWS);
        scored.into_iter().map(|(_, _, e)| e).collect()
    }

    /// Whether nothing matches, the state that shows [`NOTHING_FOUND`].
    pub fn is_empty_result(&self) -> bool {
        self.results().is_empty()
    }

    /// The highlighted entry.
    pub fn selected(&self) -> Option<&Entry> {
        self.results().get(self.selected).copied()
    }

    /// Moves the highlight one row, down or up; it wraps from the first row to the last and back.
    pub fn move_selection(&mut self, down: bool) {
        let n = self.results().len();
        if n == 0 {
            return;
        }
        if down {
            self.selected = (self.selected + 1) % n;
        } else {
            self.selected = (self.selected + n - 1) % n;
        }
    }

    /// Enter: closes the palette and returns the highlighted entry's action.
    pub fn run_selected(&mut self) -> Option<Do> {
        let action = self.selected().map(|e| e.action.clone());
        if action.is_some() {
            self.close();
        }
        action
    }
}

/// The screens `Ctrl+1` to `Ctrl+8` reach, as the rail shows them.
pub fn rail_titles() -> Vec<&'static str> {
    RAIL.iter().map(|s| s.title()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::Key;

    fn palette(private: bool) -> Palette {
        Palette::new(entries(&Keymap::default(), private))
    }

    fn mod_ref() -> ModRef {
        ModRef {
            name: "Nevada Skies".to_owned(),
            short_id: "nsk".to_owned(),
            plugins: vec!["Nevada Skies - Basic Edition.esp".to_owned()],
        }
    }

    #[test]
    fn public_build_has_no_real_data_entry() {
        for e in entries(&Keymap::default(), false) {
            assert_ne!(e.action, Do::Page(Page::RealData));
        }
        let owner = entries(&Keymap::default(), true);
        assert!(owner.iter().any(|e| e.action == Do::Page(Page::RealData)));
    }

    #[test]
    fn pages_carry_the_rail_chord_and_no_action_a_chord() {
        let p = palette(true);
        let results = p.results();
        let find = |action: &Do| *results.iter().find(|e| &e.action == action).unwrap();
        let doctor = find(&Do::Page(Page::Doctor));
        assert_eq!(doctor.shortcut, Some(Chord::ctrl(Key::Char('5'))));
        let load_order = find(&Do::Page(Page::LoadOrder));
        assert_eq!(load_order.shortcut, Some(Chord::ctrl(Key::Char('3'))));
        let lod_page = find(&Do::Page(Page::Lod));
        assert_eq!(lod_page.shortcut, None);
        let play = find(&Do::Action(Action::Play));
        assert_eq!(play.shortcut, None);
    }

    #[test]
    fn typing_filters_and_ranks() {
        let mut p = palette(false);
        p.open();
        assert!(p.is_open());
        let all = p.results().len();
        p.set_query("doc");
        let r = p.results();
        assert!(r.len() < all);
        assert_eq!(r[0].action, Do::Page(Page::Doctor));
        p.set_query("sort");
        assert_eq!(p.results()[0].action, Do::Action(Action::SortLoadOrder));
    }

    #[test]
    fn a_mod_is_found_by_name_short_id_and_plugin_file() {
        let mut p = Palette::new(mod_entries(&[mod_ref()]));
        p.open();
        p.set_query("nevada");
        assert_eq!(p.results().len(), 1);
        assert_eq!(
            p.results()[0].action,
            Do::OpenMod("Nevada Skies".to_owned())
        );
        p.set_query("NSK");
        assert_eq!(p.results().len(), 1, "the short id matches, any case");
        p.set_query("basic edition");
        assert_eq!(p.results().len(), 1, "a plugin file name matches");
        assert_eq!(p.results()[0].kind, Kind::Mod);
        assert_eq!(
            p.run_selected(),
            Some(Do::OpenMod("Nevada Skies".to_owned()))
        );
        assert!(!p.is_open(), "running the mod closes the palette");
    }

    #[test]
    fn results_cap_at_nine_rows() {
        let mut p = palette(true);
        p.open();
        assert_eq!(p.results().len(), MAX_ROWS);
        let mut many = Vec::new();
        for i in 0..20 {
            many.push(ModRef {
                name: format!("Mod {i}"),
                short_id: format!("m{i}"),
                plugins: Vec::new(),
            });
        }
        let mut big = Palette::new(mod_entries(&many));
        big.open();
        assert_eq!(big.results().len(), MAX_ROWS);
        assert_eq!(big.results().last().unwrap().title, "Mod 8");
    }

    #[test]
    fn an_unknown_query_reports_nothing_found() {
        let mut p = palette(false);
        p.open();
        p.set_query("zzzzqq");
        assert!(p.is_empty_result());
        assert_eq!(NOTHING_FOUND, "Nothing found.");
        assert!(p.selected().is_none());
        assert!(p.run_selected().is_none());
        assert!(p.is_open(), "nothing runs, so it stays open");
        assert_eq!(p.query(), "zzzzqq");
    }

    #[test]
    fn selection_wraps_at_both_ends() {
        let mut p = palette(false);
        p.open();
        assert_eq!(p.selected().unwrap().action, Do::Page(Page::Home));
        p.move_selection(false);
        assert_eq!(
            p.selected().unwrap().action,
            Do::Action(Action::InstallMod),
            "up from the first wraps to the last shown row"
        );
        p.move_selection(true);
        assert_eq!(
            p.selected().unwrap().action,
            Do::Page(Page::Home),
            "down from the last wraps to the first"
        );
    }

    #[test]
    fn selection_moves_and_runs() {
        let mut p = palette(false);
        p.open();
        p.set_query("home");
        assert_eq!(p.run_selected(), Some(Do::Page(Page::Home)));
        assert!(!p.is_open());
        p.toggle();
        assert!(p.is_open(), "Ctrl K toggles");
        assert_eq!(p.query(), "", "opening clears the query");
        p.toggle();
        assert!(!p.is_open());
        p.open();
        p.close();
        assert!(!p.is_open());
    }

    #[test]
    fn every_entry_has_a_kind() {
        let mut all = entries(&Keymap::default(), true);
        all.extend(mod_entries(&[mod_ref()]));
        for e in &all {
            let expected = match &e.action {
                Do::Page(_) => Kind::Page,
                Do::Action(_) => Kind::Action,
                Do::OpenMod(_) => Kind::Mod,
            };
            assert_eq!(e.kind, expected, "{:?} carries the wrong kind tag", e.title);
        }
        // F-OVERLAY-08 names the tags: Page, Action, Profile, Mod.
        assert_eq!(Kind::Page.label(), "Page");
        assert_eq!(Kind::Action.label(), "Action");
        assert_eq!(Kind::Profile.label(), "Profile");
        assert_eq!(Kind::Mod.label(), "Mod");
    }

    #[test]
    fn no_entry_carries_a_command_line() {
        let mut all = entries(&Keymap::default(), true);
        all.extend(mod_entries(&[mod_ref()]));
        for e in &all {
            assert!(
                !e.title.contains("raul "),
                "{:?} shows a command line",
                e.title
            );
        }
    }

    #[test]
    fn rail_titles_match_the_rail() {
        assert_eq!(rail_titles().len(), 8);
        assert_eq!(rail_titles()[0], "Home");
        assert_eq!(rail_titles()[7], "Real data");
    }
}
