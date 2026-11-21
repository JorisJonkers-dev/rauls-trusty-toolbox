// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the LOD page: the plan (`raul lod plan`), the blocks it covers with their status and
//! progress, Generate (`raul lod generate`), Stop, and the output folder.
//!
//! A block is one world of the request: the request names its worlds, or none for every world with terrain. While
//! Generate runs every block is building and the job's progress is shown; once it ends, each block shows what the
//! run did. The output is kept as one Mod of the Profile, which the result names.

use super::settings::Command;
use crate::events::{Event, LineError, Status, parse_line};
use serde::Deserialize;

/// `raul lod plan --instance <dir>`: the request a run would send to lodwright; nothing is run.
pub fn plan_command(instance: &str) -> Command {
    Command::read("lod-plan", 2, &["lod", "plan", "--instance", instance])
}

/// `raul lod generate --instance <dir>`: runs lodwright and keeps its output as one Mod.
pub fn generate_command(instance: &str) -> Command {
    Command::write(
        "lod-generate",
        2,
        &["lod", "generate", "--instance", instance],
    )
}

/// What a run reads and writes, from the request `raul lod plan --json` prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// How many active plugins it reads.
    pub plugins: usize,
    /// The worlds it covers; empty for every world with terrain.
    pub worlds: Vec<String>,
    /// The Staging tree's `Data` folder it reads.
    pub data_folder: String,
    /// Where lodwright writes before the output becomes the Mod.
    pub output_dir: String,
    /// Terrain LOD.
    pub terrain: bool,
    /// Object LOD.
    pub objects: bool,
    /// Tree LOD.
    pub trees: bool,
}

#[derive(Deserialize)]
struct Switch {
    enabled: bool,
}

#[derive(Deserialize)]
struct RequestDoc {
    contract: String,
    schema_version: u32,
    data_folder: String,
    plugins: Vec<String>,
    worlds: Vec<String>,
    output_dir: String,
    terrain: Switch,
    objects: Switch,
    trees: Switch,
}

/// Reads the `lodwright-request` v1 document `raul lod plan --json` prints.
///
/// # Errors
/// The output is not such a document, in words.
pub fn parse_plan(stdout: &str) -> Result<Plan, String> {
    let doc: RequestDoc =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not a lodwright request: {e}"))?;
    if doc.contract != "lodwright-request" || doc.schema_version != 1 {
        return Err(format!(
            "{} version {} is not a lodwright-request version 1",
            doc.contract, doc.schema_version
        ));
    }
    Ok(Plan {
        plugins: doc.plugins.len(),
        worlds: doc.worlds,
        data_folder: doc.data_folder,
        output_dir: doc.output_dir,
        terrain: doc.terrain.enabled,
        objects: doc.objects.enabled,
        trees: doc.trees.enabled,
    })
}

/// What lodwright wrote for one world and kind.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Written {
    /// The world.
    pub world: String,
    /// What it is, such as `terrain_mesh`.
    pub kind: String,
    /// Files.
    pub files: u64,
    /// Bytes.
    pub bytes: u64,
}

#[derive(Deserialize)]
struct LodwrightDoc {
    #[serde(default)]
    written: Vec<Written>,
}

#[derive(Deserialize)]
struct ErrorDoc {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct ResultDoc {
    contract: String,
    schema_version: u32,
    status: Status,
    up_to_date: bool,
    dry_run: bool,
    #[serde(rename = "mod")]
    mod_name: Option<String>,
    files: Option<u64>,
    worlds: Vec<String>,
    lodwright: Option<serde_json::Value>,
    error: Option<ErrorDoc>,
}

/// How a `raul lod generate` run ended (Contract `raul-lod-result` v1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// How it ended.
    pub status: Status,
    /// lodwright found its output current and nothing changed.
    pub up_to_date: bool,
    /// Nothing was run or written.
    pub dry_run: bool,
    /// The Mod, when the run got that far.
    pub mod_name: Option<String>,
    /// Files in the Mod.
    pub files: Option<u64>,
    /// The worlds the request named; empty for every world with terrain.
    pub worlds: Vec<String>,
    /// What lodwright wrote.
    pub written: Vec<Written>,
    /// The stable error code and the words for people, unless the status is `success`.
    pub error: Option<(String, String)>,
}

