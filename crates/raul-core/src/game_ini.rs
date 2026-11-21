// SPDX-License-Identifier: GPL-3.0-only
//! The game settings Tale of Two Wastelands needs in a Profile's `Fallout.ini` (#442).
//!
//! The engine answers with the values in the Profile's `Fallout.ini`, which hold the game's defaults: with them a new
//! game shows the box "Misconfigured INI settings detected" and plays the New Vegas opening, and the choice of
//! wasteland never appears. A Profile whose `plugins.txt` lists `TaleOfTwoWastelands.esm` gets the three values of
//! the "Custom INI" step of the Tale of Two Wastelands guide that its scripts and start quest depend on; when the
//! master leaves the Profile the values that were set go back. The keys are changed where they are (names without
//! regard to case), never added a second time, and never written to a `FalloutCustom.ini`, which the engine does not
//! read from the settings folder.

use std::fmt::Write as _;
use std::path::Path;

/// The master that makes a Profile a Tale of Two Wastelands Profile.
pub const TTW_MASTER: &str = "TaleOfTwoWastelands.esm";

/// One setting: where it is and the two values it takes.
struct Setting {
    section: &'static str,
    key: &'static str,
    /// With Tale of Two Wastelands.
    ttw: &'static str,
    /// The game's own value.
    default: &'static str,
}

const SETTINGS: [Setting; 3] = [
    // Tale of Two Wastelands' start quest, which shows the choice of wasteland; the default starts New Vegas.
    Setting {
        section: "General",
        key: "SCharGenQuest",
        ttw: "001FFFF8",
        default: "00102037",
    },
    // The one value its scripts read; 0 gives the "Misconfigured INI settings" box.
    Setting {
        section: "BackgroundLoad",
        key: "bBackgroundLoadLipFiles",
        ttw: "1",
        default: "0",
    },
    // The guide switches the New Vegas intro film off; no script checks it.
    Setting {
        section: "General",
        key: "sIntroMovie",
        ttw: "",
        default: "Fallout INTRO Vsk.bik",
    },
];

/// What [`sync`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing to do: not a Tale of Two Wastelands Profile and nothing of ours to take back, or the values are right.
    Unchanged,
    /// The file was written; one line per value that changed.
    Changed(Vec<String>),
    /// A Tale of Two Wastelands Profile whose `Fallout.ini` does not exist yet and has no default file to start from:
    /// the engine makes it at its first start and the next start sets the values.
    NoFile,
}

fn eol_of(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

/// The name of the section a line opens, if it is a header.
fn header(line: &str) -> Option<&str> {
    let t = line.trim();
    t.strip_prefix('[')?.strip_suffix(']').map(str::trim)
}

/// The value of `key` in `section`: the first such line counts, since a second one is not read.
#[must_use]
pub fn get(text: &str, section: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        if let Some(name) = header(line) {
            inside = name.eq_ignore_ascii_case(section);
        } else if inside
            && let Some((k, v)) = line.split_once('=')
            && k.trim().eq_ignore_ascii_case(key)
        {
            return Some(v.trim().to_owned());
        }
    }
    None
}

/// `text` with `key` of `section` set to `value`: the existing line is changed in place (its own spelling of the key
/// kept), the key is added at the end of its section when the section lacks it, and the section at the end of the
/// file when there is none. Line endings stay as the file has them.
#[must_use]
pub fn set(text: &str, section: &str, key: &str, value: &str) -> String {
    let eol = eol_of(text);
    let mut out = String::with_capacity(text.len() + 32);
    let mut inside = false;
    let mut done = false;
    // Where the section's last non-blank line ends in `out`, for an insert.
    let mut insert_at: Option<usize> = None;
    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\r', '\n']);
        let ending = &raw[line.len()..];
        if let Some(name) = header(line) {
            let was_inside = inside;
            inside = name.eq_ignore_ascii_case(section);
            if was_inside && !done {
                let at = insert_at.unwrap_or(out.len());
                out.insert_str(at, &format!("{key}={value}{eol}")); // insert_str, not an append
                done = true;
            }
            out.push_str(raw);
            if inside {
                insert_at = Some(out.len());
            }
            continue;
        }
        if inside
            && !done
            && let Some((k, _)) = line.split_once('=')
            && k.trim().eq_ignore_ascii_case(key)
        {
            let _ = write!(out, "{}={value}{ending}", k.trim_end());
            done = true;
            continue;
        }
        out.push_str(raw);
        if inside && !line.trim().is_empty() {
            insert_at = Some(out.len());
        }
    }
    if !done {
        if inside {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(eol);
            }
            let _ = write!(out, "{key}={value}{eol}");
        } else {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(eol);
            }
            let _ = write!(out, "[{section}]{eol}{key}={value}{eol}");
        }
    }
    out
}

