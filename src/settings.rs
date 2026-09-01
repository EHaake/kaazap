use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Persisted user preferences — the seed of the save format. The future
/// save/resume spec extends this struct rather than rebuilding it, so the
/// fields carry `#[serde(default)]`: an older or partial settings file
/// still loads, with any missing field falling back to its default.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "on")]
    pub music: bool,
    #[serde(default = "on")]
    pub sfx: bool,
}

fn on() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music: true,
            sfx: true,
        }
    }
}

impl Settings {
    /// Load settings from the config file, or defaults on any error —
    /// missing dir/file, unreadable, or malformed JSON. Never panics.
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .map(|text| Self::from_json_or_default(&text))
            .unwrap_or_default()
    }

    /// Save settings to the config file. Best effort: a failure (no config
    /// dir, unwritable path) is swallowed — audio prefs aren't worth
    /// crashing over.
    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Parse settings JSON, falling back to defaults on any parse error.
    /// The filesystem-free core of `load`, so the fallback is testable.
    fn from_json_or_default(text: &str) -> Self {
        serde_json::from_str(text).unwrap_or_default()
    }

    /// The platform config-dir path for the settings file, if resolvable.
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "kaazap")
            .map(|dirs| dirs.config_dir().join("settings.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_is_music_and_sfx_on() {
        let d = Settings::default();
        assert!(d.music && d.sfx);
    }

    #[test]
    fn settings_json_round_trips() {
        for s in [
            Settings { music: true, sfx: true },
            Settings { music: false, sfx: true },
            Settings { music: true, sfx: false },
            Settings { music: false, sfx: false },
        ] {
            let json = serde_json::to_string(&s).unwrap();
            assert_eq!(Settings::from_json_or_default(&json), s);
        }
    }

    #[test]
    fn settings_malformed_or_empty_json_falls_back_to_default() {
        for bad in ["", "not json", "[1,2,3]", "42"] {
            assert_eq!(Settings::from_json_or_default(bad), Settings::default());
        }
    }

    #[test]
    fn settings_missing_fields_default_to_on() {
        // Forward/backward compatibility: an empty object or a partial one
        // still loads, with absent fields defaulting to on.
        assert_eq!(Settings::from_json_or_default("{}"), Settings::default());
        assert_eq!(
            Settings::from_json_or_default(r#"{"music": false}"#),
            Settings { music: false, sfx: true }
        );
        assert_eq!(
            Settings::from_json_or_default(r#"{"sfx": false}"#),
            Settings { music: true, sfx: false }
        );
    }
}
