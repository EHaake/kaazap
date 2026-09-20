//! The profile that couldn't even be moved aside, in its own process: the
//! failure is reported with no kept file, and nothing the rest of the launch
//! does writes over it (spec 028, AC 11).
//!
//! Its own binary because the suspension is sticky for the process — once set
//! it silently turns every later `Profile::save` into a no-op, so a sibling
//! test (here or in `tests/profile_recovery.rs`) would pass for the wrong
//! reason. Keep this file to the one test for the same reason, plus the usual
//! one: the data root resolves once per process, and this test's first
//! assertion is that `set_root` takes.
//!
//! Unix only: the one deterministic way to make `fs::rename` fail is a
//! read-only directory, and a read-only directory doesn't block creation on
//! Windows — so there is no portable way to reach this case, and the
//! criterion is pinned here.
#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use kaazap::{
    paths::set_root,
    profile::{Profile, ProfileProblem},
};

/// The directory's entries, sorted — the whole set, debris included.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("directory readable")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_profile_that_cannot_be_moved_aside_stops_every_save_for_the_launch() {
    let root = std::env::temp_dir().join("kaazap-profile-save-suspended");
    let _ = fs::remove_dir_all(&root); // debris from an interrupted run
    fs::create_dir_all(&root).expect("scratch root");
    assert!(set_root(root.clone()), "nothing resolved the root before this");

    let profile_path = root.join("profile.json");
    let malformed = b"{\"version\": 1, \"credits\": 40".as_slice(); // truncated
    fs::write(&profile_path, malformed).expect("malformed profile");

    // A read-only directory: the file still reads, but nothing can be created,
    // renamed or removed inside — so the move aside fails.
    fs::set_permissions(&root, fs::Permissions::from_mode(0o555)).expect("read-only scratch root");
    let before = entries(&root);

    let (played_on, failure) = Profile::load();

    // Writable again the moment the load is done, before any assertion: a
    // failing assertion would otherwise leave a read-only directory behind,
    // and the next run's opening `remove_dir_all` cannot unlink entries inside
    // one — one failure would make this test permanently red until a human
    // chmods a directory under /tmp. It costs the test nothing: the suspension
    // is already set, and the saves below then get every chance to land, so
    // what blocks them is the flag rather than the permissions — which is all
    // this chmod was ever here for.
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).expect("scratch root writable");

    let failure = failure.expect("an unreadable profile is reported");
    assert_eq!(failure.problem, ProfileProblem::Unreadable);
    assert_eq!(failure.set_aside, None, "the move failed, so no file was kept");
    assert!(!played_on.differs_from_starter(), "play continues on a starter profile");

    // Nothing this session does may write over the file it couldn't read.
    let mut profile = played_on;
    profile.earn_credits(500);
    profile.save();
    Profile::default().save();

    assert_eq!(
        fs::read(&profile_path).expect("the unreadable file is still there"),
        malformed,
        "a save wrote over the file that couldn't be read",
    );
    assert_eq!(entries(&root), before, "a save left something in the directory");

    fs::remove_dir_all(&root).expect("scratch root removed");
}
