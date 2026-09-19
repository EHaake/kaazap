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

use std::{env, path::PathBuf, sync::OnceLock};

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
}
