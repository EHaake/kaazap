//! The three writers pinned on disk, in their own process: settings, profile
//! and match save all go through `paths::write_whole`, so a save that fails
//! leaves the previous file exactly as it was and a following load reads that
//! file rather than any debris (spec 028, AC 6 and 7).
//!
//! Keep this file to the one test. The data root resolves once per process,
//! and this test's first assertion is that `set_root` *takes* — a sibling test
//! in this binary would race it for the root, and both would then be writing
//! into whichever scratch directory won.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kaazap::{
    game::GameState,
    paths::set_root,
    profile::Profile,
    save,
    settings::Settings,
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

/// No `*.tmp` left behind in `dir` — the debris a half-finished write would
/// leave, which no loader reads but which shouldn't survive a good write.
fn assert_no_tmp(dir: &Path) {
    assert!(
        !entries(dir).iter().any(|name| name.ends_with(".tmp")),
        "a temp file was left in {dir:?}: {:?}",
        entries(dir)
    );
}

/// A match worth saving, marked so a reload is recognisable.
fn a_match(player_rounds: usize, opponent_rounds: usize) -> GameState {
    let mut game = GameState::new();
    game.player.rounds_won = player_rounds;
    game.opponent.rounds_won = opponent_rounds;
    game
}

#[test]
fn every_writer_lands_whole_and_a_failed_save_leaves_the_previous_file() {
    let root = std::env::temp_dir().join("kaazap-whole-file-write");
    let _ = fs::remove_dir_all(&root); // debris from an interrupted run
    fs::create_dir_all(&root).expect("scratch root");
    assert!(set_root(root.clone()), "nothing resolved the root before this");

    let settings_path: PathBuf = root.join("settings.json");
    let profile_path: PathBuf = root.join("profile.json");
    let saves_dir: PathBuf = root.join("saves");
    let save_path: PathBuf = saves_dir.join("savegame.json");

    // --- Each writer lands its contents, and leaves no debris. ---
    let settings = Settings { music_volume: 0.25, sfx_volume: 0.75, animations: false };
    settings.save();

    let mut profile = Profile::default();
    profile.earn_credits(777);
    let credits = profile.credits(); // the seed purse plus the winnings above
    profile.save();

    save::save(&a_match(2, 1));

    assert!(settings_path.is_file(), "settings written");
    assert!(profile_path.is_file(), "profile written");
    assert!(save_path.is_file(), "match save written");
    assert_eq!(Settings::load(), settings);
    let (reloaded, failure) = Profile::load();
    assert!(failure.is_none(), "the profile loaded from its own file: {failure:?}");
    assert_eq!(reloaded.credits(), credits);
    let loaded = save::load().expect("the match save loads");
    assert_eq!((loaded.player.rounds_won, loaded.opponent.rounds_won), (2, 1));
    assert_no_tmp(&root);
    assert_no_tmp(&saves_dir);

    // --- With the temp path blocked, each save fails silently. ---
    let written_settings = fs::read(&settings_path).expect("settings readable");
    let written_profile = fs::read(&profile_path).expect("profile readable");
    let written_save = fs::read(&save_path).expect("match save readable");

    // A directory where each temp file wants to go: the write fails on every
    // platform, and it fails the way an unwritable path would.
    for path in [&settings_path, &profile_path, &save_path] {
        fs::create_dir(path.with_extension("tmp")).expect("blocking directory");
    }

    let mut later_settings = settings;
    later_settings.music_volume = 1.0;
    later_settings.animations = true;
    later_settings.save();

    let mut later_profile = profile.clone();
    later_profile.earn_credits(1000);
    later_profile.save();

    save::save(&a_match(3, 0));

    // Each file is byte-for-byte what it was, and each loader still returns it.
    assert_eq!(fs::read(&settings_path).expect("settings still there"), written_settings);
    assert_eq!(fs::read(&profile_path).expect("profile still there"), written_profile);
    assert_eq!(fs::read(&save_path).expect("match save still there"), written_save);
    assert_eq!(Settings::load(), settings);
    let (reloaded, failure) = Profile::load();
    assert!(failure.is_none(), "the previous profile still loads whole: {failure:?}");
    assert_eq!(reloaded.credits(), credits);
    let loaded = save::load().expect("the previous match save still loads");
    assert_eq!((loaded.player.rounds_won, loaded.opponent.rounds_won), (2, 1));

    // --- Debris left beside the real file is neither read nor consumed. ---
    // The blocking directory goes, and a genuine `settings.tmp` takes its
    // place: half a write's worth of bytes, under the name `write_whole`
    // would have used.
    let settings_tmp = settings_path.with_extension("tmp");
    fs::remove_dir(&settings_tmp).expect("blocking directory removed");
    let debris = "{\"music_volume\":1.0,\"sfx_volume\":1.0,\"anim";
    fs::write(&settings_tmp, debris).expect("debris written");

    assert_eq!(Settings::load(), settings, "the loader read the debris");
    assert_eq!(
        fs::read(&settings_tmp).expect("the debris is still there"),
        debris.as_bytes(),
        "the loader consumed or rewrote the debris"
    );

    fs::remove_dir_all(&root).expect("scratch root removed");
}
