# Plan: Animation pass — spec 027

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-18; re-review after B1 and N1, N2, N4; R1 applied by the orchestrator)
**Implements**: `spec.md` in this directory

## Context

Today the only mover on the match board is the selection pulse: `App` ticks one
`SelectionPulse` and every screen draws its cursored element at
`pulse.emphasis()`. Everything else lands on the frame of the state change that
caused it. This spec adds a second, one-shot kind of motion — an element drawn
`Strong` for a fixed beat after it changed, the round/game popup held back for a
beat, and a stepping `.`/`..`/`...` on the opponent's thinking line — governed by
an **Animations** row in Settings.

It is a **presentation-only spec**. The whole of it is: one new module,
`src/motion.rs`, holding a `BoardMotion` struct that `App` feeds from the game
loop's tick exactly as it feeds the banter and audio observers (a snapshot diff,
spec 017); three timing constants in `lib.rs`; `board.rs` reading that struct
when it draws; a `bool` on `Settings` with a third overlay row; one README
sentence and one sentence in `design/brief.md`. `game.rs`, its phases, its
timing and its keys are not touched — the transitions observe the state after
the fact and never gate a key or a phase.

## What the code already gives us

- **A loop that ticks, then draws.** `main.rs`: one key event at most
  (`handle_key`), then `app.tick(dt)`, then `app.draw`, then a 50 ms sleep. So
  an observer run at the end of `tick` sees every change — key-driven or
  `update()`-driven — before the frame that shows it. `emit_audio_cues`,
  `update_banter` and `update_play_log` already sit there. `tick` receives `dt`,
  so the beats can be counted in `Duration`s the way `SelectionPulse` does,
  with no `Instant` and no sleeping in tests.
- **The snapshot-diff pattern.** `BanterSnapshot::of(&GameState)` plus a
  `prev: Option<…>` on `App`, seeded silently on the first observation
  (`prev_banter = None` at match entry and on Continue). `BoardMotion` keeps its
  own `prev` inside the struct because it also owns the clocks.
- **Emphasis is per cell and per `CardView`.** `CardView { emphasis, weight }`
  applies its emphasis to border and face; `draw_text_in` takes an `Emphasis`.
  Drawing a card or a figure `Strong` is a one-field change. The four levels are
  the only attributes the renderer emits (`Emphasis::attribute`).
- **Resolution has a single observable.** `draw_round_outcome_text` draws
  something exactly when `round_outcome.is_some()` or the phase is `GameOver`;
  `finalize_round` sets both on the tick after `RoundEnd`, and
  `setup_next_round`/`new_game` clear them. "The round resolved" is the
  false→true edge of that same condition.
- **The opponent's pause is one phase.** `OpponentThinking { until }` is
  entered from `player_hit`, `player_stand`, `update`'s `PlayerTurn` arm and
  `play_opponent_turn`'s over-20 branch, and leaves via `update` to
  `OpponentTurn` (one tick) and back. `status_message` returns
  `("Opponent's Turn", Muted)` only in that phase.
- **Rows only grow within a round.** `dealer_row`/`played_row` are pushed by
  hits and plays and reset to empty by `setup_next_round`/`new_game`; the
  played row fills from the back, so a card's index in its row is a stable
  identity until the clear. `score()` is the sum of both rows, so a total
  changes only when a row does.
- **The status band's rows never meet.** `draw_status` puts the alert on row 0
  and the base prompt on row 1; the compact stake sits at the right of row 0
  (spec 026). The thinking line is the base prompt, on row 1. The band is 81
  wide; `Opponent's Turn ...` is 19.
- **Settings is built to grow.** `Settings` derives serde with
  `#[serde(default = …)]` per field; `SettingsState::move_up/down` carry a
  comment saying to generalize to an index at the third row; `draw_overlay`
  sizes its box to the widest row and a fixed content height.
- **The Score is drawn `Strong` today** (`draw_side_header`, both sides,
  always). See tension 1.

## Design tensions resolved

### 1. The resting Score becomes Normal — a stated deviation from Goal 5's wording