/// Whether `plugins_txt` lists the Tale of Two Wastelands master (with or without the `*` of an enabled line).
#[must_use]
pub fn lists_ttw(plugins_txt: &str) -> bool {
    plugins_txt.lines().any(|l| {
        l.trim()
            .trim_start_matches('*')
            .trim()
            .eq_ignore_ascii_case(TTW_MASTER)
    })
}

/// Sets or takes back the values for the Profile whose files are `plugins_txt` and `fallout_ini`; when the file does
/// not exist and the Profile is a Tale of Two Wastelands one, it is started from `default_ini` (the game's own
/// `Fallout_default.ini`) when that exists.
///
/// With the master listed every value is set to its Tale of Two Wastelands value. Without it a value goes back to the
/// game's own only when it still holds the Tale of Two Wastelands one, so a setting the person made is kept; the intro
/// film goes back with the start quest.
///
/// # Errors
/// A file could not be read or written.
pub fn sync(
    plugins_txt: &Path,
    fallout_ini: &Path,
    default_ini: Option<&Path>,
) -> Result<Outcome, String> {
    sync_with(plugins_txt, fallout_ini, default_ini, true)
}

/// [`sync`] where `master_provided` says whether the staged `Data` folder holds the master. A Profile is a Tale of Two
/// Wastelands one only when `plugins.txt` lists the master and a Mod provides it: disabling the Tale of Two Wastelands
/// Mod leaves the master listed, and the values still go back (#442).
///
/// # Errors
/// As [`sync`].
pub fn sync_with(
    plugins_txt: &Path,
    fallout_ini: &Path,
    default_ini: Option<&Path>,
    master_provided: bool,
) -> Result<Outcome, String> {
    let plugins = std::fs::read_to_string(plugins_txt).unwrap_or_default();
    let ttw = master_provided && lists_ttw(&plugins);
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let before = if fallout_ini.is_file() {
        read(fallout_ini)?
    } else if !ttw {
        return Ok(Outcome::Unchanged);
    } else if let Some(default) = default_ini.filter(|p| p.is_file()) {
        read(default)?
    } else {
        return Ok(Outcome::NoFile);
    };
    let mut text = before.clone();
    let mut said = Vec::new();
    let quest_is_ttw = get(&text, "General", "SCharGenQuest")
        .is_some_and(|v| v.eq_ignore_ascii_case(SETTINGS[0].ttw));
    for (index, s) in SETTINGS.iter().enumerate() {
        let now = get(&text, s.section, s.key);
        let want = if ttw {
            Some(s.ttw)
        } else {
            let ours = match index {
                2 => quest_is_ttw && now.as_deref() == Some(""),
                _ => now
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case(s.ttw)),
            };
            ours.then_some(s.default)
        };
        if let Some(want) = want
            && now.as_deref().is_none_or(|v| !v.eq_ignore_ascii_case(want))
        {
            text = set(&text, s.section, s.key, want);
            said.push(format!("[{}] {}={}", s.section, s.key, want));
        }
    }
    if text == before && fallout_ini.is_file() {
        return Ok(Outcome::Unchanged);
    }
    if let Some(parent) = fallout_ini.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(fallout_ini, text).map_err(|e| format!("{}: {e}", fallout_ini.display()))?;
    Ok(Outcome::Changed(said))
}

