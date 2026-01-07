use crate::{H_PAD, V_PAD, config::Config};

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
}

impl Rect {
    pub fn new(x0: usize, x1: usize, y0: usize, y1: usize) -> Self {
        Self { x0, x1, y0, y1 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OverlayLayout {
    outer: Rect,
    inner: Rect,
}

impl OverlayLayout {
    pub fn new(config: Config, content_width: usize, content_height: usize) -> Self {
        let mid_x = config.num_cols / 2;
        let mid_y = config.num_rows / 2;

        // Compute box dimensions
        let box_width = content_width + 2 * H_PAD;
        let box_height = content_height + 2 * V_PAD;

        // get box corners
        let mut x0 = mid_x - box_width / 2;
        let mut y0 = mid_y - box_height / 2;
        let mut x1 = mid_x + box_width / 2;
        let mut y1 = mid_y + box_height / 2;

        let outer = Rect::new(x0, x1, y0, y1);
        
        // Inner box dimensions
        x0 = mid_x - content_width;
        y0 = mid_y - content_height;
        x1 = mid_x + content_width;
        y1 = mid_y + content_height;

        let inner = Rect::new(x0, x1, y0, y1);

        Self { outer, inner }
    }
}
