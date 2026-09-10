# Plan: Stats & records — spec 020

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

## Context

The game already produces a definite winner and a round tally for every
completed match, then discards it at game over. This spec adds a **persistent
records layer**: a career (lifetime) tally of match/round win-loss **per
opponent**, an overall win streak, campaign completions, and a separate
current-campaign-run tally — surfaced on a new full-screen **Records** `Screen`
off the start menu. It changes **no mechanic**: it observes the match-end
transition already handled in `App::tick`, extends the player-owned
`profile.json` with additive serde-defaulted fields (as `campaign` and
`credits` did — **no `PROFILE_VERSION` bump**), and reads the collection that
already exists.

This is a **read-only observer plus an additive save field**. The only change
to existing behavior is `Profile::reset_to_starter` (New Campaign), amended to
**preserve** the lifetime stats field — the mastery record is the point, so it
must outlive a run reset (analogous to Settings surviving a reset by living in
their own file). The current-run tally rides on `CampaignRun`, which
`reset_to_starter` already clears, so the run scope wipes for free.

## What the code already gives us

- **The match-end transition is observable in `tick` and fires once.** The
  RoundEnd → GameOver transition happens **only** inside `GameState::update()`
  (`game.rs:346-354`, in `finalize_round`), which runs **only** from
  `App::tick` (`app.rs:1174`). `resolve_after_action` (the keypress path) only
  ever sets `RoundEnd`, never `GameOver`. So `tick` already computes
  `phase_changed` (`app.rs:1171-1176`) by comparing the phase discriminant
  before/after `update()`; `phase_changed && matches!(game_phase, GameOver{..})`
  is true on **exactly one tick** per match (the entering tick), and false on
  every subsequent GameOver tick (the phase no longer changes). This is the
  clean once-per-match seam — no snapshot/diff state is needed (see tension §2).
- **The final round tally is fully on `GameState` at game over.** A match is
  first-to-3, so at the GameOver tick `game_state.player.rounds_won` and
  `game_state.opponent.rounds_won` hold the final decisive-round counts (ties
  were replayed and never incremented `rounds_won` — `apply_reward`,
  `game.rs:362-372`). `rounds_won` is reset to 0 only on the *rematch*
  (`NextGame`, `game.rs:828-830`), which happens after acknowledgement — so the
  values are intact at the recording seam. The player's round-wins against this
  opponent = `player.rounds_won`; round-losses = `opponent.rounds_won`. This
  makes a separate round-resolution seam unnecessary (see tension §2).
- **The opponent and the mode are both known at the seam.** The current
  opponent's stable roster id is `game_state.opponent_profile.id`
  (`opponent.rs:41`), present for both Quick Play and Campaign matches. Whether
  the match is a campaign match is `profile.campaign().in_progress().is_some()`
  — `in_progress` is set at `start_match` (`app.rs:525`, `Some` for a campaign
  launch, `None` for Quick Play) and cleared only on the player's game-over
  acknowledgement (`app.rs:756`), so it is reliably present at the GameOver tick.
- **The campaign-completion signal already exists.** `CampaignRun::run_complete()`
  (`campaign.rs:207`) is true once every planet is cleared. The existing
  campaign-win block in `tick` (`app.rs:1196-1218`) calls `mark_beaten` before
  our seam runs, so `run_complete()` reflects the just-recorded win (see
  tension §3 for ordering).
- **`reset_to_starter` is a whole-profile replace.** Today it is
  `*self = Profile::default()` (`profile.rs:118-120`), which resets `campaign`
  (and thus the new run tally) for free. Preserving one field is a
  take-reset-restore (tension §4).
- **The `Screen` shape and menu wiring are established.** `opponent_select.rs`
  is the reference screen (state struct + `handle_input(key) -> Option<Outcome>`
  + `draw(frame, config, …, pulse)`), wired into the three `match &self.screen`
  arms in `app.rs` (input `app.rs:766`, `?`-help `app.rs:714`, draw
  `app.rs:1234`) and reached from a `MenuItem` via `activate_menu_item`
  (`app.rs:1094`). Emacs nav chords arrive pre-translated: `Ctrl+B/F` → Left/
  Right, `Ctrl+P/N` → Up/Down (`resolve_key`, `app.rs:224-237`) — so the screen
  only handles arrows.