A `Strong`-for-a-beat transition on a figure that is already `Strong` at rest is
invisible, and the only stronger level, `Alert` (inverse), is rationed to
interrupts by the brief. Spec AC 3 ("a Score that did not change is never
Strong") and AC 8 ("a settled draw: … no Strong score") both describe a Score
that rests `Normal`, so that is what this plan builds: `draw_side_header` draws
`Score: N` `Normal`, and `Strong` only while `Elem::Score(side)` is in
transition. Consequence: with Animations **Off**, the board is the pre-spec
board **except that the Score is no longer bold**. Goal 5 and §The Animations
setting say "frame for frame" / "identical to the game's frames before this
spec"; the ACs are what the tests pin, and they say otherwise. **Ruled: spec
Q7, 2026-09-18 — the Score rests `Normal`**; flagged to the person in the
conformance summary (§Open questions 1).

### 2. One struct, observed in `tick`, read by `draw`

`BoardMotion` lives in `src/motion.rs` (pure logic + tests, like `banter.rs`;
no rendering import) and is a field on `App`. `App::tick` calls
`motion.observe(&game_state, dt)` when the screen is `InGame` and resets it to
`BoardMotion::default()` otherwise — that one `match` is the spec's "discarded
when the board leaves the screen" and its "first frame … drawn settled" (the
next observation after entering a match seeds silently, starting nothing).
`App::draw` passes `self.settings.animations.then_some(&self.motion)` to
`BoardView::draw`; `None` is the settled draw. Rejected: a per-element
`Instant` map (untestable without sleeping), a `GamePhase` extension (the spec
forbids it and the engine must not know), a separate `Animations` flag inside
`BoardMotion` (the board would have two ways to be settled).

### 3. Clocks count down; the popup's is a single remaining `Duration`

Each arrival is `(Elem, remaining: Duration)`; `observe` subtracts `dt` first,
drops the zeros, then diffs — so a transition started on this observation gets
its full beat. The popup is `popup_wait: Duration`: set to the popup beat on the
resolved edge, counted down, and `ZERO` whenever the board is not resolved.
"Popup due" is `popup_wait.is_zero()`, which is trivially true on a seed
(resumed save at `AwaitingNextRound` → popup on the first frame, AC 7) and after
`n` clears the outcome mid-beat (AC 4). No flag, no `Option`.

### 4. A row that shrinks is a clear, never a change

`setup_next_round` and `new_game` empty both rows and drop the total to 0. A
20→0 total is a "change" by value, but the spec wants a rematch's first frame
settled and the eye guided to what *arrived*. Rule in `observe`: if a side's
dealer or played count fell since the last observation, discard that side's
arrivals and start nothing for it; otherwise start a `Dealer`/`Played` arrival
per new index and a `Score` arrival if the total differs. A second change to the
same Score restarts its beat (the old clock is replaced — one figure, one
clock); a card index cannot arrive twice without a clear.

### 5. The thinking indicator is a suffix on the existing line, not a new message

`status_message` stays as it is (its tests pin `Opponent's Turn`, Muted).
`BoardView::status_lines` appends `motion.thinking_suffix()` — `" ."`, `" .."`,
`" ..."` stepping every `THINKING_STEP_MS` from the moment the phase became
`OpponentThinking` — when it is `Some`, which is exactly the phase in which the
base line is the opponent's. The emphasis is the line's own (Muted). On the
compact board the stake is on row 0 and this line on row 1, so they cannot meet
(pinned by a test, AC 5/9). The indicator is not a Transition: a board seeded
in `OpponentThinking` (a save resumed mid-pause) shows `Opponent's Turn .` on
its first frame — the seed starts the pause clock at zero — and that is
consistent with AC 7, which is about arrivals and the popup; the walkthrough
report should not read it as a first-frame violation.

### 6. Timing constants live in `lib.rs`, the bounds in a test

`ARRIVAL_BEAT_MS = 600`, `POPUP_BEAT_MS = 800`, `THINKING_STEP_MS = 300` sit
beside `SELECTION_PULSE_MS` and `OPPONENT_THINKING_TIME_MS`. Chosen inside the
spec's bounds: one pulse period plus a fraction, so a bold card is seen through
at least one full breath of the cursor; the popup 200 ms after the card settles;
three distinct dot states inside the 1000 ms pause. A `motion.rs` test asserts
the bounds (AC 11), so a tuning at the walkthrough that leaves them is caught.
Tuning is a sub-lettered task editing only these three lines.

### 7. Settings: three rows, one `adjust`, no disk in tests

