# Plan: Compact layout below 139 columns — spec 026

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

## Context

The match board is a fixed 89-column block (`BOARD_WIDTH`), but since spec 016
the terminal must be 139 wide because `IN_MATCH_MIN_WIDTH` adds a 25-column
panel margin on each side and `Config::min_size()` reads that constant. This
spec makes 139 a **threshold** rather than the minimum: at 139 and up the board
draws exactly as today (panel in the right margin); from 89 to 138 the same
board draws alone, centered, and a staked match's `Stake ◈ N` moves onto the
status band's upper row. Every other screen already fits 89 columns by
construction (the pre-016 minimum), and the two preview surfaces that draw a
portrait — the opponent-select preview and the map's rail — physically fit at
89 and keep drawing.

It is a **presentation-only spec**: `layout.rs` (one `Option`, one renamed
constant), `config.rs` (the minimum), `board.rs` (the panel-less arm and the
stake line), `portrait.rs` (the shared `Stake ◈ N` string), `overlay.rs` (the
play-log box's width floor, §tension 4), `records.rs` (a pure content-size
helper so a fit test exists), fit tests in the screens the spec lists, and the
README. `main.rs` and `App::resize` are **not** changed: `Config::fits` reads
`min_size()` and `resize` already rebuilds `BoardView::new(config)`, so the
switch across 139 falls out of the existing resize path. No engine, AI,
economy, wager, save, profile or dependency change.

## What the code already gives us

- **The board block never depends on the width.** `BoardLayout::new` derives
  every board rect from `left = (cols - 89) / 2` and `top`; only the panel rect
  is placed off the board's right edge (`left + BOARD_WIDTH + PANEL_GAP`). So
  "the compact board is the same board" is a matter of not drawing one rect.
  `layout_regions_are_in_bounds_and_stacked_at_several_sizes` and
  `layout_grid_holds_twelve_slots_within_the_frame_and_halves` already run at
  89×31.
- **The status band is 81 columns wide** (`x0 = left + 4`, `x1 = left + 84`)
  and two rows: `draw_status` draws the over-20 alert on row 0 and the prompt
  on row 1, both `Align::Left`. The alert is 27 characters
  (`OVER 20!  (Space/D/S: bust)`); a six-digit stake line is 14
  (`Stake ◈ 999999`). Right-aligning the stake on row 0 leaves ≥ 40 blank
  cells between them.
- **The stake already reaches the board.** `App::draw` passes
  `self.profile.campaign().stake_at_risk()` (`None` for Quick Play) into
  `BoardView::draw`, which today only forwards it to `draw_presence_extras`.
- **Resize is one path.** `main.rs`: `Event::Resize` → `Config::fits` →
  `App::resize(config)` (rebuilds `board_view`, re-creates a cached help
  `Overlay`, clears `too_small`) or `set_too_small`. The too-small screen
  (`draw_too_small`) and the startup error (`Config::from_terminal`) both
  format `Config::min_size()`, so they quote `89 x 31` as soon as the constant
  changes. The play log and Records rebuild from `self.config` every draw.
- **The map at 89 clears the rail.** `CampaignMapLayout::new(89×31)`: rail
  `x0 = 67`, field `x1 = 65`, nodes at `x = 6 + round(fx·53)` → 9, 18, 18, 29,
  29, 39, 48, 54; the widest cursored label is `▸ THE SPINDLE ◂` (15) at
  `x = 48` → right edge 55, and `▸ ZENITH ◂` at 54 → 58. No label row shares
  another planet's node column, and no two labels share a row except Cinder /
  The Anvil (row 15, spans 4–13 and 33–45). Hand-derived — **the T001 test
  at 89 is the check** (§Tests).
- **The select preview fits at 89.** `preview_rect` is `center_x + 18` wide by
  `PANEL_W` = 22: at 89, x 62..83. The "clear of the list" assertion compares
  two offsets from `center_x`, so it is width-independent.
- **The briefcase is exactly 89 wide** (`BRIEFCASE_W = 2·43 + 3`): at 89
  columns `left = 0` and the Deck panel ends at column 88.
- **Overlays size to content and center.** The widest shipped overlay text is
  53 columns (first-match popup), the widest notice 72 (`Deck, collection, and
  progress reset…`), the wager prompt ≤ 70 (`every_line_fits_seventy_columns`):
  all boxes ≤ 80 ≤ 89.