- **The scroll interaction exists (spec 019).** `overlay::draw_scrollable_overlay`
  (`overlay.rs:105`) pins a title, draws a body viewport, and clamps a scroll
  offset (`vh`, `max_off`, clamp to `[0, max_off]`, `usize::MAX` = pin bottom).
  The Records screen reuses the **same viewport/clamp discipline and the same
  keys** (Up/Down/PgUp/PgDn), but draws its own full-screen bordered chrome
  because it carries more fixed rows than the play-log window (a pager line and
  a persistent collection line) — see tension §6.
- **Monochrome is structural.** `frame.rs` has no color path; every drawer takes
  an `Emphasis`. The screen renders as glyphs + `Emphasis::Normal` (the selected
  view name may breathe with the shared `pulse`), so monochrome holds by
  construction.
- **`ALL_SIDE_CARDS` is the 15-card universe** (`card.rs:126`), and
  `Profile::collection_by_type()` (`profile.rs:230`) already returns exactly the
  distinct owned side-card types — its length is "N of 15".

## Design tensions resolved

Technical choices (mine to make); each stated with its reason. Two are
**deviations from the spec's assumed implementation** — called out as the
constitution requires, with the observable behavior unchanged.

### 1. Where stats live — an additive `stats` field on `Profile`, no version bump

Lifetime stats are a new `LifetimeStats` field on `Profile`, `#[serde(default)]`,
exactly as `campaign` and `credits` were added — an older `profile.json` (no
`stats` key) loads with an all-zero record and **no `PROFILE_VERSION` bump**
(pinned by a test mirroring `credits_persist_and_default_to_zero_for_older_profiles`).
The current-run tally is a `RunStats` field on `CampaignRun` (also
`#[serde(default)]`), so it round-trips with the run and is cleared whenever the
run is. Derived quantities (overall totals, win rate, combined per-opponent
records, collection completion) are **not stored** — computed on demand for the
screen — so there is one source of truth and nothing to keep in sync.

### 2. Recording at the match-end seam only, deriving round W/L from `rounds_won` — *deviation from the spec's assumed round-resolution seam*

The spec (Resolved decision B, and the "Recording a round" flow) assumes round
W/L needs its own **round-resolution recording point** in addition to the
match-end one. It does not: at the GameOver tick the final `rounds_won` on each
side already **is** the per-opponent round tally (player round-wins =
`player.rounds_won`, round-losses = `opponent.rounds_won`; ties never counted).
So **all recording happens at the single once-per-match seam** (tension above):
match W/L, round W/L, streak, and (for a campaign win) the run tally and
completion — read from the final `GameState` in one shot.

Reasons this is better, not just simpler:

- **It makes "an abandoned match records nothing" exactly true, including its
  rounds.** A live per-round seam would credit the rounds of a match later
  quit-to-menu; the spec explicitly wants abandoned matches to record *nothing*
  ("What it tracks"; Non-goals). Recording only at game over honors that with no
  extra bookkeeping.
- **Resume and rematch need no special handling.** A resumed match records its
  full round tally once, at its eventual game over, regardless of how many
  rounds preceded the save — no seed-silently/back-log concern like the play
  log's, because nothing is recorded until the end.
- **No transient snapshot state on `App`.** The `phase_changed` bool `tick`
  already computes is the entire edge-detector.

The observable behavior matches every acceptance criterion (a 3–1 match win
shows 3 round-wins and 1 round-loss; a tied round credits neither; abandoned
records nothing). Pinned by tests on `Profile::record_match` (T002) and
attested at the Phase 2 pause. **This is the one meaningful divergence from how
the spec sketched the mechanism; the recorded quantities are identical.**

### 3. Campaign completion increments inside the match record, ordered after `mark_beaten`

