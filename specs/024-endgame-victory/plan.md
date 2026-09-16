# Plan: Endgame, victory & what you keep — spec 024

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

## Context

Six changes around one principle (*what you earn is yours*): a **victory
notice** over the map when a run completes, a **run summary** shared by that
notice and the run-over notice, a **first-clear record** on the Records
Campaign view, a **completed marker** on the map header, **Quick Play dealing
the built deck**, and **New Campaign keeping the pool** with a separate
**Reset Everything**.

It is a **UI + profile-data spec**. The engine (`game.rs`, `player.rs`,
`card.rs`), the AI, `save.rs`, `economy.rs`, `wager.rs` and the balance data
are untouched; `tests/balance.rs` is not edited. The profile gains two
run-tally counters and one optional lifetime number, all `#[serde(default)]`;
`PROFILE_VERSION` and `SAVE_VERSION` stay 1. No new crate.

The only structural change is at the **match-resolution seam**: `App::tick`'s
two profile calls become one `Profile::resolve_match`, so the order the
first-clear record depends on lives inside one tested method rather than in
`app.rs` (tension §1).

## What the code already gives us

- **One modal at a time, routed before any screen.** `Modal` (`app.rs:270`) is
  `Option<Modal>` on `App`; `handle_key` (`app.rs:915`) walks an `else if
  matches!(self.modal, …)` chain and reaches the screen arms only when no modal
  is open. `Modal::RunOver` is the unit-like pattern — a pure
  `run_over_acknowledged` (`app.rs:351`), a `handle_run_over_input`
  (`app.rs:637`), a draw arm (`app.rs:1589`) and a bordered box built by hand in
  `draw_run_over` (`app.rs:1754`). The victory notice copies it.
- **`OverlayLayout::new(config, content_width, content_height)`** sizes a box to
  `content_width + 2·H_PAD` × `content_height + V_PAD` and insets `inner` by
  `V_PAD / 2 = 2` — border row plus exactly one blank row above and below the
  content (`layout.rs:679-721`, `lib.rs:39-43`). The design brief's even box
  padding comes free; a dismiss line on the last content row therefore already
  has its blank row below it, and needs one written above it.
- **Settlement already owns the completion edge.** `Profile::settle_campaign_match`
  (`profile.rs:244`) takes the stake out of escrow, pays `economy::win_payout`,
  calls `mark_beaten`, and counts a completion on the `!was_complete &&
  run_complete()` edge — so a rematch never re-counts (spec 021). It is called
  from exactly one place, `App::tick`'s `GameOver` block (`app.rs:1538`), and so
  is `Profile::record_match` (`app.rs:1541`). **`App::tick` calls settle first,
  then record**, while `profile.rs`'s own tests record first, then settle — the
  two are independent today, which is why the discrepancy is invisible.
- **The run tally lives on the run.** `CampaignRun.run_stats: RunStats`
  (`campaign.rs:170`) is a plain serde-defaulted field, so anything added to
  `RunStats` zeroes for free whenever the run is reset.
- **The banner already reports the net gain.** `banner_line`
  (`campaign_map.rs:309`) shows `win_payout(stake) − stake` on a win and the
  forfeited stake on a loss — the two numbers the run summary must accumulate.
- **The map header's second row is already a two-way branch.** `draw_header`
  (`campaign_map.rs:245`) draws the banner when one is showing, else the
  `Outer Rim → The Core` axis label. It also counts cleared planets inline.
- **The Records breakdown table is anchored by a fixed 5th summary row.**
  `view_body` (`records.rs:269`) pushes exactly one 5th line per breakdown view
  (streak / completions / blank), pinned by
  `by_opponent_table_is_anchored_across_breakdown_views`. Folding the first-clear
  number **into** the completions line keeps that anchor by construction.
- **Two confirms and the entry panel share one drawing fn.** `draw_two_choice`
  (`app.rs:1651`) draws title, optional Muted note, a centered two-label choice
  row at inner row 2, and a hint at row 4 (`content_height` 5).
  `ConfirmNewCampaign`'s Commit is guarded by the tested pure
  `confirm_choice` (`app.rs:330`) — spec 014's irreversible-wipe seam.
- **`reset_run`** (`app.rs:678`) is the one reset: `Profile::reset_to_starter`
  + save + `save::clear()` + `has_save = false` + banner cleared. Both the
  run-over acknowledgement and `start_new_campaign` (`app.rs:691`) call it.
