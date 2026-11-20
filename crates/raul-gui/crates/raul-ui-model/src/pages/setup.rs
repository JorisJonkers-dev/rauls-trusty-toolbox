// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the setup wizard (UI-RULES P2): New Vegas folder, TTW, Engine, Quick check, Ready.
//!
//! The wizard writes nothing to a game folder. It saves the paths with `raul config set`, makes the Instance with
//! `raul init` in the storage folder when there is none, puts the engine that ships with raul there with `raul install-nv-rs`
//! (nothing is downloaded), sets the Instance's target to the engine with `raul target` and runs the read-only `raul doctor`. It can be skipped, and run again from Settings. TTW itself is
//! set up in its own flow; the wizard only asks whether it is wanted and for the Fallout 3 folder.

use super::settings::{Command, Config, Fetch, Field, Key, Save};
use crate::wizard::DEFAULT_PROFILE;
use serde::Deserialize;

/// A step of the wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    /// Where the game is, and where the Toolbox keeps its files.
    NewVegas,
    /// Whether TTW is wanted, and the Fallout 3 folder if so.
    Ttw,
    /// The engine install.
    Engine,
    /// The read-only checks.
    Check,
    /// What was set up.
    Ready,
}

impl Step {
    /// Every step, in order.
    pub const ALL: [Step; 5] = [
        Step::NewVegas,
        Step::Ttw,
        Step::Engine,
        Step::Check,
        Step::Ready,
    ];

    /// Its number, from 1.
    pub fn number(self) -> usize {
        Step::ALL.iter().position(|s| *s == self).unwrap_or(0) + 1
    }

    fn next(self) -> Option<Step> {
        Step::ALL.get(self.number()).copied()
    }

    fn previous(self) -> Option<Step> {
        self.number().checked_sub(2).map(|i| Step::ALL[i])
    }
}

/// `raul install-nv-rs --instance <dir>`: puts the engine that ships with raul in the Instance, checked against its
/// checksums, and runs its check. It fetches nothing (#179): there is no release and no address in it.
pub fn install_command(instance: &str) -> Command {
    Command::write(
        "install-nv-rs",
        1,
        &["install-nv-rs", "--instance", instance],
    )
}

/// `raul target nv-rs --instance <dir>`: the Instance runs on the engine from now on, so the Quick check and the
/// Doctor judge the engine and not the game's own extender. The wizard runs it once the engine is in and checked.
pub fn target_command(instance: &str) -> Command {
    Command::write("target", 1, &["target", "nv-rs", "--instance", instance])
}

/// What the person reads when the engine step failed, from the last line the command wrote: the command's own prefix
/// (its name and the kind of failure) is dropped, so the words are plain (UI-RULES: no commands, no tool names).
pub fn engine_failure(line: &str) -> String {
    let mut rest = line.trim();
    if let Some(after) = ["raul install-nv-rs:", "raul target:"]
        .iter()
        .find_map(|prefix| rest.strip_prefix(prefix))
    {
        rest = after.trim_start();
        for kind in [
            "refused:",
            "verification failed:",
            "failed and was rolled back:",
        ] {
            if let Some(after) = rest.strip_prefix(kind) {
                rest = after.trim_start();
                break;
            }
        }
    }
    rest.to_owned()
}

/// Where the engine install of the wizard is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EngineStep {
    /// Not started.
    #[default]
    Idle,
    /// `raul init` makes the Instance.
    Initialising,
    /// `raul install-nv-rs` runs, then `raul target` sets the Instance to run on it.
    Installing,
    /// The engine is in the Instance and passed its check.
    Installed,
    /// A step failed; what it said. Nothing in the game folder changed.
    Failed(String),
}

/// What Next waits for, said under the disabled Next (UI-RULES G3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    /// The New Vegas folder is not saved.
    Game,
    /// The storage folder is not saved.
    Storage,
    /// A path is being saved.
    Saving,
    /// The engine is not installed.
    Engine,
    /// The checks still run.
    Checks,
}

/// One finding of the Doctor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Finding {
    /// `info`, `warning` or `error`.
    pub severity: String,
    /// What it is about.
    pub subject: String,
    /// What was found.
    pub message: String,
}

/// What the Doctor found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Checks {
    /// Findings of severity error.
    pub errors: usize,
    /// Findings of severity warning.
    pub warnings: usize,
    /// The findings, in the order printed.
    pub findings: Vec<Finding>,
}

