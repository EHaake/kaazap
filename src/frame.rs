use crossterm::style::Attribute;

use crate::{config::Config, layout::Rect};

/// Monochrome emphasis — one axis at a time, per design/brief.md. Not
/// bitflags: representable state matches allowed state. Cursor
/// selection is carried by border-glyph weight, not by this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emphasis {
    #[default]
    Normal, // default weight
    Strong, // bold    — the thing to look at
    Muted,  // dim     — inactive / secondary
    Alert,  // inverse — interrupts only, rationed
}

impl Emphasis {
    /// The terminal attribute this emphasis renders as. Normal maps to
    /// Reset (no attribute); only Emphasis ever becomes an escape code —
    /// there is no path here that emits color.
    pub fn attribute(self) -> Attribute {
        match self {
            Emphasis::Normal => Attribute::Reset,
            Emphasis::Strong => Attribute::Bold,
            Emphasis::Muted => Attribute::Dim,
            Emphasis::Alert => Attribute::Reverse,
        }
    }
}

/// One screen position: a character and its monochrome emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub emphasis: Emphasis,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            emphasis: Emphasis::Normal,
        }
    }
}

// The double-buffered frame: a grid of styled cells, indexed [x][y].
pub type Frame = Vec<Vec<Cell>>;

pub fn new_frame(config: &Config) -> Frame {
    let mut cols = Vec::with_capacity(config.num_cols);

    for _ in 0..config.num_cols {
        // A fresh column of blank cells — the dynamic terminal height
        let col = vec![Cell::default(); config.num_rows];
        cols.push(col);
    }

    cols
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}

/// Horizontal placement of text within a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Border line weight. Single is the default chrome everywhere; Heavy
/// is reserved for cursor selection; Double marks a played side card on
/// the board grid (dealer draws stay Single) — three distinct weights,
/// three distinct meanings (design/brief.md, spec 003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderWeight {
    Single,
    Heavy,
    Double,
}

/// The six box-drawing glyphs a border needs.
struct BoxGlyphs {
    horiz: char,
    vert: char,
    tl: char,
    tr: char,
    bl: char,
    br: char,
}

impl BorderWeight {
    fn glyphs(self) -> BoxGlyphs {
        match self {
            BorderWeight::Single => BoxGlyphs {
                horiz: '─',
                vert: '│',
                tl: '┌',
                tr: '┐',
                bl: '└',
                br: '┘',
            },
            BorderWeight::Heavy => BoxGlyphs {
                horiz: '━',
                vert: '┃',
                tl: '┏',
                tr: '┓',
                bl: '┗',
                br: '┛',
            },
            BorderWeight::Double => BoxGlyphs {
                horiz: '═',
                vert: '║',
                tl: '╔',
                tr: '╗',
                bl: '╚',
                br: '╝',
            },
        }
    }
}

/// Frame dimensions as (width, height); (0, 0) for an empty frame.
fn dims(frame: &Frame) -> (usize, usize) {
    if frame.is_empty() {
        (0, 0)
    } else {
        (frame.len(), frame[0].len())
    }
}

/// Clip-safe single-cell write: sets one cell's glyph + emphasis, silently
/// dropping an out-of-bounds position. The non-allocating way to place a single
/// character (vs. `draw_text` with a one-char string).
pub fn draw_char(frame: &mut Frame, x: usize, y: usize, ch: char, emphasis: Emphasis) {
    let (w, h) = dims(frame);
    if x < w && y < h {
        frame[x][y] = Cell { ch, emphasis };
    }
}

/// Draw `text` starting at (x, y), left to right. Clip-safe: writes the
/// characters that fit inside the frame and drops the rest — cannot
/// panic at any position or length. Counts in chars, not bytes.
pub fn draw_text(frame: &mut Frame, x: usize, y: usize, text: &str, emphasis: Emphasis) {
    let (w, _) = dims(frame);
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i;
        if cx >= w {
            break;
        }
        draw_char(frame, cx, y, ch, emphasis);
    }
}