On a campaign win that clears the final node, campaign-completions increments
once. `Profile::record_match` detects this itself: `if campaign && player_won &&
self.campaign.run_complete()`. This is correct because the existing campaign-win
block (`app.rs:1196-1218`) runs `mark_beaten` **before** our seam on the same
GameOver tick, so `run_complete()` already reflects the win. It cannot
double-count: a completed run exposes no next opponent, so no further campaign
match is launchable in that run, and the `phase_changed` guard fires the seam
once regardless. A new run (post-reset) can complete and increment again.
**Ordering requirement:** the new recording block is placed *after* the existing
campaign-win block in `tick`. Pinned by a `record_match` test (win that
completes the run increments; a non-final win does not; reset then re-complete
increments again).

### 4. `reset_to_starter` preserves the lifetime `stats` field only

```rust
pub fn reset_to_starter(&mut self) {
    let stats = std::mem::take(&mut self.stats);
    *self = Profile::default();
    self.stats = stats;
}
```

Everything else (collection, deck, credits, campaign — and thus the `RunStats`
on the campaign run) resets to the starter; the lifetime record survives. The
existing test `reset_to_starter_wipes_everything_back_to_a_new_profile` asserts
full serialized equality with a fresh profile — that assertion is **amended**
(not weakened) to the new contract: dirty `stats` too, then assert `stats` is
**preserved** while every other field is starter state (including an empty run
tally). This is the one deliberate behavior change to existing code (Resolved
decision D).

### 5. Per-opponent records persist; a mode has no streak of its own

Per spec.md (Entities, Non-goals, and the acceptance criteria all agree):
**`ModeRecord` stores per-opponent records only, no streak.** The overall
streak is a `Streak` field on `LifetimeStats` (tracked independently, since an
interleaved Quick-Play/Campaign sequence can't be recombined from mode-level
tallies — Resolved decision A); the run streak is a `Streak` on `RunStats`.
Because only these two streaks exist, the **Quick Play and Campaign views show
no streak line** — a streak appears only on the **Overall** view (the
overall-lifetime streak) and the **This Run** view (the run streak), so no view
displays a streak that isn't scoped to what it labels (sign-off note 2). If
per-mode streaks are later wanted, they are additive fields — no version bump.

### 6. The Records screen is a full `Screen` with its own bordered, scrollable draw

Per `CLAUDE.md`, a mode navigated *to* is a `Screen` (Resolved decision C), so
`Screen::Records { state: RecordsState }` — copied from `opponent_select.rs`'s
shape — reached from a new `MenuItem::Records`. It is **read-only**: `draw`
takes `&Profile` and never mutates it. It carries more fixed chrome than the
play-log window (a view-pager line and a persistent collection line, both
present across all four views), so rather than contort `draw_scrollable_overlay`,
the screen draws its own full-screen bordered box with the **same** viewport/
clamp math and keys as spec 019 (title/pager row, persistent collection line, a
rule, a scrolled body viewport, a footer hint). The scroll offset lives on
`RecordsState` and is clamped-and-stored-back in `draw` (which therefore takes
`&mut self`), exactly as the play log stores its clamped scroll back on `App`;
`draw` is dispatched from a dedicated `if let Screen::Records { state } =
&mut self.screen` block after the immutable screen-draw match, mirroring the
play-log draw block (`app.rs:1276`).

## Design

### 1. `src/stats.rs` — the persisted stats types + pure recording/derivation (new module, foundational)

New module (added to `lib.rs`), pure data + logic, no rendering or engine
dependency:

