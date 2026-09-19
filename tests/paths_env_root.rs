//! `KAAZAP_DATA_DIR` points a run at a scratch directory without a code
//! change — the lever a driver session or a throwaway run uses to stay off the
//! real profile folder. The root resolves once per process, so this needs a
//! binary of its own; keep this file to the one test, or the sibling would
//! race it. The empty-value half of the filter is `tests/paths_env_empty.rs`.
//! Touches no directory: it only asks where files would go.

use std::env;
use std::path::PathBuf;

use kaazap::paths::{config_dir, data_dir};

#[test]
fn env_var_root_serves_both_dirs() {
    let root = "/kaazap-env-test-root";

    // `set_var` is unsafe in edition 2024 because another thread could be
    // reading the environment. This is the only test in this binary and it
    // runs before the first path lookup, so nothing else is reading it here.
    unsafe { env::set_var("KAAZAP_DATA_DIR", root) };

    assert_eq!(data_dir(), Some(PathBuf::from(root)));
    assert_eq!(config_dir(), Some(PathBuf::from(root)));
}