/// Draw `text` on `row` (relative to the rect's top) within `rect`,
/// aligned and clipped to the rect's width. Clip-safe.
pub fn draw_text_in(
    frame: &mut Frame,
    rect: Rect,
    row: usize,
    align: Align,
    text: &str,
    emphasis: Emphasis,
) {
    let rect_w = rect.x1.saturating_sub(rect.x0) + 1;
    let rect_h = rect.y1.saturating_sub(rect.y0) + 1;
    if row >= rect_h {
        return;
    }

    // Clip the text to the rect width before placing it
    let clipped: String = text.chars().take(rect_w).collect();
    let len = clipped.chars().count();

    let x = match align {
        Align::Left => rect.x0,
        Align::Center => rect.x0 + (rect_w - len) / 2,
        Align::Right => rect.x0 + (rect_w - len),
    };

    draw_text(frame, x, rect.y0 + row, &clipped, emphasis);
}

/// Draw `text` centered on column `cx` (point-centering, saturating so a long
/// string near the left edge clamps rather than underflows). The
/// column-centered counterpart to [`draw_text_in`]'s rect-centering — the one
/// the screens reach for when centering a line on a layout's center column.
pub fn draw_text_centered(frame: &mut Frame, cx: usize, y: usize, text: &str, emphasis: Emphasis) {
    let x = cx.saturating_sub(text.chars().count() / 2);
    draw_text(frame, x, y, text, emphasis);
}

/// Draw a border box on `rect`'s perimeter with the given weight and
/// emphasis. Perimeter only — interior is the caller's to fill. Clip-safe.
pub fn draw_box(frame: &mut Frame, rect: Rect, weight: BorderWeight, emphasis: Emphasis) {
    draw_box_glyphs(frame, rect, weight.glyphs(), emphasis);
}

/// Draw an empty grid slot as four dim corner ticks — a light, unbusy
/// marker for a reserved-but-unfilled slot (a full dashed box read too
/// heavy). Always Muted so it reads as absent, not a card. Clip-safe.
pub fn draw_ghost_slot(frame: &mut Frame, rect: Rect) {
    draw_char(frame, rect.x0, rect.y0, '┌', Emphasis::Muted);
    draw_char(frame, rect.x1, rect.y0, '┐', Emphasis::Muted);
    draw_char(frame, rect.x0, rect.y1, '└', Emphasis::Muted);
    draw_char(frame, rect.x1, rect.y1, '┘', Emphasis::Muted);
}

/// Blank a rectangular region back to default cells (space, Normal).
/// Clip-safe. Clears the ground under a popup/overlay so the content
/// behind it doesn't show through — resetting emphasis too, so no stray
/// attribute is left on a blanked cell.
pub fn clear_rect(frame: &mut Frame, rect: Rect) {
    let (w, h) = dims(frame);
    for x in rect.x0..=rect.x1 {
        for y in rect.y0..=rect.y1 {
            if x < w && y < h {
                frame[x][y] = Cell::default();
            }
        }
    }
}