`SettingRow::Animations`; `SettingsAction::Louder/Quieter` become
`Right/Left` (on a volume row: louder/quieter; on Animations: toggle — the
names were about to be wrong). `move_up/down` index a `const ROWS: [SettingRow;
3]`. The value change moves out of `App::handle_settings_input` into
`Settings::adjust(&mut self, row, right: bool)` (with `VOLUME_STEP`), so the
toggle and the clamp are unit-tested without the `save()` that writes the real
config file; `App` keeps calling `set_settings`, `save` and the cue as today.

### 8. The density rule — no conflict, flagged

The Settings overlay is a cursored list (like the start menu): the acted-on
element is whichever row the cursor is on, and the rows stay compact as they do
today; the spec says the overlay's other rows are unchanged. The board gains no
row. Flagged for sign-off as a reading of the *acted-on element stands apart*
rule, not an exception to it.

## Design

### 1. `src/lib.rs` — the beats

```rust
pub mod motion;

// Spec 027 — one-shot board transitions (drawing only; see motion.rs). Bounds
// from the spec, pinned by motion::tests::beats_are_named_constants_within_bounds:
// SELECTION_PULSE_MS <= ARRIVAL_BEAT_MS <= 1000, ARRIVAL_BEAT_MS <= POPUP_BEAT_MS
// <= 1000, and THINKING_STEP_MS * 2 <= OPPONENT_THINKING_TIME_MS.
pub const ARRIVAL_BEAT_MS: u64 = 600; // a card arriving / a total changing draws Strong this long
pub const POPUP_BEAT_MS: u64 = 800; // the round/game popup waits this long after the round resolves
pub const THINKING_STEP_MS: u64 = 300; // the thinking indicator steps . / .. / ... at this cadence
```

### 2. `src/motion.rs` — `BoardMotion`

```rust
//! One-shot board transitions (spec 027): which element just changed and how
//! much of its beat is left. `App` feeds it from `tick` by diffing the game
//! state — the banter/audio observer pattern — and `board.rs` reads it when it
//! draws. Drawing state only: never saved, never read by the engine.

use std::time::Duration;
use crate::{ARRIVAL_BEAT_MS, POPUP_BEAT_MS, THINKING_STEP_MS, game::{GamePhase, GameState}, player::Player};

/// A board element that can be in transition: a card by its index in its
/// side's row (stable until the row clears), or a side's Score figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elem { Dealer(Player, usize), Played(Player, usize), Score(Player) }

impl Elem { fn side(self) -> Player { … } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SideSnapshot { dealers: usize, played: usize, score: i32 }

/// The facts the diff compares. `resolved` is exactly the condition under which
/// `draw_round_outcome_text` draws a popup today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MotionSnapshot { player: SideSnapshot, opponent: SideSnapshot, resolved: bool, thinking: bool }

impl MotionSnapshot {
    fn of(gs: &GameState) -> Self {
        // resolved = gs.round_outcome.is_some() || matches!(gs.game_phase, GamePhase::GameOver { .. })
        // thinking = matches!(gs.game_phase, GamePhase::OpponentThinking { .. })
    }
    fn side(&self, who: Player) -> SideSnapshot { … }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BoardMotion {
    prev: Option<MotionSnapshot>,
    /// Arrivals in flight: the element and the beat remaining. Independent
    /// clocks — a new one never shortens or delays another.
    arrivals: Vec<(Elem, Duration)>,
    /// Time left before the round/game popup draws; ZERO whenever the board is
    /// not resolved, so a resumed or already-shown popup is due at once.
    popup_wait: Duration,
    /// Time spent in the current `OpponentThinking` pause; None outside it.
    thinking_for: Option<Duration>,
}

impl BoardMotion {
    /// Advance every clock by `dt`, then diff `gs` against the last observation
    /// and start the transitions it implies. The first observation after a
    /// reset seeds silently: nothing starts (spec: the first frame is settled).
    pub fn observe(&mut self, gs: &GameState, dt: Duration) {
        for (_, left) in &mut self.arrivals { *left = left.saturating_sub(dt); }
        self.arrivals.retain(|(_, left)| !left.is_zero());
        self.popup_wait = self.popup_wait.saturating_sub(dt);
        if let Some(t) = &mut self.thinking_for { *t += dt; }

        let curr = MotionSnapshot::of(gs);
        match self.prev {
            None => {
                self.popup_wait = Duration::ZERO;
                self.thinking_for = curr.thinking.then_some(Duration::ZERO);
            }
            Some(prev) => {
                for who in [Player::Player, Player::Opponent] {
                    self.diff_side(who, prev.side(who), curr.side(who));
                }
                if curr.resolved && !prev.resolved { self.popup_wait = ms(POPUP_BEAT_MS); }
                if !curr.resolved { self.popup_wait = Duration::ZERO; }
                if curr.thinking && !prev.thinking { self.thinking_for = Some(Duration::ZERO); }
                if !curr.thinking { self.thinking_for = None; }
            }
        }
        self.prev = Some(curr);
    }

    /// A row that shrank is a clear (round end, rematch): drop this side's
    /// transitions and start none — a reset total is not a change to guide the
    /// eye to. Otherwise: one arrival per new card index, and the Score if the
    /// total differs (restarting its clock if it is already running).
    fn diff_side(&mut self, who: Player, prev: SideSnapshot, curr: SideSnapshot) { … }

    pub fn is_arriving(&self, elem: Elem) -> bool { self.arrivals.iter().any(|(e, _)| *e == elem) }
    pub fn popup_due(&self) -> bool { self.popup_wait.is_zero() }
    /// The thinking indicator for this instant, while the opponent is thinking.
    pub fn thinking_suffix(&self) -> Option<&'static str> { self.thinking_for.map(thinking_suffix_at) }
}

/// ` .`, ` ..`, ` ...`, cycling every THINKING_STEP_MS from the start of the pause. Pure.
pub fn thinking_suffix_at(elapsed: Duration) -> &'static str {
    const DOTS: [&str; 3] = [" .", " ..", " ..."];
    DOTS[((elapsed.as_millis() as u64 / THINKING_STEP_MS) % 3) as usize]
}

fn ms(n: u64) -> Duration { Duration::from_millis(n) }
```

