# Plan: Locked cards in the Outfitter — spec 025

> **Status**: Signed off (skeptical-reviewer at opus, 2026-09-16)
**Implements**: `spec.md` in this directory

## Context

The Outfitter today lists `economy::available_pool(run)` — only what the
current depth can buy. This spec lists all 15 cards, grouped Outer Rim /
Mid Rim / Core, with the unreached groups dimmed under a heading that names
the region that opens them, and a cursor that only visits unlocked cards.

It is a **UI-only spec**. The change is `src/shop.rs` (listing, cursor, draw,
tests), one read-only method on `RegionTier` in `src/economy.rs` (the region's
display name — no change to the tier table, `card_tier`, `deepest_reached`,
`available_pool` or prices), and `docs/economy.md`. `app.rs` is **not**
touched: `ShopOutcome::Buy(Card)` keeps its shape and the Buy arm
(`app.rs:1363-1381`) already does everything spec goal 4 asks. No engine,
profile, save or dependency change.

## What the code already gives us

- **The tier is already one function of the card.** `economy::card_tier`
  partitions `ALL_SIDE_CARDS` into Outer 7 / Mid 6 / Core 2
  (`card_tier_partitions_the_universe`), and `available_pool` is exactly
  `ALL_SIDE_CARDS` filtered by `card_tier(c) <= deepest_reached(run)`. The shop
  grouping reads the same two functions, so the list and the buyable pool
  cannot disagree (acceptance criterion 7).
- **`RegionTier` is `Ord`** (`Outer < Mid < Core`), and `region_tier(&str)`
  maps the map's region strings (`"Outer Rim"`, `"Mid Rim"`, `"Core"`) to it;
  `every_planet_region_maps_to_a_known_tier` pins that every planet uses one of
  those three strings.
- **Buying is already decided elsewhere.** `Profile::can_afford` drives the
  dimming and `Profile::try_purchase` the refusal; the app plays
  `MenuSelect`/`MenuBack` and saves on success. None of it knows about the list
  order, so reordering the list changes no buying behavior.
- **The shop is rebuilt on every visit** (`ShopState::new()` at `app.rs:701`),
  and depth cannot change while the shop is open, so the cursor never needs to
  survive a change in the unlocked count.
- **`frame::draw_text(frame, x, y, text, emphasis)`** draws left-aligned;
  `draw_text_centered` is what the rows use today.
- **`Config::min_size()`** returns `(139, 31)` (pinned in `config.rs`). The
  current fit test still measures 89 columns — a stale figure this spec's fit
  test replaces.

## Design tensions resolved

### 1. Unlocked cards are always a prefix of the list — so the cursor stays a plain index

Groups are drawn in tier order and a group is unlocked iff `tier <=
deepest_reached`, so the unlocked cards are always the **first `n`** cards of
the grouped listing (7, 13 or 15). The cursor therefore stays what it is today —
a `usize` wrapping over `0..n` — and the drawn row `i` is cursored iff `i ==
cursor` (with `i` counting cards only, not headings or blanks). No skip logic,
no per-row lookup: B2 falls out of the ordering. The claim is pinned by a test
that the prefix equals `available_pool` as a multiset and every card after it is
deeper than `deepest_reached`, at all three depths (§Tests).

Rejected: keeping `available_pool`'s order for the cursor. It is
`ALL_SIDE_CARDS` order (`+4` between `+3` and `−1`), which is not the grouped
display order, so cursor index and drawn row would diverge.

### 2. The list is one block, centered as a block

Headings are wider than rows (`Mid Rim  ·  reach the Mid Rim to unlock` is 39
columns; a row is 28). Centering each line on its own would put headings and
rows on different left edges and make the list ragged. Instead the list
(headings and rows) shares one left column, `list_left(center_x)`, computed from
the widest line the list can ever draw; headings start 3 columns in, aligned
with the card labels, as the approved sketch shows. Title, balance and hint stay
centered as today.

A side effect: a row no longer re-centers when its owned count gains a digit
(`×9` → `×10`), so buying can't nudge a row by a column. The cursor marker stays
fixed-width, so moving the cursor changes no row's length (spec: no jitter).

### 3. Heading emphasis

The spec dims locked *rows*; it says nothing about heading emphasis. Headings
are drawn **Normal** whether locked or not, so the lock sentence — the thing that
tells locked from unaffordable (goal 2) — reads at full weight, while the rows
under it recede. No new emphasis level. To be eyeballed at the driver run; the
alternative (locked headings Muted too) is a one-word change if it reads wrong.

