# Plan: Play log — spec 018

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

## Context

The board shows the current *state* of a round but not the *sequence* that produced it —
especially the opponent's timer-driven moves. This spec adds an in-match, on-demand **play
log**: a toggleable bordered overlay (`L` to open/close, `Esc` to close) drawn over the board
in the monochrome overlay vocabulary, listing (1) a running **round-outcomes** list for the
match and (2) **this round's moves** in order. It does not pause the game — the opponent's
think timer keeps running and the log updates live under the overlay.

This is a **read-only observer over a settled state machine**. **No engine phase, no AI change,
no save-format change, no new or changed card behavior.** The log reads game state that already
exists (`dealer_row`/`played_row` growth, `stood`/`bust` flags, `round_outcome`,
`GamePhase::GameOver`) and reconstructs the move sequence by diffing successive `GameState`s —
exactly the technique spec 017's banter and spec 004's audio already use. The log is transient
`App` state, **never serialized**.

This is the project's **first *dynamic* overlay.** The three existing overlays
(`OverlayKind::{GameHelp, MenuHelp, HowToPlay}`) are static text loaded from a bundled `.txt`
via `include_str!` (`overlay.rs` `read_text_from_file`). The play log renders live per-match
state instead — so it reuses the box/measure/layout machinery (`measure`, `OverlayLayout`,
`draw_box`, `draw_text_in`) but is fed a `Vec<String>` built each frame from the current
`PlayLog` + `GameState`, not a file. See tension §4.

## What the code already gives us

- **The event-detection-by-diff pattern is established (twice).** `audio.rs`
  (`AudioSnapshot::of` → `audio_cues`, driven by `App::emit_audio_cues`, `app.rs:536`) and
  `banter.rs` (`BanterSnapshot::of` → `banter_event`, driven by `App::update_banter`,
  `app.rs:553`) both diff a snapshot of a few `GameState` facts each tick. **Crucially,
  `update_banter` is called from *both* call sites** — after input in `handle_key`
  (`app.rs:860`) **and** every frame in `tick` (`app.rs:1136`). That twin call is exactly why
  banter reacts to opponent moves that resolve on the think timer inside `tick` rather than on a
  keypress. **The play log's capture must follow the same twin-call discipline**, or it will
  miss the opponent's timer-driven moves — the very moves the spec says are hardest to follow.
- **Match-start / resume / rematch lifecycle hooks already exist.** Fresh matches seed
  `prev_banter = None` at `start_match` (`app.rs:524`); resume blanks it at `Continue`
  (`app.rs:1037`); a rematch (`new_game` in place after game over) is detected in the diff via
  `game_over` true→false (`banter::match_restarted`). The play log's per-match reset mirrors
  these exactly.
- **The move data is fully on `GameState`.** A dealer draw pushes a `PlayedCard` to
  `dealer_row` (`game.rs:416`/`704`); a hand play pushes a `PlayedCard { card, value }` to
  `played_row` via `commit_play` (`game.rs:227`) carrying the resolved signed `value` (a ± or
  tiebreaker committed at its chosen sign; a flip at `value: 0`); `stood`/`bust` are per-side
  flags; `finalize_round` (`game.rs:302`) sets `round_outcome` `None`→`Some` and the busts.
  `PlayedCard::display_text()` (`card.rs:93`) already formats a played card's identity + resolved
  sign / flip label (`+4`, `-3`, `2&4`, `+1T`) — the log reuses it verbatim. `PlayerState::score()`,
  `table_full()`, and `crate::MAX_TABLE_CARDS` (= 12) give totals and the filled-table signal.
- **The Modal system already overlays the board without pausing it.** `App` holds one
  `Option<Modal>` (`app.rs:253`); `handle_key` routes all input to the open modal
  (`app.rs:626`), and `draw` renders the screen then the modal over it (`app.rs:1160`).
  **`tick` never consults `self.modal`** (`app.rs:1080`) — so the game keeps advancing while any
  modal is open. A `Modal::PlayLog` therefore gives "no pause, live update" for free: the board
  ticks underneath, and the overlay is rebuilt from live state each frame.
