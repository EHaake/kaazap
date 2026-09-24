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
//! and the controls hint (spec 029, amendment R3, superseding ruling N1). The
//! opponent's line, when `App` passes one, is drawn inside that panel on the
//! row the board's panel uses (spec 030).
//! See `specs/029-tournament-rounds` and `specs/030-series-banter`.

use crossterm::event::KeyCode;

use crate::{
    campaign::{Planet, Series, planet_by_id, series_length_label},
    config::Config,
    frame::{BorderWeight, Emphasis, Frame, draw_box, draw_text, draw_text_centered},
    layout::{Rect, VenueLayout},
    opponent::{DEFAULT_OPPONENT, OpponentProfile, opponent_by_id},
    portrait::{draw_banter_line, draw_portrait, draw_presence_panel},
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

/// The controls hint, on the venue block's last row (the terminal's last row at
/// 31 rows; ruling R9 centres the block on a taller terminal). A module
/// `const`, like the map's, so the fit test can measure it without a terminal. `board.rs`'s in-match hint's
/// single-space `·` separators rather than the venue's own `  ·  `: 55 cells
/// instead of 63, which is what lets the row centre on the art's centre at 89
/// columns without ending up flush against the frame (amendment R5).
const HINT: &str = "←/→ choose · Enter confirm · b shop · c deck · Esc menu";

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

/// The five header rows above the art, in draw order — the one source of truth
/// for their wording, so the fit test measures what is actually drawn. They are
/// compact, with no blank between them: only the acted-on row (the action row,
/// in the footer band) gets air, and that air is now the layout's
/// (`action_y ± 1`) rather than a slot in this array (spec 029, amendment R3).
/// The fifth row is the credit balance (amendment R6) — the venue is where the
/// player chooses between playing and shopping, and that is the choice a
/// balance informs. It **is** [`shop::credits_label`](crate::shop::credits_label)'s
/// output rather than a second literal of it, so the row and the Card Shop's
/// own balance line cannot drift.
fn header_rows(
    planet_name: &str,
    planet_region: &str,
    opponent: &OpponentProfile,
    series: &Series,
    credits: u32,
) -> [String; VenueLayout::HEADER_H] {
    [
        PLACE.to_string(),
        format!("{planet_name}  ·  {planet_region}"),
        format!("{}  —  {}", opponent.name, opponent.difficulty),
        series_line(series),
        crate::shop::credits_label(credits),
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

    /// Draw the venue's three bands: the five header rows, then the planet's
    /// art region with the opponent's presence panel in its own column beside
    /// it, then the action row and the controls hint. Both regions draw at
    /// every width (spec 029, amendment R3). `line` is the opponent's line
    /// as `App` has revealed it so far, drawn inside the presence panel; `None`
    /// leaves that row blank (spec 030).
    pub fn draw(
        &self,
        frame: &mut Frame,
        config: &Config,
        profile: &Profile,
        line: Option<&str>,
        pulse: Emphasis,
    ) {
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
        // The art's centre column, which every row centres on — not the
        // terminal's (amendment R5).
        let cx = layout.text_x;
        let rows = header_rows(planet.name, planet.region, &opponent, series, profile.credits());

        // The header band, compact: place, planet, opponent, series, credits.
        // The credit row is Normal, not Muted — it is information the player
        // acts on — and not Strong, which the planet name owns (amendment R6).
        let top = layout.header_y;
        draw_text_centered(frame, cx, top, &rows[0], Emphasis::Muted);
        draw_text_centered(frame, cx, top + 1, &rows[1], Emphasis::Strong);
        draw_text_centered(frame, cx, top + 2, &rows[2], Emphasis::Normal);
        draw_text_centered(frame, cx, top + 3, &rows[3], Emphasis::Normal);
        draw_text_centered(frame, cx, top + 4, &rows[4], Emphasis::Normal);

        // The art region: the planet's own drawing in its box. The box is the
        // drawing's size at every terminal size (ruling R9), so it fills it
        // exactly; the portraits' clip-safe line-by-line drawer draws it — a
        // planet is the same kind of art. The planet's name, centred, is only
        // the fallback for a planet without art.
        draw_art(frame, layout.art, planet_art(&planet, &layout), planet.name);

        // The opponent's portrait in its own column beside it, a separate
        // element (ruling M1) — the same drawer the map's rail and the select
        // screen use. Top-aligned with the art, so the rows below it are blank.
        draw_presence_panel(frame, layout.portrait, opponent.name, opponent.portrait);
        if let Some(line) = line {
            draw_banter_line(frame, layout.portrait, line);
        }

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

/// The drawing the art box holds at this layout: wide from
/// `WIDE_LAYOUT_MIN_WIDTH` columns, narrow below — `layout.wide_art`'s choice,
/// so the drawing and the box it fills come from one decision (ruling R9).
fn planet_art(planet: &Planet, layout: &VenueLayout) -> &'static str {
    if layout.wide_art { planet.art_wide } else { planet.art_narrow }
}

/// The art region: the box, and inside it the planet's drawing from the
/// interior's top-left — the box is the drawing's size, so it fills it
/// exactly (ruling R9). A planet with no art (an empty drawing) keeps the
/// placeholder instead: its name, centred, Muted — the fallback R9 keeps.
fn draw_art(frame: &mut Frame, art: Rect, drawing: &str, name: &str) {
    draw_box(frame, art, BorderWeight::Single, Emphasis::Muted);
    if drawing.is_empty() {
        let cx = (art.x0 + art.x1) / 2;
        draw_text_centered(frame, cx, (art.y0 + art.y1) / 2, name, Emphasis::Muted);
    } else {
        draw_portrait(frame, art.x0 + 1, art.y0 + 1, drawing, Emphasis::Normal);
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
        credits: u32,
    ) -> [String; VenueLayout::HEADER_H] {
        header_rows(planet_name, region, opponent, &series("cinder", opponent.id, 1, 0), credits)
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
        VenueState::new().draw(&mut frame, &config, &profile, None, Emphasis::Strong);

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
    fn the_venue_line_sits_inside_the_portrait_panel() {
        // Spec 030: the opponent's line is inside the presence panel, on its
        // interior row 14 (the panel's last interior row, as on the board),
        // strictly between the side borders, with the gap row above it blank
        // across the interior. The breathing test's no-`App` pattern.
        let line = "Here goes nothing!";
        for config in Config::fit_sizes() {
            let (cols, rows) = (config.num_cols, config.num_rows);
            let mut profile = Profile::default();
            profile.campaign_mut().begin_series("cinder", "greeb");
            let mut frame = new_frame(&config);
            VenueState::new().draw(&mut frame, &config, &profile, Some(line), Emphasis::Strong);

            let p = VenueLayout::new(config).portrait;
            let inside = |y: usize| -> String { (p.x0 + 1..=p.x1 - 1).map(|x| frame[x][y].ch).collect() };
            assert_eq!(
                inside(p.y1 - 1).trim(),
                line,
                "the line isn't on the panel's line row at {cols}×{rows}"
            );
            let row = row_text(&frame, p.y1 - 1);
            let at = row.find(line).map(|b| row[..b].chars().count());
            assert!(
                at.is_some_and(|x| x > p.x0 && x + line.chars().count() - 1 < p.x1),
                "the line isn't strictly inside the side borders at {cols}×{rows}: {row:?}"
            );
            assert!(
                inside(p.y1 - 2).chars().all(|c| c == ' '),
                "the gap row above the line isn't blank at {cols}×{rows}: {:?}",
                inside(p.y1 - 2)
            );
        }
    }

    #[test]
    fn the_venue_text_fits_the_minimum_terminal() {
        // Every drawn text row of every reachable planet/opponent pairing,
        // centered on `text_x` as `draw` centers it, lands inside the frame
        // at every size from the minimum — clear of *both* edges: `draw_text_centered`
        // clamps a left overflow to column 0 with `saturating_sub`, which is
        // exactly the "reads as a rendering fault" `spec.md` forbids, so the
        // left edge is checked too (amendment R5). The text is above and below
        // the art now, not beside it, so there is no horizontal clearance to
        // check — the layout test's vertical disjointness covers that once
        // instead of per row.
        //
        // The binding case, pinned: at 89 columns the 55-cell hint is the
        // widest row and starts at column 4. Pinning the column rather than
        // "> 0" is what makes an unsanctioned re-lengthening fail — the old
        // 63-character hint would sit at 0, and even a 57-character one at 2 —
        // instead of quietly closing on the edge.
        let narrow = VenueLayout::new(Config { num_cols: 89, num_rows: 31 });
        assert_eq!(
            narrow.text_x - HINT.chars().count() / 2,
            4,
            "the hint's left column at 89 columns"
        );

        for config in Config::sizes_from_minimum() {
            let cols = config.num_cols;
            let layout = VenueLayout::new(config);

            for planet in PLANETS {
                for id in planet.opponents {
                    let opponent = opponent_by_id(id).unwrap_or(DEFAULT_OPPONENT);
                    for selected in 0..ACTIONS.len() {
                        // `u32::MAX`: the widest representable balance, so the
                        // credit row can never become the binding row
                        // unnoticed (the breathing test draws the real one).
                        let header =
                            rows_for_opponent(planet.name, planet.region, &opponent, u32::MAX);
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
                            let x = layout.text_x.saturating_sub(w / 2);
                            assert!(
                                x >= 1 && x + w <= cols - 1,
                                "{row:?} ({w} chars) touches a frame edge at {cols} columns (x {x})"
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

    /// The brief's checklist item 4: the 23 codepoints a planet drawing may
    /// use — space, the shade and full blocks, the half blocks, the quadrant
    /// blocks, and the four ASCII marks.
    const PALETTE: &str = " ░▒▓█▀▄▌▐▖▗▘▝▙▟▛▜▚▞.'*+";

    #[test]
    fn every_planets_art_passes_the_briefs_checklist() {
        // AC 21: items 1–6 of `planet-art-brief.md`'s checklist, with the
        // canvas derived from the box the venue actually draws — never
        // restated here, so a geometry change without new art fails.
        assert_eq!(PALETTE.chars().count(), 23, "the brief's palette is 23 codepoints");

        // Item 1: exactly the sixteen expected files (read-only, from the
        // repo — never the data directory), and each is the one embedded,
        // which pins the `include_str!` pairing. Item 5's UTF-8 half: the
        // bytes decode (and `include_str!` fails the build otherwise).
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/planets");
        let found: std::collections::BTreeSet<String> = std::fs::read_dir(&dir)
            .expect("assets/planets is readable")
            .map(|entry| entry.expect("a directory entry").file_name().into_string().expect("a UTF-8 name"))
            // Hidden files (Finder's .DS_Store, say) are not deliveries.
            .filter(|name| !name.starts_with('.'))
            .collect();
        let expected: std::collections::BTreeSet<String> = PLANETS
            .iter()
            .flat_map(|p| [format!("{}-narrow.txt", p.id), format!("{}-wide.txt", p.id)])
            .collect();
        assert_eq!(found, expected, "assets/planets holds a missing, extra or misnamed file");
        for planet in PLANETS {
            for (file, embedded) in [
                (format!("{}-narrow.txt", planet.id), planet.art_narrow),
                (format!("{}-wide.txt", planet.id), planet.art_wide),
            ] {
                let bytes = std::fs::read(dir.join(&file)).expect("the art file is readable");
                let text = String::from_utf8(bytes).unwrap_or_else(|e| panic!("{file} is not UTF-8: {e}"));
                assert_eq!(text, embedded, "{file} is not the drawing embedded for {}", planet.id);
            }
        }

        // The two fit sizes choose different drawings, so neither set of
        // eight can be skipped silently.
        let layouts = Config::fit_sizes().map(VenueLayout::new);
        assert_eq!(layouts.map(|l| l.wide_art), [false, true], "the fit sizes' drawings");

        for l in layouts {
            // The canvas: the art box minus its border.
            let (w, h) = (l.art.width() - 2, l.art.height() - 2);
            let kind = if l.wide_art { "wide" } else { "narrow" };
            let mut distinct = std::collections::HashSet::new();
            for planet in PLANETS {
                let file = format!("{}-{kind}.txt", planet.id);
                let drawing = planet_art(&planet, &l);

                // Item 5's LF half, first: `str::lines` strips a `\r\n`
                // ending, so a CRLF file would pass items 2 and 3 untouched —
                // and would fail item 4 on the wrong assertion.
                assert!(!drawing.contains('\r'), "{file} has a carriage return (CRLF line endings)");

                // Item 2: exactly `h` lines, one trailing newline.
                assert!(drawing.ends_with('\n'), "{file} does not end with a newline");
                assert_eq!(drawing.lines().count(), h, "{file}: line count, canvas height {h}");

                // Item 3: every line exactly `w` wide. Characters equal
                // displayed columns here only because item 4 admits no wide,
                // zero-width or combining character.
                for (n, line) in drawing.lines().enumerate() {
                    assert_eq!(
                        line.chars().count(),
                        w,
                        "{file} line {}: width, canvas width {w}",
                        n + 1
                    );
                }

                // Item 4: only the palette.
                for (n, line) in drawing.lines().enumerate() {
                    for c in line.chars() {
                        assert!(
                            PALETTE.contains(c),
                            "{file} line {}: {c:?} (U+{:04X}) is not in the palette",
                            n + 1,
                            c as u32
                        );
                    }
                }

                distinct.insert(drawing);
            }
            // Item 6: the eight drawings of each kind are pairwise distinct.
            assert_eq!(distinct.len(), PLANETS.len(), "two {kind} drawings are identical");
        }
    }

    #[test]
    fn every_planets_art_fills_its_box_at_every_size() {
        // AC 21's "no blank space inside the frame and nothing clipped": at
        // every size, the drawing the venue picks is exactly the box's
        // interior. With the layout's every-size test (the box is on-frame),
        // it can neither leave a blank cell nor be clipped.
        for config in Config::sizes_from_minimum() {
            let l = VenueLayout::new(config);
            let (w, h) = (l.art.width() - 2, l.art.height() - 2);
            let size = format!("{}x{}", config.num_cols, config.num_rows);
            for planet in PLANETS {
                let drawing = planet_art(&planet, &l);
                assert_eq!(drawing.lines().count(), h, "{}'s drawing height at {size}", planet.id);
                assert!(
                    drawing.lines().all(|line| line.chars().count() == w),
                    "{}'s drawing is not {w} wide at {size}",
                    planet.id
                );
            }
        }
    }

    #[test]
    fn the_venue_draws_the_planets_art_inside_its_box() {
        // The one claim the data tests cannot make: `draw` puts the drawing
        // where the box is. The breathing test's no-`App` pattern.
        for config in Config::fit_sizes() {
            let l = VenueLayout::new(config);
            for planet in PLANETS {
                let mut profile = Profile::default();
                profile.campaign_mut().begin_series(planet.id, planet.opponents[0]);
                let mut frame = new_frame(&config);
                VenueState::new().draw(&mut frame, &config, &profile, None, Emphasis::Strong);

                let lines: Vec<&str> = planet_art(&planet, &l).lines().collect();
                for (i, y) in (l.art.y0 + 1..=l.art.y1 - 1).enumerate() {
                    let drawn: String = (l.art.x0 + 1..=l.art.x1 - 1).map(|x| frame[x][y].ch).collect();
                    assert_eq!(
                        Some(drawn.as_str()),
                        lines.get(i).copied(),
                        "{}'s art row {i} at {}x{}",
                        planet.id,
                        config.num_cols,
                        config.num_rows
                    );
                }
            }
        }
    }

    #[test]
    fn a_planet_without_art_shows_its_name() {
        // R9's fallback, which production never reaches (every planet has
        // art or the build fails): an empty drawing keeps the name on the
        // box's middle row; a real drawing leaves the name off it.
        let config = Config::fit_sizes()[0];
        let l = VenueLayout::new(config);
        let middle = (l.art.y0 + l.art.y1) / 2;
        let planet = PLANETS[0];

        let mut frame = new_frame(&config);
        draw_art(&mut frame, l.art, "", planet.name);
        assert!(row_text(&frame, middle).contains(planet.name), "the fallback names the planet");

        let mut frame = new_frame(&config);
        draw_art(&mut frame, l.art, planet_art(&planet, &l), planet.name);
        assert!(!row_text(&frame, middle).contains(planet.name), "a drawing replaces the name");
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