- **Records caps its box at `cols - 8`** (81 at 89) and its widest line is the
  46-column opponent row, so it is never capped at 89.
- **The play log's box is `cols · 38 %` clamped to `[40, cols - 4]`**: 52 at
  139 (interior 48), but the 40 floor at 89 (interior 36) — narrower than the
  wide one, so lines the wide log shows (`Round 1: You win — You 20 / Greeb 18
  (stand)`, 45) would clip in the compact one. See tension 4.

## Design tensions resolved

### 1. The panel is an `Option<Rect>`, chosen where the geometry is built

`BoardLayout.opponent_panel` becomes `Option<Rect>`, `Some` iff `cols >=
WIDE_LAYOUT_MIN_WIDTH`. The choice lives in `BoardLayout::new`, the one place
the board geometry is built (startup via `App::new`, resize via `App::resize`),
so there is no second flag to keep in sync and no width check in `board.rs`
beyond `if let Some(panel)`. Rejected: a `wide: bool` beside a still-computed
rect (two fields saying one thing, and the rect would be off-frame at 89), and a
`Config::is_wide()` helper (a second place to evaluate the rule).

### 2. The stake line is the panel's string, drawn only when the panel is absent

`portrait.rs` gains `pub fn stake_line(stake: u32) -> String` (`Stake ◈ N`),
used by `draw_presence_extras` and by the compact arm — one string, so the
"panel's own form" claim can't drift. The compact arm draws it `Align::Right`,
`Emphasis::Strong`, on `layout.status` row 0, after `draw_status` has drawn the
alert `Align::Left` on the same row. At 139 and up nothing is drawn on the band
(spec: the wide layout is untouched). Banter is not drawn in compact — the
`banter` argument is accepted and ignored (spec non-goal), which keeps
`BoardView::draw`'s signature and `App::draw` unchanged.

### 3. `IN_MATCH_MIN_WIDTH` is renamed, and one test helper carries both widths

The constant's name would be false after this spec (the in-match minimum is
89). It becomes `WIDE_LAYOUT_MIN_WIDTH` — same value, same derivation, new doc
— and `Config::min_size()` returns `(BOARD_WIDTH, BOARD_BLOCK_HEIGHT)`. The
tests that used the old name for "the minimum" (`opponent_select.rs`,
`layout.rs`) switch to `Config::fit_sizes()`, a `#[cfg(test)] pub fn` returning
`[89×31, 139×31]`, so every fit test loops both without spelling numbers.
`wager.rs` is on the spec's no-change list: its
`the_prompt_fits_the_minimum_terminal_unclamped` reads `min_size()` and so runs
at 89×31 from T001 on without an edit; the 139 case is implied (a centered
content-sized box that fits 89 fits any wider frame). Stated here as the one
place the acceptance criterion's "as well as 139×31" is met by argument rather
than by a loop.

### 4. The play log's width floor becomes the wide box's width

`SCROLL_MIN_W` 40 → 52 (`139 · 38 / 100`), so the compact play log is the same
box as the wide one instead of a 40-column one that clips more transcript
lines. 52 + margins fits 89 (`cols - 4 = 85`). Widths 106–136 gain a slightly
wider box than today; nothing above 139 changes. The sizing moves into a pure
`scroll_box(config) -> Rect` so the equality is a test, not a walkthrough
observation. Pre-existing and **not** fixed here: round headers with a long
opponent name (`Round 1: The Magistrate wins — You 20 / The Magistrate 19
(stand)`, 66) already clip at 139's 48-column interior; the spec doesn't ask for
a wider log, only that the compact one show what the wide one shows. Flagged for
sign-off (§Open questions).

### 5. Records gets a pure content-size helper rather than a duplicated formula

`RecordsState::draw` computes the popup's content width inline. To test that
the box is never capped at 89 without copying the arithmetic into the test, the
measurement becomes `fn content_size(bodies: &[Vec<String>], coll: &str) ->
(usize, usize)`; `draw` calls it and the test does too. Smallest change that
makes the claim checkable.

### 6. The density rule — no conflict

The stake shares the alert's row, adds no row, and the board block keeps its
fixed 31-row height. The acted-on element on the board (the cursored hand card)
is unchanged.

## Design

### 1. `src/layout.rs` — the threshold and the optional panel

