//! The tournament venue (spec 029): where a series in progress is played from.
//! While a series is running it is the only campaign match the player may play,
//! so the venue — not the map — is where they sit between its matches: it names
//! the planet, the opponent and the series score, and offers play, the
//! Card Shop, the collection and quit (ruling K1; no abandon action, a series is
//! played out or lost).
//!
//! A `Screen`, not an overlay: it is a full mode the player navigates *to* and
//! the Card Shop and the collection are reached *from* it, which is the
//! constitution's line between the two. Copies `opponent_select.rs`'s shape — a
//! small state struct, one owned outcome enum, `draw(frame, config, …, pulse)` —
//! and draws through [`VenueLayout`], which stacks it as three horizontal bands
//! at every width: the header rows, then the planet art region with the
//! opponent's presence panel in its own column beside it, then the action row
//! and the controls hint (spec 029, amendment R3, superseding ruling N1).
//! See `specs/029-tournament-rounds`.

use crossterm::event::KeyCode;

use crate::{
    campaign::{Series, planet_by_id, series_length_label},
    config::Config,
    frame::{BorderWeight, Emphasis, Frame, draw_box, draw_text, draw_text_centered},
    layout::VenueLayout,
    opponent::{DEFAULT_OPPONENT, OpponentProfile, opponent_by_id},
    portrait::draw_presence_panel,
    profile::Profile,
};

/// The four actions, in the order they are drawn (ruling K1). The shop's label
/// is [`shop::TITLE`](crate::shop::TITLE) rather than a second spelling of it
/// (amendment R2), so the button and the screen it opens cannot disagree.
const ACTIONS: [&str; 4] = ["Play", crate::shop::TITLE, "Collection", "Quit"];

/// Blank columns between two action labels — `app.rs`'s `CHOICE_GAP`, the same
/// idiom at the same spacing (that one is private to the choice panel).
const ACTION_GAP: usize = 6;

/// The place line, above the planet.
const PLACE: &str = "Tournament Hall";

/// The controls hint at the foot of the block. A module `const`, like the map's,
/// so the fit test can measure it without a terminal.
const HINT: &str = "←/→ choose  ·  Enter confirm  ·  b shop  ·  c deck  ·  Esc menu";

/// The result of a key at the venue: the cursor moved, or one of the four
/// actions was taken. One owned outcome enum, as the constitution requires; the
/// app performs the transition and plays the matching SFX.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VenueOutcome {
    Moved,
    Play,
    OpenShop,
    OpenCollection,
    QuitToMenu,
}

#[derive(Debug)]
pub struct VenueState {
    selected: usize, // index into ACTIONS
}

impl Default for VenueState {
    fn default() -> Self {
        Self::new()
    }
}

/// The series line: the score, and the length it is played to in the map's own
/// words (amendment R1). `series_length_label` is the single source of that
/// phrase; `wins_needed` stays the source of the *count*, which this line no
/// longer shows.
pub fn series_line(series: &Series) -> String {
    format!(
        "Series  {} – {}   ·   {}",
        series.player_wins,
        series.opponent_wins,
        series_length_label(&series.opponent)
    )
}

/// The action row's width, constant whichever action is cursored: every label
/// with its marker (`▸`, or the space that keeps the marker from shifting the
/// text) plus the gaps between them. Pure, so the row's fit is testable without
/// a terminal (`app.rs`'s `choice_row_width`).
pub fn action_row_width() -> usize {
    let text: usize = ACTIONS.iter().map(|l| l.chars().count() + 2).sum();
    text + ACTION_GAP * (ACTIONS.len() - 1)
}

/// The action labels as drawn, in order: `"{▸ or space} {label}"`, so the
/// cursored one is marked without changing any label's width.
fn action_labels(selected: usize) -> [String; ACTIONS.len()] {
    std::array::from_fn(|i| format!("{} {}", if i == selected { "▸" } else { " " }, ACTIONS[i]))
}

