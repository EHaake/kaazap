# Plan: Wager & loss condition — spec 021

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

## Context

The campaign economy pays on a win and never charges. This spec makes it
two-directional: a campaign launch opens a **wager prompt** (a modal over the
map), the chosen stake is **escrowed** out of the balance at match start, a win
returns it doubled and a loss forfeits it; **going broke** (balance below the
cheapest ante the player could launch) raises a **run-over modal** whose only
exit is spec 014's `reset_to_starter`; **cleared planets become rematches**
against their final opponent; and the free win credit + card drop are removed.

It is a **profile/economy change with thin UI**: the staking core (ante floor,
escrow, payout, broke test, shop reserve, once-only completion) is pure `Profile`
/ `economy` / `CampaignRun` logic with unit tests; the screens call it. The
engine (`game.rs`, `player.rs`, `card.rs`), the mid-match save format
(`save.rs`), and the board rules are untouched. Persistence is one additive
serde-defaulted field (`stake` on `NodeRef`) plus a changed *default* for a
fresh profile's credits — **no `PROFILE_VERSION` or `SAVE_VERSION` bump**.

## What the code already gives us

- **The in-flight campaign pointer is exactly the escrow's home.**
  `CampaignRun.in_progress: Option<NodeRef>` (`campaign.rs:146-167`) is set at
  `start_match` (`app.rs:543`), persisted in `profile.json`, survives a
  mid-match save/quit/Continue untouched (the `SavedGame` in `save.rs` never
  carries it — it is profile state), and is cleared on the game-over
  acknowledgement (`app.rs:791`) or replaced by the next `start_match`. Adding
  `#[serde(default)] stake: u32` to `NodeRef` gives the stake the same lifetime
  for free: it persists with the pointer, resumes with it, and is dropped with
  it. A pre-021 profile's `in_progress` (no `stake` key) loads as stake 0.
- **The once-per-match seam exists and is reliable.** `App::tick` computes
  `phase_changed` (`app.rs:1208-1213`) and spec 020 established (and reviewed)
  that `GamePhase::GameOver` is entered only inside `GameState::update()` from
  `tick`, never from a keypress, and a saved match is never at `GameOver`
  (`save.rs:89`). So `phase_changed && GameOver` fires exactly once per match —
  the seam the record block already uses (`app.rs:1264-1280`). The older
  every-tick campaign-win block (`app.rs:1233-1255`) relied on
  `!is_opponent_beaten` as its once-guard; **rematches break that guard** (a
  beaten node is launchable again), so both blocks fold into the one edge-based
  block (design §4).
- **`mark_beaten` is idempotent and `run_complete()` is derived** from the
  beaten set (`campaign.rs:172-177, 214-216`), so a rematch win can call
  `mark_beaten` harmlessly (no progress change) and completion can be detected
  as the edge `!run_complete()` before → `run_complete()` after, which a
  rematch can never produce (tension §2).
- **The difficulty scalar is already the formula.** `economy::win_reward`
  computes `(threshold − 14) × 10` (`economy.rs:96`); the ante floor is the
  same expression under a new name, and every roster opponent has a
  `stand_threshold` 15–19 (`opponent.rs`; `DEFAULT_OPPONENT` = 17).
- **Affordability already lives in the profile.** `Profile::try_purchase`
  decides (`profile.rs:219-227`) and the shop's draw computes its own
  `affordable = credits >= price` (`shop.rs:111`). Adding a reserve is one
  `Profile::can_afford` used by both.
- **The modal convention is established.** `Modal` (`app.rs:256-279`) holds
  one open panel; `handle_key` dispatches to it first; `draw` draws it last.
  `Modal::Records(RecordsState)` is the pattern for a modal with its own
  module (state + `handle_input` + `draw`), and `draw_two_choice`
  (`app.rs:1368-1401`) is the bordered-box pattern for the small confirms —
  `OverlayLayout::new(config, content_width, 5)` centers a five-row box, of
  which row 1 is currently unused (title 0, choices 2, hint 4).