### 4. No defensive tier check on Buy

`try_purchase` does not check tier, and `handle_input` can only emit
`Buy(listing()[cursor])` with `cursor < n`. The guarantee is the prefix property
(§1) plus the cursor clamp, both tested in `shop.rs`. Adding a tier check in
`app.rs` would be a second rule for a state the screen can't produce — cut per
*Simplicity*.

### 5. The density rule was checked — no conflict

The constitution's *acted-on element stands apart* rule gives the acted-on line
an empty row above and below. The spec keeps the list compact with one empty row
above each heading, matching today's shop list, and padding a moving cursor row
would shift every row below it on each keypress — breaking the spec's no-jitter
requirement. So the list's only air is the heading gaps; no conflict with
`CLAUDE.md`.

## Design

### 1. `src/economy.rs` — the region's name

```rust
impl RegionTier {
    /// The map's name for this region — the inverse of [`region_tier`]
    /// (spec 025: the Outfitter's group headings use the map's words).
    pub fn region_name(self) -> &'static str {
        match self {
            RegionTier::Outer => "Outer Rim",
            RegionTier::Mid => "Mid Rim",
            RegionTier::Core => "Core",
        }
    }
}
```

Test `region_name_is_the_inverse_of_region_tier`: for each of the three tiers,
`region_tier(t.region_name()) == t`. With the existing planet-region test this
ties the headings to the map's labels. Nothing else in the file changes.

### 2. `src/shop.rs` — listing, cursor, draw

Module doc: "spend credits on cards from the current depth-gated pool" becomes
"browse the whole card range, grouped by the region that opens it, and buy from
the groups you've reached" (cite spec 025).

Pure helpers (private, non-test — the draw and input both use them):

```rust
/// The three groups, in list order.
const TIERS: [RegionTier; 3] = [RegionTier::Outer, RegionTier::Mid, RegionTier::Core];

/// Every collectible card in list order: grouped by `economy::card_tier`
/// (Outer, Mid, Core), `ALL_SIDE_CARDS` order within a group.
fn listing() -> Vec<Card>

/// How many cards at the head of `listing()` are buyable at `depth`. The
/// unlocked cards are always a prefix, because groups are in tier order
/// (plan tension §1).
fn unlocked_count(depth: RegionTier) -> usize   // listing().filter(card_tier(c) <= depth).count()

/// A group's heading: the bare region name once reached, else
/// `"<Region>  ·  reach the <Region> to unlock"`.
fn heading(tier: RegionTier, depth: RegionTier) -> String

/// One card row — today's format, extracted so the fit test measures the
/// string the draw uses: `"{marker}  {label:<4}   {price:>3} cr   owned ×{owned}"`.
fn row_text(card: Card, cursored: bool, owned: usize) -> String

/// Rows the list occupies: each group is a blank row, its heading, its cards.
const LIST_ROWS: usize = ALL_SIDE_CARDS.len() + 2 * TIERS.len(); // 21

/// `(title_y, list_top, hint_y)`: title, balance, the list (which opens with
/// its first blank row), one blank, the hint — 25 rows, centered vertically.
fn anchors(num_rows: usize) -> (usize, usize, usize)
    // block = 2 + LIST_ROWS + 2; top = num_rows.saturating_sub(block) / 2;
    // list_top = top + 2; hint_y = list_top + LIST_ROWS + 1

/// The list's shared left column (plan tension §2): centered on `center_x` by
/// the widest line the list can draw — the locked heading of each tier that
/// can lock (Mid, Core; Outer is always reached, so its locked form is never
/// drawn and is excluded) at its 3-column indent, and every row with a
/// two-digit owned count.
fn list_left(center_x: usize) -> usize
```

At 31 rows: title 3, balance 4, Outer blank/heading 5/6, cards 7–13, Mid
blank/heading 14/15, cards 16–21, Core blank/heading 22/23, cards 24–25, blank
26, hint 27.

`ShopState { cursor: usize }` — the comment becomes "index into `listing()`,
always `< unlocked_count`". `new()` stays `cursor: 0`, which is `+1`, the first
Outer Rim card.

`handle_input(key, profile)`:
`let n = unlocked_count(economy::deepest_reached(profile.campaign()));` then the
existing body with `pool[self.cursor]` → `listing()[self.cursor]`. The `n == 0`
guard and the `self.cursor.min(n - 1)` clamp stay. Doc: "Up/Down move over the
unlocked cards only, wrapping; locked rows and headings are never selectable".