/// [`sync`] for the run folder `<instance>/run/<profile>` that nvmod prepares: the Profile's lists are
/// `<instance>/profiles/<profile>/plugins.txt`, the settings are in the run's user data.
///
/// # Errors
/// As [`sync`].
pub fn sync_run_folder(run_dir: &Path) -> Result<Outcome, String> {
    let (Some(profile), Some(instance)) =
        (run_dir.file_name(), run_dir.parent().and_then(Path::parent))
    else {
        return Ok(Outcome::Unchanged);
    };
    let plugins = instance.join("profiles").join(profile).join("plugins.txt");
    let ini = nvmod_run::my_games(&run_dir.join("userdata")).join("Fallout.ini");
    // The run folder's `Data` is the staged tree: when it exists, the master has to be in it. A folder that is not
    // staged (yet) says nothing, so the list alone decides.
    let data = run_dir.join("Data");
    let provided = !data.is_dir() || nvmod_instance::find_ci(&data, TTW_MASTER).is_some();
    sync_with(
        &plugins,
        &ini,
        Some(&run_dir.join("Fallout_default.ini")),
        provided,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULTS: &str = "[General]\r\nsIntroMovie=Fallout INTRO Vsk.bik\r\nSCharGenQuest=00102037\r\nuGridsToLoad=5\r\n\r\n[BackgroundLoad]\r\nbBackgroundLoadLipFiles=0\r\nbUseMultiThreadedFaceGen=1\r\n\r\n[Display]\r\niSize W=1280\r\n";

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("raul-ini-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_key_is_changed_in_place_whatever_its_letter_case_and_never_twice() {
        let text = "[general]\nscharGENquest=1\nother=2\nSCharGenQuest=3\n";
        let out = set(text, "General", "SCharGenQuest", "001FFFF8");
        assert_eq!(
            out,
            "[general]\nscharGENquest=001FFFF8\nother=2\nSCharGenQuest=3\n"
        );
        assert_eq!(get(&out, "General", "SCharGenQuest").unwrap(), "001FFFF8");
    }

    #[test]
    fn a_missing_key_or_section_is_added_with_the_files_line_endings() {
        let text = "[General]\r\na=1\r\n\r\n[Display]\r\nb=2\r\n";
        let out = set(text, "General", "k", "v");
        assert_eq!(out, "[General]\r\na=1\r\nk=v\r\n\r\n[Display]\r\nb=2\r\n");
        let out = set(text, "New", "k", "v");
        assert!(out.ends_with("[New]\r\nk=v\r\n"), "{out:?}");
        // A section that ends the file without a final line ending.
        assert_eq!(set("[A]\nx=1", "A", "y", "2"), "[A]\nx=1\ny=2\n");
        assert_eq!(set("", "A", "y", "2"), "[A]\ny=2\n");
    }

    #[test]
    fn the_master_is_found_with_or_without_a_star_in_any_case() {
        assert!(lists_ttw("FalloutNV.esm\n*taleoftwowastelands.ESM\n"));
        assert!(lists_ttw("TaleOfTwoWastelands.esm\n"));
        assert!(!lists_ttw("FalloutNV.esm\nTaleOfTwoWastelands.esp\n"));
    }

    #[test]
    fn a_master_no_enabled_mod_provides_does_not_make_a_ttw_profile() {
        let dir = scratch("provided");
        let (plugins, ini, default) = (
            dir.join("plugins.txt"),
            dir.join("My Games/Fallout.ini"),
            dir.join("Fallout_default.ini"),
        );
        std::fs::write(&default, DEFAULTS).unwrap();
        std::fs::write(&plugins, "FalloutNV.esm\n*TaleOfTwoWastelands.esm\n").unwrap();
        assert!(matches!(
            sync_with(&plugins, &ini, Some(&default), true).unwrap(),
            Outcome::Changed(_)
        ));
        // The Mod is disabled: the master stays listed, nothing provides it, the values go back.
        assert!(matches!(
            sync_with(&plugins, &ini, Some(&default), false).unwrap(),
            Outcome::Changed(_)
        ));
        assert_eq!(std::fs::read_to_string(&ini).unwrap(), DEFAULTS);
    }

    #[test]
    fn the_run_folder_decides_by_the_master_in_its_staged_data() {
        let root = scratch("runfolder");
        let run = root.join("run/Default");
        std::fs::create_dir_all(run.join("Data")).unwrap();
        std::fs::create_dir_all(root.join("profiles/Default")).unwrap();
        std::fs::write(
            root.join("profiles/Default/plugins.txt"),
            "*TaleOfTwoWastelands.esm\n",
        )
        .unwrap();
        std::fs::write(run.join("Fallout_default.ini"), DEFAULTS).unwrap();
        let ini = nvmod_run::my_games(&run.join("userdata")).join("Fallout.ini");
        // Staged without the master (the Mod is disabled): a plain Profile, no file made.
        assert_eq!(sync_run_folder(&run).unwrap(), Outcome::Unchanged);
        assert!(!ini.exists());
        // The master is staged: the values are set.
        std::fs::write(run.join("Data/taleoftwowastelands.ESM"), b"x").unwrap();
        assert!(matches!(
            sync_run_folder(&run).unwrap(),
            Outcome::Changed(_)
        ));
        // Disabled again: the values go back.
        std::fs::remove_file(run.join("Data/taleoftwowastelands.ESM")).unwrap();
        assert!(matches!(
            sync_run_folder(&run).unwrap(),
            Outcome::Changed(_)
        ));
        assert_eq!(std::fs::read_to_string(&ini).unwrap(), DEFAULTS);
    }

    #[test]
    fn a_ttw_profile_gets_the_values_and_a_plain_one_keeps_the_defaults() {
        let dir = scratch("sync");
        let (plugins, ini, default) = (
            dir.join("plugins.txt"),
            dir.join("My Games/Fallout.ini"),
            dir.join("Fallout_default.ini"),
        );
        std::fs::write(&default, DEFAULTS).unwrap();

        // Plain Profile, no file: nothing is made.
        std::fs::write(&plugins, "FalloutNV.esm\n").unwrap();
        assert_eq!(
            sync(&plugins, &ini, Some(&default)).unwrap(),
            Outcome::Unchanged
        );
        assert!(!ini.exists());

        // The master is listed: the file starts from the game's defaults and gets the three values.
        std::fs::write(&plugins, "FalloutNV.esm\n*TaleOfTwoWastelands.esm\n").unwrap();
        let Outcome::Changed(said) = sync(&plugins, &ini, Some(&default)).unwrap() else {
            panic!("expected a change");
        };
        assert_eq!(said.len(), 3, "{said:?}");
        let text = std::fs::read_to_string(&ini).unwrap();
        assert_eq!(get(&text, "General", "SCharGenQuest").unwrap(), "001FFFF8");
        assert_eq!(
            get(&text, "BackgroundLoad", "bBackgroundLoadLipFiles").unwrap(),
            "1"
        );
        assert_eq!(get(&text, "General", "sIntroMovie").unwrap(), "");
        assert_eq!(
            get(&text, "Display", "iSize W").unwrap(),
            "1280",
            "the rest is kept"
        );
        assert_eq!(text.matches("SCharGenQuest").count(), 1);
        assert!(text.contains("\r\n") && !text.replace("\r\n", "").contains('\n'));
        // Again: nothing changes.
        assert_eq!(
            sync(&plugins, &ini, Some(&default)).unwrap(),
            Outcome::Unchanged
        );

        // The master leaves: the values go back.
        std::fs::write(&plugins, "FalloutNV.esm\n").unwrap();
        assert!(matches!(
            sync(&plugins, &ini, Some(&default)).unwrap(),
            Outcome::Changed(_)
        ));
        assert_eq!(std::fs::read_to_string(&ini).unwrap(), DEFAULTS);
    }

    #[test]
    fn a_setting_the_person_made_is_kept_when_the_master_leaves() {
        let dir = scratch("kept");
        let (plugins, ini) = (dir.join("plugins.txt"), dir.join("Fallout.ini"));
        std::fs::write(&plugins, "FalloutNV.esm\n").unwrap();
        std::fs::write(&ini, "[General]\nSCharGenQuest=00AAAAAA\nsIntroMovie=\n[BackgroundLoad]\nbBackgroundLoadLipFiles=1\n").unwrap();
        sync(&plugins, &ini, None).unwrap();
        let text = std::fs::read_to_string(&ini).unwrap();
        assert_eq!(get(&text, "General", "SCharGenQuest").unwrap(), "00AAAAAA");
        // The intro film stays off, since the start quest was not ours; the lip-file value was ours.
        assert_eq!(get(&text, "General", "sIntroMovie").unwrap(), "");
        assert_eq!(
            get(&text, "BackgroundLoad", "bBackgroundLoadLipFiles").unwrap(),
            "0"
        );
    }

    #[test]
    fn a_ttw_profile_without_any_file_to_start_from_says_so() {
        let dir = scratch("nofile");
        let plugins = dir.join("plugins.txt");
        std::fs::write(&plugins, "TaleOfTwoWastelands.esm\n").unwrap();
        assert_eq!(
            sync(&plugins, &dir.join("Fallout.ini"), None).unwrap(),
            Outcome::NoFile
        );
    }

    #[test]
    fn the_run_folder_finds_its_profile_and_settings() {
        let dir = scratch("run");
        let run = dir.join("run/Default");
        let profile = dir.join("profiles/Default");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::create_dir_all(&run).unwrap();
        std::fs::write(profile.join("plugins.txt"), "*TaleOfTwoWastelands.esm\n").unwrap();
        std::fs::write(run.join("Fallout_default.ini"), DEFAULTS).unwrap();
        assert!(matches!(
            sync_run_folder(&run).unwrap(),
            Outcome::Changed(_)
        ));
        let ini = nvmod_run::my_games(&run.join("userdata")).join("Fallout.ini");
        assert!(
            std::fs::read_to_string(ini)
                .unwrap()
                .contains("SCharGenQuest=001FFFF8")
        );
        // A path with no parents does nothing.
        assert_eq!(sync_run_folder(Path::new("/")).unwrap(), Outcome::Unchanged);
    }
}
