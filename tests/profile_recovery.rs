//! A profile that can't be read, pinned on disk in its own process: the file
//! is kept under a dated name, the failure is reported with its reason, play
//! continues on a starter profile, and saving afterwards writes a new profile
//! without touching — or duplicating — the file that was kept (spec 028,
//! AC 8, 9 and 10).
//!
//! Keep this file to the one test. The data root resolves once per process,
//! and this test's first assertion is that `set_root` *takes* — a sibling test
//! in this binary would race it for the root, and both would then be reading
//! whichever scratch directory won.
//!
//! The `chmod 0o000` step asserts that the read *fails*: those bytes are a
//! valid profile, so running as root — where a `0o000` file is still
//! readable — makes this test fail loudly rather than pass for the wrong
//! reason. That is the honest outcome; don't soften it.

use std::{fs, path::Path};

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

/// Whether `name` is a set-aside profile: `profile-<8 digits>-<6 digits>.json`,
/// or that with a single-digit `-n` before the extension when the dated name
/// was already taken.
fn is_set_aside_name(name: &str) -> bool {
    let digits = |part: &str, len: usize| {
        part.len() == len && part.chars().all(|c| c.is_ascii_digit())
    };
    let Some(rest) = name.strip_prefix("profile-") else {
        return false;
    };
    let Some(rest) = rest.strip_suffix(".json") else {
        return false;
    };
    match rest.split('-').collect::<Vec<_>>().as_slice() {
        [date, time] => digits(date, 8) && digits(time, 6),
        [date, time, n] => digits(date, 8) && digits(time, 6) && digits(n, 1),
        _ => false,
    }
}

/// The set-aside files in `dir`, sorted — everything else (the live profile,
/// the settings) ignored.
fn set_aside_names(dir: &Path) -> Vec<String> {
    entries(dir)
        .into_iter()
        .filter(|name| is_set_aside_name(name))
        .collect()
}

/// The case the other two can't reach portably: a file that is there and
/// can't be read at all. A no-op off Unix — `chmod` is the only deterministic
/// way in, and a read-only file is still readable on Windows.
#[cfg(unix)]
fn an_unreadable_file_is_kept_too(root: &Path, good_json: &[u8]) {
    use std::os::unix::fs::PermissionsExt;

    let profile_path = root.join("profile.json");
    fs::write(&profile_path, good_json).expect("a good profile to make unreadable");
    fs::set_permissions(&profile_path, fs::Permissions::from_mode(0o000))
        .expect("the profile made unreadable");

    let (played_on, failure) = Profile::load();
    // These bytes parse cleanly, so a read that *succeeded* reports no failure
    // at all: as root, where `0o000` doesn't stop a read, this line fails —
    // deliberately, rather than passing as if the case had been covered.
    let failure = failure.expect("an unreadable profile is reported (are you running as root?)");
    assert_eq!(failure.problem, ProfileProblem::Unreadable);
    let name = failure.set_aside.expect("the unreadable file was kept");
    assert!(is_set_aside_name(&name), "not a dated set-aside name: {name}");
    assert!(!profile_path.exists(), "the unreadable profile is still in place");
    assert!(
        !played_on.differs_from_starter(),
        "play continues on a starter profile",
    );

    // `set_aside` has already *renamed* the file, so the mode goes back on the
    // set-aside path — `profile.json` no longer exists. (Removal would work
    // either way: unlinking needs write permission on the directory, not on
    // the file.) Readable again, its bytes are the ones that went in.
    let kept = root.join(&name);
    fs::set_permissions(&kept, fs::Permissions::from_mode(0o644)).expect("the kept file readable");
    assert_eq!(
        fs::read(&kept).expect("the kept file readable"),
        good_json,
        "the kept file is not the original bytes",
    );
}

#[cfg(not(unix))]
fn an_unreadable_file_is_kept_too(_root: &Path, _good_json: &[u8]) {}

