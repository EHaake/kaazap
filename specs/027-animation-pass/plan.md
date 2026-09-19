# Plan: Animation pass — spec 027

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-18; re-review after B1 and N1, N2, N4; R1 applied by the orchestrator). **Revision 2** (2026-09-19, spec Q11, orchestrator's transcription at the Phase 1b pause): the face-down flip is withdrawn — `FLIP_BEAT_MS`, `BoardMotion::is_face_down`, the board's `face_down` helper and the `?` branch of the dealer loop are removed by T003d, with their tests and bounds assertions; the heavy landing, the source ghost and everything else in Revision 1 (tension 9, §Design 1–3, §Tests) stand, read with the flip lines struck. Three constants again. **Revision 1: Signed off** (skeptical-reviewer at fable, 2026-09-18; re-review after B1 and N1, N2, N4) (2026-09-18, at the Phase 1 pause; spec Q8–Q10 — a heavy-border, face-down dealer arrival; a heavy played-card landing with a source ghost in the emptied hand slot). The new material is tension 9, the Revision 1 lines in §Design 1–3, §Files, the *Revision 1 (Phase 1b)* block in §Tests, the Phase 1b walkthrough in §Verification and §Open questions 6–8.
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
spec 017); four timing constants in `lib.rs`; `board.rs` reading that struct
when it draws; a `bool` on `Settings` with a third overlay row; one README
sentence and one sentence in `design/brief.md`. `game.rs`, its phases, its
timing and its keys are not touched — the transitions observe the state after
the fact and never gate a key or a phase.

**Revision 1** (spec Q8–Q10, ruled at the Phase 1 pause): bold on a thin
box-drawn card did not register, so the two card arrivals change *shape* as
well as weight — a dealt card lands with the heavy border, its face `?` for a
flip beat, then its value; a played card lands heavy, and the hand slot it left
shows a single-line ghost outline for the beat. The Score, the popup beat and
the thinking indicator stay as built. Tension 9 and the Revision 1 lines in
§Design 1–3 carry it; Phase 1b in `tasks.md` builds it on top of Phase 1, with
no change to `app.rs`.

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
- **A card's look is three fields** (Revision 1). `CardView { text, weight:
  BorderWeight, emphasis }` (`card.rs`): `draw` blanks the interior, draws the
  box at `weight`/`emphasis` and centres `text` on the middle row. A heavy
  border, a `?` face and an empty-faced outline are all field values on the
  view `draw_side` already builds — `CardView::new(x, y, String::new())` is
  exactly a single-line outline with an empty face at Normal.
  `BorderWeight::{Single, Heavy, Double}` already has glyphs (`frame.rs`), and
  the opponent's hidden hand already draws `"?"`.
- **A hand slot empties in one place** (Revision 1). `hand: Vec<Option<Card>>`
  is always `HAND_SIZE` long; `commit_play` sets `hand[index] = None` on the
  same mutation that pushes the played card, on either side, and nothing else
  empties a slot (`deal_hand` fills all four at a rematch). `draw_side`'s hand
  loop skips a `None` (`let Some(card) = c else { continue }`), so an emptied
  slot draws nothing today.

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
three distinct dot states inside the 1000 ms pause. `FLIP_BEAT_MS = 250`
(Revision 1): five frames face down at the loop's 50 ms — enough to register as
a card turning over — inside `150..=ARRIVAL_BEAT_MS / 2`, leaving 350 ms of the
value under the heavy border; and shorter than the popup beat, so a deciding
card has always turned face up before the popup can cover it. A `motion.rs`
test asserts the bounds (AC 11), so a tuning at the walkthrough that leaves
them is caught. Tuning is a sub-lettered task editing only these four lines.

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

### 9. Revision 1 — the arrivals change shape, on the clocks Phase 1 already runs

The Phase 1 walkthrough found bold invisible on a thin box-drawn card (spec
Revision 1). The remedy is border weight and face, not a new kind of motion,
and it costs no new clock. A dealt card draws `BorderWeight::Heavy` + `Strong`
while `Elem::Dealer` is arriving, and its face is `?` while the arrival's
*remaining* time is still inside the flip window: `is_face_down(elem)` is
`remaining + FLIP_BEAT > ARRIVAL_BEAT`, a read of the countdown Phase 1 already
keeps, so the flip ends exactly `FLIP_BEAT_MS` after the card landed and the
value shows, still heavy, for the rest of the beat. A played card draws `Heavy`
+ `Strong` while arriving and its resting `Double` after. The source ghost is
one more `Elem`, `Hand(side, slot)`, started by `diff_side` when a hand slot
went `Some → None` since the last observation (the side snapshot carries
`hand: [bool; HAND_SIZE]`), on the arrival beat, and discarded by the same
shrink rule as the side's other arrivals — a round clearing under a ghost drops
it, the spec's "ceases to exist" case. The board draws it by reusing `CardView`
with an empty face: single border, blank interior, Normal, no number key (an
empty slot has none today). Both sides run the same loops, so the opponent's
hidden `?` slot ghosts identically. Rejected: a second clock per dealer card
(two things to tick for one beat); a `face_down: bool` in the arrival tuple
(state a subtraction already gives); a new `BorderWeight` for the landing (the
spec names the heavy border the cursor uses). One consequence outside the
board: `frame.rs`'s `BorderWeight` doc ("Heavy is reserved for cursor
selection") is now one exception stale and is corrected in T003c — a
comment-only touch of a file the plan otherwise leaves alone; the close-out's
DECISIONS entry records Q8/Q9 as the exception to the brief's "distinct
weights, distinct meanings".

## Design

### 1. `src/lib.rs` — the beats

```rust
pub mod motion;

