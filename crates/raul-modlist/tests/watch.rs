// SPDX-License-Identifier: GPL-3.0-only
//! Recognising downloaded archives: names, sizes, md5, the poller and the folder list. Synthetic files only.

use std::collections::BTreeSet;
use std::path::PathBuf;

use raul_modlist::ModList;
use raul_modlist::watch::{
    By, Candidate, Entry, Md5Lookup, NexusRef, NoLookup, Poller, Verdict, Wanted, classify,
    download_dirs, is_archive, md5_file, normalize, wanted,
};

fn w(id: &str, file: Option<&str>, size: Option<u64>, md5: Option<&str>) -> Wanted {
    Wanted {
        id: id.into(),
        name: id.into(),
        file_name: file.map(Into::into),
        size,
        md5: md5.map(Into::into),
        nexus: None,
    }
}

fn cand(name: &str, size: u64) -> Candidate {
    Candidate {
        name: name.into(),
        size,
    }
}

fn run(c: &Candidate, ws: &[Wanted], hash: Option<&str>) -> Verdict {
    classify(c, ws, &mut || hash.map(Into::into), &NoLookup)
}

fn matched(id: &str, by: By) -> Verdict {
    Verdict::Match { id: id.into(), by }
}

#[test]
fn a_file_name_wins_and_ignores_case_and_browser_copies() {
    let ws = [w("a", Some("Alpha-1-0.7z"), None, None)];
    assert_eq!(
        run(&cand("alpha-1-0.7Z", 5), &ws, None),
        matched("a", By::Name)
    );
    assert_eq!(
        run(&cand("Alpha-1-0 (2).7z", 5), &ws, None),
        matched("a", By::Name)
    );
    assert_eq!(normalize("x (12).zip"), "x.zip");
    assert_eq!(normalize("x (a).zip"), "x (a).zip");
}

#[test]
fn a_name_match_with_the_wrong_size_or_md5_is_rejected() {
    let ws = [w("a", Some("A.7z"), Some(10), None)];
    assert!(matches!(
        run(&cand("A.7z", 11), &ws, None),
        Verdict::Rejected { .. }
    ));
    let ws = [w("a", Some("A.7z"), Some(10), Some("aa"))];
    assert!(matches!(
        run(&cand("A.7z", 10), &ws, Some("bb")),
        Verdict::Rejected { .. }
    ));
    assert!(matches!(
        run(&cand("A.7z", 10), &ws, None),
        Verdict::Rejected { .. }
    ));
    assert_eq!(
        run(&cand("A.7z", 10), &ws, Some("aa")),
        matched("a", By::Name)
    );
}

#[test]
fn a_renamed_file_is_found_by_a_size_only_one_entry_has() {
    let ws = [
        w("a", Some("A.7z"), Some(10), None),
        w("b", Some("B.7z"), Some(20), None),
    ];
    assert_eq!(
        run(&cand("download.7z", 20), &ws, None),
        matched("b", By::Size)
    );
}

#[test]
fn a_size_shared_by_two_entries_is_not_guessed() {
    let ws = [
        w("a", Some("A.7z"), Some(10), None),
        w("b", Some("B.7z"), Some(10), None),
    ];
    assert_eq!(run(&cand("x.7z", 10), &ws, None), Verdict::Unmatched);
}

#[test]
fn a_size_match_still_has_to_agree_on_md5() {
    let ws = [w("a", Some("A.7z"), Some(10), Some("aa"))];
    assert!(matches!(
        run(&cand("x.7z", 10), &ws, Some("bb")),
        Verdict::Rejected { .. }
    ));
}

#[test]
fn md5_names_a_file_whose_name_and_size_are_unknown() {
    let ws = [
        w("a", None, None, Some("aa")),
        w("b", None, None, Some("bb")),
    ];
    assert_eq!(
        run(&cand("x.zip", 3), &ws, Some("bb")),
        matched("b", By::Md5)
    );
    assert_eq!(run(&cand("x.zip", 3), &ws, Some("cc")), Verdict::Unmatched);
}

#[test]
fn the_md5_is_read_only_when_a_decision_needs_it() {
    let mut reads = 0;
    let ws = [w("a", Some("A.7z"), Some(10), None)];
    let v = classify(
        &cand("A.7z", 10),
        &ws,
        &mut || {
            reads += 1;
            None
        },
        &NoLookup,
    );
    assert_eq!(v, matched("a", By::Name));
    let v = classify(
        &cand("zzz.7z", 99),
        &ws,
        &mut || {
            reads += 1;
            None
        },
        &NoLookup,
    );
    assert_eq!(v, Verdict::Unmatched);
    assert_eq!(reads, 0);
}

struct Fake;
impl Md5Lookup for Fake {
    fn by_md5(&self, md5: &str) -> Option<NexusRef> {
        (md5 == "ee").then(|| NexusRef {
            game: "newvegas".into(),
            mod_id: 7,
            file_id: 70,
        })
    }
}

