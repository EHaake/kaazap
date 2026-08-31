use std::cmp::max;

use crate::{CARD_HEIGHT, CARD_WIDTH, HAND_SIZE, H_PAD, MAX_TABLE_CARDS, V_PAD, config::Config};

// A card slot is a card plus one cell of gap, in each axis.
const CARD_SLOT_W: usize = CARD_WIDTH + 1;
const CARD_SLOT_H: usize = CARD_HEIGHT + 1;

// Vertical bands of the board block, top to bottom. `board_block_height`
// is the single source these — and config's minimum terminal height —
// derive from, so a change here moves the centered block and the minimum
// size together rather than drifting apart.
const HEADER_H: usize = 2; // name / score / rounds
const HAND_H: usize = CARD_HEIGHT + 1; // hand cards + the number-labels row
const STATUS_H: usize = 2; // over-20 alert stacked over the prompt
const BAND_GAP: usize = 1; // one blank row between bands

/// Columns available to one side's cards: from the pad to the divider.
fn side_card_span(cols: usize) -> usize {
    (cols / 2).saturating_sub(H_PAD)
}

/// Cards per row in one side's board grid at this width. At the minimum
/// width this is exactly HAND_SIZE (the hand fits that many by
/// construction of `Config::min_size`); wider terminals fit more.
pub fn grid_cols(cols: usize) -> usize {
    max(1, side_card_span(cols) / CARD_SLOT_W)
}

/// Grid rows needed to show all MAX_TABLE_CARDS slots at this width — the
/// one width-driven number that card placement, ghost placement, block
/// height, and the min-size check all share, so filled cards, empty
/// slots, and the reserved height always agree and reflow together.
pub fn board_grid_rows(cols: usize) -> usize {
    MAX_TABLE_CARDS.div_ceil(grid_cols(cols))
}

/// Height in rows of the grid zone: its card rows plus the gaps between.
fn board_grid_height(cols: usize) -> usize {
    let rows = board_grid_rows(cols);
    rows * CARD_HEIGHT + rows.saturating_sub(1)
}