```rust
/// The width at which the board gains the opponent presence panel: the
/// centered board plus symmetric left/right panel margins (the left is
/// reserved empty for a future player panel). Below it the board draws
/// alone (spec 026); the terminal minimum is BOARD_WIDTH.
pub const WIDE_LAYOUT_MIN_WIDTH: usize = BOARD_WIDTH + 2 * (PANEL_GAP + PANEL_W); // 139

pub struct BoardLayout {
    …
    /// The opponent presence panel in the right margin, top-aligned with the
    /// board block — `Some` from WIDE_LAYOUT_MIN_WIDTH columns up, `None` on
    /// the compact board.
    pub opponent_panel: Option<Rect>,
}
```

In `new`: `let opponent_panel = (cols >= WIDE_LAYOUT_MIN_WIDTH).then(|| { let
panel_x0 = left + BOARD_WIDTH + PANEL_GAP; Rect::new(panel_x0, panel_x0 +
PANEL_W - 1, top, top + PANEL_H_INMATCH - 1) });`. Nothing else in `new`
changes. `Rect` gains `PartialEq, Eq` in its derive (for `assert_eq!` on
rects). The `BOARD_WIDTH` doc's "It is the minimum terminal width" sentence
becomes true again — keep it. Comments that say "139×31 minimum" (the
`CampaignMapLayout` clamp note, the briefcase `const _` message and `CELL_H`
doc) become "89×31". `config.rs` and `opponent_select.rs` are the only other
users of the old name.

### 2. `src/config.rs` — the minimum

```rust
use crate::layout::{BOARD_BLOCK_HEIGHT, BOARD_WIDTH, WIDE_LAYOUT_MIN_WIDTH};

/// The smallest terminal the layout supports, as (cols, rows): the fixed
/// board block, 89 × 31. From WIDE_LAYOUT_MIN_WIDTH columns the match adds
/// the opponent presence panel beside it (spec 026); wider/taller terminals
/// center the board and pad the margins.
pub fn min_size() -> (usize, usize) { (BOARD_WIDTH, BOARD_BLOCK_HEIGHT) }

/// The two sizes every "fits" test measures: the 89×31 minimum (compact) and
/// the 139×31 threshold (wide). Test-only.
#[cfg(test)]
pub fn fit_sizes() -> [Config; 2] {
    let (cols, rows) = Self::min_size();
    [Config { num_cols: cols, num_rows: rows }, Config { num_cols: WIDE_LAYOUT_MIN_WIDTH, num_rows: rows }]
}
```

`fits`, `from_terminal` unchanged. The test
`config_min_size_is_board_plus_panel_margins` becomes
`config_min_size_is_the_board_block`: `min_size() == (BOARD_WIDTH,
BOARD_BLOCK_HEIGHT)` and `== (89, 31)`, and `fit_sizes()` is `[89×31,
139×31]`.

### 3. `src/portrait.rs` — the shared stake string

```rust
/// The escrowed stake as the panel shows it — `Stake ◈ N`. One string for the
/// panel (spec 021) and the compact board's status band (spec 026).
pub fn stake_line(stake: u32) -> String { format!("Stake ◈ {stake}") }
```

`draw_presence_extras` uses it. Its doc's "the game's minimum terminal width"
(on `PANEL_W`) becomes "the wide layout's threshold". No other change; existing
portrait tests stand.

### 4. `src/board.rs` — the compact arm

```rust
/// Whether this view has the opponent presence panel (139 columns and up).
pub fn is_wide(&self) -> bool { self.layout.opponent_panel.is_some() }
```

In `draw`, the tail (after the popup) becomes:

```rust
match self.layout.opponent_panel {
    // Wide: the panel in the right margin, always visible (spec 016), with
    // banter, pips and the stake (specs 017, 021).
    Some(panel) => {
        draw_presence_panel(frame, panel, state.opponent_profile.name, state.opponent_profile.portrait);
        draw_presence_extras(frame, panel, banter, state.opponent.rounds_won, stake);
    }
    // Compact (spec 026): no panel, no banter, no pips — the header's
    // "Rounds won" carries what the pips showed — but a staked match keeps
    // its stake, right-aligned on the status band's upper row, where the
    // left-aligned over-20 alert leaves it 40+ cells of room.
    None => {
        if let Some(stake) = stake {
            draw_text_in(frame, self.layout.status, 0, Align::Right, &stake_line(stake), Emphasis::Strong);
        }
    }
}
```