/// Shared perimeter drawer: corners drawn last so they win at overlaps.
fn draw_box_glyphs(frame: &mut Frame, rect: Rect, g: BoxGlyphs, emphasis: Emphasis) {
    for x in rect.x0..=rect.x1 {
        draw_char(frame, x, rect.y0, g.horiz, emphasis);
        draw_char(frame, x, rect.y1, g.horiz, emphasis);
    }
    for y in rect.y0..=rect.y1 {
        draw_char(frame, rect.x0, y, g.vert, emphasis);
        draw_char(frame, rect.x1, y, g.vert, emphasis);
    }

    draw_char(frame, rect.x0, rect.y0, g.tl, emphasis);
    draw_char(frame, rect.x1, rect.y0, g.tr, emphasis);
    draw_char(frame, rect.x0, rect.y1, g.bl, emphasis);
    draw_char(frame, rect.x1, rect.y1, g.br, emphasis);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_default_is_blank_and_normal() {
        let c = Cell::default();
        assert_eq!(c.ch, ' ');
        assert_eq!(c.emphasis, Emphasis::Normal);
    }

    #[test]
    fn emphasis_default_is_normal() {
        assert_eq!(Emphasis::default(), Emphasis::Normal);
    }

    #[test]
    fn emphasis_maps_to_the_expected_attributes() {
        assert_eq!(Emphasis::Normal.attribute(), Attribute::Reset);
        assert_eq!(Emphasis::Strong.attribute(), Attribute::Bold);
        assert_eq!(Emphasis::Muted.attribute(), Attribute::Dim);
        assert_eq!(Emphasis::Alert.attribute(), Attribute::Reverse);
    }

    fn blank(w: usize, h: usize) -> Frame {
        vec![vec![Cell::default(); h]; w]
    }

    /// Read a horizontal run of `len` chars starting at (x, y).
    fn read_row(frame: &Frame, x: usize, y: usize, len: usize) -> String {
        (x..x + len).map(|cx| frame[cx][y].ch).collect()
    }

    #[test]
    fn clip_draw_text_places_chars_with_emphasis() {
        let mut f = blank(10, 3);
        draw_text(&mut f, 2, 1, "hi", Emphasis::Strong);
        assert_eq!(read_row(&f, 2, 1, 2), "hi");
        assert_eq!(f[2][1].emphasis, Emphasis::Strong);
        assert_eq!(f[4][1].ch, ' '); // nothing past the text
    }

    #[test]
    fn clip_draw_text_overrunning_right_edge_writes_only_what_fits() {
        let mut f = blank(5, 2);
        draw_text(&mut f, 3, 0, "abcd", Emphasis::Normal);
        // cols 3,4 fit ("ab"); "cd" is dropped, no panic
        assert_eq!(f[3][0].ch, 'a');
        assert_eq!(f[4][0].ch, 'b');
    }

    #[test]
    fn clip_draw_text_out_of_bounds_is_a_noop_not_a_panic() {
        let mut f = blank(4, 4);
        draw_text(&mut f, 99, 0, "x", Emphasis::Normal); // x past width
        draw_text(&mut f, 0, 99, "x", Emphasis::Normal); // y past height
        draw_text(&mut f, 0, 0, "", Emphasis::Normal); // empty
        // frame untouched
        for col in &f {
            for cell in col {
                assert_eq!(cell.ch, ' ');
            }
        }
    }

    #[test]
    fn clip_draw_text_on_empty_frame_does_not_panic() {
        let mut f: Frame = Vec::new();
        draw_text(&mut f, 0, 0, "x", Emphasis::Normal);
    }

    #[test]
    fn clip_draw_text_in_aligns_within_rect() {
        // rect [2..=7] on row 0 → width 6
        let rect = Rect::new(2, 7, 0, 2);
        let mut f = blank(10, 3);

        draw_text_in(&mut f, rect, 0, Align::Left, "ab", Emphasis::Normal);
        assert_eq!(f[2][0].ch, 'a');

        let mut f = blank(10, 3);
        draw_text_in(&mut f, rect, 0, Align::Right, "ab", Emphasis::Normal);
        assert_eq!(f[6][0].ch, 'a');
        assert_eq!(f[7][0].ch, 'b'); // flush to rect's right edge

        let mut f = blank(10, 3);
        draw_text_in(&mut f, rect, 1, Align::Center, "ab", Emphasis::Normal);
        // width 6, len 2 → offset (6-2)/2 = 2 → starts at x0+2 = 4, y0+1 = 1
        assert_eq!(f[4][1].ch, 'a');
        assert_eq!(f[5][1].ch, 'b');
    }

    #[test]
    fn clip_draw_text_in_clips_to_rect_width_and_ignores_out_of_range_rows() {
        let rect = Rect::new(0, 2, 0, 1); // width 3, height 2
        let mut f = blank(10, 3);
        draw_text_in(&mut f, rect, 0, Align::Left, "abcdef", Emphasis::Normal);
        assert_eq!(read_row(&f, 0, 0, 3), "abc");
        assert_eq!(f[3][0].ch, ' '); // clipped at rect edge

        draw_text_in(&mut f, rect, 9, Align::Left, "z", Emphasis::Normal); // row past rect
        assert_eq!(f[0][2].ch, ' ');
    }

    #[test]
    fn box_draws_corners_and_edges_single_weight() {
        let mut f = blank(6, 5);
        draw_box(&mut f, Rect::new(1, 4, 1, 3), BorderWeight::Single, Emphasis::Normal);
        assert_eq!(f[1][1].ch, '┌');
        assert_eq!(f[4][1].ch, '┐');
        assert_eq!(f[1][3].ch, '└');
        assert_eq!(f[4][3].ch, '┘');
        assert_eq!(f[2][1].ch, '─'); // top edge
        assert_eq!(f[1][2].ch, '│'); // left edge
        assert_eq!(f[2][2].ch, ' '); // interior untouched
    }

    #[test]
    fn box_heavy_weight_uses_heavy_glyphs_and_carries_emphasis() {
        let mut f = blank(6, 5);
        draw_box(&mut f, Rect::new(0, 3, 0, 2), BorderWeight::Heavy, Emphasis::Strong);
        assert_eq!(f[0][0].ch, '┏');
        assert_eq!(f[3][0].ch, '┓');
        assert_eq!(f[1][0].ch, '━');
        assert_eq!(f[0][0].emphasis, Emphasis::Strong);
    }

    #[test]
    fn box_exceeding_frame_bounds_clips_without_panic() {
        let mut f = blank(3, 3);
        draw_box(&mut f, Rect::new(0, 10, 0, 10), BorderWeight::Single, Emphasis::Normal);
        // top-left corner still placed; the rest that fit; no panic
        assert_eq!(f[0][0].ch, '┌');
    }

    #[test]
    fn box_double_weight_uses_double_glyphs_and_carries_emphasis() {
        let mut f = blank(6, 5);
        draw_box(&mut f, Rect::new(0, 3, 0, 2), BorderWeight::Double, Emphasis::Strong);
        assert_eq!(f[0][0].ch, '╔');
        assert_eq!(f[3][0].ch, '╗');
        assert_eq!(f[0][2].ch, '╚');
        assert_eq!(f[3][2].ch, '╝');
        assert_eq!(f[1][0].ch, '═'); // top edge
        assert_eq!(f[0][1].ch, '║'); // left edge
        assert_eq!(f[0][0].emphasis, Emphasis::Strong);
    }

    #[test]
    fn ghost_slot_draws_muted_corner_ticks_only() {
        let mut f = blank(6, 5);
        draw_ghost_slot(&mut f, Rect::new(1, 4, 1, 3));
        // Just the four corners, Muted
        assert_eq!(f[1][1].ch, '┌');
        assert_eq!(f[4][1].ch, '┐');
        assert_eq!(f[1][3].ch, '└');
        assert_eq!(f[4][3].ch, '┘');
        assert_eq!(f[1][1].emphasis, Emphasis::Muted);
        // No edges and no interior — that's what makes it unbusy
        assert_eq!(f[2][1].ch, ' '); // top edge blank
        assert_eq!(f[1][2].ch, ' '); // left edge blank
        assert_eq!(f[2][2].ch, ' '); // interior blank
    }

    #[test]
    fn ghost_slot_exceeding_bounds_clips_without_panic() {
        let mut f = blank(3, 3);
        draw_ghost_slot(&mut f, Rect::new(0, 10, 0, 10));
        assert_eq!(f[0][0].ch, '┌'); // corner placed, rest clipped, no panic
    }

    #[test]
    fn clear_rect_blanks_region_to_default_cells() {
        let mut f = blank(6, 5);
        draw_box(&mut f, Rect::new(1, 4, 1, 3), BorderWeight::Heavy, Emphasis::Alert);
        clear_rect(&mut f, Rect::new(1, 4, 1, 3));
        for x in 1..=4 {
            for y in 1..=3 {
                assert_eq!(f[x][y], Cell::default()); // ch and emphasis both reset
            }
        }
    }

    #[test]
    fn clear_rect_out_of_bounds_is_a_noop_not_a_panic() {
        let mut f = blank(3, 3);
        clear_rect(&mut f, Rect::new(0, 10, 0, 10)); // clips, no panic
        clear_rect(&mut Vec::new(), Rect::new(0, 0, 0, 0)); // empty frame
    }
}