```rust
/// Which mode a completed match counts toward.
#[derive(Clone, Copy)]
pub enum Mode { QuickPlay, Campaign }

/// One opponent's match + round tally within one mode.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct OpponentRecord {
    pub match_wins: u32,   pub match_losses: u32,
    pub round_wins: u32,   pub round_losses: u32,
}

/// A current + all-time-longest consecutive-match-win streak.
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct Streak { pub current: u32, pub longest: u32 }
// record(won): won → current+=1, longest=max(longest,current); loss → current=0.

/// One mode's per-opponent records, keyed by roster opponent id.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ModeRecord { #[serde(default)] opponents: BTreeMap<String, OpponentRecord> }
// get(id) -> OpponentRecord (default when absent); entry_mut(id); totals() -> (wins, losses).

/// Career stats — survive New Campaign. The additive `Profile` field.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    #[serde(default)] quick_play: ModeRecord,
    #[serde(default)] campaign: ModeRecord,
    #[serde(default)] overall_streak: Streak,
    #[serde(default)] campaign_completions: u32,
}

/// The active run's tally — lives on CampaignRun, cleared on reset.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct RunStats {
    pub match_wins: u32, pub match_losses: u32,
    pub round_wins: u32, pub round_losses: u32,
    #[serde(default)] pub streak: Streak,
}
```

- `LifetimeStats::record_match(&mut self, mode, opponent_id, player_won,
  player_rounds: u32, opp_rounds: u32)` — bump the mode's `OpponentRecord`
  (match W or L; round_wins += player_rounds; round_losses += opp_rounds) and
  `overall_streak.record(player_won)`.
- `LifetimeStats::record_campaign_completion(&mut self)` — `campaign_completions += 1`.
- `RunStats::record_match(&mut self, player_won, player_rounds, opp_rounds)` —
  the flat run tally + `streak.record(player_won)`.
- Read-side derivations for the screen (pure): `quick_play()`, `campaign()`
  accessors on `LifetimeStats`; `combined_opponent(id) -> OpponentRecord`
  (Quick Play + Campaign summed for the Overall breakdown); `overall_streak()`,
  `campaign_completions()`; `ModeRecord::totals()`. A free
  `pub fn win_rate(wins: u32, losses: u32) -> Option<u32>` — `None` when no
  matches, else the rounded integer percentage.

### 2. `src/profile.rs` — the additive `stats` field, reset preservation, the seam-facing recorder

- Add `#[serde(default)] stats: LifetimeStats` to `Profile` (and to the two test
  constructors). **No `PROFILE_VERSION` bump.** `Default` initializes it to
  `LifetimeStats::default()`.
- `pub fn stats(&self) -> &LifetimeStats` (read-only, for the screen).
- `pub fn record_match(&mut self, mode: Mode, opponent_id: &str, player_won: bool,
  player_rounds: u32, opp_rounds: u32)` — the thin method the `tick` seam calls:
  `self.stats.record_match(...)`; if `mode` is `Campaign`, also
  `self.campaign.run_stats_mut().record_match(...)` and, if
  `player_won && self.campaign.run_complete()`,
  `self.stats.record_campaign_completion()`. Keeps the seam a one-liner and the
  logic unit-testable off a `Profile`.
- `pub fn distinct_side_cards_owned(&self) -> usize` — `self.collection_by_type().len()`
  (the "N" in "N of 15"), for the persistent collection line.
- **Amend `reset_to_starter`** to the take-reset-restore of tension §4.

### 3. `src/campaign.rs` — the `RunStats` field on `CampaignRun`

- Add `#[serde(default)] run_stats: RunStats` (import `RunStats` from
  `crate::stats`). Additive, serde-defaulted — a pre-020 profile's run loads
  with an empty tally.
- `pub fn run_stats(&self) -> &RunStats` and `pub fn run_stats_mut(&mut self) ->
  &mut RunStats`. No other change — `run_complete()` etc. are reused as-is.
  Because `run_stats` is a field on `CampaignRun`, `Default`/`reset_to_starter`
  clear it for free.

### 4. `src/app.rs` — the once-per-match recording seam

A new block in `tick`, **placed after the existing campaign-win block**
(`app.rs:1218`) and before `emit_audio_cues`, mirroring that block's
field-disjoint borrow structure (`&self.screen` read for the game facts, then
`&mut self.profile`):

