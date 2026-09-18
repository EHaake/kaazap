# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 025)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T003, ready to paste on `main` after the merge
(the same close-out shape specs 020–024 used). Nothing here is applied by the
spec branch.

The `CLAUDE.md` amendment landed on `main` before the merge, in its own commit
as the constitution requires (`3777553`, plus `5b6197a` carrying the fallback
model into *Tiers by name*); nothing further to apply here — see §3. Spec 025
changed no *product* rule the constitution states.

Line numbers below are **`main`'s** at the time of drafting (2026-09-17,
`main` at `6e5db0d`), which is *ahead* of the spec branch — it already carries
two `ROADMAP.md` entries added during this spec (**More side-card types** and
the **wager-warning chore**), neither of which is touched here, plus the two
pre-merge sweep commits: `3777553` (the `CLAUDE.md` model-policy amendment, see
§3) and `6e5db0d` (the *More side-card types* entry corrected to 21 rows). Each
edit also quotes its anchor text verbatim, which is what to match on; every
anchor below was re-checked programmatically against `git show main:<file>` at
`6e5db0d` and is unique there.

---

## 1. `ROADMAP.md` — four edits

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Endgame, victory & what you keep (spec 024)**
entry, which ends with these two lines (currently `ROADMAP.md` lines 384–385,
just before the blank line and `## Backlog`):

```markdown
  `Readme.md`, `docs/economy.md` and `docs/balance.md` re-synced (the last with
  a short *Replays* note: a replay starts premium, deliberately not retuned).
```

Insert after them:

```markdown
- **Locked cards in the Outfitter** (spec 025) — the depth gate is now
  **visible**. The Outfitter lists all **15 cards** on every visit, in three
  region groups — Outer Rim 7, Mid Rim 6, Core 2 — each row carrying its price
  and owned count exactly as before. A group whose region the run hasn't
  reached is **locked**: its rows are dimmed *with their prices still showing*
  (ruling A2, revised from A3 the same day, so a player can see what they're
  saving toward), its heading reads `Mid Rim  ·  reach the Mid Rim to unlock`
  in the map's own words (C1), and the cursor **passes over it** (B2), so no
  key can buy a locked card. Because the groups are drawn in tier order and a
  group is unlocked iff `tier <= deepest_reached`, the unlocked cards are
  always the **first `n`** of the listing (7, 13 or 15) — so the cursor stayed
  what it was, a `usize` wrapping over `0..n`, with no skip logic and no
  per-row lookup; a test pins that prefix against `available_pool` as a
  multiset at all three depths. The grouping reads `economy::card_tier` and
  `economy::deepest_reached`, the same pair `available_pool` uses, so the list
  and what's buyable **cannot disagree**. The list is centered **as one block**
  (`list_left`, off the widest line the list can ever draw, headings three
  columns in), so a row no longer re-centers when an owned count gains a digit
  and the cursor never makes the list jitter. Buying is untouched —
  affordability, the ante reserve, prices, the bought/refused sounds and the
  save all behave as specs 012, 021 and 022 left them — and after **New
  Campaign** (spec 024) the Mid and Core groups lock again with their owned
  counts intact. `economy.rs` gained exactly one function,
  `RegionTier::region_name` (the inverse of `region_tier`, with a test);
  everything else is `shop.rs` (`TIERS`, `listing`, `unlocked_count`,
  `heading`, `row_text`, `LIST_ROWS`, `list_left`, and `anchors(num_rows)` in
  place of `anchors(num_rows, n)`). No engine, AI, economy, balance-data or
  save-format change: `card.rs`, `game.rs`, `player.rs`, `save.rs`,
  `profile.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are
  untouched, the tier table and prices stand as spec 022 left them, and
  `PROFILE_VERSION` / `SAVE_VERSION` stay 1. The full list, headings, balance
  and hint fit 139×31, pinned by a test and by a driver walkthrough at all
  three depths and across a New Campaign. `docs/economy.md` re-synced.
