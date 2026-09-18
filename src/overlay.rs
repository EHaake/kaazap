use crate::{config::Config, frame::{Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text_in}, layout::{OverlayLayout, Rect}};

#[derive(Debug, Copy, Clone)]
pub enum OverlayKind {
    GameHelp,
    MenuHelp,
    HowToPlay,
    Primer,
    FirstMatch,
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

    /// Size the box to the content, then draw box and text
    ///
    fn draw_overlay(&self, content: &[String], frame: &mut Frame) {
        draw_text_overlay(self.config, content, frame);
    }

    pub fn draw(&self, frame: &mut Frame) {
        // The box sizes itself to whatever text the overlay carries — no
        // per-kind width/height constants to keep in sync with the files
        let content = overlay_text(self.overlay_kind);
        self.draw_overlay(&content, frame);
    }
}

/// The shipped text for an OverlayKind, as a Vec<String>. The files are
/// compiled in with `include_str!`, so this splits a &'static str into lines
/// rather than reading from disk.
/// A free function so a modal that carries no `Overlay` (the onboarding
/// pieces) draws the same shipped text through `draw_text_overlay`.
pub fn overlay_text(kind: OverlayKind) -> Vec<String> {
    match kind {
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
        OverlayKind::Primer => {
            let s: &'static str = include_str!("../assets/primer_text.txt");
            s.lines().map(|line| line.to_string()).collect()
        }
        OverlayKind::FirstMatch => {
            let s: &'static str = include_str!("../assets/first_match_text.txt");
            s.lines().map(|line| line.to_string()).collect()
        }
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

/// Content-width floor for the scrollable overlay: the box's width at the
/// 139-column wide threshold (139 · 38 %), so the compact play log (spec 026)
/// is the same box as the wide one instead of a narrower, clippier one.
const SCROLL_MIN_W: usize = 52;
/// Minimum content height for the scrollable overlay.
const SCROLL_MIN_H: usize = 10;

/// The scrollable overlay's outer box for `config` — pure, so the width rule
/// is testable: `cols · 38 %` clamped to `[SCROLL_MIN_W, cols − 4]` wide,
/// `rows · 58 %` clamped to `[SCROLL_MIN_H, rows − 2]` tall, centered.
fn scroll_box(config: Config) -> Rect {
    let cols = config.num_cols;
    let rows = config.num_rows;
    let box_w = (cols * 38 / 100).clamp(SCROLL_MIN_W.min(cols).max(1), cols.saturating_sub(4).max(1));
    let box_h = (rows * 58 / 100).clamp(SCROLL_MIN_H.min(rows).max(1), rows.saturating_sub(2).max(1));
    let x0 = cols.saturating_sub(box_w) / 2;
    let y0 = rows.saturating_sub(box_h) / 2;
    Rect::new(x0, x0 + box_w.saturating_sub(1), y0, y0 + box_h.saturating_sub(1))
}

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
    let outer = scroll_box(config);

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

        // A body far taller than any viewport, so it genuinely overflows.
        let body: Vec<String> = (0..200).map(|i| format!("line {i}")).collect();

        for config in Config::fit_sizes() {
            let mut frame = new_frame(&config);

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

    #[test]
    fn the_play_log_box_is_the_same_width_compact_and_wide() {
        // The width floor is the wide box's width, so the compact play log
        // shows every line the wide one shows; above the floor the
        // percentage still rules.
        for (cols, rows) in [(89, 31), (139, 31)] {
            let outer = scroll_box(Config { num_cols: cols, num_rows: rows });
            assert_eq!(outer.width(), 52, "play-log box width at {cols}x{rows}");
            assert!(outer.x1 < cols, "play-log box off-frame at {cols}x{rows}: {outer:?}");
        }
        let large = scroll_box(Config { num_cols: 200, num_rows: 48 });
        assert!(large.width() > 52, "the percentage no longer rules above the floor: {large:?}");
    }

    #[test]
    fn help_texts_name_the_new_keys_and_nothing_old() {
        let game = overlay_text(OverlayKind::GameHelp);
        let has = |rows: &[String], a: &str, b: &str| {
            rows.iter().any(|l| l.contains(a) && l.contains(b))
        };
        assert!(has(&game, "Space / D", "Draw"), "no Space/D draw row: {game:?}");
        assert!(has(&game, "Enter / P", "Play"), "no Enter/P play row: {game:?}");
        assert!(has(&game, "1 2 3 4", "Select"), "no 1-4 select row: {game:?}");
        assert!(
            !game.iter().any(|l| l.contains("Enter / Space")),
            "the game overlay still pairs Enter with Space: {game:?}"
        );
        assert!(
            !has(&game, "1 2 3 4", "Play"),
            "the game overlay still says 1-4 play a card: {game:?}"
        );

        let how = overlay_text(OverlayKind::HowToPlay);
        assert!(
            how.iter().any(|l| l.contains("Campaign:")),
            "How to Play has no campaign section: {how:?}"
        );
        assert!(
            how.iter().any(|l| l.contains("Space draws")),
            "How to Play does not name Space as draw: {how:?}"
        );
        assert!(
            how.iter().any(|l| l.contains("Enter plays it")),
            "How to Play does not name Enter as play: {how:?}"
        );
        assert!(
            !how.iter().any(|l| l.contains("Enter/Space")),
            "How to Play still pairs Enter with Space: {how:?}"
        );
    }

    #[test]
    fn help_texts_fit_the_minimum_terminal_unclamped() {
        // The boxes must fit both the compact minimum and the wide threshold
        // with margin — if either outgrows the frame, OverlayLayout clamps
        // and the rows get eaten.
        for config in Config::fit_sizes() {
            let (cols, rows) = (config.num_cols, config.num_rows);
            for kind in [
                OverlayKind::GameHelp,
                OverlayKind::MenuHelp,
                OverlayKind::HowToPlay,
                OverlayKind::Primer,
                OverlayKind::FirstMatch,
            ] {
                let lines = overlay_text(kind);
                let (width, height) = measure(&lines);
                let layout = OverlayLayout::new(config, width, height);
                assert_eq!(
                    layout.outer.height(),
                    height + crate::V_PAD,
                    "box height clamped — {kind:?} outgrew {cols}x{rows}"
                );
                assert_eq!(
                    layout.outer.width(),
                    width + 2 * crate::H_PAD,
                    "box width clamped — {kind:?} at {cols}x{rows}"
                );
                assert!(
                    layout.outer.y1 < rows && layout.outer.x1 < cols,
                    "box off-frame: {kind:?} at {cols}x{rows}"
                );
            }
        }
    }

    #[test]
    fn onboarding_texts_are_the_spec_text_and_fit() {
        // The two onboarding pieces ship the spec's text exactly, and their
        // boxes fit both fit sizes unclamped over the map or board.
        for (kind, line_count, title, dismiss) in [
            (
                OverlayKind::Primer,
                10,
                "=====  Your first campaign  =====",
                "Enter to continue",
            ),
            (
                OverlayKind::FirstMatch,
                12,
                "=====  How a match works  =====",
                "Enter to begin",
            ),
        ] {
            let lines = overlay_text(kind);
            assert_eq!(lines.len(), line_count, "{kind:?} is not the spec's text: {lines:?}");
            assert_eq!(lines[0], title, "{kind:?} title row: {lines:?}");
            assert_eq!(
                lines[lines.len() - 1].trim(),
                dismiss,
                "{kind:?} dismiss line: {lines:?}"
            );

            let (width, height) = measure(&lines);
            for config in Config::fit_sizes() {
                let (cols, rows) = (config.num_cols, config.num_rows);
                let layout = OverlayLayout::new(config, width, height);
                assert_eq!(
                    layout.outer.height(),
                    height + crate::V_PAD,
                    "box height clamped — {kind:?} outgrew {cols}x{rows}"
                );
                assert_eq!(
                    layout.outer.width(),
                    width + 2 * crate::H_PAD,
                    "box width clamped — {kind:?} at {cols}x{rows}"
                );
                assert!(
                    layout.outer.y1 < rows && layout.outer.x1 < cols,
                    "box off-frame: {kind:?} at {cols}x{rows}"
                );
            }
        }
    }

    #[test]
    fn onboarding_texts_breathe_only_around_the_dismiss_line() {
        // The dismiss line is the acted-on element: a blank row above it, and
        // the box's own bottom padding row below — so the text itself must not
        // end on a blank line, nor double up blanks anywhere else.
        for kind in [OverlayKind::Primer, OverlayKind::FirstMatch] {
            let lines = overlay_text(kind);
            let last = lines.len() - 1;
            assert!(
                !lines[last].trim().is_empty(),
                "{kind:?} ends on a blank line — the box already pads below: {lines:?}"
            );
            assert!(
                lines[last - 1].trim().is_empty(),
                "{kind:?} has no blank row above the dismiss line: {lines:?}"
            );
            for pair in lines.windows(2) {
                assert!(
                    !(pair[0].trim().is_empty() && pair[1].trim().is_empty()),
                    "{kind:?} has two consecutive blank lines: {lines:?}"
                );
            }
        }
    }
}
