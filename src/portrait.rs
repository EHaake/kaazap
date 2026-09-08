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

/// Draw the in-match-only presence extras over an already-drawn presence panel:
/// the opponent `banter` line (if `Some`) centered on interior row 14, and the
/// opponent's round-win pips on interior row 15. Computes the interior exactly
/// as `draw_presence_panel` does. `opponent_rounds_won` filled glyphs (Strong)
/// followed by the remaining empty glyphs (Muted), the whole run centered. This
/// is a separate drawer called only by the in-match board, so the two preview
/// callers of `draw_presence_panel` keep showing name + portrait only.
/// Clip-safe (delegates to clip-safe `draw_text`/`draw_text_in`).
pub fn draw_presence_extras(
    frame: &mut Frame,
    panel: Rect,
    banter: Option<&str>,
    opponent_rounds_won: usize,
) {
    let interior = Rect::new(panel.x0 + 1, panel.x1 - 1, panel.y0 + 1, panel.y1 - 1);

    // Banter on interior row 14, centered — draw_text_in clips to the interior
    // width so an over-long line can never overrun the border.
    if let Some(line) = banter {
        draw_text_in(frame, interior, 14, Align::Center, line, Emphasis::Normal);
    }

    // Pips on interior row 15: `filled` filled glyphs then the rest empty, drawn
    // one glyph at a time at stride 2 (a single blank cell between each) so the
    // row reads `● ○ ○` rather than cramped. The whole span is
    // `ROUND_PIPS * 2 - 1` cells wide, centered in the interior.
    let filled = opponent_rounds_won.min(ROUND_PIPS);
    let span = ROUND_PIPS * 2 - 1;
    let start_x = interior.x0 + interior.width().saturating_sub(span) / 2;
    let pip_y = interior.y0 + 15;
    for i in 0..ROUND_PIPS {
        let (glyph, emphasis) = if i < filled {
            ('●', Emphasis::Strong)
        } else {
            ('○', Emphasis::Muted)
        };
        draw_text(frame, start_x + i * 2, pip_y, &glyph.to_string(), emphasis);
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

    // A full-size in-match panel anchored at the frame origin, and the frame to
    // draw it into. Interior spans x 1..=PANEL_W-2, y 1..=PANEL_H_INMATCH-2;
    // banter lands on interior row 14 (y = 15), pips on interior row 15 (y = 16).
    fn inmatch_panel() -> (Frame, Rect) {
        let f = blank(PANEL_W, PANEL_H_INMATCH);
        let panel = Rect::new(0, PANEL_W - 1, 0, PANEL_H_INMATCH - 1);
        (f, panel)
    }

    #[test]
    fn draw_presence_extras_is_clip_safe_off_frame() {
        // panel entirely past the right edge — nothing lands, no panic
        let mut f = blank(4, 4);
        let panel = Rect::new(99, 99 + PANEL_W - 1, 0, PANEL_H_INMATCH - 1);
        draw_presence_extras(&mut f, panel, Some("hello"), 2);

        // panel past the bottom edge — no panic
        let mut f = blank(4, 4);
        let panel = Rect::new(0, PANEL_W - 1, 99, 99 + PANEL_H_INMATCH - 1);
        draw_presence_extras(&mut f, panel, Some("hello"), 2);

        // empty frame — no panic
        let mut f: Frame = Vec::new();
        let panel = Rect::new(0, PANEL_W - 1, 0, PANEL_H_INMATCH - 1);
        draw_presence_extras(&mut f, panel, Some("hello"), 3);
    }

    #[test]
    fn pip_row_carries_exactly_round_pips_markers() {
        for rounds_won in 0..=ROUND_PIPS {
            let (mut f, panel) = inmatch_panel();
            draw_presence_extras(&mut f, panel, None, rounds_won);

            let pip_y = 16; // interior.y0 (1) + 15
            let mut filled = 0;
            let mut empty = 0;
            for col in f.iter() {
                match col[pip_y].ch {
                    '●' => filled += 1,
                    '○' => empty += 1,
                    _ => {}
                }
            }
            assert_eq!(filled, rounds_won, "filled pips for rounds_won={rounds_won}");
            assert_eq!(empty, ROUND_PIPS - rounds_won, "empty pips for rounds_won={rounds_won}");
            assert_eq!(filled + empty, ROUND_PIPS, "total pips for rounds_won={rounds_won}");
        }
    }

    #[test]
    fn rounds_won_over_round_pips_is_clamped() {
        let (mut f, panel) = inmatch_panel();
        draw_presence_extras(&mut f, panel, None, 99);
        let pip_y = 16;
        let filled = f.iter().filter(|col| col[pip_y].ch == '●').count();
        let empty = f.iter().filter(|col| col[pip_y].ch == '○').count();
        assert_eq!(filled, ROUND_PIPS);
        assert_eq!(empty, 0);
    }

    #[test]
    fn pip_glyphs_are_spaced_one_blank_cell_apart() {
        let (mut f, panel) = inmatch_panel();
        draw_presence_extras(&mut f, panel, None, 1);

        let pip_y = 16;
        // Collect the columns carrying a pip glyph, left to right.
        let cols: Vec<usize> = f
            .iter()
            .enumerate()
            .filter(|(_, col)| matches!(col[pip_y].ch, '●' | '○'))
            .map(|(x, _)| x)
            .collect();

        assert_eq!(cols.len(), ROUND_PIPS, "one glyph per pip");
        // Consecutive glyphs sit two cells apart, so the cell between them is
        // never itself a pip glyph.
        for pair in cols.windows(2) {
            assert_eq!(pair[1] - pair[0], 2, "glyphs at stride 2");
            let between = pair[0] + 1;
            assert!(
                !matches!(f[between][pip_y].ch, '●' | '○'),
                "cell {between} between glyphs must not be a pip glyph"
            );
        }
    }

    #[test]
    fn banter_fits_inside_interior_and_clips_at_the_border() {
        let banter_y = 15; // interior.y0 (1) + 14
        let left_border = 0; // panel.x0
        let right_border = PANEL_W - 1; // panel.x1

        // A line exactly BANTER_MAX_WIDTH long lands fully inside the interior.
        let fit: String = std::iter::repeat('x').take(BANTER_MAX_WIDTH).collect();
        let (mut f, panel) = inmatch_panel();
        draw_presence_extras(&mut f, panel, Some(&fit), 0);
        for x in 1..=(PANEL_W - 2) {
            assert_eq!(f[x][banter_y].ch, 'x', "interior col {x} should carry banter");
        }
        assert_ne!(f[left_border][banter_y].ch, 'x', "left border untouched");
        assert_ne!(f[right_border][banter_y].ch, 'x', "right border untouched");

        // A line longer than the interior is clipped: nothing on or past the border.
        let long: String = std::iter::repeat('y').take(BANTER_MAX_WIDTH + 10).collect();
        let (mut f, panel) = inmatch_panel();
        draw_presence_extras(&mut f, panel, Some(&long), 0);
        assert_ne!(f[left_border][banter_y].ch, 'y', "left border untouched");
        assert_ne!(f[right_border][banter_y].ch, 'y', "right border untouched");
    }
}
