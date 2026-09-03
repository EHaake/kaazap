# Tasks: Economy & progression (subsystem C)

Each task builds (`cargo build`) and tests (`cargo test`) green before it's done,
with actual output reported. One commit per task referencing its ID; the draft PR
(opened at T001) tracks the diff. Reviewed **per task** — this is foundational
persistence + a new screen. Do not weaken a test to make it pass.

Branch `012-economy` created with the spec/plan/tasks. Plan:
`~/.claude/plans/iterative-meandering-blum.md` (mirrors this spec's docs).

---

- [x] **T001 — Profile economy state**
  Add `credits: u32` (`#[serde(default)]`, starter 0) to `Profile`
  (`src/profile.rs`), keeping `PROFILE_VERSION = 1`. Methods `credits`,
  `earn_credits` (saturating), `grant_card` (push to collection), `try_purchase`
  (spend-then-grant if affordable). Update the `profile_with` test helper.
  *Verify: `cargo build` no new warnings; `cargo test` green — pre-C JSON loads
  with `credits == 0` (round-trip), `earn`/`grant`/`try_purchase` behave (affordable
  vs broke), existing profile/deck tests still pass. Open the draft PR.*

- [x] **T002 — Economy module**
  New `src/economy.rs` (pure logic): `RegionTier` + `region_tier`,
  `deepest_reached`, `card_tier`, `available_pool`, `card_price`, and
  `win_reward(threshold, pool, roll)` with `WinReward`.
  *Verify: `cargo test` green — `card_tier` partitions all 15 `ALL_SIDE_CARDS`;
  every `PLANETS` region maps to a known tier; the pool grows Outer ⊆ Mid ⊆ Core
  and stays ⊆ the universe; prices > 0; `win_reward` is deterministic given `roll`;
  `cargo build` clean.*

- [x] **T003 — Win rewards + map reveal**
  Hang the reward on the once-per-win seam (`src/app.rs:832-839`): earn credits +
  drop a card from `available_pool`, store `last_reward`, save. Show the credit
  balance in the campaign-map header and the reward line on return from a win
  (`src/campaign_map.rs`). Board stays economy-free.
  *Verify: `cargo test` green — reward logic covered by the pure `win_reward` +
  `Profile` tests; existing campaign/app tests still pass; `cargo build` clean.
  Driver: a campaign win shows the reward and grows `profile.json`.*

- [ ] **T004 — Shop screen**
  New `src/shop.rs` (`ShopState`/`ShopOutcome` + `handle_input` + `draw`) and the
  `Screen::Shop` wiring (`lib.rs`, `screen.rs`, `app.rs`'s three arms + `open_shop`),
  reached from the campaign map via `MapOutcome::OpenShop`. Buying goes through
  `try_purchase`.
  *Verify: `cargo test` green — shop `handle_input` outcomes; an unaffordable buy
  no-ops; menu/map tests updated for the new affordance; `cargo build` clean.*

- [ ] **T005 — Verification & close-out**
  Full driver sweep (back up `profile.json`/`saves/`): win a campaign match (credit
  + drop shown, collection grew), open the shop from the map (pool matches depth,
  prices + balance, affordable buy grants + deducts, too-dear refused), the
  deck-builder sees new copies, shop legible at 89×31, no panics. Run the
  `skeptical-reviewer`. Write `docs/economy.md` (mechanics + tuning snapshot) and
  update the README. On the human's word: `ROADMAP.md` (C → Shipped) + a
  `DECISIONS.md` spec-012 entry on `main`; mark ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported verbatim;
  reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The economy
extends the **profile** (`credits` is an additive `#[serde(default)]` field, the
`campaign`-field precedent — no version bump) and hangs rewards on the existing
once-per-win campaign seam (`app.rs:832-839`), which already excludes Quick Play.
The pool is `card::ALL_SIDE_CARDS` gated by the map's three regions; keep the
reward math a pure `win_reward(threshold, pool, roll)` so it's roll-seam-testable
(the spec-010 pattern). The shop is a `Screen` copying `deck_builder.rs`; the
collection is a bag of copies, so a grant/purchase is one `Vec::push` and the
deck-builder sees it for free. Board/engine stay economy-free.
