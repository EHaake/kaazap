# Plan: Economy & progression (subsystem C)

Technical design for `spec.md`. The master plan
(`~/.claude/plans/iterative-meandering-blum.md`) mirrors this. Content +
persistence over shipped patterns — no engine, board-renderer, or save-format
change.

## Confirmed seams (exploration)

- **Profile is the extension point** (`profile.rs:40-52`): `collection: Vec<Card>`
  (bag of copies — grant = `Vec::push`), `deck`, `campaign`, each
  `#[serde(default)]`. The `campaign` field (spec 009, no version bump) is the
  `credits: u32` precedent. Persist via explicit `self.profile.save()` after each
  mutation (`app.rs:588-604`).
- **The win seam is `App::tick` (`app.rs:826-839`)** — fires once per node
  (`!is_opponent_beaten` guard) on a campaign `GameOver { winner: Player::Player }`;
  Quick Play (`in_progress == None`) is skipped, which is the campaign-only ruling
  for free. `opponent_by_id(&node.opponent)` → `stand_threshold` (15→19) scales
  credits.
- **`collection_by_type() -> Vec<CardEntry{card,owned,in_deck}>` (`profile.rs:172`)**
  is the shop's read primitive; `try_add_to_deck`/`deck_is_valid` pick up granted
  copies for free.
- **Screen convention** (`deck_builder.rs`): a cursor state struct +
  `handle_input(key, &Profile) -> Option<Outcome>` + `draw(frame, config,
  &Profile, pulse)`, wired into `app.rs`'s three `match &self.screen` arms.
  `GamePhase` is in-match only, so the shop is a `Screen`.

## 1. Profile economy state — `src/profile.rs`

Add `credits: u32` (`#[serde(default)]`, starter 0), keep `PROFILE_VERSION = 1`.
Methods (mirror `try_add_to_deck`'s "validate in Profile" style):
`credits(&self) -> u32`, `earn_credits(&mut self, u32)` (saturating), `grant_card(
&mut self, Card)` (`collection.push`), `try_purchase(&mut self, Card, price: u32)
-> bool` (spend-then-grant if affordable). Add `credits: 0` to the `profile_with`
helper (`profile.rs:201`) and a pre-C-JSON default test.

## 2. Economy module — new `src/economy.rs` (pure, no rendering)

- `pub enum RegionTier { Outer, Mid, Core }` (derive `PartialOrd`/`Ord`) +
  `region_tier(&str) -> RegionTier` (the three region strings; unknown → `Outer`).
- `deepest_reached(run: &CampaignRun) -> RegionTier` = max tier over
  `planet_unlocked` planets (reads `campaign::PLANETS` + `Planet.region`).
- `card_tier(Card) -> RegionTier` — Outer: `+1 +2 +3 -1 -2 -3 ±1`; Mid: `+4 -4 ±2
  ±3 2&4 3&6`; Core: `±6 ±1T`.
- `available_pool(run) -> Vec<Card>` = `ALL_SIDE_CARDS` filtered by `card_tier <=
  deepest_reached`.
- `card_price(Card) -> u32` by tier (Outer 20 / Mid 50 / Core 120 — tunable).
- `pub struct WinReward { pub credits: u32, pub card: Card }` +
  `win_reward(threshold: usize, pool: &[Card], roll: usize) -> WinReward`:
  `credits = (threshold.saturating_sub(14) * 10) as u32`; `card = pool[roll %
  pool.len()]`. Pure/roll-seam-testable (spec-010 pattern).

## 3. Win rewards + reveal — `src/app.rs`, `src/campaign_map.rs`

In the once-per-win block (`app.rs:832-839`), after `mark_beaten`: resolve the
opponent, `let pool = economy::available_pool(self.profile.campaign());` (always
non-empty — Outer is always unlocked), `let reward = economy::win_reward(thresh,
&pool, rand::random_range(0..pool.len()));`, `earn_credits` + `grant_card`, set
`self.last_reward = Some(reward)`, `save()`. Add `last_reward: Option<WinReward>`
to `App`. Reveal **inline** on the campaign map — a concise line (credits + card)
via `last_reward`, cleared on next navigation; add `credits` to the map header
(`campaign_map.rs:208`). Board stays economy-free.

## 4. Shop — new `src/shop.rs` + wiring

`Screen::Shop { state: ShopState }`, `ShopState { cursor: usize }`, `ShopOutcome {
Moved, Buy(Card), Back }`. `handle_input(&mut self, key, &Profile) ->
Option<ShopOutcome>` (arrows/`wasd` move; Enter/Space `Buy`; Esc/`x` `Back`).
`draw(&self, frame, config, &Profile, pulse)` — balance, the available pool rows
(card · price · owned, unaffordable `Muted`, cursored pulsing), a hint. App:
`Buy(card)` → `try_purchase(card, economy::card_price(card))` then `save()` + SFX.
Reached from the map via a new `MapOutcome::OpenShop` (a shop key + hint) →
`App::open_shop()`.

Compiler-enforced wiring: `lib.rs` (`pub mod economy; pub mod shop;`), `screen.rs`
(`Shop` variant), `app.rs` (three `match &self.screen` arms + import + `open_shop`
+ `last_reward`), `campaign_map.rs` (`MapOutcome::OpenShop` + header credits +
reward line).

## Tests

- **Profile:** pre-C JSON → `credits == 0` round-trip; `earn_credits` accumulates;
  `try_purchase` grants + deducts when affordable, no-ops when broke; `grant_card`
  → `collection_by_type` reflects it.
- **Economy:** `card_tier` partitions all 15 `ALL_SIDE_CARDS`; every `PLANETS`
  region is a known tier; pool grows Outer ⊆ Mid ⊆ Core and ⊆ universe; prices > 0;
  `win_reward` deterministic given `roll` (credits by threshold; card by index).
- **Shop:** `handle_input` outcomes; an unaffordable `Buy` changes nothing.
- **Win hook:** covered by the pure `win_reward` + `Profile` tests (the `tick`
  seam has no existing unit test; the hook stays thin over tested pieces).

## Files

New: `src/economy.rs`, `src/shop.rs`. Modified: `src/profile.rs`, `src/app.rs`,
`src/screen.rs`, `src/lib.rs`, `src/campaign_map.rs`. Close-out: `docs/economy.md`,
README, `ROADMAP.md`, `DECISIONS.md`. No change: `game.rs`/`player.rs`/`card.rs`
values, `board.rs`, `opponent.rs`, the save format.

## Verification

`cargo build` (no new warnings) + `cargo test` (verbatim). Driver sweep (back up
real `profile.json`/`saves/`): win a campaign match → credit + drop shown on the
map, collection grew (inspect `profile.json`); open the shop from the map → pool
matches depth, prices + balance shown, an affordable buy deducts + grants, a
too-dear one refused; the deck-builder sees the new copies; snapshot the shop at
89×31; no panics.