- **Monochrome is structural.** `frame.rs` has no color path; every drawer takes an `Emphasis`,
  never a color. The log renders as characters + `Emphasis::Normal`, so monochrome holds by
  construction.
- **The reserved left margin is geometry, not this overlay's concern.** `IN_MATCH_MIN_WIDTH`
  (`layout.rs:39`) reserves an empty left margin for the future player-status panel (spec 016).
  The play log is a **centered** overlay sized by `OverlayLayout::new` (`layout.rs:654`, clamped
  in-bounds) — it is not a left-margin panel and claims no reserved space.

## Design tensions resolved

Technical choices (mine to make); each stated with its reason.

### 1. Where the log lives — a `PlayLog` on `App`, transient, never saved

The log is presentation/observation, not game state, and must not reach the save file. It lives
on `App` as a single owned field `play_log: PlayLog` (always present; empty outside a match),
alongside `prev_audio` / `prev_banter` as transient in-game state. `PlayLog` owns its
accumulated `round-outcomes` list, its per-round `moves` list, its opponent-name label, and its
own diff seed (`Option<PlayLogSnapshot>`). It is never added to `save.rs`'s serialized types.
Whether the overlay is *open* is separate transient UI state — a `Modal::PlayLog` variant
(tension §3), not a field on `PlayLog`.

### 2. Capture strategy — a per-tick delta diff, not engine recording hooks (the central design)

The spec's "natural recording points are the `apply_*_action` methods" is a *why-it's-cheap*
observation, **not** a license to edit `game.rs`; the constitution forbids drawing/observing
code from mutating state and this spec forbids engine change. So — as banter and audio do — the
log **observes from the outside** by diffing successive `GameState`s. Banter fires *one* event
per transition; the play log must capture *every* discrete move as an ordered list, so the diff
is richer. It rests on one invariant, which the tests pin:

> **At most one card is added to one side per capture.** A keypress funnels through a single
> `apply_game_action` (one `player_hit` / `play_card` / `commit_sign_choice`, then
> `resolve_after_action`, which adds no cards); a `tick` runs one `update()` step (`OpponentTurn`
> plays exactly one `opponent_hit` / `commit_play` / `opponent_stand`). `resolve_after_action`
> and `finalize_round` never push cards — they only flip `stood`/`bust` and set `round_outcome`.

Given that invariant, a diff of `(prev snapshot) → (current GameState)` reconstructs the moves
between them exactly:

- **New dealer cards** — indices `prev.dealer_len .. curr.dealer_row.len()` on each side →
  `Move::Draw`. (≤ 1 per side per diff.)
- **New played cards** — indices `prev.played_len .. curr.played_row.len()` → `Move::Play`,
  carrying the `PlayedCard` (identity + resolved `value` / flip label).
- **Stand** — `!prev.stood && curr.stood` → `Move::Stand`.
- **Bust** — `!prev.bust && curr.bust` → `Move::Bust`. (This is how a live over-20 side busted
  at `finalize_round`, or a standing side flipped over 20, surfaces as a visible bust.)

Because a played `±`/tiebreaker only lands on the board when its sign is *committed*
(`commit_sign_choice`) — `AwaitingSignChoice` pushes nothing — the log records the play once,
with the resolved sign, and never records the intermediate prompt. This satisfies "record moves
as they happen, once already visible on the board," including "never reveal an opponent hand card
before it's played" (the opponent's card is on `played_row` the same tick it plays, never before).

### 3. The overlay is a `Modal::PlayLog`, not a new mechanism — and it does not pause the game

A panel opened *over* `Screen::InGame` and dismissed back to it is, by the constitution's own
line, an **overlay/modal** — the same category as How to Play. So the open/closed state is a new
`Modal::PlayLog` variant (unit-like — it carries no data; content is rebuilt from live state each
draw). This buys "no pause" for free: `tick` ignores `self.modal`, so the opponent's timer keeps
running and its moves are captured live while the log is up. Consistent with every other modal,
`Modal::PlayLog` **captures input while open** — the player cannot play a card until the log is
closed, but the *game* is not paused (the opponent still moves, the log still updates). The spec's
"a move made while it's open appears immediately" refers to those timer-driven opponent moves, and
they do. If a sign choice was pending when the log opened, the phase is untouched and closing
returns to it.