// Spec 027 — one-shot board transitions (drawing only; see motion.rs). Bounds
// from the spec, pinned by motion::tests::beats_are_named_constants_within_bounds:
// SELECTION_PULSE_MS <= ARRIVAL_BEAT_MS <= 1000, ARRIVAL_BEAT_MS <= POPUP_BEAT_MS
// <= 1000, THINKING_STEP_MS * 2 <= OPPONENT_THINKING_TIME_MS, and (Revision 1)
// 150 <= FLIP_BEAT_MS and FLIP_BEAT_MS * 2 <= ARRIVAL_BEAT_MS.
pub const ARRIVAL_BEAT_MS: u64 = 600; // a card arriving / a total changing draws Strong this long
pub const FLIP_BEAT_MS: u64 = 250; // a dealt card's face reads `?` for this long at the start of its arrival (Revision 1)
pub const POPUP_BEAT_MS: u64 = 800; // the round/game popup waits this long after the round resolves
pub const THINKING_STEP_MS: u64 = 300; // the thinking indicator steps . / .. / ... at this cadence
```

Revision 1 adds `FLIP_BEAT_MS` (T003b); the other three are as shipped in T001.

### 2. `src/motion.rs` — `BoardMotion`

```rust
//! One-shot board transitions (spec 027): which element just changed and how
//! much of its beat is left. `App` feeds it from `tick` by diffing the game
//! state — the banter/audio observer pattern — and `board.rs` reads it when it
//! draws. Drawing state only: never saved, never read by the engine.

use std::time::Duration;
use crate::{ARRIVAL_BEAT_MS, FLIP_BEAT_MS, HAND_SIZE, POPUP_BEAT_MS, THINKING_STEP_MS, game::{GamePhase, GameState}, player::Player};

/// A board element that can be in transition: a card by its index in its
/// side's row (stable until the row clears), a side's Score figure, or — the
/// source ghost, Revision 1 — the hand slot a side card was just played from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elem { Dealer(Player, usize), Played(Player, usize), Score(Player), Hand(Player, usize) }

impl Elem { fn side(self) -> Player { … } }  // covers Hand too

