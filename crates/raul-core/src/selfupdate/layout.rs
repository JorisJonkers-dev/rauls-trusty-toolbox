// SPDX-License-Identifier: GPL-3.0-only
//! Which kind of install the running `raul` is, decided from its own path, the environment and the files beside it
//! (ADR-0020). Only the installs that the file swap can replace are updatable; the others say why, in plain words.

use std::path::{Path, PathBuf};

use super::extract::INVENTORY;

/// The system the program runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// macOS.
    Mac,
}

impl Os {
    /// The system this build runs on.
    #[must_use]
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Linux
        }
    }
}

/// Why an install cannot be updated from inside raul.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// Installed with Nix, which updates it.
    Nix,
    /// Installed by the system's package manager.
    System,
    /// macOS: updates are downloaded from the release page for now.
    Mac,
    /// The folder cannot be written by this person.
    ReadOnly,
    /// An installer (for example the Windows setup) or a development build: not a package raul can replace file by file.
    Other,
}

impl Reason {
    /// What to tell the person.
    #[must_use]
    pub fn plain(self) -> &'static str {
        match self {
            Self::Nix => "raul was installed with Nix, so Nix updates it.",
            Self::System => {
                "raul was installed by your system's package manager, so the package manager updates it."
            }
            Self::Mac => "On a Mac, download the new version from the release page.",
            Self::ReadOnly => {
                "raul cannot change the folder it is installed in. Download the new version from the release page."
            }
            Self::Other => {
                "This copy of raul was installed in a way raul cannot update itself. Download the new version from the release page."
            }
        }
    }
}

/// How this install is updated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    /// A folder holding `raul`, the packaged engine and `INSTALLED-FILES.sha256` (the tarball and the portable zip).
    Folder {
        /// The folder.
        root: PathBuf,
        /// The file name of the program in it.
        exe: String,
        /// The system, which decides the archive.
        os: Os,
    },
    /// A single `AppImage` file.
    AppImage {
        /// The file.
        file: PathBuf,
    },
    /// Not updatable from inside raul.
    NotHere(Reason),
}

impl Layout {
    /// The release file that updates this install to `version`, or `None` when it cannot be updated.
    #[must_use]
    pub fn asset(&self, version: &str) -> Option<String> {
        match self {
            Self::Folder {
                os: Os::Windows, ..
            } => Some(format!("raul-{version}-windows-x64-portable.zip")),
            Self::Folder { .. } => Some(format!("raul-{version}-linux-x64.tar.gz")),
            Self::AppImage { .. } => Some(format!("raul-{version}-linux-x64.AppImage")),
            Self::NotHere(_) => None,
        }
    }

    /// The folder the update works in (staging, the previous version, the journal), or `None` when not updatable.
    #[must_use]
    pub fn root(&self) -> Option<&Path> {
        match self {
            Self::Folder { root, .. } => Some(root),
            Self::AppImage { file } => file.parent(),
            Self::NotHere(_) => None,
        }
    }

    /// The program to start after the swap to check the new version and to restart into.
    #[must_use]
    pub fn program(&self) -> Option<PathBuf> {
        match self {
            Self::Folder { root, exe, .. } => Some(root.join(exe)),
            Self::AppImage { file } => Some(file.clone()),
            Self::NotHere(_) => None,
        }
    }
}

/// Whether a file can be made in `dir` (and then is removed).
fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".raul-write-test-{}", std::process::id()));
    let made = std::fs::File::create(&probe).is_ok();
    let _ = std::fs::remove_file(&probe);
    made
}

