use crate::{config::Config, frame::Frame, layout::OverlayLayout};

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

    /// Draw Text Helper
    ///
    fn draw_text(&self, text: &str, x: usize, y: usize, frame: &mut Frame) {
        for (i, ch) in text.chars().enumerate() {
            frame[x + i][y] = ch;
        }
    }

    /// Read content from text file and call helper to draw it into the overlay
    ///
    fn add_content(&self, layout: OverlayLayout, frame: &mut Frame) {
        let content = self.read_text_from_file();

        // get inner box corners
        let x = layout.inner.x0;
        let y = layout.inner.y0;

        for (i, line) in content.iter().enumerate() {
            self.draw_text(line, x, y + i, frame);
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
                frame[x][y] = ' ';
            });
        });
    }

    /// Draw border helper
    ///
    fn draw_border(&self, layout: OverlayLayout, frame: &mut Frame) {
        // get box corners
        let x0 = layout.outer.x0;
        let x1 = layout.outer.x1;
        let y0 = layout.outer.y0;
        let y1 = layout.outer.y1;

        // borders
        (x0..=x1).for_each(|x| {
            frame[x][y0] = '-';
            frame[x][y1] = '-';
        });

        (y0..=y1).for_each(|y| {
            frame[x0][y] = '|';
            frame[x1][y] = '|';
        });

        // corners
        frame[x0][y0] = '+';
        frame[x1][y0] = '+';
        frame[x0][y1] = '+';
        frame[x1][y1] = '+';
    }

    /// Take the content size and call the functions necessary to draw the overlay
    ///
    fn draw_overlay(&self, content_width: usize, content_height: usize, frame: &mut Frame) {
        // Compute the layout based on content width and config
        let layout = OverlayLayout::new(self.config, content_width, content_height);
        // Draw spaces inside of entire box
        self.clear_overlay_box(layout, frame);
        // Draw the borders
        self.draw_border(layout, frame);
        // Draw the text content
        self.add_content(layout, frame);
    }

    // TODO: Stop using magic numbers of content size
    pub fn draw(&self, frame: &mut Frame) {
        match self.overlay_kind {
            OverlayKind::GameHelp => {
                let content_width = 42;
                let content_height = 8; // fits the 12 lines of game_overlay_text.txt

                self.draw_overlay(content_width, content_height, frame);
            }
            OverlayKind::MenuHelp => {
                let content_width = 32;
                let content_height = 4;

                self.draw_overlay(content_width, content_height, frame);
            }
        }
    }
}
