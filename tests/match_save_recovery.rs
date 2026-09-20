//! A match save that can't be read, pinned on disk in its own process: it is
//! reported once, removed, and leaves no **Continue** behind — and the report
//! doesn't come back on the next launch, because the file is gone (spec 028,
//! AC 12). Nothing is kept: a match is not a run.
//!
//! Keep this file to the one test. The data root resolves once per process,
//! and this test's first assertion is that `set_root` *takes* — a sibling test
//! in this binary would race it for the root, and both would then be reading
//! whichever scratch directory won.

use std::fs;

use kaazap::{game::GameState, paths::set_root, save};

#[test]
fn an_unreadable_match_save_is_reported_once_and_removed() {
    let root = std::env::temp_dir().join("kaazap-match-save-recovery");
    let _ = fs::remove_dir_all(&root); // debris from an interrupted run
    assert!(set_root(root.clone()), "nothing resolved the root before this");

    let save_path = root.join("saves").join("savegame.json");

    // --- A first launch: no file, and no data directory. Silent. ---
    assert!(!save::check_at_launch(), "a first launch reports a failure");
    assert!(!root.exists(), "the check created the data directory");
    assert!(!save::exists(), "a first launch offers Continue");

    // --- A malformed save: reported, and removed so it can't report twice. ---
    fs::create_dir_all(save_path.parent().expect("the saves directory")).expect("scratch root");
    fs::write(&save_path, b"{\"version\": 1, \"player\":".as_slice()).expect("malformed save");

    assert!(save::check_at_launch(), "a malformed save is not reported");
    assert!(!save_path.exists(), "the malformed save is still in place");
    assert!(!save::exists(), "a removed save still offers Continue");

    // The whole point of removing it: the next launch has nothing to report.
    assert!(!save::check_at_launch(), "the notice repeats on the next launch");

    // --- A wrong-version save: a real one, with its version bumped past the
    // gate, so this covers the version check rather than a parse failure. ---
    save::save(&GameState::new());
    let current = fs::read_to_string(&save_path).expect("the save readable");
    let wrong_version = current.replace("\"version\": 1", "\"version\": 2");
    assert_ne!(wrong_version, current, "the version field wasn't where expected");
    fs::write(&save_path, &wrong_version).expect("wrong-version save");

    assert!(save::check_at_launch(), "a wrong-version save is not reported");
    assert!(!save_path.exists(), "the wrong-version save is still in place");
    assert!(!save::exists(), "a removed save still offers Continue");

    // --- A valid save: silent, kept, and resumable. ---
    save::save(&GameState::new());
    assert!(!save::check_at_launch(), "a valid save is reported as a failure");
    assert!(save_path.is_file(), "the valid save was removed");
    assert!(save::exists(), "the valid save no longer offers Continue");

    fs::remove_dir_all(&root).expect("scratch root removed");
}
