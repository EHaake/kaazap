use crate::{config::Config, frame::{Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text_in}, layout::{OverlayLayout, Rect}};

#[derive(Debug, Copy, Clone)]
pub enum OverlayKind {
    GameHelp,
    MenuHelp,
    HowToPlay,
}

#[derive(Debug)]
pub struct Overlay {
    overlay_kind: OverlayKind,
    config: Config,
}

impl Overlay {
    pub fn new(overlay_kind: OverlayKind, config: Config) -> Self {
        Self {
            overlay_kind,
            config,
        }
    }

    /// This overlay's kind — used to rebuild it on a terminal resize.
    pub fn kind(&self) -> OverlayKind {
        self.overlay_kind
    }

    /// Open a text file and read it into a Vec<String> based on OverlayKind
    ///
    fn read_text_from_file(&self) -> Vec<String> {
        match self.overlay_kind {
            OverlayKind::GameHelp => {
                let s: &'static str = include_str!("../assets/game_overlay_text.txt");
                s.lines().map(|line| line.to_string()).collect()
            }
            OverlayKind::MenuHelp => {
                let s: &'static str = include_str!("../assets/menu_overlay_text.txt");
                s.lines().map(|line| line.to_string()).collect()
            }
            OverlayKind::HowToPlay => {
                let s: &'static str = include_str!("../assets/how_to_play_text.txt");
                s.lines().map(|line| line.to_string()).collect()
            }
        }
    }

    /// Size the box to the content, then draw box and text
    ///
    fn draw_overlay(&self, content: &[String], frame: &mut Frame) {
        draw_text_overlay(self.config, content, frame);
    }

    pub fn draw(&self, frame: &mut Frame) {
        // The box sizes itself to whatever text the overlay carries — no
        // per-kind width/height constants to keep in sync with the files
        let content = self.read_text_from_file();
        self.draw_overlay(&content, frame);
    }
}

/// Size a box to `content` and draw it into `frame`: measure the text,
/// build the layout, clear the region, draw the border, then draw the
/// text — the first line centered as a title, the rest left-aligned so
/// lists and columns stay lined up. Exposed as a free function so
/// dynamic content (e.g. the play log) can render through the same box
/// machinery without going through `Overlay`.
pub fn draw_text_overlay(config: Config, content: &[String], frame: &mut Frame) {
    let (width, height) = measure(content);
    let layout = OverlayLayout::new(config, width, height);

    clear_rect(frame, layout.outer);
    draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
    for (i, line) in content.iter().enumerate() {
        if i == 0 {
            draw_text_in(frame, layout.inner, 0, Align::Center, line.trim(), Emphasis::Normal);
        } else {
            draw_text_in(frame, layout.inner, i, Align::Left, line, Emphasis::Normal);
        }
    }
}

/// The result of drawing a scrollable overlay: the clamped scroll offset the
/// caller should store back (so the next key press starts from a real value),
/// plus whether the body is pinned at the top or bottom. The caller uses
/// `at_bottom` to keep its "follow the latest" flag pinned.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScrollResult {
    pub scroll: usize,
    pub at_top: bool,
    pub at_bottom: bool,
}

/// Minimum content width for the scrollable overlay — at least as wide as a
/// full round header reads comfortably.
const SCROLL_MIN_W: usize = 40;
/// Minimum content height for the scrollable overlay.
const SCROLL_MIN_H: usize = 10;