/// Reads the `raul-lod-result` from the `raul-events` stream `raul lod generate --json` printed. As for TTW, the
/// stream must end with `finished` and its status must be the result's; anything else is not a result.
///
/// # Errors
/// No result, a newer stream or result, a result that does not read, or a stream that ended otherwise; in words.
pub fn read_result(stdout: &str) -> Result<Outcome, String> {
    let (mut document, mut finished) = (None, None);
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        match parse_line(line) {
            Ok(parsed) => match parsed.event {
                Event::Result { result } => document = Some(result),
                Event::Finished { status, .. } => finished = Some(status),
                _ => {}
            },
            Err(LineError::Newer(v)) => {
                return Err(format!(
                    "`raul lod generate` speaks raul-events version {v}; this build reads version 1"
                ));
            }
            Err(_) => {}
        }
    }
    let document = document.ok_or("`raul lod generate` printed no result")?;
    let doc: ResultDoc = serde_json::from_value(document)
        .map_err(|e| format!("the result of `raul lod generate` could not be read: {e}"))?;
    if doc.contract != "raul-lod-result" || doc.schema_version != 1 {
        return Err(format!(
            "{} version {} is not a raul-lod-result version 1",
            doc.contract, doc.schema_version
        ));
    }
    match finished {
        Some(status) if status == doc.status => {}
        Some(status) => {
            return Err(format!(
                "`raul lod generate` ended as {status:?} but its result says {:?}",
                doc.status
            ));
        }
        None => {
            return Err("`raul lod generate` stopped before it said how it ended".to_owned());
        }
    }
    let written = doc
        .lodwright
        .and_then(|v| serde_json::from_value::<LodwrightDoc>(v).ok())
        .map_or_else(Vec::new, |l| l.written);
    Ok(Outcome {
        status: doc.status,
        up_to_date: doc.up_to_date,
        dry_run: doc.dry_run,
        mod_name: doc.mod_name,
        files: doc.files,
        worlds: doc.worlds,
        written,
        error: doc.error.map(|e| (e.code, e.message)),
    })
}

/// Where the page is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// No Instance is known yet.
    #[default]
    NoInstance,
    /// The plan is being read.
    Planning,
    /// The plan is shown.
    Planned,
    /// Generate runs.
    Generating,
    /// Stop was asked; the run is ending.
    Stopping,
}

/// The status of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStatus {
    /// Not generated while the window is open; whether earlier output is current is not known.
    NotGenerated,
    /// Being generated.
    Building,
    /// Generated by the last run.
    Built,
    /// lodwright found the output current; nothing was written.
    UpToDate,
    /// The last run did not finish; the Instance is as it was.
    Failed,
}

/// One block: a world of the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The world, or `None` for every world with terrain.
    pub world: Option<String>,
    /// Its status.
    pub status: BlockStatus,
    /// Files the last run wrote for it.
    pub files: u64,
    /// Bytes the last run wrote for it.
    pub bytes: u64,
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

/// Progress of a running Generate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Progress {
    /// The phase, such as `terrain`.
    pub phase: String,
    /// Units done.
    pub done: u64,
    /// Units in all, once known.
    pub total: Option<u64>,
}

impl Progress {
    /// The share done, 0 to 100, once the total is known.
    pub fn percent(&self) -> Option<u8> {
        let total = self.total.filter(|t| *t > 0)?;
        u8::try_from(self.done.min(total) * 100 / total).ok()
    }
}

/// The state of the LOD page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// Where it is.
    pub phase: Phase,
    /// The Instance the plan was read for.
    pub instance: Option<String>,
    /// The plan, once read.
    pub plan: Option<Plan>,
    /// How the last Generate ended.
    pub outcome: Option<Outcome>,
    /// The progress of the running Generate.
    pub progress: Progress,
    /// The last problem, shown until the next step starts.
    pub problem: Option<Problem>,
}

impl State {
    /// The plan is read (again).
    pub fn planning(&mut self) {
        self.phase = Phase::Planning;
        self.problem = None;
    }

    /// The plan arrived.
    pub fn planned(&mut self, plan: Plan) {
        self.plan = Some(plan);
        self.phase = Phase::Planned;
    }

    /// Generate started.
    pub fn generating(&mut self) {
        self.phase = Phase::Generating;
        self.problem = None;
        self.progress = Progress::default();
    }

    /// Stop was asked.
    pub fn stopping(&mut self) {
        if self.phase == Phase::Generating {
            self.phase = Phase::Stopping;
        }
    }

    /// Generate ended with `outcome`; a problem from while it ran (a Stop that did not work) is over.
    pub fn generated(&mut self, outcome: Outcome) {
        self.outcome = Some(outcome);
        self.phase = Phase::Planned;
        self.problem = None;
    }

