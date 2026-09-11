# Spec: Wager & loss condition — spec 021

**Status**: Draft — pending review
**Depends on**: spec 012 (economy), spec 014 (New Campaign / `reset_to_starter`),
spec 020 (stats & records)

## Summary

The campaign economy is one-directional: a win pays credits and drops a free
card, a loss costs nothing, and you re-fight any wall for free. Failure has no
teeth, and a better deck is never actually *required*. This spec makes the
economy two-directional: every campaign match is played **for a stake** the
player chooses above a per-opponent **ante floor**; a win pays the stake, a loss
costs it; and **going broke ends the run** — a full reset to the starter deck
and seed purse, so a bad run costs the deck you built, not just some credits.
Cards now come only from the shop, bought with wagered credits.

It also adds **rematches** against beaten opponents, so a player can grind
small, safe stakes against easy opponents to fund cards — the lever the later
balance pass tunes against. Everything here is **tunable-constants-first**: the
numbers are first guesses for the balance pass to change, not the point.

## Goals

1. **Stake every campaign match.** Launching a campaign match asks for a wager
   between the opponent's ante floor and the player's full balance.
2. **Even-money payout.** A win returns the stake plus the same again; a loss
   forfeits it. The old free win credit and free card drop are gone.
3. **Going broke ends the run.** If a loss leaves the player unable to cover
   the cheapest available ante, the run is over: a full reset to the starter
   profile (deck, collection, progress, seed credits), settings and lifetime
   records surviving.
4. **Rematches.** A cleared planet can be replayed at normal stakes without
   changing progress, so credits can be earned after the first clear.
5. **No free escape.** The stake leaves the balance when the match starts and
   stays committed through a mid-match save; discarding a staked match forfeits
   it.
6. **Tunable.** Seed purse, ante floor formula, stake step, and payout are
   named constants with documented defaults, changeable without touching the
   flow.
7. **No regression.** Quick Play stays stakes-free and unchanged; every existing
   screen, rule, and save loads and behaves as before; no save-format version
   bump.

## Non-goals (explicitly deferred)

- **Endgame / victory award** — what beating the whole campaign pays. Deferred
  by human ruling (`ROADMAP.md`); with rematches a completed run just stays
  playable.
- **The casual-campaign vs. roguelike-mode identity question** (spec E). Human-
  ruled deferral: for now stakes go *into* the existing campaign.
- **A keep-your-cards (soft) restart.** Noted future difficulty lever if the
  full reset playtests too punishing; not built speculatively.
- **A maximum stake cap.** All-in is legal. A cap is a noted balance lever the
  balance pass can add; one fewer constant to guess at now.
- **Selling cards back to the shop.** Would be a second way out of "broke" and
  complicates the loss condition; not needed for the loop to work.
- **Odds / better-than-even payouts for harder opponents.** Difficulty is
  expressed through the ante floor; a payout multiplier stays a balance lever.
- **A rare card drop.** Removed entirely rather than kept at a low chance;
  re-adding it later is cheap if shop-only proves too dry.
- **Balance.** Whether these numbers make progression *feel* right is the
  separate **Difficulty & economy balance pass** (`ROADMAP.md`), which needs this
  loop live first.

## Entities

- **Stake** — the credits committed to the campaign match in flight. Chosen at
  launch, held out of the balance until the match resolves, remembered across a
  mid-match save.
- **Ante floor** — an opponent's minimum stake, derived from its difficulty (the
  same threshold scalar spec 012's reward used).
- **Seed purse** — the credits a fresh (or reset) profile starts with.
- **Broke** — a balance that can't cover the cheapest ante among the nodes the
  player could launch right now.

## Key behavior

### Tunable constants (first guesses; the balance pass owns these)

| Constant | Default | Meaning |
|---|---|---|
| Seed purse | **50** | Credits a fresh / reset profile starts with (was 0). One Outer card plus a few minimum antes. |
| Ante floor | **(threshold − 14) × 10** → 10 / 20 / 30 / 40 / 50 | Minimum stake per opponent, by its stand threshold. |
| Stake step | **5** | Increment the wager prompt moves in. |
| Payout | **1 : 1** | A win returns the stake plus the same again. |
| Maximum stake | the full balance | No cap (see Non-goals). |

