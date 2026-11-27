// SPDX-License-Identifier: GPL-3.0-only
//! Exporting a Mod list from a synthetic MO2-style Instance: made-up names and ids, no mod or game data.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use raul_modlist::{ExportError, ExportOptions, ModList, Source, export_modlist};
use raul_testkit::{covers, ids};

struct Dir(PathBuf);

impl Dir {
    fn new() -> Dir {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "raul-modlist-export-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Dir(p)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// Highest priority first, as MO2 writes it; separators follow their members.
fn instance() -> Dir {
    let d = Dir::new();
    let r = &d.0;
    write(
        r,
        "profiles/Test/modlist.txt",
        "# This file was automatically generated\r\n+Sky Glow\r\n-Old Rain\r\n+Manual Thing\r\n+Lights_separator\r\n+Bare Fix\r\n+Fixes_separator\r\n",
    );
    write(
        r,
        "profiles/Test/plugins.txt",
        "# generated\r\n*Sky Glow.esp\r\n*Bare Fix.esm\r\nOld Rain.esp\r\n",
    );
    write(
        r,
        "mods/Sky Glow/meta.ini",
        "[General]\ngameName=newvegas\nmodid=4242\nversion=d2026.3.1.0\nfileid=0\ninstallationFile=C:\\dl\\Sky Glow-4242-1-2-3-1700000000.7z\n",
    );
    write(r, "mods/Sky Glow/Sky Glow.esp", "");
    write(r, "mods/Sky Glow/Unlisted.esp", "");
    write(
        r,
        "mods/Old Rain/meta.ini",
        "[General]\nmodid=77\nversion=2.0\nfileid=9001\ninstallationFile=Old Rain-77-2-0-1600000000.zip\n",
    );
    write(r, "mods/Old Rain/Old Rain.esp", "");
    write(
        r,
        "mods/Manual Thing/meta.ini",
        "[General]\nmodid=0\ninstallationFile=thing.7z\n",
    );
    write(r, "mods/Bare Fix/meta.ini", "[General]\n");
    write(r, "mods/Bare Fix/Bare Fix.esm", "");
    d
}

fn opts() -> ExportOptions {
    ExportOptions::default()
}

#[test]
fn a_profile_is_exported_in_priority_order_and_round_trips() {
    covers(&[ids::mods::modlist::EXPORT_ROUNDTRIP]);
    let d = instance();
    let list = export_modlist(&d.0, &opts()).unwrap();
    let names: Vec<&str> = list.mods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["Bare Fix", "Manual Thing", "Old Rain", "Sky Glow"]);
    assert_eq!(list.id, "test");
    let groups: Vec<&str> = list.groups.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(groups, ["fixes", "lights"]);
    assert_eq!(list.mods[0].group.as_deref(), Some("fixes"));
    assert_eq!(list.mods[3].group.as_deref(), Some("lights"));

    let text = list.to_toml().unwrap();
    assert_eq!(ModList::parse(&text).unwrap(), list);
}

#[test]
fn nexus_ids_versions_archive_names_and_plugins_are_read() {
    covers(&[ids::mods::modlist::EXPORT_ROUNDTRIP]);
    let d = instance();
    let list = export_modlist(&d.0, &opts()).unwrap();
    let sky = list.mods.iter().find(|m| m.name == "Sky Glow").unwrap();
    // the MO2 placeholder version is replaced by the one in the archive name
    assert_eq!(sky.version.as_deref(), Some("1.2.3"));
    assert_eq!(sky.plugins, ["Sky Glow.esp"]);
    assert!(sky.enabled);
    assert_eq!(
        sky.source,
        Source::Nexus {
            game: "newvegas".into(),
            mod_id: 4242,
            file: None,
            file_name: Some("Sky Glow-4242-1-2-3-1700000000.7z".into()),
            size: None,
            md5: None,
            uploaded: Some(1_700_000_000),
        }
    );
    let rain = list.mods.iter().find(|m| m.name == "Old Rain").unwrap();
    assert!(!rain.enabled);
    assert!(rain.why_off.as_deref().unwrap().contains("Test"));
    assert_eq!(rain.version.as_deref(), Some("2.0"));
    assert!(matches!(
        rain.source,
        Source::Nexus {
            file: Some(9001),
            ..
        }
    ));
}

#[test]
fn a_mod_without_a_nexus_id_is_manual_with_a_note() {
    covers(&[ids::mods::modlist::EXPORT_MANUAL_WITHOUT_NEXUS_ID]);
    let d = instance();
    let list = export_modlist(&d.0, &opts()).unwrap();
    for name in ["Manual Thing", "Bare Fix"] {
        let m = list.mods.iter().find(|m| m.name == name).unwrap();
        match &m.source {
            Source::Manual { note, .. } => assert!(note.as_deref().unwrap().contains("Nexus")),
            other => panic!("{name}: {other:?}"),
        }
    }
    let m = list.mods.iter().find(|m| m.name == "Manual Thing").unwrap();
    assert!(matches!(&m.source, Source::Manual { file_name: Some(f), .. } if f == "thing.7z"));
}

#[test]
fn a_listed_mod_with_no_folder_is_manual() {
    covers(&[ids::mods::modlist::EXPORT_MANUAL_WITHOUT_NEXUS_ID]);
    let d = instance();
    fs::remove_dir_all(d.0.join("mods/Bare Fix")).unwrap();
    let list = export_modlist(&d.0, &opts()).unwrap();
    let m = list.mods.iter().find(|m| m.name == "Bare Fix").unwrap();
    assert_eq!(m.plugins.len(), 0);
    assert!(matches!(&m.source, Source::Manual { note: Some(n), .. } if n.contains("no folder")));
    ModList::parse(&list.to_toml().unwrap()).unwrap();
}

#[test]
fn names_that_collide_as_slugs_get_distinct_ids() {
    let d = Dir::new();
    write(&d.0, "profiles/P/modlist.txt", "+A B\n+a-b\n");
    let list = export_modlist(&d.0, &opts()).unwrap();
    let ids: Vec<&str> = list.mods.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["a-b", "a-b-2"]);
    ModList::parse(&list.to_toml().unwrap()).unwrap();
}

