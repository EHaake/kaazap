//! The Records overlay: a centered, scrollable read-only popup of lifetime and
//! per-run statistics. Four views — Overall, Quick Play, Campaign, This Run —
//! paged left/right, with a scrollable body. State + input + pure content
//! builders live here; the draw and app wiring are T005.
//!
//! See `specs/020-stats-and-records`.

use crossterm::event::KeyCode;

use crate::opponent::OPPONENTS;
use crate::stats::{LifetimeStats, RunStats, win_rate};
use crate::{
    card::ALL_SIDE_CARDS,
    config::Config,
    frame::{Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text_in},
    layout::Rect,
    profile::Profile,
};

/// How many lines a PageUp/PageDown moves the scroll.
const PAGE: usize = 10;

/// Which view is showing, in display order.
#[derive(Debug, Clone, Copy)]
pub enum RecordsView {
    Overall,
    QuickPlay,
    Campaign,
    ThisRun,
}

/// The four views in display order; `RecordsState::view` indexes this.
const VIEWS: [RecordsView; 4] = [
    RecordsView::Overall,
    RecordsView::QuickPlay,
    RecordsView::Campaign,
    RecordsView::ThisRun,
];

/// What a keypress on the Records screen resolves to.
#[derive(Debug, Clone, Copy)]
pub enum RecordsOutcome {
    Moved,
    Back,
}

/// The Records screen's transient state: which view, and how far scrolled.
#[derive(Debug)]
pub struct RecordsState {
    view: usize,
    scroll: usize,
}

impl Default for RecordsState {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordsState {
    pub fn new() -> Self {
        Self { view: 0, scroll: 0 }
    }

    /// The current view index into `VIEWS`.
    pub fn view(&self) -> usize {
        self.view
    }

    /// The current scroll offset.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Clamp the scroll offset (T005's draw uses this once it knows the body
    /// height and viewport).
    pub fn set_scroll(&mut self, scroll: usize) {
        self.scroll = scroll;
    }

    pub fn handle_input(&mut self, key: KeyCode) -> Option<RecordsOutcome> {
        match key {
            KeyCode::Left => {
                self.view = (self.view + VIEWS.len() - 1) % VIEWS.len();
                self.scroll = 0;
                Some(RecordsOutcome::Moved)
            }
            KeyCode::Right => {
                self.view = (self.view + 1) % VIEWS.len();
                self.scroll = 0;
                Some(RecordsOutcome::Moved)
            }
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                Some(RecordsOutcome::Moved)
            }
            KeyCode::Down => {
                self.scroll += 1;
                Some(RecordsOutcome::Moved)
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(PAGE);
                Some(RecordsOutcome::Moved)
            }
            KeyCode::PageDown => {
                self.scroll += PAGE;
                Some(RecordsOutcome::Moved)
            }
            KeyCode::Esc => Some(RecordsOutcome::Back),
            KeyCode::Char('x') => Some(RecordsOutcome::Back),
            _ => None,
        }
    }

