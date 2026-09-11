# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 021)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T007, ready to paste on `main` after the merge
(the same close-out shape spec 020 used). Nothing here is applied by the spec
branch.

---

## 0. `CLAUDE.md` — one-sentence amendment — **APPLIED on main** (commit 7081514, 2026-09-11; wording tidied in a follow-up commit)

`CLAUDE.md` is repo-wide too, so this rides to `main` with the rest of the
close-out rather than on the spec branch. In the **What this project is**
paragraph, the sentence describing the reward loop is false after spec 021 (the
free win credit and the card drop are gone, matches are staked, a loss costs the
stake, and going broke resets the run).

Replace this sentence (currently `CLAUDE.md` line 17):

```markdown
Wins earn currency and/or card packs that unlock better side-deck cards
for future matches.
```

with:

```markdown
Every campaign match is played for a stake the player chooses: a win pays
it back double, a loss keeps it, and going broke ends the run and resets
it. Credits buy better side-deck cards in the shop.
```

The rest of the paragraph (the first sentence and the closing
"personal project… portfolio, itch.io" sentence) stays exactly as is.

---

## 1. `ROADMAP.md` — three edits

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Stats & records (spec 020)** entry (currently
ends at the line "…No engine/save-format change.", just before `## Backlog`):

```markdown
- **Wager & loss condition** (spec 021) — the campaign economy turned
  two-directional. Every campaign match is played for a **stake** chosen in a
  wager prompt over the map (opens at a per-opponent **ante floor** of
  `(threshold − 14) × 10`, walks in steps of 5 up to the full balance, no cap);
  the stake is **escrowed at launch**, a win pays it back **double**, a loss
  keeps it. Spec 012's free card drop is gone — cards come only from the shop —
  and a fresh or reset profile now starts with a **seed purse of 50**. Cleared
  planets stay launchable as **rematches** against their final opponent (the
  grinding lever the balance pass tunes against), which change no progress and
  can't re-count a campaign completion. Dropping below the cheapest launchable
  ante ends the run: a modal notice on the map, acknowledged into spec 014's
  `reset_to_starter` (starter deck, no progress, seed purse; settings and
  lifetime records survive). The shop holds that cheapest ante back as a
  **reserve**, so shopping can never end a run. Persisted as an additive
  `#[serde(default)] stake` on `NodeRef` beside the in-flight pointer — **no
  `PROFILE_VERSION` bump, no `SAVE_VERSION` bump**, and no engine change
  (`game.rs` / `player.rs` / `card.rs` / `save.rs` untouched). Every number is a
  tunable constant in `economy.rs`; see `docs/economy.md`.
```

### 1b. Replace the **Wager & loss condition** bullet under "Stakes, loss condition & difficulty balance"

Replace the whole bullet (currently `ROADMAP.md` lines ~364–377, beginning
"- **Wager & loss condition** — make the economy two-directional") with:

```markdown
- **Wager & loss condition** — ✅ **Shipped (spec 021** — see Shipped above and
  `docs/economy.md`). The economy is two-directional: a player-chosen stake per
  campaign match (ante floor by opponent threshold, even money, no cap),
  escrowed at launch; going broke ends the run through spec 014's full reset
  (starter deck *and* seed purse, settings surviving). The free card drop was
  removed outright rather than made rare, so cards come only from the shop,
  bought with wagered credits; rematches on cleared planets were added in the
  same spec so safe grinding actually exists. The softer *keep-your-cards*
  restart remains a noted future difficulty-lever if full reset playtests too
  punishing. Still **deferred to a later spec/discussion (human-ruled):** what
  "beating the game" awards (the endgame/victory) and the
  casual-campaign-vs-roguelike-mode identity question (see **E · Roguelike
  mode**).
```

### 1c. Unblock the **Difficulty & economy balance pass** bullet

In the bullet that follows, replace the sentence:

```markdown
  Needs the wager loop
  live first.
```

with:

```markdown
  **Now unblocked** — the wager loop shipped (spec
  021), and every lever it owns (seed purse, ante floors, stake step, payout
  ratio, shop prices) is a tunable constant in `economy.rs`, with the rematch
  grind live for the invariant above to be tuned against.
```

(The section heading "### Stakes, loss condition & difficulty balance (now being
sequenced)" stays as is — the balance pass under it is still open.)

---

## 2. `DECISIONS.md` — append a new section at the end of the file

Append after the spec 020 section's closing paragraph ("No
`game.rs`/`player.rs`/`card.rs`/`save.rs` change; …"):

```markdown
## Wager & loss condition (spec 021)

The campaign economy becomes two-directional: every campaign match is played for
a stake, a loss costs it, and going broke ends the run. All of the following were
ruled with the human on 2026-09-10, on the recommendations as proposed.

- **Rematches on cleared planets, against the final opponent.** The balance
  pass's invariant ("safe minimum-wager grinding can't fund the next tier")
  presumes grinding exists — and it didn't: a cleared planet was a no-op. The
  final opponent is the simplest rule; a per-opponent picker can come later.
- **Seed purse of 50.** A fresh profile had 0 credits, which can't stake
  anything. 50 is one Outer-tier card or a few minimum antes.
- **The ante floor reuses the threshold scalar** — the same `(threshold − 14) × 10`
  the old win reward used, so difficulty keeps living in one number.