/// Total height of the fixed board block (header, grid, hand, status, and
/// the gaps between). Non-increasing in width — wider terminals pack more
/// cards per row and so need fewer grid rows — which is what lets config
/// gate the minimum height on the single worst case (minimum width).
pub fn board_block_height(cols: usize) -> usize {
    HEADER_H + BAND_GAP + board_grid_height(cols) + BAND_GAP + HAND_H + BAND_GAP + STATUS_H
}

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
    pub grid: Rect,   // one capped card grid: dealer draws + played cards
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

        // The board is one fixed-height block, centered vertically so it
        // doesn't spread across a tall terminal. Every band position
        // derives from `top` and the shared band constants — the same
        // constants board_block_height sums, so the reserve and the
        // layout can't drift.
        let block_h = board_block_height(cols);
        let top = rows.saturating_sub(block_h) / 2;
        let per_row = grid_cols(cols);
        let grid_h = board_grid_height(cols);

        let y_header = top;
        let y_grid = y_header + HEADER_H + BAND_GAP;
        let y_hand = y_grid + grid_h + BAND_GAP;
        let y_status = y_hand + HAND_H + BAND_GAP;

        // header/hand span the half to the pad; the grid is exactly
        // `per_row` slots wide so cards_per_row of it equals grid_cols.
        let side = |left: usize, right: usize| SideLayout {
            header: Rect::new(left, right, y_header, y_header + HEADER_H - 1),
            grid: Rect::new(left, left + per_row * CARD_SLOT_W - 1, y_grid, y_grid + grid_h - 1),
            hand: Rect::new(left, right, y_hand, y_hand + CARD_HEIGHT - 1),
        };

        let player = side(H_PAD, divider_x.saturating_sub(H_PAD));
        let opponent = side(divider_x + H_PAD, cols.saturating_sub(H_PAD));

        // Two candidate status positions, each two rows (alert over
        // prompt), both inside the reserved block. `status_right` sits to
        // the right of the player's hand on wide terminals; `status_below`
        // spans the block's bottom band. board.rs picks between them so
        // the status never overlaps cards.
        let status_right = Rect::new(H_PAD, divider_x.saturating_sub(2), y_hand + 2, y_hand + 3);
        let status_below =
            Rect::new(H_PAD, cols.saturating_sub(H_PAD), y_status, y_status + STATUS_H - 1);

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
        for r in [side.header, side.grid, side.hand] {
            assert!(in_bounds(r, cols, rows), "region {r:?} out of bounds");
        }
        // The bands stack without overlapping each other
        assert!(vertically_disjoint(side.header, side.grid));
        assert!(vertically_disjoint(side.grid, side.hand));
    }

    #[test]
    fn layout_regions_are_in_bounds_and_stacked_at_several_sizes() {
        // minimum enforced size, a typical size, and an odd one — all now
        // at or above the raised minimum height (~30)
        for (cols, rows) in [(89, 30), (180, 48), (120, 40)] {
            let l = BoardLayout::new(cfg(cols, rows));
            assert_eq!(l.divider_x, cols / 2);
            assert_side_sane(l.player, cols, rows);
            assert_side_sane(l.opponent, cols, rows);
            assert!(in_bounds(l.status_right, cols, rows));
            assert!(in_bounds(l.status_below, cols, rows));
        }
    }

    #[test]
    fn layout_block_is_vertically_centered() {
        let (cols, rows) = (120, 50);
        let l = BoardLayout::new(cfg(cols, rows));
        let top_margin = l.player.header.y0;
        let bottom_margin = rows - (top_margin + board_block_height(cols));
        // centered to within the one row integer division can leave over
        assert!(
            top_margin.abs_diff(bottom_margin) <= 1,
            "block not centered: top {top_margin}, bottom {bottom_margin}"
        );
    }

    #[test]
    fn layout_status_goes_below_when_it_cannot_fit_right() {
        // At the minimum width the hand fills the half, so a normal-
        // length prompt can't sit to the right — it must go below.
        let narrow = BoardLayout::new(cfg(89, 30));
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
    fn layout_grid_holds_twelve_slots_within_the_frame_and_halves() {
        // Every one of the MAX_TABLE_CARDS slot positions lands inside the
        // frame, and each side's cards stay on its side of the divider —
        // reflowing by width (4 per row at the minimum).
        let (min_cols, min_rows) = Config::min_size();
        let l = BoardLayout::new(cfg(min_cols, min_rows));
        let per = grid_cols(min_cols);
        assert_eq!(per, 4);

        for i in 0..MAX_TABLE_CARDS {
            let (x, y) = card_slot(l.player.grid, i % per, i / per);
            assert!(x + CARD_WIDTH <= min_cols && y + CARD_HEIGHT <= min_rows, "player {i} off-frame");
            assert!(x + CARD_WIDTH - 1 < l.divider_x, "player {i} crosses divider");

            let (ox, oy) = card_slot(l.opponent.grid, i % per, i / per);
            assert!(ox > l.divider_x, "opponent {i} not right of divider");
            assert!(ox + CARD_WIDTH <= min_cols && oy + CARD_HEIGHT <= min_rows, "opponent {i} off-frame");
        }
    }

    #[test]
    fn card_slot_wraps_rows_by_column_and_row() {
        let zone = Rect::new(4, 43, 4, 20);
        let per = 4;
        assert_eq!(card_slot(zone, 0 % per, 0 / per), (4, 4)); // first slot
        assert_eq!(card_slot(zone, 3 % per, 3 / per), (4 + 3 * CARD_SLOT_W, 4)); // last on row 0
        assert_eq!(card_slot(zone, 4 % per, 4 / per), (4, 4 + CARD_SLOT_H)); // wraps
    }

    #[test]
    fn grid_cols_is_at_least_one_even_when_absurdly_narrow() {
        assert_eq!(grid_cols(1), 1);
        assert!(grid_cols(0) >= 1);
    }

    #[test]
    fn layout_board_grid_rows_reflow_by_width() {
        let (min_cols, _) = Config::min_size();
        // At the minimum width the grid packs HAND_SIZE (4) per row, so
        // the 12 slots need 3 rows; wider terminals need fewer.
        assert_eq!(grid_cols(min_cols), HAND_SIZE);
        assert_eq!(board_grid_rows(min_cols), 3);
        assert_eq!(board_grid_rows(130), 2); // ~6 per row → 2 rows
        assert_eq!(board_grid_rows(248), 1); // ~12 per row → 1 row
        // Never zero, even absurdly narrow
        assert!(board_grid_rows(1) >= 1);
    }

    #[test]
    fn layout_grid_rows_and_block_height_are_non_increasing_in_width() {
        // Monotonicity is what makes config's single worst-case (minimum
        // width) minimum-height gate sound: no wider terminal ever needs
        // a taller block than the minimum-width one.
        let mut prev_rows = usize::MAX;
        let mut prev_h = usize::MAX;
        for cols in (10..400).step_by(1) {
            let r = board_grid_rows(cols);
            let h = board_block_height(cols);
            assert!(r >= 1);
            assert!(r <= prev_rows, "grid rows grew with width at cols={cols}");
            assert!(h <= prev_h, "block height grew with width at cols={cols}");
            prev_rows = r;
            prev_h = h;
        }
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