### 4. First dynamic overlay — a content-driven draw path beside the static one

`Overlay` couples `OverlayKind` → a bundled `.txt` via `read_text_from_file`; the play log has no
file. Rather than shoehorn dynamic content into `Overlay`, extract the box machinery `draw_overlay`
already runs (`measure` → `OverlayLayout::new` → `clear_rect` → `draw_box` → per-line
`draw_text_in`) into a **public free function** `overlay::draw_text_overlay(config, &[String],
frame)`, and have `Overlay::draw_overlay` call it. `Overlay` and the three static overlays are
behaviorally unchanged (same measure/layout/border/first-line-centered-title convention). The play
log builds its `Vec<String>` from `PlayLog::render_lines` and calls `draw_text_overlay` directly.
Because the content is rebuilt from live state and `self.config` every frame, `Modal::PlayLog`
carries no cached `Config` and needs **no resize rebuild** — unlike `Modal::Help`, whose
`resize` arm (`app.rs:590`) rebuilds a cached `Overlay`; the play log's arm is simply absent, which
is correct.

### 5. Ordering and totals within a diff

Moves within a single diff are emitted **player side, then opponent side**, and within a side
**draw/play, then stand, then bust** — the causal order (a draw fills the table → auto-stand →
bust; a player flip → the standing opponent busts). Only the acting side adds a card, and only a
player flip can bust the *other* side, so player-then-opponent is always chronologically correct.

Each move's **resulting total is `side.score()` read from the current `GameState`** at capture.
This is exact because of the one-card invariant (§2): standing and busting don't change a score,
and at most one card was added, so `score()` equals the total immediately after that move. A flip
play's resulting total is the acting side's post-flip `score()`; a flip-induced opponent bust
shows the opponent's post-flip `score()` — both correct.

### 6. Round-outcome resolution — a precedence classification (bust > filled-table > stand)

The spec's round-outcome entry carries the winner/tie, **both** final totals, and **how it
resolved** — one of: a bust (which side), a stand, or a filled-table auto-stand. A round can end
with mixed causes (one side stands manually, the other auto-stands on a full table), so "how it
resolved" needs a single, defined classification. Computed from the `GameState` at the
`round_outcome` `None`→`Some` transition (rows still intact — they clear only on the next
`NextRound`), by precedence:

1. **both busted** → `Resolution::BothBust` (a tie);
2. **one side busted** → `Resolution::Bust(side)`;
3. else **a side filled the table** (`player.table_full() || opponent.table_full()`, i.e.
   `table_card_count() >= MAX_TABLE_CARDS`) → `Resolution::FilledTable`;
4. else **both stood** → `Resolution::Stand`.

This precedence (a bust is the most salient way a round ends; a full table next; a plain stand
last) is a stated design decision, not a spec-settled fact — it is pinned by a test (T002) so it
can't drift. Both final totals are `player.score()` / `opponent.score()` at that transition (the
raw totals, including a busted side's over-20 value — informative, e.g. "25, bust").

### 7. Reset lifecycle — explicit hooks plus in-diff detection

Mirroring banter's seeding, `PlayLog::reset()` (clears both lists, sets the diff seed to `None`
so the first observe seeds silently, records the opponent name) is called at the two match-entry
sites: `start_match` (`app.rs:512`, beside `prev_banter = None`) and `Continue`/resume
(`app.rs:1027`). A resumed match therefore opens with an empty log and, because the first observe
only seeds (emits nothing when the prior snapshot is `None`), logs only from resume onward — the
restored cards on the board are never back-logged.

Within a live match, two transitions are detected in the diff itself:

- **New round → clear the move list.** `setup_next_round` (`game.rs:795`) empties all four rows
  and resets `round_outcome` to `None`; nothing else empties a row (draws/plays only grow them).
  So "all four rows empty in `curr` while `prev` had any non-empty" is a clean signal that a new
  round began → `moves.clear()`. The round-outcomes list is untouched.
