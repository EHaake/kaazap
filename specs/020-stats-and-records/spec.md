# Spec 020 — Stats & records

**Status**: Draft — pending review
**Depends on**: spec 008 (profile / collection), spec 009 (campaign run),
spec 011 (roster), spec 012 (economy — for `reset_to_starter` from spec 014).

## Summary

A persistent **records** layer: the game now remembers how you've done and
shows it on a dedicated **Records** screen off the start menu. It tracks
match and round win/loss **per opponent**, a **win streak**, **collection
completion**, and **campaign completions**, split by mode (Quick Play vs
Campaign) with a combined overall view and a separate current-campaign view.
It's a "mastery" layer — a reason to keep playing once the campaign is
cleared — built on the existing `profile.json` save, changing no mechanic.

## Why

The game already produces a rich stream of outcomes every session and throws
all of it away the moment a match ends. There's no way to see whether you're
getting better, which opponent keeps beating you, or how much of the card
universe you've collected. A lightweight, persistent record turns that
discarded history into a sense of progress and mastery that outlives any one
campaign run. It's cheap to build: the match-end and round-end transitions
are already observable in `App::tick` (the same seams the economy reward and
the play log hang on), and `profile.json` is already the player-owned save
that additive fields extend without a version bump (as `campaign` and
`credits` did).

## User-facing behavior

### Getting there

- A new **Records** item on the start menu opens a full-screen **Records
  `Screen`** (a mode you navigate *to*, in the vocabulary of `CLAUDE.md` —
  built by copying `opponent_select.rs`, not bolted onto an existing screen).
- The screen is **read-only** — it displays; it never edits the profile.
- `Esc` (and the usual back key) returns to the start menu, with the menu's
  selection preserved.
- Monochrome, bordered, in the spec 002 visual vocabulary, and legible at the
  minimum terminal size (139×31).

### What it tracks

For every **completed** match (one that reaches game over — first-to-3 round
wins, so a match always has a definite winner and never ties) and every
**completed** round within it, the game records the outcome. Both **Quick
Play** and **Campaign** matches count. A match abandoned before game over
(quit to menu mid-match) records nothing.

The tracked quantities:

- **Match wins / losses, per opponent** — keyed by roster opponent.
- **Round wins / losses, per opponent** — sub-match granularity (winning a
  match 3–2 records 3 round-wins and 2 round-losses against that opponent).
- **Win streak** — consecutive **match** wins; any match loss breaks it.
  Tracked as a live *current* streak and an all-time *longest*.
- **Collection completion** — how much of the 15-card side-card universe you
  own, as "N of 15 types" and a percentage (derived live from the profile's
  collection; not separately stored).
- **Campaign completions** — how many times you've finished the campaign
  (reached the final node cleared).

### The two time scopes

Records exist at two scopes, which map onto the two things New Campaign
(`reset_to_starter`, spec 014) does and does not clear:

- **Lifetime / career** — accumulates across everything and **survives New
  Campaign**. This is the mastery record. (`reset_to_starter` is amended to
  preserve it — the one behavior change to existing code, analogous to how
  Settings already survive a reset by living in a separate file.)
- **Current campaign (this run)** — only the campaign matches of the *active*
  run, and **wiped when a new campaign begins**. It rides on the campaign
  run's own state, so New Campaign clears it for free.

### What the screen shows

The screen presents four **views**, navigable with left/right (and the emacs
`Ctrl+B`/`Ctrl+F` mirrors, per spec 006), with the current view named:

1. **Overall (lifetime)** — Quick Play + Campaign combined: total matches,
   wins, losses, **win rate %**, current and longest win streak, and the
   per-opponent match/round breakdown.
2. **Quick Play (lifetime)** — the same, restricted to Quick Play matches.
3. **Campaign (lifetime)** — the same, restricted to campaign matches, plus
   the **campaign-completions** count.
4. **This Run (current campaign)** — matches, wins, losses, win rate, and the
   current run's streak for the active campaign only. Empty (a clear "no
   matches this run yet" state) before the first campaign match of a run.

A persistent line (header or footer, present across all views) shows
**collection completion** ("Cards: N of 15 — P%").

