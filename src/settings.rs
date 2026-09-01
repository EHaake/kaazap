use std::{fs, path::PathBuf};

use crossterm::event::KeyCode;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    frame::{Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text_in},
    layout::OverlayLayout,
};

/// Persisted user preferences — the seed of the save format. The future
/// save/resume spec extends this struct rather than rebuilding it, so the
/// fields carry `#[serde(default)]`: an older or partial settings file
/// still loads, with any missing field falling back to its default.
/// Volumes are 0.0–1.0; 0.0 means that channel is off.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default = "default_music_volume")]
    pub music_volume: f32,
    #[serde(default = "default_sfx_volume")]
    pub sfx_volume: f32,
}

fn default_music_volume() -> f32 {
    0.5 // background music sits a little under the effects
}
fn default_sfx_volume() -> f32 {
    0.8
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music_volume: default_music_volume(),
            sfx_volume: default_sfx_volume(),
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
    Louder,
    Quieter,
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

    /// Map a key to a settings action: ↑/↓ (w/s) move between rows, ←/→
    /// (a/d) adjust the selected row's volume, Esc goes back.
    pub fn handle_input(&self, key: KeyCode) -> Option<SettingsAction> {
        match key {
            KeyCode::Up | KeyCode::Char('w') => Some(SettingsAction::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(SettingsAction::Down),
            KeyCode::Right | KeyCode::Char('d') => Some(SettingsAction::Louder),
            KeyCode::Left | KeyCode::Char('a') => Some(SettingsAction::Quieter),
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

    /// Draw the settings panel as a bordered overlay over the menu: a
    /// centered title, the two volume rows (each a labelled bar +
    /// percentage; the selected one carries the `▸` marker and breathes
    /// with the pulse), and a controls hint. Sized and boxed like How to
    /// Play, so the two menu panels read the same.
    pub fn draw_overlay(
        &self,
        frame: &mut Frame,
        config: &Config,
        settings: Settings,
        pulse: Emphasis,
    ) {
        let rows = [
            (SettingRow::Music, "Music", settings.music_volume),
            (SettingRow::Sfx, "Sound FX", settings.sfx_volume),
        ];
        // "▸ " on the selected row, two spaces otherwise, so every row is
        // the same width and the bars stay column-aligned.
        let row_texts: Vec<String> = rows
            .iter()
            .map(|(row, label, vol)| {
                let marker = if self.selected == *row { "▸ " } else { "  " };
                let pct = (vol * 100.0).round() as u32;
                format!("{marker}{label:<9}{} {pct:>3}%", volume_bar(*vol))
            })
            .collect();

        let title = "Settings";
        let hint = "↑/↓ select  ·  ←/→ volume  ·  Esc back";

        // Box sized to the widest line; a fixed 6-row content column:
        // title, gap, the two rows, gap, hint.
        let content_width = row_texts
            .iter()
            .map(|s| s.chars().count())
            .chain([title.chars().count(), hint.chars().count()])
            .max()
            .unwrap_or(0);
        let layout = OverlayLayout::new(*config, content_width, 6);

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);

        draw_text_in(frame, layout.inner, 0, Align::Center, title, Emphasis::Strong);
        for (i, (row, _, _)) in rows.iter().enumerate() {
            let emphasis = if self.selected == *row { pulse } else { Emphasis::Normal };
            draw_text_in(frame, layout.inner, 2 + i, Align::Center, &row_texts[i], emphasis);
        }
        draw_text_in(frame, layout.inner, 5, Align::Center, hint, Emphasis::Muted);
    }
}

/// A fixed-width volume bar like `[██████░░░░]` for a 0.0–1.0 level.
fn volume_bar(vol: f32) -> String {
    const SEGMENTS: usize = 10;
    let filled = (vol * SEGMENTS as f32).round().clamp(0.0, SEGMENTS as f32) as usize;
    let mut bar = String::with_capacity(SEGMENTS + 2);
    bar.push('[');
    for i in 0..SEGMENTS {
        bar.push(if i < filled { '█' } else { '░' });
    }
    bar.push(']');
    bar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_volumes_are_audible() {
        let d = Settings::default();
        assert!(d.music_volume > 0.0 && d.music_volume <= 1.0);
        assert!(d.sfx_volume > 0.0 && d.sfx_volume <= 1.0);
    }

    #[test]
    fn settings_json_round_trips() {
        // Exact-in-f32 levels so the JSON round-trip compares equal.
        for s in [
            Settings { music_volume: 0.0, sfx_volume: 1.0 },
            Settings { music_volume: 0.5, sfx_volume: 0.5 },
            Settings { music_volume: 1.0, sfx_volume: 0.0 },
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
    fn settings_missing_or_legacy_fields_use_defaults() {
        // A partial object, or an older file with the retired bool fields,
        // still loads — absent volumes fall back to their defaults.
        assert_eq!(Settings::from_json_or_default("{}"), Settings::default());
        assert_eq!(
            Settings::from_json_or_default(r#"{"music_volume": 0.0}"#),
            Settings { music_volume: 0.0, sfx_volume: default_sfx_volume() }
        );
        // Legacy {"music","sfx"} bools are unknown now → ignored, defaults.
        assert_eq!(
            Settings::from_json_or_default(r#"{"music": true, "sfx": false}"#),
            Settings::default()
        );
    }

    #[test]
    fn settings_volume_bar_reflects_the_level() {
        assert_eq!(volume_bar(0.0), "[░░░░░░░░░░]");
        assert_eq!(volume_bar(1.0), "[██████████]");
        assert_eq!(volume_bar(0.5), "[█████░░░░░]");
    }
}