- **`reset_to_starter` is the single reset** (`profile.rs:126-130`) and
  `start_new_campaign` (`app.rs:500-508`) is its app-level wrapper (save,
  clear the match save, drop the banner, open the map). The run-over
  acknowledgement calls exactly that.
- **The presence panel has reserved geometry.** `PANEL_H_INMATCH` = 18
  (`portrait.rs:20`) gives 16 interior rows, all used (name 0, portrait 1–12,
  gap 13, banter 14, pips 15). The panel is top-aligned with the 31-row board
  block, so growing it to 20 rows keeps it inside the 139×31 minimum
  (`layout.rs` test asserts `opponent_panel.y1 <= 30`; it becomes 19). The
  campaign-map rail sizes itself from `PORTRAIT_HEIGHT` (`layout.rs:224`), not
  `PANEL_H_INMATCH`, so it is unaffected.
- **The map banner slot exists.** `last_reward: Option<WinReward>` is set at
  the win seam, drawn on the header's second row, and cleared on the next map
  key (`app.rs:919-921`). Generalizing it to a `MapBanner` gives the "Won N",
  "Lost N", and "can't cover the ante" messages one transient slot.
- **`OpponentProfile` and `Planet` are `Copy`** (`OPPONENTS[3]` is copied in
  `save.rs` tests; `Planet` derives `Copy`), so the wager modal can hold both by
  value with no lifetimes.

## Design tensions resolved

### 1. The stake lives on `NodeRef`, not on `CampaignRun` or the match save

`NodeRef { planet, opponent, stake }` with `#[serde(default)] pub stake: u32`.
The stake is *part of* the in-flight match context: it exists iff a campaign
match is in flight, persists in `profile.json` beside the pointer, and is
discarded with it. A separate `CampaignRun.stake` field would need its own
clear on every path that clears the pointer; a `SavedGame` field would put
economy data in the engine save and need a save-format change. The
`NodeRef` literal constructors (`launch`, two tests) gain the field.

### 2. Settlement and completion move into one `Profile::settle_campaign_match` — *deviation from spec 020's plan*

Spec 020 counted a campaign completion inside `Profile::record_match` as
`player_won && run_complete()`, relying on "a completed run exposes no
launchable match". Rematches void that. The completion check moves out of
`record_match` into the new `settle_campaign_match`, which owns `mark_beaten`
and detects completion as the **edge** `!was_complete && run_complete()` around
it. A rematch (node already beaten) leaves `beaten` unchanged, so the edge
never fires; a first clear of the final node fires it exactly once. This also
removes spec 020's "record block must run after the win block" ordering
requirement — the ordering is internal to one method. `record_match` keeps
doing lifetime + run-tally recording only. Pinned by T002 tests.

### 3. Exactly-once payout is a data property: `take_stake` zeroes the escrow

`settle_campaign_match` calls `CampaignRun::take_stake()` (returns the stake
and sets it to 0) before paying. Even if the seam fired twice, the second
settlement would pay `win_payout(0) = 0` and `mark_beaten` is idempotent.
Consequence: at game over the escrow reads 0, so the in-match stake line
(design §8) disappears on the game-over tick while the outcome popup and the
map banner carry the result. A player who quits at the game-over screen
without acknowledging leaves a stale `in_progress` with stake 0 — harmless
(nothing at risk, nothing to pay) and replaced by the next launch.

### 4. One resolution block on the `phase_changed` edge

The every-tick campaign-win block and the edge-based record block become one
block, in this order: `settle_campaign_match` (escrow → payout/forfeit,
`mark_beaten`, completion) → `record_match` (stats) → `save`. The
`!is_opponent_beaten` once-guard is gone; the once-guarantee is the
`phase_changed` edge (established in spec 020) plus tension §3.

### 5. The broke check is a pure predicate run at two app seams