#[test]
fn several_profiles_need_a_name_and_a_wrong_name_is_refused() {
    let d = instance();
    write(&d.0, "profiles/Other/modlist.txt", "");
    let e = export_modlist(&d.0, &opts()).unwrap_err();
    assert!(
        matches!(&e, ExportError::Profile(m) if m.contains("--profile")),
        "{e}"
    );
    let named = ExportOptions {
        profile: Some("test".into()),
        ..opts()
    };
    assert_eq!(export_modlist(&d.0, &named).unwrap().mods.len(), 4);
    let wrong = ExportOptions {
        profile: Some("Nope".into()),
        ..opts()
    };
    assert!(matches!(
        export_modlist(&d.0, &wrong),
        Err(ExportError::Profile(_))
    ));
}

#[test]
fn an_instance_without_profiles_is_an_io_error() {
    let d = Dir::new();
    assert!(matches!(
        export_modlist(&d.0, &opts()),
        Err(ExportError::Io(_))
    ));
}

#[test]
fn beautiful_written_out_as_an_instance_comes_back_with_the_same_mods_order_and_nexus_files() {
    covers(&[ids::mods::modlist::EXPORT_ROUNDTRIP]);
    let beautiful = raul_modlist::bundled("beautiful").unwrap().unwrap();
    let d = Dir::new();
    // MO2 order is highest first and a separator follows its members.
    let mut lines = vec!["# synthetic".to_string()];
    let mut open: Option<&str> = None;
    for m in beautiful.mods.iter().rev() {
        let group = m.group.as_deref();
        if group != open {
            if let Some(g) = open {
                lines.push(format!("+{}_separator", title(&beautiful, g)));
            }
            open = group;
        }
        lines.push(format!("{}{}", if m.enabled { '+' } else { '-' }, m.name));
        let mut meta = String::from("[General]\n");
        if let Source::Nexus {
            mod_id,
            file,
            file_name,
            ..
        } = &m.source
        {
            writeln!(meta, "modid={mod_id}\nfileid={}", file.unwrap_or(0)).unwrap();
            if let Some(f) = file_name {
                writeln!(meta, "installationFile={f}").unwrap();
            }
        }
        write(&d.0, &format!("mods/{}/meta.ini", m.name), &meta);
    }
    if let Some(g) = open {
        lines.push(format!("+{}_separator", title(&beautiful, g)));
    }
    write(&d.0, "profiles/Beautiful/modlist.txt", &lines.join("\n"));

    let list = export_modlist(&d.0, &opts()).unwrap();
    let back = ModList::parse(&list.to_toml().unwrap()).unwrap();
    assert_eq!(back.mods.len(), beautiful.mods.len());
    for (a, b) in beautiful.mods.iter().zip(&back.mods) {
        assert_eq!((&a.name, a.enabled), (&b.name, b.enabled));
        if let Source::Nexus { mod_id, file, .. } = &a.source {
            assert!(
                matches!(&b.source, Source::Nexus { mod_id: m, file: f, .. } if m == mod_id && f == file),
                "{}",
                a.name
            );
        }
    }
}

fn title(list: &ModList, group: &str) -> String {
    list.groups
        .iter()
        .find(|g| g.id == group)
        .map(|g| g.title.clone())
        .unwrap()
}