#[derive(Deserialize)]
struct DoctorDoc {
    findings: Vec<Finding>,
    summary: DoctorSummary,
}

#[derive(Deserialize)]
struct DoctorSummary {
    errors: usize,
    warnings: usize,
}

/// Reads what `raul doctor --instance <dir> --json` printed.
///
/// # Errors
/// When it is not that document.
pub fn parse_doctor(stdout: &str) -> Result<Checks, String> {
    let doc: DoctorDoc = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul doctor report: {e}"))?;
    Ok(Checks {
        errors: doc.summary.errors,
        warnings: doc.summary.warnings,
        findings: doc.findings,
    })
}

#[derive(Deserialize)]
struct Copy {
    game: String,
    path: String,
}

/// The first copy of `game` (`new-vegas` or `fallout3`) that `raul detect --json` found.
///
/// # Errors
/// When it is not that document.
pub fn parse_detect(stdout: &str, game: &str) -> Result<Option<String>, String> {
    let copies: Vec<Copy> =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not a raul detect list: {e}"))?;
    Ok(copies.into_iter().find(|c| c.game == game).map(|c| c.path))
}

/// `raul detect`: looks in the usual Steam, GOG and Epic libraries; read only.
pub fn detect_command() -> Command {
    Command::read("detect", 1, &["detect"])
}

/// `raul init --base <game> --instance <dir>`: makes the Instance; refuses one that exists.
pub fn init_command(base: &str, instance: &str) -> Command {
    Command::write("init", 1, &["init", "--base", base, "--instance", instance])
}

/// `raul profile create --name Default --instance <dir>`: the profile the Mods and the load order live in, made
/// right after the Instance so no step ever asks for one. Running it on an Instance that has the profile already
/// changes nothing.
pub fn profile_command(instance: &str) -> Command {
    Command::write(
        "profile-create",
        2,
        &[
            "profile",
            "create",
            "--name",
            DEFAULT_PROFILE,
            "--instance",
            instance,
        ],
    )
}

/// The two commands a first run's Instance step runs, in order: `raul init` and the default profile.
pub fn init_and_profile_commands(base: &str, instance: &str) -> [Command; 2] {
    [init_command(base, instance), profile_command(instance)]
}

/// The same two commands as one job, for the equivalent-command strip: `raul init ... && raul profile create ...`.
pub fn init_and_profile_command(base: &str, instance: &str) -> Command {
    let [init, profile] = init_and_profile_commands(base, instance);
    // Only the line a person reads: the second command follows the first after `&&`, which `shown` leaves bare.
    let mut words = init.words.clone();
    words.push("&&".to_owned());
    words.push("raul".to_owned());
    words.extend(profile.words);
    Command { words, ..init }
}

/// `raul doctor --instance <dir>`: read only.
pub fn doctor_command(instance: &str) -> Command {
    Command::read("doctor", 1, &["doctor", "--instance", instance])
}

/// A storage folder to suggest when none is set: the user's data folder, then `raul`.
pub fn suggest_storage(
    xdg_data_home: Option<&str>,
    home: Option<&str>,
    local_app_data: Option<&str>,
) -> Option<String> {
    let trim = |p: &str| p.trim_end_matches(['/', '\\']).to_owned();
    if let Some(d) = xdg_data_home.filter(|d| !d.is_empty()) {
        return Some(format!("{}/raul", trim(d)));
    }
    if let Some(d) = local_app_data.filter(|d| !d.is_empty()) {
        return Some(format!("{}\\raul", trim(d)));
    }
    home.filter(|h| !h.is_empty())
        .map(|h| format!("{}/.local/share/raul", trim(h)))
}

/// The state of the wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// The step shown.
    pub step: Step,
    /// The configuration as read when the wizard opened.
    pub config: Fetch<Config>,
    /// The New Vegas folder.
    pub game: Field,
    /// The storage folder.
    pub storage: Field,
    /// Whether TTW is wanted.
    pub want_ttw: bool,
    /// The Fallout 3 folder, asked only when TTW is wanted.
    pub fallout3: Field,
    /// Next was pressed and waits for the saves of the step.
    pub advancing: bool,
    /// The last Find: `Some(None)` when it found nothing.
    pub found: Option<Option<String>>,
    /// Why Find failed, when it did.
    pub find_failed: Option<String>,
    /// The engine install.
    pub engine: EngineStep,
    /// The checks.
    pub checks: Fetch<Checks>,
}