/// `hand[i]`: whether slot i holds a card (Revision 1 — a true→false slot
/// starts a `Hand` ghost). `of` builds it as
/// `std::array::from_fn(|i| p.hand.get(i).is_some_and(Option::is_some))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SideSnapshot { dealers: usize, played: usize, score: i32, hand: [bool; HAND_SIZE] }

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
    /// eye to. Otherwise: one arrival per new card index, the Score if the
    /// total differs (restarting its clock if it is already running), and
    /// (Revision 1) a `Hand(who, i)` ghost for each slot that went filled →
    /// empty — all on the arrival beat.
    fn diff_side(&mut self, who: Player, prev: SideSnapshot, curr: SideSnapshot) {
        …
        for i in 0..HAND_SIZE {
            if prev.hand[i] && !curr.hand[i] { self.arrivals.push((Elem::Hand(who, i), ms(ARRIVAL_BEAT_MS))); }
        }
        …
    }

    pub fn is_arriving(&self, elem: Elem) -> bool { self.arrivals.iter().any(|(e, _)| *e == elem) }
    /// Revision 1: whether `elem` is still inside the flip window of its
    /// arrival — the first FLIP_BEAT_MS of the beat. A read of the countdown,
    /// not a clock of its own; true only for a `Dealer` arrival (a played
    /// card shows its value from its first frame, spec Q9).
    pub fn is_face_down(&self, elem: Elem) -> bool {
        self.arrivals.iter().any(|(e, left)| {
            matches!(e, Elem::Dealer(..)) && *e == elem && *left + ms(FLIP_BEAT_MS) > ms(ARRIVAL_BEAT_MS)
        })
    }
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
- `fn face_down(motion: Option<&BoardMotion>, e: Elem) -> bool { motion.is_some_and(|m| m.is_face_down(e)) }`
  beside `arriving` (Revision 1).