`draw`'s doc says `banter` is shown only on the wide layout. Draw order matters
only in that `draw_status` (alert, `Align::Left`) runs first; the popup
(`popup_rect`, centered between header and hand, rows 10–16 at 31 rows) never
reaches the band (row 29), so the stake stays visible under the round-outcome
and game-over popups — pinned by the T002 test at `GamePhase::GameOver`.

### 5. `src/overlay.rs` — the play-log box

```rust
/// Content-width floor for the scrollable overlay: the box's width at the
/// 139-column wide threshold (139 · 38 %), so the compact play log (spec 026)
/// is the same box as the wide one instead of a narrower, clippier one.
const SCROLL_MIN_W: usize = 52;

/// The scrollable overlay's outer box for `config` — pure, so the width rule
/// is testable: `cols · 38 %` clamped to `[SCROLL_MIN_W, cols − 4]` wide,
/// `rows · 58 %` clamped to `[SCROLL_MIN_H, rows − 2]` tall, centered.
fn scroll_box(config: Config) -> Rect
```

`draw_scrollable_overlay` calls `scroll_box` and is otherwise unchanged.

### 6. `src/records.rs` — the content-size helper

```rust
/// The popup's content size across ALL views — widest of every body line, every
/// pager label, the collection line and the footer; tallest body — so the box
/// keeps one size as you page. Pure, so the fit test measures what `draw` does.
fn content_size(bodies: &[Vec<String>], coll: &str) -> (usize, usize)
```

`draw` replaces its inline `content_w`/`content_h` with a call; `FOOTER_MEASURE`
moves next to it. No drawing change.

### 7. `src/app.rs` — tests only

No code change: `resize`, `set_too_small`, `draw_too_small` and the draw arms
already do what the spec asks once `min_size()` and `BoardLayout` change. Two
tests are added (§Tests) and two fit tests loop `fit_sizes()`.

### 8. `Readme.md` — the terminal-size paragraph

Replace the paragraph at ~line 57 with: Kaazap needs a terminal at least
**89 × 31** (columns × rows) — the fixed board. At **139 columns or wider** the
match also shows the opponent's portrait panel beside the board; narrower than
that, the board draws alone and a staked match's stake sits on the status line.
The layout follows the terminal width, including a resize mid-match. Below
89 × 31 it shows the required size and exits (or, if resized below it while
running, pauses until the terminal grows back).

## Files

- `src/layout.rs` — `WIDE_LAYOUT_MIN_WIDTH` (renamed), `opponent_panel:
  Option<Rect>`, `Rect: PartialEq`, comment figures; tests.
- `src/config.rs` — `min_size()` = the board block, `fit_sizes()`; test.
- `src/portrait.rs` — `stake_line`; one doc line.
- `src/board.rs` — `is_wide`, the compact arm; tests.
- `src/overlay.rs` — `SCROLL_MIN_W`, `scroll_box`; tests loop both widths.
- `src/records.rs` — `content_size`; one fit test.
- `src/opponent_select.rs`, `src/shop.rs`, `src/app.rs` — tests only.
- `Readme.md` — the terminal-size paragraph.
- `specs/026-compact-layout/closeout-main-docs.md` (T005).
- **No change**: `main.rs`, `menu.rs`, `campaign_map.rs`, `campaign.rs`,
  `deck_builder.rs`, `play_log.rs`, `frame.rs`, `wager.rs`, `card.rs`,
  `game.rs`, `player.rs`, `save.rs`, `profile.rs`, `economy.rs`,
  `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`, the assets.

## Tests

Each claim names the task that owns its check.

- **The minimum is 89×31 and the threshold 139** (AC 1) — T001:
  `config_min_size_is_the_board_block` (`config.rs`) pins `(89, 31)` and
  `fit_sizes()`; `the_wide_threshold_is_the_board_plus_panel_margins`
  (`layout.rs`) pins `WIDE_LAYOUT_MIN_WIDTH == 139`.
- **The panel exists at 139 and not at 138** (AC 2) — T001:
  `the_panel_appears_at_the_threshold_and_not_below` replaces
  `board_and_panel_fit_the_minimum_terminal`: at 139×31 `opponent_panel ==
  Some(Rect::new(117, 138, 0, 19))` (spec 016's position: in bounds, strictly
  right of `opponent.hand.x1`, bottom ≤ row 30); at 138×31 and 89×31 `None`;
  at 180×48 `Some` and in bounds.
