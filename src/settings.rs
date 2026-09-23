use std::{fs, path::PathBuf};

use crossterm::event::KeyCode;
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
    /// Spec 027: the board's one-shot transitions and the opponent's spoken
    /// lines (spec 030). Off draws the board settled and each line whole.
    /// A file without the key reads as On.
    #[serde(default = "default_animations")]
    pub animations: bool,
    /// Spec 030 (ruling 10A): the opponent's spoken-word burble, and no
    /// other sound. A file without the key reads as the default, equal to
    /// Sound FX's. Declared last on purpose: serde also accepts a struct
    /// as a positional JSON array, and the malformed-JSON test's `[1,2,3]`
    /// must keep failing on the `animations` bool.
    #[serde(default = "default_voices_volume")]
    pub voices_volume: f32,
}

fn default_music_volume() -> f32 {
    0.5 // background music sits a little under the effects
}
fn default_sfx_volume() -> f32 {
    0.8
}
fn default_voices_volume() -> f32 {
    0.8
}
fn default_animations() -> bool {
    true
}

/// How much one ←/→ press moves a volume row.
const VOLUME_STEP: f32 = 0.1;

impl Default for Settings {
    fn default() -> Self {
        Self {
            music_volume: default_music_volume(),
            sfx_volume: default_sfx_volume(),
            animations: default_animations(),
            voices_volume: default_voices_volume(),
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
            crate::paths::write_whole(&path, &json);
        }
    }

    /// Parse settings JSON, falling back to defaults on any parse error.
    /// The filesystem-free core of `load`, so the fallback is testable.
    fn from_json_or_default(text: &str) -> Self {
        serde_json::from_str(text).unwrap_or_default()
    }

    /// `<config_dir>/settings.json`, if a config dir is resolvable.
    fn config_path() -> Option<PathBuf> {
        crate::paths::config_dir().map(|dir| dir.join("settings.json"))
    }

    /// Apply ←/→ on `row`: a volume steps by VOLUME_STEP and clamps to 0..=1;
    /// Animations toggles either way. Pure — the caller persists.
    pub fn adjust(&mut self, row: SettingRow, right: bool) {
        let delta = if right { VOLUME_STEP } else { -VOLUME_STEP };
        match row {
            SettingRow::Music => {
                self.music_volume = (self.music_volume + delta).clamp(0.0, 1.0);
            }
            SettingRow::Sfx => {
                self.sfx_volume = (self.sfx_volume + delta).clamp(0.0, 1.0);
            }
            SettingRow::Voices => {
                self.voices_volume = (self.voices_volume + delta).clamp(0.0, 1.0);
            }
            SettingRow::Animations => self.animations = !self.animations,
        }
    }
}

/// A row on the settings screen: the three volumes (spec 004; Voices, spec
/// 030) and the board's Animations toggle (spec 027).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingRow {
    Music,
    Sfx,
    Voices,
    Animations,
}

/// Top to bottom, as drawn.
const ROWS: [SettingRow; 4] =
    [SettingRow::Music, SettingRow::Sfx, SettingRow::Voices, SettingRow::Animations];