`Profile::is_broke()` = `credits < economy::cheapest_floor(run)`. It is
evaluated (a) on the **game-over acknowledgement** of a campaign match, after
the map opens, and (b) at **campaign entry** — `enter_campaign_continue`'s
no-save branch and the confirmed discard-and-enter — through one
`App::enter_campaign_map()` helper (open the map, then raise `Modal::RunOver`
if broke). It is *not* run on every `open_campaign_map` (shop/deck-builder
Back) — the shop reserve makes those paths unable to create a broke state, and
keeping the check at the two spec'd seams keeps intent legible. A win can
never leave the player broke (`credits ≥ win_payout(stake) ≥ 2·floor`), so
the ack-time check needs no "only after a loss" guard.

### 6. The shop reserve is inside `try_purchase`, surfaced as "spendable"

`Profile::can_afford(price)` = `credits ≥ price + cheapest_floor(run)`;
`try_purchase` uses it, and the shop's dimming reads it — one rule, one place.
Because a card can now be refused with `credits ≥ price`, the shop's balance
row also shows the spendable amount (`Credits: ◈ 53  ·  spendable ◈ 43`) so
the dimming is explicable. Cosmetic wording is the implementer's.

### 7. The wager prompt is a stake grid, not free arithmetic

`WagerState` stores an index `k` over the grid `floor + k·STAKE_STEP`, clamped
to the balance: `stake = min(floor + k·STEP, balance)`, `k ∈ 0..=k_max`,
`k_max = ceil((balance − floor) / STEP)`. Left/Right move `k`. This makes
"all-in" reachable when the balance is off-grid (floor 10, balance 53 →
10, 15, …, 50, 53) and Left from all-in lands back on the grid (50), with no
special cases. A move that can't happen (Left at the floor, Right at the
balance) returns `None` (no move cue), like the map's `step`.

### 8. The in-match stake line grows the presence panel by two rows

`PANEL_H_INMATCH` 18 → 20; `draw_presence_extras` gains `stake: Option<u32>`
and draws `Stake ◈ N` centered on interior row 17 (row 16 blank, mirroring the
gap above the banter row). `Some` only for a staked campaign match; Quick Play
passes `None` and the rows stay blank. No layout constant for the minimum size
changes (`IN_MATCH_MIN_WIDTH` uses `PANEL_W`; the height minimum is the board
block).

### 9. Discard confirmations get a second line, not a longer title

`draw_two_choice` gains `note: Option<&str>` drawn Muted on the unused row 1.
`Discard your saved match?` / `New campaign? Erases progress, credits & cards.`
keep their titles; when a stake is at risk the note reads
`…and forfeit your N-credit stake.` The Quick-Play-over-a-save confirm keeps
today's semantics: the forfeit takes effect when the Quick Play match actually
starts (`start_match(None)` replaces the pointer); backing out of opponent
select leaves the save and the escrow intact — exactly as the save itself is
only discarded then.

### 10. Terminal minimum

The spec says the prompt and notice "fit 89×31 on the map"; the app's global
minimum has been **139×31** since spec 016 (`Config::min_size`). The boxes are
small (`OverlayLayout`-centered, < 70 columns) and fit either; the close-out
snapshots are at 139×31.

## Design

### 1. `src/economy.rs` — constants, floor, payout, cheapest floor; drop the card drop

```rust
/// Credits a fresh or reset profile starts with (spec 021).
pub const SEED_PURSE: u32 = 50;
/// Ante floor = (threshold − ANTE_BASE_THRESHOLD) × ANTE_PER_THRESHOLD_STEP.
pub const ANTE_BASE_THRESHOLD: usize = 14;
pub const ANTE_PER_THRESHOLD_STEP: u32 = 10;
/// The wager prompt's increment.
pub const STAKE_STEP: u32 = 5;
/// Winnings per credit staked (1 = even money): a win returns stake × (1 + PAYOUT_RATIO).
pub const PAYOUT_RATIO: u32 = 1;

pub fn ante_floor(threshold: usize) -> u32 { threshold.saturating_sub(ANTE_BASE_THRESHOLD) as u32 * ANTE_PER_THRESHOLD_STEP }
pub fn win_payout(stake: u32) -> u32 { stake.saturating_mul(1 + PAYOUT_RATIO) }  // what returns to the balance
pub fn ante_floor_for(opponent_id: &str) -> u32  // opponent_by_id(id).map_or(crate::STAND_THRESHOLD, |o| o.stand_threshold) → ante_floor
/// Min ante over every node the player could launch now (unlocked planets ×
/// `launchable_opponent`); 0 if none (unreachable — the start planet is always unlocked).
pub fn cheapest_floor(run: &CampaignRun) -> u32

/// How a staked campaign match settled (for the map banner).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakeOutcome { Won(u32), Lost(u32) }   // the stake, not the payout
```