impl Default for State {
    fn default() -> Self {
        State {
            step: Step::NewVegas,
            config: Fetch::Idle,
            game: Field::new(Key::NewVegas, None),
            storage: Field::new(Key::StorageRoot, None),
            want_ttw: false,
            fallout3: Field::new(Key::Fallout3, None),
            advancing: false,
            found: None,
            find_failed: None,
            engine: EngineStep::Idle,
            checks: Fetch::Idle,
        }
    }
}

impl State {
    /// The wizard on its first step, its fields filled from `config` and the storage folder from `suggestion`
    /// when none is set. A suggestion is typed in, not saved: Next saves it.
    pub fn opened(config: Config, suggestion: Option<&str>) -> State {
        let mut storage = Field::new(Key::StorageRoot, config.get(Key::StorageRoot));
        if storage.saved.is_none()
            && let Some(s) = suggestion
        {
            s.clone_into(&mut storage.draft);
        }
        State {
            game: Field::new(Key::NewVegas, config.get(Key::NewVegas)),
            fallout3: Field::new(Key::Fallout3, config.get(Key::Fallout3)),
            want_ttw: config.get(Key::Fallout3).is_some(),
            storage,
            config: Fetch::Ready(config),
            ..State::default()
        }
    }

    /// The fields of the step shown, in the order they are saved.
    pub fn step_fields(&self) -> Vec<Key> {
        match self.step {
            Step::NewVegas => vec![Key::NewVegas, Key::StorageRoot],
            Step::Ttw if self.want_ttw => vec![Key::Fallout3],
            _ => Vec::new(),
        }
    }

    /// The field of `key`.
    pub fn field(&self, key: Key) -> Option<&Field> {
        match key {
            Key::NewVegas => Some(&self.game),
            Key::StorageRoot => Some(&self.storage),
            Key::Fallout3 => Some(&self.fallout3),
            Key::TtwPackage | Key::Instance => None,
        }
    }

    /// The field of `key`, to change.
    pub fn field_mut(&mut self, key: Key) -> Option<&mut Field> {
        match key {
            Key::NewVegas => Some(&mut self.game),
            Key::StorageRoot => Some(&mut self.storage),
            Key::Fallout3 => Some(&mut self.fallout3),
            Key::TtwPackage | Key::Instance => None,
        }
    }

    /// Whether a save runs.
    pub fn saving(&self) -> bool {
        [&self.game, &self.storage, &self.fallout3]
            .iter()
            .any(|f| f.save == Save::Saving)
    }

    /// The next field of the step to save on the way to Next: one with a change that was not refused.
    pub fn next_save(&self) -> Option<Key> {
        self.step_fields().into_iter().find(|k| {
            self.field(*k)
                .is_some_and(|f| f.changed() && matches!(f.save, Save::Idle))
        })
    }

    /// The Instance, once the storage folder is saved.
    pub fn instance(&self) -> Option<String> {
        self.storage
            .saved
            .as_deref()
            .map(|root| format!("{}/instance", root.trim_end_matches(['/', '\\'])))
    }

    /// What keeps Next from moving on, if anything. A change Next would save does not keep it.
    pub fn missing(&self) -> Option<Missing> {
        if self.saving() {
            return Some(Missing::Saving);
        }
        let ready = |f: &Field| f.saved.is_some() || (f.changed() && f.save == Save::Idle);
        match self.step {
            Step::NewVegas if !ready(&self.game) => Some(Missing::Game),
            Step::NewVegas if !ready(&self.storage) => Some(Missing::Storage),
            Step::Engine if self.engine != EngineStep::Installed => Some(Missing::Engine),
            Step::Check if self.checks.is_pending() => Some(Missing::Checks),
            _ => None,
        }
    }

    /// Moves on when nothing is missing and nothing is left to save. Returns whether it moved.
    pub fn advance(&mut self) -> bool {
        if self.missing().is_some() || self.next_save().is_some() {
            return false;
        }
        let blocked =
            |f: &Field| f.changed() || matches!(f.save, Save::Refused(_) | Save::Failed(_));
        if self
            .step_fields()
            .iter()
            .any(|k| self.field(*k).is_some_and(blocked))
        {
            return false;
        }
        self.advancing = false;
        match self.step.next() {
            Some(next) => {
                self.step = next;
                self.found = None;
                self.find_failed = None;
                true
            }
            None => false,
        }
    }

