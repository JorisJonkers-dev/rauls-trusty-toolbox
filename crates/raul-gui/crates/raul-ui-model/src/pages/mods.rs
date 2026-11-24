// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the Mods page: the Profile's Mods in priority order with whether each is enabled, a
//! search and a filter over them, the selected Mod, and the one change at a time the page makes (enable, disable,
//! move one place, remove).
//!
//! The page reads `raul mods list`, `raul lod mods` and `raul addons list`. The second names the generated LOD Mods,
//! which get their own treatment: Regenerate on the LOD page instead of Remove and Update, no update check and no
//! conflict warnings, with one line that says why. The third names raul's add-on Mods, which are tagged as made by
//! raul and otherwise are Mods like any other: turning one off keeps it off. Every change is one `raul mods` command; the list is read again after it.

use super::settings::Command;
use serde::Deserialize;

/// `raul mods list --instance <dir>`: the Profile's Mods in priority order.
pub fn list_command(instance: &str) -> Command {
    Command::read("mods-list", 2, &["mods", "list", "--instance", instance])
}

/// `raul rollback --instance <dir>`: takes back the last change of the Instance, here the last Mods change.
pub fn undo_command(instance: &str) -> Command {
    Command::write("rollback", 1, &["rollback", "--instance", instance])
}

/// `raul lod mods --instance <dir>`: which of them are generated LOD.
pub fn generated_command(instance: &str) -> Command {
    Command::read("lod-mods", 2, &["lod", "mods", "--instance", instance])
}

/// `raul addons list --instance <dir>`: which of them are raul's add-ons.
pub fn addons_command(instance: &str) -> Command {
    Command::read(
        "addons-list",
        2,
        &["addons", "list", "--instance", instance],
    )
}

/// `raul mods enable <mod> --instance <dir>`.
pub fn enable_command(instance: &str, name: &str) -> Command {
    Command::write(
        "mods-enable",
        2,
        &["mods", "enable", name, "--instance", instance],
    )
}

/// `raul mods disable <mod> --instance <dir>`.
pub fn disable_command(instance: &str, name: &str) -> Command {
    Command::write(
        "mods-disable",
        2,
        &["mods", "disable", name, "--instance", instance],
    )
}

/// `raul mods reorder <mod> --position <n> --instance <dir>`: the Mod ends at place `position` (1 is the highest
/// priority).
pub fn move_command(instance: &str, name: &str, position: usize) -> Command {
    let position = position.to_string();
    Command::write(
        "mods-reorder",
        2,
        &[
            "mods",
            "reorder",
            name,
            "--position",
            &position,
            "--instance",
            instance,
        ],
    )
}

/// `raul mods remove <mod> --instance <dir>`: the Mod's folder goes to the Instance's trash and its plugins leave
/// the lists.
pub fn remove_command(instance: &str, name: &str) -> Command {
    Command::write(
        "mods-remove",
        2,
        &["mods", "remove", name, "--instance", instance],
    )
}

/// How out of date a generated LOD Mod is, as the run's recorded fingerprint and the Load order and settings now say
/// (raul#66). The page shows it as a chip next to the Mod; a Mod that is current or not measured has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staleness {
    /// A change since the run may show in the LOD.
    May,
    /// A change since the run shows in the LOD: generate again.
    Must,
}

/// What marks a Mod as generated LOD.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Generated {
    /// It is this Profile's own LOD Mod, the one a run replaces.
    pub own: bool,
    /// It has the name every Profile shared before; the Profile's next run puts its own Mod in its place.
    pub legacy: bool,
    /// The generator version that built it, when its marker says.
    pub version: Option<String>,
    /// How out of date it is; `None` when it is current or nothing measures it.
    pub stale: Option<Staleness>,
    /// What changed, in a line, when `stale` is set.
    pub stale_reason: Option<String>,
}

