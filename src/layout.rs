use crate::{CARD_HEIGHT, CARD_WIDTH, HAND_SIZE, H_PAD, MAX_TABLE_CARDS, V_PAD, config::Config};
use crate::portrait::{PANEL_GAP, PANEL_H_INMATCH, PANEL_W, PORTRAIT_HEIGHT};

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

/// The minimum terminal width in match: the centered board plus symmetric
/// left/right panel margins. The right margin holds the opponent presence
/// panel; the equal left margin is reserved empty for a future player panel.
pub const IN_MATCH_MIN_WIDTH: usize = BOARD_WIDTH + 2 * (PANEL_GAP + PANEL_W); // 139: board + symmetric left/right panel margins (the left is reserved empty for a future player panel)

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

    pub fn height(&self) -> usize {
        self.y1.saturating_sub(self.y0) + 1
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
    // The opponent presence panel in the right margin, top-aligned with the board block.
    pub opponent_panel: Rect,
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

        // Opponent presence panel: fixed-size, anchored in the right margin PANEL_GAP
        // past the centered board's right edge, top-aligned with the board block. The
        // equal left margin is left empty — reserved for a future player-status panel.
        let panel_x0 = left + BOARD_WIDTH + PANEL_GAP;
        let opponent_panel = Rect::new(panel_x0, panel_x0 + PANEL_W - 1, top, top + PANEL_H_INMATCH - 1);

        Self {
            divider_x,
            player,
            opponent,
            status,
            opponent_panel,
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
    const ITEM_SPACING: usize = 2;

    pub fn new(config: Config, title_height: usize, num_items: usize, trailing_height: usize) -> Self {
        const TITLE_GAP: usize = 3; // blank rows between the title art and items

        // The whole menu (title art, gap, items, plus any trailing content a
        // screen draws below the items — a blurb/hint) is one block centered
        // vertically, to match the board. Items span (n-1)*spacing + 1 rows;
        // `trailing_height` reserves the rows below them so a long list plus its
        // footer still fits the minimum terminal.
        let items_height = num_items.saturating_sub(1) * Self::ITEM_SPACING + 1;
        let block_height = title_height + TITLE_GAP + items_height + trailing_height;
        let title_top = config.num_rows.saturating_sub(block_height) / 2;

        Self {
            center_x: config.num_cols / 2,
            title_top,
            items_top: title_top + title_height + TITLE_GAP,
            item_spacing: Self::ITEM_SPACING,
        }
    }
}

/// Top-left (x, y) of the card at grid position (col, row) within a zone.
/// Callers pass col/row directly: dealer wraps (col = i % per_row), the
/// played/hand rows don't (col = i, row = 0).
pub fn card_slot(zone: Rect, col: usize, row: usize) -> (usize, usize) {
    (zone.x0 + col * CARD_SLOT_W, zone.y0 + row * CARD_SLOT_H)
}

/// Full-screen geometry for the campaign map — a deliberate departure from the
/// centered fixed-blocks above. It spans the whole terminal: a header band at
/// the top, an info-panel band at the bottom, and the node field between, with
/// planets placed at scaled normalized positions. Computed per-draw from
/// `Config`, so a resize needs no map-specific code.
#[derive(Debug, Copy, Clone)]
pub struct CampaignMapLayout {
    pub header: Rect,
    pub field: Rect,
    pub panel: Rect,
    /// The opponent-preview rail on the right of the field band, near its top
    /// (T007 draws the focused planet's face here). The field shrinks to clear it.
    pub portrait_panel: Rect,
}

impl CampaignMapLayout {
    const HEADER_H: usize = 2;
    const PANEL_H: usize = 5;
    // Inset the field so a node glyph and its label near an edge stay on-frame.
    const FIELD_MARGIN_X: usize = 6;
    const FIELD_MARGIN_Y: usize = 1;
    // Gap between the reduced field's right edge and the portrait rail, so no
    // node or (cursored) label lands on the rail.
    const FIELD_RAIL_GAP: usize = 2;

    pub fn new(config: Config) -> Self {
        let last_x = config.num_cols.saturating_sub(1);
        let last_y = config.num_rows.saturating_sub(1);

        let header = Rect::new(0, last_x, 0, Self::HEADER_H.saturating_sub(1));
        let panel_top = config.num_rows.saturating_sub(Self::PANEL_H);
        let panel = Rect::new(0, last_x, panel_top, last_y);
        // The field sits between the bands, clamped so it never inverts on a
        // short terminal (the global 139×31 minimum keeps it comfortable).
        let field_top = Self::HEADER_H;
        let field_bottom = panel_top.saturating_sub(1).max(field_top);

        // Opponent-preview rail: a fixed-width strip on the right of the field band,
        // near its top (T007 draws the focused planet's face here). The node field
        // shrinks to leave a gap before it so no node or label lands on the rail.
        let rail_h = 2 + 1 + PORTRAIT_HEIGHT; // border + name row + portrait (snug — no reserved rows)
        let rail_x0 = last_x.saturating_sub(PANEL_W - 1);
        let portrait_panel = Rect::new(rail_x0, last_x, field_top, field_top + rail_h - 1);
        let field = Rect::new(0, rail_x0.saturating_sub(Self::FIELD_RAIL_GAP), field_top, field_bottom);

        Self { header, field, panel, portrait_panel }
    }

    /// The cell (x, y) for a planet at normalized (fx, fy), placed within the
    /// field inset by a margin so edge planets (and their labels) stay on-frame.
    pub fn node_pos(&self, fx: f32, fy: f32) -> (usize, usize) {
        let x0 = self.field.x0 + Self::FIELD_MARGIN_X;
        let x1 = self.field.x1.saturating_sub(Self::FIELD_MARGIN_X).max(x0);
        let y0 = self.field.y0 + Self::FIELD_MARGIN_Y;
        let y1 = self.field.y1.saturating_sub(Self::FIELD_MARGIN_Y).max(y0);
        let x = x0 + (fx.clamp(0.0, 1.0) * (x1 - x0) as f32).round() as usize;
        let y = y0 + (fy.clamp(0.0, 1.0) * (y1 - y0) as f32).round() as usize;
        (x, y)
    }
}

/// Which briefcase panel a card grid slot belongs to. The deck-builder's
/// two-panel view moves a card *copy* between the Collection (owned, not
/// placed) and the Deck (placed); [`BriefcaseLayout::card_origin`] keys off
/// it, and the screen's per-panel cursors will too.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Panel {
    Collection,
    Deck,
}

