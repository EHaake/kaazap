# Plan: Tournament rounds — spec 029

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

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
  (139), used by `BoardLayout::opponent_panel`. The venue's rail uses the same
  test, so N1 is one line, not a mechanism.
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
| Back from the Outfitter | `open_campaign_map()` | `open_campaign_home()` |
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
/// floor `is_broke`, the Outfitter's reserve and the wager prompt's warning all
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
/// payout with `take_stake`'s zeroing, and spec 029 extends it to cover the
/// series tally and `mark_beaten`, neither of which is idempotent on its own.
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
replacement, and two ways to empty one escrow is one too many. `stake_at_risk()`
is untouched, so `App::stake_to_show`'s "escrow while playing, banner at game
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

### 7. The venue's art region and the portrait are separate by requirement

Two later specs are named in `spec.md` — authoring per-planet art, and showing
more than one opponent on a planet — and each needs the split. That is a
requirement that exists now (a stated constraint from the person, ruling M1 as
amended), not speculative generality, so the layout holds two Rects rather than
one panel that a later spec would have to break apart.

They are one field, not two, because they are always both present or both
absent:

```rust
/// The wide layout's right rail (spec 029): the per-planet art region and, beside
/// it, the opponent's presence panel. Two Rects because two later specs need them
/// separate — the art is the planet's and does not change with the opponent, and a
/// planet may later hold more than one opponent. `None` below
/// WIDE_LAYOUT_MIN_WIDTH, exactly as the board's presence panel (ruling N1).
pub rail: Option<VenueRail>,
pub struct VenueRail { pub art: Rect, pub portrait: Rect }
```

One small struct instead of two `Option<Rect>` fields makes "both or neither" a
fact of the type rather than a comment plus a test.

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

```rust
/// The per-planet art region reserved at the venue (spec 029, ruling M1): a
/// plain placeholder in this spec, its contents replaced by a later one without
/// the layout around it moving. Sized to match the presence panel's height so
/// the two elements beside each other share a top and a bottom edge.
pub const VENUE_ART_W: usize = 30;
pub const VENUE_PANEL_H: usize = 2 + 1 + PORTRAIT_HEIGHT; // 15, as opponent_select's preview

pub struct VenueRail { pub art: Rect, pub portrait: Rect }

/// The venue's geometry: a centered text block, and — from
/// WIDE_LAYOUT_MIN_WIDTH columns up — a right rail holding the planet art
/// region and the opponent's presence panel beside it. Below that width the
/// rail is `None` and the text block centers on the whole terminal, exactly as
/// the board drops its presence panel (ruling N1).
pub struct VenueLayout {
    pub center_x: usize,
    pub top: usize,          // first row of the text block
    pub rail: Option<VenueRail>,
}

impl VenueLayout {
    /// Rows of the text block: place, planet, opponent, series, blank, actions,
    /// blank, hint (`venue::BLOCK_H`).
    pub fn new(config: Config, block_h: usize) -> Self { … }
}
```

Arithmetic, pinned by T004's test: rail width = `VENUE_ART_W + PANEL_GAP +
PANEL_W` = 30 + 3 + 22 = 55, right margin 3, so at 139 columns the rail spans
81..=135 and the text area is 0..=80 with `center_x = 40`; the widest text row
(the hint, ~62 cells) spans 9..=71, clear of the rail. `top = (rows -
block_h) / 2` = 11 at 31 rows; the rail is vertically centered on its own
height, rows 8..=22. At 89 columns `rail` is `None` and `center_x = 44`.

`VENUE_ART_W = 30` is a chosen canvas size, not derived from anything: 30×15
character cells is roughly square on screen and comfortably bigger than a
portrait. Changing it later is one constant and one test.

### 5. `src/venue.rs` (new) + `src/screen.rs`

Copies `opponent_select.rs`'s shape exactly.

```rust
/// The result of a key at the venue: the cursor moved, or one of the four
/// actions was taken. One owned outcome enum, as the constitution requires.
pub enum VenueOutcome { Moved, Play, OpenShop, OpenCollection, QuitToMenu }

pub struct VenueState { selected: usize }  // index into ACTIONS

/// The four actions, in the order they are drawn (ruling K1). No abandon
/// action: a series is played out or lost.
const ACTIONS: [&str; 4] = ["Play", "Outfitter", "Collection", "Quit"];

impl VenueState {
    pub fn new() -> Self;
    /// ←/→ (and `a`/`d`) step the cursor, wrapping; Enter/Space take the
    /// highlighted action; `b` and `c` open the Outfitter and the collection
    /// wherever the cursor is, as on the map; Esc/`x` quit to the menu, the
    /// same as the Quit action.
    pub fn handle_input(&mut self, key: KeyCode) -> Option<VenueOutcome>;
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis);
}
```

Eight rows, in `VenueLayout`'s block, centered on `center_x`:

| Row | Content | Emphasis |
|---|---|---|
| 0 | `Tournament Hall` | Muted |
| 1 | `{planet.name}  ·  {planet.region}` | Strong |
| 2 | `{opponent.name}  —  {opponent.difficulty}` | Normal |
| 3 | `Series  {p} – {o}   ·   first to {n}` | Normal |
| 4 | *(blank)* | |
| 5 | the action row | cursored label takes `pulse` |
| 6 | *(blank)* | |
| 7 | `←/→ choose  ·  Enter confirm  ·  b shop  ·  c deck  ·  Esc menu` | Muted |

The action row is `draw_choice_panel`'s idiom: each label drawn as `"{▸ or
space} {label}"` at a fixed stride, so the row's width does not change as the
cursor moves and the *acted-on row* is one row with an empty row above and below
it, the rest compact — the constitution's density rule, satisfied statically.
Both the row and the header rows come from pure functions
(`action_row_width()`, `series_line(series)`) so the fit is tested without a
terminal.