Remove `WinReward`, `win_reward`, and `win_reward_scales_credits_and_picks_the_card_by_roll`
— in T004, together with the tick block that calls them, so each task builds
on its own. `available_pool`, `card_price`, tiers stay (the shop). Module doc:
the roll seam is gone — nothing here is random any more.

### 2. `src/campaign.rs` — the stake on `NodeRef`, rematch rule, escrow accessors

- `NodeRef` gains `#[serde(default)] pub stake: u32`.
- `pub fn launchable_opponent(&self, planet: &Planet) -> Option<&'static str>` —
  `next_opponent(planet).or_else(|| planet.opponents.last().copied())`: the
  next un-beaten opponent, or the **final** opponent of a cleared planet (the
  rematch). Callers gate on `planet_unlocked`.
- `pub fn stake_at_risk(&self) -> Option<u32>` — `in_progress` stake when `> 0`.
- `pub fn take_stake(&mut self) -> u32` — returns the in-flight stake and sets
  it to 0 (0 when no match is in flight).

### 3. `src/profile.rs` — seed purse, escrow, settlement, broke test, reserve

- `Default`: `credits: economy::SEED_PURSE`. The serde field default stays 0
  (`#[serde(default)]`), so an existing file keeps its balance and a pre-012
  file (no `credits` key) still loads 0; `reset_to_starter` (unchanged code)
  now yields 50 because it replaces with `Profile::default()`.
- `pub fn stake_match(&mut self, node: NodeRef) -> bool` — if
  `node.stake <= credits`: `credits -= node.stake`, `set_in_progress(Some(node))`,
  `true`; else nothing, `false`.
- `pub fn settle_campaign_match(&mut self, player_won: bool) -> Option<StakeOutcome>`:
  ```rust
  let node = self.campaign.in_progress()?.clone();       // None for Quick Play
  let stake = self.campaign.take_stake();
  if player_won {
      self.credits = self.credits.saturating_add(economy::win_payout(stake));
      let was_complete = self.campaign.run_complete();
      self.campaign.mark_beaten(&node.planet, &node.opponent);   // idempotent — a rematch changes nothing
      if !was_complete && self.campaign.run_complete() {
          self.stats.record_campaign_completion();
      }
      Some(StakeOutcome::Won(stake))
  } else {
      Some(StakeOutcome::Lost(stake))
  }
  ```
- `record_match`: **remove** the `player_won && run_complete()` completion
  clause (tension §2); the Campaign arm keeps the run-tally call.
- `pub fn is_broke(&self) -> bool` — `credits < economy::cheapest_floor(&self.campaign)`.
- `pub fn can_afford(&self, price: u32) -> bool` — `credits >= price.saturating_add(cheapest_floor)`;
  `try_purchase` uses it (deducts `price` only, never the reserve).
- Remove `apply_win_reward` and its test and drop the `WinReward` import (in
  T004, with its caller; T002 only makes the old test seed-relative so it keeps
  passing meanwhile). Keep `earn_credits` (tests) and `grant_card` (`try_purchase`).

### 4. `src/app.rs` — the flow

- **`Modal`** gains `Wager(WagerState)` and `RunOver` (unit-like).
- **`last_reward: Option<WinReward>`** → `banner: Option<MapBanner>`; all
  `last_reward = None` sites become `banner = None`.