    /// Draw the centered, bordered Records popup: a pager/title row (breathing
    /// with `pulse`), the persistent collection line, a rule, the scrollable body
    /// for the current view, and a footer hint. Mirrors the spec-019 overlay clamp
    /// math (four fixed rows here — pager, collection, rule, footer — so the body
    /// starts at row 3 and `vh = inner_h - 4`), writing the clamped scroll back so
    /// input can't run the offset past the end.
    pub fn draw(&mut self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        let cols = config.num_cols;
        let rows = config.num_rows;
        // A centered popup with clear margins — the same overlay family as the play
        // log / Settings, not full-screen (product-owner change). Content scrolls.
        let box_w = (cols * 55 / 100).clamp(56.min(cols).max(1), cols.saturating_sub(6).max(1));
        let box_h = (rows * 75 / 100).clamp(16.min(rows).max(1), rows.saturating_sub(2).max(1));
        let x0 = cols.saturating_sub(box_w) / 2;
        let y0 = rows.saturating_sub(box_h) / 2;
        let outer = Rect::new(x0, x0 + box_w.saturating_sub(1), y0, y0 + box_h.saturating_sub(1));
        clear_rect(frame, outer);
        draw_box(frame, outer, BorderWeight::Single, Emphasis::Normal);

        let inner = Rect::new(
            outer.x0 + 2,
            outer.x1.saturating_sub(2),
            outer.y0 + 1,
            outer.y1.saturating_sub(1),
        );
        let inner_w = inner.width();
        let inner_h = inner.height();

        // Fixed rows: pager/title (breathes with pulse), collection line, rule.
        draw_text_in(frame, inner, 0, Align::Center, &pager_label(self.view), pulse);
        draw_text_in(
            frame,
            inner,
            1,
            Align::Center,
            &collection_line(profile.distinct_side_cards_owned(), ALL_SIDE_CARDS.len()),
            Emphasis::Normal,
        );
        draw_text_in(frame, inner, 2, Align::Left, &"─".repeat(inner_w), Emphasis::Normal);

        // Body viewport with the spec-019 clamp.
        let body = view_body(
            VIEWS[self.view],
            profile.stats(),
            profile.campaign().run_stats(),
        );
        let vh = inner_h.saturating_sub(4); // rows 0 pager, 1 collection, 2 rule, last footer
        let max_off = body.len().saturating_sub(vh);
        let scroll = self.scroll.min(max_off);
        self.scroll = scroll; // store the clamped value back
        let at_top = scroll == 0;
        let at_bottom = scroll == max_off;
        let block_w = body.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let end = (scroll + vh).min(body.len());
        for (i, line) in body[scroll..end].iter().enumerate() {
            let padded = format!("{line:<block_w$}");
            draw_text_in(frame, inner, 3 + i, Align::Center, &padded, Emphasis::Normal);
        }

        // Footer hint on the last inner row, with up/down markers when overflowing.
        if inner_h >= 5 {
            let up = if max_off > 0 && !at_top { '▲' } else { ' ' };
            let down = if max_off > 0 && !at_bottom { '▼' } else { ' ' };
            let hint = format!("{up} ◂/▸ view · ↑/↓ scroll · Esc back {down}");
            draw_text_in(frame, inner, inner_h - 1, Align::Center, &hint, Emphasis::Normal);
        }
    }
}

// --- Pure content builders (testable without a Frame) ---

/// The collection summary line: `"Cards: N of 15 — P%"`.
pub fn collection_line(owned_types: usize, total_types: usize) -> String {
    let pct = if total_types == 0 {
        0
    } else {
        owned_types * 100 / total_types
    };
    format!("Cards: {owned_types} of {total_types} — {pct}%")
}

/// The short title for a view.
pub fn view_title(view: RecordsView) -> &'static str {
    match view {
        RecordsView::Overall => "Overall",
        RecordsView::QuickPlay => "Quick Play",
        RecordsView::Campaign => "Campaign",
        RecordsView::ThisRun => "This Run",
    }
}

/// The pager/title row label: `"Records — <Title>  (i/N)"`, 1-based.
pub fn pager_label(view_idx: usize) -> String {
    let idx = view_idx % VIEWS.len();
    format!(
        "Records — {}  ({}/{})",
        view_title(VIEWS[idx]),
        idx + 1,
        VIEWS.len()
    )
}

/// Render a win rate as `"P%"`, or `"—"` when there are no matches.
fn win_rate_str(wins: u32, losses: u32) -> String {
    match win_rate(wins, losses) {
        Some(p) => format!("{p}%"),
        None => "—".to_string(),
    }
}

/// A summary block over a wins/losses pair: matches played, won, lost, win rate.
fn summary_lines(wins: u32, losses: u32) -> Vec<String> {
    let played = wins + losses;
    vec![
        format!("Matches played: {played}"),
        format!("Won: {wins}"),
        format!("Lost: {losses}"),
        format!("Win rate: {}", win_rate_str(wins, losses)),
    ]
}