/// Draw a fixed, larger, padded overlay with a pinned title and a vertically
/// scrolled body — the play-log window (spec 019). `scroll` is the top body
/// line shown; `usize::MAX` means "pin to the bottom". Returns the clamped
/// scroll offset and the at-top/at-bottom flags. Monochrome (`Emphasis::Normal`
/// throughout), reusing the same box machinery as [`draw_text_overlay`].
pub fn draw_scrollable_overlay(
    config: Config,
    title: &str,
    body: &[String],
    scroll: usize,
    frame: &mut Frame,
) -> ScrollResult {
    // A roomy but not full-screen box — clearly larger and airier than spec
    // 018's content-sized overlay, yet leaving a margin so a normal
    // multi-round match overflows into scrolling. Sized directly rather than
    // through `OverlayLayout` (whose fixed padding is tuned for the small
    // static overlays and would push this near full-screen).
    let cols = config.num_cols;
    let rows = config.num_rows;
    let box_w = (cols * 38 / 100).clamp(SCROLL_MIN_W.min(cols).max(1), cols.saturating_sub(4).max(1));
    let box_h = (rows * 58 / 100).clamp(SCROLL_MIN_H.min(rows).max(1), rows.saturating_sub(2).max(1));
    let x0 = cols.saturating_sub(box_w) / 2;
    let y0 = rows.saturating_sub(box_h) / 2;
    let outer = Rect::new(x0, x0 + box_w.saturating_sub(1), y0, y0 + box_h.saturating_sub(1));

    clear_rect(frame, outer);
    draw_box(frame, outer, BorderWeight::Single, Emphasis::Normal);

    // Interior inset two columns each side (border + one padding column) for
    // breathing room; one row for the border top/bottom.
    let inner = Rect::new(
        outer.x0 + 2,
        outer.x1.saturating_sub(2),
        outer.y0 + 1,
        outer.y1.saturating_sub(1),
    );
    let inner_w = inner.width();
    let inner_h = inner.height();

    // Interior rows: 0 title, 1 rule, 2 blank (breathing room), 3.. body, last
    // row the scroll hint. Body viewport = inner_h minus those four rows.
    let vh = inner_h.saturating_sub(4);
    let max_off = body.len().saturating_sub(vh);
    let scroll = scroll.min(max_off);
    let at_top = scroll == 0;
    let at_bottom = scroll == max_off;

    draw_text_in(frame, inner, 0, Align::Center, title.trim(), Emphasis::Normal);
    if inner_h > 1 {
        let rule: String = "─".repeat(inner_w);
        draw_text_in(frame, inner, 1, Align::Left, &rule, Emphasis::Normal);
    }
    // Body viewport at rows 3.. (row 2 left blank). draw_text_in clips lines
    // wider than the rect, so no manual truncation is needed.
    let end = (scroll + vh).min(body.len());
    for (i, line) in body[scroll..end].iter().enumerate() {
        draw_text_in(frame, inner, 3 + i, Align::Left, line, Emphasis::Normal);
    }
    // Scroll hint on the last inner row, arrows marking more above/below — only
    // when the body actually overflows the viewport.
    if inner_h >= 4 && max_off > 0 {
        let up = if at_top { ' ' } else { '▲' };
        let down = if at_bottom { ' ' } else { '▼' };
        let hint = format!("{up} ↑/↓ · PgUp/PgDn {down}");
        draw_text_in(frame, inner, inner_h - 1, Align::Center, &hint, Emphasis::Normal);
    }

    ScrollResult { scroll, at_top, at_bottom }
}

/// Content dimensions of an overlay's text: widest line (in chars) and
/// number of lines.
fn measure(content: &[String]) -> (usize, usize) {
    let width = content.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    (width, content.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_measure_uses_widest_line_and_line_count() {
        let content = vec![
            "short".to_string(),
            "a much longer line".to_string(),
            "mid".to_string(),
        ];
        assert_eq!(measure(&content), (18, 3));
    }

    #[test]
    fn overlay_measure_counts_chars_not_bytes() {
        // "±" is multi-byte; width must be char count (3), not byte len
        let content = vec!["±1T".to_string()];
        assert_eq!(measure(&content), (3, 1));
    }

    #[test]
    fn overlay_measure_of_empty_content_is_zero() {
        assert_eq!(measure(&[]), (0, 0));
    }

    #[test]
    fn scrollable_overlay_pins_to_bottom_and_clamps_overscroll() {
        use crate::frame::new_frame;

        let config = Config { num_cols: 139, num_rows: 31 };
        let mut frame = new_frame(&config);
        // A body far taller than any viewport, so it genuinely overflows.
        let body: Vec<String> = (0..200).map(|i| format!("line {i}")).collect();

        // usize::MAX pins to the bottom.
        let pinned = draw_scrollable_overlay(config, "Play Log", &body, usize::MAX, &mut frame);
        assert!(pinned.at_bottom);
        assert!(!pinned.at_top);

        // An over-large explicit scroll clamps to the same max offset.
        let clamped = draw_scrollable_overlay(config, "Play Log", &body, 9_999, &mut frame);
        assert_eq!(clamped.scroll, pinned.scroll);
        assert!(clamped.at_bottom);

        // Zero scroll sits at the top.
        let top = draw_scrollable_overlay(config, "Play Log", &body, 0, &mut frame);
        assert!(top.at_top);
        assert_eq!(top.scroll, 0);
    }
}