`draw(frame, config, profile, pulse)`: title and balance unchanged. Then
`depth = deepest_reached`, `left = list_left(center_x)`, `y = list_top`, card
index `i = 0`; for each tier in `TIERS`: skip one row (`y += 1`), draw
`heading(tier, depth)` at `(left + 3, y)` in `Emphasis::Normal`, `y += 1`; for
each card of that tier in `listing()` order: `locked = card_tier(card) > depth`,
`cursored = !locked && i == self.cursor`, emphasis `pulse` if cursored, else
`Muted` if `locked || !profile.can_afford(price)`, else `Normal`; draw
`row_text(card, cursored, profile.owned_count(card))` at `(left, y)`; `y += 1`,
`i += 1`. Hint at `hint_y`, unchanged. The draw may iterate `listing()` once and
emit a heading whenever the tier changes — either shape is fine as long as it
reads `listing()` and `heading()`.

### 3. `docs/economy.md` — § The shop and its reserve

Replace "It lists the currently-available pool — each card with its price and
how many you own —" with the new list: all 15 cards grouped Outer Rim / Mid Rim /
Core; unreached groups dimmed under `<Region>  ·  reach the <Region> to unlock`
with prices and owned counts showing; the cursor visits unlocked cards only
(spec 025), so a locked card can't be bought. Add one sentence that the grouping
reads `card_tier` / `deepest_reached`, the same functions `available_pool` does.
In § The depth-gated pool, "Which cards you can buy is gated…" stays true; add
"the Outfitter shows the locked tiers too" only if the paragraph otherwise reads
as if they were hidden.

§ Tuning & guards (~line 260): its guard inventory names
`the_full_pool_fits_the_minimum_terminal` (`shop.rs`), which T001 renames —
change it to `the_full_list_fits_the_minimum_terminal`, and add the new shop
guards (`the_listing_groups_every_card_by_tier`,
`the_unlocked_prefix_is_the_available_pool_at_every_depth`,
`headings_name_the_region_and_lock_until_reached`,
`arrows_wrap_over_the_unlocked_cards_only`,
`a_reset_map_relocks_groups_but_keeps_owned_counts`) and
`region_name_is_the_inverse_of_region_tier` (`economy.rs`).

## Files

- `src/shop.rs` — helpers, cursor over the unlocked prefix, grouped draw; tests.
- `src/economy.rs` — `RegionTier::region_name`; one test.
- `docs/economy.md` — the shop section and the Tuning & guards test inventory.
- `specs/025-outfitter-locked-cards/closeout-main-docs.md` (T003).
- **No change**: `app.rs`, `card.rs`, `game.rs`, `player.rs`, `save.rs`,
  `profile.rs`, `campaign.rs`, `campaign_map.rs`, `frame.rs`, `Readme.md` (its
  one shop sentence stays true), `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`.

## Tests

Each claim names the task that owns its check. All in T001 unless noted.

- **The region name inverts `region_tier`** —
  `region_name_is_the_inverse_of_region_tier` (economy.rs).
- **The listing is all 15 cards, grouped, in today's order within a group**
  (AC 1) — `the_listing_groups_every_card_by_tier`: `listing().len() == 15`;
  tiers are non-decreasing along it; group sizes 7/6/2; the Outer group is
  exactly `+1 +2 +3 −1 −2 −3 ±1`, the Core group `±6 ±1T`; `listing()` is a
  permutation of `ALL_SIDE_CARDS`.
