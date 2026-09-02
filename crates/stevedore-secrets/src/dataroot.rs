//! Making a store CLI's data directory owner-only before it runs.
//!
//! Both store CLIs keep vault state under the user's data home, and both leave
//! the mode to the umask — `dcli` writes a full local copy of the vault at
//! `0644`. Neither re-modes a path that already exists, so a directory
//! stevedore creates `0700` first is one the vendor fills but does not widen.
//!
//! This is a mode on a directory, not a sandbox. The tool still runs as the
//! user, with the network and the OS keyring in reach.

use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    dashlane::client::DCLI,
    error::{Error, Result},
    proton::client::PASS_CLI,
};

/// `dcli`'s directory under the data home.
const DASHLANE_DIR: &str = "dashlane-cli";
/// `pass-cli`'s directory under the data home.
const PROTON_DIR: &str = "proton-pass-cli";

/// The data home every store CLI here derives from `HOME`.
#[cfg(target_os = "macos")]
const DATA_HOME: &str = "Library/Application Support";
#[cfg(not(target_os = "macos"))]
const DATA_HOME: &str = ".local/share";

/// Make `program`'s data directory owner-only, creating it if it is absent.
///
/// A program stevedore has no verified path for is left alone, as is one whose
/// `HOME` is unset — the tool itself will fail on the same missing answer.
///
/// # Errors
///
/// [`Error::Io`] if the directory cannot be created or tightened. This is
/// deliberately fatal: hardening that fails quietly is the failure it exists to
/// prevent.
pub(crate) fn prepare(program: &'static str) -> Result<()> {
    let Some(dir) = data_dir_from_env(program) else {
        return Ok(());
    };
    make_private(&dir).map_err(|source| Error::Io {
        doing: "preparing the data directory for",
        program,
        source,
    })
}

fn data_dir_from_env(program: &str) -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    if home.is_empty() {
        return None;
    }
    let xdg = env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    data_dir(program, Path::new(&home), xdg.as_deref())
}

/// Where `program` keeps its state.
///
/// Keyed on the program name rather than reached through a `Store` trait: the
/// two CLIs resolve their roots differently, so this is a launch policy and not
/// a store behaviour.
fn data_dir(program: &str, home: &Path, xdg: Option<&Path>) -> Option<PathBuf> {
    match program {
        PASS_CLI => Some(xdg_data_home(home, xdg).join(PROTON_DIR)),
        DCLI => Some(home.join(DATA_HOME).join(DASHLANE_DIR)),
        _ => None,
    }
}

/// The data home as `pass-cli` resolves it: the `dirs` crate, which reads
/// `XDG_DATA_HOME` on Linux and ignores it when it is not an absolute path.
fn xdg_data_home(home: &Path, xdg: Option<&Path>) -> PathBuf {
    if cfg!(target_os = "linux")
        && let Some(dir) = xdg.filter(|p| p.is_absolute())
    {
        return dir.to_path_buf();
    }
    home.join(DATA_HOME)
}

/// Create `dir` owner-only, or clear group and other from one already there.
///
/// The parent is the shared data home, so it is created with the ordinary mode;
/// only the leaf is stevedore's to restrict.
#[cfg(unix)]
fn make_private(dir: &Path) -> std::io::Result<()> {
    use std::{
        fs::{self, DirBuilder, Permissions},
        io::ErrorKind,
        os::unix::fs::{DirBuilderExt as _, PermissionsExt as _},
    };

    if let Some(parent) = dir.parent() {
        fs::create_dir_all(parent)?;
    }
    match DirBuilder::new().mode(0o700).create(dir) {
        Ok(()) => return Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }

    let mode = match fs::metadata(dir) {
        Ok(meta) if meta.is_dir() => meta.permissions().mode(),
        // Something else holds the path. The vendor will say so better than a
        // chmod would.
        Ok(_) => return Ok(()),
        Err(e) => return Err(e),
    };
    if mode & 0o077 != 0 {
        fs::set_permissions(dir, Permissions::from_mode(mode & !0o077))?;
    }
    Ok(())
}

/// Unix modes are the whole mechanism, so elsewhere there is nothing to apply.
#[cfg(not(unix))]
fn make_private(_dir: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/home/someone";

    /// A path under the system temporary directory that no other test uses.
    fn scratch(name: &str) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static NEXT: AtomicUsize = AtomicUsize::new(0);
        env::temp_dir().join(format!(
            "stevedore-dataroot-{}-{name}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn dcli_derives_its_root_from_home_alone() {
        let home = Path::new(HOME);
        assert_eq!(
            data_dir(DCLI, home, Some(Path::new("/elsewhere"))),
            Some(home.join(DATA_HOME).join(DASHLANE_DIR)),
            "XDG_DATA_HOME does not reach `dcli`"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn pass_cli_follows_an_absolute_xdg_data_home() {
        assert_eq!(
            data_dir(PASS_CLI, Path::new(HOME), Some(Path::new("/elsewhere"))),
            Some(Path::new("/elsewhere").join(PROTON_DIR)),
            "`pass-cli` resolves its root with the `dirs` crate"
        );
    }

    #[test]
    fn pass_cli_ignores_a_relative_xdg_data_home() {
        let home = Path::new(HOME);
        assert_eq!(
            data_dir(PASS_CLI, home, Some(Path::new("relative/share"))),
            Some(home.join(DATA_HOME).join(PROTON_DIR)),
            "`dirs` discards a data home that is not an absolute path"
        );
    }

    #[test]
    fn an_unknown_program_has_no_data_directory() {
        assert!(
            data_dir("curl", Path::new(HOME), None).is_none(),
            "only the store CLIs stevedore has a verified path for are touched"
        );
    }

    #[cfg(unix)]
    mod unix {
        use std::{
            fs::{self, Permissions},
            os::unix::fs::PermissionsExt as _,
        };

        use super::{super::make_private, scratch};

        fn mode_of(path: &std::path::Path) -> u32 {
            fs::metadata(path)
                .expect("the test just created this path")
                .permissions()
                .mode()
                & 0o777
        }

        #[test]
        fn creates_a_missing_directory_owner_only() {
            let dir = scratch("create");
            make_private(&dir).expect("creating under the temporary directory");

            let mode = mode_of(&dir);
            assert_eq!(mode, 0o700, "created mode was {mode:o}");

            fs::remove_dir_all(&dir).expect("cleaning up");
        }

        #[test]
        fn clears_group_and_other_from_an_existing_directory() {
            let dir = scratch("tighten");
            fs::create_dir(&dir).expect("creating under the temporary directory");
            fs::set_permissions(&dir, Permissions::from_mode(0o755)).expect("widening it");

            make_private(&dir).expect("tightening it");

            let mode = mode_of(&dir);
            assert_eq!(mode, 0o700, "tightened mode was {mode:o}");

            fs::remove_dir_all(&dir).expect("cleaning up");
        }

        #[test]
        fn leaves_a_path_that_is_not_a_directory_alone() {
            let path = scratch("not-a-directory");
            fs::write(&path, b"whatever").expect("writing under the temporary directory");
            fs::set_permissions(&path, Permissions::from_mode(0o644)).expect("setting its mode");

            make_private(&path).expect("a non-directory is not stevedore's to re-mode");

            let mode = mode_of(&path);
            assert_eq!(mode, 0o644, "mode was {mode:o}");

            fs::remove_file(&path).expect("cleaning up");
        }
    }
}