- **Launch gate.** `launch_campaign_node(planet, opponent)` becomes the
  pre-prompt gate: deck-invalid → divert to the deck builder (as today); else
  `floor = ante_floor(opp.stand_threshold)`; if `credits < floor` →
  `banner = Some(MapBanner::CantCover { floor })`, `Sfx::MenuBack`; else
  `modal = Some(Modal::Wager(WagerState::new(planet, opp, credits)))`.
- **`fn handle_wager_input(&mut self, key)`** (a `Modal::Wager` branch in
  `handle_key`, before the `CampaignEntry` branch): `Moved` → `MenuMove`;
  `Cancel` → `modal = None`, `MenuBack`; `Commit` → read `(planet_id,
  opponent, stake)` off the state, `modal = None`, `MenuSelect`,
  `start_match(opponent, Some(NodeRef { planet, opponent, stake }))`.
- **`start_match`**: replace `set_in_progress(campaign)` with
  ```rust
  match campaign {
      // Escrow (spec 021): the stake leaves the balance now. The prompt
      // clamps to the balance, so this cannot fail; if it ever did, launch nothing.
      Some(node) => if !self.profile.stake_match(node) { return; },
      None => self.profile.campaign_mut().set_in_progress(None),
  }
  ```
  (the `profile.save()` that follows persists the escrow before the board opens).
- **`tick`**: delete the every-tick campaign-win block; the edge block becomes
  ```rust
  if phase_changed && let Screen::InGame { game_state, .. } = &self.screen
      && matches!(game_state.game_phase, GamePhase::GameOver { .. })
  {
      let player_won = …; let opponent_id = …; let (player_rounds, opp_rounds) = …;
      let mode = if self.profile.campaign().in_progress().is_some() { Mode::Campaign } else { Mode::QuickPlay };
      if let Some(outcome) = self.profile.settle_campaign_match(player_won) {
          self.banner = Some(MapBanner::Settled(outcome));
      }
      self.profile.record_match(mode, opponent_id, player_won, player_rounds, opp_rounds);
      self.profile.save();
  }
  ```
  `mode` is read **before** settlement only for clarity — settlement never
  clears the pointer (the ack does).
- **Game-over acknowledgement** (`app.rs:780-796`): `open_campaign_map()` →
  `enter_campaign_map()`.
- **`fn enter_campaign_map(&mut self)`**: `open_campaign_map(); if
  self.profile.is_broke() { self.modal = Some(Modal::RunOver); }`. Used by the
  ack, `enter_campaign_continue`'s no-save branch, and the
  `PendingStart::Campaign` Yes arm — which additionally makes the forfeit
  explicit: `set_in_progress(None)` + `profile.save()` beside the existing
  `save::clear()`.
- **`fn handle_run_over_input(&mut self, key)`**: `run_over_acknowledged(key)`
  (pure: `Enter | ' '` → true, everything else false — Esc does **not**
  dismiss) → `modal = None; start_new_campaign()`. Else ignore.
- **`draw`**: `Some(Modal::Wager(state)) => state.draw(frame, &self.config, pulse)`;
  `Some(Modal::RunOver) => self.draw_run_over(frame)` — a `draw_two_choice`-
  style bordered box: title `You're broke — the run is over.`, note `Deck,
  collection, and progress reset to the starter; your records stay.`, hint
  `Enter  continue` (Emphasis: title Strong, note Normal, hint Muted).
- **`draw_two_choice`** gains `note: Option<&str>` (row 1, Muted);
  `draw_confirm_new_game` / `draw_confirm_new_campaign` pass
  `self.profile.campaign().stake_at_risk().map(|s| format!("…and forfeit your {s}-credit stake."))`;
  `draw_campaign_entry` passes `None`.
- **`Screen::InGame` draw**: `self.board_view.draw(game_state, cursor,
  self.banter, self.profile.campaign().stake_at_risk(), pulse, frame)`.