### 3. `src/board.rs` — reading the motion

`BoardView::draw` gains `motion: Option<&BoardMotion>` (before `frame`);
`None` is the settled draw and must produce the frame the pre-spec code did
apart from tension 1. Internals:

- `fn arriving(motion: Option<&BoardMotion>, e: Elem) -> bool { motion.is_some_and(|m| m.is_arriving(e)) }`
  (free fn beside `status_message`).
- `draw_side_header(side, name, who: Player, p, motion, frame)`: the Score line
  draws `if arriving(motion, Elem::Score(who)) { Strong } else { Normal }`
  (tension 1). Name, `Rounds won`, `BUSTED!!`/`Stood` unchanged.
- `draw_top_info` passes `Player::Player` / `Player::Opponent` and `motion`.
- `draw_side(side, ps, who: Player, selection, motion, frame)`: the dealer loop
  sets `v.emphasis = Strong` when `arriving(motion, Elem::Dealer(who, i))`; the
  played loop likewise for `Elem::Played(who, j)` (keeping `Double`). Slot
  counter, ghosts and the hand are untouched.
- `status_lines(state, cursor, motion)`: after `status_message`, `if let (Some((text, _)), Some(suffix)) = (base.as_mut(), motion.and_then(|m| m.thinking_suffix())) { text.push_str(suffix) }`.
- The popup: `if motion.is_none_or(|m| m.popup_due()) { self.draw_round_outcome_text(state, frame) }`.
- Draw order is unchanged (popup after the sides, panel/stake last).

`draw`'s doc gains: `motion` is the in-flight transitions (spec 027); `None`
draws every element settled — the Animations-off board.

### 4. `src/app.rs` — the observer and the setting