```

### 1b. Inline annotation on the spec 012 Shipped entry (currently lines 152–154)

The **Economy & progression (spec 012)** entry describes the shop as showing
that depth-gated pool, in the present tense. Using the repo's inline-supersession
convention, replace:

```markdown
  progressively unlock more of the 15-card universe. A **shop** on the campaign
  map (the Outfitter, `b`) spends credits on that same pool, showing prices, owned
  counts, and a live balance (also in the map header). Everything grows the
```

with:

```markdown
  progressively unlock more of the 15-card universe. A **shop** on the campaign
  map (the Outfitter, `b`) spends credits on that same pool, showing prices, owned
  counts, and a live balance (also in the map header), and listing only that
  unlocked pool — **superseded by spec 025**, which lists all 15 cards grouped
  by region with the un-reached groups dimmed and locked; what the shop sells,
  and what it costs, is unchanged. Everything grows the
```

### 1c. Inline annotation on the backlog's shipped-marked *C · Economy & progression* bullet (currently lines 415–418)

Replace (the anchor ends mid-sentence, at the end of line 418 — keep the
trailing "Scarcity is the depth" so the match lands on a line boundary):

```markdown
- **C · Economy & progression** — ✅ **Shipped (spec 012** — see Shipped above
  and `docs/economy.md`). Credits from wins; a card pool that **unlocks by
  campaign depth**; a **shop** selling from the unlocked pool; and a **random
  card drop from each win** pulled from that same pool. Scarcity is the depth
```

with:

```markdown
- **C · Economy & progression** — ✅ **Shipped (spec 012** — see Shipped above
  and `docs/economy.md`). Credits from wins; a card pool that **unlocks by
  campaign depth**; a **shop** selling from the unlocked pool (and, until
  **spec 025**, listing only that pool — spec 025 supersedes the listing half:
  all 15 cards, grouped by region, the un-reached groups locked and skipped by
  the cursor, while what the shop *sells* is unchanged); and a **random
  card drop from each win** pulled from that same pool. Scarcity is the depth
```

### 1d. Mark the backlog entry this spec shipped (currently lines 593–599)

Under "### Onboarding, endgame & release readiness (suggested 2026-09-13, after
spec 022)", replace the whole **Show locked cards in the Outfitter** bullet:

```markdown
- **Show locked cards in the Outfitter** (raised by the person 2026-09-16,
  playing early in a campaign: the shop offered only +1 to +3, −1 to −3 and
  ±1, and nothing said the rest exist). The shop hides every card above the
  deepest region reached, so the depth gate reads as missing cards. Taken
  ahead of the compact layout as spec 025: the Outfitter lists the whole
  15-card universe grouped by region, the locked rows visibly locked and
  saying where they open. No economy, price or pool change.
```

with:

```markdown
- **Show locked cards in the Outfitter** — ✅ **Shipped (spec 025** — see
  Shipped above and `docs/economy.md`). Raised by the person 2026-09-16,
  playing early in a campaign: the shop offered only +1 to +3, −1 to −3 and
  ±1, and nothing said the rest exist — it hid every card above the deepest
  region reached, so the depth gate read as missing cards. Taken ahead of the
  compact layout: the Outfitter now lists the whole 15-card universe grouped by
  region, the locked rows dimmed with their prices showing under a heading that
  names the region that opens them, and the cursor passing over them. No
  economy, price or pool change; **new card types** stayed their own backlog
  item (below).
```

**Leave alone** the two newer backlog entries added on `main` during this spec:
**More side-card types** (currently line 645, under *Other*) — spec 025's
non-goal, deliberately still open, and it already refers to the Outfitter's
full list — and the **wager-warning chore** (currently line 571).

---

## 2. `DECISIONS.md` — two edits

### 2a. The spec 012 scarcity bullet is clarified, not superseded (currently lines 248–251)

`grep -n "available pool\|shop" DECISIONS.md` on `main` returns ten hits
(lines 82, 113, 245, 251, 346, 706, 713, 766, 802, 1004). Read and judged:
**nine of them are about what the shop *sells* or *reserves*, not what it
lists**, and all nine stay true — spec 025 changed display only. (Line 82's
"credits **buy** from that available pool in a **shop**", line 245's "the
campaign depth that gates its stock", line 346's "acquiring cards stays the
shop's job", spec 021's reserve bullets at 706/713/766, spec 022's Quick Play
bullet at 802 and spec 023's `open_campaign_map` note at 1004 all need no
edit.) The one worth annotating is spec 012's scarcity ruling, because it is
the ruling the new list could be read as contradicting. Replace:

```markdown
- **Scarcity is distribution, not new card types** (pre-ruled, reaffirmed) — the
  15-card universe is complete, so progression gates *acquiring* the existing cards
  by campaign depth (three region tiers, Outer ⊆ Mid ⊆ Core). Both the win-drop
  and the shop draw from that one growing pool.
