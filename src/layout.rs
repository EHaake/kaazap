use crate::{H_PAD, V_PAD, config::Config};

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x0: usize,
    pub x1: usize,
    pub y0: usize,
    pub y1: usize,
}

impl Rect {
    pub fn new(x0: usize, x1: usize, y0: usize, y1: usize) -> Self {
        Self { x0, x1, y0, y1 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OverlayLayout {
    pub outer: Rect,
    pub inner: Rect,
}

impl OverlayLayout {
    pub fn new(config: Config, content_width: usize, content_height: usize) -> Self {
        let mid_x = config.num_cols / 2;
        let mid_y = config.num_rows / 2;

        // Compute outer box dimensions
        let box_width = content_width + 2 * H_PAD;
        let box_height = content_height + 2 * V_PAD;

        // get box corners
        let mut x0 = mid_x - box_width / 2;
        let mut y0 = mid_y - box_height / 2;
        let mut x1 = mid_x + box_width / 2;
        let mut y1 = mid_y + box_height / 2;

        let outer = Rect::new(x0, x1, y0, y1);
        
        // Inner box dimensions
        x0 += H_PAD / 2;
        y0 += V_PAD / 2;
        x1 -= H_PAD / 2;
        y1 -= V_PAD / 2;

        let inner = Rect::new(x0, x1, y0, y1);

        Self { outer, inner }
    }
}