- Field `motion: BoardMotion` (doc: "One-shot board transitions in flight
  (spec 027). Observed every tick while in a match, reset to default off the
  board; never saved."); `App::new` sets `BoardMotion::default()`.
- `tick`, after `update_play_log()`:
  ```rust
  // Transitions are drawing state: observe the board while it is on screen
  // (after update(), so the opponent's move is seen on the frame it lands),
  // and discard them the moment it is not.
  match &self.screen {
      Screen::InGame { game_state, .. } => self.motion.observe(game_state, dt),
      _ => self.motion = BoardMotion::default(),
  }
  ```
  `tick` is the only observer call: the loop runs it between every key and
  every draw. (While `Modal::FirstMatch` holds the match, `update()` is skipped
  but the observation still runs — nothing changes, so nothing starts; a
  transition begun before the hold settles under it. Harmless.)
- `draw`, the `InGame` arm: pass `self.settings.animations.then_some(&self.motion)`.
- `handle_settings_input`: `Up`/`Down`/`Back` as today; `Left | Right` →
  `let row = s.selected(); self.settings.adjust(row, matches!(action, SettingsAction::Right)); self.audio.set_settings(self.settings); self.settings.save(); self.audio.play(Sfx::MenuMove);`.
  `VOLUME_STEP` moves to `settings.rs`.
- No change to `resize`, `start_match`, Continue, the save path, the banter or
  audio observers, or `stake_to_show`.

### 5. `src/settings.rs` — the Animations row

```rust
pub struct Settings {
    …,
    /// Spec 027: the board's one-shot transitions. Off draws the board settled.
    /// A file without the key reads as On.
    #[serde(default = "default_animations")]
    pub animations: bool,
}
fn default_animations() -> bool { true }

/// How much one ←/→ press moves a volume row. (From app.rs.)
const VOLUME_STEP: f32 = 0.1;

impl Settings {
    /// Apply ←/→ on `row`: a volume steps by VOLUME_STEP and clamps to 0..=1;
    /// Animations toggles either way. Pure — the caller persists.
    pub fn adjust(&mut self, row: SettingRow, right: bool) { … }
}

pub enum SettingRow { Music, Sfx, Animations }
/// Top to bottom, as drawn.
const ROWS: [SettingRow; 3] = [SettingRow::Music, SettingRow::Sfx, SettingRow::Animations];

pub enum SettingsAction { Up, Down, Left, Right, Back }  // Left/Right were Quieter/Louder
```

`move_up`/`move_down`: the index of `selected` in `ROWS`, moved by one and
clamped (replacing the two-row special case and its "generalize" comment).
`draw_overlay`: a `row_text(row, settings) -> String` per row — the volume rows
as today, `Animations` as `format!("{marker}{label:<9}{}", if settings.animations { "On" } else { "Off" })`;
the content column becomes 7 (title, gap, three rows, gap, hint), rows at
`2 + i`, hint at row 6; the hint reads `↑/↓ select  ·  ←/→ change  ·  Esc back`.
Existing tests' `Settings { … }` literals gain `animations: true` (and
`src/audio.rs`'s three test literals gain `..Settings::default()` — a tests-only
edit in a file that is otherwise untouched).

### 6. `Readme.md` and `design/brief.md`

- `Readme.md` ~line 19: "audio with a settings menu (music/SFX volume, a global
  mute)" → "audio with a settings menu (music/SFX volume, a global mute, and an
  **Animations** on/off row for the board's card and score transitions)".
- `design/brief.md`, Motion, one sentence appended to the second paragraph
  (after "If everything moves, nothing is emphasized."): "**Amendment (spec
  027).** The one-thing-moves rule counts *continuous* motion: a one-shot
  emphasis transition — a card arriving, a total changing, a popup held back a
  beat — may run alongside the selection pulse, because it ends on its own
  within a beat and never breathes."

## Files

- `src/lib.rs` — `pub mod motion;`, the three constants.
- `src/motion.rs` — new: `Elem`, `BoardMotion`, `thinking_suffix_at`; tests.
- `src/board.rs` — `draw` takes `Option<&BoardMotion>`; `draw_side`,
  `draw_side_header`, `status_lines`, the popup guard; tests.
- `src/app.rs` — the `motion` field, the `tick` observer, the `draw` argument,
  `handle_settings_input` via `Settings::adjust`; tests.
- `src/settings.rs` — `animations`, `adjust`, `VOLUME_STEP`, the third row,
  `Left`/`Right`; tests. `src/audio.rs` — three test literals only.
- `Readme.md` — one clause. `design/brief.md` — one sentence.
- `specs/027-animation-pass/closeout-main-docs.md` (T005).
- **No change**: `src/game.rs`, `main.rs`, `frame.rs`, `render.rs`,
  `layout.rs`, `portrait.rs`, `card.rs`, `player.rs`, `save.rs`,
  `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`, `campaign_map.rs`,
  `opponent.rs`, `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`, the assets.

## Tests

Each claim names the task that owns its check.

- **The beats are named constants inside the bounds** (AC 11) — T001,
  `motion.rs`: `beats_are_named_constants_within_bounds`:
  `SELECTION_PULSE_MS <= ARRIVAL_BEAT_MS <= 1000`,
  `ARRIVAL_BEAT_MS <= POPUP_BEAT_MS <= 1000`,
  `THINKING_STEP_MS * 2 <= OPPONENT_THINKING_TIME_MS`.
