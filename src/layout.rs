use std::cmp::max;

use crate::{CARD_HEIGHT, CARD_WIDTH, HAND_SIZE, H_PAD, V_PAD, config::Config};

// A card slot is a card plus one cell of gap, in each axis.
const CARD_SLOT_W: usize = CARD_WIDTH + 1;
const CARD_SLOT_H: usize = CARD_HEIGHT + 1;

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

    pub fn width(&self) -> usize {
        self.x1.saturating_sub(self.x0) + 1
    }
}

/// One player's half of the board: the header strip and the three card
/// zones. Rects are inclusive, in frame [x][y] coordinates.
#[derive(Debug, Copy, Clone)]
pub struct SideLayout {
    pub header: Rect, // name / score / rounds
    pub dealer: Rect, // dealer-draw grid (wraps into rows)
    pub played: Rect, // side cards played (single row)
    pub hand: Rect,   // hand (single row; numbers drawn just below)
}

/// The whole game board's geometry, a pure function of terminal size —
/// the single source of truth board.rs draws against.
#[derive(Debug, Copy, Clone)]
pub struct BoardLayout {
    pub divider_x: usize,
    pub player: SideLayout,
    pub opponent: SideLayout,
    pub status_right: Rect, // to the right of the hand (wide terminals)
    pub status_below: Rect, // under the board (narrow terminals)
}

impl BoardLayout {
    pub fn new(config: Config) -> Self {
        let cols = config.num_cols;
        let rows = config.num_rows;
        let divider_x = cols / 2;

        // Vertical bands, top to bottom. The hand sits one row higher
        // than the very bottom so two rows are free beneath it for the
        // "below" status position (used when the terminal is too narrow
        // to fit the status to the right of the hand).
        let dealer_y = 4;
        let hand_y = rows.saturating_sub(CARD_HEIGHT + 3);
        let played_y = hand_y.saturating_sub(CARD_HEIGHT + 1);

        let side = |left: usize, right: usize| SideLayout {
            header: Rect::new(left, right, 1, 2),
            dealer: Rect::new(left, right, dealer_y, played_y.saturating_sub(1)),
            played: Rect::new(left, right, played_y, played_y + CARD_HEIGHT - 1),
            hand: Rect::new(left, right, hand_y, hand_y + CARD_HEIGHT - 1),
        };

        let player = side(H_PAD, divider_x.saturating_sub(H_PAD));
        let opponent = side(divider_x + H_PAD, cols.saturating_sub(H_PAD));

        // Two candidate status positions, each two rows (alert over
        // prompt). `status_right` sits to the right of the hand on the
        // player half; `status_below` spans the bottom, under everything.
        // board.rs picks between them so the status never overlaps cards.
        let status_right = Rect::new(
            H_PAD,
            divider_x.saturating_sub(2),
            hand_y + 2,
            hand_y + 3,
        );
        let status_below = Rect::new(
            H_PAD,
            cols.saturating_sub(H_PAD),
            rows.saturating_sub(2),
            rows.saturating_sub(1),
        );

        Self {
            divider_x,
            player,
            opponent,
            status_right,
            status_below,
        }
    }

    /// Right edge (column) of the last hand slot's card.
    fn hand_cards_right(&self) -> usize {
        self.player.hand.x0 + (HAND_SIZE - 1) * (CARD_WIDTH + 1) + CARD_WIDTH - 1
    }

    /// Can a status line of `text_len` chars sit to the right of the
    /// hand (right-aligned near the divider) without overlapping the
    /// cards? If not, the caller uses `status_below` instead.
    pub fn status_fits_right(&self, text_len: usize) -> bool {
        if text_len == 0 {
            return true;
        }
        let left_edge = self.status_right.x1.saturating_sub(text_len.saturating_sub(1));
        left_edge > self.hand_cards_right() + 1
    }
}

/// Start-menu geometry from terminal size and the title art's height.
#[derive(Debug, Copy, Clone)]
pub struct MenuLayout {
    pub center_x: usize,
    pub title_top: usize,
    pub items_top: usize,
    pub item_spacing: usize,
}

impl MenuLayout {
    pub fn new(config: Config, title_height: usize) -> Self {
        let title_top = 5;
        // Gap below the title art, then the menu items — no longer the
        // scattered y+15/+2 magic the menu used to inline.
        const TITLE_GAP: usize = 3;
        Self {
            center_x: config.num_cols / 2,
            title_top,
            items_top: title_top + title_height + TITLE_GAP,
            item_spacing: 2,
        }
    }
}

/// How many cards fit across a zone before wrapping to the next row.
pub fn cards_per_row(zone: Rect) -> usize {
    max(1, zone.width() / CARD_SLOT_W)
}