- **New match / rematch → clear both lists.** A rematch (`new_game` in place after game over) is
  the only in-game `game_over` true→false transition (`match_restarted(prev, curr)`), the same
  signal banter uses. On it, `moves` **and** `outcomes` clear. (`start_match`/resume already
  hard-reset via `reset()`.)

Because a reset diff empties rows (adds nothing) and a move diff adds to rows (empties nothing),
the reset branches and the move-append branch are mutually exclusive per diff; the append slices
are always valid (`curr.len >= prev.len`) outside a reset.

### 8. Overflow — show the most recent moves that fit

The board caps a round at `MAX_TABLE_CARDS` (12) per side, so a round's move count is bounded
(~ up to ~26 move lines plus a handful of outcome lines). `OverlayLayout::new` already clamps the
box in-bounds. If the content would exceed the available inner height, `render_lines` keeps the
title, the round-outcomes section, the two section headers, and the **most recent** move lines
that fit (dropping oldest moves) — the spec's stated acceptable degradation. Kept simple and
pure (a small trim helper takes the inner-height budget), tested at T003.

## Design

### 1. `src/play_log.rs` — capture + summarization + rendering (new module, foundational)

New module (added to `lib.rs`), pure logic + formatting, no dependency on rendering internals:

```rust
pub enum Move {
    Draw  { side: Player, value: i8, total: i32 },
    Play  { side: Player, card: PlayedCard, total: i32 }, // card.display_text() = sign/flip resolution
    Stand { side: Player, total: i32 },
    Bust  { side: Player, total: i32 },
}

pub enum Resolution { Bust(Player), BothBust, FilledTable, Stand }

pub struct RoundSummary {
    outcome: RoundOutcome,
    player_total: i32,
    opponent_total: i32,
    resolution: Resolution,
}

// Minimal diff seed — lengths + flags + the two match-lifecycle bits. Totals and
// card identities are read from the live GameState in `moves_since`.
struct PlayLogSnapshot {
    p_dealer_len: usize, p_played_len: usize, p_stood: bool, p_bust: bool,
    o_dealer_len: usize, o_played_len: usize, o_stood: bool, o_bust: bool,
    game_over: bool,
    outcome_present: bool,
}

pub struct PlayLog {
    outcomes: Vec<RoundSummary>,   // accumulates across the match
    moves: Vec<Move>,              // per-round; cleared at each new round
    opponent_name: String,         // side label for rendering
    prev: Option<PlayLogSnapshot>, // diff seed; None = seed silently next observe
}
```

- `PlayLogSnapshot::of(gs: &GameState)` — reads the eight per-side lengths/flags,
  `matches!(gs.game_phase, GamePhase::GameOver { .. })`, and `gs.round_outcome.is_some()`.
- **Pure** `fn moves_since(prev: &PlayLogSnapshot, gs: &GameState) -> Vec<Move>` — the §2/§5
  delta reconstruction, in player-then-opponent, draw→stand→bust order. Testable with a hand-built
  `GameState` + `prev`.
- **Pure** `fn match_restarted(prev, curr) -> bool` = `prev.game_over && !curr.game_over`;
  `fn round_reset(prev, gs) -> bool` = all four `curr` rows empty && some `prev` len > 0.
- **Pure** `fn summarize_round(gs: &GameState) -> RoundSummary` — the §6 classification.
- `PlayLog::reset(&mut self, opponent_name: &str)` — clears both lists, `prev = None`, stores name.
- `PlayLog::observe(&mut self, gs: &GameState)` — the orchestrator: on `Some(prev)`, if
  `match_restarted` → clear both; else if `round_reset` → clear moves; else append
  `moves_since(prev, gs)` and, on `outcome_present` `false`→`true`, push `summarize_round(gs)`.
  Always store `prev = Some(of(gs))`.