/// Geometry for the deck-builder's two-panel "briefcase": a screen title, a
/// "Deck: N/10" readout over the right panel, two side-by-side bordered
/// panels — Collection (left) | Deck (right), each with a title-label anchor
/// on its top border — and a controls hint. Sized purely from [`Config`], so
/// it unit-tests without a terminal. Each panel is a fixed content-sized album:
/// a [`Self::COLS`] × [`Self::ROWS`] grid holding every card type at once (no
/// scrolling), its border hugging the grid, and the whole block centered in the
/// terminal like [`BoardLayout`].
#[derive(Debug, Copy, Clone)]
pub struct BriefcaseLayout {
    pub center_x: usize,  // screen center — the title and hint center here
    pub title_y: usize,
    pub readout_x: usize, // center of the deck panel — the N/10 readout sits here
    pub readout_y: usize,
    pub hint_y: usize,
    pub collection: Rect, // left panel perimeter (bordered)
    pub deck: Rect,       // right panel perimeter (bordered)
    pub collection_label: (usize, usize), // (x, y) where "Collection" draws, on its top border
    pub deck_label: (usize, usize),       // (x, y) where "Deck" draws, on its top border
    pub cols: usize,      // 4 — cards per panel row
    pub rows: usize,      // 4 — card rows per panel (16 slots, 15 used)
}

// The album is a fixed COLS×ROWS grid with one slot per card type. If the card
// universe ever outgrows that grid, the fixed layout would silently overflow
// (`card_origin` past a panel) — grow the grid and re-check the 139×31 fit rather
// than letting it slide. Ties the hardcoded ROWS/COLS to `ALL_SIDE_CARDS`.
const _: () = assert!(
    crate::card::ALL_SIDE_CARDS.len() <= BriefcaseLayout::COLS * BriefcaseLayout::ROWS,
    "ALL_SIDE_CARDS outgrew the briefcase album grid; grow BriefcaseLayout::ROWS/COLS and re-check the 139×31 fit",
);