- `draw_side(side, ps, who: Player, selection, motion, frame)`, as revised
  (Revision 1 replaces T002's emphasis-only lines): the dealer loop, while
  `arriving(motion, Elem::Dealer(who, i))`, sets `v.weight = BorderWeight::Heavy`
  and `v.emphasis = Strong`, and `v.text = "?".to_string()` while
  `face_down(motion, Elem::Dealer(who, i))` too; the played loop sets
  `v.weight = if arriving(motion, Elem::Played(who, j)) { Heavy } else { Double }`
  and `Strong` while arriving. The hand loop's `let Some(card) = c else { continue }`
  becomes: on `None`, `if arriving(motion, Elem::Hand(who, i)) { let (x, y) = card_slot(side.hand, i, 0); CardView::new(x, y, String::new()).draw(frame); }`
  then `continue` — the source ghost: single-line outline, blank interior,
  Normal, no number key. Both sides run the same loops. The slot counter, the
  grid's dim ghost ticks and the hand's `Some` arms (selected: heavy breathing
  border; others: Muted; opponent: `?`) are untouched. With `motion == None`
  every branch takes its settled arm, so the Off frame is unchanged.
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

- `src/lib.rs` — `pub mod motion;`, the three constants; `FLIP_BEAT_MS`
  (Revision 1, T003b).
- `src/motion.rs` — new: `Elem`, `BoardMotion`, `thinking_suffix_at`; tests.
  Revision 1 (T003b): `Elem::Hand`, `SideSnapshot::hand`, the ghost rule in
  `diff_side`, `is_face_down`; tests.
- `src/board.rs` — `draw` takes `Option<&BoardMotion>`; `draw_side`,
  `draw_side_header`, `status_lines`, the popup guard; tests. Revision 1
  (T003c): `face_down`, the heavy/`?`/ghost arms of `draw_side`'s three loops;
  tests.
- `src/frame.rs` — one doc-comment line on `BorderWeight` (Revision 1, T003c);
  no code change.
- `src/app.rs` — the `motion` field, the `tick` observer, the `draw` argument,
  `handle_settings_input` via `Settings::adjust`; tests.
- `src/settings.rs` — `animations`, `adjust`, `VOLUME_STEP`, the third row,
  `Left`/`Right`; tests. `src/audio.rs` — three test literals only.
- `Readme.md` — one clause. `design/brief.md` — one sentence.
- `specs/027-animation-pass/closeout-main-docs.md` (T005).
- **No change**: `src/game.rs`, `main.rs`, `render.rs`,
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

*Revision 1 (Phase 1b).* Phase 1 tests that are **edited**: the bounds test
gains two assertions; the board's arrivals test gains glyph and face-cell
assertions. No existing assertion changes meaning — `Strong` on the corner
cell stays true under a heavy border — so none is removed or weakened.

- **The flip beat is a named constant inside its bounds** (AC 11) — T003b,
  `motion.rs`: `beats_are_named_constants_within_bounds` gains
  `150 <= FLIP_BEAT_MS` and `FLIP_BEAT_MS * 2 <= ARRIVAL_BEAT_MS`.
- **A dealt card is face down for the flip beat, then faces up for the rest
  of its beat; a played card is never face down** (AC 1, 2) — T003b:
  `a_dealt_card_is_face_down_for_the_flip_beat_then_faces_up`: seed; push a
  dealer card to the player; `observe(ZERO)` → `is_face_down(Dealer(P, 0))`
  and arriving; `observe(FLIP − 1 ms)` still face down; `observe(1 ms)` → not
  face down, still arriving; `observe(ARRIVAL − FLIP)` → settled. Push a
  played card → arriving and `!is_face_down(Played(P, 0))` on its first frame
  (the `Dealer(..)` guard in `is_face_down` is what makes this hold — the
  countdown alone would say face down). The same dealer sequence on the
  opponent.
- **A play ghosts the hand slot it came from for one beat; a draw-and-play
  ghosts together; a re-deal starts none; a clear drops a ghost** (AC 2, 7) —
  T003b: `a_play_ghosts_its_hand_slot_for_one_beat`: seed on a game with
  `player.hand = [Some(Plus(3)), Some(Minus(2)), None, None]` and
  `opponent.hand = [Some(Plus(2)), None, None, None]` set by hand; player
  `hand[1] = None` and push `PlayedCard { Minus(2), -2 }` → `Hand(P, 1)` and
  `Played(P, 0)` arriving, `Hand(P, 0)` and `Hand(P, 2)` not;
  `observe(ARRIVAL − 1 ms)` still; `observe(1 ms)` settled. Opponent
  `hand[0] = None` plus a dealer card and a played card in one observation →
  `Hand(O, 0)`, `Dealer(O, 0)`, `Played(O, 0)` all arriving. Then a slot
  `None → Some` (a rematch's deal) starts nothing; a ghost in flight is gone
  after the side's rows clear (`!is_arriving(Hand(..))`).
- **The board draws a dealt card heavy with `?` then its value, a played card
  heavy then double, and the settled frame is the Off frame** (AC 1, 2, 8, 9)
  — T003c, `board.rs`: `arrivals_draw_strong_then_settle_on_both_layouts`
  extended: on the first frame the corner cell of the player's dealer slot 0
  is `┏` and its middle interior row (`y + CARD_HEIGHT / 2`, `x + 1 ..= x +
  CARD_WIDTH − 2`, read as a string and trimmed — centring may round the face
  off by one) is `?`; the played slot 11's corner is `┏` and its middle row
  reads `+3` on that same first frame; the opponent's dealer slot 0 is `┏`
  with `?`. After `observe(FLIP)` and a redraw the player's middle row reads
  `5`, the corner still `┏` and `Strong`. After the settle the corners are
  `┌`, `╔`, `┌` (the existing `Normal` and settled-equals-Off assertions
  stand).
- **The emptied hand slot draws the source ghost for the beat, on both sides
  and both layouts, then blanks** (AC 2, 8, 9) — T003c:
  `a_played_card_lands_heavy_and_its_hand_slot_ghosts`: for `cols` in
  `[89, 139]`; seed with `player.hand = [Some(Plus(3)), None, None, None]`,
  `opponent.hand = [Some(Plus(2)), None, None, None]`; the changed game has
  `hand[0] = None` and `played_row = [Plus(3)]` on the player, `Plus(2)` on
  the opponent; draw with `Some(&motion)`: at `card_slot(player.hand, 0, 0)`
  the four corner cells are `┌ ┐ └ ┘` at `Normal`, the top edge's second cell
  is `─`, the whole middle interior row (`x + 1 ..= x + CARD_WIDTH − 2` at
  `y + CARD_HEIGHT / 2`) is blank, and the number-key cell below it
  (`x + CARD_WIDTH / 2`, `y + CARD_HEIGHT`) is `' '`; the same corners at
  `card_slot(opponent.hand, 0, 0)`; the played slot 11's corner is `┏` and
  `Strong` on both sides and its middle row reads `+3` (player) / `+2`
  (opponent) on this first frame — a played card is never face down (B1).
  `observe(ARRIVAL)`, redraw → the hand slot's corner
  cell is `' '`, the played corner `╔` `Normal`, and the frame equals the
  `None` frame.
- **The Off frame has no heavy border, no `?` and no ghost** (AC 8) — T004's
  Off step in the app test (below) now also asserts slot 0's corner is `┌` and
  its middle interior row reads `7` (trimmed) with `animations = false`, and
  `┏` / `?` again with it `true` (no tick in between, so the flip has not
  elapsed). The ghost's Off case is the board test's `None` frame.
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
  loop draws every 50 ms and the beats are 600/800 ms (the flip 250 ms), so a
  snapshot taken right after a key catches the heavy, face-down state, one at
  about 0.4 s the heavy face-up state, and one taken a second later the
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
    bounds (a sub-lettered task editing only the three constants). *Done —
    logged in the tier log's Phase 1 summary; the finding became Revision 1.*
  - **After the Phase 1b review (T003c)** (Revision 1), at **89×31 and
    139×31**, a Quick Play match: on a hit the new card lands with the thick
    border and a `?` face, the `?` becomes its value after about a quarter
    second, and the border thins to match the row about half a second after
    that; on a play the card lands thick-bordered and settles to its double
    border, while the hand slot it left shows an empty outline for the same
    beat, then blank; the opponent's move lands its dealt card (thick, `?`
    then value), its played card (thick) and the outline in its hidden hand all
    at once; the Score, the popup beat and the thinking dots exactly as at the
    Phase 1 walkthrough. Report the timing feel in plain language; the person
    then plays it and may ask for a tuning within the bounds (T003a, now any
    of the four constants).
  - **After the Phase 2 review (T004)**: Settings shows three rows; `↓` reaches
    Animations; `←`/`→` flip `On`/`Off`; the settings file on disk shows
    `"animations": false`; a match with it Off shows thin single-bordered
    dealt cards with their value from the first frame, no `?`, double-bordered
    plays, no outline in an emptied hand slot, no bold score, the popup on the
    resolving frame, and the plain thinking line; back On, the Phase 1 and 1b
    checks hold again; a settings file with the key deleted reads On. Restore
    the real settings file afterwards.

## Non-goals (from spec)

Nothing outside the match board; no face-down reveal for any card but a dealer
card (Q8); no stake flash; no portrait animation (spec 016's deferral closes at
merge); no departure transitions (the source ghost is a landing cue, not a
departure — Q9); the pulse is not held; the Animations row governs these
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

Revision 1, settled here as design and flagged for sign-off:

6. **`FLIP_BEAT_MS = 250`** (tension 6) — five 50 ms frames face down, inside
   the spec's `150..=300`; tunable at the Phase 1b pause.
7. **The flip is a read of the arrival's countdown, and the ghost is an
   `Elem`** (tension 9) — no new clock, no new struct; `is_face_down` compares
   the remaining time against `ARRIVAL_BEAT_MS − FLIP_BEAT_MS`, and the ghost
   is `Elem::Hand` on the same beat and the same shrink rule as a card. The
   ghost is drawn by `CardView` with an empty face rather than a bespoke
   `draw_box` call, so its rect and interior blanking are the card's.
8. **`frame.rs` gains one doc-comment line** (tension 9) — `BorderWeight`'s
   "Heavy is reserved for cursor selection" now names the arrival-beat
   exception. The file was on the plan's no-change list for its code, which is
   untouched; T005's check becomes "comment-only". The alternative — a stale
   doc on the design system's own weight enum — was judged worse than the
   one-line touch.
