# Plan: Tournament rounds — spec 029

> **Status**: Signed off (skeptical-reviewer, 2026-09-21 — second amendment revision: one review, one re-review, B1 and B2 resolved and seven notes applied)
**Implements**: `spec.md` in this directory

The pre-amendment plan was signed off by the `skeptical-reviewer` on 2026-09-20
(one review, one re-review, B1–B6 resolved; two second-look notes carried to the
tier log and the pre-merge sweep), and **Phases 1 and 2 were implemented,
reviewed, committed and attested under it**. The **first** amendment revision
(R1 the series wording, R2 the Card Shop rename, R3 the art as horizontal bands
at every width) was signed off on 2026-09-21 — one review, one re-review, B1
resolved and five notes applied, three carried to the pre-merge sweep — and
shipped as T004a, T005a and T005b, which are implemented, reviewed, committed
and attested. Nothing about any of them is reopened here.

This **second** revision covers `spec.md`'s *Amendment, 2026-09-21 (second)* —
**R4** the art about 15–20 % smaller, **R5** every text row centred on the art
region rather than the terminal, **R6** the credit balance on the venue, and the
**per-planet art brief** as a document on this branch — and touches only what
that amendment falsifies: §Design tension 7 (one sentence), §Design 4, §Design 5,
§Files, the venue bullets in §Tests, §Verification (a fourth walkthrough entry)
and §Open questions 2, **which the person has now closed**. Everything Phase 1
and Phases 3–5 rest on is unchanged, including §Design tensions 1–6.
`tasks.md` gains **T005c** (R4/R5/R6) and **T005d** (the art brief) in Phase 2.

## Context

Today a campaign opponent is beaten by one match: `Profile::settle_campaign_match`
calls `mark_beaten` on any campaign win, and the planet clears from the beaten
set. This spec puts a **series** between the match and the opponent, a **venue**
between the map and the match, and a **lock** that makes the series the only
campaign match playable while it runs.

It adds one `Screen` (`Venue`), one persisted field on `CampaignRun`
(`Option<Series>`), one persisted bool on `NodeRef`, one layout struct, and one
new module (`src/venue.rs`). It changes no engine file (`game.rs`, `card.rs`,
`player.rs`), no save format version, no roster or economy constant, and adds no
crate. `SAVE_VERSION` and `PROFILE_VERSION` both stay 1: the two new persisted
fields are additive and `#[serde(default)]`, the pattern spec 021 used for
`NodeRef::stake`.

The three places this is most likely to go wrong, and where the design work
went:

1. **The return target.** `src/app.rs` currently sends Back from the shop and
   the deck builder to the campaign map. The venue is a second target. Spec 015
   shipped a bug here. §Design tension 1.
2. **The broke rule.** Spec 021's `is_broke` is one predicate over one floor.
   Ruling O1 changes what the floor *is* while locked, and must not become two
   floors that can disagree. §Design tension 2.
3. **Exactly-once settlement.** Paying a stake once is a *data* property today
   (`take_stake` zeroes the escrow, `mark_beaten` is idempotent), not an
   ordering rule. A series tally increment is idempotent under neither.
   §Design tension 3.

## What the code already gives us

- **Everything derived, nothing stored twice.** `campaign.rs` stores only
  `beaten` + `in_progress` + `run_stats`; cleared / unlocked / next opponent are
  computed from `PLANETS`. The series follows the same discipline: the tallies
  and the locked node are stored; *how many wins are needed* is derived from the
  opponent id, and *which screen the campaign opens on* is derived from whether
  a series exists.
- **A screen is a small, copyable shape.** `opponent_select.rs` is the
  constitution's reference: a state struct, `handle_input(key, …) -> Option<One
  Outcome>`, `draw(frame, config, …, pulse)`, wired into the three `match
  &self.screen` arms in `app.rs`. `campaign_map.rs` is the same shape with a
  full-screen layout struct and a `&Profile` argument. The venue copies both.
- **A right-hand preview rail already exists twice.** `opponent_select.rs`'s
  `preview_rect` and `CampaignMapLayout::portrait_panel` both place a
  `PANEL_W`-wide `draw_presence_panel` beside centered text. The venue's
  portrait is a third caller of the same drawer; only the art region beside it
  is new.
- **The wide/compact split is one comparison.** `cols >= WIDE_LAYOUT_MIN_WIDTH`
  (139), used by `BoardLayout::opponent_panel`. (**Revised 2026-09-21**: the
  venue's art and portrait used the same test under ruling N1; the amendment's
  R3 supersedes N1, so the venue makes that comparison nowhere — it derives both
  regions from the terminal's own size at every width. The board's use of the
  threshold is untouched.)
- **The wager prompt is a modal over whatever screen is current.** It is opened
  from `launch_campaign_node`, and cancelling it just sets `self.modal = None` —
  the screen underneath is untouched. Opening it over the venue therefore makes
  "declining returns to the venue" (AC 4) fall out with no new code.
- **The settlement is already one seam.** `tick` resolves a finished match on the
  single `phase_changed && GameOver` edge, through `Profile::resolve_match`,
  which records statistics and then settles. Nothing else settles.
- **`draw_notice` is screen-agnostic**, so `Modal::RunOver`, `Victory` and
  `Primer` draw over the venue without a new arm.
- **The density rule already has a shape**: `draw_choice_panel` draws its labels
  in **one row** with a constant-width `"{▸ or space} {label}"` per label and a
  blank row above and below that row (`choice_rows`, and the
  `a_choice_panel_keeps_a_blank_row_around_the_choice_row` test). The venue's
  four actions use the same shape, so the acted-on element is one row and the
  rest of the text stays compact, statically — no rows moving as the cursor
  moves.
- **Two tests pin text assets by exact line count**: `overlay.rs`'s
  `onboarding_texts_are_the_spec_text_and_fit` (Primer = 10 lines) and
  `help_texts_fit_the_minimum_terminal_unclamped`. Both will need their
  constants updated when the texts grow — deliberately, in the task that grows
  them (T009), not as a surprise at the build step. **How to Play has a hard
  ceiling of 27 lines** (`height + V_PAD == 31`); it is at 24, so this spec may
  add at most two lines to it.

## Design tensions resolved

### 1. The return target is **derived from the lock**, never remembered

Spec 015's bug was a *remembered* origin that one path forgot to set. Adding a
second remembered target (a `BuilderOrigin::Venue`, a `ShopOrigin`) would double
the number of places that can forget. So the venue introduces no origin at all:

`App::open_campaign_home` is the **only** place that assigns
`Screen::CampaignMap` or `Screen::Venue`, and it reads
`self.profile.campaign().series().is_some()` — three lines, an inline `if`, no
new type. (An earlier draft wrapped the choice in a `CampaignHome` enum plus a
`campaign_home(bool)` mapping in the `back_destination` spirit, for
testability. Dropped at sign-off: a unit test of a two-variant mapping over a
boolean is a tautology, and it would test the wrong thing — the real failure
mode is a *second* assignment site somewhere else in `app.rs`, which no mapping
test can catch and a grep can. See §Tests.)

Every door goes through it:

| Door | Before | After |
|---|---|---|
| Menu → Start Campaign → Continue | `enter_campaign_map(true)` | `enter_campaign(true)` |
| Campaign-entry confirm (discard save) | `enter_campaign_map(true)` | `enter_campaign(true)` |
| New Campaign / Reset Everything | `enter_campaign_map(true)` | `enter_campaign(true)` |
| Game-over acknowledgement | `enter_campaign_map(false)` | `enter_campaign(false)` |
| Back from the shop (the Card Shop after amendment R2) | `open_campaign_map()` | `open_campaign_home()` |
| Back from the deck builder (`BackTo::Map`) | `open_campaign_map()` | `open_campaign_home()` |
| Map → Enter on an un-beaten opponent | opened the wager prompt | writes the series, then `open_campaign_home()` |

Why this does not drift — **given one invariant that is reviewed, not enforced
by the type system**: the venue exists *exactly* when a series is in progress,
which is exactly what `open_campaign_home` reads, **and nothing else assigns a
campaign screen**. There is no second copy of the answer to keep in step, and no
path that can set the lock without also changing what the doors do — the map's
launch does not pick the venue, it writes the lock and asks the same function
everyone else asks. The deck-builder divert from an invalid deck gets the venue
return for free for the same reason, which a remembered origin would have needed
a third variant for. The invariant is checked mechanically at T006 and at the
Phase 2 review with
`grep -n "self\.screen = Screen::\(CampaignMap\|Venue\)" src/app.rs`, which must
return exactly two lines, both inside `open_campaign_home`.

`BuilderOrigin::Map` is renamed to `BuilderOrigin::Campaign` and `BackTo::Map`
to `BackTo::Campaign`, because "the map" is no longer where either of them
returns. Same for `map_entry_modal` → `campaign_entry_modal`. Renames rather
than comments: a name that asserts something false is the failure this design is
avoiding. (Three renames, ~8 lines including two test names.)

Rejected: a `Venue` variant on `BuilderOrigin` plus an origin field on
`ShopState` (two remembered origins, four set sites, the spec-015 shape); an
`App::campaign_return: Screen` field (state that can be stale across a quit).

### 2. One floor, one predicate: `reserve_floor`

Ruling O1 changes what "the cheapest ante the player must be able to cover"
*is* while locked, not how many predicates there are. So `economy::cheapest_floor`
keeps its body and becomes **private**, and one new public function fronts it:

```rust
/// The ante the player must be able to cover for the run to continue — the one
/// floor `is_broke`, the Card Shop's reserve and the wager prompt's warning all
/// read. While a series is in progress it is the **locked opponent's** floor:
/// that is the only campaign match the player may play, so a cheaper ante on a
/// planet they are not allowed to visit must not keep a lost run alive
/// (spec 029, ruling O1). Otherwise it is the cheapest ante over every
/// launchable node, exactly as spec 021 had it.
pub fn reserve_floor(run: &CampaignRun) -> u32 {
    match run.series() {
        Some(series) => ante_floor_for(&series.opponent),
        None => cheapest_floor(run),
    }
}
```

All four callers move to it and nothing else changes shape:
`Profile::is_broke`, `Profile::can_afford` (and through it `try_purchase` and
the shop's dimming), `shop.rs`'s *spendable* readout, and
`launch_campaign_node`'s `reserve` argument to `WagerState::new`. Spec 021's
existing `cheapest_floor_is_the_min_over_launchable_nodes` test stands unedited
against the now-private function (same module).

One consequence worth stating because it removes a screen's worth of work:
while locked, `reserve_floor` **equals this opponent's own ante floor**, so a
player who is at the venue and not broke can always cover the match the venue
offers. The wager prompt's `CantCover` branch is therefore unreachable from the
venue, and the venue needs no banner. The branch stays (the map still needs it)
and the venue simply draws nothing for it. This is a claim, so it is a test
(T002).

`tests/balance.rs` calls `cheapest_floor` at **two** sites (a bound and the
report header); both move to `reserve_floor`, which returns the same value for a
default run, so the simulator's output is unchanged. `profile.rs`'s tests call
it at two more; they move too. `docs/economy.md` and one prose line in
`docs/balance.md` name the old function and the old rule, and are part of this
change rather than the close-out's (§Files).

**One visible consequence, named because no walkthrough can reach it.** The
wager prompt's warning row ("lose this and the run is over") is driven by
`WagerState`'s `reserve`, which is this floor. While locked against a deep
opponent the reserve rises from the map's cheapest (10, Cinder's rematch
forever) to that opponent's own ante — 50 against Rix and up — so the warning
fires at far lower stakes than it does today. That is correct under O1 and
probably desirable, but it is a real change in how often a player sees that row,
it is unreachable in any walkthrough this spec runs (it needs a Core-depth run),
and it is therefore recorded here and in the close-out's DECISIONS entry rather
than attested.

Rejected: a `floor_for(run, opponent)` two-argument function (two call shapes,
two chances to pass the wrong one); leaving `is_broke` alone and special-casing
the lock at its two call sites (two copies of the rule, which is what the
bundle's point 3 forbids).

**Pre-merge sweep correction (2026-09-22).** The claim above held only for a
player *already at* the venue. The floor also rises at a third door — the map's
launch of an un-beaten opponent — and nothing checked the balance there, so a
player not broke on the map (Cinder's rematch at 10) could launch a 20-ante
series, be locked into it with no playable match, and lose the run at the next
campaign entry. T011a makes the map's launch refuse an uncovered series exactly
as `open_wager` refuses an uncovered match, through one shared check; with that,
"a player at the venue and not broke can always cover the venue's match" is true.
And the warning's input is no longer `reserve_floor` itself but
`reserve_after_a_loss` — the floor the broke check reads after this match is
lost — because a loss that decides the series releases the lock (sweep N1).

### 3. Settling exactly once stays a **data** property, and now covers everything

Today the guarantee is two separate facts: `take_stake` zeroes the escrow so a
second payout pays `win_payout(0) == 0`, and `mark_beaten` is a set insert. A
series tally increment has neither property, and `mark_beaten` firing on the
*wrong* match would clear a planet early.

Rather than add a second defensive mechanism beside the escrow, this plan folds
both into one: the in-flight node carries a `settled` flag, and one method takes
the whole settlement out of it.

```rust
pub struct NodeRef {
    pub planet: String,
    pub opponent: String,
    #[serde(default)] pub stake: u32,
    /// Whether this match has already been settled (spec 029). Serde-defaults
    /// to `false`, so a node written before this spec settles normally.
    #[serde(default)] pub settled: bool,
}