/// One entry of the Profile's mod list.
#[allow(clippy::struct_excessive_bools)] // each is a fact `raul mods list` reports about the entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModEntry {
    /// The folder name, or a separator's label.
    pub name: String,
    /// Its place, 1 the highest priority (it wins a file conflict); separators count.
    pub position: usize,
    /// Whether the Profile enables it.
    pub enabled: bool,
    /// A separator: a heading in the list, not a Mod.
    pub separator: bool,
    /// An entry the game manages itself (shown, never changed here).
    pub foreign: bool,
    /// Whether its folder exists.
    pub installed: bool,
    /// Set when it is generated LOD.
    pub generated: Option<Generated>,
    /// It is one of raul's add-ons, made on this machine (`raul addons`).
    pub addon: bool,
}

impl ModEntry {
    /// An enabled, installed Mod at `position`, as fixtures and tests write one.
    pub fn named(name: impl Into<String>, position: usize) -> ModEntry {
        ModEntry {
            name: name.into(),
            position,
            enabled: true,
            separator: false,
            foreign: false,
            installed: true,
            generated: None,
            addon: false,
        }
    }

    /// Whether the page may change it: a Mod, not a separator or an entry the game manages.
    pub fn changeable(&self) -> bool {
        !self.separator && !self.foreign
    }
}

/// Which entries the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// Every entry.
    #[default]
    All,
    /// Enabled Mods.
    Enabled,
    /// Disabled Mods.
    Disabled,
}

impl Filter {
    /// Every filter, in the order the page offers them.
    pub const ALL: [Filter; 3] = [Filter::All, Filter::Enabled, Filter::Disabled];
}

/// Where the page is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// No Instance is set.
    #[default]
    NoInstance,
    /// The list is being read.
    Loading,
    /// The list is shown.
    Listed,
    /// A change runs.
    Changing,
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

/// The state of the Mods page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// Where it is.
    pub phase: Phase,
    /// The Instance the list was read from.
    pub instance: Option<String>,
    /// The Profile, once a command named it.
    pub profile: Option<String>,
    /// The entries, highest priority first.
    pub mods: Vec<ModEntry>,
    /// The search text; empty shows every entry.
    pub query: String,
    /// The filter.
    pub filter: Filter,
    /// The selected Mod, by name.
    pub selected: Option<String>,
    /// The last problem, shown until the next step starts.
    pub problem: Option<Problem>,
    /// The generated LOD Mods, as last read; the entries of the same names are marked.
    pub lod: GeneratedList,
    /// The Mod names of raul's add-ons, as last read; the entries of the same names are marked.
    pub addons: Vec<String>,
    /// What Undo would take back, in a plain line: the last change made on this page; `None` when there is nothing to
    /// undo.
    pub undo: Option<String>,
    /// When what Undo would take back is a Remove: the Mod's name, and whether a list has shown it gone yet. The Undo
    /// is dropped when the Mod is listed again by another route (installed again), as a second copy would be wrong.
    pub undo_removed: Option<(String, bool)>,
}

impl State {
    /// The list is read (again).
    pub fn loading(&mut self) {
        self.phase = Phase::Loading;
        self.problem = None;
    }

    /// The list arrived. The selection stays when the Mod is still there.
    pub fn listed(&mut self, profile: String, mods: Vec<ModEntry>) {
        self.profile = Some(profile);
        self.mods = mods;
        if let Some(name) = &self.selected
            && !self.mods.iter().any(|m| &m.name == name)
        {
            self.selected = None;
        }
        self.apply_marks();
        if let Some((name, gone)) = &mut self.undo_removed {
            let listed = self.mods.iter().any(|m| !m.separator && &m.name == name);
            if !listed {
                *gone = true;
            } else if *gone {
                self.undo = None;
                self.undo_removed = None;
            }
        }
        self.phase = Phase::Listed;
    }

    /// The generated LOD Mods arrived: they are marked, every other entry is not. The list may come before or after.
    pub fn mark_generated(&mut self, generated: GeneratedList) {
        self.lod = generated;
        self.apply_marks();
    }

    /// raul's add-on Mods arrived: they are marked, every other entry is not. The list may come before or after.
    pub fn mark_addons(&mut self, names: Vec<String>) {
        self.addons = names;
        self.apply_marks();
    }