```rust
if phase_changed
    && let Screen::InGame { game_state, .. } = &self.screen
    && matches!(game_state.game_phase, GamePhase::GameOver { .. })
{
    let player_won =
        matches!(game_state.game_phase, GamePhase::GameOver { winner: Player::Player });
    let opponent_id = game_state.opponent_profile.id;
    let player_rounds = game_state.player.rounds_won as u32;
    let opp_rounds = game_state.opponent.rounds_won as u32;
    let mode = if self.profile.campaign().in_progress().is_some() {
        Mode::Campaign
    } else {
        Mode::QuickPlay
    };
    self.profile.record_match(mode, opponent_id, player_won, player_rounds, opp_rounds);
    self.profile.save();
}
```

`phase_changed` is already computed at `app.rs:1171`; this fires on exactly the
entering GameOver tick (tension §2). No new `App` field, no reset hook (there is
no per-match transient state to reset). Quick Play (no `in_progress`) records to
the Quick Play mode; campaign records to Campaign + the run tally + completion.

### 5. `src/records.rs` — the Records screen (new module, copies `opponent_select.rs`)

```rust
pub enum RecordsView { Overall, QuickPlay, Campaign, ThisRun } // 4, in display order
pub enum RecordsOutcome { Moved, Back }
pub struct RecordsState { view: usize, scroll: usize } // view 0..4
```

- `handle_input(&mut self, key) -> Option<RecordsOutcome>`: Left → previous view
  (wrapping, `scroll = 0`), Right → next view (wrapping, `scroll = 0`) — the
  emacs `Ctrl+B/F` arrive as Left/Right; Up → `scroll = scroll.saturating_sub(1)`,
  Down → `scroll += 1` (clamped in `draw`), PageUp/PageDown → ± a page constant;
  all return `Moved`. Esc / `x` → `Back`. Unknown keys → `None`. (Copies
  `OpponentSelectState::handle_input`.)
- **Pure content builders** (testable without a `Frame`):
  - `collection_line(owned_types: usize, total_types: usize) -> String` —
    `"Cards: N of 15 — P%"` (`P = owned*100/total`).
  - `view_body(view, stats: &LifetimeStats, run: &RunStats) -> Vec<String>` — the
    scrollable body:
    - Overall/QuickPlay/Campaign: a summary block — matches played, won, lost,
      win rate (`win_rate` → `"P%"` or `"—"`); the **Overall** view additionally
      shows the overall streak (`current` / `longest`) and the **Campaign** view
      additionally shows `campaign_completions` (the Quick Play view shows
      neither — no streak line, no completions); then a blank line, a
      `By opponent:` header and a
      column header, then **one row per `opponent::OPPONENTS` entry** (the whole
      roster, faced or not, by `name`) with match `W–L` and round `W–L`, using
      the combined record for Overall and the single `ModeRecord` for the mode
      views. A never-faced opponent renders `0–0` / `0–0` (visible, not hidden).
    - ThisRun: if `run` has zero matches, a single clear
      `"No matches this run yet."` line; else the same summary block over `run`
      (matches/won/lost/win rate/streak), with no per-opponent breakdown.
  - `view_title(view) -> &'static str` and a pager label (e.g. `"Records —
    Overall  (1/4)"`).
  Exact punctuation/column widths are monochrome-text cosmetics; tests assert
  required substrings, not exact strings.
- `draw(&mut self, frame, config: &Config, profile: &Profile, pulse)`: compute a
  full-screen bordered `Rect`, `clear_rect` + `draw_box`; draw the title/pager
  row (selected view name may breathe with `pulse`), the persistent
  `collection_line(profile.distinct_side_cards_owned(), ALL_SIDE_CARDS.len())`,
  a rule, then the body viewport from `view_body(view, profile.stats(),
  profile.campaign().run_stats())` using the spec-019 clamp
  (`vh = body rows`, `max_off = body.len().saturating_sub(vh)`, clamp
  `self.scroll`, store it back), and a footer hint
  (`◂/▸ view · ↑/↓ scroll · Esc back`, with ▲/▼ when the body overflows). All
  `Emphasis::Normal` except the optional pulse on the view name.

### 6. `src/screen.rs`, `src/menu.rs`, `src/app.rs` — wiring

- `screen.rs`: add `Records { state: RecordsState }` to `Screen` (import from
  `crate::records`).