/// Take this match's settlement — the node and its escrowed stake — **once**.
/// The first call marks the node settled and empties the escrow; every later
/// call returns `None`, so a second settlement pays nothing, moves no series
/// tally and beats nobody. Settling exactly once is therefore a property of the
/// data, not an ordering rule about who calls what: spec 021 bought that for the
/// payout by zeroing the escrow, and spec 029 extends it to cover the series
/// tally and `mark_beaten`, neither of which is idempotent on its own.
/// It does **not** cover `Profile::record_match`, which runs before settlement —
/// a second `resolve_match` would still double-count statistics, as it would
/// before this spec; that one is still guarded only by the `GameOver` edge.
pub fn take_settlement(&mut self) -> Option<(NodeRef, u32)> {
    let node = self.in_progress.as_mut()?;
    if std::mem::replace(&mut node.settled, true) { return None; }
    let stake = std::mem::take(&mut node.stake);
    Some((node.clone(), stake))
}
```

`take_stake` is **deleted** — `take_settlement` is its only caller's
replacement, and two ways to empty one escrow is one too many.

**One consequence, recorded because it moves three existing assertions**
(decision review, 2026-09-20). Because `settle_campaign_match` now opens with
`take_settlement()?`, a **second** settlement of one match returns `None` where
it previously returned `Some(StakeOutcome::Won(0))` / `Some(Lost(0))` — the
escrow was empty, so it paid nothing, but it still returned a value. That is
visible in exactly three `profile.rs` tests (`settling_a_win_…`,
`settling_a_loss_…`, `resolve_match_moves_the_run_credit_counters_…`), whose
second-settlement lines move to `None` in T001, and **nowhere in production**:
`app.rs`'s one caller already handles `None` for every Quick Play match, and the
`phase_changed && GameOver` discriminant edge fires once. The run credit tally is
unaffected either way, because the old second call recorded `win_payout(0)` — a
zero. Acceptance criterion 15 therefore still holds, and this is the evidence for
it.

`stake_at_risk()` is untouched, so `App::stake_to_show`'s "escrow while playing, banner at game
over" behaviour (spec 026) is unchanged: the escrow still reads zero the moment
settlement runs.

The returned clone carries `settled: true` and `stake: 0`; the caller uses only
`planet` and `opponent` from it, and the stake from the tuple. The pointer
itself is **not** cleared — it still clears on the player's acknowledgement,
which is what routes a finished campaign match back into the campaign.

**What it does not cover, stated so the doc comment doesn't overclaim.**
`take_settlement` guards the payout, the series tally and `mark_beaten` — the
three things settlement does. It does **not** guard `record_match`, which
`resolve_match` runs *before* settling, so a second `resolve_match` on one match
would still double-count lifetime and run statistics. That is pre-existing and
still guarded only by the `phase_changed && GameOver` edge; this spec neither
fixes nor worsens it, and `take_settlement`'s doc says so in one clause rather
than claiming the whole resolution is idempotent.

**A match already in flight when this spec ships belongs to a series too**
(sign-off B1). `spec.md` §Saving and resuming is explicit: *"a match left in
flight resolves as the first match of a fresh series against that opponent."*
The first draft of this plan settled such a match through the `NotInSeries`
path, which beats the opponent on a win — a player mid-match against Greeb when
they upgrade would have cleared Cinder in one match, the exact outcome this spec
exists to remove. The approved clause drove the fix, not the other way round:
`record_series_match` **begins a series** when none is running and the opponent
is not yet beaten, and credits the match to it (→ `Continues` at 1–0, routed to
the venue by the derived door like any other undecided series).

The discriminator is `is_opponent_beaten`, not whether a series happens to
exist, which is what makes the property airtight: **a beaten opponent is a
rematch, an un-beaten one is always in a series, so no settled match can beat an
opponent who has not lost one.** `SeriesOutcome::NotInSeries` therefore means
exactly one thing — a rematch against an already-beaten opponent — and the
`NotInSeries && player_won` arm of `beats` (§Design 3) can only ever re-mark
someone already beaten: a no-op, kept because it preserves today's rematch path
literally and costs nothing, not because it does work.

It also means the rule "two wins take an opponent" becomes true in **Phase 1**,
before the venue exists — so Phase 1 is not the invisible phase the first draft
claimed, and its `walkthrough:` marking changes accordingly (`tasks.md`).

Rejected: relying on the `phase_changed && GameOver` edge alone (true today, and
an ordering rule — exactly what spec 021's comment says the design is not); a
separate "matches already counted" ledger on `Series` (more state, same answer);
migrating in-flight nodes at load (`Profile::load` would need campaign rules, and
a node that is never settled would be migrated for nothing).

### 4. The series is its own field, not part of the in-flight node

`Option<Series>` lives on `CampaignRun` beside `in_progress`, not inside it,
because the two have different lifetimes: the pointer is cleared by a Quick Play
match (`start_match(_, None)` calls `set_in_progress(None)`) and by the
kill-with-no-save forfeit in `enter_campaign_continue`, and neither of those
should end a series. Spec: "Quitting to the menu from the venue leaves the
series in progress."

That has one visible consequence the board must handle: during a Quick Play
match started mid-series, a series *is* in progress but this match is not part
of it. The board's series line is therefore gated on the in-flight node matching
the series (§Design 6), not on the series existing.

### 5. How many wins a series needs is derived, never stored

```rust
/// The final opponent — the only best-of-five (ruling G1). One constant rather
/// than a roster field: `opponent.rs` is balance data this spec must not touch,
/// and a second `sovereign` would be a map bug, not a series rule.
pub const FINAL_OPPONENT: &str = "sovereign";

/// Match wins that take a series against `opponent`: 3 for the final opponent,
/// 2 for everyone else (spec 029, ruling G1).
pub fn wins_needed(opponent: &str) -> u32 { if opponent == FINAL_OPPONENT { 3 } else { 2 } }

/// What a launch from the map commits the player to, for the planet detail
/// (AC 2): "Best of 3", or "Best of 5" for the final opponent.
pub fn series_length_label(opponent: &str) -> &'static str { … }
```

Stored in the save it would be a second source of truth that a hand-edited or
older file could contradict. Derived, a pre-029 profile resuming into a fresh
series gets the right length for free. A test pins that every roster opponent
except `sovereign` needs 2, `sovereign` needs 3, and that `FINAL_OPPONENT` is
the last opponent of the last planet on the map.

### 6. Where the series score shows during a match

The wide layout's presence panel is **full**: its 18 interior rows are name (0),
portrait (1–12), gap (13), banter (14), pips (15), gap (16), stake (17). Growing
`PANEL_H_INMATCH` would push the panel past the 31-row minimum.

The status band, however, has room on **both** layouts: two rows the full board
width (81 cells at 89 columns), with the over-20 alert left on row 0, the stake
right on row 0 (compact only), and the turn prompt left on row 1. The longest
turn prompt is 64 characters. So:

**The series score draws right-aligned on the status band's lower row, at both
widths** — one call, no layout branch, and it satisfies AC 11 at 89×31 and
139×31 with the same code. `Series 1 – 0` is 12 cells; 64 + 12 = 76 ≤ 81, and
the fit is a test (T008), not an argument.

It is built by a pure function so the Quick Play and rematch cases are testable
without an `App`:

```rust
/// The series score the board shows (spec 029) — `None` unless this match is a
/// match of the series in progress. A rematch has no series; a Quick Play match
/// started mid-series has no campaign pointer; and a campaign pointer left over
/// from another node is not this series. All three fall out of the same check.
fn board_series_line(node: Option<&NodeRef>, series: Option<&Series>) -> Option<String>
```

**One consequence, flagged** (§Open questions 1): the line is derived from the
live series, and a *deciding* match ends the series during `tick`'s settlement —
so on the game-over frame of the last match of a series the line is gone, while
the "YOU WIN THE GAME" popup is up and the map banner that follows names the
result. Every playable frame of every series match shows the score. If the
person reads AC 11's "during every series match" as including that one frame,
the fix is a one-shot `App` field in the `victory_due` idiom, holding the final
score until the acknowledgement — a sub-lettered task, not a redesign.

### 7. The art region and the portrait are separate by requirement, and the art takes everything else

**Rewritten 2026-09-21** for `spec.md`'s amendment R3. What the amendment
changed is the *shape* of the venue, not this split: two later specs are named
in `spec.md` — authoring per-planet art, and showing more than one opponent on a
planet — and each needs the art and the portrait to be separate regions. That is
a requirement that exists now (a stated constraint from the person, ruling M1 as
amended twice), not speculative generality, so the layout holds two Rects rather
than one panel a later spec would have to break apart.

What the amendment *does* remove is the `Option` around them, and with it the
wrapper struct:

```rust
// Before (T004, ruling N1): both regions existed only at 139 columns and up.
pub rail: Option<VenueRail>,
pub struct VenueRail { pub art: Rect, pub portrait: Rect }

// After (R3): both regions exist at every width, so they are two plain fields.
pub art: Rect,
pub portrait: Rect,
```

`VenueRail` earned its place for exactly one reason — it made "both or neither"
a fact of the type instead of a comment plus a test. Under R3 there is no
"neither": the art is the point of the screen and draws at 89 columns too. A
one-variant wrapper whose doc comment still describes a condition that no longer
exists is the false-name defect this spec renamed four symbols to avoid, so
`VenueRail` and `VENUE_ART_W` are **deleted** rather than kept as an
unconditional container. Nothing outside `venue.rs` reads either, and backward
compatibility with our own three-task-old code is not a constraint (CLAUDE.md,
*Simplicity*).

Two consequences worth stating, because both are places a reader could expect
more machinery than the plan builds:

- **The art is a rectangle, not an L.** The portrait's column is 22 wide and its
  panel 15 rows tall; the art band is 23 rows at the 31-row minimum, so 8 rows
  under the portrait stay blank. Handing those 176 cells to a second art Rect
  would make the art region non-rectangular, and every later art spec would then
  author around a notch. Not worth it; the rows stay blank.
- **The art's size is a subtraction, not a ratio.** The portrait is fixed-size
  (the art block is 18×12 by spec 016), and the header and footer rows are
  fixed counts, so "the art takes the rest" is literal arithmetic. No scale
  factor, no minimum-art-size constant, no breakpoint.
  **Revised 2026-09-21 by amendment R4, and stated as the divergence it is.**
  The *height* is still a pure subtraction. The *width* is now a fraction —
  seven eighths of the columns left of the portrait's gap — because R4 asks for
  a **percentage** reduction and one fixed column inset cannot deliver the same
  percentage at 58 columns of art and at 108 (the arithmetic is in §Design 4,
  which shows no single inset lands inside 15–20 % at both fit sizes). So this
  plan takes exactly one scale factor, in one expression, with its reason; there
  is still no minimum-art-size constant and still no breakpoint.

## Design

### 1. `src/campaign.rs`

```rust
/// The series in progress (spec 029): the run of matches against one opponent
/// on one planet that beats them. At most one exists at a time, and while one
/// does it is the only campaign match the player may play — the **lock**.
/// Cleared by the same reset paths that clear the rest of the run, because it
/// is a plain field on `CampaignRun`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    pub planet: String,
    pub opponent: String,
    pub player_wins: u32,
    pub opponent_wins: u32,
}