The rail, when `Some`: `draw_box` around `rail.art` (Single, Muted) with the
planet's name centered inside it, Muted — the plain placeholder of ruling M1;
then `draw_presence_panel(frame, rail.portrait, opponent.name,
opponent.portrait)`, the same drawer the map's rail and the select screen use. A
bordered region with one muted label rather than an empty box, so it reads as a
reserved placeholder rather than a rendering fault (§Open questions 2).

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

- `assets/primer_text.txt`, after the Outfitter paragraph, a blank row and:
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

## Files

- `src/campaign.rs` — `Series`, `SeriesOutcome`, `FINAL_OPPONENT`,
  `wins_needed`, `series_length_label`, `NodeRef::settled`, `take_settlement`
  (replacing `take_stake`), `series`/`begin_series`/`record_series_match`; tests.
- `src/economy.rs` — `reserve_floor`; `cheapest_floor` made private; tests.
- `src/profile.rs` — `Settlement::series`, `settle_campaign_match`,
  `resolve_match`, `is_broke`, `can_afford`; tests (the `Settlement` literals
  gain a field, the two `economy::cheapest_floor` calls at ~901 and ~1274 move,
  and the settlement helpers learn to play a whole series).
- `src/shop.rs` — one line (`reserve_floor`).
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
- `README.md` (**T009**) — ~80–81 says "Launching a match opens a **wager
  prompt**", which is no longer what launching an un-beaten planet does.
- `src/layout.rs` — `VENUE_ART_W`, `VENUE_PANEL_H`, `VenueRail`, `VenueLayout`;
  tests.
- `src/venue.rs` (new) + `src/lib.rs` (one `pub mod` line) — the screen; tests.
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
  expected line count).
- `specs/029-tournament-rounds/closeout-main-docs.md` (T011).
- **No change**: `src/game.rs`, `src/card.rs`, `src/player.rs`, `src/save.rs`,
  `src/opponent.rs`, `src/wager.rs`, `src/portrait.rs`, `src/frame.rs`,
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
  the same profile with no series is not broke; and `can_afford(20)` is false in
  the first and true in the second, so the Outfitter reserves the locked floor.
- **Every campaign door lands on the venue iff a series is in progress** (AC 9,
  and the bundle's point 2) — **not a unit test, deliberately.** The mapping
  itself is an `if` over a boolean and a test of it would be a tautology; the
  failure that actually costs something is a *second* assignment site elsewhere
  in `app.rs`, which no mapping test can see. So this rule is pinned by a
  mechanical check in T006's Verify and at the Phase 2 review —
  `grep -n "self\.screen = Screen::\(CampaignMap\|Venue\)" src/app.rs` returning
  exactly two lines, both inside `open_campaign_home` — and attested in the
  Phase 2 walkthrough, which presses every door.
- **The venue fits both layouts, and the rail is wide-only and non-overlapping**
  (AC 16) — T004 `layout.rs`: at 89×31 `rail` is `None` and the block is
  on-frame; at 139×31 `rail` is `Some`, `art.x1 + PANEL_GAP < portrait.x0`, both
  Rects are on-frame, their tops and bottoms match, and the widest venue text
  row centered on `center_x` ends left of `art.x0`.
- **The venue's rows read right and breathe** (AC 3, 17) — T005 `venue.rs`:
  `the_action_row_keeps_its_width_as_the_cursor_moves` (all four selections give
  the same width, and exactly one carries `▸`); `the_venue_block_breathes_only
  _around_the_action_row` (row 4 and row 6 blank, rows 0–3 non-blank, no other
  blank); `the_venue_text_fits_the_minimum_terminal` (every row, at both
  `Config::fit_sizes()`, centered on `VenueLayout::center_x`, lands inside the
  frame and — at 139 — left of the rail); `series_line` reads
  `Series  1 – 0   ·   first to 2` and `first to 3` for the final opponent.
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
  - **After the Phase 2 review**, at 89×31 and again at 139×31: from a fresh
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
2. **The art placeholder carries the planet's name** (§Design 5). Ruling M1 says
   a plain placeholder; an empty bordered box reads as a rendering fault at a
   walkthrough, and one muted label reads as a reservation. **Sign-off: the
   person's to rule, and the walkthrough is the moment — stands as drafted
   provided the Phase 2 pause report names it as an open choice.** Note the
   venue's second row already names the planet six rows above the region, so the
   label repeats a fact. One line to empty it.
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