    /// Goes back a step; the Engine and Check steps keep what they did.
    pub fn back(&mut self) {
        if let Some(previous) = self.step.previous() {
            self.step = previous;
            self.advancing = false;
            self.found = None;
            self.find_failed = None;
        }
    }

    /// Whether the checks found something to look at.
    pub fn to_look_at(&self) -> usize {
        self.checks.ready().map_or(0, |c| c.errors + c.warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_install_names_no_release_and_no_address() {
        let c = install_command("/i");
        let shown = c.shown();
        assert_eq!(shown, "raul install-nv-rs --instance /i");
        for word in ["--release", "http", "download", "fetch"] {
            assert!(!shown.contains(word), "{word}: {shown}");
        }
    }

    #[test]
    fn a_failed_engine_step_reads_without_the_commands_prefix() {
        for (line, plain) in [
            (
                "raul install-nv-rs: refused: This copy of raul does not come with the game engine.",
                "This copy of raul does not come with the game engine.",
            ),
            (
                "raul install-nv-rs: failed and was rolled back: the check did not pass",
                "the check did not pass",
            ),
            (
                "raul install-nv-rs: verification failed: it is damaged",
                "it is damaged",
            ),
            ("something else", "something else"),
        ] {
            assert_eq!(engine_failure(line), plain);
        }
    }
    use crate::pages::settings::Answer;

    fn saved(state: &mut State, key: Key, value: &str) {
        if let Some(f) = state.field_mut(key) {
            f.saved_as(&Answer::Entries(Config::with("", &[(key, value)])));
        }
    }

    #[test]
    fn the_target_command_sets_the_engine_on_the_instance() {
        let c = target_command("/storage/instance");
        assert_eq!(c.id, "target");
        assert!(c.writes);
        assert_eq!(
            c.words,
            ["target", "nv-rs", "--instance", "/storage/instance"]
        );
        assert_eq!(c.rest(), ["nv-rs", "--instance", "/storage/instance"]);
        assert_eq!(
            engine_failure("raul target: refused: the Instance is in use"),
            "the Instance is in use"
        );
    }

    #[test]
    fn the_engine_install_command_names_the_instance_and_no_release() {
        let c = install_command("/storage/instance");
        assert_eq!(c.id, "install-nv-rs");
        assert!(c.writes);
        assert_eq!(
            c.words,
            ["install-nv-rs", "--instance", "/storage/instance"]
        );
    }

    #[test]
    fn steps_run_in_order_and_back() {
        assert_eq!(Step::NewVegas.number(), 1);
        assert_eq!(Step::Ready.number(), 5);
        assert_eq!(Step::Ready.next(), None);
        assert_eq!(Step::NewVegas.previous(), None);
        assert_eq!(Step::Check.previous(), Some(Step::Engine));
    }

    #[test]
    fn the_first_step_needs_the_game_and_the_storage_folder() {
        let mut s = State::opened(Config::with("", &[]), None);
        assert_eq!(s.missing(), Some(Missing::Game));
        assert!(!s.advance());
        s.game.draft = "/games/nv".into();
        assert_eq!(s.missing(), Some(Missing::Storage));
        s.storage.draft = "/s".into();
        assert_eq!(s.missing(), None, "Next saves what is typed");
        assert_eq!(s.next_save(), Some(Key::NewVegas));
        assert!(!s.advance(), "not before the saves");
        saved(&mut s, Key::NewVegas, "/games/nv");
        assert_eq!(s.next_save(), Some(Key::StorageRoot));
        saved(&mut s, Key::StorageRoot, "/s");
        assert!(s.advance());
        assert_eq!(s.step, Step::Ttw);
        assert_eq!(s.instance().as_deref(), Some("/s/instance"));
    }

    #[test]
    fn a_refused_path_stops_next_until_it_is_changed() {
        let mut s = State::opened(Config::with("", &[(Key::StorageRoot, "/s")]), None);
        s.game.draft = "/downloads".into();
        s.game.save = Save::Refused("not a New Vegas folder".into());
        assert_eq!(s.next_save(), None);
        assert!(!s.advance());
        assert_eq!(s.step, Step::NewVegas);
    }

    #[test]
    fn a_suggested_storage_folder_is_typed_not_saved() {
        let s = State::opened(Config::with("", &[]), Some("/home/p/.local/share/raul"));
        assert_eq!(s.storage.draft, "/home/p/.local/share/raul");
        assert_eq!(s.storage.saved, None);
        assert!(s.storage.changed());
        let kept = State::opened(Config::with("", &[(Key::StorageRoot, "/mine")]), Some("/x"));
        assert_eq!(kept.storage.draft, "/mine");
    }

    #[test]
    fn ttw_is_optional_and_asks_for_fallout3_only_when_wanted() {
        let mut s = State {
            step: Step::Ttw,
            ..State::default()
        };
        assert!(s.step_fields().is_empty());
        assert_eq!(s.missing(), None);
        s.want_ttw = true;
        assert_eq!(s.step_fields(), [Key::Fallout3]);
        assert_eq!(s.missing(), None, "the Fallout 3 folder can be set later");
        assert!(s.advance());
        assert_eq!(s.step, Step::Engine);
    }

    #[test]
    fn the_engine_and_the_checks_hold_next() {
        let mut s = State {
            step: Step::Engine,
            ..State::default()
        };
        assert_eq!(s.missing(), Some(Missing::Engine));
        s.engine = EngineStep::Installed;
        assert!(s.advance());
        assert_eq!(s.missing(), Some(Missing::Checks));
        s.checks = Fetch::Failed("could not run".into());
        assert!(s.advance(), "a failed check does not trap the wizard");
        assert_eq!(s.step, Step::Ready);
        s.back();
        assert_eq!(s.step, Step::Check);
    }

    #[test]
    fn doctor_and_detect_are_read() {
        let c = parse_doctor(
            r#"{"base":"/g","findings":[{"code":"a.b","severity":"warning","subject":"x","message":"y"}],"summary":{"info":0,"warnings":1,"errors":0}}"#,
        )
        .unwrap();
        assert_eq!((c.errors, c.warnings, c.findings.len()), (0, 1, 1));
        assert!(parse_doctor("{}").is_err());
        let list = r#"[{"game":"fallout3","path":"/g/fo3","storefront":"Steam","version":null,"supported_version":null,"missing_dlc":[]},{"game":"new-vegas","path":"/g/nv","storefront":"GOG","version":"1.4.0.525","supported_version":true,"missing_dlc":[]}]"#;
        assert_eq!(
            parse_detect(list, "new-vegas").unwrap().as_deref(),
            Some("/g/nv")
        );
        assert_eq!(
            parse_detect(list, "fallout3").unwrap().as_deref(),
            Some("/g/fo3")
        );
        assert_eq!(parse_detect("[]", "new-vegas").unwrap(), None);
        assert!(parse_detect("{}", "new-vegas").is_err());
    }

    #[test]
    fn commands_show_as_typed() {
        assert_eq!(
            init_command("/g/nv", "/s/instance").shown(),
            "raul init --base /g/nv --instance /s/instance"
        );
        assert_eq!(doctor_command("/i").shown(), "raul doctor --instance /i");
        assert_eq!(detect_command().shown(), "raul detect");
        assert!(init_command("/a", "/b").writes && !doctor_command("/i").writes);
    }

    #[test]
    fn the_instance_step_makes_the_default_profile_with_it() {
        let [init, profile] = init_and_profile_commands("/g/nv", "/s/instance");
        assert_eq!(
            init.shown(),
            "raul init --base /g/nv --instance /s/instance"
        );
        assert_eq!(
            profile.shown(),
            "raul profile create --name Default --instance /s/instance"
        );
        assert!(profile.writes);
        assert_eq!(profile.id, "profile-create");
        assert_eq!(
            init_and_profile_command("/g/nv", "/s/instance").shown(),
            "raul init --base /g/nv --instance /s/instance && raul profile create --name Default --instance /s/instance"
        );
    }

    #[test]
    fn the_storage_suggestion_follows_the_platform_folders() {
        assert_eq!(
            suggest_storage(Some("/x/data/"), Some("/home/p"), None).as_deref(),
            Some("/x/data/raul")
        );
        assert_eq!(
            suggest_storage(None, Some("/home/p"), None).as_deref(),
            Some("/home/p/.local/share/raul")
        );
        assert_eq!(
            suggest_storage(None, None, Some(r"C:\Users\p\AppData\Local")).as_deref(),
            Some(r"C:\Users\p\AppData\Local\raul")
        );
        assert_eq!(suggest_storage(Some(""), None, None), None);
    }
}