- `menu.rs`: add `MenuItem::Records`, its `Display` arm (`"Records"`), and place
  it in the item list (after `SideDeck`, before `HowToPlay` — a reasonable slot;
  a pure UX detail left open by the spec). Update the two menu tests' expected
  counts (5→6 without a save, 6→7 with) and order.
- `app.rs`:
  - `activate_menu_item`: `MenuItem::Records => self.open_records()`.
  - `fn open_records(&mut self)` — `self.screen = Screen::Records { state:
    RecordsState::new() }` (copies `open_opponent_select`'s shape, no deck-valid
    guard — Records needs no deck).
  - `handle_key`: a `Screen::Records { state }` arm routing `handle_input` →
    `Moved` plays `Sfx::MenuMove`, `Back` plays `Sfx::MenuBack` and
    `self.screen = self.start_menu()` (preserving the menu selection is the
    default — `start_menu()` rebuilds the menu; matching the other screens'
    back behavior, which the spec accepts).
  - `?`-help arm (`app.rs:723`): add `Screen::Records { .. }` to the
    no-overlay group (it carries its own footer hint).
  - `draw`: `Screen::Records { .. } => {}` in the immutable match, then a
    dedicated `if let Screen::Records { state } = &mut self.screen { state.draw(
    frame, &self.config, &self.profile, pulse) }` block after it (tension §6).

## Files

- `src/stats.rs` — **new**: `Mode`, `OpponentRecord`, `Streak`, `ModeRecord`,
  `LifetimeStats`, `RunStats`, `win_rate`; recording + derivation methods; logic
  tests. Added to `src/lib.rs`.
- `src/records.rs` — **new**: `RecordsView`, `RecordsOutcome`, `RecordsState` +
  `handle_input`; pure `collection_line`/`view_body`/`view_title`; `draw`;
  input + content tests. Added to `src/lib.rs`.
- `src/profile.rs` — `stats` field (serde-default, no version bump), `stats()`,
  `record_match`, `distinct_side_cards_owned`, amended `reset_to_starter` (+
  amended reset test, + older-profile-defaults test).
- `src/campaign.rs` — `run_stats` field (serde-default), `run_stats()` /
  `run_stats_mut()`; round-trip test.
- `src/app.rs` — the once-per-match recording block in `tick`; `open_records`;
  the `Screen::Records` input / `?`-help / draw wiring.
- `src/screen.rs` — `Screen::Records` variant.
- `src/menu.rs` — `MenuItem::Records` + `Display` + list slot; updated menu
  tests.
- `DECISIONS.md`, `ROADMAP.md`, `spec.md` — close-out.
- **No change**: `game.rs`, `player.rs`, `card.rs` (read-only), `save.rs` (the
  mid-match save is unchanged — stats live in `profile.json`), `board.rs`,
  `render.rs`, `frame.rs`, `overlay.rs`, `layout.rs`, `banter.rs`, `audio.rs`,
  `play_log.rs`, `opponent.rs` (roster read-only).

## Tests

Each behavioral claim names the task that owns its check. Two are
human-attested (marked).

- **Recording updates the right tally** (T002): `Profile::record_match` bumps
  the correct mode's `OpponentRecord` — a Quick Play win adds a match-win +
  `player_rounds` round-wins + `opp_rounds` round-losses under Quick Play and
  nothing under Campaign; a campaign loss the mirror under Campaign; round W/L
  derives from `rounds_won` (a 3–1 win → 3 round-wins, 1 round-loss). Guards the
  match/round acceptance criteria and the tension §2 derivation.