Per-opponent breakdowns list roster opponents by **name** with their match
record (W–L) and round record (W–L). An opponent you've never faced reads as
a dash / zeroes rather than being hidden, so the roster you *haven't* beaten
is visible as unfinished business. If a view's content exceeds the screen, it
**scrolls** (reusing the scroll interaction from spec 019).

### Empty / first-run state

A brand-new profile has no records: every count is zero, streaks are zero,
collection completion reflects the starter collection, campaign completions is
zero, and "This Run" shows its no-matches-yet state. The screen must read
cleanly in this state, not blank or broken.

## Entities

Conceptual — the concrete types live in `plan.md`.

- **Opponent record** — for one opponent: match wins, match losses, round
  wins, round losses.
- **Mode record** — a set of opponent records: one mode's per-opponent
  tallies. Two of these persist in the profile: Quick Play and Campaign. A
  mode record carries **no streak of its own** — streaks are tracked only at
  the overall-lifetime and current-run scopes (see Non-goals).
- **Overall streak** — current and longest match-win streak across *all*
  matches regardless of mode; tracked in its own right because an interleaved
  Quick-Play/Campaign match sequence can't be reconstructed by combining
  mode-level tallies.
- **Campaign completions** — a lifetime counter.
- **Current-run record** — the active campaign run's own tally (matches
  won/lost, rounds won/lost, current/longest run streak), living with the
  campaign run so a reset clears it.
- **Collection completion** — not stored; computed from the existing
  collection against the 15-card universe (`card::ALL_SIDE_CARDS`).

All persisted fields are additive and serde-defaulted on the existing
`profile.json` — **no `PROFILE_VERSION` bump**, exactly as `campaign` and
`credits` were added.

## Key user flows

### Recording a match

You finish a match (win or lose) against an opponent. On reaching game over,
exactly once: the opponent's match W/L is incremented in the relevant lifetime
mode record; the overall streak advances (a win) or resets (a loss); if it was
a campaign match, the current-run record updates too; and if that win cleared
the campaign's final node, campaign-completions increments. The round tallies
were already accumulated as each round of the match resolved. The profile is
saved.

### Recording a round