/// Top-left (x, y) of the card at grid position (col, row) within a zone.
/// Callers pass col/row directly: dealer wraps (col = i % per_row), the
/// played/hand rows don't (col = i, row = 0).
pub fn card_slot(zone: Rect, col: usize, row: usize) -> (usize, usize) {
    (zone.x0 + col * CARD_SLOT_W, zone.y0 + row * CARD_SLOT_H)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(cols: usize, rows: usize) -> Config {
        Config {
            num_cols: cols,
            num_rows: rows,
        }
    }

    fn in_bounds(r: Rect, cols: usize, rows: usize) -> bool {
        r.x0 <= r.x1 && r.y0 <= r.y1 && r.x1 < cols && r.y1 < rows
    }

    fn vertically_disjoint(a: Rect, b: Rect) -> bool {
        a.y1 < b.y0 || b.y1 < a.y0
    }

    fn assert_side_sane(side: SideLayout, cols: usize, rows: usize) {
        for r in [side.header, side.dealer, side.played, side.hand] {
            assert!(in_bounds(r, cols, rows), "region {r:?} out of bounds");
        }
        // The card zones stack without overlapping each other
        assert!(vertically_disjoint(side.header, side.dealer));
        assert!(vertically_disjoint(side.dealer, side.played));
        assert!(vertically_disjoint(side.played, side.hand));
    }

    #[test]
    fn layout_regions_are_in_bounds_and_stacked_at_several_sizes() {
        // minimum enforced size, a typical size, and an odd one
        for (cols, rows) in [(89, 24), (180, 48), (91, 33)] {
            let l = BoardLayout::new(cfg(cols, rows));
            assert_eq!(l.divider_x, cols / 2);
            assert_side_sane(l.player, cols, rows);
            assert_side_sane(l.opponent, cols, rows);
            assert!(in_bounds(l.status_right, cols, rows));
            assert!(in_bounds(l.status_below, cols, rows));
        }
    }

    #[test]
    fn layout_status_goes_below_when_it_cannot_fit_right() {
        // At the minimum width the hand fills the half, so a normal-
        // length prompt can't sit to the right — it must go below.
        let narrow = BoardLayout::new(cfg(89, 24));
        assert!(!narrow.status_fits_right(30));

        // A very wide terminal leaves room to the right for it.
        let wide = BoardLayout::new(cfg(240, 48));
        assert!(wide.status_fits_right(30));
    }

    #[test]
    fn layout_halves_do_not_cross_the_divider() {
        let l = BoardLayout::new(cfg(180, 48));
        assert!(l.player.hand.x1 < l.divider_x);
        assert!(l.opponent.hand.x0 > l.divider_x);
    }

    #[test]
    fn card_slot_wraps_rows_by_cards_per_row() {
        let zone = Rect::new(4, 43, 4, 20); // width 40 → 4 slots per row
        assert_eq!(cards_per_row(zone), 4);

        let per = cards_per_row(zone);
        // slot 0 at the origin
        assert_eq!(card_slot(zone, 0 % per, 0 / per), (4, 4));
        // slot 3 is the last on row 0
        assert_eq!(card_slot(zone, 3 % per, 3 / per), (4 + 3 * CARD_SLOT_W, 4));
        // slot 4 wraps to the next row
        assert_eq!(card_slot(zone, 4 % per, 4 / per), (4, 4 + CARD_SLOT_H));
    }

    #[test]
    fn cards_per_row_is_at_least_one_even_when_narrow() {
        let zone = Rect::new(0, 3, 0, 0); // narrower than a card
        assert_eq!(cards_per_row(zone), 1);
    }

    #[test]
    fn overlay_layout_stays_in_bounds_even_when_content_exceeds_the_frame() {
        // A box taller/wider than the terminal must clamp, never underflow
        // or run off-screen (the min-terminal overlay panic the review
        // flagged). Try content bigger than the frame in both axes.
        for (cols, rows) in [(89, 24), (67, 20), (30, 10)] {
            let cfg = Config { num_cols: cols, num_rows: rows };
            for (cw, ch) in [(20, 5), (200, 200), (0, 0)] {
                let l = OverlayLayout::new(cfg, cw, ch);
                for r in [l.outer, l.inner] {
                    assert!(r.x0 <= r.x1 && r.y0 <= r.y1, "inverted rect {r:?}");
                    assert!(r.x1 < cols && r.y1 < rows, "out of bounds {r:?}");
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OverlayLayout {
    pub outer: Rect,
    pub inner: Rect,
}

impl OverlayLayout {
    pub fn new(config: Config, content_width: usize, content_height: usize) -> Self {
        let cols = config.num_cols;
        let rows = config.num_rows;
        let mid_x = cols / 2;
        let mid_y = rows / 2;

        // Box sized to content + padding, but never larger than the frame
        // — an overlay taller/wider than the terminal is clamped rather
        // than letting the centering math underflow (panic) or the box
        // run off-screen. Positions saturate for the same reason.
        let box_width = (content_width + 2 * H_PAD).min(cols);
        let box_height = (content_height + 2 * V_PAD).min(rows);

        let x0 = mid_x.saturating_sub(box_width / 2);
        let y0 = mid_y.saturating_sub(box_height / 2);
        let x1 = (x0 + box_width.saturating_sub(1)).min(cols.saturating_sub(1));
        let y1 = (y0 + box_height.saturating_sub(1)).min(rows.saturating_sub(1));

        let outer = Rect::new(x0, x1, y0, y1);

        // Inner box: shrink by half the padding, clamped so it never
        // inverts (x0 > x1) on a box squeezed down to the frame.
        let inner = Rect::new(
            (x0 + H_PAD / 2).min(x1),
            x1.saturating_sub(H_PAD / 2),
            (y0 + V_PAD / 2).min(y1),
            y1.saturating_sub(V_PAD / 2),
        );

        Self { outer, inner }
    }
}
