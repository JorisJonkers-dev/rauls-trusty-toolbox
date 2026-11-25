// SPDX-License-Identifier: GPL-3.0-only
//! The opt-in update check against the Toolbox's GitHub Releases (ADR-0005).

use raul_core::update::{
    FETCH_TIMEOUT_SECS, MAX_LIST_BYTES, Settings, curl_args, newer_release, runner,
};
use raul_testkit::{covers, ids};

const RELEASES: &str = r#"[
  {"tag_name": "v0.3.0-rc.1", "prerelease": true, "draft": false, "html_url": "https://example.invalid/rc"},
  {"tag_name": "v0.2.0", "prerelease": false, "draft": false, "html_url": "https://example.invalid/0.2.0"},
  {"tag_name": "v0.4.0", "prerelease": false, "draft": true, "html_url": "https://example.invalid/draft"},
  {"tag_name": "v0.1.0", "prerelease": false, "draft": false, "html_url": "https://example.invalid/0.1.0"}
]"#;

#[test]
fn a_newer_release_is_offered() {
    covers(&[ids::update::check::NEWER_OFFERED]);
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
    covers(&[ids::update::check::NO_DRAFTS]);
    let r = newer_release("0.2.0", RELEASES).unwrap();
    assert!(r.is_none(), "{r:?}");
}

#[test]
fn update_checks_are_off_until_the_person_turns_them_on() {
    covers(&[ids::update::check::OFF_BY_DEFAULT]);
    let dir = nvmod_testkit::TempDir::new("raul-settings-missing");
    assert!(!Settings::load(&dir.path().join("settings.toml")).check_for_updates);
}

#[test]
fn a_saved_setting_is_read_back() {
    let dir = nvmod_testkit::TempDir::new("raul-settings");
    let path = dir.path().join("raul/settings.toml");
    Settings {
        check_for_updates: true,
        ..Settings::default()
    }
    .save(&path)
    .unwrap();
    assert!(Settings::load(&path).check_for_updates);
}

#[test]
fn saving_the_update_setting_keeps_the_other_settings() {
    let dir = nvmod_testkit::TempDir::new("raul-settings-keep");
    let path = dir.path().join("settings.toml");
    std::fs::write(&path, "theme = \"bleach\"\ncheck_for_updates = false\n").unwrap();
    Settings {
        check_for_updates: true,
        ..Settings::default()
    }
    .save(&path)
    .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("theme = \"bleach\""), "{text}");
    assert!(Settings::load(&path).check_for_updates);
}

#[test]
fn a_settings_file_that_does_not_parse_is_not_overwritten() {
    let dir = nvmod_testkit::TempDir::new("raul-settings-bad");
    let path = dir.path().join("settings.toml");
    std::fs::write(&path, "check_for_updates = [oops").unwrap();
    assert!(
        Settings {
            check_for_updates: true,
            ..Settings::default()
        }
        .save(&path)
        .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "check_for_updates = [oops"
    );
}

#[test]
fn a_release_is_newer_than_its_own_pre_releases() {
    let json = r#"[{"tag_name": "v0.2.0", "prerelease": false, "draft": false, "html_url": "u"}]"#;
    for running in [
        "0.1.0-dev.7",
        "0.2.0-dev.7",
        "0.2.0-rc.1",
        "0.2.0-rc.10",
        "0.0.0-dev.123",
    ] {
        assert_eq!(
            newer_release(running, json).unwrap().unwrap().version,
            "0.2.0",
            "{running}"
        );
    }
    assert_eq!(newer_release("0.2.1-dev.1", json).unwrap(), None);
}

#[test]
fn a_tag_with_a_pre_release_part_is_never_offered_even_when_not_marked() {
    let json = r#"[
      {"tag_name": "v0.3.0-rc.2", "prerelease": false, "draft": false, "html_url": "rc"},
      {"tag_name": "v0.2.0", "prerelease": false, "draft": false, "html_url": "0.2.0"}
    ]"#;
    assert_eq!(
        newer_release("0.1.0", json).unwrap().unwrap().version,
        "0.2.0"
    );
    assert_eq!(newer_release("0.3.0-rc.1", json).unwrap(), None);
}

#[test]
fn build_metadata_does_not_make_a_release_newer() {
    let json =
        r#"[{"tag_name": "v0.2.0+linux.9", "prerelease": false, "draft": false, "html_url": "u"}]"#;
    assert_eq!(newer_release("0.2.0", json).unwrap(), None);
    assert_eq!(newer_release("0.2.0+build.1", json).unwrap(), None);
    assert_eq!(
        newer_release("0.1.0+build.1", json)
            .unwrap()
            .unwrap()
            .version,
        "0.2.0+linux.9"
    );
}

#[test]
fn a_tag_that_is_not_semver_is_skipped() {
    let json = r#"[
      {"tag_name": "v0.9", "prerelease": false, "draft": false, "html_url": "short"},
      {"tag_name": "nightly", "prerelease": false, "draft": false, "html_url": "name"},
      {"tag_name": "v0.2.0", "prerelease": false, "draft": false, "html_url": "0.2.0"}
    ]"#;
    assert_eq!(
        newer_release("0.1.0", json).unwrap().unwrap().version,
        "0.2.0"
    );
}