impl BriefcaseLayout {
    /// Four card columns per panel.
    pub const COLS: usize = 4;
    /// Four card rows per panel — a fixed 4×4 = 16-slot grid, 15 of them filled
    /// by the card universe (`ALL_SIDE_CARDS`), the 16th left empty. Every type
    /// gets a slot, so the album never scrolls.
    pub const ROWS: usize = 4;
    /// Horizontal pitch inside a panel: a card plus a one-cell gap — the same
    /// pitch the board grid uses.
    const CARD_PITCH_X: usize = CARD_WIDTH + 1;
    /// A card cell's height: the card plus its count-caption row, with no
    /// inter-row gap — packed so four rows fit the 139×31 minimum.
    const CELL_H: usize = CARD_HEIGHT + 1;
    /// One interior column of breathing room between a panel's border and its
    /// card grid, each side.
    const PANEL_PAD_X: usize = 1;
    /// Blank columns between the two panels.
    const PANEL_GAP: usize = 3;

    /// Interior card-grid width: COLS cards with a gap between each.
    const GRID_W: usize = Self::COLS * Self::CARD_PITCH_X - 1;
    /// A panel's full width, including its border and interior padding.
    const PANEL_W: usize = Self::GRID_W + 2 * Self::PANEL_PAD_X + 2;
    /// A panel's full height: the ROWS card cells plus the top and bottom
    /// borders.
    const PANEL_H: usize = Self::ROWS * Self::CELL_H + 2;
    /// The two panels plus the gap between them — the fixed block width,
    /// centered in the terminal like the board.
    const BRIEFCASE_W: usize = 2 * Self::PANEL_W + Self::PANEL_GAP;
    /// The fixed block height: a title row, a readout row, the panels, and a
    /// hint row, stacked with no inter-row gaps so it packs into 31 rows.
    const BLOCK_H: usize = 1 + 1 + Self::PANEL_H + 1;

    pub fn new(config: Config) -> Self {
        let cols = config.num_cols;
        let rows = config.num_rows;

        // A fixed-size block (title, readout, panels, hint), centered in both
        // axes like the board — wider/taller terminals just pad the margins.
        // Rows stack tight: title, readout, panels, hint, no gaps between.
        let top = rows.saturating_sub(Self::BLOCK_H) / 2;
        let title_y = top;
        let readout_y = title_y + 1;
        let panel_y0 = readout_y + 1;
        let panel_y1 = panel_y0 + Self::PANEL_H - 1;
        let hint_y = panel_y1 + 1;

        // A fixed-width two-panel block, centered like the board.
        let left = cols.saturating_sub(Self::BRIEFCASE_W) / 2;
        let collection = Rect::new(left, left + Self::PANEL_W - 1, panel_y0, panel_y1);
        let deck_x0 = left + Self::PANEL_W + Self::PANEL_GAP;
        let deck = Rect::new(deck_x0, deck_x0 + Self::PANEL_W - 1, panel_y0, panel_y1);

        Self {
            center_x: cols / 2,
            title_y,
            readout_x: (deck.x0 + deck.x1) / 2,
            readout_y,
            hint_y,
            collection,
            deck,
            collection_label: (collection.x0 + 2, collection.y0),
            deck_label: (deck.x0 + 2, deck.y0),
            cols: Self::COLS,
            rows: Self::ROWS,
        }
    }

