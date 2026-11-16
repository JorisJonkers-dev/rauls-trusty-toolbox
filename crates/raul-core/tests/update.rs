// SPDX-License-Identifier: GPL-3.0-only
//! The opt-in update check against the Toolbox's GitHub Releases (ADR-0005, nv-linux ADR-0010).

use raul_core::update::{Settings, newer_release};

const RELEASES: &str = r#"[
  {"tag_name": "v0.3.0-rc.1", "prerelease": true, "draft": false, "html_url": "https://example.invalid/rc"},
  {"tag_name": "v0.2.0", "prerelease": false, "draft": false, "html_url": "https://example.invalid/0.2.0"},
  {"tag_name": "v0.4.0", "prerelease": false, "draft": true, "html_url": "https://example.invalid/draft"},
  {"tag_name": "v0.1.0", "prerelease": false, "draft": false, "html_url": "https://example.invalid/0.1.0"}
]"#;

#[test]
fn a_newer_release_is_offered() {
    let r = newer_release("0.1.0", RELEASES).unwrap().unwrap();
    assert_eq!(r.version, "0.2.0");
    assert_eq!(r.url, "https://example.invalid/0.2.0");
}

#[test]
fn the_running_version_is_not_offered_again() {
    assert_eq!(newer_release("0.2.0", RELEASES).unwrap(), None);
}

#[test]
fn a_version_ten_is_newer_than_version_nine() {
    let json = r#"[{"tag_name": "v0.10.0", "prerelease": false, "draft": false, "html_url": "u"}]"#;
    assert_eq!(
        newer_release("0.9.0", json).unwrap().unwrap().version,
        "0.10.0"
    );
}

#[test]
fn drafts_and_prereleases_are_never_offered() {
    let r = newer_release("0.2.0", RELEASES).unwrap();
    assert!(r.is_none(), "{r:?}");
}

#[test]
fn update_checks_are_off_until_the_person_turns_them_on() {
    let dir = nvmod_testkit::TempDir::new("raul-settings-missing");
    assert!(!Settings::load(&dir.path().join("settings.toml")).check_for_updates);
}

#[test]
fn a_saved_setting_is_read_back() {
    let dir = nvmod_testkit::TempDir::new("raul-settings");
    let path = dir.path().join("raul/settings.toml");
    Settings {
        check_for_updates: true,
    }
    .save(&path)
    .unwrap();
    assert!(Settings::load(&path).check_for_updates);
}