/// The scrollable body lines for a view.
pub fn view_body(view: RecordsView, stats: &LifetimeStats, run: &RunStats) -> Vec<String> {
    match view {
        RecordsView::ThisRun => {
            if run.match_wins + run.match_losses == 0 {
                return vec!["No matches this run yet.".to_string()];
            }
            let mut lines = summary_lines(run.match_wins, run.match_losses);
            lines.push(format!(
                "Streak: {} (best {})",
                run.streak.current, run.streak.longest
            ));
            lines
        }
        RecordsView::Overall | RecordsView::QuickPlay | RecordsView::Campaign => {
            let (wins, losses) = match view {
                RecordsView::Overall => {
                    let (qw, ql) = stats.quick_play().totals();
                    let (cw, cl) = stats.campaign().totals();
                    (qw + cw, ql + cl)
                }
                RecordsView::QuickPlay => stats.quick_play().totals(),
                RecordsView::Campaign => stats.campaign().totals(),
                RecordsView::ThisRun => unreachable!(),
            };

            let mut lines = summary_lines(wins, losses);

            if let RecordsView::Overall = view {
                let s = stats.overall_streak();
                lines.push(format!("Streak: {} (best {})", s.current, s.longest));
            }
            if let RecordsView::Campaign = view {
                lines.push(format!(
                    "Campaign completions: {}",
                    stats.campaign_completions()
                ));
            }

            lines.push(String::new());
            lines.push("By opponent:".to_string());
            lines.push(format!(
                "{:<16} {:>9} {:>9}",
                "Opponent", "Matches", "Rounds"
            ));

            for o in OPPONENTS.iter() {
                let rec = match view {
                    RecordsView::Overall => stats.combined_opponent(o.id),
                    RecordsView::QuickPlay => stats.quick_play().get(o.id),
                    RecordsView::Campaign => stats.campaign().get(o.id),
                    RecordsView::ThisRun => unreachable!(),
                };
                lines.push(format!(
                    "{:<16} {:>9} {:>9}",
                    o.name,
                    format!("{}–{}", rec.match_wins, rec.match_losses),
                    format!("{}–{}", rec.round_wins, rec.round_losses),
                ));
            }

            lines
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Mode;

    #[test]
    fn left_right_page_views_wrapping_and_reset_scroll() {
        let mut s = RecordsState::new();
        assert_eq!(s.view(), 0);

        // From view 0, Left wraps to view 3.
        s.set_scroll(5);
        assert!(matches!(
            s.handle_input(KeyCode::Left),
            Some(RecordsOutcome::Moved)
        ));
        assert_eq!(s.view(), 3);
        assert_eq!(s.scroll(), 0, "Left resets scroll");

        // From view 3, Right wraps to view 0.
        s.set_scroll(5);
        assert!(matches!(
            s.handle_input(KeyCode::Right),
            Some(RecordsOutcome::Moved)
        ));
        assert_eq!(s.view(), 0);
        assert_eq!(s.scroll(), 0, "Right resets scroll");

        // Right steps forward through all four.
        for expected in [1, 2, 3, 0] {
            s.handle_input(KeyCode::Right);
            assert_eq!(s.view(), expected);
        }
    }

    #[test]
    fn up_down_change_scroll_up_saturates() {
        let mut s = RecordsState::new();
        assert!(matches!(
            s.handle_input(KeyCode::Up),
            Some(RecordsOutcome::Moved)
        ));
        assert_eq!(s.scroll(), 0, "Up saturates at 0");

        s.handle_input(KeyCode::Down);
        assert_eq!(s.scroll(), 1);
        s.handle_input(KeyCode::Down);
        assert_eq!(s.scroll(), 2);
        s.handle_input(KeyCode::Up);
        assert_eq!(s.scroll(), 1);
    }

    #[test]
    fn page_keys_move_by_page() {
        let mut s = RecordsState::new();
        s.handle_input(KeyCode::PageDown);
        assert_eq!(s.scroll(), PAGE);
        s.handle_input(KeyCode::PageDown);
        assert_eq!(s.scroll(), 2 * PAGE);
        s.handle_input(KeyCode::PageUp);
        assert_eq!(s.scroll(), PAGE);
        // Saturates.
        s.set_scroll(3);
        s.handle_input(KeyCode::PageUp);
        assert_eq!(s.scroll(), 0);
    }

    #[test]
    fn esc_and_x_go_back_unknown_none() {
        let mut s = RecordsState::new();
        assert!(matches!(
            s.handle_input(KeyCode::Esc),
            Some(RecordsOutcome::Back)
        ));
        assert!(matches!(
            s.handle_input(KeyCode::Char('x')),
            Some(RecordsOutcome::Back)
        ));
        assert!(s.handle_input(KeyCode::Char('q')).is_none());
        assert!(s.handle_input(KeyCode::Enter).is_none());
    }

    #[test]
    fn every_opponent_name_appears_in_each_breakdown_view() {
        let stats = LifetimeStats::default();
        let run = RunStats::default();
        for view in [
            RecordsView::Overall,
            RecordsView::QuickPlay,
            RecordsView::Campaign,
        ] {
            let body = view_body(view, &stats, &run).join("\n");
            for o in OPPONENTS.iter() {
                assert!(
                    body.contains(o.name),
                    "{} missing from {:?} body",
                    o.name,
                    view_title(view)
                );
            }
        }
    }

    #[test]
    fn never_faced_opponent_reads_zero_zero() {
        let stats = LifetimeStats::default();
        let run = RunStats::default();
        let body = view_body(RecordsView::Overall, &stats, &run).join("\n");
        assert!(body.contains("0–0"), "never-faced opponent should show 0–0");
    }

    #[test]
    fn overall_row_equals_quick_play_plus_campaign() {
        let mut stats = LifetimeStats::default();
        // Quick Play: a win 3–1.
        stats.record_match(Mode::QuickPlay, "greeb", true, 3, 1);
        // Campaign: a loss 2–3.
        stats.record_match(Mode::Campaign, "greeb", false, 2, 3);
        let run = RunStats::default();

        let name = OPPONENTS
            .iter()
            .find(|o| o.id == "greeb")
            .map(|o| o.name)
            .expect("greeb in roster");

        let overall = view_body(RecordsView::Overall, &stats, &run);
        let row = overall
            .iter()
            .find(|l| l.contains(name))
            .expect("greeb row present");
        // Combined: match 1–1, round 5–4.
        assert!(row.contains("1–1"), "combined match W–L: {row}");
        assert!(row.contains("5–4"), "combined round W–L: {row}");
    }

    #[test]
    fn summary_carries_matches_won_lost_winrate() {
        let mut stats = LifetimeStats::default();
        stats.record_match(Mode::QuickPlay, "greeb", true, 3, 1);
        stats.record_match(Mode::QuickPlay, "greeb", false, 1, 3);
        let run = RunStats::default();
        let body = view_body(RecordsView::QuickPlay, &stats, &run).join("\n");
        assert!(body.contains("Matches played: 2"));
        assert!(body.contains("Won: 1"));
        assert!(body.contains("Lost: 1"));
        assert!(body.contains("50%"));
    }

    #[test]
    fn campaign_has_completions_line_others_do_not() {
        let mut stats = LifetimeStats::default();
        stats.record_campaign_completion();
        let run = RunStats::default();

        let campaign = view_body(RecordsView::Campaign, &stats, &run).join("\n");
        assert!(campaign.contains("Campaign completions"));

        let overall = view_body(RecordsView::Overall, &stats, &run).join("\n");
        assert!(!overall.contains("Campaign completions"));

        let quick = view_body(RecordsView::QuickPlay, &stats, &run).join("\n");
        assert!(!quick.contains("Campaign completions"));
    }

    #[test]
    fn streak_line_on_overall_and_this_run_not_quick_or_campaign() {
        let mut stats = LifetimeStats::default();
        stats.record_match(Mode::QuickPlay, "greeb", true, 3, 1);
        let mut run = RunStats::default();
        run.record_match(true, 3, 2);

        let overall = view_body(RecordsView::Overall, &stats, &run).join("\n");
        assert!(overall.contains("Streak:"), "Overall shows a streak");

        let this_run = view_body(RecordsView::ThisRun, &stats, &run).join("\n");
        assert!(this_run.contains("Streak:"), "This Run shows a streak");

        let quick = view_body(RecordsView::QuickPlay, &stats, &run).join("\n");
        assert!(!quick.contains("Streak:"), "Quick Play shows no streak");

        let campaign = view_body(RecordsView::Campaign, &stats, &run).join("\n");
        assert!(!campaign.contains("Streak:"), "Campaign shows no streak");
    }

    #[test]
    fn this_run_empty_then_summary() {
        let stats = LifetimeStats::default();

        let empty = view_body(RecordsView::ThisRun, &stats, &RunStats::default());
        assert_eq!(empty, vec!["No matches this run yet.".to_string()]);

        let mut run = RunStats::default();
        run.record_match(true, 3, 2);
        let body = view_body(RecordsView::ThisRun, &stats, &run).join("\n");
        assert!(body.contains("Matches played: 1"));
        assert!(body.contains("Won: 1"));
    }

    #[test]
    fn collection_line_renders_n_of_15_and_percent() {
        let line = collection_line(3, 15);
        assert!(line.contains("3 of 15"), "line: {line}");
        assert!(line.contains("20%"), "line: {line}");
    }

    #[test]
    fn pager_label_is_one_based() {
        assert!(pager_label(0).contains("Overall"));
        assert!(pager_label(0).contains("(1/4)"));
        assert!(pager_label(3).contains("This Run"));
        assert!(pager_label(3).contains("(4/4)"));
    }
}