/// What a key does on the settings screen — resolved by `App` (which owns
/// the `Settings` and the `Audio`), since the row values live there.
#[derive(Debug, Clone, Copy)]
pub enum SettingsAction {
    Up,
    Down,
    Left,
    Right,
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
    /// (a/d) adjust the selected row (a volume steps, Animations toggles);
    /// Esc — and Enter/Space, the keys that opened the panel — close it,
    /// matching How to Play.
    pub fn handle_input(&self, key: KeyCode) -> Option<SettingsAction> {
        match key {
            KeyCode::Up | KeyCode::Char('w') => Some(SettingsAction::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(SettingsAction::Down),
            KeyCode::Right | KeyCode::Char('d') => Some(SettingsAction::Right),
            KeyCode::Left | KeyCode::Char('a') => Some(SettingsAction::Left),
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => Some(SettingsAction::Back),
            _ => None,
        }
    }

    // The cursor's index in ROWS, moved by one and clamped at either end.
    fn index(&self) -> usize {
        ROWS.iter().position(|r| *r == self.selected).unwrap_or(0)
    }
    pub fn move_up(&mut self) {
        self.selected = ROWS[self.index().saturating_sub(1)];
    }
    pub fn move_down(&mut self) {
        self.selected = ROWS[(self.index() + 1).min(ROWS.len() - 1)];
    }

    /// Draw the settings panel as a bordered overlay over the menu: a
    /// centered title, the three volume rows (each a labelled bar +
    /// percentage) and the Animations row (`On`/`Off`) — the selected one
    /// carries the `▸` marker and breathes with the pulse — and a controls
    /// hint. Sized and boxed like How to Play, so the two menu panels read
    /// the same.
    pub fn draw_overlay(
        &self,
        frame: &mut Frame,
        config: &Config,
        settings: Settings,
        pulse: Emphasis,
    ) {
        let row_texts: Vec<String> =
            ROWS.iter().map(|row| self.row_text(*row, settings)).collect();

        let title = "Settings";
        let hint = "↑/↓ select  ·  ←/→ change  ·  Esc back";

        // Box sized to the widest line; a fixed 8-row content column:
        // title, gap, the four rows, gap, hint.
        let content_width = row_texts
            .iter()
            .map(|s| s.chars().count())
            .chain([title.chars().count(), hint.chars().count()])
            .max()
            .unwrap_or(0);
        let layout = OverlayLayout::new(*config, content_width, 8);

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);

        // Title emphasis matches the other overlays (How to Play, `?` help),
        // which draw their centered title line `Normal`.
        draw_text_in(frame, layout.inner, 0, Align::Center, title, Emphasis::Normal);
        for (i, row) in ROWS.iter().enumerate() {
            let emphasis = if self.selected == *row { pulse } else { Emphasis::Normal };
            draw_text_in(frame, layout.inner, 2 + i, Align::Center, &row_texts[i], emphasis);
        }
        draw_text_in(frame, layout.inner, 7, Align::Center, hint, Emphasis::Muted);
    }

    /// One row's text: "▸ " on the selected row, two spaces otherwise, and
    /// every row padded to the volume rows' width, so the centered rows
    /// share the marker and label columns. The three volume rows also share
    /// the bar/percent columns; the Animations label is one character wider
    /// than the volume label column, so its value sits one space after it.
    fn row_text(&self, row: SettingRow, settings: Settings) -> String {
        let marker = if self.selected == row { "▸ " } else { "  " };
        match row {
            SettingRow::Music => volume_row(marker, "Music", settings.music_volume),
            SettingRow::Sfx => volume_row(marker, "Sound FX", settings.sfx_volume),
            SettingRow::Voices => volume_row(marker, "Voices", settings.voices_volume),
            SettingRow::Animations => {
                let value = if settings.animations { "On" } else { "Off" };
                // Padded to the volume rows' width (marker 2 + label 9 +
                // bar 12 + " 100%" 5 = 28): 2 + "Animations " 11 + 15.
                format!("{marker}Animations {value:<15}")
            }
        }
    }
}