/// Decide the layout for the program at `exe`. `appimage` is the `APPIMAGE` variable, the path of the running `AppImage`.
#[must_use]
pub fn detect(exe: &Path, appimage: Option<&Path>, os: Os) -> Layout {
    if os == Os::Mac {
        return Layout::NotHere(Reason::Mac);
    }
    if os == Os::Linux
        && let Some(file) = appimage.filter(|f| f.is_file())
    {
        return match file.parent() {
            Some(dir) if writable(dir) => Layout::AppImage {
                file: file.to_path_buf(),
            },
            _ => Layout::NotHere(Reason::ReadOnly),
        };
    }
    if exe.starts_with("/nix/store") {
        return Layout::NotHere(Reason::Nix);
    }
    let Some(dir) = exe.parent() else {
        return Layout::NotHere(Reason::Other);
    };
    if dir.join(INVENTORY).is_file() {
        let Some(name) = exe.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            return Layout::NotHere(Reason::Other);
        };
        return if writable(dir) {
            Layout::Folder {
                root: dir.to_path_buf(),
                exe: name,
                os,
            }
        } else {
            Layout::NotHere(Reason::ReadOnly)
        };
    }
    if os == Os::Linux && (exe.starts_with("/usr") || exe.starts_with("/opt")) {
        return Layout::NotHere(Reason::System);
    }
    Layout::NotHere(Reason::Other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    #[test]
    fn a_folder_with_its_list_is_a_portable_install() {
        let t = TempDir::new("layout-folder");
        write_file(t.path(), "raul", b"x");
        write_file(t.path(), INVENTORY, b"");
        let l = detect(&t.path().join("raul"), None, Os::Linux);
        assert_eq!(
            l,
            Layout::Folder {
                root: t.path().to_path_buf(),
                exe: "raul".to_owned(),
                os: Os::Linux
            }
        );
        assert_eq!(l.asset("2.0.0").unwrap(), "raul-2.0.0-linux-x64.tar.gz");
        assert_eq!(l.root(), Some(t.path()));
        assert_eq!(l.program(), Some(t.path().join("raul")));
        let w = detect(&t.path().join("raul.exe"), None, Os::Windows);
        assert_eq!(
            w.asset("2.0.0").unwrap(),
            "raul-2.0.0-windows-x64-portable.zip"
        );
    }

    #[test]
    fn an_appimage_is_one_file() {
        let t = TempDir::new("layout-appimage");
        write_file(t.path(), "raul.AppImage", b"x");
        let file = t.path().join("raul.AppImage");
        let l = detect(
            Path::new("/tmp/.mount_x/usr/bin/raul"),
            Some(&file),
            Os::Linux,
        );
        assert_eq!(l, Layout::AppImage { file: file.clone() });
        assert_eq!(l.asset("2.0.0").unwrap(), "raul-2.0.0-linux-x64.AppImage");
        assert_eq!(l.root(), Some(t.path()));
        // An APPIMAGE that names no file is not believed.
        let l = detect(
            Path::new("/usr/bin/raul"),
            Some(&t.path().join("gone")),
            Os::Linux,
        );
        assert_eq!(l, Layout::NotHere(Reason::System));
    }

    #[test]
    fn nix_the_system_a_mac_an_installer_and_a_build_are_not_updated_here() {
        let t = TempDir::new("layout-other");
        write_file(t.path(), "raul", b"x");
        let here = t.path().join("raul");
        for (exe, os, reason) in [
            ("/nix/store/abc-raul/bin/raul", Os::Linux, Reason::Nix),
            ("/usr/bin/raul", Os::Linux, Reason::System),
            (
                "/Applications/Raul.app/Contents/MacOS/raul",
                Os::Mac,
                Reason::Mac,
            ),
        ] {
            assert_eq!(
                detect(Path::new(exe), None, os),
                Layout::NotHere(reason),
                "{exe}"
            );
        }
        // No list beside it: an installer's folder or a development build.
        assert_eq!(
            detect(&here, None, Os::Windows),
            Layout::NotHere(Reason::Other)
        );
        for reason in [
            Reason::Nix,
            Reason::System,
            Reason::Mac,
            Reason::ReadOnly,
            Reason::Other,
        ] {
            let text = reason.plain();
            assert!(
                !text.contains("raul update") && !text.contains("--"),
                "{text}"
            );
        }
        assert_eq!(Layout::NotHere(Reason::Nix).asset("1.0.0"), None);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_that_cannot_be_written_is_read_only() {
        use std::os::unix::fs::PermissionsExt;
        let t = TempDir::new("layout-readonly");
        write_file(t.path(), "raul", b"x");
        write_file(t.path(), INVENTORY, b"");
        std::fs::set_permissions(t.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        let l = detect(&t.path().join("raul"), None, Os::Linux);
        std::fs::set_permissions(t.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        // Running as root can write anywhere: then there is nothing to show.
        if !matches!(l, Layout::Folder { .. }) {
            assert_eq!(l, Layout::NotHere(Reason::ReadOnly));
        }
    }
}