/// What a settled campaign match did to the series (spec 029).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesOutcome {
    /// A **rematch against an already-beaten opponent** — the only match that
    /// belongs to no series, and the one case that settles exactly as it did
    /// before this spec. A match against an un-beaten opponent never lands
    /// here: if it arrives with no series running (a match left in flight
    /// across the upgrade to this spec), one is started for it.
    NotInSeries,
    /// The tally moved; the series is still undecided.
    Continues,
    /// The player took the series: the opponent is beaten, now.
    Won,
    /// The opponent took it: the score is discarded, the opponent stays
    /// un-beaten, nothing further is taken (ruling C1).
    Lost,
}

impl CampaignRun {
    pub fn series(&self) -> Option<&Series> { self.series.as_ref() }

    /// Start a series against `opponent` on `planet`, at 0–0. Overwrites any
    /// existing one; the lock means a second can't be reached (the map is the
    /// only caller and is unreachable while locked).
    pub fn begin_series(&mut self, planet: &str, opponent: &str) { … }

    /// Credit a settled match to the series in progress, in three cases and no
    /// others:
    ///
    /// - **The series being played** — this node is the locked one. The
    ///   winner's tally goes up by one, and if it reaches `wins_needed` the
    ///   series ends and the lock is released; the score is discarded either
    ///   way it ends.
    /// - **A rematch** — the opponent is *already beaten*, so this match
    ///   belongs to no series: `NotInSeries`, and nothing moves. This is the
    ///   **only** case that returns it.
    /// - **Anything else is a match against an un-beaten opponent with no
    ///   series of its own**, which means a match left in flight across the
    ///   upgrade to this spec. `spec.md` §Saving and resuming: it "resolves as
    ///   the first match of a fresh series against that opponent" — so one is
    ///   started here and credited (1–0, `Continues`). Without this the match
    ///   would beat its opponent outright and clear a world in one.
    ///
    /// The test is therefore [`Self::is_opponent_beaten`], not whether a series
    /// happens to exist: a beaten opponent is a rematch and an un-beaten one is
    /// always in a series, so no settled match can beat an opponent who has not
    /// lost one. (A *different* series running when that third case fires is
    /// unreachable — the lock means only the locked node can be played — and it
    /// is replaced rather than special-cased, because the alternative is a
    /// variant that exists only for a state the design forbids.)
    ///
    /// Called once per match: `take_settlement` is the guard (§Design tension 3).
    pub fn record_series_match(&mut self, planet: &str, opponent: &str, player_won: bool)
        -> SeriesOutcome { … }
}
```

`CampaignRun` gains `#[serde(default)] series: Option<Series>`; `NodeRef` gains
`#[serde(default)] pub settled: bool`; `take_stake` is replaced by
`take_settlement` (§Design tension 3); `FINAL_OPPONENT`, `wins_needed` and
`series_length_label` as in §Design tension 5.

### 2. `src/economy.rs`

`cheapest_floor` becomes private; `pub fn reserve_floor(run) -> u32` as in
§Design tension 2.

### 3. `src/profile.rs`

```rust
pub struct Settlement {
    pub outcome: StakeOutcome,
    pub completed_run: bool,
    /// What this match did to the series (spec 029). `NotInSeries` for a
    /// rematch, which is exactly this spec's no-op case.
    pub series: SeriesOutcome,
}

fn settle_campaign_match(&mut self, player_won: bool) -> Option<(StakeOutcome, SeriesOutcome)> {
    // One take, one time — the stake, the series credit and the right to beat
    // the opponent all come out of it together (plan §Design tension 3).
    let (node, stake) = self.campaign.take_settlement()?;
    let series = self.campaign.record_series_match(&node.planet, &node.opponent, player_won);
    if player_won {
        self.credits = self.credits.saturating_add(economy::win_payout(stake));
    }
    // The opponent is beaten when the series is won. A series match that did
    // not decide it beats nobody. The `NotInSeries` arm is a rematch win, which
    // since sign-off B1 can only ever re-mark an opponent who is *already*
    // beaten — a no-op, kept because it preserves today's rematch path
    // literally, not because it does work.
    let beats = matches!(series, SeriesOutcome::Won)
        || (matches!(series, SeriesOutcome::NotInSeries) && player_won);
    if beats {
        let was_complete = self.campaign.run_complete();
        self.campaign.mark_beaten(&node.planet, &node.opponent);
        if !was_complete && self.campaign.run_complete() {
            let matches = self.campaign.run_stats().matches_played();
            self.stats.record_campaign_completion(matches);
        }
    }
    let outcome = if player_won { StakeOutcome::Won(stake) } else { StakeOutcome::Lost(stake) };
    Some((outcome, series))
}
```