- **Player-chosen stake, floor to full balance, no cap.** "Bigger, riskier bets"
  has to be a real lever; a cap is a later balance knob, not a launch rule.
- **Even money, and the old win credit removed.** Difficulty now lives in the
  floor, so scaling the payout by threshold too would double-count it. A first
  clear earns progress only.
- **The free card drop is removed outright** rather than made rare. Cards come
  from the shop, bought with wagered credits; a rare drop is cheap to add back if
  playtest wants it. (This also removed the only randomness in `economy.rs` — the
  injected `roll` seam went with it.)
- **Escrow at launch.** The stake leaves the balance when the match starts, so
  the map header is honest mid-match and discarding a saved match can't dodge a
  loss.
- **Broke = can't cover the cheapest launchable floor, checked after a loss and
  at campaign entry; the shop reserves that floor.** The floor stays a real floor
  (no "last stand" below it), and shopping can never end a run. The entry-time
  check doubles as the migration path for a pre-021 profile with no credits: it
  meets the run-over flow rather than a dead map. Accepted for a personal project
  over a progress-preserving top-up.
- **Run-over is a modal on the map, reset via spec 014's path** — one reset
  operation, with lifetime records preserved per spec 020. There is no decline:
  the reset *is* the loss condition.
- **The stake is shown in-match; no "runs ended broke" counter.** The wager's
  tension belongs on screen (it rides in the existing presence panel, inside the
  139×31 minimum); a bust counter is a cheap additive field if the balance pass
  wants it.
- **A campaign completion counts once.** Spec 020's "a completed run exposes no
  launchable match" assumption no longer holds with rematches, so completion is
  tied explicitly to the win that completes the run.

Design tensions resolved during planning:

- **The stake lives on `NodeRef`, not on `CampaignRun` or the match save.**
  `NodeRef { planet, opponent, stake }` with `#[serde(default)] stake: u32`: the
  stake is *part of* the in-flight match context — it exists iff a campaign match
  is in flight, persists in `profile.json` beside the pointer, and is discarded
  with it. A `CampaignRun.stake` field would need its own clear on every path
  that clears the pointer; a `SavedGame` field would put economy data in the
  engine save and force a save-format change.
- **Settlement and completion move into one `Profile::settle_campaign_match` —
  superseding spec 020's `record_match` clause.** Spec 020 counted a completion
  inside `record_match` as `campaign && player_won && run_complete()`, relying on
  "a completed run exposes no launchable match"; rematches void that. The check
  moved into `settle_campaign_match`, which owns `mark_beaten` and detects
  completion as the **edge** `!was_complete && run_complete()` around it — a
  rematch leaves `beaten` unchanged so the edge never fires, and a first clear of
  the final node fires it exactly once. This also dissolves spec 020's "the
  record block must run after the win block" ordering requirement: the ordering
  is now internal to one method, and `record_match` does lifetime + run-tally
  recording only.
- **Exactly-once payout is a data property, not an ordering rule.**
  `settle_campaign_match` calls `CampaignRun::take_stake()` (returns the stake
  and zeroes the escrow) before paying, and `mark_beaten` is idempotent — so even
  a double-fired seam would pay `win_payout(0) == 0`. Consequence: the escrow
  reads 0 from the game-over tick on, so the in-match stake line disappears there
  while the outcome popup and the map banner carry the result. Quitting at the
  game-over screen without acknowledging leaves a stale `in_progress` with stake
  0 — harmless, and replaced by the next launch.
- **One resolution block on the `phase_changed` edge.** The spec-012 every-tick
  campaign-win block and the spec-020 edge-based record block became a single
  block in `App::tick`: `settle_campaign_match` → `record_match` → `save`. The
  `!is_opponent_beaten` once-guard is gone (rematches make it meaningless); the
  once-guarantee is the `phase_changed` edge plus the zeroed escrow above.
- **The broke check is a pure predicate run at two app seams.**
  `Profile::is_broke()` = `credits < economy::cheapest_floor(run)`, evaluated on
  the game-over acknowledgement of a campaign match and at campaign entry, both
  through one `App::enter_campaign_map()` helper. It is deliberately *not* run on
  every `open_campaign_map` (shop / deck-builder Back): the shop reserve makes
  those paths unable to create a broke state, and keeping the check at the two
  spec'd seams keeps the intent legible. A staked win can never leave the player
  broke (`credits ≥ win_payout(stake) ≥ 2 × floor`), so no "only after a loss"
  guard is needed; the one exception is a stake-0 pointer (a pre-021 campaign
  save on a 0-credit profile), where the accepted migration path simply arrives
  one match later.

No `game.rs`/`player.rs`/`card.rs`/`save.rs` change; `PROFILE_VERSION` and
`SAVE_VERSION` both stay 1 (the stake is an additive `#[serde(default)]` field).
Monochrome by construction.
```

---

## 3. Not drafted here (deliberately)

- The spec 020 section's bullet "**Campaign-completion detection via
  `run_complete()`, ordered after `mark_beaten`**" is left **as written**: the new
  section states plainly that it supersedes that clause, and `DECISIONS.md` reads
  as a dated log. Editing the older bullet in place is a separate call if the
  person prefers a pointer there.