- **The first observation starts nothing; a resumed popup is due at once**
  (AC 7) — T001: `the_first_observation_starts_nothing`: a `GameState` with
  cards in both rows on both sides at `AwaitingNextRound` with an outcome →
  `observe(ZERO)` → no `is_arriving` for any of its cards or either Score,
  `popup_due()`; the same at `OpponentThinking` → `thinking_suffix() == Some(" .")`.
- **A dealt card and its total are Strong for one beat, the rest of the row is
  not** (AC 1, 3) — T001: `a_dealt_card_and_its_total_arrive_for_one_beat`:
  seed on a fresh game; push a dealer card to the player; `observe(ZERO)` →
  `Dealer(Player, 0)` and `Score(Player)` arriving, `Dealer(Player, 1)` and
  `Score(Opponent)` not; `observe(ARRIVAL − 1 ms)` still arriving;
  `observe(1 ms)` settled. Repeated for the opponent's side.
- **A played card arrives; a draw-and-play arrives together; the total is not
  Strong if it did not change** (AC 2, 3) — T001:
  `a_play_arrives_with_the_total_and_a_draw_and_play_arrive_together`: push a
  `PlayedCard { Plus(3), 3 }` to the player's played row → `Played(Player, 0)`
  and `Score(Player)`; on the opponent push a dealer card and a played card in
  one observation → `Dealer(Opponent, 0)` and `Played(Opponent, 0)` both
  arriving; push a `PlayedCard { Flip(TwoFour), 0 }` (a zero-value play) →
  `Played` arriving, `Score` not.
- **Transitions are independent** (§Concurrency) — T001:
  `transitions_run_on_independent_clocks`: two hits 200 ms apart → both
  arriving; the first settles at `ARRIVAL`, the second 200 ms later; the Score's
  clock was restarted by the second hit and settles with it.
- **A cleared row discards its transitions and a reset total never starts one**
  (AC 7 rematch, §Concurrency) — T001:
  `a_cleared_row_discards_its_transitions_and_starts_none`: with arrivals in
  flight, empty both rows (score 20 → 0) as `setup_next_round` does →
  `observe(ZERO)` → nothing arriving on that side, `Score` not arriving; the
  same from `GameOver` to `PlayerTurn` with rounds reset (a rematch).
- **The popup waits one popup beat after the round resolves, and not at all
  once the outcome is gone** (AC 4) — T001:
  `the_popup_waits_one_beat_and_n_clears_the_wait`: seed at `PlayerTurn`; set
  `round_outcome = Some(PlayerWon)`, phase `AwaitingNextRound`, `observe(ZERO)`
  → `!popup_due()`; `observe(POPUP − 1 ms)` still not; `observe(1 ms)` due.
  Again from a fresh edge, then clear the outcome and return to `PlayerTurn`
  mid-beat → `popup_due()` and nothing arriving. The same edge into `GameOver`.
- **The thinking indicator steps and reverts** (AC 5) — T001:
  `the_thinking_indicator_steps_through_the_pause_and_reverts`:
  `thinking_suffix_at` at `0`, `STEP`, `2·STEP`, `3·STEP` is `" ."`, `" .."`,
  `" ..."`, `" ."`; on a `BoardMotion`: phase `OpponentThinking` → `Some(" .")`,
  after `STEP` → `Some(" ..")`, phase `PlayerTurn` → `None`; sampling every
  50 ms across `OPPONENT_THINKING_TIME_MS` yields ≥ 2 distinct suffixes.
