use crate::{H_PAD, V_PAD, config::Config, frame::Frame};

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

    fn add_content(&mut self) {}

    pub fn draw(&self, frame: &mut Frame) {
        let mid_x = self.config.num_cols / 2;
        let mid_y = self.config.num_rows / 2;
        
        // draw a vertical divider down the middle
        let mid = self.config.num_cols / 2;
        for y in 0..self.config.num_rows {
            if mid < frame.len() && y < frame[0].len() {
                frame[mid][y] = '|';
            }
        }
        // End draw vertical divider

        self.draw_border(mid_x, mid_y, frame);
    }

    /// Draw Text Helper
    ///
    fn draw_text(&self, text: &str, x: usize, y: usize, frame: &mut Frame) {
        for (i, ch) in text.chars().enumerate() {
            frame[x + i][y] = ch;
        }
    }

    /// Draw border helper
    ///
    fn draw_border(&self, mid_x: usize, mid_y: usize, frame: &mut Frame) {
        let content_width = 10;
        let content_height = 5;

        // Compute box dimensions
        let box_width = content_width + 2 * H_PAD + 2;
        let box_height = content_height + 2 * V_PAD + 2;

        // get box corners
        let x0 = mid_x - box_width / 2;
        let y0 = mid_y - box_height / 2;
        let x1 = mid_x + box_width / 2;
        let y1 = mid_y + box_height / 2;

        // borders
        for x in x0..=x1 {
            frame[x][y0] = '-';
            frame[x][y1] = '-';
        }

        for y in y0..=y1 {
            frame[x0][y] = '|';
            frame[x1][y] = '|';
        }
        
        // corners
        frame[x0][y0] = '+';
        frame[x1][y0] = '+';
        frame[x0][y1] = '+';
        frame[x1][y1] = '+';


    }
}