- **Pure** `PlayLog::render_lines(&self, inner_height_budget: usize) -> Vec<String>` — builds the
  overlay content: title (line 0, centered by the overlay convention), a "Round outcomes" header
  + one line per `RoundSummary`, a blank, a "This round" header + one line per `Move`, applying
  the §8 most-recent-that-fit trim to the move lines. Side label: `Player::Player` → "You",
  `Player::Opponent` → `self.opponent_name`. Each move line contains the acting side label, the
  value / card text (`PlayedCard::display_text()` for a play) and the resulting total; each
  outcome line contains the winner/tie, both totals, and the resolution. Exact punctuation is a
  monochrome-text cosmetic detail; tests assert the required substrings are present.

### 2. `src/overlay.rs` — a public content-driven draw path (first dynamic overlay)

Extract the body of the private `draw_overlay` into
`pub fn draw_text_overlay(config: Config, content: &[String], frame: &mut Frame)` (measure →
`OverlayLayout::new` → `clear_rect` → `draw_box` → first-line-centered / rest-left `draw_text_in`).
`Overlay::draw_overlay` becomes a one-line call to it. `measure`, the title convention, and the
three static overlays are byte-for-byte unchanged in behavior.

### 3. `src/app.rs` — field, reset hooks, twin-call capture, toggle, draw

- **Field**: `play_log: PlayLog` (grouped with `prev_banter` as transient in-game state; init
  `PlayLog::default()` — empty).
- **`Modal::PlayLog`** variant (unit-like) added to the `Modal` enum (`app.rs:253`).
- **Capture**: `fn update_play_log(&mut self)` mirroring `update_banter` — early-return unless
  `Screen::InGame`, else `self.play_log.observe(game_state)`. Called right after `update_banter`
  at **both** sites: `handle_key` (`app.rs:860`) and `tick` (`app.rs:1136`).