    /// The mod list holds no generated LOD Mod (the person deleted it): what the last run of this session built is
    /// not there any more, so the blocks go back to "not generated" and the result line goes away. Not while a run
    /// is on its way.
    pub fn mod_gone(&mut self) {
        if self.phase == Phase::Planned
            && self
                .outcome
                .as_ref()
                .is_some_and(|o| o.status == Status::Success && !o.dry_run)
        {
            self.outcome = None;
        }
    }

    /// The step that ran failed: back to the plan, with the problem shown.
    pub fn failed(&mut self, problem: Problem) {
        self.phase = Phase::Planned;
        self.problem = Some(problem);
    }

    /// Whether a command of this page runs.
    pub fn busy(&self) -> bool {
        matches!(
            self.phase,
            Phase::Planning | Phase::Generating | Phase::Stopping
        )
    }

    /// The blocks: one per world of the plan, or one for every world with terrain, with what the last run did.
    pub fn blocks(&self) -> Vec<Block> {
        let mut worlds: Vec<Option<String>> = match &self.plan {
            Some(p) if !p.worlds.is_empty() => p.worlds.iter().cloned().map(Some).collect(),
            Some(_) => vec![None],
            None => Vec::new(),
        };
        // A run names the worlds it covered: they replace the open row.
        if let Some(o) = &self.outcome {
            let named: Vec<&String> = o
                .worlds
                .iter()
                .chain(o.written.iter().map(|w| &w.world))
                .collect();
            if matches!(worlds.as_slice(), [None]) && !named.is_empty() {
                worlds.clear();
            }
            for w in named {
                if !worlds.iter().any(|x| x.as_ref() == Some(w)) {
                    worlds.push(Some(w.clone()));
                }
            }
        }
        let status = if matches!(self.phase, Phase::Generating | Phase::Stopping) {
            BlockStatus::Building
        } else {
            match &self.outcome {
                None => BlockStatus::NotGenerated,
                // A stopped run leaves the Instance as it was: nothing is known to be wrong.
                Some(o) if o.dry_run || o.status == Status::Cancelled => BlockStatus::NotGenerated,
                Some(o) if o.status != Status::Success => BlockStatus::Failed,
                Some(o) if o.up_to_date => BlockStatus::UpToDate,
                Some(_) => BlockStatus::Built,
            }
        };
        worlds
            .into_iter()
            .map(|world| {
                let (files, bytes) = self.outcome.as_ref().map_or((0, 0), |o| {
                    o.written
                        .iter()
                        .filter(|w| Some(&w.world) == world.as_ref())
                        .fold((0, 0), |(f, b), w| (f + w.files, b + w.bytes))
                });
                Block {
                    world,
                    status,
                    files,
                    bytes,
                }
            })
            .collect()
    }

    /// How many blocks are not known to be current.
    pub fn not_current(&self) -> usize {
        self.blocks()
            .iter()
            .filter(|b| !matches!(b.status, BlockStatus::Built | BlockStatus::UpToDate))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../contracts/fixtures/raul-events")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn plan(worlds: &[&str]) -> Plan {
        Plan {
            plugins: 3,
            worlds: worlds.iter().map(|w| (*w).to_owned()).collect(),
            data_folder: "/i/.raul/staging/Default/Data".into(),
            output_dir: "/i/.raul/lod/out".into(),
            terrain: true,
            objects: true,
            trees: false,
        }
    }

    #[test]
    fn the_published_request_reads_as_a_plan() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../contracts/fixtures/lodwright-request/profile-settings.json");
        let p = parse_plan(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(p.plugins, 1);
        assert_eq!(p.worlds, ["SyntheticWorld"]);
        assert!(p.terrain && p.objects && !p.trees);
        assert!(parse_plan("{}").is_err());
        let other = r#"{"contract":"other","schema_version":1,"data_folder":"","plugins":[],"worlds":[],"output_dir":"","terrain":{"enabled":true},"objects":{"enabled":true},"trees":{"enabled":true}}"#;
        assert!(parse_plan(other).is_err());
    }

    #[test]
    fn the_published_streams_read_as_results() {
        let ok = read_result(&fixture("lod-success.ndjson")).unwrap();
        assert_eq!(ok.status, Status::Success);
        assert!(!ok.up_to_date);
        assert_eq!(ok.mod_name.as_deref(), Some("Generated LOD (Default)"));
        assert_eq!(ok.written[0].files, 12);
        let same = read_result(&fixture("lod-up-to-date.ndjson")).unwrap();
        assert!(same.up_to_date && same.written.is_empty());
    }

