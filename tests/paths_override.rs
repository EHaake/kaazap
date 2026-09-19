//! The `paths::set_root` override, in its own process. The root is resolved
//! once per process, and the lib's own unit tests resolve it early (any
//! `App::new` does, through `Settings::load`), so this half of the contract
//! lives here rather than in `src/paths.rs`. Touches no real directory: it
//! only asks where files would go, and never writes one.
//!
//! Keep this file to the one test: its first assertion is that the override
//! *takes*, which holds only while nothing else in this binary has resolved
//! the root first. A sibling test here would race it.

use std::path::PathBuf;

use kaazap::paths::{config_dir, data_dir, set_root};

#[test]
fn override_root_serves_both_dirs_and_resolves_once() {
    let root = PathBuf::from("/kaazap-test-root");
    assert!(set_root(root.clone()), "nothing resolved the root before this");

    // One directory holds the profile, the match save and the settings.
    assert_eq!(data_dir(), Some(root.clone()));
    assert_eq!(config_dir(), Some(root.clone()));

    // Resolved now, so a second override doesn't take and nothing moves.
    assert!(!set_root(PathBuf::from("/kaazap-test-root-two")));
    assert_eq!(data_dir(), Some(root.clone()));
    assert_eq!(config_dir(), Some(root));
}