- **`player_deck_for(is_campaign, built)`** (`app.rs:432`) is the whole Quick
  Play deck decision, and `DEFAULT_SIDE_DECK` (`app.rs:14`) and `stats::Mode`
  (`app.rs:35`) are imported into `app.rs` for exactly one use each
  (`app.rs:436`, `app.rs:1534-1536`).
- **Every campaign map entry funnels through `enter_campaign_map(from_menu)`**
  (`app.rs:624`), which opens the map and assigns `self.modal =
  map_entry_modal(broke, primer_due)` — unconditionally, since all four callers
  have no modal open at the call (spec 023 Phase 2 review). Shop / builder Backs
  keep `open_campaign_map`.
- **Tests never touch disk.** No test calls `Profile::save` or `save::save`;
  `start_match`, the game-over settlement and every mark save the profile. So
  App-level tests must stay on paths that write nothing (spec 023 tension §4) —
  which is why every decision below is a pure fn.

## Design tensions resolved

### 1. The first-clear number forces an order — so one method owns it

The record is "the run's matches played **including** the completing match",
captured on the completion edge inside `settle_campaign_match`. But the run
tally is bumped by `record_match`, which `App::tick` calls *after* settling —
so today's order would record one match short, while `profile.rs`'s own tests
(which record first) would read it correctly. A latent disagreement between the
app and its tests is exactly what spec 021 removed when it moved the completion
edge into one method.