```

with:

```markdown
- **Scarcity is distribution, not new card types** (pre-ruled, reaffirmed) — the
  15-card universe is complete, so progression gates *acquiring* the existing cards
  by campaign depth (three region tiers, Outer ⊆ Mid ⊆ Core). Both the win-drop
  and the shop draw from that one growing pool.
  **Clarified — not superseded — by spec 025**: the shop still *draws* from
  that one growing pool, so what you can buy and what it costs are unchanged,
  but it now *lists* all 15 cards with the un-reached groups dimmed and locked.
  The gate reads the same `card_tier` / `deepest_reached` pair `available_pool`
  does, so the list cannot drift from the pool. See *Locked cards in the
  Outfitter (spec 025)* below.
```

### 2b. Append a new section at the end of the file

Append at the **end of the file** — `DECISIONS.md`'s last two lines are
currently the tail of the "## Endgame, victory & what you keep (spec 024)"
section:

```
both stay 1, so a pre-024 profile loads with zero counters and no record.
Monochrome by construction.
```

(`Monochrome by construction.` alone is *not* unique in the file; end-of-file
is the anchor.) Append after them:

```markdown
## Locked cards in the Outfitter (spec 025)

A new player saw seven cards in the Outfitter and nothing saying the other
eight exist, so the depth gate read as missing cards rather than as a promise.
The person hit it in play on 2026-09-16 and ruled the same day. The spec
changes what the shop **shows** and nothing about what it sells, what that
costs, or when a region opens.

- **A2 — locked rows show their price, dimmed.** First ruled **A3** (a
  `locked` word in place of the price), reversed the same day so a player can
  see what they are saving toward. The cost, accepted on record: in a
  monochrome UI a locked row and an unaffordable row look alike, so the
  difference is carried by the group heading and by the cursor (which stops on
  an unaffordable card but never on a locked one).
- **B2 — the cursor skips locked rows.** They are there to be seen, not
  selected, so Enter on a locked card cannot happen — which is what lets the
  buy path stay exactly as it was.
- **C1 — the heading names the region** (`Mid Rim  ·  reach the Mid Rim to
  unlock`), in the map's own labels, rather than naming the planet that opens
  the group. One vocabulary for depth across the map and the shop.
- **Taken ahead of the compact layout**, the other queued shop-adjacent item,
  and **new card types (+5, +6, −5, −6, ±4, ±5) were logged as their own
  roadmap item** rather than folded in: they would move `card.rs`, the tier
  table, the prices and the spec 022 curve, and this spec moves none of those.

Design tensions resolved during planning:

- **Unlocked cards are always a prefix of the list — so the cursor stays a
  plain index.** Groups are drawn in tier order and a group is unlocked iff
  `tier <= deepest_reached`, so the unlocked cards are always the **first `n`**
  cards of the grouped listing (7, 13 or 15). The cursor therefore stayed a
  `usize` wrapping over `0..n`, and a drawn row `i` is cursored iff `i ==
  cursor` (counting cards only). **B2 falls out of the ordering** — no skip
  logic, no per-row lookup. The claim is pinned by a test that the prefix
  equals `available_pool` as a multiset and that every card after it is deeper
  than `deepest_reached`, at all three depths. Rejected: keeping
  `available_pool`'s order for the cursor — it is `ALL_SIDE_CARDS` order (`+4`
  between `+3` and `−1`), not the grouped display order, so the cursor index
  and the drawn row would diverge.
