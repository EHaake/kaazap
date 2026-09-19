//! The other half of the `KAAZAP_DATA_DIR` filter: an empty value is not a
//! root. Without `!is_empty()` an exported-but-blank variable would resolve
//! the root to `""`, quietly writing the profile and the save into the
//! working directory. The root resolves once per process, so this needs a
//! binary of its own; keep this file to the one test, or the sibling would
//! race it. Touches no directory: it only asks where files would go.

use std::env;
use std::path::PathBuf;

use kaazap::paths::{config_dir, data_dir};

#[test]
fn empty_env_var_falls_through_to_the_platform_dirs() {
    // `set_var` is unsafe in edition 2024 because another thread could be
    // reading the environment. This is the only test in this binary and it
    // runs before the first path lookup, so nothing else is reading it here.
    unsafe { env::set_var("KAAZAP_DATA_DIR", "") };

    // Not an empty root — and the platform dirs, which are absolute. (Whether
    // they're resolvable at all is the platform's call, hence the `Option`.)
    for dir in [data_dir(), config_dir()] {
        assert_ne!(dir, Some(PathBuf::new()));
        assert!(dir.is_none_or(|path| path.is_absolute()));
    }
}