**Resolution**: one `Profile::resolve_match(opponent_id, player_won,
player_rounds, opp_rounds) -> Option<Settlement>` that records then settles,
with `record_match` and `settle_campaign_match` becoming private to `profile.rs`
(each has exactly one caller in `app.rs` today). `resolve_match` also derives
the `Mode` itself from `in_progress` — the same expression `app.rs` computes
today — so the app no longer needs `stats::Mode`. `App::tick`'s block becomes
one call plus the banner and the completion flag. The ordering is then internal,
documented, and unit-tested in `profile.rs` through `resolve_match` (which is
also the order the existing `win_node` test helper already uses). Rejected
alternatives: a `+ 1` inside the edge (correct only under the app's order,
silently wrong under the tests'), and keeping two app-level calls plus a third
`record_first_clear` (an ordering rule spanning three calls, unverifiable
without an App that writes to disk).

### 2. The completion signal reaches the notice as a one-shot App flag

The notice is raised on the **acknowledgement** of the game-over popup, one key
event after the settlement that completed the run. `Settlement.completed_run`
(the `!was_complete && run_complete()` edge) is therefore stored on `App` as
`victory_due: bool` at settlement and **taken** (`mem::take`) by the next
`enter_campaign_map`, whose only reachable caller in that window is the
acknowledgement. Consequences, all spec'd: a rematch win sets nothing; a
replayed campaign's completing win sets it again; quitting before
acknowledging loses the notice but not the completion or the payout (both are
already persisted). Not persisted, by design — a flag on disk would be a
save-format change for a transient.

### 3. Both notices are one box with one content builder each

`draw_run_over` becomes `draw_notice(&self, frame, &[String])`: row 0 Strong
(title), the last row Muted (the dismiss line), the rest Normal, all centered,
`OverlayLayout::new(config, widest, lines.len())`. Two pure builders —
`run_over_notice_lines` and `victory_notice_lines` — assemble the full content
including the blank separator rows, so the exact text, the row count, the
breathing room and the fit are all testable without a terminal. Centering every
row (rather than left-aligning the summary block) keeps the existing notice's
look and avoids re-deriving the T006a centering problem.

### 4. The three-choice panel generalizes `draw_two_choice`, and fixes its
breathing room when a note is present

`draw_two_choice` becomes `draw_choice_panel(frame, title, note, labels:
&[&str], selected: usize, hint, pulse)`; the two confirms pass two labels and
`selected = 0 | 1`, the entry panel passes three. The choice row is the acted-on
element, so it gets a blank row above and below: **without** a note the rows stay
as today (title 0, choices 2, hint 4, height 5); **with** a note the note takes
row 1 and the choices move to row 3, hint row 5, height 6 — otherwise the note
would sit flush against the acted-on row, which the design brief forbids and
which this spec's own New Campaign confirm would show whenever a stake is in
flight. This is a correction inside a screen this spec changes, not new scope.

### 5. `player_deck_for` is deleted rather than kept with one branch

With Quick Play dealing the built deck there is one answer for both modes, so
the fn, its `is_campaign` argument, and the `DEFAULT_SIDE_DECK` / `Mode`
imports in `app.rs` all go; `start_match` deals `self.profile.deck().to_vec()`.
Keeping a one-line wrapper would be indirection the spec doesn't demand
(constitution: *Simplicity*). The cost is that the claim loses its pure-fn test:
`start_match` writes the profile and the save, so no disk-free App test can
assert the deal. It is pinned structurally instead — `grep` shows `app.rs` no
longer names `DEFAULT_SIDE_DECK` at all, so it *cannot* deal the standard deck
— plus the Phase 3 driver check with a non-standard built deck. Flagged in
§Tests and in §Open questions.

### 6. New Campaign resets the map through the profile, reusing the reset tail

`Profile::reset_campaign_run` is `self.campaign = CampaignRun::default()` —
beaten set, in-flight pointer *and its escrowed stake*, and the run tally all
go; credits, collection, deck, lifetime stats and the onboarding marks are
untouched by construction (nothing else is written). The app-side tail both
resets share (clear the match save, `has_save = false`, drop the stale banner)
is pulled out of `reset_run` as `discard_match_and_banner`, and one
`start_fresh_campaign(scope)` plays the cue and calls `enter_campaign_map(true)`
for both scopes — so the broke check the spec asks for after New Campaign is
the existing one, not a new path.

### 7. The `Reset Everything` confirm is retitled

`spec.md` quotes its title as `… Erases progress, credits & cards.` — the tail,
after the leading ellipsis. Under the new three-choice panel the current
head ("New campaign?") would name the wrong choice, so the title becomes
**`Reset everything? Erases progress, credits & cards.`** Flagged as a wording
decision, not a behavior change; the person can edit it in T007.

## Design

### 1. `src/stats.rs` — the counters, the record, and the shared summary

```rust
pub struct RunStats {
    … existing fields …
    /// Credits won this run (spec 024): the **net gain** on each settled win
    /// (`win_payout(stake) − stake` — the number the map banner reports).
    /// Additive and serde-defaulted, so a pre-024 run loads with zero, and
    /// zeroed with the run because the tally lives on `CampaignRun`.
    #[serde(default)] pub credits_won: u32,
    /// Credits lost this run (spec 024): each forfeited stake on a settled
    /// loss. A stake forfeited by discarding a saved match settles nothing
    /// and counts toward neither counter.
    #[serde(default)] pub credits_lost: u32,
}

impl RunStats {
    /// Matches played this run — wins plus losses.
    pub fn matches_played(&self) -> u32
    pub fn record_credits_won(&mut self, amount: u32)   // saturating
    pub fn record_credits_lost(&mut self, amount: u32)  // saturating
}

pub struct LifetimeStats {
    … existing fields …
    /// Matches played in the run that produced this profile's **first**
    /// campaign completion (spec 024). Set once, on the completions 0 → 1
    /// edge, and never changed — a profile that had already completed a run
    /// before this spec has no record and never gains one. Serde-defaulted
    /// to `None`.
    #[serde(default)] first_clear_matches: Option<u32>,
}

impl LifetimeStats {
    pub fn first_clear_matches(&self) -> Option<u32>
    /// Record a completed campaign run. `matches_played` is the run's match
    /// count *including* the completing match; on the profile's first
    /// completion — completions still 0 — it becomes the first-clear record.
    pub fn record_campaign_completion(&mut self, matches_played: u32) {
        if self.campaign_completions == 0 {
            self.first_clear_matches = Some(matches_played);
        }
        self.campaign_completions += 1;
    }
}

/// The run summary both notices show (spec 024): one block, defined once and
/// rendered twice. Pure, so the wording is testable without a terminal; the
/// world counts are passed in because the map graph is not a stats concern.
pub fn run_summary_lines(
    run: &RunStats,
    worlds_cleared: usize,
    worlds_total: usize,
) -> [String; 3]
```

The three lines, exactly:

```
Matches played {played}  ·  won {wins}  ·  lost {losses}
Credits won {won}  ·  lost {lost}
Best streak {longest}  ·  Worlds cleared {cleared}/{total}
```

### 2. `src/campaign.rs` — one derived count

```rust
/// How many planets are cleared — the map header's progress figure and the
/// run summary's "worlds cleared" (spec 024). Derived, never stored.
pub fn worlds_cleared(&self) -> usize
```

`campaign_map.rs`'s `draw_header` switches its inline `filter(...).count()` to
this. Nothing else in `campaign.rs` changes.

### 3. `src/profile.rs` — one resolution seam, one map-only reset, one predicate

```rust
/// How a finished campaign match settled (spec 024).
pub struct Settlement {
    pub outcome: StakeOutcome,
    /// Whether this settlement was the win that **completed the run** — the
    /// `!was_complete && run_complete()` edge, so a rematch is never one and a
    /// replayed campaign's final win is. The once-per-completion signal the
    /// victory notice rides on.
    pub completed_run: bool,
}

/// Resolve a finished match: record it into lifetime + run statistics, then
/// settle any campaign stake. One method owns the **order** (spec 024): the
/// first-clear record is captured on the completion edge from the run tally,
/// so the completing match must already be recorded when settlement runs.
/// `Mode` is derived here from the in-flight pointer — settlement never clears
/// it, so reading it first or last is the same answer. Returns `None` for a
/// Quick Play match (recorded, nothing to settle). Callers pair this with
/// `save`.
pub fn resolve_match(
    &mut self,
    opponent_id: &str,
    player_won: bool,
    player_rounds: u32,
    opp_rounds: u32,
) -> Option<Settlement>

/// Reset the campaign map only — spec 024's New Campaign: the beaten set, the
/// in-flight pointer (and its escrowed stake) and the run tally all go with
/// `CampaignRun::default()`; credits, collection, deck, lifetime stats and the
/// onboarding marks are untouched. Supersedes spec 014's "New Campaign = full
/// fresh start"; `reset_to_starter` is now reached only by Reset Everything
/// and the run-over acknowledgement. The caller persists and clears any match
/// save.
pub fn reset_campaign_run(&mut self)

/// Whether anything the campaign-entry choices would affect exists (spec 024):
/// the run has progress, or the pool differs from the starter — credits,
/// collection or deck. A truly fresh profile opens the map directly.
pub fn differs_from_starter(&self) -> bool
```

`record_match` and `settle_campaign_match` lose their `pub` (T003, once
`app.rs` no longer calls them). Inside `settle_campaign_match`:

- a win adds `win_payout(stake).saturating_sub(stake)` to
  `run_stats.credits_won`; a loss adds `stake` to `credits_lost` — both before
  the win/loss branch returns, both zero-safe for a stake-0 pointer;
- the completion edge passes `self.campaign.run_stats().matches_played()` to
  `record_campaign_completion` and the method returns
  `Settlement { outcome, completed_run }`.

### 4. `src/app.rs` — the victory notice, the three-choice entry, the deal

```rust
/// The victory notice (spec 024): raised over the campaign map when the player
/// acknowledges the game-over popup of the match that completed the run. Unit-
/// like — its content is rebuilt from the run tally on each draw. Enter, Space
/// or Esc dismiss it; nothing else acts while it is up. Transient: no seen
/// mark, and quitting under it loses only the notice.
Modal::Victory,
/// Shown at campaign entry when there is anything the choices would affect:
/// Continue / New Campaign / Reset Everything (spec 024, superseding spec
/// 014's two-choice panel). `choice` is the highlighted one, defaulting to
/// Continue — the safe option.
Modal::CampaignEntry { choice: CampaignChoice },
/// The destructive confirmation behind New Campaign (map only) and Reset
/// Everything (the full wipe) — `scope` says which. Default-No (spec 014's
/// tested `confirm_choice` seam, unchanged).
Modal::ConfirmReset { on_yes: bool, scope: ResetScope },

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CampaignChoice { Continue, NewCampaign, ResetEverything }
impl CampaignChoice {
    const ALL: [Self; 3];
    fn label(self) -> &'static str;      // "Continue" / "New Campaign" / "Reset Everything"
    fn step(self, forward: bool) -> Self; // wraps, so n = 2's old toggle behaviour generalizes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResetScope { MapOnly, Everything }

/// Whether a key dismisses a notice the player may wave away — the primer, the
/// first-match popup (spec 023) and the victory notice (spec 024). Enter,
/// Space and Esc; every other key is swallowed. (Renamed from
/// `onboarding_dismissed`, which now under-describes it.)
fn notice_dismissed(key: KeyCode) -> bool

/// What a freshly opened campaign map raises (specs 021 + 023 + 024): the
/// run-over notice if broke, else the victory notice if a completion is
/// waiting, else the primer if it is due, else nothing. A completing win can
/// never leave the player broke and the primer is menu-entry only, so the
/// precedence never actually arbitrates — it is written down so it can't
/// drift.
fn map_entry_modal(broke: bool, victory_due: bool, primer_due: bool) -> Option<Modal>

/// The victory notice's content, title first and dismiss line last, blank rows
/// included (spec 024).
fn victory_notice_lines(run: &RunStats, cleared: usize, total: usize) -> Vec<String>
/// The run-over notice's content, now carrying the same run summary between
/// its title and its reset note.
fn run_over_notice_lines(run: &RunStats, cleared: usize, total: usize) -> Vec<String>
/// The width of a choice row: the labels, their markers, and the gaps between.
fn choice_row_width(labels: &[&str]) -> usize
```

Contents, in order (blank strings are blank rows):

```
victory_notice_lines                       run_over_notice_lines
0  Campaign complete — the house's best     0  You're broke — the run is over.
   has lost.                               1
1                                          2  Matches played …
2  Your cards and credits are yours to      3  Credits won …
   keep.                                    4  Best streak …
3  Rematches stay open; New Campaign        5
   replays the map with your deck.          6  Deck, collection, and progress
4                                              reset to the starter; your
5  Matches played …                             records stay.
6  Credits won …                            7
7  Best streak …                            8  Enter  continue
8
9  Enter  continue
```

(10 and 9 content rows → 14- and 13-row boxes, widest line 65 and 70 chars →
73- and 78-column boxes: unclamped at 139×31.)

Wiring:

- **`App` gains `victory_due: bool`** (false in `new`). In `tick`'s `GameOver`
  block, the two profile calls collapse to
  `if let Some(s) = self.profile.resolve_match(opponent_id, player_won,
  player_rounds, opp_rounds) { self.banner = Some(MapBanner::Settled(s.outcome));
  self.victory_due |= s.completed_run; }` followed by the existing
  `self.profile.save()`. The block's comment is rewritten; `mode` and the
  `stats::Mode` import go.
- **`enter_campaign_map`** takes `victory_due` from the flag and passes it:
  `let victory_due = std::mem::take(&mut self.victory_due);` then
  `self.modal = map_entry_modal(self.profile.is_broke(), victory_due,
  from_menu && !self.profile.primer_seen());`. Its four call sites are
  unchanged.
- **`handle_key`'s modal chain** gains one branch after `RunOver`:
  `Some(Modal::Victory)` → if `notice_dismissed(key)` then `modal = None` and
  `Sfx::MenuSelect`, else nothing. `draw` gains
  `Some(Modal::Victory) => self.draw_notice(frame, &victory_notice_lines(…))`,
  and the `RunOver` arm switches to `draw_notice` with
  `run_over_notice_lines(…)`. Both read
  `self.profile.campaign().run_stats()`, `…campaign().worlds_cleared()` and
  `campaign::PLANETS.len()`.
- **`draw_run_over` is replaced** by `draw_notice(&self, frame, lines:
  &[String])` (tension §3).
- **Campaign entry**: `activate_menu_item`'s `StartCampaign` arm gates on
  `self.profile.differs_from_starter()` instead of
  `campaign().has_progress()`. `handle_campaign_entry_input` moves the
  highlight with `CampaignChoice::step` on ←/→/a/d, commits on Enter/Space —
  `Continue` → `enter_campaign_continue()`, `NewCampaign` → `ConfirmReset {
  on_yes: false, scope: MapOnly }`, `ResetEverything` → `ConfirmReset {
  on_yes: false, scope: Everything }` — and closes on Esc, exactly as today.
- **`handle_confirm_reset_input`** is the renamed
  `handle_confirm_new_campaign_input`, unchanged except that `Commit` reads the
  scope and calls `start_fresh_campaign(scope)`.
- **The resets**: `discard_match_and_banner` (save cleared, `has_save = false`,
  banner `None`) is pulled out of `reset_run`; `reset_map_only` =
  `profile.reset_campaign_run()` + `profile.save()` + that tail;
  `start_fresh_campaign(scope)` picks the reset, plays `Sfx::MenuSelect` and
  calls `enter_campaign_map(true)` — replacing `start_new_campaign`.
  `reset_run` itself is unchanged and still serves the run-over path.
- **The deal**: `player_deck_for` is deleted; `start_match`'s
  `GameState::with_opponent(opponent, self.profile.deck().to_vec())` is the one
  deal for both modes, with a comment citing spec 024. `is_campaign` goes; the
  fn's doc drops the "Quick Play deals the standard deck" paragraph and keeps
  the deck-valid precondition (which now binds Quick Play again, via the
  existing `open_opponent_select` divert).
- **`draw_choice_panel`** (tension §4) replaces `draw_two_choice`;
  `draw_confirm_new_game` passes `["Yes", "No"]`, `draw_confirm_reset` passes
  the scope's title (`"New campaign? Resets the map; you keep your cards and
  credits."` / `"Reset everything? Erases progress, credits & cards."`) with
  `stake_forfeit_note()` unchanged, and `draw_campaign_entry` passes the three
  labels with `hint: "←/→ choose  ·  Enter select  ·  Esc back"`.

### 5. `src/campaign_map.rs` — the completed marker

```rust
/// The header's second row when no banner is showing: the rim→core axis, or —
/// once the run is complete (spec 024) — a marker that stays until the map is
/// reset. Pure, so both the wording and the switch are testable.
fn axis_line(run_complete: bool) -> (&'static str, Emphasis)
```

`("★  Campaign complete", Emphasis::Strong)` when complete, else
`("Outer Rim  →  The Core", Emphasis::Muted)`. `draw_header`'s `None` branch
calls it; the banner branch is untouched, so a settled or can't-cover banner
still takes the row while it shows. The planet panel's "Campaign complete —
rematches stay open." line is unchanged.

### 6. `src/records.rs` — the first-clear line

`view_body`'s `RecordsView::Campaign` arm builds the same fixed 5th row with
the record folded in when set:

```
Campaign completions: 2  ·  first clear in 14 matches
```

and today's `Campaign completions: N` when unset. Still exactly one line, so
`by_opponent_table_is_anchored_across_breakdown_views` keeps holding; the box
widens by content measurement as it already does for every view.

### 7. `src/opponent_select.rs`, `README.md`, `docs/balance.md`

- `QUICK_PLAY_NOTE` becomes `"Quick Play deals your deck. Nothing is staked."`
  (rows and footer reserve unchanged — the note is still one Muted line at
  `y + 4`).
- `README.md` lines 82–88: New Campaign is described as resetting the map while
  keeping cards and credits, with Reset Everything named as the full wipe; the
  Quick Play sentence drops "and deals you the **standard** side deck — campaign
  matches deal the one you built" for a statement that every match deals the
  deck you built.
- `docs/balance.md` gains a short **### Replays** subsection after *### The
  economy bounds*: the measured curve describes a starter-deck run; since spec
  024 a New Campaign keeps the pool, so a replay starts premium and is easier
  than the curve assumes — opt-in, deliberately not retuned, no constant moved.

### 8. Close-out text (repo-wide files, applied on `main` after the merge)

`specs/024-endgame-victory/closeout-main-docs.md` (023's shape): **ROADMAP** —
the endgame/victory item and the "run summary on the run-over notice" backlog
item ship together. **DECISIONS** — spec 024's six resolved decisions; the
reversal of spec 022's "Quick Play deals the standard (premium) deck" and of
spec 014's "New Campaign = full fresh start" (both quoted as superseded, with
the reason: since spec 021 every match is even money, so a replay earns no
more than rematches already can, and going broke becomes the only thing that
takes the pool — which also answers spec 021's casual-versus-roguelike
question in the casual direction); tension §1 (one `resolve_match` owns the
record-then-settle order), §2 (the victory flag is transient App state), §5
(`player_deck_for` deleted), §7 (the Reset Everything title).

## Files

- `src/stats.rs` — two run counters + `matches_played`, `first_clear_matches`,
  `record_campaign_completion(matches_played)`, `run_summary_lines`; tests.
- `src/campaign.rs` — `worlds_cleared`; test.
- `src/profile.rs` — `Settlement`, `resolve_match`, the two counters bumped in
  settlement, the first-clear edge, `reset_campaign_run`,
  `differs_from_starter`, `record_match` / `settle_campaign_match` privatized;
  tests.
- `src/app.rs` — `Modal::{Victory, CampaignEntry{choice}, ConfirmReset{..}}`,
  `CampaignChoice`, `ResetScope`, `notice_dismissed`, `map_entry_modal/3`, the
  two notice builders, `draw_notice`, `draw_choice_panel`, `choice_row_width`,
  `victory_due`, the `tick` resolution block, the campaign-entry handlers, the
  reset trio, `start_match`'s deal, `player_deck_for` and the
  `DEFAULT_SIDE_DECK` / `Mode` imports removed; tests.
- `src/campaign_map.rs` — `axis_line`, `draw_header`; test.
- `src/records.rs` — the Campaign completions line; tests.
- `src/opponent_select.rs` — `QUICK_PLAY_NOTE`; test.
- `README.md`, `docs/balance.md`.
- `specs/024-endgame-victory/closeout-main-docs.md` (T010).
- **No change**: `game.rs`, `player.rs`, `card.rs`, `save.rs`, `economy.rs`,
  `wager.rs`, `shop.rs`, `deck_builder.rs`, `menu.rs`, `layout.rs`, `frame.rs`,
  `render.rs`, `audio.rs`, `banter.rs`, `play_log.rs`, `overlay.rs`,
  `board.rs`, `opponent.rs`, `settings.rs`, `assets/*`, `tests/balance.rs`,
  `docs/economy.md`, `docs/opponents.md`, `Cargo.toml`, `Cargo.lock`.

## Tests

Each claim names the task that owns its check. Driver items are marked.

- **The counters and the record default, round-trip and read back** (T001):
  `RunStats::default()` → both counters 0 and `matches_played() == 0`;
  `LifetimeStats::default().first_clear_matches()` → `None`;
  `{"match_wins":2}` deserializes with both counters 0 and no record;
  `record_credits_won/lost` accumulate and saturate.
- **The first completion sets the record, later ones never do** (T001):
  `record_campaign_completion(14)` on a fresh `LifetimeStats` → completions 1,
  record `Some(14)`; a second call with 9 → completions 2, record still
  `Some(14)`; a `LifetimeStats` deserialized with `campaign_completions: 3` and
  no record → a completion bumps to 4 and the record stays `None` (the spec's
  "an existing profile never gains one").
- **The summary block reads the run** (T001): `run_summary_lines` on a tally of
  4 wins / 2 losses, 640 won / 210 lost, best streak 6, 7 of 8 worlds → the
  three exact strings, `matches played 6`, `7/8`.
- **Settlement moves the counters, and only settlement does** (T002): a staked
  win adds `win_payout(stake) − stake` to `credits_won` and nothing to
  `credits_lost`; a loss adds the stake to `credits_lost`; a second settlement
  of the same pointer (escrow emptied) adds 0 to both; staking then
  `set_in_progress(None)` without settling leaves both at 0 (the discarded
  saved match); `reset_campaign_run` and `reset_to_starter` both zero them.
- **The counters round-trip through `profile.json` and default for a pre-024
  document** (T002): dirty both, JSON → `from_json` → same values;
  `{"version":1,"collection":[],"deck":[]}` → both 0, record `None`,
  `PROFILE_VERSION == 1`.
- **`resolve_match` counts the completing match** (T002): drive a full run
  through `resolve_match` only (stake → resolve, every node) — completions 1
  and `first_clear_matches() == Some(n)` where `n` is the run tally's
  `matches_played()` at that moment, including losses along the way; a rematch
  win afterwards leaves both unchanged; after `reset_campaign_run` a second
  full run gives completions 2 and the record still `Some(n)`.
- **`resolve_match` returns the completion edge and the mode** (T002): the
  final clearing win → `Some(Settlement { completed_run: true, .. })`; a
  rematch win on the complete run → `completed_run: false`; a match with no
  pointer → `None`, and it still recorded to Quick Play lifetime stats (not to
  the run tally).
- **The map-only reset keeps the pool, the full reset doesn't** (T002): on a
  dirtied profile (progress, in-flight staked pointer, bought card, edited
  deck, credits, lifetime stats, both onboarding marks),
  `reset_campaign_run` clears progress, the pointer, the stake and the run
  tally, and leaves credits, collection, deck, lifetime stats and the marks
  exactly as they were; `reset_to_starter` still does what its existing test
  says.
- **The entry predicate** (T002): `Profile::default()` → `false`; a beaten
  node, a credit spent or earned, a granted card, a removed deck card, and a
  staked in-flight match each → `true`.
- **`worlds_cleared` counts cleared planets** (T002): fresh → 0; Cinder cleared
  → 1; a full sweep → `PLANETS.len()`.
- **Notice precedence** (T004): `map_entry_modal(true, true, true)` → `RunOver`;
  `(false, true, true)` → `Victory`; `(false, false, true)` → `Primer`;
  `(false, false, false)` → `None`.
- **Dismiss keys** (T004): `notice_dismissed` true for Enter / Space / Esc
  only (the renamed spec-023 test, extended in its doc to name the victory
  notice).
- **Both notices read right, breathe right and fit** (T004): each builder's
  exact first line, exact last line (`Enter  continue`), the row above the
  last is blank, no two consecutive blank lines, the three summary lines appear
  in order between the spec'd blocks, the run-over notice still contains its
  reset note *after* the summary; `OverlayLayout::new(min_config, widest,
  lines.len())` is unclamped (box width < 139, height < 31) for both.
- **The choice row fits** (T004 or T007 — T007): `choice_row_width(["Continue",
  "New Campaign", "Reset Everything"])` plus the widest title measured through
  `OverlayLayout` at 139×31 is unclamped; the three labels are exactly the
  spec's.
- **The highlight steps and wraps** (T007): `CampaignChoice::step` forward from
  Continue → NewCampaign → ResetEverything → Continue, and backward the
  reverse; `confirm_choice` still commits only on Enter/Space with Yes (the
  existing spec-014 test, untouched).
- **The map's completed marker** (T005): `axis_line(false)` → the axis label,
  Muted; `axis_line(true)` → `"★  Campaign complete"`, Strong. That a banner
  still wins the row is the untouched branch above it — read at the phase
  review, and shown by the driver.
- **The Records line** (T006): with the record unset, the Campaign view
  contains `Campaign completions: 0` and not `first clear`; with it set to 14
  and completions 2, the view contains `Campaign completions: 2  ·  first clear
  in 14 matches`; the existing anchored-table and completions-line tests pass
  unchanged.
- **Quick Play deals the built deck** (T008): *structural + driver* — `grep -n
  "DEFAULT_SIDE_DECK\|Mode::" src/app.rs` is empty (so the standard deck is not
  nameable there) and `start_match` has one deal expression; the existing
  `quick_play_deals_the_standard_deck_and_campaign_deals_the_built_one` test is
  deleted with `player_deck_for`. The behavioral check is the Phase 3 driver
  run with a non-standard built deck (tension §5, §Open questions 1).
- **The Quick Play line fits** (T008): `QUICK_PLAY_NOTE == "Quick Play deals
  your deck. Nothing is staked."`, and the existing
  `the_full_roster_and_footer_fit_the_minimum_terminal` still passes with the
  note and hint rows on-frame.
- **The victory notice's once-only edge, in play** — *driver* (Phase 2 pause,
  profile + saves backed up and checksum-restored): a scratch profile driven to
  the final node → the completing win's acknowledgement lands on the map with
  the notice; map keys do nothing under it; Enter dismisses and the settled
  banner is still on the header; the completed marker stays after navigating; a
  rematch win raises nothing; quitting under the notice and re-entering shows
  no notice but the completion still counted; the Records Campaign view shows
  the first-clear line.