- **Streak transitions** (T002): `Streak::record` — consecutive wins raise
  `current` and track `longest`; a loss zeroes `current` and preserves
  `longest`. The overall streak advances/resets across an interleaved
  Quick-Play/Campaign win sequence (can't be recombined from mode streaks).
- **Campaign completion** (T002): a campaign win that makes `run_complete()`
  true increments `campaign_completions`; a non-final campaign win does not;
  after `reset_to_starter` a fresh full clear increments it again. Guards the
  completion-persists / not-reset-by-New-Campaign criteria.
- **Serde is additive, no version bump** (T002): a `LifetimeStats` and a
  `RunStats` round-trip through JSON; a pre-020 `profile.json` (no `stats` key,
  no `run_stats` key) loads with defaults and is accepted (mirrors
  `credits_persist_and_default_to_zero_for_older_profiles`). `PROFILE_VERSION`
  is unchanged.
- **Reset preserves lifetime, clears the run** (T002): the amended
  `reset_to_starter` test — dirty stats + a run tally + credits + campaign
  progress, reset, then `stats` is preserved while credits/collection/deck/
  campaign/run tally are starter.
- **Derived totals & win rate** (T002): `ModeRecord::totals`, `combined_opponent`
  (Quick Play + Campaign summed), and `win_rate` (`None`/`0%` at zero matches;
  rounded percentage otherwise, e.g. 3 of 4 → 75%).
- **Screen input** (T004): `RecordsState::handle_input` — Left/Right page
  through the four views wrapping and reset scroll; Up/Down/PgUp/PgDn change
  scroll; Esc/`x` → `Back`; unknown → `None`. (Copies `opponent_select` nav
  tests.)
- **View content** (T004): `view_body` — every `OPPONENTS` name appears in each
  breakdown view; the Overall breakdown equals Quick Play + Campaign per
  opponent; the summary shows matches/won/lost/win-rate substrings; the Campaign
  view shows a completions line and the others do not; ThisRun with zero matches
  shows the "no matches this run yet" line and with matches shows the run
  summary; a never-faced opponent shows `0–0`. `collection_line` renders
  "N of 15" and the percentage. Guards the four-view / persistent-line /
  whole-roster / empty-state criteria.
- **The recording seam fires once per match, in both modes; the screen reads
  correctly at 139×31; long views scroll; the change persists across a restart**
  — *driven, not unit-tested.* Verified by playing (T003 for recording, T005 for
  the screen): finish a Quick Play match and a Campaign match, quit, relaunch,
  and confirm the Records screen shows the updated match/round W/L, the streak
  advancing on wins and resetting on a loss, campaign completions after clearing
  the final node (and unchanged after New Campaign while This Run empties), the
  collection line matching the collection, all four views paging with Left/Right,
  long views scrolling, and the empty first-run state reading cleanly — all
  monochrome at 139×31 and wider. **Marked needing verification** (driver /
  person attestation). Back up + checksum-restore the real profile/saves first.
- **No engine/save-format change** — structural: no edit under
  `game.rs`/`player.rs`/`card.rs`/`save.rs`; the mid-match save format is
  untouched. **Confirmed by the diff at the sweep** (T006).

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim per the constitution.
- **Driver / person attestation** (back up + checksum-restore the real profile/
  saves first, per the standing data-safety practice): the T003 and T005
  checklists above, snapshotted at 139×31 and wider, across a Quick Play match,
  a campaign match, a campaign completion, and a New Campaign.

## Non-goals (from spec)

No achievements/rewards tied to stats (records are informational; nothing in
gameplay reads them); no time-series/graphs/per-session history (running
aggregates only); no in-app "clear records" control (lifetime records are
permanent; the only reset is New Campaign clearing This Run); no recording of
abandoned/quit matches (only game-over matches count — see tension §2); no
per-mode streaks as separate stored/displayed stats (tension §5); no stats for
the generic `default` fallback opponent (the roster is the set of real
opponents).

## Open questions

None. Two design decisions surfaced during planning are now settled:
1. Per-mode streak: **not stored** — Erik-ruled (2026-09-09). spec.md was
   updated so Entities, Non-goals, and the acceptance criteria all agree, and
   the two mode views omit the streak line (tension §5). Streaks are
   overall-lifetime + current-run only; additive to add later if ever wanted.
2. Recording happens only at the match-end seam (tension §2), deriving round W/L
   from `rounds_won`, rather than at a separate round-resolution seam as the
   spec sketched. Observable behavior is identical and strictly more consistent
   with "abandoned records nothing"; sign-off confirmed it satisfies the
   round-W/L and abandoned criteria.