/// A labelled volume row: bar + percentage.
fn volume_row(marker: &str, label: &str, vol: f32) -> String {
    let pct = (vol * 100.0).round() as u32;
    format!("{marker}{label:<9}{} {pct:>3}%", volume_bar(vol))
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
            Settings { music_volume: 0.0, sfx_volume: 1.0, voices_volume: 0.0, animations: true },
            Settings { music_volume: 0.5, sfx_volume: 0.5, voices_volume: 0.5, animations: false },
            Settings { music_volume: 1.0, sfx_volume: 0.0, voices_volume: 1.0, animations: true },
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
            Settings {
                music_volume: 0.0,
                sfx_volume: default_sfx_volume(),
                voices_volume: default_voices_volume(),
                animations: true
            }
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

    #[test]
    fn settings_closes_on_esc_enter_or_space() {
        // Esc — and the Enter/Space that open the panel — all close it, so
        // it dismisses like How to Play (close with the key you opened on).
        let s = SettingsState::default();
        for key in [KeyCode::Esc, KeyCode::Enter, KeyCode::Char(' ')] {
            assert!(matches!(s.handle_input(key), Some(SettingsAction::Back)));
        }
    }

    #[test]
    fn settings_animations_default_on_missing_key_on_and_off_reads_off() {
        // Spec 027 (AC 8): On by default, On from a file without the key,
        // Off from a file with it Off, and the field round-trips.
        assert!(Settings::default().animations);
        assert!(Settings::from_json_or_default("{}").animations);
        assert!(!Settings::from_json_or_default(r#"{"animations": false}"#).animations);
        let off = Settings { animations: false, ..Settings::default() };
        let json = serde_json::to_string(&off).unwrap();
        assert_eq!(Settings::from_json_or_default(&json), off);
    }

    #[test]
    fn adjust_toggles_animations_and_steps_volumes() {
        let mut s = Settings::default();
        s.adjust(SettingRow::Animations, true);
        assert!(!s.animations, "→ toggles Off");
        s.adjust(SettingRow::Animations, false);
        assert!(s.animations, "← toggles back On");

        let mut s =
            Settings { music_volume: 0.5, sfx_volume: 0.5, voices_volume: 0.5, animations: true };
        s.adjust(SettingRow::Music, true);
        assert!((s.music_volume - 0.6).abs() < 1e-6);
        s.adjust(SettingRow::Sfx, false);
        assert!((s.sfx_volume - 0.4).abs() < 1e-6);
        assert!(s.animations, "a volume step leaves Animations alone");

        for _ in 0..20 {
            s.adjust(SettingRow::Music, true);
            s.adjust(SettingRow::Sfx, false);
        }
        assert_eq!(s.music_volume, 1.0, "clamped at 1");
        assert_eq!(s.sfx_volume, 0.0, "clamped at 0");
    }

    #[test]
    fn settings_rows_move_over_four_rows_and_clamp() {
        let mut s = SettingsState::default();
        assert_eq!(s.selected(), SettingRow::Music);
        s.move_down();
        assert_eq!(s.selected(), SettingRow::Sfx);
        s.move_down();
        assert_eq!(s.selected(), SettingRow::Voices);
        s.move_down();
        assert_eq!(s.selected(), SettingRow::Animations);
        s.move_down();
        assert_eq!(s.selected(), SettingRow::Animations, "clamped at the bottom");
        s.move_up();
        assert_eq!(s.selected(), SettingRow::Voices);
        s.move_up();
        assert_eq!(s.selected(), SettingRow::Sfx);
        s.move_up();
        assert_eq!(s.selected(), SettingRow::Music);
        s.move_up();
        assert_eq!(s.selected(), SettingRow::Music, "clamped at the top");
    }

    #[test]
    fn the_animations_row_reads_on_or_off_and_fits() {
        fn row_text(frame: &Frame, y: usize) -> String {
            frame.iter().map(|col| col[y].ch).collect()
        }
        for cols in [89, 139] {
            let config = Config { num_cols: cols, num_rows: 31 };
            for (animations, word) in [(true, "On"), (false, "Off")] {
                let settings = Settings { animations, ..Settings::default() };
                let state = SettingsState::default();
                let mut frame = crate::frame::new_frame(&config);
                state.draw_overlay(&mut frame, &config, settings, Emphasis::Normal);
                let found = (0..31).map(|y| row_text(&frame, y)).find(|r| r.contains("Animations"));
                let row = found.unwrap_or_else(|| panic!("an Animations row at {cols}"));
                // The row's content sits between the box's side borders.
                let content = row.trim_end_matches([' ', '│']).trim_end();
                assert!(content.ends_with(word), "{cols}: {row:?} ends in {word}");
                // Aligned with the volume rows: the label starts in the Music
                // row's label column (the marker column is the one before),
                // and the value follows the label after one space.
                let music = (0..31).map(|y| row_text(&frame, y)).find(|r| r.contains("Music"));
                let music = music.unwrap_or_else(|| panic!("a Music row at {cols}"));
                // A substring's column, in characters (the rows hold box glyphs).
                let col = |r: &str, s: &str| r[..r.find(s).unwrap()].chars().count();
                let label_col = col(&row, "Animations");
                assert_eq!(label_col, col(&music, "Music"), "{cols}: label columns");
                assert_eq!(music.chars().nth(label_col - 2), Some('▸'), "{cols}: marker column");
                assert_eq!(col(&row, word), label_col + "Animations ".chars().count(), "{cols}: value column");
                // The box is inside the frame: its corners are drawn.
                let content_width = "↑/↓ select  ·  ←/→ change  ·  Esc back".chars().count();
                let layout = OverlayLayout::new(config, content_width, 8);
                assert!(layout.outer.x1 < cols && layout.outer.y1 < 31);
                assert_eq!(frame[layout.outer.x0][layout.outer.y0].ch, '┌');
                assert_eq!(frame[layout.outer.x1][layout.outer.y1].ch, '┘');
            }
        }
    }

    #[test]
    fn the_voices_volume_defaults_loads_and_persists() {
        // Spec 030 (ruling 10A, AC 17): the default equals Sound FX's.
        assert_eq!(Settings::default().voices_volume, default_voices_volume());
        assert_eq!(default_voices_volume(), default_sfx_volume());
        // A file without the key loads Voices at its default and keeps every
        // other value as written, so an older file is not reset.
        assert_eq!(Settings::from_json_or_default("{}").voices_volume, default_voices_volume());
        let old = r#"{"music_volume":0.3,"sfx_volume":0.4,"animations":false}"#;
        assert_eq!(
            Settings::from_json_or_default(old),
            Settings {
                music_volume: 0.3,
                sfx_volume: 0.4,
                voices_volume: default_voices_volume(),
                animations: false
            }
        );
        assert_eq!(Settings::from_json_or_default(r#"{"voices_volume":0.2}"#).voices_volume, 0.2);
        let s = Settings { voices_volume: 0.25, ..Settings::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(Settings::from_json_or_default(&json), s);
    }

    #[test]
    fn adjust_steps_and_clamps_the_voices_volume() {
        let mut s = Settings {
            music_volume: 0.5,
            sfx_volume: 0.5,
            voices_volume: 0.5,
            animations: true,
        };
        s.adjust(SettingRow::Voices, true);
        assert!((s.voices_volume - 0.6).abs() < 1e-6, "→ steps up");
        s.adjust(SettingRow::Voices, false);
        s.adjust(SettingRow::Voices, false);
        assert!((s.voices_volume - 0.4).abs() < 1e-6, "← steps down");
        for _ in 0..20 {
            s.adjust(SettingRow::Voices, true);
        }
        assert_eq!(s.voices_volume, 1.0, "clamped at 1");
        for _ in 0..20 {
            s.adjust(SettingRow::Voices, false);
        }
        assert_eq!(s.voices_volume, 0.0, "clamped at 0");
        assert_eq!(s.music_volume, 0.5, "Music untouched");
        assert_eq!(s.sfx_volume, 0.5, "Sound FX untouched");
        assert!(s.animations, "Animations untouched");
    }

    #[test]
    fn the_voices_row_sits_between_sound_fx_and_animations() {
        fn row_text(frame: &Frame, y: usize) -> String {
            frame.iter().map(|col| col[y].ch).collect()
        }
        for cols in [89, 139] {
            let config = Config { num_cols: cols, num_rows: 31 };
            let settings = Settings { voices_volume: 0.7, ..Settings::default() };
            let state = SettingsState::default();
            let mut frame = crate::frame::new_frame(&config);
            state.draw_overlay(&mut frame, &config, settings, Emphasis::Normal);
            let rows: Vec<String> = (0..31).map(|y| row_text(&frame, y)).collect();
            let find = |s: &str| {
                rows.iter().position(|r| r.contains(s)).unwrap_or_else(|| panic!("a {s} row at {cols}"))
            };
            let (sfx, voices, animations) = (find("Sound FX"), find("Voices"), find("Animations"));
            assert_eq!(voices, sfx + 1, "{cols}: Voices directly below Sound FX");
            assert_eq!(animations, voices + 1, "{cols}: Animations directly below Voices");
            // Its label in the Music row's label column, then a bar and a
            // percentage, like the other volumes.
            let col = |r: &str, s: &str| r[..r.find(s).unwrap()].chars().count();
            let music = &rows[find("Music")];
            let row = &rows[voices];
            assert_eq!(col(row, "Voices"), col(music, "Music"), "{cols}: label columns");
            assert!(row.contains("[███████░░░]"), "{cols}: a bar in {row:?}");
            assert!(row.contains(" 70%"), "{cols}: a percentage in {row:?}");
            // The box is inside the frame: its corners are drawn.
            let content_width = "↑/↓ select  ·  ←/→ change  ·  Esc back".chars().count();
            let layout = OverlayLayout::new(config, content_width, 8);
            assert!(layout.outer.x1 < cols && layout.outer.y1 < 31);
            assert_eq!(frame[layout.outer.x0][layout.outer.y0].ch, '┌');
            assert_eq!(frame[layout.outer.x1][layout.outer.y1].ch, '┘');
        }
    }
}