- **No engine / AI / economy / save change** (T010 sweep): `git diff main
  --stat` lists no `game.rs`, `player.rs`, `card.rs`, `save.rs`, `economy.rs`,
  `wager.rs`, `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`;
  `PROFILE_VERSION == 1`, `SAVE_VERSION == 1`; warning count equals `main`'s.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim per the constitution.
- **Driver / person attestation** (back up + checksum-restore the real profile
  and saves first, and drive a scratch profile): **Phase 2 pause** — the
  victory notice on a completed run, the completed marker, the run-over notice
  with its summary (drive a profile broke), both at 139×31; **Phase 3 pause** —
  Start Campaign's three choices on a profile with a pool, New Campaign keeping
  cards and credits while clearing the map, Reset Everything wiping, Esc and No
  changing nothing, New Campaign on a near-empty balance meeting the run-over
  notice, and a Quick Play match dealt from a deliberately non-standard built
  deck.

## Non-goals (from spec)

No credit bonus or unique card for the win; no per-completion history or
"best replay" record; no retuning of the difficulty or economy curve for
replays and no New Game Plus scaling; no softer run-over; no per-opponent
rematch picker, card selling or other economy change; no change to the board's
game-over popup.

## Open questions

None product-level that blocks drafting. Three things are settled here as
design and flagged for the sign-off and the person:

1. **"Quick Play deals the built deck" has no unit test** (tension §5): the
   deal is inside `start_match`, which writes the profile and the save, so it
   is pinned structurally (`DEFAULT_SIDE_DECK` no longer nameable in `app.rs`)
   and by the Phase 3 driver run.
2. **The `Reset Everything` confirm is retitled** to
   `Reset everything? Erases progress, credits & cards.` (tension §7) — the
   spec quotes only the tail.
3. **The confirm panels gain a blank row above the choice row when a note is
   showing** (tension §4), correcting a standing breathing-room miss on a
   screen this spec already changes.