#[test]
fn a_profile_that_cannot_be_read_is_kept_reported_and_played_past() {
    let root = std::env::temp_dir().join("kaazap-profile-recovery");
    let _ = fs::remove_dir_all(&root); // debris from an interrupted run
    assert!(set_root(root.clone()), "nothing resolved the root before this");

    let profile_path = root.join("profile.json");

    // --- A first launch: no file, and no data directory. Silent. ---
    let (starter, failure) = Profile::load();
    assert!(failure.is_none(), "a first launch is silent: {failure:?}");
    assert!(!starter.differs_from_starter(), "a first launch is the starter profile");
    assert!(!root.exists(), "loading created the data directory");

    // --- The directory there but empty: still a first launch. ---
    fs::create_dir_all(&root).expect("scratch root");
    let (starter, failure) = Profile::load();
    assert!(failure.is_none(), "an empty data directory is silent: {failure:?}");
    assert!(!starter.differs_from_starter(), "still the starter profile");
    assert!(entries(&root).is_empty(), "loading wrote something: {:?}", entries(&root));

    // --- A malformed profile: kept under a dated name, reported, played past. ---
    let malformed = b"{\"version\": 1, \"credits\": 40".as_slice(); // truncated
    fs::write(&profile_path, malformed).expect("malformed profile");

    let (played_on, failure) = Profile::load();
    let failure = failure.expect("a malformed profile is reported");
    assert_eq!(failure.problem, ProfileProblem::Unreadable);
    let first_name = failure.set_aside.expect("the malformed file was kept");
    assert!(is_set_aside_name(&first_name), "not a dated set-aside name: {first_name}");
    assert_eq!(
        fs::read(root.join(&first_name)).expect("the kept file readable"),
        malformed,
        "the kept file is not the original bytes",
    );
    assert!(!profile_path.exists(), "the malformed profile is still in place");
    assert!(!played_on.differs_from_starter(), "play continues on a starter profile");

    // --- Saving afterwards writes a new profile and leaves the kept file alone. ---
    let mut profile = played_on;
    profile.earn_credits(500);
    let credits = profile.credits();
    profile.save();

    assert!(profile_path.is_file(), "the save wrote no new profile");
    let (reloaded, failure) = Profile::load();
    assert!(failure.is_none(), "the new profile doesn't parse: {failure:?}");
    assert_eq!(reloaded.credits(), credits);
    assert_eq!(
        fs::read(root.join(&first_name)).expect("the kept file readable"),
        malformed,
        "the save wrote over the kept file",
    );
    assert_eq!(
        set_aside_names(&root),
        vec![first_name.clone()],
        "the save made a second set-aside copy",
    );

    // A whole, parseable profile's bytes — the unreadable step below needs a
    // file that would load fine if only it could be read.
    let good = fs::read(&profile_path).expect("the new profile readable");

    // --- A wrong-version document: a different reason, a different name. ---
    let wrong_version = b"{\"version\": 2}".as_slice();
    fs::write(&profile_path, wrong_version).expect("wrong-version profile");

    let (played_on, failure) = Profile::load();
    let failure = failure.expect("a wrong-version profile is reported");
    assert_eq!(failure.problem, ProfileProblem::WrongVersion);
    let second_name = failure.set_aside.expect("the wrong-version file was kept");
    assert!(is_set_aside_name(&second_name), "not a dated set-aside name: {second_name}");
    assert_ne!(second_name, first_name, "the second file took the first one's name");
    assert!(!profile_path.exists(), "the wrong-version profile is still in place");
    assert!(!played_on.differs_from_starter(), "play continues on a starter profile");

    // Both kept files are intact, each holding what was written to it.
    assert_eq!(
        fs::read(root.join(&first_name)).expect("the first kept file readable"),
        malformed,
    );
    assert_eq!(
        fs::read(root.join(&second_name)).expect("the second kept file readable"),
        wrong_version,
    );

    // --- And a file that can't be read at all (Unix only). ---
    an_unreadable_file_is_kept_too(&root, &good);

    fs::remove_dir_all(&root).expect("scratch root removed");
}
