//! Where kaazap's files live. The profile (`profile.rs`), the match save
//! (`save.rs`) and the settings (`settings.rs`) each used to resolve
//! `ProjectDirs` for themselves; they all come here instead, so one override
//! points every one of them at a scratch directory — for a test, or for a
//! throwaway run that must not touch the real data folder.
//!
//! The root is resolved once, on first use: an explicit [`set_root`] wins,
//! then the `KAAZAP_DATA_DIR` environment variable, then the platform
//! directories. Only in that last case do [`data_dir`] and [`config_dir`]
//! differ — an override is one directory holding all three files, which is
//! the point of it (the filenames don't collide).
//!
//! Since spec 028 this module owns both halves of the question: where those
//! three files live, and how they are written. Every one of them goes through
//! [`write_whole`] — beside it, then renamed over it — so a write that fails
//! partway leaves the previous file exactly as it was.

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use directories::ProjectDirs;

/// The process's root directory: `Some` when overridden (explicitly or by the
/// environment), `None` when the platform directories are in use. Resolved at
/// most once — the first lookup fixes it for the life of the process.
static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Point every kaazap file at `root`, if nothing has resolved the root yet.
/// Returns whether the override took. For tests and for aiming a throwaway
/// run at a scratch directory — deliberately not `cfg(test)`-only, since
/// integration tests and the driver call it from outside the crate.
pub fn set_root(root: PathBuf) -> bool {
    ROOT.set(Some(root)).is_ok()
}

/// The overriding root, resolving it from the environment on first use.
fn root() -> Option<&'static PathBuf> {
    ROOT.get_or_init(|| {
        env::var_os("KAAZAP_DATA_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
    .as_ref()
}

/// The directory for the profile and the match save, if resolvable.
pub fn data_dir() -> Option<PathBuf> {
    match root() {
        Some(root) => Some(root.clone()),
        None => ProjectDirs::from("", "", "kaazap").map(|dirs| dirs.data_dir().to_path_buf()),
    }
}

/// The directory for the settings file, if resolvable.
pub fn config_dir() -> Option<PathBuf> {
    match root() {
        Some(root) => Some(root.clone()),
        None => ProjectDirs::from("", "", "kaazap").map(|dirs| dirs.config_dir().to_path_buf()),
    }
}

/// Write `contents` to `path` whole (spec 028): the bytes go to a temp file
/// beside it, which is then renamed over it, so the contents are complete
/// before they become the file the game reads and an interrupted write leaves
/// the previous file exactly as it was. (The replacement is atomic on POSIX;
/// on Windows it is a rename that may not be, but still never leaves a
/// half-written file under the real name — and fails outright, as a silent
/// no-op save, if another process holds the target open.) The temp name is
/// fixed, so repeated interrupted writes leave one piece of debris rather than
/// a pile, and it is never `*.json`, so no loader reads it. Best-effort like
/// the three callers it serves: any failure removes the temp file and returns
/// `false`, leaving the previous file alone. Not durable against a power cut —
/// see plan §3.
pub(crate) fn write_whole(path: &Path, contents: &str) -> bool {
    let tmp = path.with_extension("tmp");
    if fs::write(&tmp, contents).is_err() || fs::rename(&tmp, path).is_err() {
        let _ = fs::remove_file(&tmp);
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // The root is process-global, and this binary's `App::new` tests resolve it
    // (via `Settings::load`), so a unit test can't count on winning the race to
    // set it. The override half of the contract is therefore an integration
    // test, in its own process: `tests/paths_override.rs`. What is testable
    // here is the other half — once resolved, the root stays put.
    #[test]
    fn set_root_does_not_take_once_the_root_is_resolved() {
        let _ = data_dir(); // resolves the root, whatever it resolves to
        assert!(!set_root(PathBuf::from("/kaazap-never-written")));
    }

    /// A scratch directory of this test's own, empty, under the system temp
    /// directory. `write_whole` takes the path it writes, so these tests need
    /// no data root at all — and must not ask for one, which would resolve the
    /// process's root out from under the test above.
    fn scratch(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("kaazap-write-whole-{name}"));
        let _ = fs::remove_dir_all(&dir); // debris from an interrupted run
        fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    /// The directory's entries, sorted — the whole set, debris included.
    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .expect("scratch directory readable")
            .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn write_whole_replaces_the_file_and_leaves_no_debris() {
        let dir = scratch("replaces");
        let path = dir.join("data.json");

        assert!(write_whole(&path, "first"));
        assert!(write_whole(&path, "second"));

        assert_eq!(fs::read_to_string(&path).expect("written"), "second");
        assert_eq!(entries(&dir), vec!["data.json".to_string()]);

        fs::remove_dir_all(&dir).expect("scratch removed");
    }

    #[test]
    fn a_failed_write_leaves_the_previous_file_untouched() {
        let dir = scratch("failed");
        let path = dir.join("data.json");
        fs::write(&path, "the previous file").expect("previous file");

        // A directory where the temp file wants to go: the write fails on
        // every platform, and it fails the way an unwritable path would.
        fs::create_dir(path.with_extension("tmp")).expect("blocking directory");

        assert!(!write_whole(&path, "never lands"));
        assert_eq!(
            fs::read_to_string(&path).expect("previous file still there"),
            "the previous file"
        );

        fs::remove_dir_all(&dir).expect("scratch removed");
    }

    #[test]
    fn repeated_failed_writes_do_not_accumulate() {
        let dir = scratch("repeated");
        let path = dir.join("data.json");
        fs::write(&path, "the previous file").expect("previous file");
        fs::create_dir(path.with_extension("tmp")).expect("blocking directory");

        let before = entries(&dir);
        for _ in 0..3 {
            assert!(!write_whole(&path, "never lands"));
        }

        assert_eq!(entries(&dir), before);

        fs::remove_dir_all(&dir).expect("scratch removed");
    }
}
