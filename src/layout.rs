use crate::{CARD_HEIGHT, CARD_WIDTH, HAND_SIZE, H_PAD, MAX_TABLE_CARDS, V_PAD, config::Config};

// A card slot is a card plus one cell of gap, in each axis.
const CARD_SLOT_W: usize = CARD_WIDTH + 1;
const CARD_SLOT_H: usize = CARD_HEIGHT + 1;

// Vertical bands of the board block, top to bottom. BOARD_BLOCK_HEIGHT
// sums these, and config's minimum terminal height is exactly that — so a
// change here moves the centered block and the minimum size together.
const HEADER_H: usize = 2; // name / score / rounds
const HAND_H: usize = CARD_HEIGHT + 1; // hand cards + the number-labels row
const STATUS_H: usize = 2; // over-20 alert stacked over the prompt
const BAND_GAP: usize = 1; // one blank row between bands
const HAND_GAP: usize = 2; // a touch more separation above the hand

// The board is a fixed-size block, centered in both axes. The grid is a
// constant GRID_COLS × GRID_ROWS so its rows are always full (no ragged
// partial row) and its columns line up with the hand — no width-driven
// reflow, and nothing to adjust per terminal size.
pub const GRID_COLS: usize = HAND_SIZE; // 4 — matches the hand width
const GRID_ROWS: usize = MAX_TABLE_CARDS.div_ceil(GRID_COLS); // 3, evenly full
const GRID_H: usize = GRID_ROWS * CARD_HEIGHT + (GRID_ROWS - 1); // card rows + gaps

/// Fixed height of the centered board block (header, grid, hand, status,
/// and the gaps between). config's minimum terminal height is exactly this.
pub const BOARD_BLOCK_HEIGHT: usize =
    HEADER_H + BAND_GAP + GRID_H + HAND_GAP + HAND_H + BAND_GAP + STATUS_H;

/// Fixed inner width of the board (both halves + the divider) — the same
/// on every terminal; wider terminals center it and pad the margins. It is
/// the minimum terminal width: a full GRID_COLS-card hand on each side of
/// the divider.
pub const BOARD_WIDTH: usize = 2 * (H_PAD + HAND_SIZE * CARD_SLOT_W) + 1;

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

/// One player's half of the board: the header strip, the card grid, and
/// the hand. Rects are inclusive, in frame [x][y] coordinates.
#[derive(Debug, Copy, Clone)]
pub struct SideLayout {
    pub header: Rect, // name / score / rounds
    pub grid: Rect,   // one capped card grid: dealer draws + played cards
    pub hand: Rect,   // hand (single row; numbers drawn just below)
}

/// The whole game board's geometry — a fixed-size block centered in the
/// terminal, the single source of truth board.rs draws against.
#[derive(Debug, Copy, Clone)]
pub struct BoardLayout {
    pub divider_x: usize,
    pub player: SideLayout,
    pub opponent: SideLayout,
    pub status: Rect, // two rows (alert over prompt) below the hand
}

impl BoardLayout {
    pub fn new(config: Config) -> Self {
        let cols = config.num_cols;
        let rows = config.num_rows;

        // A fixed-size board, centered in both axes — the same layout on
        // every terminal, just more margin on bigger ones. Nothing below
        // depends on the terminal size beyond these two centering offsets.
        let left = cols.saturating_sub(BOARD_WIDTH) / 2;
        let top = rows.saturating_sub(BOARD_BLOCK_HEIGHT) / 2;
        let divider_x = left + BOARD_WIDTH / 2;

        let y_header = top;
        let y_grid = y_header + HEADER_H + BAND_GAP;
        let y_hand = y_grid + GRID_H + HAND_GAP;
        let y_status = y_hand + HAND_H + BAND_GAP;

        // header/hand span the half to the pad; the grid is exactly
        // GRID_COLS card slots wide, aligned with the hand's first card.
        let side = |half_left: usize, half_right: usize| SideLayout {
            header: Rect::new(half_left, half_right, y_header, y_header + HEADER_H - 1),
            grid: Rect::new(
                half_left,
                half_left + GRID_COLS * CARD_SLOT_W - 1,
                y_grid,
                y_grid + GRID_H - 1,
            ),
            hand: Rect::new(half_left, half_right, y_hand, y_hand + CARD_HEIGHT - 1),
        };

        let player = side(left + H_PAD, divider_x.saturating_sub(H_PAD));
        let opponent = side(divider_x + H_PAD, (left + BOARD_WIDTH).saturating_sub(H_PAD + 1));

        // One status band: two rows below the hand (alert over prompt).
        // With a fixed narrow board there's no room beside the hand, so
        // the wide-terminal "status to the right" case is gone.
        let status = Rect::new(
            left + H_PAD,
            (left + BOARD_WIDTH).saturating_sub(H_PAD + 1),
            y_status,
            y_status + STATUS_H - 1,
        );

        Self {
            divider_x,
            player,
            opponent,
            status,
        }
    }
}

