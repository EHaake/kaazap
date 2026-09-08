//! Opponent portraits: a fixed-size monochrome block-art face — the portrait
//! counterpart to card.rs's CardView. Pure rendering + dimensions, with no
//! dependency on opponent.rs (the art strings live with the profiles). Spec 016.

use crate::frame::{Align, BorderWeight, Emphasis, Frame, draw_box, draw_text, draw_text_in};
use crate::layout::Rect;

/// The fixed art-block size every portrait is authored to: PORTRAIT_HEIGHT
/// lines of at most PORTRAIT_WIDTH cells, one glyph per cell.
pub const PORTRAIT_WIDTH: usize = 18;
pub const PORTRAIT_HEIGHT: usize = 12;

/// Presence-panel geometry, derived from the portrait block so the in-match
/// panel and the game's minimum terminal width move together if the portrait
/// is ever resized. PANEL_H_INMATCH reserves rows below the portrait for the
/// later banter line / round pips (spec 016 reserves the space, doesn't build it).
const PANEL_PAD_X: usize = 1;
pub const PANEL_GAP: usize = 3;
pub const PANEL_W: usize = PORTRAIT_WIDTH + 2 * PANEL_PAD_X + 2; // 22: portrait + interior pad + border
pub const PANEL_H_INMATCH: usize = 2 + 1 + PORTRAIT_HEIGHT + 1 + 2; // 18: border, name, portrait, gap, reserved

/// The panel interior width a banter line must fit within, and the number of
/// round pips (first-to-3, matching `ROUND_PIPS` uses in game.rs). Used by the
/// banter fit test (`banter.rs`) and the in-match panel-extras drawer (T003).
pub const BANTER_MAX_WIDTH: usize = PANEL_W - 2; // 20: interior width lines must fit
#[allow(dead_code)] // consumed by draw_presence_extras (T003)
const ROUND_PIPS: usize = 3; // first-to-3

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

/// A bordered opponent-presence panel: a single-weight box with the opponent
/// `name` (Strong) centered on the top interior row and the portrait centered
/// below it. Any panel height beyond that is left blank — the reserved space
/// for the later banter line / round pips (spec 016 reserves, doesn't build).
/// One drawer, three callers (opponent-select preview, in-match panel, campaign
/// rail); each sizes the `panel` Rect. Clip-safe (delegates to clip-safe drawers).
pub fn draw_presence_panel(frame: &mut Frame, panel: Rect, name: &str, art: &str) {
    draw_box(frame, panel, BorderWeight::Single, Emphasis::Normal);
    let interior = Rect::new(panel.x0 + 1, panel.x1 - 1, panel.y0 + 1, panel.y1 - 1);
    // Name on the top interior row, centered, Strong.
    draw_text_in(frame, interior, 0, Align::Center, name, Emphasis::Strong);
    // Portrait centered horizontally, on the row just below the name.
    let px = interior.x0 + interior.width().saturating_sub(PORTRAIT_WIDTH) / 2;
    draw_portrait(frame, px, panel.y0 + 2, art, Emphasis::Normal);
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