- **The unlocked prefix is exactly the buyable pool** (AC 3, AC 7, tension §1)
  — `the_unlocked_prefix_is_the_available_pool_at_every_depth`: for a fresh run,
  a Mid run (`cinder/greeb`, `scree/dax` beaten) and a Core run, `n =
  unlocked_count(deepest_reached(run))` is 7 / 13 / 15, `listing()[..n]` sorted
  equals `available_pool(run)` sorted (compare by `format!("{c:?}")` or by
  containment + length — `Card` needn't gain `Ord`), and every card in
  `listing()[n..]` has `card_tier(c) > deepest_reached(run)`.
- **Headings** (AC 2, AC 3) — `headings_name_the_region_and_lock_until_reached`:
  at `Outer` depth, `"Outer Rim"`, `"Mid Rim  ·  reach the Mid Rim to unlock"`,
  `"Core  ·  reach the Core to unlock"`; at `Mid`, Mid bare and Core locked; at
  `Core`, all three bare.
- **The cursor wraps over unlocked cards only** (AC 4) — rewrite
  `arrows_move_over_the_pool_and_wrap` as
  `arrows_wrap_over_the_unlocked_cards_only`: on a fresh profile the cursor
  opens at 0 (`listing()[0] == Card::Plus(1)`); Up → 6; Down → 0; `s` → 1; and
  over 20 Down presses the cursor stays `< 7`. On a Core profile Up from 0 → 14
  (unchanged behavior when all are unlocked).
- **No key buys a locked card** (AC 4) — rewrite
  `enter_and_space_buy_the_highlighted_card`: on a fresh profile, Up then Enter
  → `Buy(Card::PlusMinus(1))` (the last Outer card — the wrap did not reach
  `+4`); then Down pressed 7 times with Enter after each, every `Buy(c)` has
  `card_tier(c) == RegionTier::Outer`, and the 7th Down lands back on
  `Buy(Card::Plus(1))` (the wrap is exactly 7). On a Core profile Down then Enter/Space →
  `Buy(listing()[1])`.
- **Esc/`x` and unknown keys** — existing test, unchanged.
- **A reset map relocks with owned counts intact** (AC 6) —
  `a_reset_map_relocks_groups_but_keeps_owned_counts`: Core profile,
  `grant_card(Card::Plus(4))` and `grant_card(Card::Tiebreaker)`,
  `reset_campaign_run()` → `unlocked_count(deepest_reached(..)) == 7`,
  `owned_count` of both is 1, and the Mid and Core headings are the locked form.
- **It fits 139×31** (AC 8) — rewrite `the_full_pool_fits_the_minimum_terminal`
  as `the_full_list_fits_the_minimum_terminal`, taking `(cols, rows)` from
  `Config::min_size()`: `anchors(rows)` gives `list_top > title_y + 1` and
  `hint_y < rows`; `list_top + LIST_ROWS < hint_y`; for `left =
  list_left(cols / 2)`, every `row_text(card, true, 99)` satisfies `left + len
  <= cols` and the locked heading of Mid and Core (the tiers that can lock;
  Outer excluded) satisfies `left + 3 + len <= cols`; the
  balance row at 99 999 fits centered. Also assert `LIST_ROWS == 21` so a new
  card type (roadmap) trips the test and re-checks the fit.
- **Buying is unchanged** (AC 5) — structural: `git diff main -- src/app.rs
  src/profile.rs` is empty (T003), so affordability, reserve, price, sound and
  save run through the untouched Buy arm and the existing `profile.rs`
  purchase tests; plus a buy and a refused buy at the driver run (T002).
- **Locked vs unaffordable reads right, the pulse, no jitter, the fit on
  screen** (AC 2, AC 8, design requirements) — *driver*, T002.
- **No forbidden change** (AC 9) — T003: `git diff main --stat` lists none of
  `card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`; `git diff main --
  src/economy.rs` adds only `region_name` and its test.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim.
- **Driver walkthrough after T002** (the orchestrator, via the `run-kaazap`
  skill, at 139×31; the real profile, settings and save backed up and
  checksum-restored afterwards): fresh profile → Outfitter shows three groups,
  Mid and Core dimmed under their lock headings, cursor on `+1`; ↑ wraps to `±1`,
  ↓ from `±1` wraps to `+1`, never landing below the Outer group; buy an
  affordable card (count and balance update) and try one past the reserve
  (refused, dimmed); a profile with the Mid Rim reached → Mid group bare and
  navigable, Core still locked; a Core profile → all bare; a completed profile
  that owns Mid/Core cards after New Campaign → both groups locked again with
  their owned counts. Check headings read Normal against dimmed rows (tension
  §3) and nothing clips at 139×31. On the Core profile (every region reached,
  so no long heading is drawn but `list_left` still reserves for one), eyeball
  whether the 28-column rows sit visibly left of the centered title (tension
  §2) — report it; it is a layout finding, not a blocker by itself.

## Non-goals (from spec)

No new card types; no change to tiers, prices or when a region opens; no unlock
regions in the deck builder's album; no card descriptions or detail panel; no
map, primer or How to Play text change.

## Open questions

None product-level. Settled here as design and flagged for sign-off:

1. **Headings are Normal, locked or not** (tension §3) — the spec is silent on
   heading emphasis.
2. **The list is left-aligned on a shared, block-centered column** (tension §2)
   rather than each row centered on its own as today — follows the approved
   sketch; rows no longer shift by a column when an owned count gains a digit.