/// Start-menu geometry from terminal size, the title art's height, and
/// the number of menu items.
#[derive(Debug, Copy, Clone)]
pub struct MenuLayout {
    pub center_x: usize,
    pub title_top: usize,
    pub items_top: usize,
    pub item_spacing: usize,
}

impl MenuLayout {
    pub fn new(config: Config, title_height: usize, num_items: usize) -> Self {
        const TITLE_GAP: usize = 3; // blank rows between the title art and items
        const ITEM_SPACING: usize = 2;

        // The whole menu (title art, gap, items) is one block centered
        // vertically, to match the board. Items span (n-1)*spacing + 1 rows.
        let items_height = num_items.saturating_sub(1) * ITEM_SPACING + 1;
        let block_height = title_height + TITLE_GAP + items_height;
        let title_top = config.num_rows.saturating_sub(block_height) / 2;

        Self {
            center_x: config.num_cols / 2,
            title_top,
            items_top: title_top + title_height + TITLE_GAP,
            item_spacing: ITEM_SPACING,
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
        // The fixed board fits the frame and its bands stack — at the
        // minimum size and larger (where it's centered with margin).
        for (cols, rows) in [(89, 31), (180, 48), (120, 40)] {
            let l = BoardLayout::new(cfg(cols, rows));
            assert_side_sane(l.player, cols, rows);
            assert_side_sane(l.opponent, cols, rows);
            assert!(in_bounds(l.status, cols, rows));
        }
    }

    #[test]
    fn layout_block_is_centered_in_both_axes() {
        let (cols, rows) = (180, 48);
        let l = BoardLayout::new(cfg(cols, rows));

        // Vertical: equal margin above the header and below the block.
        let top_margin = l.player.header.y0;
        let bottom_margin = rows - (top_margin + BOARD_BLOCK_HEIGHT);
        assert!(top_margin.abs_diff(bottom_margin) <= 1, "v: {top_margin} vs {bottom_margin}");

        // Horizontal: board left = header.x0 - H_PAD; equal margin each side.
        let board_left = l.player.header.x0 - H_PAD;
        let right_margin = cols - (board_left + BOARD_WIDTH);
        assert!(board_left.abs_diff(right_margin) <= 1, "h: {board_left} vs {right_margin}");
    }

    #[test]
    fn layout_halves_do_not_cross_the_divider() {
        let l = BoardLayout::new(cfg(180, 48));
        assert!(l.player.hand.x1 < l.divider_x);
        assert!(l.opponent.hand.x0 > l.divider_x);
    }

    #[test]
    fn menu_layout_centers_the_menu_block_vertically() {
        // Title art 8 rows + 3-row gap + 2 items (spacing 2 → 3 rows) = 14,
        // centered in 48 rows → title_top = 17.
        let l = MenuLayout::new(cfg(89, 48), 8, 2);
        let block_height = 8 + 3 + ((2 - 1) * 2 + 1); // = 14
        assert_eq!(l.title_top, (48 - block_height) / 2);
        // equal margin above the title art and below the last item
        let last_item_row = l.items_top + (2 - 1) * l.item_spacing;
        assert!(l.title_top.abs_diff(48 - (last_item_row + 1)) <= 1);
    }

    #[test]
    fn layout_grid_is_a_fixed_four_by_three() {
        // The grid never reflows: GRID_COLS wide, MAX_TABLE_CARDS filling
        // whole rows (no ragged partial row).
        assert_eq!(GRID_COLS, 4);
        assert_eq!(MAX_TABLE_CARDS.div_ceil(GRID_COLS), 3);
        assert_eq!(MAX_TABLE_CARDS % GRID_COLS, 0); // rows are always full
    }

    #[test]
    fn layout_grid_holds_twelve_slots_within_the_frame_and_halves() {
        // Every slot position lands inside the frame with each side's cards
        // on its own side of the divider — at the minimum size and larger.
        for (cols, rows) in [(89, 31), (180, 48)] {
            let l = BoardLayout::new(cfg(cols, rows));
            for i in 0..MAX_TABLE_CARDS {
                let (x, y) = card_slot(l.player.grid, i % GRID_COLS, i / GRID_COLS);
                assert!(x + CARD_WIDTH <= cols && y + CARD_HEIGHT <= rows, "player {i} off-frame");
                assert!(x + CARD_WIDTH - 1 < l.divider_x, "player {i} crosses divider");

                let (ox, oy) = card_slot(l.opponent.grid, i % GRID_COLS, i / GRID_COLS);
                assert!(ox > l.divider_x, "opponent {i} not right of divider");
                assert!(ox + CARD_WIDTH <= cols && oy + CARD_HEIGHT <= rows, "opponent {i} off-frame");
            }
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