    /// Top-left (x, y) of the 9×5 card box at grid slot `index` (row-major, in
    /// `0..cols*rows`) inside `panel`'s interior. Slots `0..=14` hold the 15
    /// card types; the 16th (`index == 15`) is the empty grid corner.
    pub fn card_origin(&self, panel: Panel, index: usize) -> (usize, usize) {
        let border = match panel {
            Panel::Collection => self.collection,
            Panel::Deck => self.deck,
        };
        let (col, row) = (index % self.cols, index / self.cols);
        let x0 = border.x0 + 1 + Self::PANEL_PAD_X;
        let y0 = border.y0 + 1;
        (x0 + col * Self::CARD_PITCH_X, y0 + row * Self::CELL_H)
    }
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
        for (cols, rows) in [(89, 31), (139, 31), (180, 48), (120, 40)] {
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
    fn board_and_panel_fit_the_minimum_terminal() {
        // At the grown minimum the opponent presence panel sits in the right
        // margin, on-frame, right of the opponent half (no overlap with the
        // board's content), top-aligned with the board block.
        let (cols, rows) = (IN_MATCH_MIN_WIDTH, 31);
        let l = BoardLayout::new(cfg(cols, rows));
        assert!(in_bounds(l.opponent_panel, cols, rows), "opponent panel off-frame");
        // The board's rightmost Rect is the opponent hand/header/status (all
        // share that right edge); the panel sits strictly right of it.
        assert!(l.opponent_panel.x0 > l.opponent.hand.x1, "panel overlaps the board");
        assert!(l.opponent_panel.y1 <= 30, "panel bottom below the board block");
        assert!(l.opponent_panel.y1 <= rows - 1, "panel bottom off-frame");
    }

    #[test]
    fn menu_layout_centers_the_menu_block_vertically() {
        // Title art 8 rows + 3-row gap + 2 items (spacing 2 → 3 rows) = 14,
        // centered in 48 rows → title_top = 17.
        let l = MenuLayout::new(cfg(89, 48), 8, 2, 0);
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
    fn campaign_map_layout_fits_the_minimum_terminal() {
        // At the 139×31 minimum the three bands stack in-bounds and every planet
        // node (and the label row just below it) lands within the field.
        let (cols, rows) = (IN_MATCH_MIN_WIDTH, 31);
        let l = CampaignMapLayout::new(cfg(cols, rows));
        for r in [l.header, l.field, l.panel, l.portrait_panel] {
            assert!(in_bounds(r, cols, rows), "band {r:?} out of bounds");
        }
        assert!(vertically_disjoint(l.header, l.field));
        assert!(vertically_disjoint(l.field, l.panel));
        // The portrait rail sits in the field band, clear of the bottom info panel.
        assert!(l.portrait_panel.y1 < l.panel.y0, "rail overlaps the info panel band");
        for p in crate::campaign::PLANETS {
            let (x, y) = l.node_pos(p.fx, p.fy);
            assert!(x < cols, "{} node x off-frame", p.id);
            assert!(y + 1 <= l.field.y1, "{} node + label overflow the field", p.id);
            // Node stays inside the reduced field and clear of the portrait rail.
            assert!(x <= l.field.x1, "{} node escapes the reduced field", p.id);
            assert!(x < l.portrait_panel.x0, "{} node lands on the rail", p.id);
        }
    }

    #[test]
    fn the_campaign_map_is_legible_at_the_minimum_terminal() {
        // The bigger (spec 011) map's hand-authored positions must not collide or
        // clip at the 139×31 minimum — the guard the renderer doesn't provide:
        // unique node cells, each cursored label on-frame, and no two labels
        // overlapping on a shared row (worst case: both cursored).
        use crate::campaign::PLANETS;
        use crate::campaign_map::cursored_label;

        let (cols, _) = (IN_MATCH_MIN_WIDTH, 31);
        let l = CampaignMapLayout::new(cfg(IN_MATCH_MIN_WIDTH, 31));

        struct Placed {
            id: &'static str,
            x: usize,
            y: usize,
            lx0: usize,
            lx1: usize,
            ly: usize,
        }
        let placed: Vec<Placed> = PLANETS
            .iter()
            .map(|p| {
                let (x, y) = l.node_pos(p.fx, p.fy);
                let len = cursored_label(p.name).chars().count();
                let lx0 = x.saturating_sub(len / 2); // matches draw_text_centered
                Placed { id: p.id, x, y, lx0, lx1: lx0 + len - 1, ly: y + 1 }
            })
            .collect();

        for a in &placed {
            assert!(a.x < cols, "{} node off-frame", a.id);
            assert!(a.ly <= l.field.y1, "{} label row overflows the field", a.id);
            assert!(a.lx1 < cols, "{} cursored label clips the right edge", a.id);
            // Node stays inside the reduced field, and node + cursored label
            // clear the portrait rail (no glyph lands on it).
            assert!(a.x <= l.field.x1, "{} node escapes the reduced field", a.id);
            assert!(a.lx1 < l.portrait_panel.x0, "{} cursored label lands on the rail", a.id);
        }
        for (i, a) in placed.iter().enumerate() {
            for b in &placed[i + 1..] {
                assert!(
                    !(a.x == b.x && a.y == b.y),
                    "{} and {} share a node cell",
                    a.id,
                    b.id
                );
                if a.ly == b.ly {
                    let overlap = a.lx0 <= b.lx1 && b.lx0 <= a.lx1;
                    assert!(!overlap, "{} and {} labels overlap on row {}", a.id, b.id, a.ly);
                }
                // A label must not land on another planet's node glyph (a label
                // sits one row below its own node, so this only bites cross-pairs
                // whose rows happen to coincide).
                if a.ly == b.y {
                    assert!(!(a.lx0 <= b.x && b.x <= a.lx1), "{}'s label covers {}'s node", a.id, b.id);
                }
                if b.ly == a.y {
                    assert!(!(b.lx0 <= a.x && a.x <= b.lx1), "{}'s label covers {}'s node", b.id, a.id);
                }
            }
        }
    }

    #[test]
    fn briefcase_fits_the_minimum_terminal() {
        // Two bordered panels — Collection | Deck — each a fixed 4×4 album of
        // every card type, with a shared title, the "Deck: N/10" readout over
        // the deck panel, and a controls hint, must fit the 139×31 minimum with
        // every card slot inside its panel's border and clear of the hint.
        let (cols, rows) = (IN_MATCH_MIN_WIDTH, 31);
        let l = BriefcaseLayout::new(cfg(cols, rows));

        // A fixed 4 columns × 4 rows (16 slots, 15 used) — no scrolling.
        assert_eq!(l.cols, 4);
        assert_eq!(l.rows, 4);

        // Both panel borders sit on-frame and don't overlap (Collection left of Deck).
        assert!(in_bounds(l.collection, cols, rows), "collection panel off-frame");
        assert!(in_bounds(l.deck, cols, rows), "deck panel off-frame");
        assert!(l.collection.x1 < l.deck.x0, "panels overlap");

        // Chrome rows are ordered and on-frame; the readout sits over the deck panel.
        assert!(l.title_y < l.readout_y && l.readout_y < l.hint_y, "chrome rows out of order");
        assert!(l.hint_y < rows, "hint line off-frame");
        assert!(l.deck.x0 <= l.readout_x && l.readout_x <= l.deck.x1, "readout not over the deck");

        // Each panel's title-label anchor sits inside that panel.
        for (anchor, pane) in [(l.collection_label, l.collection), (l.deck_label, l.deck)] {
            assert!(pane.x0 <= anchor.0 && anchor.0 <= pane.x1, "label x outside its panel");
            assert!(pane.y0 <= anchor.1 && anchor.1 <= pane.y1, "label y outside its panel");
        }

        // Every slot of the fixed grid (all COLS×ROWS, not a hardcoded count) in
        // both panels lands a full 9×5 card strictly inside that panel's border,
        // its caption row clear of the hint.
        for panel in [Panel::Collection, Panel::Deck] {
            let border = match panel {
                Panel::Collection => l.collection,
                Panel::Deck => l.deck,
            };
            for i in 0..(l.cols * l.rows) {
                let (x, y) = l.card_origin(panel, i);
                assert!(x > border.x0 && x + CARD_WIDTH <= border.x1, "slot {i} escapes panel x");
                assert!(y > border.y0 && y + CARD_HEIGHT <= border.y1, "slot {i} escapes panel y");
                assert!(y + CARD_HEIGHT < l.hint_y, "slot {i} overlaps the hint line");
            }
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