#[test]
fn a_running_version_that_is_not_semver_is_an_error() {
    assert!(newer_release("0.2", RELEASES).is_err());
    assert!(newer_release("0.2.0.1", RELEASES).is_err());
}

#[test]
fn curl_is_asked_for_https_only_without_the_persons_curlrc() {
    covers(&[ids::update::check::HTTPS_ONLY]);
    let args = curl_args("https://example.invalid/releases");
    assert_eq!(args[0], "--disable");
    let joined = args.join(" ");
    assert!(
        joined.contains("--proto =https --proto-redir =https"),
        "{joined}"
    );
    assert!(joined.contains("--max-time 30"), "{joined}");
    assert_eq!(args.last().unwrap(), "https://example.invalid/releases");
}

#[test]
fn the_update_runner_has_limits() {
    let r = runner();
    assert_eq!(r.max_output, MAX_LIST_BYTES);
    assert!(r.timeout > std::time::Duration::from_secs(FETCH_TIMEOUT_SECS));
}

#[cfg(unix)]
mod through_the_runner {
    use std::time::Duration;

    use raul_core::update::{RELEASES_URL, check, runner};
    use raul_runner::Runner;

    use super::RELEASES;

    fn fake_curl(dir: &nvmod_testkit::TempDir, body: &str) -> std::path::PathBuf {
        nvmod_testkit::write_script(dir.path(), "curl", body)
    }

    #[test]
    fn the_check_runs_curl_with_an_argument_vector_and_reads_its_answer() {
        let dir = nvmod_testkit::TempDir::new("raul-update-curl");
        let args = dir.path().join("args");
        let curl = fake_curl(
            &dir,
            &format!(
                "printf '%s\\n' \"$@\" > '{}'\ncat <<'EOF'\n{RELEASES}\nEOF",
                args.display()
            ),
        );
        let newer = check(&runner(), &curl, "0.1.0").unwrap().unwrap();
        assert_eq!(newer.version, "0.2.0");
        let seen = std::fs::read_to_string(&args).unwrap();
        assert_eq!(seen.lines().next(), Some("--disable"), "{seen}");
        assert_eq!(seen.lines().last(), Some(RELEASES_URL), "{seen}");
    }

    #[test]
    fn curl_sees_a_scrubbed_environment() {
        let dir = nvmod_testkit::TempDir::new("raul-update-env");
        let env = dir.path().join("env");
        let curl = fake_curl(&dir, &format!("env > '{}'\necho '[]'", env.display()));
        assert_eq!(check(&runner(), &curl, "0.1.0").unwrap(), None);
        let text = std::fs::read_to_string(&env).unwrap();
        let names: Vec<&str> = text.lines().filter_map(|l| l.split('=').next()).collect();
        assert!(names.contains(&"PATH"), "{text}");
        assert!(text.contains("LC_ALL=C"), "{text}");
        for private in ["HOME", "USER", "LOGNAME", "XDG_CONFIG_HOME"] {
            assert!(!names.contains(&private), "{private} reached curl: {text}");
        }
    }

    #[test]
    fn a_failing_curl_is_reported() {
        let dir = nvmod_testkit::TempDir::new("raul-update-fail");
        let curl = fake_curl(
            &dir,
            "echo 'curl: (22) The requested URL returned error: 403' >&2\nexit 22",
        );
        let e = check(&runner(), &curl, "0.1.0").unwrap_err();
        assert!(e.contains("exit code 22") && e.contains("403"), "{e}");
    }

    #[test]
    fn a_curl_that_hangs_is_stopped_at_the_timeout() {
        let dir = nvmod_testkit::TempDir::new("raul-update-hang");
        let curl = fake_curl(&dir, "exec sleep 30");
        let quick = Runner {
            timeout: Duration::from_millis(300),
            ..runner()
        };
        let e = check(&quick, &curl, "0.1.0").unwrap_err();
        assert!(e.contains("ran longer than"), "{e}");
    }

    #[test]
    fn a_curl_that_writes_too_much_is_stopped_at_the_cap() {
        let dir = nvmod_testkit::TempDir::new("raul-update-cap");
        let curl = fake_curl(
            &dir,
            "while :; do echo xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx; done",
        );
        let small = Runner {
            max_output: 1024,
            ..runner()
        };
        let e = check(&small, &curl, "0.1.0").unwrap_err();
        assert!(e.contains("cap"), "{e}");
    }

    #[test]
    fn a_missing_curl_is_reported() {
        let dir = nvmod_testkit::TempDir::new("raul-update-missing");
        let e = check(&runner(), &dir.path().join("curl"), "0.1.0").unwrap_err();
        assert!(e.contains("cannot start"), "{e}");
    }
}

#[test]
fn the_lod_acknowledgement_is_kept_beside_the_update_setting() {
    let dir = nvmod_testkit::TempDir::new("raul-settings-lod");
    let path = dir.path().join("settings.toml");
    assert!(!Settings::load(&path).lod_advanced_understood);
    Settings {
        lod_advanced_understood: true,
        ..Settings::default()
    }
    .save(&path)
    .unwrap();
    let mut s = Settings::load(&path);
    assert!(s.lod_advanced_understood);
    s.check_for_updates = true;
    s.save(&path).unwrap();
    let back = Settings::load(&path);
    assert!(back.lod_advanced_understood && back.check_for_updates);
}