- Shop `Buy` arm unchanged (the rule is in `try_purchase`).

### 5. `src/wager.rs` — the wager prompt (new module, copies `records.rs`'s modal shape)

```rust
pub enum WagerOutcome { Moved, Commit, Cancel }
pub struct WagerState { planet: Planet, opponent: OpponentProfile, floor: u32, max: u32, k: usize }
impl WagerState {
    /// Precondition: balance >= ante_floor(opponent.stand_threshold) (the launch gate checks).
    pub fn new(planet: Planet, opponent: OpponentProfile, balance: u32) -> Self  // k = 0 → opens at the floor
    pub fn stake(&self) -> u32                 // min(floor + k·STAKE_STEP, max)
    pub fn planet_id(&self) -> &'static str; pub fn opponent(&self) -> OpponentProfile
    pub fn handle_input(&mut self, key: KeyCode) -> Option<WagerOutcome>
    //  Left/'a' → k−1 (None at 0); Right/'d' → k+1 (None at k_max); Enter/' ' → Commit;
    //  Esc/'x' → Cancel; else None.  (Ctrl+B/F arrive as Left/Right via resolve_key.)
    pub fn lines(&self) -> Vec<String>         // pure content for tests + draw
    pub fn draw(&self, frame: &mut Frame, config: &Config, pulse: Emphasis)
}
```

`lines()`: `Wager — {opponent.name} · {planet.name}`; `Ante ◈ {floor}   Balance ◈ {max}`;
`◂  Stake ◈ {stake}  ▸`; `Win +{win_payout(stake) − stake}   ·   Lose −{stake}`;
hint `←/→ stake  ·  Enter play  ·  Esc back`. `draw` uses
`OverlayLayout::new(config, widest_line, lines.len())` + `clear_rect` +
`draw_box`, `draw_text_in` centered; the stake row takes `pulse`, the rest
`Normal`/`Muted` — monochrome by construction. `▸`/`◂` may dim when the step
is unavailable (cosmetic).

### 6. `src/campaign_map.rs` — rematch launch, the banner

- `MapBanner { Settled(StakeOutcome), CantCover { floor: u32 } }` (replaces the
  `WinReward` import); `draw(…, banner: Option<&MapBanner>, …)`.
- Enter/Space: `run.launchable_opponent(&planet).map(|opponent| Launch { … })`
  — a cleared planet now launches its final opponent. The doc comment and
  `enter_on_a_cleared_planet_is_a_no_op` change accordingly.
- `draw_header`: `Settled(Won(s))` → `★  Won {s} credits` (Strong);
  `Settled(Lost(s))` → `Lost {s} credits` (Normal); `CantCover { floor }` →
  `Can't cover the {floor}-credit ante` (Normal); `None` → the axis label.