Each time a round resolves (a side busts, stands out, or fills the table),
the winning/losing side's round tally is credited against the current
opponent, in the same mode buckets as the match. (A tied round credits
neither — matches are what's first-to-3, and a tie replays.)

### Viewing records

From the start menu you open **Records**, land on the **Overall** view, page
left/right through the four views, scroll a long view if needed, and press
`Esc` to return to the menu where you left off.

### Starting a new campaign

You pick New Campaign. Your **lifetime** records and campaign-completions are
untouched; the **current-run** record resets to empty along with the rest of
the run. The next campaign match begins a fresh "This Run" tally.

## Non-goals (explicitly deferred)

- **Achievements / unlocks / rewards tied to stats** — records are
  informational only; nothing in gameplay or the economy reads them. Tying
  progression to milestones is a separate design question (and interacts with
  the stakes/roguelike work), deferred.
- **Time-series / graphs / per-session history** — no charts, no "last 10
  matches," no dated log. The tracked quantities are running aggregates, not a
  timeline; a history view is bigger scope than this mastery layer.
- **An in-app "clear records" control** — no UI to wipe lifetime stats.
  Lifetime records are meant to be permanent; the only reset is New Campaign
  clearing the current-run scope. (Deleting `profile.json` remains the manual
  escape hatch, as for any profile state.)
- **Recording abandoned/quit matches** — only matches that reach game over
  count. A partial match has no defined winner, and counting quits would let a
  losing position be erased by quitting. Deferred as simply out of scope.
- **Per-mode (Quick Play / Campaign) streaks as separate displayed stats** —
  the two streaks shown are the lifetime **overall** streak and the current
  **run** streak, the two that read as meaningful. A pure per-mode streak adds
  state for little payoff; deferred unless it's later wanted.
- **Stats for the generic fallback opponent** — the roster (spec 011) is the
  set of real opponents; the `default` fallback profile isn't something you
  face through normal play, so it isn't broken out.

## Design requirements

- The screen is usable and correct at the **minimum terminal (139×31)** —
  four views + a persistent completion line + a 10-opponent breakdown won't
  all fit at once, which is why the views are paged and long content scrolls
  rather than being crammed.
- The **empty/first-run state** is a first-class case, not an afterthought: a
  fresh profile's Records screen reads cleanly with zeroes and dashes.
- The **per-opponent breakdown** shows the whole roster, faced or not, so the
  screen doubles as a "who's left" view.
- The most-looked-at numbers — win rate and streak on the Overall view — are
  the ones to make immediately legible.
- Monochrome by construction; no color path (per `design/brief.md`).

## Acceptance criteria

- [ ] A **Records** item on the start menu opens a full-screen Records screen;
      `Esc` returns to the menu with its selection preserved.
- [ ] Finishing a match against an opponent updates that opponent's match W/L,
      for both Quick Play and Campaign matches; the change persists across a
      restart (written to `profile.json`).
- [ ] Round outcomes within a match are recorded per opponent as round W/L
      (e.g. a 3–1 match win shows 3 round-wins, 1 round-loss against that
      opponent); a tied round credits neither side.
- [ ] The win streak counts consecutive match wins and resets to zero on a
      match loss; both the current streak and the all-time longest are shown.
- [ ] The Records screen shows four views — Overall, Quick Play, Campaign,
      This Run — navigable left/right (and the emacs mirrors); Overall is the
      combined Quick Play + Campaign totals.
- [ ] Each view shows matches played, wins, losses, and win rate %; the
      Campaign view additionally shows campaign completions; the This Run view
      reflects only the active campaign run.
- [ ] A persistent line shows collection completion as "N of 15" and a
      percentage, matching the profile's actual collection.
- [ ] Finishing the campaign (final node cleared) increments campaign
      completions; the count persists across restarts and is **not** reset by
      New Campaign.
- [ ] New Campaign leaves lifetime records and campaign completions unchanged
      while clearing the This Run tally.
- [ ] A brand-new profile's Records screen renders cleanly (zeroes, dashes, a
      no-matches This Run state) and is correct at the 139×31 minimum terminal;
      long views scroll.
- [ ] All new logic (recording, streak transitions, derived totals/win-rate,
      collection completion, reset preservation) is unit-tested; `cargo test`
      passes.

## Resolved decisions

- **Both modes count, split + combined (A).** Quick Play and Campaign each get
  their own per-opponent record; Overall is their combination; a separate
  This Run view covers the active campaign. Per-opponent totals combine by
  summing the two mode records; the Overall streak is tracked independently
  (an interleaved sequence can't be recombined from the mode streaks).
- **All four roadmap metrics kept, plus additions (B).** Added: **win rate %**
  (derived), **current streak shown beside longest** (free — tracked anyway),
  and **campaign completions** (a real replay/mastery signal, cheap). Left out:
  overall lifetime totals as their own stored numbers (derived), per-mode
  streaks (see non-goals).
- **Round-level recording included (B).** Round W/L is tracked per opponent in
  addition to match W/L — a better skill signal than match record alone. It's
  the one quantity needing a round-resolution recording point rather than only
  the match-end one; ruled worth it.
- **A new `Records` Screen, not a campaign-map panel (C).** A full mode you
  navigate to is a `Screen` per `CLAUDE.md`; the map panel would be cramped and
  mix concerns. Reached from a new start-menu item.
- **Lifetime records survive New Campaign; a current-run record is separate
  (D).** Career records are the point of a mastery layer, so `reset_to_starter`
  is amended to preserve the lifetime stats field. The current-run scope rides
  on the campaign run, which reset already clears — so the split needs no new
  reset logic beyond preserving the lifetime field.
- **Streak = consecutive match wins, any loss breaks it (E).** Match-level
  only; rounds have no streak.
- **Menu placement / exact per-view layout / paging vs. scrolling within a
  view are UX details** left to `plan.md` and the implementer, within the
  design requirements above (paged views, scroll long content, legible at
  139×31, whole-roster breakdown, clean empty state).
