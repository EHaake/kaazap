use crate::{config::Config, frame::{BorderWeight, Emphasis, Frame, draw_box, draw_text}, layout::OverlayLayout};

#[derive(Debug, Copy, Clone)]
pub enum OverlayKind {
    GameHelp,
    MenuHelp,
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
        }
    }

    /// Draw already-loaded content into the overlay's inner box
    ///
    fn add_content(&self, content: &[String], layout: OverlayLayout, frame: &mut Frame) {
        let x = layout.inner.x0;
        let y = layout.inner.y0;

        for (i, line) in content.iter().enumerate() {
            draw_text(frame, x, y + i, line, Emphasis::Normal);
        }
    }

    /// Clear any existing chars from the overlay box
    ///
    fn clear_overlay_box(&self, layout: OverlayLayout, frame: &mut Frame) {
        // get box corners
        let x0 = layout.outer.x0;
        let x1 = layout.outer.x1;
        let y0 = layout.outer.y0;
        let y1 = layout.outer.y1;

        (x0..=x1).for_each(|x| {
            (y0..=y1).for_each(|y| {
                frame[x][y].ch = ' ';
            });
        });
    }

    /// Draw border helper
    ///
    fn draw_border(&self, layout: OverlayLayout, frame: &mut Frame) {
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
    }

    /// Size the box to the content, then draw box and text
    ///
    fn draw_overlay(&self, content: &[String], frame: &mut Frame) {
        let (width, height) = measure(content);
        let layout = OverlayLayout::new(self.config, width, height);

        self.clear_overlay_box(layout, frame);
        self.draw_border(layout, frame);
        self.add_content(content, layout, frame);
    }

    pub fn draw(&self, frame: &mut Frame) {
        // The box sizes itself to whatever text the overlay carries — no
        // per-kind width/height constants to keep in sync with the files
        let content = self.read_text_from_file();
        self.draw_overlay(&content, frame);
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