- **The board draws arrivals Strong and settles; Off is the settled frame**
  (AC 1, 2, 3, 8, 9) — T002, `board.rs`:
  `arrivals_draw_strong_then_settle_on_both_layouts`: for `cols` in `[89, 139]`,
  a game with `dealer_row = [Dealer(5)]`, `played_row = [Plus(3)]` on the player
  and `dealer_row = [Dealer(7)]` on the opponent; a `BoardMotion` seeded on the
  empty game then observed on this one; draw with `Some(&motion)`: the
  top-left border cell of `card_slot(player.grid, 0, 0)` is `Strong`, of
  `card_slot(player.grid, 3, 2)` (played index 11) is `Strong`, of the
  opponent's slot 0 is `Strong`; the `Score: 8` cells on the player's header
  row 0 are `Strong`, the opponent's `Score: 7` cells `Strong`; the `Rounds
  won: 0` cells `Normal`. Then `observe(ARRIVAL)` and draw again → every one
  of those cells `Normal`; that frame equals the frame drawn with `None`
  (`assert!(a == b)`), and differs from the first.
- **The popup is absent during the beat and the deciding card is visible and
  Strong; present after; present at once when Off** (AC 4) — T002:
  `the_popup_waits_and_the_deciding_card_shows_through`: seed at `PlayerTurn`
  with one dealer card; push a second card, set `round_outcome`, phase
  `AwaitingNextRound`, `observe(ZERO)`; draw → no row contains `You won this
  round!`, slot 1's border cell is `Strong`; `observe(POPUP)`, draw → some row
  contains it; draw with `None` on the resolving frame → present.
- **The thinking line carries the indicator, keeps Muted, and never reaches the
  stake** (AC 5, 9) — T002:
  `the_thinking_line_steps_muted_and_stays_clear_of_the_stake`: at 89×31 and
  139×31, phase `OpponentThinking`, `stake = Some(999_999)`, motion observed at
  `2·STEP` → row `status.y0 + 1` from `status.x0` reads `Opponent's Turn ...`
  with every cell `Muted`; at 89 row `status.y0` still ends in the stake line;
  with `None` the row reads `Opponent's Turn` and the next cell is blank.
- **`status_message` is unchanged** (AC 6, structural) — T002: the existing
  `status_opponent_turn_is_muted` and every other `board.rs` test pass
  unedited apart from `drawn_board`'s added `None` argument.
