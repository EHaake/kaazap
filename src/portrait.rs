//! Opponent portraits: a fixed-size monochrome block-art face — the portrait
//! counterpart to card.rs's CardView. Pure rendering + dimensions, with no
//! dependency on opponent.rs (the art strings live with the profiles). Spec 016.

use crate::frame::{Emphasis, Frame, draw_text};

/// The fixed art-block size every portrait is authored to: PORTRAIT_HEIGHT
/// lines of at most PORTRAIT_WIDTH cells, one glyph per cell.
pub const PORTRAIT_WIDTH: usize = 18;
pub const PORTRAIT_HEIGHT: usize = 12;

/// Draw a portrait's art with its top-left at (x, y): each line of `art` is
/// drawn left-to-right at (x, y + row) via `draw_text`. Clip-safe — every cell
/// goes through `draw_char`'s bounds guard, so an off-frame position (or a
/// portrait overrunning an edge) drops the out-of-range cells and never panics,
/// mirroring `CardView::draw`. `art` is a plain multi-line string.
pub fn draw_portrait(frame: &mut Frame, x: usize, y: usize, art: &str, emphasis: Emphasis) {
    for (row, line) in art.lines().enumerate() {
        draw_text(frame, x, y + row, line, emphasis);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Cell;

    fn blank(w: usize, h: usize) -> Frame {
        vec![vec![Cell::default(); h]; w]
    }

    #[test]
    fn art_rows_land_top_to_bottom_left_to_right() {
        let mut f = blank(4, 4);
        draw_portrait(&mut f, 0, 0, "ab\ncd", Emphasis::Normal);
        assert_eq!(f[0][0].ch, 'a');
        assert_eq!(f[1][0].ch, 'b');
        assert_eq!(f[0][1].ch, 'c');
        assert_eq!(f[1][1].ch, 'd');
    }

    #[test]
    fn draw_portrait_is_clip_safe_off_frame() {
        let (w, h) = (4usize, 4usize);

        // x past width — nothing lands, no panic
        let mut f = blank(w, h);
        draw_portrait(&mut f, 99, 0, "ab\ncd", Emphasis::Normal);

        // y past height — nothing lands, no panic
        let mut f = blank(w, h);
        draw_portrait(&mut f, 0, 99, "ab\ncd", Emphasis::Normal);

        // empty frame — no panic
        let mut f: Frame = Vec::new();
        draw_portrait(&mut f, 0, 0, "ab\ncd", Emphasis::Normal);

        // overrunning the bottom edge: at y = h-1 only the first row fits
        let mut f = blank(w, h);
        draw_portrait(&mut f, 0, h - 1, "a\nb\nc", Emphasis::Normal);
        assert_eq!(f[0][h - 1].ch, 'a'); // fitting row landed
    }
}