- **The board's rects are identical across the threshold apart from
  centering** (AC 2, design requirement) — T001:
  `board_rects_are_identical_across_the_threshold_apart_from_centering`: for
  `cols` in `[89, 138, 139, 180]` at 31 rows, take `left = player.header.x0 -
  H_PAD` and compare every board rect (`divider_x`, both `SideLayout`s'
  `header`/`grid`/`hand`, `status`) shifted by `-left` to the 89 case.
- **The map fits and reads at 89 and 139** (AC 7) — T001:
  `campaign_map_layout_fits_the_minimum_terminal` and
  `the_campaign_map_is_legible_at_the_minimum_terminal` loop
  `Config::fit_sizes()`; the assertions are unchanged (every node in the
  reduced field, every cursored label on frame and left of the rail, unique
  node cells, no label overlap, no label over another node). The 89 arithmetic
  in §What the code already gives us is the expectation; the test is the check.
- **The briefcase fits both widths** (AC 3) — T001: `briefcase_fits_the_minimum_terminal`
  loops `fit_sizes()`; `overlay_layout_pads_content_symmetrically` likewise.
- **The select preview is on frame at 89** (AC 7) — T001:
  `preview_panel_is_on_frame_and_clear_of_the_list_at_the_minimum` and
  `the_full_roster_and_footer_fit_the_minimum_terminal` loop `fit_sizes()`.