- **A match's first frame is settled; a hit transitions; Off settles it on the
  next frame; the round advancing mid-beat drops the wait; leaving resets**
  (AC 4, 7, 8) — T003, `app.rs`:
  `the_board_transitions_after_a_hit_and_settles_when_animations_are_off`:
  `App::new(89×31)` with a fresh `Profile::default()`. **No game key is
  pressed**: every key that reaches the engine ends in `save_game()`
  (`handle_key`, the `game_changed` branch), which writes the real save file,
  so rows and phases are set by hand and only `tick`/`draw` run — nothing here
  touches disk. `screen = InGame` at `PlayerTurn`; `tick(ZERO)`; `draw` → no
  `Strong` cell inside the player's grid rect; push a dealer card by hand and
  set the phase to `OpponentThinking { until: Instant::now() +
  Duration::from_secs(3600) }` (what a hit does; **the deadline must be far in
  the future** — `tick` runs `update()` before it observes, and an elapsed
  `until` would move the phase to `OpponentTurn`, save the real file and lose
  the thinking line; one comment in the test says so); `tick(ZERO)`; draw →
  slot 0's border `Strong` and the status row contains `Opponent's Turn .`;
  `app.settings.animations = false` (T004 adds the field; T003 ships the test
  without this step); draw → slot 0 `Normal`, status `Opponent's Turn` with a
  blank after; back `true`, draw → `Strong` again. Then set
  `round_outcome`/`AwaitingNextRound` by hand, `tick(ZERO)`, draw → no popup
  text; clear the outcome and set `PlayerTurn` with both rows emptied (what
  `n` does), `tick(ZERO)`, draw → no popup, no `Strong` in the grid. Finally
  `handle_key(Esc)` (the `Menu` arm sets no `game_changed`, so no save) →
  `StartMenu`; `tick(ZERO)` → `app.motion == BoardMotion::default()`. That
  `n` itself acts on the frame it is pressed is structural — `game.rs` and the
  key path are untouched (T005's diff check) — and seen at the walkthrough.
- **A resumed board at the popup draws it on its first frame** (AC 7) — T003:
  `a_resumed_popup_draws_on_the_first_frame`: `screen = InGame` at
  `AwaitingNextRound` with an outcome and cards in the rows (as Continue leaves
  it); `tick(ZERO)`; draw → the popup text is present and no grid cell is
  `Strong`.
- **The setting: default, missing key, Off, round trip, adjust** (AC 8) —
  T004, `settings.rs`: `settings_animations_default_on_missing_key_on_and_off_reads_off`
  (`Settings::default().animations`, `from_json_or_default("{}")`,
  `r#"{"animations": false}"#`, and a full round trip); `adjust_toggles_animations_and_steps_volumes`
  (toggle both ways; volume ±0.1 and clamped at 0 and 1);
  `settings_rows_move_over_three_rows_and_clamp` (`move_down` ×3 → Animations,
  `move_up` ×3 → Music); `the_animations_row_reads_on_or_off_and_fits`
  (`draw_overlay` at 89×31 and 139×31 with `animations` true then false → some
  row contains `Animations` and ends in `On` / `Off`; the box is inside the
  frame). Existing round-trip and defaults tests updated for the field.
- **The App routes ←/→ on the Animations row to the toggle** (AC 8) — T004:
  routed through `Settings::adjust`, which is tested; the `App` arm calls
  `save()` and so is checked in the Phase 2 walkthrough (the settings file
  shows `"animations": false` after a toggle), not in a unit test.
- **Keys are never delayed; no `GamePhase`, timing constant or `apply_*`
  changes** (AC 6, 12) — structural: `git diff main...HEAD -- src/game.rs` is
  empty and `handle_key`'s `InGame` arm is unchanged (T005), and the `n`/`g`
  steps of the Phase 1 walkthrough.
- **The pulse keeps breathing** (AC 10) — structural: `SelectionPulse` and
  `draw_side`'s hand branch are untouched (T002 diff); the brief's sentence is
  read in the T004 diff; seen at the walkthrough.
- **README names the row** (AC 13) — T004: `grep -n "Animations" Readme.md`.
- **No forbidden change, no new crate, no color** (AC 12) — T005: `git diff
  main...HEAD --stat` lists none of the forbidden files; `SAVE_VERSION` and
  `PROFILE_VERSION` still 1; `grep -rn "Color" src/` finds nothing new.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim.
- **Driver walkthroughs** (the orchestrator, `run-kaazap` skill; the real
  profile, settings and save backed up and checksum-restored afterwards). The
  loop draws every 50 ms and the beats are 600/800 ms, so a snapshot taken
  right after a key catches the bold state and one taken a second later the
  settled state.
  - **After the Phase 1 review (T003)**, at **89×31 and 139×31**, a Quick Play
    match: on a hit the new card and the Score bold, the older cards and
    `Rounds won` not, settled about half a second later; the cursor still
    breathing meanwhile; the opponent's line reading `Opponent's Turn .`, `..`,
    `...` through its pause and plain `Opponent's Turn` gone when it acts; the
    opponent's dealt and played cards bold on the frame they land; at a round's
    end the deciding card visible and bold with no popup, then the popup;
    pressing `n` inside that beat starting the next round at once with no popup
    ever shown; the game-over popup likewise with `g`; after `g` a settled
    board; Esc to the menu and Continue on a saved match → settled first frame,
    and a save left at the round popup → popup on the first frame. At 89 the
    stake on row 0 with the dots on row 1. Report the timing feel in plain
    language; the person then plays it and may ask for a tuning within the
    bounds (a sub-lettered task editing only the three constants).
  - **After the Phase 2 review (T004)**: Settings shows three rows; `↓` reaches
    Animations; `←`/`→` flip `On`/`Off`; the settings file on disk shows
    `"animations": false`; a match with it Off shows no bold card or score, the
    popup on the resolving frame, and the plain thinking line; back On, the
    Phase 1 checks hold again; a settings file with the key deleted reads On.
    Restore the real settings file afterwards.

## Non-goals (from spec)

Nothing outside the match board; no face-down reveal; no stake flash; no
portrait animation (spec 016's deferral closes at merge); no departure
transitions; the pulse is not held; the Animations row governs these
transitions only; no change to the opponent's thinking time, the phase machine,
the save format, the profile, the economy, the AI or balance data.

## Open questions

1. **The Score's resting weight** (tension 1): the spec's ACs 3 and 8 need
   the Score `Normal` at rest so a `Strong` beat can show, while Goal 5 says
   the Off board is the pre-spec board "frame for frame". This plan follows
   the ACs (rest `Normal`, both settings). **Ruled: spec Q7, 2026-09-18 — the
   Score rests Normal; flagged to the person in the conformance summary.**

Settled here as design and flagged for sign-off:

2. **`SettingsAction::Louder/Quieter` → `Right/Left`** and `Settings::adjust`
   (tension 7) — a rename the spec doesn't ask for, made so the names stay true
   on the toggle row and so the toggle is unit-testable without a disk write.
3. **The Settings hint** becomes `←/→ change` (was `←/→ volume`) — wording the
   spec leaves to the plan.
4. **The density rule** on the Settings overlay (tension 8) — read as not
   applying to a cursored list, matching the start menu.
5. **The three beat values** (tension 6) — 600 / 800 / 300 ms; tunable within
   the bounds at the Phase 1 pause.