/// The four header rows above the art, in draw order — the one source of truth
/// for their wording, so the fit test measures what is actually drawn. They are
/// compact, with no blank between them: only the acted-on row (the action row,
/// in the footer band) gets air, and that air is now the layout's
/// (`action_y ± 1`) rather than a slot in this array (spec 029, amendment R3).
fn header_rows(
    planet_name: &str,
    planet_region: &str,
    opponent: &OpponentProfile,
    series: &Series,
) -> [String; VenueLayout::HEADER_H] {
    [
        PLACE.to_string(),
        format!("{planet_name}  ·  {planet_region}"),
        format!("{}  —  {}", opponent.name, opponent.difficulty),
        series_line(series),
    ]
}

impl VenueState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// Handle a key: Left/Right (and `a`/`d`) step the cursor, wrapping;
    /// Enter/Space take the highlighted action; `b` and `c` open the Card Shop
    /// and the collection wherever the cursor is, as on the map; Esc/`x` quit to
    /// the menu, the same as the Quit action. Returns `None` for keys this
    /// screen ignores.
    pub fn handle_input(&mut self, key: KeyCode) -> Option<VenueOutcome> {
        match key {
            KeyCode::Left => {
                self.move_selection(-1);
                Some(VenueOutcome::Moved)
            }
            KeyCode::Right => {
                self.move_selection(1);
                Some(VenueOutcome::Moved)
            }
            KeyCode::Enter => Some(self.take_selected()),
            KeyCode::Esc => Some(VenueOutcome::QuitToMenu),
            KeyCode::Char(c) => match c {
                'a' => {
                    self.move_selection(-1);
                    Some(VenueOutcome::Moved)
                }
                'd' => {
                    self.move_selection(1);
                    Some(VenueOutcome::Moved)
                }
                ' ' => Some(self.take_selected()),
                'b' => Some(VenueOutcome::OpenShop),
                'c' => Some(VenueOutcome::OpenCollection),
                'x' => Some(VenueOutcome::QuitToMenu),
                _ => None,
            },
            _ => None,
        }
    }

    /// The outcome of the highlighted action.
    fn take_selected(&self) -> VenueOutcome {
        match self.selected {
            0 => VenueOutcome::Play,
            1 => VenueOutcome::OpenShop,
            2 => VenueOutcome::OpenCollection,
            _ => VenueOutcome::QuitToMenu,
        }
    }

    /// Move the selection by `delta` over the actions, wrapping at the ends.
    fn move_selection(&mut self, delta: isize) {
        let n = ACTIONS.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(n) as usize;
    }

    /// Draw the venue's three bands: the four header rows, then the planet's
    /// art region with the opponent's presence panel in its own column beside
    /// it, then the action row and the controls hint. Both regions draw at
    /// every width (spec 029, amendment R3).
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        // `App` only shows this screen while a series is in progress, and a
        // series names a planet from the `const` graph — the early return says
        // so rather than unwrapping.
        let Some(series) = profile.campaign().series() else {
            return;
        };
        let Some(planet) = planet_by_id(&series.planet) else {
            return;
        };
        // The map's fallback for an id the roster doesn't know.
        let opponent = opponent_by_id(&series.opponent).unwrap_or(DEFAULT_OPPONENT);

        let layout = VenueLayout::new(*config);
        let cx = layout.center_x;
        let rows = header_rows(planet.name, planet.region, &opponent, series);

        // The header band, compact: place, planet, opponent, series.
        let top = layout.header_y;
        draw_text_centered(frame, cx, top, &rows[0], Emphasis::Muted);
        draw_text_centered(frame, cx, top + 1, &rows[1], Emphasis::Strong);
        draw_text_centered(frame, cx, top + 2, &rows[2], Emphasis::Normal);
        draw_text_centered(frame, cx, top + 3, &rows[3], Emphasis::Normal);

        // The art region: reserved now, filled with a plain placeholder — a
        // bordered region carrying the planet's name, so it reads as reserved
        // rather than as a rendering fault (ruling M1).
        draw_box(frame, layout.art, BorderWeight::Single, Emphasis::Muted);
        let art_cx = (layout.art.x0 + layout.art.x1) / 2;
        let art_cy = (layout.art.y0 + layout.art.y1) / 2;
        draw_text_centered(frame, art_cx, art_cy, planet.name, Emphasis::Muted);

        // The opponent's portrait in its own column beside it, a separate
        // element (ruling M1) — the same drawer the map's rail and the select
        // screen use. Top-aligned with the art, so the rows below it are blank.
        draw_presence_panel(frame, layout.portrait, opponent.name, opponent.portrait);

        // The action row: each label at a fixed stride, so the row's width
        // doesn't change as the cursor moves and only the cursored label takes
        // the pulse (`app.rs`'s `draw_choice_panel`). `action_y ± 1` stay blank
        // — the air around the acted-on row is the layout's now.
        let mut x = cx.saturating_sub(action_row_width() / 2);
        for (i, label) in action_labels(self.selected).iter().enumerate() {
            let emphasis = if i == self.selected { pulse } else { Emphasis::Normal };
            draw_text(frame, x, layout.action_y, label, emphasis);
            x += label.chars().count() + ACTION_GAP;
        }

        draw_text_centered(frame, cx, layout.hint_y, HINT, Emphasis::Muted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::{FINAL_OPPONENT, PLANETS};
    use crate::frame::new_frame;

    /// One drawn row as a string (`board.rs`'s test helper).
    fn row_text(frame: &Frame, y: usize) -> String {
        frame.iter().map(|col| col[y].ch).collect()
    }

    fn series(planet: &str, opponent: &str, player_wins: u32, opponent_wins: u32) -> Series {
        Series {
            planet: planet.to_string(),
            opponent: opponent.to_string(),
            player_wins,
            opponent_wins,
        }
    }

    /// The header rows as `draw` builds them, for a planet / opponent pair.
    fn rows_for_opponent(
        planet_name: &str,
        region: &str,
        opponent: &OpponentProfile,
    ) -> [String; VenueLayout::HEADER_H] {
        header_rows(planet_name, region, opponent, &series("cinder", opponent.id, 1, 0))
    }

    #[test]
    fn the_action_row_keeps_its_width_as_the_cursor_moves() {
        // draw_choice_panel's idiom: the marker column is always there, so the
        // row measures the same however the cursor sits and the text under it
        // never shifts.
        for selected in 0..ACTIONS.len() {
            let row = action_labels(selected).join(&" ".repeat(ACTION_GAP));
            assert_eq!(
                row.chars().count(),
                action_row_width(),
                "action row width disagrees with action_row_width() at {selected}"
            );
            assert_eq!(
                row.chars().filter(|&c| c == '▸').count(),
                1,
                "exactly one label carries the marker at {selected}"
            );
            assert!(row.contains(ACTIONS[selected]), "the cursored label is drawn at {selected}");
        }
    }

    #[test]
    fn the_venue_rows_breathe_only_around_the_action_row() {
        // The constitution's density rule (AC 17), on the drawn frame rather
        // than on an array of strings: the acted-on row has an entirely blank
        // row above and below it, and the header rows, the action row and the
        // hint all carry text. `Profile::default()` touches no disk and
        // nothing here calls `save()`, so no `App` is needed.
        let config = Config { num_cols: 89, num_rows: 31 };
        let mut profile = Profile::default();
        profile.campaign_mut().begin_series("cinder", "greeb");

        let mut frame = new_frame(&config);
        VenueState::new().draw(&mut frame, &config, &profile, Emphasis::Strong);

        let layout = VenueLayout::new(config);
        for blank in [layout.action_y - 1, layout.action_y + 1] {
            assert!(
                row_text(&frame, blank).chars().all(|c| c == ' '),
                "row {blank} should be the air around the action row, got {:?}",
                row_text(&frame, blank)
            );
        }
        let filled: Vec<usize> = (layout.header_y..layout.header_y + VenueLayout::HEADER_H)
            .chain([layout.action_y, layout.hint_y])
            .collect();
        for y in filled {
            assert!(
                row_text(&frame, y).chars().any(|c| c != ' '),
                "row {y} should carry text"
            );
        }
    }

    #[test]
    fn the_venue_text_fits_the_minimum_terminal() {
        // Every drawn text row of every reachable planet/opponent pairing,
        // centered on `center_x` as `draw` centers it, lands inside the frame
        // at both layout widths. The text is above and below the art now, not
        // beside it, so there is no horizontal clearance to check — the layout
        // test's vertical disjointness covers that once instead of per row.
        for config in Config::fit_sizes() {
            let cols = config.num_cols;
            let layout = VenueLayout::new(config);

            for planet in PLANETS {
                for id in planet.opponents {
                    let opponent = opponent_by_id(id).unwrap_or(DEFAULT_OPPONENT);
                    for selected in 0..ACTIONS.len() {
                        let header = rows_for_opponent(planet.name, planet.region, &opponent);
                        // The action row is no longer one of the returned rows,
                        // so it is measured explicitly — the same string the
                        // stride loop draws, and the widest cursored row.
                        let action = action_labels(selected).join(&" ".repeat(ACTION_GAP));
                        let rows: Vec<&str> =
                            header.iter().map(|r| r.as_str()).chain([action.as_str(), HINT]).collect();
                        for row in rows.iter().filter(|r| !r.is_empty()) {
                            let w = row.chars().count();
                            // `draw_text_centered`'s placement, and the same
                            // expression the action row's own `x` uses.
                            let x = layout.center_x.saturating_sub(w / 2);
                            assert!(
                                x + w <= cols,
                                "{row:?} ({w} chars) clips the right edge at {cols} columns"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_series_line_names_the_score_and_the_length() {
        assert_eq!(series_line(&series("cinder", "greeb", 1, 0)), "Series  1 – 0   ·   Best of 3");
        assert_eq!(
            series_line(&series("zenith", FINAL_OPPONENT, 2, 1)),
            "Series  2 – 1   ·   Best of 5"
        );

        // The phrase is the map's own (amendment R1), not two literals that
        // happen to agree with it today.
        for s in [series("cinder", "greeb", 1, 0), series("zenith", FINAL_OPPONENT, 2, 1)] {
            let label = series_length_label(&s.opponent);
            assert!(
                series_line(&s).contains(label),
                "{:?} does not name the length {label:?}",
                series_line(&s)
            );
        }
    }

    #[test]
    fn the_venue_keys_move_confirm_and_shortcut() {
        let mut s = VenueState::new();
        assert_eq!(s.selected, 0);

        // ←/→ and a/d wrap over the four actions.
        s.handle_input(KeyCode::Left);
        assert_eq!(s.selected, ACTIONS.len() - 1);
        s.handle_input(KeyCode::Right);
        assert_eq!(s.selected, 0);
        s.handle_input(KeyCode::Char('d'));
        assert_eq!(s.selected, 1);
        s.handle_input(KeyCode::Char('a'));
        assert_eq!(s.selected, 0);
        assert_eq!(s.handle_input(KeyCode::Right), Some(VenueOutcome::Moved));
        assert_eq!(s.selected, 1);

        // Enter and Space take the highlighted action, in ACTIONS' order.
        let mut s = VenueState::new();
        for expected in [
            VenueOutcome::Play,
            VenueOutcome::OpenShop,
            VenueOutcome::OpenCollection,
            VenueOutcome::QuitToMenu,
        ] {
            assert_eq!(s.handle_input(KeyCode::Enter), Some(expected));
            assert_eq!(s.handle_input(KeyCode::Char(' ')), Some(expected));
            s.handle_input(KeyCode::Right);
        }

        // `b` and `c` work from any cursor position, as on the map; Esc and `x`
        // quit to the menu; everything else is ignored.
        for selected in 0..ACTIONS.len() {
            let mut s = VenueState::new();
            s.selected = selected;
            assert_eq!(s.handle_input(KeyCode::Char('b')), Some(VenueOutcome::OpenShop));
            assert_eq!(s.handle_input(KeyCode::Char('c')), Some(VenueOutcome::OpenCollection));
            assert_eq!(s.handle_input(KeyCode::Esc), Some(VenueOutcome::QuitToMenu));
            assert_eq!(s.handle_input(KeyCode::Char('x')), Some(VenueOutcome::QuitToMenu));
            assert!(s.handle_input(KeyCode::Char('z')).is_none());
            assert!(s.handle_input(KeyCode::Up).is_none());
            assert_eq!(s.selected, selected, "an ignored key moved the cursor");
        }
    }
}