- `draw_panel`: the cleared status reads `Cleared — Enter to rematch {name}.`
  and the complete status `Campaign complete — rematches stay open.` (wording
  cosmetic). The right-rail preview uses `launchable_opponent` (same result as
  today's `next.or_else(last)`).

### 7. `src/shop.rs` — reserve-aware dimming and readout

`affordable = profile.can_afford(price)`; balance row
`Credits: ◈ {credits}  ·  spendable ◈ {credits.saturating_sub(cheapest_floor)}`.
`handle_input` unchanged.

### 8. `src/portrait.rs`, `src/board.rs` — the in-match stake line

`PANEL_H_INMATCH = 2 + 1 + PORTRAIT_HEIGHT + 1 + 2 + 2` (20).
`draw_presence_extras(frame, panel, banter, opponent_rounds_won, stake: Option<u32>)`
draws `Stake ◈ {n}` centered, Strong, on interior row 17 when `Some`.
`BoardView::draw(state, cursor, banter, stake, pulse, frame)` forwards it.

### 9. Docs

`docs/economy.md` is rewritten around the two-directional loop: constants
table (seed purse, ante floor formula and per-threshold table, stake step,
payout, no cap), escrow/settlement, rematches, broke + run-over, the shop
reserve, persistence (`stake` on `NodeRef`, seed purse for new/reset only, no
version bump), and the guard tests. `README.md` feature blurb (lines 13–14,
24–25, 74–75) drops "card packs"/"drop cards" and mentions stakes, rematches,
and going broke. `ROADMAP.md`: Wager & loss shipped; the balance pass now
unblocked. `DECISIONS.md`: the resolved decisions plus tensions §1–§5.

## Files

- `src/economy.rs` — constants, `ante_floor`, `ante_floor_for`, `win_payout`,
  `cheapest_floor`, `StakeOutcome`; `WinReward`/`win_reward` removed; tests.
- `src/campaign.rs` — `NodeRef.stake`, `launchable_opponent`, `stake_at_risk`,
  `take_stake`; tests.
- `src/profile.rs` — seed purse default, `stake_match`, `settle_campaign_match`,
  `is_broke`, `can_afford`, reserve in `try_purchase`, completion clause out of
  `record_match`, `apply_win_reward` removed; tests (incl. amended reset,
  affordability, completion tests; `NodeRef` literal gains `stake: 0`).
- `src/wager.rs` — **new**: `WagerState`, `WagerOutcome`, `handle_input`,
  `lines`, `draw`; tests. Added to `src/lib.rs`.
- `src/app.rs` — `Modal::Wager`/`RunOver`, `banner`, launch gate, wager and
  run-over handlers + draws, escrow in `start_match`, the merged resolution
  block, `enter_campaign_map`, explicit forfeit on campaign discard, confirm
  notes, stake passed to the board; `run_over_acknowledged` + test.
- `src/campaign_map.rs` — `MapBanner`, rematch launch, banner/status text; tests.
- `src/shop.rs` — `can_afford` dimming, spendable readout.
- `src/portrait.rs`, `src/board.rs` — stake row; tests.
- `docs/economy.md`, `README.md`, `ROADMAP.md`, `DECISIONS.md`, `spec.md` — close-out.
- **No change**: `game.rs`, `player.rs`, `card.rs`, `save.rs` (the stake is
  profile state), `stats.rs`, `records.rs`, `menu.rs` (the confirm text is
  drawn in `app.rs`), `layout.rs`, `frame.rs`, `render.rs`, `overlay.rs`,
  `deck_builder.rs`, `opponent*.rs`, `banter.rs`, `audio.rs`, `play_log.rs`.

## Tests

Each claim names the task that owns its check. Driver items are marked.

- **Ante floor is the difficulty scalar** (T001): `ante_floor(15..=19)` =
  10/20/30/40/50; `ante_floor_for("greeb")` = 10, unknown id → the baseline
  (17 → 30). **Payout is even money** (T001): `win_payout(20)` = 40, `win_payout(0)` = 0.
- **Cheapest floor = min over launchable nodes** (T001): equals the
  independently computed min over unlocked planets × `launchable_opponent` for
  a fresh run, a half-cleared run, and a complete run — and is 10 in all three
  (Cinder's rematch keeps the floor at 10 forever).
- **Rematch rule** (T001): `launchable_opponent` — uncleared → next un-beaten;
  The Anvil with only Brakka beaten → Kesh; The Anvil cleared → Kesh (final);
  cleared single-opponent Cinder → Greeb.
- **Escrow** (T002): `stake_match` with 50 credits and stake 20 → 30 credits,
  `in_progress` `Some` with stake 20, `stake_at_risk() == Some(20)`; stake 60 →
  `false`, nothing changed.
- **Settlement** (T002): from 30 credits / stake 20: won → 70 credits,
  `Won(20)`, node marked beaten, stake now 0 and `stake_at_risk()` `None`;
  lost → 30 credits, `Lost(20)`, node not beaten; a second settle returns
  `Won(0)`/`Lost(0)` and changes nothing; Quick Play (no pointer) → `None`,
  credits untouched. A zero-stake campaign pointer (pre-021 save) settles: win
  marks beaten, pays 0.
- **Rematch changes no progress and counts no completion** (T002): a complete
  run, pointer on Zenith/Sovereign (already beaten), settle a win → completions
  unchanged, beaten set unchanged, credits paid; settle a loss → same, nothing
  un-beaten. **A first clear that completes the run counts once** (T002): the
  rewritten `campaign_completion_counts_only_a_final_clearing_win…` test —
  all-but-final via `settle` (0 completions), final via `settle` (1), reset,
  re-clear (2). `record_match` alone never increments.
- **Broke** (T002): fresh run with 9 credits → broke; 10 → not; a complete run
  with 10 → not (rematch floor); a match in flight does not change the answer.
- **Shop reserve** (T002, rewrite of `earning_grows_the_balance_and_purchase_…`):
  fresh run (floor 10), 60 credits, price 50 → buys (10 left); 59 → refused,
  `can_afford` false, nothing changes; a purchase never deducts the reserve.
- **Persistence, no version bump** (T002): a `NodeRef` JSON without `stake`
  loads as 0; a profile with a staked pointer round-trips the stake;
  `Profile::default().credits() == SEED_PURSE`; an older profile JSON without
  `credits` → 0, with `credits: 75` → 75 (amend the existing credits test);
  `reset_to_starter` → `SEED_PURSE` (amend the reset test's `credits reset`
  assertion); `PROFILE_VERSION == 1`; `save.rs` untouched (sweep).
- **Wager grid** (T003): opens at the floor; floor 10 / balance 53: Right walks
  10,15,…,50,53 then `None`; Left from 53 → 50, down to 10 then `None`;
  balance == floor → Left/Right `None`; Enter → `Commit` at the current stake;
  Space → `Commit`; Esc/`x` → `Cancel`; unknown → `None`; `lines()` contains
  the opponent name, `Ante ◈ 10`, `Balance ◈ 53`, and `+{stake}`/`−{stake}`;
  every line ≤ 70 columns.
- **Map launch** (T004): Enter on cleared Cinder → `Launch { cinder, greeb }`
  (replaces the no-op test); uncleared → next un-beaten (existing test).
- **Run-over acknowledgement** (T005): `run_over_acknowledged` true for
  Enter/Space, false for Esc/`x`/others.
- **Stake row** (T006): `draw_presence_extras(.., Some(40))` puts `◈ 40` on
  frame row 18 (interior 17) and `None` leaves it blank; the banter/pip tests
  still pass at rows 15/16; the board layout test's `opponent_panel.y1 <= 30`
  still holds.
- **End-to-end flow, legibility at 139×31, persistence across restart** —
  *driven, not unit-tested* (T004, T005, T006, T007). Marked needing
  verification: prompt opens/clamps/commits/escrows (`◈` drops), win/loss
  banners, rematch on a cleared planet with no progress change and no
  completion bump, mid-match quit → Continue resumes with the stake line and
  the confirm note, broke after a loss → run-over → fresh map at `◈ 50` with
  records intact, shop dims a card the reserve blocks, Quick Play shows no
  prompt/line/payout, no panics.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim per the constitution.
- **Driver / person attestation** (back up + checksum-restore the real
  profile/saves first): the T004 and T005/T006 checklists, snapshotted at
  139×31 and wider.

## Non-goals (from spec)

No endgame award; no roguelike-mode identity change; no keep-your-cards soft
restart; no stake cap; no selling cards; no odds/payout scaling by opponent;
no rare card drop; no balance tuning (the constants are first guesses for the
balance pass).

## Open questions

None product-level. Two edges are settled here as design, flagged for the
sign-off:
1. The in-match stake line disappears on the game-over tick (tension §3) —
   the outcome popup and the map banner carry the result.
2. A resumed pre-021 campaign save has stake 0: it settles normally (first
   clear still marks beaten) and the banner reads "Won 0 credits" / "Lost 0
   credits" for that one match — honest, and gone after the first staked launch.
