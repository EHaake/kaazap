use std::cmp::max;

use crate::{CARD_HEIGHT, CARD_WIDTH, H_PAD, V_PAD, config::Config};

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
    pub status: Rect, // player-half strip owning one status message
}

impl BoardLayout {
    pub fn new(config: Config) -> Self {
        let cols = config.num_cols;
        let rows = config.num_rows;
        let divider_x = cols / 2;

        // Vertical bands, top to bottom — same positions the board has
        // always used, now named instead of inlined as magic numbers.
        let dealer_y = 4;
        let hand_y = rows.saturating_sub(CARD_HEIGHT + 2);
        let played_y = hand_y.saturating_sub(CARD_HEIGHT + 1);
        let status_y = rows.saturating_sub(5);

        let side = |left: usize, right: usize| SideLayout {
            header: Rect::new(left, right, 1, 2),
            dealer: Rect::new(left, right, dealer_y, played_y.saturating_sub(1)),
            played: Rect::new(left, right, played_y, played_y + CARD_HEIGHT - 1),
            hand: Rect::new(left, right, hand_y, hand_y + CARD_HEIGHT - 1),
        };

        let player = side(H_PAD, divider_x.saturating_sub(H_PAD));
        let opponent = side(divider_x + H_PAD, cols.saturating_sub(H_PAD));

        // The status message shares the hand's vertical band but sits to
        // the right of the cards, on the player half (as it always has).
        let status = Rect::new(H_PAD, divider_x.saturating_sub(1), status_y, status_y);

        Self {
            divider_x,
            player,
            opponent,
            status,
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
        for (cols, rows) in [(67, 24), (180, 48), (91, 33)] {
            let l = BoardLayout::new(cfg(cols, rows));
            assert_eq!(l.divider_x, cols / 2);
            assert_side_sane(l.player, cols, rows);
            assert_side_sane(l.opponent, cols, rows);
            assert!(in_bounds(l.status, cols, rows));
        }
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
