// SPDX-License-Identifier: GPL-3.0-only
//! The LOD Mod's identity: its name, one per Profile, and the marker in its `meta.ini` that says it is generated LOD
//! and what built it. The marker is an MO2-compatible section of `meta.ini`, so Mod Organizer 2 and other tools keep
//! it, and every other key of the file is left as it was. A Mod is generated LOD when it carries the marker, or when
//! it has the name every Profile shared before the name held the Profile ([`LEGACY_MOD_NAME`]).

use std::path::Path;

use nvmod_formats::metaini::{MetaIni, quote_value};
use nvmod_instance::find_ci;

/// The name the LOD Mod had before it was one Mod per Profile. A Mod of this name is generated LOD; the next
/// generate of a Profile that lists it moves its place and state to that Profile's own LOD Mod.
pub const LEGACY_MOD_NAME: &str = "LOD (lodwright)";

/// The `meta.ini` section of the marker.
pub const SECTION: &str = "raulGeneratedLod";

/// The `meta.ini` file name in a Mod folder.
pub const META_FILE: &str = "meta.ini";

/// The LOD Mod of `profile`: one Mod for every world of that Profile.
#[must_use]
pub fn mod_name(profile: &str) -> String {
    format!("Generated LOD ({profile})")
}

/// What the marker says about a generated LOD Mod.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    /// The Profile the LOD was generated for.
    pub profile: String,
    /// lodwright's digest of the request the LOD was built from.
    pub request_digest: String,
    /// The lodwright version that built it.
    pub lodwright_version: String,
}

const GENERATED: &str = "generated";
const PROFILE: &str = "profile";
const REQUEST_DIGEST: &str = "requestDigest";
const LODWRIGHT_VERSION: &str = "lodwrightVersion";

impl Marker {
    /// The marker in `meta`, or `None` when it has none.
    #[must_use]
    pub fn read(meta: &MetaIni) -> Option<Marker> {
        if !meta.get(SECTION, GENERATED)?.eq_ignore_ascii_case("true") {
            return None;
        }
        let field = |key| meta.get_string(SECTION, key).unwrap_or_default();
        // A marker without its Profile is not one this command wrote.
        let profile = field(PROFILE);
        if profile.is_empty() {
            return None;
        }
        Some(Marker {
            profile,
            request_digest: field(REQUEST_DIGEST),
            lodwright_version: field(LODWRIGHT_VERSION),
        })
    }

    /// Set the marker in `meta`; every other line stays as it was.
    pub fn write(&self, meta: &mut MetaIni) {
        meta.set(SECTION, GENERATED, "true");
        meta.set(SECTION, PROFILE, &quote_value(&self.profile));
        meta.set(SECTION, REQUEST_DIGEST, &quote_value(&self.request_digest));
        meta.set(
            SECTION,
            LODWRIGHT_VERSION,
            &quote_value(&self.lodwright_version),
        );
    }
}

/// The marker of the Mod folder `mod_dir`, or `None` when its `meta.ini` is missing, unreadable or has none.
#[must_use]
pub fn read_marker(mod_dir: &Path) -> Option<Marker> {
    let bytes = std::fs::read(find_ci(mod_dir, META_FILE)?).ok()?;
    Marker::read(&MetaIni::parse(&bytes))
}

/// Whether the Mod `name` in the folder `mods` is generated LOD: it carries the marker, or it has the old shared
/// name.
#[must_use]
pub fn is_generated(mods: &Path, name: &str) -> bool {
    name.eq_ignore_ascii_case(LEGACY_MOD_NAME)
        || find_ci(mods, name).is_some_and(|dir| read_marker(&dir).is_some())
}

