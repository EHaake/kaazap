use std::{fs, path::PathBuf};

use crossterm::event::KeyCode;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    frame::{Emphasis, Frame, draw_text},
};

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

/// A row on the settings screen. Two for now (the spec's first settings);
/// the save/campaign specs add more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingRow {
    Music,
    Sfx,
}

/// What a key does on the settings screen — resolved by `App` (which owns
/// the `Settings` and the `Audio`), since the row values live there.
#[derive(Debug, Clone, Copy)]
pub enum SettingsAction {
    Up,
    Down,
    Toggle,
    Back,
}

/// The settings screen's own state: which row the cursor is on. Drawn with
/// the shared monochrome + cursor vocabulary, like the menu.
#[derive(Debug)]
pub struct SettingsState {
    selected: SettingRow,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self { selected: SettingRow::Music }
    }
}

impl SettingsState {
    pub fn selected(&self) -> SettingRow {
        self.selected
    }

    /// Map a key to a settings action (or nothing). Mirrors the menu's
    /// vocabulary: ↑/↓ (w/s) move, Enter/Space toggle, Esc goes back.
    pub fn handle_input(&self, key: KeyCode) -> Option<SettingsAction> {
        match key {
            KeyCode::Up | KeyCode::Char('w') => Some(SettingsAction::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(SettingsAction::Down),
            KeyCode::Enter | KeyCode::Char(' ') => Some(SettingsAction::Toggle),
            KeyCode::Esc => Some(SettingsAction::Back),
            _ => None,
        }
    }

    // Two rows, clamped: up → the top row, down → the bottom. (Generalize
    // to an index when there are more than two.)
    pub fn move_up(&mut self) {
        self.selected = SettingRow::Music;
    }
    pub fn move_down(&mut self) {
        self.selected = SettingRow::Sfx;
    }

    /// Draw the settings screen: a centered title, the toggle rows (the
    /// selected one carries the `▸` marker and breathes with the pulse),
    /// and a controls hint — reading like the start menu.
    pub fn draw(&self, frame: &mut Frame, config: &Config, settings: Settings, pulse: Emphasis) {
        let center_x = config.num_cols / 2;
        const BLOCK_H: usize = 7;
        let top = config.num_rows.saturating_sub(BLOCK_H) / 2;

        let title = "Settings";
        draw_text(
            frame,
            center_x.saturating_sub(title.chars().count() / 2),
            top,
            title,
            Emphasis::Strong,
        );

        // Rows left-aligned as a list; values line up in a column.
        let row_x = center_x.saturating_sub(7);
        for (i, (row, label, on)) in [
            (SettingRow::Music, "Music", settings.music),
            (SettingRow::Sfx, "Sound FX", settings.sfx),
        ]
        .into_iter()
        .enumerate()
        {
            let selected = self.selected == row;
            let marker = if selected { "▸ " } else { "  " };
            let value = if on { "On" } else { "Off" };
            let emphasis = if selected { pulse } else { Emphasis::Normal };
            let text = format!("{marker}{label:<9}{value}");
            draw_text(frame, row_x, top + 2 + i * 2, &text, emphasis);
        }

        let hint = "↑/↓ select  ·  Enter toggle  ·  Esc back";
        draw_text(
            frame,
            center_x.saturating_sub(hint.chars().count() / 2),
            top + 6,
            hint,
            Emphasis::Muted,
        );
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