`resolve_match` is unchanged in shape — it still records statistics before
settling (spec 024's ordering) — and carries `series` into the `Settlement` it
returns. `is_broke` and `can_afford` swap `cheapest_floor` for `reserve_floor`
and are otherwise untouched: `is_broke` stays one pure predicate over the
balance and one floor, evaluated at the same two seams.

### 4. `src/layout.rs`

**Rewritten 2026-09-21** (amendment R3) and **re-proportioned the same day**
(amendment R4, R5, R6). `VENUE_ART_W` and `VenueRail` are deleted; `VenueLayout`
is three horizontal bands, at every width. What R4/R5/R6 change, and nothing
else: the header band gains a fifth row (the credit balance), the art gets
smaller in both axes, and `center_x` is replaced by `text_x` — the **art
region's** centre, which is where every text row now centres.

```rust
/// Height of the opponent's presence panel at the venue — border + name row +
/// portrait, the same 15 rows `opponent_select`'s preview and the map's rail
/// use. The art region beside it is *not* this height: it takes the whole band
/// (spec 029, amendment R3).
pub const VENUE_PANEL_H: usize = 2 + 1 + PORTRAIT_HEIGHT; // 15

/// The venue's geometry (spec 029, amended 2026-09-21, twice): three horizontal
/// bands at **every** width, not a text block with a right rail. Five header
/// rows at the top; then the band holding the per-planet art region and, in its
/// own column beside it, the opponent's presence panel (amendment R3,
/// superseding rulings M1's layout and N1 entirely); then the acted-on action
/// row with an empty row above and below it, and the controls hint on the last
/// row (the constitution's density rule, satisfied by the geometry rather than
/// by what `draw` happens to skip).
///
/// Every text row centres on [`Self::text_x`], the **art's** centre column, not
/// the terminal's — the portrait's column makes those two different points, and
/// text centred on the terminal reads as shifted off the thing it labels
/// (amendment R5).
///
/// Full-screen and computed per-draw from [`Config`], like
/// [`CampaignMapLayout`] — the venue holds no cached geometry and a resize
/// needs no venue-specific code.
#[derive(Debug, Copy, Clone)]
pub struct VenueLayout {
    /// The column every text row centres on: the **art region's** middle, which
    /// is well left of the terminal's middle because the portrait's column and
    /// its gap take 25 columns off the right (amendment R5). Named for what it
    /// is for rather than for what it is derived from: `center_x` would read as
    /// the terminal's centre, which is exactly what R5 stopped using.
    pub text_x: usize,
    /// First of the [`Self::HEADER_H`] header rows (place, planet, opponent,
    /// series, credits), in draw order from here.
    pub header_y: usize,
    /// The per-planet art region — the largest element on the screen. A plain
    /// placeholder in this spec; a later spec replaces its contents without the
    /// layout around it moving.
    pub art: Rect,
    /// The opponent's presence panel, in its own column to the right of the
    /// art, sharing the art's top edge. Its own fixed height
    /// ([`VENUE_PANEL_H`]) — the art is taller.
    pub portrait: Rect,
    /// The action row: the one row the player acts on, with `action_y - 1` and
    /// `action_y + 1` empty.
    pub action_y: usize,
    /// The controls hint, on the terminal's last row.
    pub hint_y: usize,
}

impl VenueLayout {
    /// Header rows above the art: place, planet, opponent, series, credits —
    /// compact, no blank between them. The credit row is amendment R6 and it
    /// gets **no** air: only the acted-on row does (the constitution's density
    /// rule, in its corrected form).
    pub const HEADER_H: usize = 5;
    /// Rows below the art: blank, the action row, blank, the hint.
    pub const FOOTER_H: usize = 4;
    /// The portrait's right margin, and the left edge of the columns the art is
    /// centred within.
    const MARGIN_X: usize = 3;
    /// The share of the columns left of the portrait's gap that the art takes
    /// (amendment R4): seven eighths. A **fraction** rather than a fixed column
    /// inset, because R4 asks for a percentage and the same inset is a
    /// different percentage at 58 columns of art and at 108 — no single inset
    /// lands inside "about 15–20 % smaller" at both fit sizes (plan §Design 4).
    /// The trim is split between the art's two margins, so the art stays
    /// centred in its span and [`Self::text_x`] does not move with it.
    const ART_W_NUM: usize = 7;
    const ART_W_DEN: usize = 8;

    pub fn new(config: Config) -> Self { … }
}
```

`new` takes only `Config` (`block_h` went with R3, and `venue::BLOCK_H` with
it). The arithmetic, in the order the code computes it (each derived edge
clamped with the `.max()` idiom `CampaignMapLayout::new` already uses for
`field_bottom`, so a terminal below the enforced 89×31 minimum degrades instead
of inverting a Rect):

```
header_y     = 0
portrait.x1  = cols - 1 - MARGIN_X
portrait.x0  = portrait.x1 - (PANEL_W - 1)
span_x0      = MARGIN_X                               // the art's available columns:
span_x1      = portrait.x0 - PANEL_GAP - 1  (≥ span_x0)  //   everything left of the gap
span_w       = span_x1 - span_x0 + 1
art_w        = span_w * ART_W_NUM / ART_W_DEN  (≥ 1)  // R4
art.x0       = span_x0 + (span_w - art_w) / 2         // centred in the span
art.x1       = art.x0 + art_w - 1
art.y0       = portrait.y0 = HEADER_H                 // 5 rows of header (R6)
art.y1       = rows - 1 - FOOTER_H              (≥ art.y0)
portrait.y1  = portrait.y0 + VENUE_PANEL_H - 1
text_x       = (art.x0 + art.x1) / 2                  // R5
action_y     = rows - 3
hint_y       = rows - 1
```

Pinned concretely by T005c's test at both `Config::fit_sizes()`:

| | 89×31 | 139×31 |
|---|---|---|
| header rows (place, planet, opponent, series, **credits**) | 0..=4 | 0..=4 |
| the art's available span | x 3..=60 — 58 wide | x 3..=110 — 108 wide |
| `art` | x **7..=56**, y **5..=26** — **50×22 = 1100 cells** | x **10..=103**, y **5..=26** — **94×22 = 2068 cells** |
| clear columns between art and portrait | 57..=63 — **7** (`PANEL_GAP` = 3 plus the trim's right half) | 104..=113 — **10** |
| `portrait` | x 64..=85, y **5..=19** — 22×15 = 330 | x 114..=135, y **5..=19** |
| blank / action / blank / hint | 27 / 28 / 29 / 30 | 27 / 28 / 29 / 30 |
| `text_x` — the art's centre | **31** | **56** |
| *the terminal's centre, no longer used* | *44* | *69* |

**R4, checked rather than asserted.** Against R3's first attempt (58×23 = 1334
cells at 89 columns, 108×23 = 2484 at 139) the art is now **1100 cells — 17.5 %
smaller — at 89, and 2068 cells — 16.8 % smaller — at 139**. Both inside R4's
"about 15–20 % smaller overall", and within 0.8 points of each other, which is
the point of taking a fraction of the width instead of an inset. The test
asserts the *band* (`80 × old ≤ 100 × new ≤ 85 × old` at each fit size, with
1334 and 2484 as literals), not just the two new numbers, so R4 stays honoured
if the arithmetic is ever touched again. Acceptance criterion 16 still holds
arithmetically: the art is more than three times the portrait's 330 at the
minimum size and strictly larger at 139 than at 89.

**Why no fixed inset would do.** With the height at 22 rows, hitting 15–20 %
needs the art 48.5–51.5 columns wide at 89 (an inset of 7 to 9) and 90.3–96.0
at 139 (an inset of 12 to 17). Those ranges do not overlap, so one constant
cannot satisfy both — hence the one scale factor this plan takes, flagged as a
divergence in §Design tension 7 rather than slipped in.

**R5: the hint gets shorter, and that is the decision.** Centring on `text_x`
moves every row about 13 columns left at 89 columns, and the 63-character hint
centred on 31 would span columns 0..=62 — flush against the frame edge, which
`spec.md` forbids in as many words. So the hint drops from the venue's `  ·  `
separators to the board's own single-space `·`:

```
before (63): ←/→ choose  ·  Enter confirm  ·  b shop  ·  c deck  ·  Esc menu
after  (55): ←/→ choose · Enter confirm · b shop · c deck · Esc menu
```

Centred on 31 that spans **columns 4..=58** — four clear columns from the edge,
which is the widest-row case and therefore the binding one. At 139 it spans
29..=83. The reason this is the right lever rather than a cosmetic one:
`board.rs`'s in-match hint already uses ` · ` at this exact spacing, so the
venue is adopting the project's existing idiom, not inventing a second style.

Rejected, each for a stated reason:

- **Centring only the header on the art.** `spec.md` forbids it directly —
  "aligning only the header would trade one visible mismatch for another".
- **Clamping the hint's `x` to a minimum.** The row would then be centred on
  nothing, and the off-centre look R5 exists to remove comes back for that one
  row — while `saturating_sub` already does exactly this silently, which is the
  bug the fit test now catches (§Tests).
- **Splitting the hint over two rows.** It costs a row the art needs and puts a
  second text row inside the footer band, where only the action row may have
  air.
- **Widening the art until it contains the 63-character hint.** Impossible at 89
  columns: the portrait's column and its gap take 25 of the 89, so the widest
  art that can exist there is 58 — and R4 requires about 50.

The shortened hint still overhangs the art box by **3 columns left and 2 right**
at 89 columns, and sits wholly inside the art's columns at 139. `spec.md`
sanctions the overhang explicitly ("either the hint overhangs the art or it gets
shorter"); this design does both, which is why the overhang is 3 columns rather
than 5.

**Nothing collides, and the margins are no longer equal — deliberately.** Under
R3 the art and the portrait had equal outer margins so the *band* sat centred
under text that centred on the terminal. R5 makes that property pointless: the
text follows the art, so the band no longer has to be centred under anything.
The portrait therefore stays exactly where it was (x `cols-25 ..= cols-4` at
both widths — **its Rect's columns do not move in this revision**) and the art's
trim is split between its own two margins, which widens the gap between the two
regions from 3 clear columns to 7 (89) and 10 (139). "Two distinct elements with
room between them" gets strictly better, and the only assertion that has to give
is the one pinning the gap at *exactly* `PANEL_GAP` (§Tests).

Rejected: re-centring the whole art+portrait band as a unit, which would restore
equal margins and the exact `PANEL_GAP` — at the cost of moving the portrait
inward as the art shrinks, which nothing asked for and which would move four
more pinned numbers.

The header rows and the footer rows are the only text, and both bands are
vertically disjoint from the art (`HEADER_H == art.y0`, `art.y1 + 2 ==
action_y`), so horizontal clearance between text and the regions is not a
constraint at all. The widest rows — the hint at 55 cells and the action row at
53 — centred on `text_x` span 4..=58 and 5..=57 at 89 columns, 29..=83 and
30..=82 at 139: clear of both frame edges at both sizes, with the narrow case
binding.

The 31-row minimum leaves the art band 22 rows, 7 more than the portrait panel
needs. That slack is the art's (§Design tension 7), and the portrait is
**top-aligned** with the art rather than centred in the band, so the two regions
share a visible top edge — `CampaignMapLayout::portrait_panel`'s idiom, anchored
at its band's top. Centring it would align it with nothing.

The `FOOTER_H`/`action_y` double encoding that the first amendment's sign-off
carried to the pre-merge sweep (S4) is **left exactly as it is**: the footer is
bottom-anchored, `FOOTER_H` is what the art's bottom edge is derived from, and
quietly resolving a sweep item inside an amendment task would hide it from the
sweep that owns it.

**Revised for ruling R7 (2026-09-22, T005f).** The art keeps its width
(`(cols − 31) × 7/8`, so R4's 1100 / 2068 cells and the brief's 48×20 / 92×20
canvases are unchanged), but the portrait no longer anchors to the right
margin. The gap between art and portrait is exactly `PANEL_GAP` (3) at every
width, and the group — art, gap, portrait — is centred:
`group_w = art_w + PANEL_GAP + PANEL_W`, `art.x0 = (cols − group_w) / 2`,
`portrait.x0 = art.x1 + PANEL_GAP + 1`. At 89×31: art `(7, 56, 5, 26)`
(unchanged), portrait `(60, 81, 5, 19)`, outer margins 7 and 7. At 139×31: art
`(10, 103, 5, 26)` (unchanged), portrait `(107, 128, 5, 19)`, margins 10 and
10. `text_x` is unchanged (31 / 56), so every text row and the hint stay
where T005c put them. This supersedes §Design 4's "the margins are no longer
equal — deliberately": the person found the resulting 7- and 10-column gaps
read as the portrait drifting right.

### 5. `src/venue.rs` (new) + `src/screen.rs`

Copies `opponent_select.rs`'s shape exactly.

```rust
/// The result of a key at the venue: the cursor moved, or one of the four
/// actions was taken. One owned outcome enum, as the constitution requires.
pub enum VenueOutcome { Moved, Play, OpenShop, OpenCollection, QuitToMenu }

pub struct VenueState { selected: usize }  // index into ACTIONS

/// The four actions, in the order they are drawn (ruling K1). No abandon
/// action: a series is played out or lost. The shop's label is `shop::TITLE`
/// (amendment R2) — see below.
const ACTIONS: [&str; 4] = ["Play", crate::shop::TITLE, "Collection", "Quit"];

impl VenueState {
    pub fn new() -> Self;
    /// ←/→ (and `a`/`d`) step the cursor, wrapping; Enter/Space take the
    /// highlighted action; `b` and `c` open the Card Shop and the collection
    /// wherever the cursor is, as on the map; Esc/`x` quit to the menu, the
    /// same as the Quit action.
    pub fn handle_input(&mut self, key: KeyCode) -> Option<VenueOutcome>;
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis);
}
```

**Rows revised 2026-09-21** (amendment R1, R2, R3) and **again the same day**
(R5, R6). The rows are not one contiguous block: **five** are the header band,
four the footer band, and the art sits between them. `venue::BLOCK_H` and the
single `text_rows` array are gone; the header band comes from one pure function
and the footer is drawn from `action_labels` plus the `HINT` const.

```rust
/// The five header rows, in draw order — the one source of truth for their
/// wording, so the fit test measures what is actually drawn. `credits` is the
/// balance row amendment R6 adds: the venue is where the player chooses between
/// playing and shopping, and that is the choice a balance informs.
fn header_rows(
    planet_name: &str,
    planet_region: &str,
    opponent: &OpponentProfile,
    series: &Series,
    credits: u32,
) -> [String; VenueLayout::HEADER_H]
```

| Row (at 31 rows) | Content | Emphasis |
|---|---|---|
| `header_y` = 0 | `Tournament Hall` | Muted |
| 1 | `{planet.name}  ·  {planet.region}` | Strong |
| 2 | `{opponent.name}  —  {opponent.difficulty}` | Normal |
| 3 | `Series  {p} – {o}   ·   {Best of n}` | Normal |
| 4 | `Credits: ◈ {credits}` (R6) | Normal |
| `art.y0`..`art.y1` = 5..=26 | the art region, and the presence panel in its own column beside it | Muted / (the panel's own) |
| 27 | *(blank — the air above the acted-on row)* | |
| `action_y` = 28 | the action row | cursored label takes `pulse` |
| 29 | *(blank)* | |
| `hint_y` = 30 | `←/→ choose · Enter confirm · b shop · c deck · Esc menu` (55 cells, R5) | Muted |

**The credit row's wording is `shop.rs`'s, not a second phrasing — and after
the sign-off it is not a second *literal* either.** The first draft of this
section asserted the shared wording while writing it out twice, which is
exactly the option R1 and R2 rejected on this very screen (R1 routed the
series phrase through `campaign::series_length_label` and asserted the venue
line contains it; R2 made the button read `shop::TITLE`). Nothing could have
caught a drift here: `shop.rs` builds its balance string inline in `draw`, so
no test can reach it, and the plan adds no test for R6. So `shop.rs` gains
`pub fn credits_label(credits: u32) -> String` returning `Credits: ◈ {credits}`;
its own balance row becomes that plus the ` · spendable ◈ …` clause, and the
venue's fifth header row **is** the function's output. One source, two readers,
the same shape `shop::TITLE` already has. The Card
Shop's own balance line opens `Credits: ◈ {credits}`, and the venue's balance
row exists precisely to inform the trip to that screen, so the player meets the
same four words and the same glyph on both. `Emphasis::Normal`, not Muted:
it is information the player acts on, so it must not be dimmed — and not
Strong either, which the planet name owns as the screen's subject. It is a
compact header row like the four above it and gets **no** empty row of its own;
only the action row does (the constitution's density rule, and the reason the
row count moved from 4 to 5 rather than 4 to 6).

The action row is unchanged: `draw_choice_panel`'s idiom, each label drawn as
`"{▸ or space} {label}"` at a fixed stride, so the row's width does not change
as the cursor moves. What changed is that the empty rows around it are now the
layout's (`action_y ± 1`), not array slots — the density rule is a geometric
fact rather than two blank strings in a list, which is also what makes it
checkable on a drawn frame (§Tests).

**Every row is drawn from `layout.text_x`** (R5), which is the art's centre
column, so `draw`'s single `let cx = layout.center_x;` becomes `let cx =
layout.text_x;`, and the only other move in the drawing body is that the
placeholder label drops its own `art_cx` and uses `cx` — after this revision
those are the same expression, so it is one fewer computation of one fact
(re-review, 2026-09-21). One consequence
worth naming because it is R5's whole visible point: the placeholder's own
centred label sits at `(art.x0 + art.x1) / 2`, which is now **the same column**
every text row centres on — the header sits squarely over the art instead of
13 columns right of it.

The series line reads **`Best of 3`**, not `first to 2` (amendment R1), and it
reads it from `campaign::series_length_label` — the same function the map's
planet detail calls (§Design 7), so the player meets one phrase for one idea and
the two screens cannot drift:

```rust
/// The series line: the score, and the length it is played to in the map's own
/// words (amendment R1). `series_length_label` is the single source of that
/// phrase; `wins_needed` stays the source of the *count*, which this line no
/// longer shows.
pub fn series_line(series: &Series) -> String {
    format!(
        "Series  {} – {}   ·   {}",
        series.player_wins,
        series.opponent_wins,
        series_length_label(&series.opponent)
    )
}
```

The middle action label reads **`Card Shop`** (amendment R2), and it reads it
from `shop.rs` rather than spelling it a second time:

```rust
// src/shop.rs — hoisted out of `draw` to module scope, unchanged otherwise.
/// The shop screen's title, and the label the venue's action row shows for it
/// (spec 029, amendment R2). One const, two readers: a button that says one
/// thing and opens a screen headed another is the defect R2 exists to fix, and
/// a shared const makes it unrepresentable rather than merely tested.
pub const TITLE: &str = "Card Shop";

// src/venue.rs
const ACTIONS: [&str; 4] = ["Play", crate::shop::TITLE, "Collection", "Quit"];
```

`"Card Shop"` is the same nine characters as `"Outfitter"`, so
`action_row_width()` is 53 either way and no fit assertion moves (§Tests). The
venue's and the map's `HINT` lines say `b shop`, which is already the new name's
own word — R2's "the key hints where they name it" therefore costs zero lines
here, stated so the absence does not read as a miss.

The art and the portrait draw **unconditionally** — no `if let Some(rail)`:
`draw_box` around `layout.art` (Single, Muted) with the planet's name centered
in it, then `draw_presence_panel(frame, layout.portrait, opponent.name,
opponent.portrait)`, the same drawer the map's rail and the select screen use.
The placeholder's *contents* **stay as they are**: the person closed the
question on 2026-09-21 — the region keeps the planet's name until there is real
art (§Open questions 2, now closed). So R4 changes the region's size and R5 the
column its label centres on; the label itself is untouched.

`draw` reads the series from `&Profile` and returns early if there is none —
`App` only shows this screen while one is in progress, and the early return says
so rather than unwrapping.

`screen.rs` gains `Venue { state: VenueState }`; `lib.rs` gains `pub mod venue;`.

### 6. `src/app.rs`

```rust
/// Open whichever screen the campaign's doors lead to right now (spec 029): the
/// venue while a series is in progress, the map otherwise. **The only place
/// either screen is set.** The answer is derived from the lock rather than
/// remembered by whoever is returning, so there is no origin for a path to
/// forget to set (spec 015's return-path bug).
fn open_campaign_home(&mut self) {
    self.screen = if self.profile.campaign().series().is_some() {
        Screen::Venue { state: VenueState::new() }
    } else {
        Screen::CampaignMap { state: CampaignMapState::new(&self.profile) }
    };
}

/// Enter the campaign and raise whatever the entry calls for — `enter_campaign_map`
/// renamed and re-pointed. The broke check is one of spec 029's two named seams
/// and runs here whichever screen opens, against `reserve_floor` (ruling O1).
fn enter_campaign(&mut self, from_menu: bool) {
    self.open_campaign_home();
    let victory_due = std::mem::take(&mut self.victory_due);
    let primer_due = from_menu && !self.profile.primer_seen();
    self.modal = campaign_entry_modal(self.profile.is_broke(), victory_due, primer_due);
}

/// A launch from the map: a planet with an un-beaten opponent **starts a
/// series** and opens the venue, staking nothing (ruling J1); a cleared planet
/// launches its final opponent as a single staked rematch, exactly as today
/// (ruling F1/L1). `is_opponent_beaten` is the discriminator, so The Anvil's
/// second opponent still starts a series while a cleared Anvil rematches.
fn launch_from_map(&mut self, planet: &str, opponent: &str) {
    if !self.profile.deck_is_valid() {
        self.open_deck_builder(BuilderOrigin::Campaign);
    } else if self.profile.campaign().is_opponent_beaten(planet, opponent) {
        self.open_wager(planet, opponent);          // today's launch_campaign_node body
    } else {
        self.profile.campaign_mut().begin_series(planet, opponent);
        self.profile.save();
        self.open_campaign_home();                   // → the venue, derived
    }
}
```

`launch_campaign_node` becomes `open_wager(planet, opponent)` — the same body
minus the deck guard, which both callers now do — and gains a second caller: the
venue's `Play`, which passes the locked series' planet and opponent. The prompt
opens as a modal over the venue, so cancelling lands back on the venue with
nothing staked (AC 4).

The venue's arm in `handle_key`, beside the map's:

```rust
Screen::Venue { state } => match state.handle_input(key) {
    Some(VenueOutcome::Moved) => self.audio.play(Sfx::MenuMove),
    Some(VenueOutcome::Play) => { … MenuSelect; self.play_series_match(); }
    Some(VenueOutcome::OpenShop) => { … MenuSelect; self.open_shop(); }
    Some(VenueOutcome::OpenCollection) => { … MenuSelect; self.open_deck_builder(BuilderOrigin::Campaign); }
    Some(VenueOutcome::QuitToMenu) => { … MenuBack; self.screen = self.start_menu(); }
    None => {}
},
```

`play_series_match` reads the series and calls the deck guard then `open_wager`.
The `?`-help arm lists `Screen::Venue` with the other screens that carry their
own hint line (no overlay). The draw arm calls `state.draw(frame, &self.config,
&self.profile, pulse)`. No `tick` arm (no animation), no `resize` arm, no
`save_game` arm.

The game-over acknowledgement arm is unchanged except for its call:
`enter_campaign(false)`. It routes correctly with no new condition, because
`tick` has already settled the match and either left the series live (→ venue,
AC 5) or resolved it (→ map, AC 6).

`board_series_line` (§Design tension 6) and the `BoardView::draw` call site;
`MapBanner` construction gains the settlement's `series`.

### 7. `src/campaign_map.rs`

- `MapBanner::Settled(StakeOutcome)` becomes `Settled { outcome: StakeOutcome,
  series: SeriesOutcome }`. `banner_line` prefixes the series result when it is
  `Won`/`Lost` and is otherwise byte-for-byte what it prints today:

  | Case | Line | Emphasis |
  |---|---|---|
  | rematch / no series | `★  Won 40 credits` · `Lost 20 credits` | as today |
  | series won | `★  Series won  ·  Won 40 credits` | Strong |
  | series lost | `Series lost  ·  Lost 20 credits` | Normal |

  A mixed case (series won, stake lost) cannot occur: the deciding match is
  always won by whoever the series result names. That is a claim, so it is a
  test (T003).
- The planet detail gains the series length, right-aligned on the panel's
  `{name} · {region}` row and drawn only when the planet is **not** cleared (a
  rematch commits the player to nothing): `Best of 3` / `Best of 5` from
  `series_length_label` (AC 2). Right-aligning on an otherwise empty row adds a
  row of content without moving any existing row, so every existing map test
  stands unedited.

The map needs no lock handling: it is unreachable while locked.

### 8. `src/board.rs`

`BoardView::draw` gains a `series: Option<&str>` parameter beside `stake`, drawn
right-aligned on the status band's **lower** row (`self.layout.status`, row 1)
with `Emphasis::Strong`, at both widths (§Design tension 6). Nothing else in the
file changes.

### 9. Text (AC 18)

- `assets/primer_text.txt`, after the Card Shop paragraph (the Outfitter
  paragraph until T005b renames it), a blank row and:
  > Beat an opponent by winning 2 matches of 3 — the
  > last opponent, 3 of 5. A series, once started, is
  > played out.

  10 lines → 14. `onboarding_texts_are_the_spec_text_and_fit`'s expected count
  moves 10 → 14 in the same task. No line is wider than the current widest (51),
  so the box does not grow.
- `assets/how_to_play_text.txt`, appended to the Campaign paragraph, **two lines
  only** (the 27-line ceiling; it is at 24):
  > An opponent is beaten by 2 match wins of 3 (the last,
  > 3 of 5), and a series once started is played out.

**Shipped wording (T009, 2026-09-22), superseding the strings above.** Ruling
R1 postdates this section and made "Best of 3" / "Best of 5" the one phrase for
the series length, so the texts use it. The primer: "Each opponent is Best of
3: win 2 matches of 3. / The last is Best of 5: win 3 of 5. A series, / once
started, is played out." How to Play: "Opponents are Best of 3, the last Best of
5, / and a series, once started, is played out." — the phrase without the
counts, because the How to Play line drafted above is 53 columns against the
panel's widest 44 and would have widened the box. The overlay assertions check
"Best of 3", "Best of 5" and "played out" in both, and the counts in the
primer.

### 10. `tests/balance.rs` + `docs/balance.md` (AC 19)

A pure function in the simulator, and a column in its report:

```rust
/// The chance of taking a first-to-`needed` series at a per-match rate `w`
/// (matches are independent and a match always produces a winner). Best of
/// three: w²(3 − 2w). Best of five: w³(6w² − 15w + 10).
fn series_rate(w: f64, needed: u32) -> f64
```

Its guard test uses the three figures `spec.md` already flagged to the person,
which is what makes it a check rather than a restatement: 0.45 → 0.425, 0.60 →
0.648 at best of three; 0.33 → 0.205 at best of five; and 0.50 → 0.50 at both.

`docs/balance.md` gains a **Series rates** section under the measured curve: the
closed forms, and the measured per-match table converted to series rates (best
of three for every opponent, best of five for `sovereign`), with a sentence
naming what it shows — the curve sharpens in both directions, which is what
ruling G1 and spec 022's gates wanted. No economy constant, roster value or
starter-deck entry changes.

### 11. `specs/029-tournament-rounds/planet-art-brief.md` (new)

The second amendment's new deliverable: the brief a design agent is handed to
author the eight planets' venue art, written the way spec 016's
`portrait-art-brief.md` was and living beside it — same directory-of-the-spec
convention, same `-art-brief.md` name, same "hand this whole file over as the
prompt" framing. Documentation, not behaviour: it changes no code and no
acceptance criterion, and authoring the art stays a non-goal of this spec.

**The canvas, and it is the art region's *interior*.** `draw_box` keeps a
single-weight border around `layout.art`, and the art draws inside it, so the
drawable canvas is the Rect minus its border: **48 × 20 at 89 columns** (art
50×22) and **92 × 20 at 139** (art 94×22), from §Design 4's pinned table. The
brief states the derivation as well as the numbers, and says they must be
re-derived from `VenueLayout` — not copied from the brief — if the layout ever
moves again, naming
`the_venue_bands_stack_and_the_art_takes_the_rest` as where they are pinned.
Keeping the border rather than handing the art the whole Rect is deliberate: the
border is already drawn, it reads as a viewport onto the planet, and it means no
asset can collide with the box glyphs.

**Two assets per planet, not one — the load-bearing decision, and the reason.**
The two regions are the same 20 rows but **48 and 92 columns**: the wide one is
nearly twice the narrow one. A single asset would have to be either the wide
grid centre-cropped at 89 columns, throwing away 48 % of the composition on the
one size ruling R3 said the person least wants weak, or the narrow grid centred
in the wide region, leaving 22 blank columns on each side and undoing exactly
the dominance R3 and R4 are about. So the brief asks for **16 files** — eight
planets × `{id}-narrow.txt` (48×20) and `{id}-wide.txt` (92×20) under
`assets/planets/` — and says in as many words that the narrow grid is its **own
composition of the same subject**, not a crop of the wide one. This spends the
cost `spec.md`'s R3 already named and the person accepted ("each planet's art
must work at two quite different sizes") rather than hiding it behind a crop
rule. The brief records the rejected single-asset alternative in one line — a
92-wide grid composed so its central 48 columns stand alone, cropped at the
minimum size — so the later art spec can revisit it knowingly rather than
blind.

**How it loads is a later spec's job, and the brief says so.** The portraits'
mechanism is the model: authored text under `assets/`, `include_str!`-embedded
into a `&'static str` field on the profile struct (`OpponentProfile.portrait`),
drawn line-by-line by a clip-safe drawer. Planet art would sit on a field on
`Planet` in `campaign.rs` and the venue would pick the widest asset that fits
the region and centre it. **That rule is exact at the two fit sizes and
unsettled at every width in between, and the brief says so rather than reading
as settled.** The art interior is `span_w * 7/8 - 2` of a span that varies
continuously with the terminal width, so it is 48 or 92 only at exactly 89 and
139 columns: at 120 the interior is 75 and centring the 48-wide grid leaves 13
and 14 blank columns; at 138 the interior is 91 and it leaves 21 and 22 — the
same emptiness §Design 11's two-asset decision rejects the single-asset option
for — and above 139 the gap reopens and widens with the terminal. **The choice
between letterbox, stretch, tile/extend and authoring a third size is the
deferred art spec's to make**; this spec designs no mechanism for it, and the
brief lists the four with what each costs so that spec chooses knowingly.
Either way the artifact is unchanged: 48×20 and 92×20 are what the two fit
sizes need under all four. **None of that is built here**: the brief states it
as the intended shape and states equally plainly that wiring it up is the
deferred art spec's work, so nobody reads the brief as a work order against this
branch.

## Files

- `src/campaign.rs` — `Series`, `SeriesOutcome`, `FINAL_OPPONENT`,
  `wins_needed`, `series_length_label`, `NodeRef::settled`, `take_settlement`
  (replacing `take_stake`), `series`/`begin_series`/`record_series_match`; tests.
- `src/economy.rs` — `reserve_floor`; `cheapest_floor` made private; tests.
- `src/profile.rs` — `Settlement::series`, `settle_campaign_match`,
  `resolve_match`, `is_broke`, `can_afford`; tests (the `Settlement` literals
  gain a field, the two `economy::cheapest_floor` calls at ~901 and ~1274 move,
  and the settlement helpers learn to play a whole series).
- `src/shop.rs` — one line (`reserve_floor`, T002); then (**T005b**, amendment
  R2) `TITLE` hoisted out of `draw` to a module-level `pub const TITLE: &str =
  "Card Shop"` and the module doc's "the between-worlds outfitter". The
  `specs/025-outfitter-locked-cards` reference in that same module doc is a
  **directory path and stays** — it is the one permitted survivor of T005b's
  grep gate.
  Then (**T005c**, second amendment, added at its sign-off) `pub fn
  credits_label(credits: u32) -> String`, hoisted out of the inline balance
  `format!` so R6's venue row reads the shop's own wording rather than a second
  literal of it — the shape `TITLE` already has. Its remainder
  (`"  ·  spendable ◈ …"`, double-spaced) is preserved byte-for-byte; the shop
  screen renders identically.
- `tests/balance.rs` — **two** `cheapest_floor` call sites (~455 and ~596) move
  to `reserve_floor` (T002); then `series_rate`, the report column and its guard
  test (T010).
- `docs/economy.md` (**T002**) — the reference document for exactly this
  machinery, and this spec falsifies it in seven places: `take_stake`'s entry
  (~107), the exactly-once paragraph (~113–118), the `is_broke` sentence
  (~125–128, the most load-bearing line in the document — the floor, the
  function name and the `enter_campaign_map` seam all move), the shop reserve
  and the "cheapest_floor is 10 in every run state" tuning note (~134, ~151–153,
  ~157, ~165–167), "reached from the campaign map with `b`" (~239, ~262) and the
  named test (~287). A spec that renames four symbols because a false name is a
  defect cannot leave the document that explains them asserting the old rule.
  Rides the branch (CLAUDE.md git conventions), not the close-out.
- `docs/balance.md` — line ~195 names `cheapest_floor` in prose and moves with
  the rename in **T002**; the *Series rates* section is added in **T010**. T010's
  "additions only" bar is therefore about T010's own diff and is not weakened by
  this.
- `README.md` — ~80–81 says "Launching a match opens a **wager prompt**", which
  is no longer what launching an un-beaten planet does (**T009**); and line ~29
  calls the shop "a shop on the map", which **T005b** capitalizes to the screen's
  actual name now that it has one the player reads ("a **Card Shop** on the
  map"). Two lines, two tasks, no overlap.
- **The Card Shop rename's remaining sites** (**T005b**, amendment R2) — comment
  and message text only, no behavior: `src/economy.rs` (two doc comments),
  `src/app.rs` (four comments/docs at ~320, ~749, ~1509, ~3178),
  `src/profile.rs` (one assertion *message* at ~1244 and one comment at ~1247 —
  neither is an asserted value), `assets/primer_text.txt:7`,
  `assets/how_to_play_text.txt:20` (one line each, identical width — see
  §Tests), `docs/economy.md` (~192 and ~311). **Deliberately not renamed**:
  **earlier** specs' directories and `DECISIONS.md` (R2 says so — they record
  what those specs did; **spec 029's own `spec.md` is not in that category and
  was corrected instead**, at the amendment sign-off — see its R2 section),
  `ROADMAP.md` (not among R2's enumerated sites; its Outfitter lines are mostly
  the shipped-spec record, and roadmap grooming commits to `main` rather than to
  a spec branch — a one-line chore if the person wants it), `CLAUDE.md`'s spec
  directory reference, and the `specs/025-outfitter-locked-cards` path in
  `src/shop.rs`. Symbols keep their names: `ShopState`, `ShopOutcome`,
  `open_shop`, `shop.rs` all already say "shop", so R2 renames no code — this is
  the one place in this spec where a rename is *not* warranted, because none of
  these names is false.
- `src/layout.rs` — `VENUE_ART_W`, `VENUE_PANEL_H`, `VenueRail`, `VenueLayout`;
  tests (T004). Then **T004a** (amendment R3): `VENUE_ART_W` and `VenueRail`
  **deleted**, `VenueLayout` rebuilt as the three bands of §Design 4,
  `VenueLayout::new` losing its `block_h` parameter, and
  `the_venue_rail_is_wide_only_and_clear` replaced. `VENUE_PANEL_H` stays as the
  portrait panel's height. `BriefcaseLayout::BLOCK_H` in the same file is an
  unrelated private constant and is untouched. Then **T005c** (amendments R4,
  R5, R6): `HEADER_H` 4 → 5, `ART_W_NUM`/`ART_W_DEN` added, the art centred in
  its span, `center_x` renamed to `text_x` and re-derived from the art, and both
  venue tests re-pinned — the concrete Rects, the gap assertion, the left-margin
  assertion and the area figures, each enumerated with its old and new value in
  `tasks.md`. The two other `center_x` fields in this file — `MenuLayout`'s
  (~152) and `BriefcaseLayout`'s (~269) — belong to other screens, really are
  the terminal's centre, and are **not** renamed.
- `src/venue.rs` (new) + `src/lib.rs` (one `pub mod` line) — the screen; tests
  (T005). Then **T004a** (the bands: `BLOCK_H` and `text_rows` replaced by
  `header_rows` plus the footer's own anchors, the art and portrait drawn
  unconditionally, the module doc's N1 sentence corrected), **T005a** (R1, the
  series line) and **T005b** (R2, the action label and three doc comments).
  Then **T005c**: `header_rows` gains a `credits: u32` parameter and returns
  five rows, `draw` draws the credit row and reads `layout.text_x`, `HINT`
  drops to the board's ` · ` separators (63 → 55 cells), and the fit test gains
  a **left**-edge bar. `src/lib.rs` is not touched again.
- `specs/029-tournament-rounds/planet-art-brief.md` (new, **T005d**) — the
  per-planet art brief the second amendment makes a deliverable, in the shape
  and the location of `specs/016-opponent-portraits/portrait-art-brief.md`.
  Documentation only: no code, no asset and no test changes with it, and
  authoring the art stays a non-goal (§Design 11).
- `src/screen.rs` — the `Venue` variant.
- `src/app.rs` — `open_campaign_home`,
  `enter_campaign` (renamed), `launch_from_map`, `open_wager` (renamed),
  `play_series_match`, the three `Screen::Venue` arms, the four door call sites,
  `campaign_entry_modal` (renamed), `BackTo::Campaign`, `board_series_line`, the
  `MapBanner` construction and `stake_to_show`'s pattern; tests.
- `src/deck_builder.rs` — `BuilderOrigin::Campaign` (renamed).
- `src/campaign_map.rs` — `MapBanner::Settled`'s shape, `banner_line`, the
  planet detail's series length; tests.
- `src/board.rs` — the `series` parameter and its one draw call; tests.
- `assets/primer_text.txt`, `assets/how_to_play_text.txt`, `src/overlay.rs` (one
  expected line count) — **T009**, the added series lines. T005b also touches
  one existing line in each asset for the rename; `src/overlay.rs` is **not**
  part of that (the rename changes no line count and no line width).
- `specs/029-tournament-rounds/closeout-main-docs.md` (T011).
- `src/wager.rs` — **doc only, no behavior change** (added at the sign-off
  re-review, finding B6). `WagerState::new`'s `reserve` doc names
  `economy::cheapest_floor` and calls it "the run's cheapest ante"; under ruling
  O1 the caller passes `reserve_floor`, which while a series is locked is *that
  opponent's* ante. The comment therefore becomes false in `src/` — the exact
  defect the four renames exist to avoid — so T002 corrects it, along with the
  test-helper doc that names the same function. Exempting `wager.rs` from
  T002's grep gate instead would satisfy the gate and leave the false claim
  standing, which is the wrong trade.
- **No change**: `src/game.rs`, `src/card.rs`, `src/player.rs`, `src/save.rs`,
  `src/opponent.rs`, `src/portrait.rs`, `src/frame.rs`,
  `src/render.rs`, `src/main.rs`, `src/paths.rs`, `src/settings.rs`,
  `Cargo.toml`, `Cargo.lock`, the portrait assets.

## Tests

Each claim names the task that owns its check. No new test constructs an `App`
(spec 028 §Design tension 9: `App::new` reads — and since spec 028 repairs — the
real data directory; the roadmap follow-up that would fix it is still open).
Every decision this plan makes is therefore behind a pure function or a method
on `CampaignRun`/`Profile`.

- **A series takes 2 wins, the final opponent 3, and fewer beats nobody**
  (AC 1) — T001 `campaign.rs`:
  `wins_needed_is_two_except_for_the_final_opponent` (every roster opponent;
  `FINAL_OPPONENT` is the last opponent of the last planet) and
  `a_series_resolves_only_at_the_required_wins` (drive `record_series_match`
  through 1–0, 1–1, 2–1 → `Continues`, `Continues`, `Won`, and the series is
  cleared; the 0–2 and 2-of-5 paths likewise).
- **Settling twice settles once** (the bundle's point 4) — T001
  `take_settlement_hands_over_the_match_exactly_once` (node with a stake → the
  node and the stake, then `None`; the escrow is empty and `stake_at_risk` is
  `None` afterwards) and T003
  `resolving_a_match_twice_pays_beats_and_counts_once` (a series at 1–0,
  `resolve_match` twice: credits move once, the tally reaches 2 once,
  `mark_beaten` happens once, `campaign_completions` moves at most once).
- **A pre-029 node deserializes unsettled and unstaked** (AC 13) — T001: a
  `NodeRef` from `{"planet":…,"opponent":…}` has `settled: false` and `stake: 0`,
  and `take_settlement` hands it over once.
- **A match left in flight across the upgrade becomes the first match of a fresh
  series, and does not win a world** (AC 13, sign-off B1) — T001
  `a_match_in_flight_with_no_series_starts_one` (`record_series_match` against
  an un-beaten opponent with `series()` `None` → `Continues`, series 1–0 against
  that node) and T003 `a_pre_029_match_in_flight_does_not_beat_its_opponent` (a
  `CampaignRun` deserialized with an `in_progress` node and no `series` key,
  `resolve_match` with a win → the opponent is **un-beaten**, the planet
  **uncleared**, the stake paid, and the series at 1–0). This is the claim that
  a confident sentence stood in for in the first draft; it is the test that
  would have caught it.
- **A rematch still settles exactly as it did** (AC 12, 15) — T003: the existing
  rematch tests keep their values — a rematch win pays, beats nothing new,
  counts no completion — because an already-beaten opponent is the one case that
  still reaches `NotInSeries`.
- **A series win beats the opponent exactly then; a series loss takes nothing
  further** (AC 1, 7, 8) — T003 `profile.rs`: a full best-of-three against
  `cinder`/`greeb` — after match 1 (won) the opponent is **not** beaten and the
  planet is **not** cleared; after match 2 (won) it is, the planet clears,
  `scree`/`ashfall` unlock and `campaign_completions` is unchanged mid-run;
  separately, 0–2 leaves the opponent un-beaten, the planet uncleared, the
  credits down by exactly the two stakes, and the series cleared to `None`.
- **The deciding match's series result always agrees with its match result**
  (§Design 7's mixed-case claim) — T003: over both lengths and both winners,
  `SeriesOutcome::Won` ⟺ the settled match was won.
- **Reset paths clear the lock** (AC 14) — T003: with a series at 1–0,
  `reset_campaign_run` and `reset_to_starter` each leave `series()` `None`;
  `run_over` reaches the same code (`reset_to_starter`).
- **`reserve_floor` is the locked opponent's floor while locked, the cheapest
  otherwise, and at the venue equals this opponent's own ante** (AC 10, and
  §Design tension 2's unreachability claim) — T002 `economy.rs`:
  `reserve_floor_follows_the_lock` — a half-cleared run reserves 10 (Cinder's
  rematch) with no series, and reserves `ante_floor_for("rix")` = 50 with a
  series against `rix`; for every roster opponent, a run locked against them has
  `reserve_floor == ante_floor_for(id)`, so a player who is not broke can always
  cover the venue's match. Spec 021's
  `cheapest_floor_is_the_min_over_launchable_nodes` stands unedited.
- **Broke is judged against that floor at the two seams** (AC 10) — T002
  `profile.rs`: credits 20, locked against `rix` (floor 50) → `is_broke`, while
  the same profile with no series is not broke; and the shop reserves the
  locked floor (the assertion's message text becomes "the Card Shop reserves…"
  at T005b — a message, not a value). **Corrected at T002 (2026-09-20)**: this bullet originally asked
  for `can_afford(20)` to be false while locked and **true** while free, which is
  arithmetically impossible — free, the floor is 10, so `can_afford(20)` is
  `20 >= 20 + 10`, false. The claim is real, the illustrative numbers were not;
  the test asserts it at a price the difference can show, `can_afford(10)`
  (`20 >= 10 + 50` false while locked, `20 >= 10 + 10` true while free), and
  keeps `!can_afford(20)` while locked.
- **Every campaign door lands on the venue iff a series is in progress** (AC 9,
  and the bundle's point 2) — **not a unit test, deliberately.** The mapping
  itself is an `if` over a boolean and a test of it would be a tautology; the
  failure that actually costs something is a *second* assignment site elsewhere
  in `app.rs`, which no mapping test can see. So this rule is pinned by a
  mechanical check in T006's Verify and at the Phase 2 review —
  `grep -n "self\.screen = Screen::\(CampaignMap\|Venue\)" src/app.rs` returning
  exactly two lines, both inside `open_campaign_home` — and attested in the
  Phase 2 walkthrough, which presses every door.
- ~~**The venue fits both layouts, and the rail is wide-only and
  non-overlapping**~~ / ~~**The venue's rows read right and breathe**~~ — T004
  and T005's original claims, **superseded 2026-09-21** by the four bullets
  below. Recorded rather than deleted because T004a and T005a change assertions
  those two tasks shipped, and the list of what changed is part of the task
  lines (`tasks.md`).
- **The venue's bands stack, and the art takes every row and column the text
  does not need** (AC 16) — T004a, **re-pinned at T005c** (R4/R5/R6),
  `layout.rs`, `the_venue_bands_stack_and_the_art_takes_the_rest`: at both
  `Config::fit_sizes()` — `header_y == 0` and `header_y + HEADER_H == art.y0`
  (the header rows sit directly above the art; `HEADER_H` is 5 from T005c, and
  this assertion is symbolic so it moves value without moving text); `art` and
  `portrait` both on-frame; `art.x1 + PANEL_GAP < portrait.x0` (**at least**
  `PANEL_GAP` clear columns — the exact-gap `assert_eq!` T004a shipped is
  **deleted** at T005c, because R4's trim widens the gap to 7 columns at 89 and
  10 at 139; its surviving half is this line and the exact counts come from the
  pinned Rects); `art.x0 >= MARGIN_X` (T004a's `art.x0 == 3` is **replaced** —
  the art is centred in its span now, so its left margin is 7 at 89 and 10 at
  139); `portrait.x1 == cols - 4` (**unchanged in value** — the portrait's
  columns do not move in this revision); `portrait.y0 == art.y0` and
  `portrait.y1 <= art.y1` and `portrait.height() == VENUE_PANEL_H` (shared top,
  the art taller); `art.y1 + 2 == action_y`, `action_y + 2 == hint_y`,
  `hint_y == rows - 1` (AC 17's air, as geometry). And the concrete Rects of
  §Design 4's table are pinned at both sizes, in the
  `assert_eq!(WIDE_LAYOUT_MIN_WIDTH, 139)` spirit — the relational assertions
  above would let a uniform arithmetic slip through, the pinned numbers would
  not. **T005c moves all four**: art `(3,60,4,26)` → `(7,56,5,26)` and portrait
  `(64,85,4,18)` → `(64,85,5,19)` at 89; art `(3,110,4,26)` → `(10,103,5,26)`
  and portrait `(114,135,4,18)` → `(114,135,5,19)` at 139.
- **Every text row centres on the art, not on the terminal** (amendment R5) —
  T005c `layout.rs`, in the same bands test: `text_x == (art.x0 + art.x1) / 2`
  at both fit sizes, and pinned concretely — **31 at 89 columns and 56 at 139**,
  against the terminal centres of 44 and 69 that R5 stopped using, named in the
  test's comment so a reader sees what the numbers are *not*. This is the claim
  the whole ruling rests on, so it is asserted rather than left to the Rect pins
  (which would satisfy it by coincidence if `text_x` were computed some other
  way).
- **The art region is the largest element, at both widths, larger when the
  terminal is wider, and 15–20 % smaller than R3's first attempt** (AC 16's
  amended sentence, and R4) — T004a, **extended at T005c**, `layout.rs`,
  `the_art_region_dominates_at_both_widths`: at each fit size the art's area
  exceeds the portrait's (1100 and 2068 against 330), and the art's area at 139
  strictly exceeds its area at 89 (1100 → 2068) — both assertions keep their
  text and only their values move. **T005c adds R4's band**, which is the claim
  the person will eyeball and therefore the one that needs a check: at each fit
  size, `80 × old <= 100 × new && 100 × new <= 85 × old`, with R3's figures
  (**1334** at 89, **2484** at 139) as literals in the test. That asserts "about
  15–20 % smaller" rather than asserting two new numbers that happen to satisfy
  it today, so a later touch to the fraction or to `HEADER_H` fails here instead
  of drifting. The test's comment, which currently states the 1334/2484 figures
  as the *current* areas, becomes false and moves with them.
- **The venue's rows breathe and fit** (AC 3, 17) — T004a `venue.rs`:
  `the_action_row_keeps_its_width_as_the_cursor_moves` — unchanged from T005,
  and it stays green through T005b too, because `"Card Shop"` is the same nine
  characters as `"Outfitter"` and `action_row_width()` is 53 either way;
  `the_venue_rows_breathe_only_around_the_action_row` — **now a frame test**,
  replacing T005's array-index version: draw the venue at 89×31 into
  `frame::new_frame` over a `Profile::default()` with
  `campaign_mut().begin_series("cinder", "greeb")` (no `App` — plan §Tests'
  standing rule), then assert rows `action_y - 1` and `action_y + 1` are
  **entirely blank** while the header rows, `action_y` and `hint_y` carry
  text. The density rule is now a fact about the drawn frame rather than about
  two empty strings in an array, which is the stronger form of the same check;
  `the_venue_text_fits_the_minimum_terminal` — the header rows, the action
  row (as `action_labels(selected).join(gap)`, the same string the stride loop
  draws — it is no longer one of the returned rows, so the test names it
  explicitly rather than losing it) and the hint, for every planet × opponent ×
  selection, centered on `center_x` — `text_x` from T005c, a mechanical rename —
  inside the frame at both fit sizes. Its
  "ends left of `rail.art.x0`" clause **is deleted**
  (the text is above and below the art now, not beside it) and is replaced by
  the vertical-disjointness assertions in the layout test, checked once instead
  of per row.
- **No drawn row touches either frame edge** (amendment R5, and `spec.md`'s "it
  must not end up flush against column 0, which reads as a rendering fault") —
  T005c `venue.rs`, in `the_venue_text_fits_the_minimum_terminal`. The test's
  bar was `x + w <= cols`, a **right**-edge check only, and `x` comes from
  `text_x.saturating_sub(w / 2)`, which clamps a left overflow silently to
  column 0 — so the exact failure R5 forbids would have passed. The bar becomes
  `x >= 1 && x + w <= cols - 1` for every measured row, **plus the binding case
  pinned**: at 89 columns the hint's left column is exactly **4**. Pinning 4
  rather than "greater than 0" is what makes an unsanctioned re-lengthening of
  the hint fail — 63 characters would put it at 0, and even 57 would put it at 3
  — instead of quietly closing on the edge. This is the claim §Design 4's R5
  decision is made of, and it is the one place a confident sentence would
  otherwise have stood in for a check.
- **The credit balance is on the screen** (amendment R6) — T005c `venue.rs`,
  and it needs **no new test**: `the_venue_rows_breathe_only_around_the_action
  _row` asserts every row in `header_y .. header_y + HEADER_H` carries text on
  the drawn frame, so raising `HEADER_H` to 5 makes row 4 — the credit row — a
  row the existing assertion requires to be non-blank. Its *coverage* grows
  without its text changing, which is worth saying out loud so the phase review
  does not read the absence of a new test as a gap. The row's width is covered
  by the fit test, which measures it at `u32::MAX` credits so the widest
  representable balance can never become the binding row unnoticed.
- **The venue and the map say the same words for the series length** (amendment
  R1) — T005a `venue.rs`, `the_series_line_names_the_score_and_the_length`, both
  of whose expected strings change value: `Series  1 – 0   ·   Best of 3` and
  `Series  2 – 1   ·   Best of 5`. Plus one added assertion — `series_line`
  contains `series_length_label(&series.opponent)` — so R1's actual point (one
  phrase for one idea) survives a later edit to either screen instead of resting
  on two literals that happen to match today.
- **The art brief's canvas matches the layout it was derived from** (the second
  amendment's new deliverable) — T005d, and deliberately **a check at authoring
  time rather than a standing test**: the brief is a document, nothing in the
  build reads it, and a unit test asserting a Markdown file's numbers would pin
  prose. So the check is T005d's own Verify — re-derive 48×20 and 92×20 from
  `the_venue_bands_stack_and_the_art_takes_the_rest`'s pinned Rects (`50 - 2` ×
  `22 - 2`, `94 - 2` × `22 - 2`) and confirm the brief states the same pair, and
  confirm the eight planet ids in the brief are exactly `PLANETS`'. The brief
  itself then carries the standing obligation forward: it tells the later art
  spec to re-derive the canvas from `VenueLayout` rather than trust the
  document, and names the test that pins it. Stated as a verification gap on
  purpose rather than left to be discovered: if the geometry moves again before
  the art is authored, nothing in CI will notice, and the brief is where that is
  written down.
- **The shop is the Card Shop wherever the player reads it** (amendment R2) —
  T005b, and deliberately **a gate rather than a test**: `grep -rniE "outfitter"
  src/ assets/ docs/ README.md` returns only `src/shop.rs`'s
  `specs/025-outfitter-locked-cards` path. A unit test asserting a UI string
  equals itself would be the tautology sign-off rejected for `CampaignHome`; and
  the one invariant that *is* worth making impossible — the button and the
  screen header disagreeing — is a shared `const`, not an assertion
  (§Design 5).
- **Declining the wager returns to the venue** (AC 4) and **a match saved
  mid-play resumes and then routes by AC 5 and AC 6** (AC 13) — **walkthrough
  only**, Phase 2. Neither has a unit test and neither can get one without
  constructing an `App`: the first is the wager modal's existing Esc arm leaving
  the screen underneath untouched, the second spans a quit, a relaunch and a
  resume. Both are keystrokes in the Phase 2 script (§Verification) rather than
  claims left standing on the plan's word.
- **The venue's keys** (AC 3) — T005: `←`/`→`/`a`/`d` wrap over four actions;
  Enter and Space take the highlighted one; `b` and `c` give `OpenShop` and
  `OpenCollection` from any cursor position; Esc and `x` give `QuitToMenu`;
  unknown keys give `None`.
- **The map names the series length before the launch, and not on a cleared
  planet** (AC 2, 12) — T007 `campaign_map.rs`: `series_length_label` is
  `Best of 3` for every opponent but `sovereign` and `Best of 5` for it; the
  panel's detail row carries it for an un-cleared planet and not for a cleared
  one.
- **The banner names the series result beside the settlement** (AC 6) — T007:
  `banner_line` over the four cases in §Design 7's table, including that
  `NotInSeries` and `Continues` print exactly today's string.
- **The board shows the score during a series match and not otherwise** (AC 11)
  — T008 `app.rs` (pure): `board_series_line` is `Some` for a node matching the
  series, `None` for no node (Quick Play started mid-series, with the series
  still live), `None` for a node with no series (a rematch), and `None` for a
  node that is not the series' node.
- **The score fits beside the longest turn prompt at both widths** (AC 11) —
  T008 `board.rs`: at both `Config::fit_sizes()`, the longest
  `status_message` (the ±/tiebreaker prompt, 64 cells) plus the widest drawn
  series line (`Series 9 – 9`, 12 cells — the board carries the score only; the
  length lives at the venue and on the map) leave at least one blank cell
  between them inside `layout.status.width()` (81); and a drawn
  frame at 89×31 has the line ending at `status.x1` on row 1 with the prompt
  ending left of it — the
  `the_compact_board_carries_the_stake_clear_of_the_alert` idiom, one row down.
- **The texts say the rule and still fit** (AC 18) — T009 `overlay.rs`: the
  existing `help_texts_fit_the_minimum_terminal_unclamped`,
  `onboarding_texts_are_the_spec_text_and_fit` (count 10 → 14) and
  `onboarding_texts_breathe_only_around_the_dismiss_line` all pass, plus two
  added assertions that the Primer and How to Play each contain "2 matches of 3"
  / "2 match wins of 3" and "3 of 5".
- **The series rates are the closed forms** (AC 19) — T010 `tests/balance.rs`:
  `series_rate_matches_the_closed_form`, the vectors in §Design 10.
- **Nothing forbidden changed** (AC 20) — T011: `git diff main...HEAD --stat`
  lists none of `src/game.rs`, `src/card.rs`, `src/player.rs`, `src/save.rs`,
  `src/opponent.rs`, `Cargo.toml`, `Cargo.lock`; `grep -n "VERSION"
  src/save.rs src/profile.rs` still reads 1 and 1; `git diff main...HEAD --
  src/opponent.rs docs/economy.md` is empty of balance-value changes.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim.
- **Driver walkthroughs** (the orchestrator, `run-kaazap` skill, **with
  `KAAZAP_DATA_DIR` pointed at a scratch directory** — these walkthroughs play
  campaign matches and reset runs; the real profile is never in play, and the
  report says which directory was used).
  - **After the Phase 1 review**, at 89×31: from a fresh scratch profile, Start
    Campaign → Enter on Cinder → wager → win the match. **The planet does not
    clear**, the map still shows Greeb as the next opponent, and Enter launches
    him again. Win a second time: now Cinder clears and Scree and Ashfall
    unlock. Then lose the first match of the next opponent and win the two
    after it — still cleared. There is no venue yet and the map is still
    reachable between matches; that arrives in Phase 2. (Phase 1 was marked
    `walkthrough: none` in the first draft of this plan on the strength of a
    settlement rule that sign-off B1 replaced; the two-wins rule now becomes
    true here, which is something the person can see and therefore something
    they should.)
  - **After the Phase 2 review** — **run and attested by the person 2026-09-21,
    against the pre-amendment venue**; kept as the record of what was attested,
    not as a script to re-run. Its layout sentences ("at 139 a bordered art
    region … at 89 neither") describe the design amendment R3 replaces; the
    re-walkthrough below covers the new layout and wording only, and must not
    re-run this script. At 89×31 and again at 139×31: from a fresh
    scratch profile, Start Campaign → Enter on Cinder — **the venue opens, at
    0–0, with nothing staked** (the credit balance is unchanged). Read the
    venue: the planet, the opponent, `Series 0 – 0 · first to 2`, four actions
    with one marked, a hint line; at 139 a bordered art region with the planet's
    name in it and the opponent's portrait beside it, neither overlapping nor
    clipped; at 89 neither, and nothing clipped. Press `b` and Esc — back at the
    venue, not the map. Press `c` and Esc — back at the venue. **Choose Play,
    then Esc at the wager prompt** — back at the venue, nothing staked, the
    balance unchanged (AC 4, which nothing else checks). Play a match and
    lose it deliberately (bust twice): the popup, then **the venue at 0 – 1**.
    Quit to the menu from the venue, re-enter the campaign — **the venue again,
    at 0 – 1**, and the map is not reachable. **Start the next match, quit to the
    menu mid-play, relaunch, take Continue** — the match resumes where it was;
    finish it and confirm it lands on the venue with the score moved, or on the
    map if it decided the series (AC 13, likewise otherwise unchecked). Win two:
    the second win lands on the **map**, Cinder cleared. Then Enter on cleared
    Cinder: the **wager prompt**, no venue (AC 12).
    **Two questions for the person in this pause report**, asked as questions
    rather than reported as findings, because both are about what the venue
    shows and that is theirs to decide: (a) the art region's placeholder is the
    **planet's name**, and the venue's second row already names the planet six
    rows above it — do they want it emptier (§Open questions 2)? (b) the venue
    shows **no credit balance**, and it is the screen where the player chooses
    between playing and shopping, which is the choice a balance informs. It is
    deliberately not in this design; ask before it becomes one.
  - **After the amendment tasks (T004a, T005a, T005b) and their phase
    re-review**, at 89×31 and again at 139×31, on a scratch `KAAZAP_DATA_DIR`:
    start a series from Cinder to land at the venue. The venue now reads as
    **bands** — four rows at the top (the hall, the planet, the opponent, the
    series), then a large bordered art region with the opponent's portrait in
    its own column beside it, then the action row and the hint at the foot. The
    art region is the biggest thing on the screen at **both** sizes and visibly
    bigger at the wider one; nothing is clipped, the portrait does not touch the
    art, and the action row still has an empty row above and below it (AC 16,
    17). The series row reads `Series 0 – 0 · Best of 3` — the words the map's
    planet detail gains at **T007**, which has not landed yet, so the pause
    report must not claim the map already says them (sign-off, 2026-09-21) (R1).
    The middle action says **Card Shop**, and taking it (or `b`) opens a screen
    headed **Card Shop**; Esc returns to the venue (R2). Then the **one item
    Phase 2 never attested** (Phase 2 review note 23, uncovered because the
    driver could not deliver Tab): at the venue press `c`, remove a card so the
    deck is invalid, Back, then Play — the deck builder opens, and Back from it
    returns to the venue. **Nothing else from the Phase 2 script is re-run**;
    the flow, the lock, the routing and the settlement are already attested and
    none of these three tasks touches them. **The still-open question is
    re-put**, unchanged and unanswered: the art region's placeholder is the
    planet's name, in a much larger box now — emptier, or as it is
    (§Open questions 2)?
  - **After the second amendment's tasks (T005c, T005d) and their phase
    re-review**, at 89×31 and again at 139×31, on a scratch `KAAZAP_DATA_DIR`:
    start a series from Cinder to land at the venue. Three things to look at,
    and nothing else is re-run.
    (a) **The proportions** (R4). The art region is still clearly the biggest
    thing on the screen and still visibly bigger at the wider size, but smaller
    than it was — about a sixth smaller by area at both sizes. This is the
    person's eyeball call, and it is the one thing in this pause only they can
    settle; the numbers behind it are in §Design 4's table.
    (b) **The alignment** (R5). Every row of text — the hall, the planet, the
    opponent, the series, the credit balance, the action row and the hint — now
    sits centred over the **art box**, not over the whole screen, so the block
    of text reads as belonging to the picture under it rather than drifting
    right of it. The hint is shorter than it was (single spaces around its
    dots), and at the narrow size it sits four columns clear of the left edge
    rather than against it.
    (c) **The balance** (R6). The fifth header row reads `Credits: ◈ …`, the
    same words the Card Shop's own balance line uses, and it changes after a
    match settles: play one and come back to the venue to see it move.
    Also confirm the action row still has an empty row above and below it and
    the header rows are still tight together (AC 17), and that nothing is
    clipped at either size (AC 16).
    **T005d has nothing to look at** — it writes the per-planet art brief, a
    document on this branch — but the pause report should say in one line that
    it exists and what it commits to, because it asks a design agent for
    **two** grids per planet (one for each layout width) and that is the cost
    ruling R3 named. If the person would rather spend eight drawings than
    sixteen, the brief records the single-asset alternative and the later art
    spec can take it.
    **Nothing else from either earlier Phase 2 script is re-run**: the flow,
    the lock, the routing, the settlement, the `Best of 3` wording and the Card
    Shop rename are all attested and none of these two tasks touches them.
  - **After the Phase 3 review**, at both widths: on the map, the planet detail
    of an un-cleared planet says `Best of 3` (and Zenith's, once reachable or by
    inspection, `Best of 5`); during a series match the status band's lower row
    carries `Series 1 – 0` with the turn prompt beside it and nothing overlapping
    at 89 columns; during a **rematch** the line is absent. After the deciding
    match the map banner names the series result and the stake together.
    **One question for the person**: after a match that does *not* decide the
    series, the band shows the freshly-updated score while the game-over popup is
    up; after the match that *does* decide it — won or lost — the band shows
    nothing, because the series is over by then (§Open questions 1). Show them
    both frames and ask what that last frame should say. Holding the final score
    there is a sub-lettered task, not a redesign.
  - **After the Phase 4 review**: on a fresh scratch profile, the first-campaign
    primer names the two-of-three rule, the three-of-five final and the
    commitment, fits 89×31 unclamped, and has one empty row above its dismiss
    line; How to Play's campaign section says the same and is not clipped.

## Non-goals (from spec)

No per-planet venue art beyond a reserved region and a plain placeholder, no
per-planet music, no series-aware banter, no new records or statistics, no
rematch series, no abandon action, no re-tuning of the curve, and no change to
the match engine, the opponent AI, the save format, the wager arithmetic or the
balance data.

## Open questions

Settled here as design. **All five were reviewed at sign-off (2026-09-20) and
stand**; the first two carry a walkthrough obligation recorded with them.
**Question 2 is closed by the person as of 2026-09-21** (the placeholder keeps
the planet's name), so only question 1 still owes the person an answer — at the
Phase 3 pause.

1. **The in-match series line is gone on the deciding match's game-over frame**
   (§Design tension 6). It is derived from the live series, and the deciding
   match ends the series during settlement, one tick before the popup. Every
   *playable* frame of every series match shows it, and the map banner that
   follows names the result. **Sign-off: not an AC 11 violation — it is present
   on every frame the player can act in.** But it leaves the two game-over
   frames *inconsistent* with each other: after a non-deciding match the band
   shows the freshly-updated score, after the deciding one (won or lost) it
   shows nothing. **The Phase 3 walkthrough must show the person both frames and
   ask what the last one should say.** If they want the final score held, it is
   a one-shot field on `App` in the `victory_due` idiom, taken at the
   acknowledgement: a sub-lettered task, not a redesign. Held back for now
   because one meaning of "a series is in progress" is worth more than one frame
   of a number.
2. ~~**The art placeholder carries the planet's name**~~ — **CLOSED by the
   person, 2026-09-21** (`spec.md`, *Amendment, 2026-09-21 (second)*: "the
   placeholder keeps the planet's name until there is real art. Plan §Open
   questions 2 is **closed** — keep the label"). It was asked at the Phase 2
   pause and re-put at the first amendment's re-walkthrough; the answer is keep
   it. So **no task changes the placeholder's contents**: T005c changes the
   region's size (R4) and the column its label centres on (R5), and the label
   itself is untouched. Recorded rather than deleted because two task lines
   (T004a, T005b's pause text) point at this entry as an open choice, and a
   reader arriving from them needs to find the answer here rather than the
   question.
3. **Esc and `x` at the venue quit to the menu**, the same as the Quit action.
   `spec.md` lists four actions and does not bind Esc; every other screen in the
   game backs out on Esc/`x`, and there is nothing else for it to mean here (no
   abandon action).
4. **Four renames** (`take_stake` → `take_settlement`, `cheapest_floor` →
   `reserve_floor`, `BuilderOrigin::Map`/`BackTo::Map` → `…::Campaign`,
   `map_entry_modal` → `campaign_entry_modal`). Each is a name that would
   otherwise assert something false after this spec, and each is ≤ 4 lines plus
   a test name. Flagged because renames cost review attention that a diff
   otherwise spends on behaviour.
5. **The venue draws no banner.** The wager prompt's `CantCover` refusal is
   unreachable from the venue (§Design tension 2), so the only banner the venue
   could show cannot fire there. **Sign-off verified the reasoning** —
   `MapBanner::CantCover` is set in exactly one place, `reserve_floor` equals the
   locked opponent's ante while locked, `is_broke` is checked at both seams,
   `can_afford` reserves the same floor, and credits otherwise only fall by
   staking, so not-broke ⇒ the branch cannot fire from the venue — **and
   corrected the consequence sentence**: if it *did* fire, the player would not
   merely "hear the back cue and see nothing change". `self.banner` is `App`
   state cleared only by the map's own input arm, so a stale `CantCover` would
   surface on their next visit to the map. Pinned by T002's test rather than by
   a branch, and this is the reason the test matters rather than being
   defensive decoration.
6. **`CampaignHome` was dropped** (§Design tension 1). The first draft wrapped
   the map-or-venue choice in a two-variant enum plus a `campaign_home(bool)`
   mapping, for testability. Sign-off observed that the unit test would be a
   near-tautology, and on reflection it would also have tested the wrong thing:
   the failure that costs something is a second `self.screen = Screen::…`
   assignment elsewhere, which only a grep catches. The enum is gone, the `if`
   is inline, and the grep is in T006's Verify and the Phase 2 review. One fewer
   type, one fewer test, and the check now points at the real risk.
