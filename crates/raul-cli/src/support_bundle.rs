// SPDX-License-Identifier: GPL-3.0-only
//! `raul support-bundle [--instance <dir>] [--hide-names] [--out <file>] [--apply] [--json]`: what a bug report
//! needs, in one zip. Without `--apply` it prints the plan, every item with its size; with `--apply` it writes the
//! zip (atomically). The items are the versions and the system, the paths of `raul config`, the Instance's mod and
//! plugin lists and its Doctor report, and the last run logs. Nothing is sent anywhere, nothing reads a game file's
//! contents, and the home folder never appears: it is written as `~`. `--hide-names` also replaces every Mod and
//! plugin name with a stable hash.

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};

use raul_core::context::Context;

const USAGE_LINE: &str =
    "support-bundle takes [--instance <dir>] [--hide-names] [--out <file>] [--apply] [--json]";

/// How many of the newest run logs go in, and how much of each (the end of a long one).
const LOGS: usize = 5;
const LOG_BYTES: usize = 1024 * 1024;

/// One file of the bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Its path inside the zip.
    pub name: String,
    /// What it is, in words.
    pub what: &'static str,
    /// Its content.
    pub data: Vec<u8>,
}

#[derive(Debug, Default)]
struct Options {
    instance: Option<PathBuf>,
    hide_names: bool,
    out: Option<PathBuf>,
    apply: bool,
    json: bool,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{a} needs a value"))
        };
        match a.as_str() {
            "--hide-names" => o.hide_names = true,
            "--apply" => o.apply = true,
            "--json" => o.json = true,
            "--instance" => o.instance = Some(PathBuf::from(value()?)),
            "--out" => o.out = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(o)
}

/// What a command prints, as text: its standard output, or its standard error when it printed none.
fn captured(ctx: &Context, args: &[&str]) -> String {
    let words: Vec<String> = args.iter().map(|w| (*w).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    crate::run_in(ctx, &words, &mut out, &mut err);
    let text = if out.is_empty() { err } else { out };
    String::from_utf8_lossy(&text).into_owned()
}

/// The names under `key` (`mods`, `plugins`) of a `--json` list document.
fn names_of(document: &str, key: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(document) else {
        return Vec::new();
    };
    value
        .get(key)
        .and_then(|a| a.as_array())
        .map_or_else(Vec::new, |rows| {
            rows.iter()
                .filter_map(|r| r.get("name").and_then(|n| n.as_str()).map(str::to_owned))
                .collect()
        })
}

/// The stable stand-in for a name: `<kind>-` and the first eight hex digits of its SHA-256.
fn hidden(kind: &str, name: &str) -> String {
    format!("{kind}-{}", &raul_runner::sha256_hex(name.as_bytes())[..8])
}

/// The newest run logs of `dir`, newest first, each cut to its last [`LOG_BYTES`].
fn run_logs(dir: &Path) -> Vec<Item> {
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "ndjson"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse(f.0));
    files
        .into_iter()
        .take(LOGS)
        .filter_map(|(_, path)| {
            let bytes = std::fs::read(&path).ok()?;
            let from = bytes.len().saturating_sub(LOG_BYTES);
            Some(Item {
                name: format!("logs/{}", path.file_name()?.to_string_lossy()),
                what: "a run log (the end of it when it is long)",
                data: bytes[from..].to_vec(),
            })
        })
        .collect()
}