    fn apply_marks(&mut self) {
        for m in &mut self.mods {
            m.addon = !m.separator && self.addons.iter().any(|a| a.eq_ignore_ascii_case(&m.name));
            m.generated = self
                .lod
                .mods
                .iter()
                .find(|g| g.name.eq_ignore_ascii_case(&m.name))
                .map(|g| Generated {
                    own: g.own,
                    legacy: g.legacy,
                    version: g.lodwright_version.clone(),
                    stale: g.stale.as_ref().and_then(|s| match s.state.as_str() {
                        "may" => Some(Staleness::May),
                        "must" => Some(Staleness::Must),
                        _ => None,
                    }),
                    stale_reason: g
                        .stale
                        .as_ref()
                        .filter(|s| matches!(s.state.as_str(), "may" | "must"))
                        .map(|s| s.reason.clone()),
                });
        }
    }

    /// A change was started.
    pub fn changing(&mut self) {
        self.phase = Phase::Changing;
        self.problem = None;
    }

    /// A change went through: Undo can take it back, and says what it would take back.
    pub fn undoable(&mut self, what: String) {
        self.undo = Some(what);
        self.undo_removed = None;
    }

    /// A Remove went through: like [`State::undoable`], and the Undo is dropped when the Mod comes back by another
    /// route.
    pub fn undoable_removal(&mut self, what: String, name: &str) {
        self.undoable(what);
        self.undo_removed = Some((name.to_owned(), false));
    }

    /// The undo went through (or the change it was for is gone): nothing is left to undo here.
    pub fn undone(&mut self) {
        self.undo = None;
        self.undo_removed = None;
    }

    /// A step failed; the list stays as it was read.
    pub fn failed(&mut self, problem: Problem) {
        if self.instance.is_some() {
            self.phase = Phase::Listed;
        }
        self.problem = Some(problem);
    }

    /// Whether a command of this page runs.
    pub fn busy(&self) -> bool {
        matches!(self.phase, Phase::Loading | Phase::Changing)
    }

    /// How many Mods (not separators) the Profile lists.
    pub fn mod_count(&self) -> usize {
        self.mods.iter().filter(|m| !m.separator).count()
    }

    /// How many of them are enabled.
    pub fn enabled_count(&self) -> usize {
        self.mods
            .iter()
            .filter(|m| !m.separator && m.enabled)
            .count()
    }

    /// Whether the search or the filter hides anything.
    pub fn is_filtered(&self) -> bool {
        !self.query.trim().is_empty() || self.filter != Filter::All
    }

    /// Clears the search and the filter.
    pub fn clear_filter(&mut self) {
        self.query.clear();
        self.filter = Filter::All;
    }

    /// The entries the search and the filter let through, in priority order. A filtered list shows Mods only.
    pub fn visible(&self) -> Vec<&ModEntry> {
        let q = self.query.trim().to_lowercase();
        self.mods
            .iter()
            .filter(|m| !(self.is_filtered() && m.separator))
            .filter(|m| match self.filter {
                Filter::All => true,
                Filter::Enabled => m.enabled,
                Filter::Disabled => !m.enabled,
            })
            .filter(|m| q.is_empty() || m.name.to_lowercase().contains(&q))
            .collect()
    }

    /// How many Mods the filtered list shows.
    pub fn visible_mods(&self) -> usize {
        self.visible().iter().filter(|m| !m.separator).count()
    }

    /// Selects the entry `name`; an unknown name clears the selection.
    pub fn select(&mut self, name: &str) {
        self.selected = self
            .mods
            .iter()
            .any(|m| m.name == name)
            .then(|| name.to_owned());
    }

    /// The selected entry.
    pub fn selected(&self) -> Option<&ModEntry> {
        let name = self.selected.as_deref()?;
        self.mods.iter().find(|m| m.name == name)
    }

    /// The place `name` moves to one step up (`up`) or down, or `None` at the end of the list.
    pub fn step(&self, name: &str, up: bool) -> Option<usize> {
        let at = self.mods.iter().find(|m| m.name == name)?.position;
        if up {
            (at > 1).then(|| at - 1)
        } else {
            (at < self.mods.len()).then_some(at + 1)
        }
    }
}

#[derive(Deserialize)]
struct ListDoc {
    profile: String,
    mods: Vec<RowDoc>,
}

#[derive(Deserialize)]
struct RowDoc {
    position: usize,
    name: String,
    state: String,
    kind: String,
    installed: bool,
}