- **The stake is on the compact band, clear of the alert, for the whole
  match; never on the wide band** (AC 4, AC 5) — T002, `board.rs`:
  `the_compact_board_carries_the_stake_clear_of_the_alert`: `BoardView::new(89×31)`
  (`!is_wide()`), `over_20_game()` (PlayerTurn, score 25), draw with
  `stake = Some(999_999)` into `new_frame`; on row `layout.status.y0` the chars
  from `status.x0` read `OVER 20!  (Space/D/S: bust)` and the chars ending at
  `status.x1` read `Stake ◈ 999999` (`stake_line(999_999)`), the alert's last
  column is left of the stake's first column, and every cell between is blank
  (`' '`); row `status.y0 + 1` holds the prompt. Then with `game_phase =
  GameOver { winner: Player }` the same row still ends in the stake line (the
  popup doesn't reach it). `quick_play_shows_no_stake_line`: `stake = None` →
  no `Stake` on either status row. `the_wide_board_keeps_the_stake_in_the_panel`:
  `BoardView::new(139×31)` (`is_wide()`), same draw → no `Stake` on the status
  rows, and the panel row `panel.y0 + 18` contains `◈ 999999`.
- **Emphasis** (design requirement) — T002: the stake cells on the compact row
  carry `Emphasis::Strong` (read `frame[x][y].emphasis`).
- **Every overlay text fits both widths unclamped** (AC 3) — T003:
  `help_texts_fit_the_minimum_terminal_unclamped`,
  `onboarding_texts_are_the_spec_text_and_fit` (`overlay.rs`),
  `the_campaign_entry_panel_fits_the_minimum_terminal`,
  `both_notices_read_right_breathe_and_fit_the_minimum_terminal` (`app.rs`),
  `the_full_list_fits_the_minimum_terminal` (`shop.rs`),
  `turn_hints_fit_the_status_band` (`board.rs`) loop `Config::fit_sizes()`.
- **The play log's box is the same width at 89 and 139** (tension 4) — T003:
  `the_play_log_box_is_the_same_width_compact_and_wide`: `scroll_box(89×31)`
  and `scroll_box(139×31)` both 52 wide and in bounds (`x1 < cols`); at 200×48
  it is wider (the percentage still rules above the floor).
  `scrollable_overlay_pins_to_bottom_and_clamps_overscroll` loops `fit_sizes()`.
- **Records is never capped at 89** (AC 3) — T003:
  `the_records_popup_is_never_capped_at_the_minimum_terminal`: a
  `LifetimeStats` with three-digit counts on one opponent in both modes
  (`record_match` ×N or a loop), `record_campaign_completion(999)` so the
  first-clear suffix draws, and a `RunStats` with a streak; `content_size`
  over all four `view_body`s and `collection_line(15, 15)`; for each
  `fit_sizes()` config, `content_w + 10 <= cols - 8` and `content_h + 8 <= rows
  - 4` (the `draw` clamps, cross-referenced in a comment).
- **The wager prompt fits** (AC 3) — no task: `wager.rs` unchanged; its test
  reads `min_size()` and so measures 89×31 from T001 on (tension 3).
- **A resize across 139 keeps the match and toggles the panel** (AC 6) — T004,
  `app.rs`: `a_resize_across_the_threshold_keeps_the_match_and_toggles_the_panel`:
  `App::new(139×31)`, `screen = InGame { GameState::new() boxed, HandCursor
  after one move_right }`, `modal = Help(Overlay::new(GameHelp, 139×31))`;
  record the cursor index and `player.hand`; `resize(138×31)` → `!is_too_small`,
  `!board_view.is_wide()`, still `InGame` with the same index and hand, modal
  still `Help` of kind `GameHelp`; `draw` into a 138×31 frame (no panic);
  `resize(139×31)` → `is_wide()` and the same checks again.
- **The too-small screen quotes 89 x 31** (AC 1) — T004:
  `the_too_small_screen_quotes_the_new_minimum`: `draw_too_small` into a 40×10
  frame; some row contains `Need at least 89 x 31`.
- **The wide layout is unchanged** (AC 8) — structural: `BoardLayout::new`'s
  board arithmetic is untouched (T001 diff), the existing layout tests pass
  untouched, `git diff main...HEAD -- src/portrait.rs` adds only `stake_line`
  and its use (T005); plus the 139×31 driver walkthrough (T004).
- **Nothing clipped at 89 on every screen** (AC 3, Goal 1) — driver walkthrough
  at 89×31, T004.
- **No forbidden change** (AC 10) — T005: `git diff main...HEAD --stat` lists
  none of `card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`,
  `economy.rs`, `wager.rs`, `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`;
  `PROFILE_VERSION`/`SAVE_VERSION` unchanged.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim.
- **Driver walkthroughs after T004** (the orchestrator, `run-kaazap` skill; the
  real profile, settings and save backed up and checksum-restored afterwards),
  at **89×31** and **139×31**:
  - 89×31: menu, `?` help, How to Play, Settings, Records (all four views),
    opponent select (preview panel right of the list, every row and the blurb on
    frame), campaign map (all eight planets and labels on frame, none on the
    rail; header credits and the info panel readable), Outfitter, deck builder
    (both panels, title, readout, hint), a staked campaign match: wager prompt
    (with the all-in warning row), then the board with `Stake ◈ N` at the right
    of the status band's upper row from the first frame, through an over-20
    moment (alert left, stake right, no collision), a round-outcome popup and
    the game-over popup; the play log (`L`) at its 52-column box; a Quick Play
    match with no stake line. Nothing clipped, no panel, no banter line.
  - 139×31: a staked match shows spec 016's panel with portrait, banter, pips
    and stake, and **no** stake on the status band; the board's cards are in
    the same columns relative to the board as at 89.
  - Resize mid-match (the driver's terminal, or a real terminal by hand): 139 →
    138 drops the panel and shows the band stake on the next frame; 138 → 139
    restores the panel; the hand cursor, an open `?` overlay and the play log
    survive both. Below 89×31: the too-small screen reads `Need at least 89 x
    31` and play resumes on growing back.
  - Report the play-log clipping of long round headers if seen (pre-existing,
    tension 4) as a finding, not a blocker.

## Non-goals (from spec)

No third layout between 114 and 138; no portrait, name panel or banter line on
the compact board; no setting to force the compact board; no change to the
wide layout; no player-status panel; no engine, AI, economy, wager, save or
balance change.

## Open questions

None product-level. Settled here as design and flagged for sign-off:

1. **`IN_MATCH_MIN_WIDTH` → `WIDE_LAYOUT_MIN_WIDTH`** (tension 3) — a rename
   the spec doesn't ask for, made because the old name would be false.
2. **The play-log box floor rises from 40 to 52** (tension 4) — so the compact
   log is the wide log's box; widths 106–136 get a slightly wider box than
   today. The pre-existing clipping of long round headers at 139 is left alone.
3. **`wager.rs`'s fit test is not looped** (tension 3) — the file is on the
   spec's no-change list; it measures 89×31 via `min_size()` and the 139 case
   follows.
4. **Records gains a pure `content_size`** (tension 5) — a small extraction so
   the records fit test the acceptance criteria name can exist.