/// The `meta.ini` of a newly generated LOD Mod: `previous` (the replaced Mod's file, every key kept, such as the
/// category the user gave it) with the marker set. A Mod made for the first time (no `previous`) also gets a note
/// saying where it comes from; a replaced Mod keeps its notes as they are, also when the user removed them.
#[must_use]
pub fn meta_ini(previous: Option<&[u8]>, marker: &Marker) -> Vec<u8> {
    let mut meta = MetaIni::parse(previous.unwrap_or_default());
    marker.write(&mut meta);
    if previous.is_none() {
        let note = format!(
            "Generated LOD of the Profile {}. Generate it again on the LOD page; files changed here are replaced.",
            marker.profile
        );
        meta.set("General", "notes", &quote_value(&note));
    }
    meta.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    fn marker(profile: &str) -> Marker {
        Marker {
            profile: profile.to_owned(),
            request_digest: "sha256:00ff".to_owned(),
            lodwright_version: "0.3.0".to_owned(),
        }
    }

    #[test]
    fn the_name_holds_the_profile() {
        assert_eq!(mod_name("Default"), "Generated LOD (Default)");
        assert_ne!(mod_name("A"), mod_name("B"));
    }

    #[test]
    fn the_marker_is_written_and_read_back_and_other_keys_stay() {
        let user = b"[General]\r\ngameName=newvegas\r\ncategory=\"3,\"\r\nnotes=mine\r\n\r\n[Custom]\r\nkey=1\r\n";
        // A comma makes Qt read a list, so the Profile is written quoted.
        let bytes = meta_ini(Some(user), &marker("Main, second"));
        let text = String::from_utf8(bytes.clone()).unwrap();
        for kept in [
            "category=\"3,\"",
            "notes=mine",
            "[Custom]\r\nkey=1",
            "gameName=newvegas",
        ] {
            assert!(text.contains(kept), "{kept} in {text}");
        }
        assert!(text.contains("\r\n[raulGeneratedLod]\r\n"), "{text}");
        let read = Marker::read(&MetaIni::parse(&bytes)).unwrap();
        assert_eq!(read.request_digest, "sha256:00ff");
        assert_eq!(read.lodwright_version, "0.3.0");
        assert_eq!(read.profile, "Main, second");
        assert!(text.contains("profile=\"Main, second\""), "{text}");
    }

    #[test]
    fn a_new_meta_ini_gets_a_note_and_a_second_write_changes_only_the_marker() {
        let first = meta_ini(None, &marker("Default"));
        let text = String::from_utf8(first.clone()).unwrap();
        assert!(text.contains("notes="), "{text}");
        assert!(text.contains("Default"), "{text}");
        let mut next = marker("Default");
        next.request_digest = "sha256:abcd".to_owned();
        let second = meta_ini(Some(&first), &next);
        assert_eq!(
            Marker::read(&MetaIni::parse(&second))
                .unwrap()
                .request_digest,
            "sha256:abcd"
        );
        assert_eq!(
            String::from_utf8(second).unwrap().matches("notes=").count(),
            1,
            "the note is not repeated"
        );
    }

    #[test]
    fn a_replaced_meta_ini_keeps_every_user_line_byte_for_byte_and_gets_no_note_back() {
        let user = b"[General]\r\ngameName=newvegas\r\ncategory=\"3,\"\r\n\r\n[Custom]\r\nkey = spaced value\r\n";
        let text = String::from_utf8(meta_ini(Some(user), &marker("Default"))).unwrap();
        assert!(
            !text.contains("notes="),
            "a note the user removed stays removed: {text}"
        );
        let user_part = String::from_utf8(user.to_vec()).unwrap();
        for line in user_part.lines().filter(|l| !l.is_empty()) {
            assert!(text.contains(line), "{line:?} changed in {text}");
        }
    }

    #[test]
    fn a_marker_without_its_profile_is_not_one() {
        let meta =
            MetaIni::parse(b"[raulGeneratedLod]\r\ngenerated=true\r\nrequestDigest=sha256:00\r\n");
        assert_eq!(Marker::read(&meta), None);
    }

    #[test]
    fn a_mod_is_generated_by_its_marker_or_the_old_name() {
        let dir = TempDir::new("raul-lod-marker");
        let mods = dir.path();
        write_file(
            mods,
            "Generated LOD (Default)/meta.ini",
            &meta_ini(None, &marker("Default")),
        );
        write_file(
            mods,
            "Generated LOD (Other)/meta.ini",
            b"[General]\ngameName=newvegas\n",
        );
        write_file(mods, "Plain/readme.txt", b"x");
        write_file(
            mods,
            "LOD (lodwright)/meta.ini",
            b"[General]\ngameName=newvegas\n",
        );
        assert!(is_generated(mods, "Generated LOD (Default)"));
        assert!(
            !is_generated(mods, "Generated LOD (Other)"),
            "the name alone is not enough"
        );
        assert!(!is_generated(mods, "Plain"));
        assert!(!is_generated(mods, "Missing"));
        assert!(is_generated(mods, "LOD (lodwright)"));
        assert!(is_generated(mods, "lod (LODWRIGHT)"));
        assert_eq!(
            read_marker(&mods.join("Generated LOD (Default)")).map(|m| m.profile),
            Some("Default".to_owned())
        );
        let off = b"[raulGeneratedLod]\ngenerated=false\nprofile=Default\n";
        write_file(mods, "Off/meta.ini", off);
        assert!(!is_generated(mods, "Off"));
    }
}