- **The list is one block, centered as a block.** Headings are wider than rows
  (`Mid Rim  ·  reach the Mid Rim to unlock` is 39 columns; a row is 28), so
  centering each line on its own would put headings and rows on different left
  edges and leave the list ragged. Instead the whole list shares one left
  column, `list_left(center_x)`, computed from the widest line it can ever
  draw, with headings three columns in, aligned with the card labels. Title,
  balance and hint stay centered as before. Two consequences, both accepted:
  a row no longer re-centers when its owned count gains a digit (`×9` → `×10`),
  so buying can't nudge a row sideways; and because the block is sized for the
  *widest possible* line (the locked Mid Rim heading at its indent, 42 columns),
  the rows themselves sit left of centre — at 139 columns `list_left` is 48, a
  row is 27–28 columns wide, so the row block centres on column 62 against a
  title centred on 69: **about seven columns left**. It is most visible on a
  profile that has unlocked everything, where no long locked heading is on
  screen to fill the width. Measured at the driver walkthrough and reported at
  the phase pause, where the person attested the screen looked good; left as is.
- **Headings are drawn Normal, locked or not.** The spec dims locked *rows* and
  says nothing about heading emphasis. Drawing every heading Normal keeps the
  lock sentence — the thing that tells locked from unaffordable — at full
  weight while the rows under it recede, and adds no new emphasis level.
- **No defensive tier check on Buy.** `try_purchase` does not look at tier, and
  `handle_input` can only emit `Buy(listing()[cursor])` with `cursor < n`. The
  guarantee is the prefix property plus the cursor clamp, both tested in
  `shop.rs`; a tier check in `app.rs` would be a second rule for a state the
  screen cannot produce — cut per the constitution's *Simplicity*.
- **The density rule was checked — no conflict.** The *acted-on element stands
  apart* rule gives the acted-on line an empty row above and below. The list
  stays compact with one empty row above each heading, matching today's shop
  list: padding a moving cursor row would shift every row below it on each
  keypress, which is exactly the jitter the spec forbids. So the list's only
  air is the heading gaps.

No engine change (`card.rs`, `game.rs`, `player.rs`, `save.rs` untouched), no
AI change, no balance data moved (`economy.rs`'s tier table and prices are as
spec 022 left them; the file gained only `RegionTier::region_name`, the inverse
of `region_tier`, plus its test), no profile-format change,
`tests/balance.rs` / `Cargo.toml` / `Cargo.lock` untouched, and no new crate.
`PROFILE_VERSION` and `SAVE_VERSION` both stay 1. Monochrome by construction:
Normal, Muted and the existing cursor pulse, no new emphasis level.
```

---

## 3. Not drafted here (deliberately)

- **`docs/economy.md` is a spec file on the branch** — its shop section now
  describes the grouped, always-full list (T002), and the affordability-dimming
  sentence was qualified in T003 so it no longer reads as the only reason a row
  is dim. It rides into `main` with the merge and needs no close-out edit.
- **`Readme.md`, `docs/balance.md` and `docs/opponents.md` are untouched** —
  none of them describes the Outfitter's listing, and no number moved.
- **The two `ROADMAP.md` entries added on `main` during this spec** — *More
  side-card types* and the *wager-warning chore* — are left exactly as they
  are. The first is this spec's explicit non-goal and is still open (and was
  corrected on `main` in commit `6e5db0d` — the Outfitter's list is **21 rows**,
  15 cards plus a blank and a heading per group, not 15); the second is an
  unrelated chore.
- **The `CLAUDE.md` amendment already landed on `main`** (commit `3777553`,
  before the merge, in its own commit as the constitution requires): the
  Fallback clause now points at the session-fallback model the person names
  (`claude-opus-5` since 2026-09-16) and the History paragraph records that
  spec 025 ran entirely on it. That is a **model-policy** amendment, not a
  product rule — spec 025 changed no rule the constitution states about the
  game: the Outfitter is still a `Screen` following the `opponent_select`
  shape, drawing never mutates state, the *acted-on element stands apart* rule
  was checked and found not to apply to a scrollless compact list (recorded
  above), and the verification command is unchanged. **Nothing further to apply
  here.**
- **Tier-log observations stay in `specs/025-outfitter-locked-cards/tasks.md`**
  (the sign-off notes N1–N5, the Phase 1 review notes N1–N6, the driver
  walkthrough's layout measurement): process evidence for the model-policy
  experiments, not project decisions.