    #[test]
    fn a_stream_cut_short_or_at_odds_with_itself_is_not_a_result() {
        let full = fixture("lod-success.ndjson");
        let cut: String = full
            .lines()
            .filter(|l| !l.contains("\"finished\""))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(read_result(&cut).unwrap_err().contains("before it said"));
        let odd = full.replace(
            "\"event\": \"finished\", \"status\": \"success\"",
            "\"event\": \"finished\", \"status\": \"failed\"",
        );
        assert_ne!(odd, full);
        assert!(read_result(&odd).unwrap_err().contains("ended as"));
        let newer = full.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1);
        assert!(read_result(&newer).unwrap_err().contains("version 2"));
        assert!(read_result("").is_err());
    }

    #[test]
    fn blocks_follow_the_plan_and_the_last_run() {
        let mut s = State::default();
        assert!(s.blocks().is_empty());
        s.planning();
        assert!(s.busy());
        s.planned(plan(&[]));
        let b = s.blocks();
        assert_eq!((b.len(), b[0].world.clone()), (1, None));
        assert_eq!(s.not_current(), 1);
        s.generating();
        assert_eq!(s.blocks()[0].status, BlockStatus::Building);
        s.problem = Some(Problem {
            id: "stop",
            cause: String::new(),
            command: None,
        });
        s.stopping();
        assert_eq!(s.phase, Phase::Stopping);
        s.generated(read_result(&fixture("lod-success.ndjson")).unwrap());
        let b = s.blocks();
        assert_eq!(b.len(), 1, "the world the run named replaces the open row");
        assert_eq!(b[0].world.as_deref(), Some("SyntheticWorld"));
        assert_eq!((b[0].status, b[0].files), (BlockStatus::Built, 12));
        assert_eq!(s.not_current(), 0);
        assert!(s.problem.is_none(), "a problem from during the run is over");
    }

    #[test]
    fn a_deleted_mod_takes_the_result_of_the_run_with_it() {
        let mut s = State::default();
        s.planned(plan(&["A"]));
        let mut o = read_result(&fixture("lod-up-to-date.ndjson")).unwrap();
        o.worlds = vec!["A".into()];
        s.generated(o);
        assert!(s.blocks().iter().all(|b| b.status == BlockStatus::UpToDate));
        s.generating();
        s.mod_gone();
        assert_eq!(s.phase, Phase::Generating, "a run on its way is left alone");
        assert!(s.outcome.is_some());
        s.generated(s.outcome.clone().unwrap());
        s.mod_gone();
        assert!(s.outcome.is_none());
        assert!(
            s.blocks()
                .iter()
                .all(|b| b.status == BlockStatus::NotGenerated)
        );
    }

    #[test]
    fn a_failed_up_to_date_or_dry_run_shows_on_every_block() {
        let mut s = State::default();
        s.planned(plan(&["A", "B"]));
        let mut o = read_result(&fixture("lod-up-to-date.ndjson")).unwrap();
        o.worlds = vec!["A".into(), "B".into()];
        s.generated(o.clone());
        assert!(s.blocks().iter().all(|b| b.status == BlockStatus::UpToDate));
        o.status = Status::Failed;
        s.generated(o.clone());
        assert!(s.blocks().iter().all(|b| b.status == BlockStatus::Failed));
        o.status = Status::Cancelled;
        s.generated(o.clone());
        assert!(
            s.blocks()
                .iter()
                .all(|b| b.status == BlockStatus::NotGenerated),
            "a stopped run is not a problem"
        );
        o.dry_run = true;
        o.status = Status::Success;
        s.generated(o);
        assert_eq!(s.not_current(), 2);
        s.failed(Problem {
            id: "x",
            cause: "y".into(),
            command: None,
        });
        assert_eq!(s.phase, Phase::Planned);
    }

    #[test]
    fn progress_is_a_share_once_the_total_is_known() {
        let mut p = Progress::default();
        assert_eq!(p.percent(), None);
        p.total = Some(4);
        p.done = 1;
        assert_eq!(p.percent(), Some(25));
        p.done = 9;
        assert_eq!(p.percent(), Some(100));
        assert_eq!(
            generate_command("/i").shown(),
            "raul lod generate --instance /i"
        );
        assert!(generate_command("/i").writes && !plan_command("/i").writes);
    }
}