- **Reset**: `self.play_log.reset(<opponent name>)` at `start_match` (`app.rs:512`, using the
  opponent's name/id before the profile is moved into the game state) and at `Continue`
  (`app.rs:1027`, using `game.opponent.name`).
- **Toggle / routing** in `handle_key`: when `Modal::PlayLog` is open, `L` or `Esc` closes it
  (play the back cue, consistent with Help). When no modal is open and the screen is
  `Screen::InGame`, `L` opens `Modal::PlayLog` (`m`-mute stays ahead of it, unchanged). Capital
  `L` only — lowercase `l` remains the sign-minus binding (`game.rs:130`), untouched; `L` is
  otherwise unbound.
- **Draw**: a `Some(Modal::PlayLog)` arm in the modal-draw match (`app.rs:1160`) that, when the
  screen is `Screen::InGame`, builds `self.play_log.render_lines(<inner height budget from
  self.config>)` and calls `overlay::draw_text_overlay(self.config, &lines, frame)`.
- **Resize**: no new arm needed (see §4) — `self.config` is already updated in `resize`
  (`app.rs:587`) and the content is rebuilt each draw.

## Files

- `src/play_log.rs` — **new**: `Move`, `Resolution`, `RoundSummary`, `PlayLogSnapshot` + `of`,
  `moves_since`, `match_restarted`, `round_reset`, `summarize_round`, `PlayLog` +
  `reset`/`observe`/`render_lines`; logic + formatting tests. Added to `src/lib.rs`.
- `src/overlay.rs` — extract `pub fn draw_text_overlay`; `Overlay::draw_overlay` delegates to it.
  Static overlays unchanged.
- `src/app.rs` — `play_log` field, `Modal::PlayLog`, `update_play_log` (two call sites), two reset
  sites, the `L`/`Esc` toggle routing, the modal-draw arm.
- `DECISIONS.md`, `ROADMAP.md` — close-out (play-log design record; roadmap item shipped).
- **No change**: `game.rs`, `player.rs`, `card.rs` (reused read-only), `save.rs` (log never
  saved), `board.rs`, `render.rs`, `frame.rs`, `layout.rs`, `banter.rs`, `audio.rs`, `opponent.rs`.

## Tests

Each behavioral claim names the task that owns its check. Two are human-attested (marked).

- **Delta diff reconstructs each move once, in order** (T001): from a `prev` snapshot and a
  `GameState` mutated by one move, `moves_since` yields exactly that move with the right side,
  value/card, and resulting total — for a dealer draw, a fixed-value play, a `±`/tiebreaker play
  committed at each sign, a flip play (identity + post-flip total), a stand, and a bust; a
  keypress-then-resolve that draws-fills-stands-busts in one diff yields draw→stand→bust in that
  order; a player flip that busts the standing opponent yields the player-play then the
  opponent-bust; a no-change diff yields nothing. Guards "lists the current round's moves in
  order… value/card and resulting total… for both player and opponent."
- **Resulting total equals the post-move score** (T001): each emitted move's `total` equals the
  acting side's `score()` on the mutated `GameState`. Guards the one-card invariant (§2/§5).
- **Round-outcome summary & resolution classification** (T002): `summarize_round` yields the
  right `RoundOutcome`, both totals, and the `Resolution` under the §6 precedence — a lone bust,
  a both-bust tie, a full-table auto-stand (no bust), and a plain double-stand. Guards "each entry
  shows the winner (or tie), both sides' final totals, and how the round resolved."
- **Lifecycle** (T001 for moves, T002 for outcomes): after `observe`ing a completed round then a
  `setup_next_round`-style empty state, `moves` is empty while `outcomes` still holds the prior
  round; after a `new_game`-style `game_over` true→false, both lists clear; a first `observe` from
  `prev = None` (fresh or resumed mid-match state) emits nothing (seeds silently). Guards "the move
  list clears at a new round while outcomes persist," "a new match/rematch clears both," and "a
  resumed match opens empty and logs from resume onward."
- **Rendering** (T003): `render_lines` places the title first, a round-outcomes section then a
  this-round section, labels the player "You" and the opponent by name, includes each required
  substring (side/value/total for moves; winner/totals/resolution for outcomes), and — given a
  height budget smaller than the content — keeps the headers + outcomes + the most recent move
  lines that fit (oldest dropped). Guards the two-section layout, the "distinguished by name"
  requirement, and the §8 overflow rule.
- **Static overlays unchanged** (T004): `overlay::measure`'s existing tests still pass; the three
  `OverlayKind`s still render via `draw_text_overlay` with the title-centered/rest-left convention
  (exercised by running). Guards "consistent with the existing overlays… bordered, monochrome,
  self-sizing/centered."
- **`L` toggles; game keeps ticking; capture runs at both sites** — *driven, not unit-tested.*
  Verified by playing a match (T007): `L` opens the log in-match and `L`/`Esc` close it; while it
  is open the opponent's timer-driven moves appear live without reopening; the current round's
  moves (both sides, every kind incl. sign/flip) and the round-outcomes list read correctly; a new
  round empties the moves while outcomes persist; a rematch clears both; a resumed match opens
  empty and logs from resume on; nothing renders in color; the overlay is centered and claims no
  left margin. **Marked needing verification** (T007 driver / person attestation).
- **No engine/save change** — structural: no edit under `game.rs`/`player.rs`/`card.rs`/`save.rs`;
  `PlayLog` is never serialized. **Confirmed by the diff at the sweep** (T008).

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25` — no new
  warnings, reported verbatim per the constitution.
- **Driver / person attestation** (back up + checksum-restore the real profile/saves first, per
  the standing data-safety practice): the T007 checklist above, snapshotted at 139×31 and wider,
  in both a Quick Play and a Campaign match.

## Non-goals (from spec)

Not persisted, exported, or a post-match summary screen; not a full cross-round transcript
(completed rounds collapse to their outcome); no always-on panel and no claim on the reserved
left margin; no filtering/search/heavy scrolling (bounded round, most-recent-that-fit on
overflow); no color (monochrome per spec 002); no new engine phase, AI change, or card-behavior
change.

## Open questions

None. The one point the spec leaves to interpretation — how a mixed-cause round is classified as
a single "how it resolved" value — is a technical detail resolved in tension §6 (bust >
filled-table > stand, test-pinned), not a product fork.
