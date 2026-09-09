use crate::{config::Config, frame::{Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text_in}, layout::OverlayLayout};

#[derive(Debug, Copy, Clone)]
pub enum OverlayKind {
    GameHelp,
    MenuHelp,
    HowToPlay,
}

#[derive(Debug)]
pub struct Overlay {
    overlay_kind: OverlayKind,
    config: Config,
}

impl Overlay {
    pub fn new(overlay_kind: OverlayKind, config: Config) -> Self {
        Self {
            overlay_kind,
            config,
        }
    }

    /// This overlay's kind — used to rebuild it on a terminal resize.
    pub fn kind(&self) -> OverlayKind {
        self.overlay_kind
    }

    /// Open a text file and read it into a Vec<String> based on OverlayKind
    ///
    fn read_text_from_file(&self) -> Vec<String> {
        match self.overlay_kind {
            OverlayKind::GameHelp => {
                let s: &'static str = include_str!("../assets/game_overlay_text.txt");
                s.lines().map(|line| line.to_string()).collect()
            }
            OverlayKind::MenuHelp => {
                let s: &'static str = include_str!("../assets/menu_overlay_text.txt");
                s.lines().map(|line| line.to_string()).collect()
            }
            OverlayKind::HowToPlay => {
                let s: &'static str = include_str!("../assets/how_to_play_text.txt");
                s.lines().map(|line| line.to_string()).collect()
            }
        }
    }

    /// Size the box to the content, then draw box and text
    ///
    fn draw_overlay(&self, content: &[String], frame: &mut Frame) {
        draw_text_overlay(self.config, content, frame);
    }

    pub fn draw(&self, frame: &mut Frame) {
        // The box sizes itself to whatever text the overlay carries — no
        // per-kind width/height constants to keep in sync with the files
        let content = self.read_text_from_file();
        self.draw_overlay(&content, frame);
    }
}

/// Size a box to `content` and draw it into `frame`: measure the text,
/// build the layout, clear the region, draw the border, then draw the
/// text — the first line centered as a title, the rest left-aligned so
/// lists and columns stay lined up. Exposed as a free function so
/// dynamic content (e.g. the play log) can render through the same box
/// machinery without going through `Overlay`.
pub fn draw_text_overlay(config: Config, content: &[String], frame: &mut Frame) {
    let (width, height) = measure(content);
    let layout = OverlayLayout::new(config, width, height);

    clear_rect(frame, layout.outer);
    draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
    for (i, line) in content.iter().enumerate() {
        if i == 0 {
            draw_text_in(frame, layout.inner, 0, Align::Center, line.trim(), Emphasis::Normal);
        } else {
            draw_text_in(frame, layout.inner, i, Align::Left, line, Emphasis::Normal);
        }
    }
}

/// Content dimensions of an overlay's text: widest line (in chars) and
/// number of lines.
fn measure(content: &[String]) -> (usize, usize) {
    let width = content.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    (width, content.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_measure_uses_widest_line_and_line_count() {
        let content = vec![
            "short".to_string(),
            "a much longer line".to_string(),
            "mid".to_string(),
        ];
        assert_eq!(measure(&content), (18, 3));
    }

    #[test]
    fn overlay_measure_counts_chars_not_bytes() {
        // "±" is multi-byte; width must be char count (3), not byte len
        let content = vec!["±1T".to_string()];
        assert_eq!(measure(&content), (3, 1));
    }

    #[test]
    fn overlay_measure_of_empty_content_is_zero() {
        assert_eq!(measure(&[]), (0, 0));
    }
}