/// Reads what `raul mods list --json` printed: the Profile and its entries.
///
/// # Errors
/// The output is not such a document, in words.
pub fn parse_list(stdout: &str) -> Result<(String, Vec<ModEntry>), String> {
    let doc: ListDoc =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not a mod list: {e}"))?;
    let mods = doc
        .mods
        .into_iter()
        .map(|r| ModEntry {
            enabled: r.state != "disabled",
            foreign: r.state == "foreign",
            separator: r.kind == "separator",
            installed: r.installed,
            position: r.position,
            name: r.name,
            generated: None,
            addon: false,
        })
        .collect();
    Ok((doc.profile, mods))
}

/// One generated LOD Mod, as `raul lod mods --json` (Contract `raul-lod-mods` v1) names it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GeneratedRow {
    /// The Mod's folder name.
    pub name: String,
    /// It is the Profile's own LOD Mod.
    pub own: bool,
    /// It has the old shared name.
    pub legacy: bool,
    /// The generator version, when its marker says.
    pub lodwright_version: Option<String>,
    /// How out of date it is, for the Profile's own Mod.
    #[serde(default)]
    pub stale: Option<StaleRow>,
    /// When it was generated, in seconds since 1970 UTC, when the marker file says.
    #[serde(default)]
    pub generated_at: Option<u64>,
    /// How many files it holds, when the folder could be read.
    #[serde(default)]
    pub files: Option<u64>,
}

/// The staleness `raul lod mods --json` reports: `unknown`, `current`, `may` or `must`, and why.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StaleRow {
    /// The state.
    pub state: String,
    /// What changed, or why it is not known.
    #[serde(default)]
    pub reason: String,
}

/// The generated LOD Mods of a Profile.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneratedList {
    /// The name a run gives the Profile's own LOD Mod.
    pub own_name: String,
    /// The generated LOD Mods the Profile lists.
    pub mods: Vec<GeneratedRow>,
}

#[derive(Deserialize)]
struct GeneratedDoc {
    contract: String,
    schema_version: u32,
    own_name: String,
    mods: Vec<GeneratedRow>,
}

/// Reads what `raul lod mods --json` printed.
///
/// # Errors
/// The output is not a `raul-lod-mods` version 1 document, in words.
pub fn parse_generated(stdout: &str) -> Result<GeneratedList, String> {
    let doc: GeneratedDoc =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not a raul-lod-mods: {e}"))?;
    if doc.contract != "raul-lod-mods" || doc.schema_version != 1 {
        return Err(format!(
            "{} version {} is not a raul-lod-mods version 1",
            doc.contract, doc.schema_version
        ));
    }
    Ok(GeneratedList {
        own_name: doc.own_name,
        mods: doc.mods,
    })
}

#[derive(Deserialize)]
struct AddonsDoc {
    schema_version: u32,
    contract: String,
    addons: Vec<AddonRow>,
}

#[derive(Deserialize)]
struct AddonRow {
    name: String,
    enabled: Option<bool>,
}