#[test]
fn the_nexus_lookup_seam_names_a_file_by_its_mod() {
    let mut e = w("a", None, None, None);
    e.nexus = Some(("newvegas".into(), 7, None));
    let ws = [e];
    let v = classify(&cand("x.7z", 1), &ws, &mut || Some("ee".into()), &Fake);
    assert_eq!(v, matched("a", By::Lookup));
    let v = classify(&cand("x.7z", 1), &ws, &mut || Some("ff".into()), &Fake);
    assert_eq!(v, Verdict::Unmatched);
}

#[test]
fn md5_of_a_file_is_the_known_digest() {
    let dir = std::env::temp_dir().join(format!("raul-watch-md5-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("f.bin");
    std::fs::write(&p, b"abc").unwrap();
    assert_eq!(md5_file(&p).unwrap(), "900150983cd24fb0d6963f7d28e17f72");
    std::fs::remove_dir_all(&dir).unwrap();
}

fn entry(name: &str, size: u64) -> Entry {
    Entry {
        path: PathBuf::from("/d").join(name),
        size,
    }
}

#[test]
fn archives_are_told_from_partial_and_hidden_files() {
    assert!(is_archive("A.7z") && is_archive("a.ZIP") && is_archive("a.rar"));
    for no in [
        "a.7z.part",
        "a.crdownload",
        "a.7z.opdownload",
        ".a.7z",
        "notes.txt",
        "a.7z.tmp",
    ] {
        assert!(!is_archive(no), "{no}");
    }
}

#[test]
fn a_new_archive_is_ready_once_its_size_has_settled() {
    let mut p = Poller::new();
    p.baseline(&[entry("old.7z", 5)]);
    assert_eq!(
        p.poll(&[entry("old.7z", 5), entry("new.7z", 100)]),
        Vec::<Entry>::new()
    );
    assert_eq!(
        p.poll(&[entry("old.7z", 5), entry("new.7z", 250)]),
        Vec::<Entry>::new(),
        "still growing"
    );
    let ready = p.poll(&[entry("old.7z", 5), entry("new.7z", 250)]);
    assert_eq!(ready, vec![entry("new.7z", 250)]);
    assert_eq!(
        p.poll(&[entry("new.7z", 250)]),
        Vec::<Entry>::new(),
        "reported once"
    );
}

#[test]
fn empty_and_partial_files_are_never_ready() {
    let mut p = Poller::new();
    let l = [
        entry("z.7z", 0),
        entry("a.7z.part", 9),
        entry("b.crdownload", 9),
    ];
    assert_eq!(p.poll(&l), Vec::<Entry>::new());
    assert_eq!(p.poll(&l), Vec::<Entry>::new());
}

#[test]
fn a_single_scan_takes_every_archive_without_waiting() {
    let mut p = Poller::new();
    let got = p.take_all(&[entry("a.7z", 1), entry("n.txt", 1), entry("b.zip", 2)]);
    assert_eq!(got.len(), 2);
    assert_eq!(p.take_all(&[entry("a.7z", 1)]), Vec::<Entry>::new());
}

#[test]
fn download_folders_follow_the_xdg_file_then_home_then_the_instance() {
    let env = |k: &str| (k == "HOME").then(|| "/home/u".to_owned());
    let dirs = download_dirs(
        &env,
        Some("# c\nXDG_DOWNLOAD_DIR=\"$HOME/Herunterladen\"\n"),
        Some(PathBuf::from("/inst/downloads")),
    );
    assert_eq!(
        dirs,
        vec![
            PathBuf::from("/home/u/Herunterladen"),
            PathBuf::from("/home/u/Downloads"),
            PathBuf::from("/inst/downloads"),
        ]
    );
    let none = download_dirs(&|_| None, None, None);
    assert_eq!(none, Vec::<PathBuf>::new());
    let dup = download_dirs(&env, Some("XDG_DOWNLOAD_DIR=\"$HOME/Downloads\""), None);
    assert_eq!(dup, vec![PathBuf::from("/home/u/Downloads")]);
}

#[test]
fn wanted_files_come_from_enabled_missing_mods_with_a_file_source() {
    let list = ModList::parse(
        r#"
schema = "raul-modlist/1"
id = "t"
name = "T"
game = "fnv"
target = "vanilla"
plugins = "loot"
[[mods]]
id = "n"
name = "N"
source = { type = "nexus", game = "newvegas", mod = 1, file = 2, file_name = "N.7z", size = 9, md5 = "abcdefabcdefabcdefabcdefabcdefab" }
[[mods]]
id = "m"
name = "M"
source = { type = "manual", file_name = "M.zip" }
[[mods]]
id = "present"
name = "P"
source = { type = "manual", file_name = "P.zip" }
[[mods]]
id = "tool"
name = "Tool"
source = { type = "generated", tool = "lodwright-rs" }
"#,
    )
    .unwrap();
    let missing: BTreeSet<String> = ["n", "m", "tool"].map(String::from).into();
    let got = wanted(&list, &missing);
    let ids: Vec<&str> = got.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, ["n", "m"]);
    assert_eq!(
        got[0].md5.as_deref(),
        Some("abcdefabcdefabcdefabcdefabcdefab"),
        "md5 is carried over"
    );
    assert_eq!(got[0].nexus, Some(("newvegas".into(), 1, Some(2))));
}