### Launching a staked match

From the campaign map, Enter on a launchable node opens a **wager prompt** over
the map instead of starting the match directly. It shows the opponent, its ante
floor, the player's balance, and what a win pays. The stake starts at the floor;
←/→ move it in stake steps between the floor and the full balance; Enter commits
and starts the match; Esc backs out to the map with nothing changed.

If the balance can't cover the node's floor, Enter refuses with a short message
(a soft no-op, like an unaffordable shop buy) and no prompt opens.

On commit the stake **leaves the balance** (the map header's `◈ N` drops by it)
and the match begins as today.

### Rematches

A **cleared planet** is launchable: Enter on it stakes a match against the
planet's **final** opponent. A rematch is a normal staked match — same floor,
same payout — and changes no campaign progress: nothing is un-beaten or
re-beaten, no world unlocks, and a **campaign completion is counted only by the
win that completes the run**, never by a later rematch win. Match and round
records (spec 020) record rematches like any other match.

An uncleared planet launches its next un-beaten opponent, exactly as today.

### Resolving a match

- **Win** — the balance gains twice the stake (the stake back plus the winnings).
  The map's return banner reads "Won N credits" (no card). A first clear also
  marks the opponent beaten and unlocks as today.
- **Loss** — the stake is simply gone (it left the balance at launch). The banner
  reads "Lost N credits". The node stays open to retry, as today.
- **Quick Play** is untouched: no prompt, no stake, no payout.

### Going broke

After a **loss**, if the balance is below the cheapest ante floor among the nodes
the player could launch (with rematches, that is the Outer Rim's easiest
opponent), the run is over. A **run-over modal** appears over the map: it says
the player is broke and the run has ended, and Enter acknowledges it. On
acknowledgement the profile is **fully reset** through the same operation as
spec 014's New Campaign — starter deck and collection, no progress, the seed
purse — and a fresh map opens (first world only, `◈ 50`). Settings and lifetime
records survive exactly as they do for New Campaign. There is no way to decline:
the reset *is* the loss condition.

The same check runs at **campaign entry** (Continue), so a profile that somehow
can't cover any ante with no match in flight — in practice, a pre-021 profile
that never earned credits — meets the same run-over flow rather than a map
where nothing can be launched.

The **shop refuses a purchase** that would leave the balance below that cheapest
floor, so shopping can never end a run; the refused card reads as unaffordable,
the way an over-budget card does today.

### Stakes and the mid-match save

A staked campaign match that is saved mid-match **resumes with its stake
intact**: the balance already reflects the escrow, and the match resolves on
resume exactly as if never interrupted. **Discarding** a saved staked match —
via Quick Play over a save, Start Campaign's discard confirm, or New Campaign —
**forfeits the stake**; the existing discard confirmations say so ("…and forfeit
your N-credit stake") when a stake is at risk.

### The stake in-match

While a staked match is in play, the stake is visible beside the board (in the
existing presence panel, within the 139×31 minimum), so the tension of the wager
is on screen throughout. Quick Play shows no stake line.

## Design requirements

- **The staking core is deterministic and testable** — ante floor, affordability,
  escrow, payout, the broke test, and the shop reserve are pure profile/economy
  operations, unit-tested, with the screens as thin callers. No randomness is
  introduced (the card-drop roll is removed, not replaced).
- **All numbers are named constants** in one place, with the defaults above,
  documented in `docs/economy.md` for the balance pass.
- **The wager prompt and the run-over notice follow the modal convention** —
  opened over the map, dismissed back to it, one modal at a time, safe/neutral
  default (the prompt opens at the floor; Esc changes nothing).
- **The engine, board, and match rules are untouched.** `GameState` stays
  economy-agnostic; the stake is campaign/profile context, like the in-progress
  node pointer (spec 009). The in-match stake line is display only.
- **Additive persistence, no version bump** — the stake rides with the existing
  campaign in-flight context as a serde-defaulted field; a pre-021 profile or
  save loads with no stake (0) and no `PROFILE_VERSION` / save-version bump. Seed
  credits apply to *new and reset* profiles only; an existing profile keeps its
  balance.
- **The reset is spec 014's operation**, not a second reset path — one
  `reset_to_starter`, so the run-over reset can't drift from New Campaign's.
- **Monochrome, legible at the minimums** — the prompt and the notice fit 89×31
  on the map; the stake line fits the 139×31 in-match layout.

## Acceptance criteria

- [ ] Enter on a launchable campaign node opens a wager prompt showing the
      opponent, its floor, the balance, and the payout; ←/→ adjust the stake in
      steps between the floor and the full balance; Enter starts the match with
      the balance reduced by the stake; Esc returns to the map with nothing
      changed.
- [ ] A node whose floor exceeds the balance refuses to launch with a message
      and opens no prompt.
- [ ] Ante floors follow the difficulty scalar (10 / 20 / 30 / 40 / 50 by
      threshold 15–19) and are a single tunable formula.
- [ ] A win pays twice the stake into the balance and shows "Won N credits";
      a loss pays nothing and shows "Lost N credits"; no card is dropped on
      either. A first clear still marks the opponent beaten and unlocks as
      before.
- [ ] Enter on a cleared planet stakes a rematch against its final opponent;
      a rematch win or loss changes no campaign progress and never increments
      campaign completions; per-opponent match/round records still record it.
- [ ] A loss that leaves the balance below the cheapest launchable floor shows
      the run-over notice; acknowledging it resets the profile to the starter
      (starter deck and collection, no progress, seed purse) and opens a fresh
      map; settings and lifetime records are preserved; an in-progress save is
      cleared.
- [ ] The same run-over flow fires on Continue for a profile that can't cover
      any floor with no match in flight.
- [ ] The shop refuses a purchase that would leave the balance below the
      cheapest launchable floor, and reads it as unaffordable.
- [ ] A staked match saved mid-match resumes with its stake and resolves
      normally; discarding a saved staked match forfeits the stake, and the
      discard confirmation says so.
- [ ] A fresh profile starts with the seed purse (50); an existing profile keeps
      its balance; pre-021 profiles and saves load with no version bump.
- [ ] Quick Play shows no prompt, stakes nothing, pays nothing, and shows no
      stake line; a staked match shows its stake beside the board.
- [ ] `docs/economy.md` documents the new loop and every constant; `cargo test`
      green (staking core, broke test, shop reserve, completion-once, and
      modal-flow decision tests); `cargo build` no new warnings; legible at the
      minimums; no panics in play.

## Resolved decisions

All ruled with the human this session (2026-09-10), on the recommendations as
proposed:

- **Rematches on cleared planets, against the final opponent** — the balance
  pass's invariant ("safe minimum-wager grinding can't fund the next tier")
  presumes grinding exists, and it didn't: a cleared planet was a no-op. The
  final opponent is the simplest rule; a per-opponent picker can come later if
  wanted.
- **Seed purse of 50** — a fresh profile had 0 credits, which can't stake.
- **Ante floor reuses the threshold scalar** — the same (threshold − 14) × 10 the
  old win reward used, so difficulty keeps one number.
- **Player-chosen stake, floor to full balance, no cap** — "bigger, riskier bets"
  must be a real lever; a cap is a later balance knob.
- **Even money, old win credit removed** — difficulty now lives in the floor, so
  scaling the payout too would double-count it. First clears earn only
  progress.
- **Free card drop removed outright** rather than made rare — cards come from the
  shop; a rare drop is cheap to add back if playtest wants it.
- **Escrow at launch** — the stake leaves the balance when the match starts, so
  the header is honest mid-match and discarding a saved match can't dodge a
  loss.
- **Broke = can't cover the cheapest launchable floor, checked after a loss and
  at campaign entry; the shop reserves that floor** — the floor stays a real
  floor (no "last stand" below it), and shopping can never end a run. The
  entry-time check doubles as the migration path for a pre-021 profile with no
  credits: it meets the run-over flow rather than a dead map. Accepted for a
  personal project over a progress-preserving top-up.
- **Run-over is a modal on the map, reset via spec 014's path** — one reset
  operation, lifetime records preserved per spec 020.
- **Stake shown in-match; no "runs ended broke" counter** — the wager's tension
  belongs on screen; a bust counter is a cheap additive field if the balance
  pass wants it.
- **Campaign completion counts once** — spec 020's "a completed run exposes no
  launchable match" assumption no longer holds with rematches, so completion is
  tied to the completing win explicitly.