/// Reads what `raul addons list --json` printed: the names of the add-on Mods the Profile lists.
///
/// # Errors
/// The output is not a `raul-addons` version 1 document, in words.
pub fn parse_addons(stdout: &str) -> Result<Vec<String>, String> {
    let doc: AddonsDoc =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not a raul-addons: {e}"))?;
    if doc.contract != "raul-addons" || doc.schema_version != 1 {
        return Err(format!(
            "{} version {} is not a raul-addons version 1",
            doc.contract, doc.schema_version
        ));
    }
    Ok(doc
        .addons
        .into_iter()
        .filter(|a| a.enabled.is_some())
        .map(|a| a.name)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_on_mods_are_marked_by_name_whether_the_list_comes_first_or_last() {
        let doc = r#"{"schema_version":1,"contract":"raul-addons","instance":"/i","profile":"Default","dry_run":true,
            "addons":[{"id":"small-lamp-glow","version":1,"name":"Raul Small Lamp Glow","plugin":"Raul Small Lamp Glow.esp",
            "trigger_plugin":"Street Lights TTW.esm","summary":"s","status":"up_to_date","enabled":false,"changes":[],"error":null},
            {"id":"other","version":1,"name":"Not Listed","plugin":"x.esp","trigger_plugin":"y.esm","summary":"s",
            "status":"not_needed","enabled":null,"changes":[],"error":null}]}"#;
        let names = parse_addons(doc).unwrap();
        assert_eq!(names, ["Raul Small Lamp Glow"]);
        let mut s = listed();
        s.mark_addons(names.clone());
        assert!(s.mods.iter().all(|m| !m.addon), "no entry of that name yet");
        let mut mods = s.mods.clone();
        mods.push(ModEntry::named("raul small lamp glow", 5));
        s.listed("Default".into(), mods);
        assert!(
            s.mods
                .iter()
                .any(|m| m.addon && m.name == "raul small lamp glow")
        );
        assert!(
            parse_addons(r#"{"schema_version":2,"contract":"raul-addons","addons":[]}"#).is_err()
        );
        assert!(parse_addons("not json").is_err());
    }

    fn listed() -> State {
        let mut s = State {
            instance: Some("/i".into()),
            ..State::default()
        };
        let mut sep = ModEntry::named("Visuals", 2);
        sep.separator = true;
        let mut off = ModEntry::named("Beta", 3);
        off.enabled = false;
        s.listed(
            "Default".into(),
            vec![
                ModEntry::named("Alpha", 1),
                sep,
                off,
                ModEntry::named("Generated LOD (Default)", 4),
            ],
        );
        s
    }

    #[test]
    fn the_staleness_of_the_generated_mod_reads_from_the_command_and_current_shows_no_chip() {
        let doc = |stale: &str| {
            format!(
                r#"{{"schema_version":1,"contract":"raul-lod-mods","instance":"/i","profile":"Default",
            "own_name":"Generated LOD (Default)","mods":[{{"name":"Generated LOD (Default)","position":1,
            "enabled":true,"own":true,"legacy":false,"profile":"Default","request_digest":"sha256:00",
            "lodwright_version":"0.3.0","stale":{stale}}}]}}"#
            )
        };
        let mut s = State::default();
        s.listed(
            "Default".into(),
            vec![ModEntry::named("Generated LOD (Default)", 1)],
        );
        for (json, want) in [
            (
                r#"{"state":"must","reason":"Settings."}"#,
                Some(Staleness::Must),
            ),
            (r#"{"state":"may","reason":"Order."}"#, Some(Staleness::May)),
            (r#"{"state":"current","reason":""}"#, None),
            (r#"{"state":"unknown","reason":"No run."}"#, None),
            ("null", None),
        ] {
            s.mark_generated(parse_generated(&doc(json)).unwrap());
            let g = s.mods[0].generated.clone().unwrap();
            assert_eq!(g.stale, want, "{json}");
            assert_eq!(g.stale_reason.is_some(), want.is_some(), "{json}");
        }
    }

    #[test]
    fn the_list_and_the_generated_mods_read_from_the_commands() {
        let list = r#"{"profile":"Default","mods":[
            {"position":1,"name":"Alpha","state":"enabled","kind":"mod","installed":true,"categories":[]},
            {"position":2,"name":"Visuals","state":"enabled","kind":"separator","installed":true,"categories":[]},
            {"position":3,"name":"Beta","state":"disabled","kind":"mod","installed":false,"categories":[]},
            {"position":4,"name":"DLC","state":"foreign","kind":"mod","installed":true,"categories":[]}]}"#;
        let (profile, mods) = parse_list(list).unwrap();
        assert_eq!(profile, "Default");
        assert!(mods[1].separator && !mods[1].changeable());
        assert!(!mods[2].enabled && !mods[2].installed);
        assert!(mods[3].foreign && mods[3].enabled && !mods[3].changeable());
        assert!(parse_list("[]").is_err());

        let generated = r#"{"schema_version":1,"contract":"raul-lod-mods","instance":"/i","profile":"Default",
            "own_name":"Generated LOD (Default)","mods":[{"name":"generated lod (default)","position":4,
            "enabled":true,"own":true,"legacy":false,"profile":"Default","request_digest":"d","lodwright_version":"0.3.0"}]}"#;
        let g = parse_generated(generated).unwrap();
        let mut s = listed();
        s.mark_generated(g.clone());
        let lod = &s.mods[3];
        assert_eq!(
            lod.generated,
            Some(Generated {
                own: true,
                legacy: false,
                version: Some("0.3.0".into()),
                stale: None,
                stale_reason: None,
            })
        );
        assert!(s.mods[0].generated.is_none());
        s.listed("Default".into(), s.mods.clone());
        assert!(
            s.mods[3].generated.is_some(),
            "a new list keeps the marks it had"
        );
        s.mark_generated(GeneratedList::default());
        assert!(
            s.mods[3].generated.is_none(),
            "a second read replaces the marks"
        );
        assert!(parse_generated(&generated.replace("raul-lod-mods", "other")).is_err());
    }

    #[test]
    fn the_search_and_the_filter_narrow_the_list_and_clear_resets_both() {
        let mut s = listed();
        assert_eq!(s.visible().len(), 4);
        assert_eq!((s.mod_count(), s.enabled_count()), (3, 2));
        s.filter = Filter::Disabled;
        assert_eq!(
            s.visible()
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>(),
            ["Beta"]
        );
        s.filter = Filter::Enabled;
        s.query = "lod".into();
        assert_eq!(s.visible_mods(), 1);
        assert!(s.is_filtered());
        s.clear_filter();
        assert!(!s.is_filtered());
        assert_eq!(s.visible().len(), 4, "the separator is back");
    }

    #[test]
    fn a_mod_steps_one_place_and_stops_at_the_ends() {
        let s = listed();
        assert_eq!(s.step("Alpha", true), None);
        assert_eq!(s.step("Alpha", false), Some(2));
        assert_eq!(s.step("Beta", true), Some(2));
        assert_eq!(s.step("Generated LOD (Default)", false), None);
        assert_eq!(s.step("Missing", true), None);
    }

    #[test]
    fn the_selection_follows_the_list_and_a_failure_keeps_it() {
        let mut s = listed();
        s.select("Beta");
        assert_eq!(s.selected().map(|m| m.position), Some(3));
        s.select("Nope");
        assert!(s.selected().is_none());
        s.select("Beta");
        s.changing();
        assert!(s.busy());
        s.failed(Problem {
            id: "x",
            cause: "no".into(),
            command: None,
        });
        assert_eq!(s.phase, Phase::Listed);
        assert!(s.selected().is_some());
        s.listed("Default".into(), vec![ModEntry::named("Alpha", 1)]);
        assert!(
            s.selected().is_none(),
            "a removed Mod is no longer selected"
        );
    }

    #[test]
    fn the_undo_of_a_remove_goes_when_the_mod_is_listed_again() {
        let mut s = State::default();
        s.listed("Default".into(), vec![ModEntry::named("Alpha", 1)]);
        s.undoable_removal("Undo brings Alpha back from the trash.".to_owned(), "Alpha");
        // a list that still shows it (read before the removal landed) does not drop the Undo
        s.listed("Default".into(), vec![ModEntry::named("Alpha", 1)]);
        assert!(s.undo.is_some());
        s.listed("Default".into(), vec![ModEntry::named("Beta", 1)]);
        assert!(s.undo.is_some(), "gone from the list: Undo stays");
        s.listed(
            "Default".into(),
            vec![ModEntry::named("Beta", 1), ModEntry::named("Alpha", 2)],
        );
        assert_eq!(s.undo, None, "installed again: nothing to undo");
        // another kind of change is not touched by the list
        s.undoable("Undo turns Beta off again.".to_owned());
        s.listed("Default".into(), vec![ModEntry::named("Beta", 1)]);
        assert!(s.undo.is_some());
    }

    #[test]
    fn a_change_can_be_undone_once_and_the_undo_names_what_it_takes_back() {
        let mut s = State::default();
        assert_eq!(s.undo, None, "nothing to undo at first");
        s.undoable("Undo turns A off again.".to_owned());
        assert_eq!(s.undo.as_deref(), Some("Undo turns A off again."));
        s.undone();
        assert_eq!(s.undo, None);
        let c = undo_command("/i");
        assert!(c.writes);
        assert_eq!(c.shown(), "raul rollback --instance /i");
    }
}