/// Every item of the bundle for `instance`, with the home folder written as `~` and, when `hide_names`, every Mod
/// and plugin name replaced by its hash. `home` and `logs` are where the person's home and run logs are.
#[must_use]
pub fn items(
    ctx: &Context,
    instance: Option<&Path>,
    hide_names: bool,
    home: Option<&Path>,
    logs: &Path,
) -> Vec<Item> {
    let mut items = vec![Item {
        name: "versions.txt".into(),
        what: "the Toolbox version, the system and the processor",
        data: format!(
            "raul {}\nos {}\narch {}\n",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH
        )
        .into_bytes(),
    }];
    items.push(Item {
        name: "config.txt".into(),
        what: "the folders `raul config` knows",
        data: captured(ctx, &["config", "list"]).into_bytes(),
    });
    let mut names: Vec<(String, String)> = Vec::new();
    if let Some(root) = instance {
        let i = root.to_string_lossy().into_owned();
        let mods = captured(ctx, &["mods", "list", "--instance", &i, "--json"]);
        let plugins = captured(ctx, &["plugins", "list", "--instance", &i, "--json"]);
        names.extend(
            names_of(&mods, "mods")
                .into_iter()
                .map(|n| (hidden("mod", &n), n)),
        );
        names.extend(
            names_of(&plugins, "plugins")
                .into_iter()
                .map(|n| (hidden("plugin", &n), n)),
        );
        for (name, what, data) in [
            ("mods.json", "the Mods of the Profile", mods),
            ("plugins.json", "the plugins in Load order", plugins),
            (
                "doctor.json",
                "the Doctor's report on the Instance",
                captured(ctx, &["doctor", "--instance", &i, "--json"]),
            ),
        ] {
            items.push(Item {
                name: name.into(),
                what,
                data: data.into_bytes(),
            });
        }
    }
    items.extend(run_logs(logs));
    // Longest names first, so a name that contains another is replaced whole.
    names.sort_by_key(|(_, n)| std::cmp::Reverse(n.len()));
    for item in &mut items {
        let mut text = String::from_utf8_lossy(&item.data).into_owned();
        if let Some(home) = home {
            let home = home.to_string_lossy();
            text = text.replace(home.as_ref(), "~");
            text = text.replace(&home.replace('\\', "\\\\"), "~");
        }
        if hide_names {
            for (stand_in, name) in &names {
                text = text.replace(name.as_str(), stand_in);
            }
        }
        item.data = text.into_bytes();
    }
    items
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// The bytes of a zip that stores `items` uncompressed, with the fixed time 1980-01-01 so that the same items make
/// the same zip.
///
/// # Errors
/// An item is larger than a zip without the 64-bit extension holds.
pub fn zip(items: &[Item]) -> Result<Vec<u8>, String> {
    let too_big = || "an item is too large for the bundle".to_owned();
    let mut out = Vec::new();
    let mut central = Vec::new();
    for item in items {
        let size = u32::try_from(item.data.len()).map_err(|_| too_big())?;
        let name_len = u16::try_from(item.name.len()).map_err(|_| too_big())?;
        let offset = u32::try_from(out.len()).map_err(|_| too_big())?;
        let crc = crc32(&item.data);
        // flags 0x0800: the name is UTF-8; date 1980-01-01 is 0x0021.
        let common = |buf: &mut Vec<u8>| {
            buf.extend_from_slice(&[20, 0, 0, 8, 0, 0, 0, 0, 0x21, 0]);
            buf.extend_from_slice(&crc.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            buf.extend_from_slice(&name_len.to_le_bytes());
        };
        out.extend_from_slice(&0x0403_4B50_u32.to_le_bytes());
        common(&mut out);
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(item.name.as_bytes());
        out.extend_from_slice(&item.data);
        central.extend_from_slice(&0x0201_4B50_u32.to_le_bytes());
        central.extend_from_slice(&[20, 0]);
        common(&mut central);
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(item.name.as_bytes());
    }
    let count = u16::try_from(items.len()).map_err(|_| too_big())?;
    let start = u32::try_from(out.len()).map_err(|_| too_big())?;
    let len = u32::try_from(central.len()).map_err(|_| too_big())?;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4B50_u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&start.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    Ok(out)
}

/// The home folder, as the system names it.
fn home() -> Option<PathBuf> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    var("HOME").or_else(|| var("USERPROFILE"))
}

pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let options = match parse(args) {
        Ok(o) => o,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let instance = options
        .instance
        .clone()
        .or_else(|| crate::lod::configured_instance(ctx));
    let list = items(
        ctx,
        instance.as_deref(),
        options.hide_names,
        home().as_deref(),
        &raul_drivers::run_log_dir(),
    );
    let target = options
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from("raul-support-bundle.zip"));
    let written = if options.apply {
        match write(&target, &list) {
            Ok(()) => true,
            Err(why) => {
                let _ = writeln!(err, "raul support-bundle: {why}");
                return crate::FAILED;
            }
        }
    } else {
        false
    };
    let text = if options.json {
        let rows: Vec<_> = list
            .iter()
            .map(|i| serde_json::json!({ "name": i.name, "what": i.what, "bytes": i.data.len() }))
            .collect();
        let doc = serde_json::json!({
            "hide_names": options.hide_names,
            "written": written.then(|| target.display().to_string()),
            "items": rows,
        });
        format!("{doc}\n")
    } else {
        let mut text = String::new();
        for i in &list {
            let _ = writeln!(
                text,
                "  {:<28} {:>9} bytes  {}",
                i.name,
                i.data.len(),
                i.what
            );
        }
        if written {
            let _ = writeln!(text, "raul support-bundle: wrote {}", target.display());
        } else {
            text.push_str("raul support-bundle: this is the plan; add --apply to write the zip\n");
        }
        text
    };
    crate::write_or_fail(out, err, &text)
}

/// Writes the zip to `target` through a temporary file next to it, so a half-written bundle is never left.
fn write(target: &Path, list: &[Item]) -> Result<(), String> {
    let bytes = zip(list)?;
    let temp = target.with_extension("zip.part");
    std::fs::write(&temp, bytes).map_err(|e| format!("{}: {e}", temp.display()))?;
    std::fs::rename(&temp, target).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("{}: {e}", target.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_testkit::{covers, ids};

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(|w| (*w).to_owned()).collect()
    }

    /// The names and contents of a zip this module made, read back from its central directory.
    fn read_zip(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
        let u16at = |at: usize| usize::from(u16::from_le_bytes([bytes[at], bytes[at + 1]]));
        let u32at = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as usize
        };
        let end = bytes.len() - 22;
        assert_eq!(u32at(end), 0x0605_4B50);
        let count = u16at(end + 10);
        let mut at = u32at(end + 16);
        let mut found = Vec::new();
        for _ in 0..count {
            assert_eq!(u32at(at), 0x0201_4B50);
            let crc = u32::try_from(u32at(at + 16)).unwrap();
            let size = u32at(at + 24);
            let name_len = u16at(at + 28);
            let local = u32at(at + 42);
            let name = String::from_utf8(bytes[at + 46..at + 46 + name_len].to_vec()).unwrap();
            let data_at = local + 30 + u16at(local + 26) + u16at(local + 28);
            let data = bytes[data_at..data_at + size].to_vec();
            assert_eq!(crc32(&data), crc, "the checksum of {name}");
            found.push((name, data));
            at += 46 + name_len;
        }
        found
    }

    #[test]
    fn the_zip_holds_exactly_the_items_and_is_the_same_each_time() {
        covers(&[ids::cli::commands::SUPPORT_BUNDLE]);
        let items = vec![
            Item {
                name: "a.txt".into(),
                what: "a",
                data: b"hello".to_vec(),
            },
            Item {
                name: "logs/b.ndjson".into(),
                what: "b",
                data: Vec::new(),
            },
        ];
        let bytes = zip(&items).unwrap();
        assert_eq!(bytes, zip(&items).unwrap());
        assert_eq!(
            read_zip(&bytes),
            vec![
                ("a.txt".to_owned(), b"hello".to_vec()),
                ("logs/b.ndjson".to_owned(), Vec::new())
            ]
        );
        // The CRC-32 of "123456789" is the check value of the standard.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    /// A synthetic Instance with a Profile, a plugin list and a run log folder with six logs.
    fn fixture(label: &str) -> (Context, PathBuf, PathBuf, PathBuf) {
        use nvmod_testkit::BaseBuilder;
        let dir = std::env::temp_dir().join(format!("raul-bundle-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let base = dir.join("home").join("base");
        BaseBuilder::steam().write(&base);
        let root = dir.join("home").join("instance");
        let cancel = nvmod_api::CancelToken::new();
        nvmod_api::init::create(
            &nvmod_api::init::CreateRequest {
                instance: root.clone(),
                base,
            },
            &mut nvmod_api::NoProgress,
            &cancel,
        )
        .unwrap();
        nvmod_api::profile::create(
            &nvmod_api::profile::CreateRequest {
                instance: root.clone(),
                name: None,
            },
            &mut nvmod_api::NoProgress,
            &cancel,
        )
        .unwrap();
        let logs = dir.join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        for n in 0..6 {
            std::fs::write(
                logs.join(format!("run-{n}.ndjson")),
                format!("{{\"n\":{n}}}\n"),
            )
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        let ctx = Context::default()
            .with_config_folder(dir.join("home").join("config"))
            .without_run_log();
        (ctx, root, logs, dir)
    }

    #[test]
    fn the_plan_names_every_item_and_the_newest_five_run_logs() {
        let (ctx, root, logs, dir) = fixture("plan");
        let list = items(&ctx, Some(&root), false, None, &logs);
        let names: Vec<&str> = list.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "versions.txt",
                "config.txt",
                "mods.json",
                "plugins.json",
                "doctor.json",
                "logs/run-5.ndjson",
                "logs/run-4.ndjson",
                "logs/run-3.ndjson",
                "logs/run-2.ndjson",
                "logs/run-1.ndjson",
            ]
        );
        // Without an Instance the Instance's items are not planned.
        let bare = items(&ctx, None, false, None, &dir.join("no-logs"));
        assert_eq!(
            bare.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(),
            ["versions.txt", "config.txt"]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn hiding_names_leaves_no_plugin_name_and_the_home_folder_is_a_tilde() {
        covers(&[ids::cli::commands::SUPPORT_BUNDLE]);
        let (ctx, root, logs, dir) = fixture("hide");
        let home = dir.join("home");
        let shown = items(&ctx, Some(&root), false, Some(&home), &logs);
        let plain: String = shown
            .iter()
            .map(|i| String::from_utf8_lossy(&i.data).into_owned())
            .collect();
        assert!(
            plain.contains("FalloutNV.esm"),
            "the fixture names a plugin"
        );
        assert!(!plain.contains(home.to_string_lossy().as_ref()), "{plain}");
        let hidden_items = items(&ctx, Some(&root), true, Some(&home), &logs);
        let masked: String = hidden_items
            .iter()
            .map(|i| String::from_utf8_lossy(&i.data).into_owned())
            .collect();
        assert!(!masked.contains("FalloutNV"), "{masked}");
        assert!(masked.contains(&hidden("plugin", "FalloutNV.esm")));
        // The stand-in is stable.
        assert_eq!(hidden("plugin", "A.esp"), hidden("plugin", "A.esp"));
        assert_ne!(hidden("plugin", "A.esp"), hidden("plugin", "B.esp"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_command_prints_a_plan_and_with_apply_writes_a_zip_with_those_items() {
        let (ctx, root, _, dir) = fixture("run");
        let i = root.to_string_lossy().into_owned();
        let target = dir.join("bundle.zip");
        let t = target.to_string_lossy().into_owned();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(
            &ctx,
            &words(&["--instance", &i, "--out", &t, "--json"]),
            &mut out,
            &mut err,
        );
        assert_eq!(code, crate::OK, "{}", String::from_utf8_lossy(&err));
        let plan: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(plan["written"], serde_json::Value::Null);
        assert!(!target.exists(), "a plan writes nothing");
        let planned: Vec<String> = plan["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["name"].as_str().unwrap().to_owned())
            .collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(
            &ctx,
            &words(&["--instance", &i, "--out", &t, "--apply", "--hide-names"]),
            &mut out,
            &mut err,
        );
        assert_eq!(code, crate::OK, "{}", String::from_utf8_lossy(&err));
        assert!(String::from_utf8(out).unwrap().contains("wrote"));
        let inside: Vec<String> = read_zip(&std::fs::read(&target).unwrap())
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        // The plan and the zip name the same items (the run logs of the person's own state folder may differ).
        for name in [
            "versions.txt",
            "config.txt",
            "mods.json",
            "plugins.json",
            "doctor.json",
        ] {
            assert!(
                planned.iter().any(|p| p == name) && inside.iter().any(|p| p == name),
                "{name}"
            );
        }
        assert!(!dir.join("bundle.zip.part").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_command_line_that_cannot_be_understood_is_a_usage_error_and_a_bad_target_fails() {
        let (ctx, _, _, dir) = fixture("usage");
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(&ctx, &words(&["--bogus"]), &mut out, &mut err),
            crate::USAGE
        );
        assert_eq!(
            run(&ctx, &words(&["--out"]), &mut out, &mut err),
            crate::USAGE
        );
        let missing = dir
            .join("no-folder")
            .join("b.zip")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            run(
                &ctx,
                &words(&["--out", &missing, "--apply"]),
                &mut out,
                &mut err
            ),
            crate::FAILED
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
