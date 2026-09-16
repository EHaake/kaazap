# Spec: Endgame, victory & what you keep — spec 024

**Status**: Approved (2026-09-15)
**Depends on**: spec 021 (wager & loss, run-over, rematches), spec 020 (stats &
records), spec 014 (New Campaign — partly superseded here), spec 022 (Quick
Play deals the standard deck — superseded here), spec 023 (onboarding texts)

## Summary

Beating the Sovereign does almost nothing today. The board shows the ordinary
win popup, the map comes back with a settled banner and one muted line
("Campaign complete — rematches stay open"), and a lifetime counter goes up by
one. Losing has a full notice and a reset behind it; winning has nothing. This
spec gives the win an ending and makes one principle explicit across the game:

> **What you earn is yours.** The profile is one pool of cards and credits
> shared by every mode. A new player has the starter deck and the seed purse,
> earns credits only by staking campaign matches, and buys cards only at the
> Outfitter. Everything they earn stays theirs — in the campaign, in Quick
> Play, and across replays. Only going broke takes it away.

Concretely:

1. **A victory notice** over the map when the run is completed, carrying a
   **run summary** (matches, credits won and lost, best streak), and a note
   that the cards and credits are kept and that rematches and New Campaign
   remain open. The run does **not** end; the map stays open.
2. **The same run summary on the run-over notice**, so going broke reads as a
   score, not just a wipe. (Folds in the roadmap's "run summary on the
   run-over notice" item.)
3. **A lifetime record** of the first completion — how many matches it took —
   on the Records screen's Campaign view.
4. **The map shows its completed state** after the win, persistently.
5. **Quick Play deals the deck you built**, not the standard deck. Supersedes
   the spec 022 ruling.
6. **New Campaign keeps your cards and credits** and resets the map only. A
   separate **Reset Everything** choice does what New Campaign did before.
   Supersedes spec 014's "New Campaign = full fresh start".

No engine, AI, balance-data, or save-format change. The profile gains additive
fields only; `PROFILE_VERSION` stays 1.

## Goals

1. **The win has an ending.** Acknowledging the game-over popup of the match
   that completes the run lands on the map with a victory notice over it.
   One key dismisses it. It appears once per completion — a rematch win on a
   completed run never raises it; the completing win of a replayed campaign
   does.
2. **The run summary exists once and shows twice.** One block of lines —
   matches played, won and lost, credits won and credits lost this run, best
   streak, worlds cleared — on both the victory notice and the run-over
   notice. The numbers are the current run's, zeroed with the run.
3. **A first-clear record.** The number of matches the first completed run
   took is kept for life, shown on the Records Campaign view, and never
   overwritten by a replay. An existing profile that has already completed a
   run has no record and never gains one; its Records line reads as it does
   today. (See *Resolved decisions*, the record flag.)
4. **The map reads as complete.** After the win, the map's rim→core axis
   label gives way to a completed marker that stays until the map is reset;
   the planet panel's completion line and the rematch flow are unchanged.
5. **Quick Play deals the built deck.** The one line that dealt the standard
   deck is gone, and every text that said so is corrected. An incomplete deck
   diverts to the deck-builder as it already does.
6. **New Campaign keeps the pool.** From the Campaign entry panel, New
   Campaign clears the map (beaten opponents, the in-flight match and its
   stake, the run tally) and keeps credits, collection, deck, lifetime
   records and the onboarding marks. Reset Everything is the old full wipe,
   behind the same default-No confirm it has today.
7. **No regression.** No change to scoring, round or match resolution, the
   AI, the wager prompt, settlement math, the ante floors, the shop, the
   run-over reset itself, or any save format; an existing profile and an
   existing mid-match save load as before.

## Non-goals (explicitly deferred)

- **A credit bonus or a unique card for the win.** Credits after completion
  buy nothing new, and a new card is an engine and balance change.
- **A per-completion history or a "best replay" record.** One first-clear
  number. Archiving whole runs is the roadmap's separate *Archive the last
  run at reset* item.
- **Retuning the difficulty or economy curve for replays.** A replay with a
  premium deck is easier than the spec 022 curve assumed. It is opt-in, and
  the curve is left alone.
- **Scaling difficulty on replay** (a New Game Plus that also sharpens the
  opponents). Noted as a later lever, not built.
- **A softer run-over.** Going broke still resets everything. That is the loss
  condition's teeth (spec 021), and the only thing that now takes the pool.
- **A per-opponent rematch picker, selling cards, or any economy change.**
- **Changing the game-over popup on the board.** The ending lives on the map,
  where the run-over notice already lives.

## Entities

- **Victory notice** — a modal over the galaxy map, raised on the completing
  win's return to the map. Title, a short keep-your-pool note, the run
  summary, a dismiss line. Transient: no seen mark; quitting under it loses
  only the notice, never the completion or the payout.
- **Run summary** — the lines both notices share, computed from the run tally:
  matches played, won, lost; credits won this run; credits lost this run;
  best streak; worlds cleared out of the total.
- **Run tally** — the existing per-run statistics, extended with two additive
  counters: **credits won** (net gain on each settled win, the same number the
  map banner reports) and **credits lost** (each forfeited stake). Both
  default to zero, both zero with the run.
- **First-clear record** — one optional lifetime number: matches played in
  the run that produced the profile's first completion, set on that
  completion's edge and never changed after. Additive, default unset.
- **Campaign entry panel** — the existing Continue / New Campaign choice at
  Start Campaign, now three choices: **Continue**, **New Campaign**, **Reset
  Everything**.

## Key behavior

### The victory notice

- Raised when the player acknowledges the game-over popup of a campaign match
  whose settlement completed the run — the same once-only edge that counts a
  campaign completion. The player lands on the map with the notice over it.
- A rematch win on an already-complete run raises nothing; the settled banner
  behaves as today. Completing a replayed campaign raises it again.
- A completing win can never leave the player broke, so it never competes
  with the run-over notice. The primer is raised on menu entry only, so it
  never competes either. One modal at a time holds.
- Proposed text (the person edits; fewest words wins):

  ```
  Campaign complete — the house's best has lost.

  Your cards and credits are yours to keep.
  Rematches stay open; New Campaign replays the map with your deck.

  Matches played 14  ·  won 11  ·  lost 3
  Credits won 640  ·  lost 210
  Best streak 6  ·  Worlds cleared 8/8

                    Enter  continue
  ```

- Enter, Space or Esc dismisses it (a notice the player may wave away, like
  the onboarding pieces). While it is up, no map key does anything. Dismissal
  plays the menu-select sound. After dismissal the map is live; the settled
  banner for the completing win shows in the header until the first
  navigation, as any settled banner does.
- Quitting with the notice up loses the notice. The completion is already
  counted and paid at settlement; the run's matches and streak stay readable
  on the Records screen's This Run view (the credit counters and worlds
  cleared are shown on the notices only).

### The run summary on the run-over notice

- The run-over notice keeps its title and its reset note and gains the same
  summary block between them. Its dismiss rules are unchanged: Enter or
  Space acknowledge and reset; Esc is ignored.
- Proposed shape:

  ```
  You're broke — the run is over.

  Matches played 9  ·  won 3  ·  lost 6
  Credits won 80  ·  lost 130
  Best streak 2  ·  Worlds cleared 2/8

  Deck, collection, and progress reset to the starter; your records stay.

                    Enter  continue
  ```

- The two credit counters are bumped at settlement: a win adds its net gain
  (payout minus stake — the same number the map banner shows), a loss adds
  the forfeited stake. A stake forfeited by discarding a saved match is
  **not** counted: nothing settled. Both live with the run tally and reset
  with it.

### The first-clear record

- Set once, on the edge that takes the profile's completion count from zero
  to one, to the run's matches played including the completing match. Never
  updated afterward — replays do not touch it.
- Shown on the Records Campaign view folded into the existing completions
  line so the breakdown table keeps its fixed offset: `Campaign completions:
  2  ·  first clear in 14 matches`. When unset, the line reads as it does
  today.
- An existing profile with completions already above zero never sets it
  (the edge has passed). Accepted: this is a record for runs from here on.

### The map after the win

- While the run is complete, the header's second row shows a completed marker
  in place of the `Outer Rim → The Core` axis label — proposed `★  Campaign
  complete`. A settled or can't-cover banner still takes that row while it is
  showing, exactly as today. The planet panel's "Campaign complete — rematches
  stay open." line and the rematch flow are unchanged.
- New Campaign and Reset Everything both return the map to its fresh state,
  so the marker goes with the run.

### Quick Play deals the built deck

- A Quick Play match deals the player's built deck, the same deck a campaign
  match deals. The standard deck keeps its role as the opponents' baseline.
- The opponent select screen's line becomes a statement of what Quick Play
  is — proposed `Quick Play deals your deck. Nothing is staked.` — and the
  README's Quick Play sentence is corrected. How to Play does not mention the
  Quick Play deck today and does not need to.
- Consequence, accepted: a fresh profile's Quick Play deals the starter deck,
  so Quick Play against the Core is hard until the player has shopped.
- The incomplete-deck divert to the builder at Quick Play entry already
  exists and is unchanged.

### New Campaign and Reset Everything

- **When the panel shows.** Start Campaign opens the Campaign entry panel when
  there is anything the choices would affect: the run has progress, or the
  pool differs from the starter (credits, collection or deck). A truly fresh
  profile still opens the map directly, as today.
- **Continue** is unchanged.
- **New Campaign** — resets the map: beaten opponents cleared, the in-flight
  match and its escrowed stake dropped, the run tally zeroed, the saved match
  (if any) cleared, the stale banner cleared. Credits, collection, deck,
  lifetime records, onboarding marks and settings are kept. Behind a
  default-No confirm — proposed title `New campaign? Resets the map; you keep
  your cards and credits.` — carrying the existing "…and forfeit your
  N-credit stake." note when a match is in flight. Then the map opens as a
  menu entry does today (the broke check runs: a player whose balance cannot
  cover the fresh map's cheapest ante meets the run-over notice, which is the
  honest outcome).
- **Reset Everything** — the current New Campaign behaviour, unchanged: the
  full wipe to the starter, behind the current default-No confirm with the
  current title (`… Erases progress, credits & cards.`) and stake note.
  Lifetime records and onboarding marks survive it, as today.
- Esc backs out of the panel and of either confirm without change.
- The primer never shows on either path: its mark survives both, as today.

### Persistence

- Two additive counters on the run tally and one additive optional number on
  the lifetime stats, all serde-defaulted. `PROFILE_VERSION` stays 1; the
  match save format is untouched. A pre-024 profile loads with zero counters
  and no record.

## Design requirements

- **Modal convention.** The victory notice is a modal over the map — one
  modal at a time, the same bordered box the run-over notice uses, monochrome.
- **Density and breathing room** (design brief; `CLAUDE.md`). The dismiss
  line is the acted-on element and gets an empty row above and below it; the
  summary rows stay compact, one blank row separating the text blocks. The
  box pads its content evenly — one empty row above and below — never a slab
  of empty rows.
- **The Campaign entry panel** keeps the two-choice panel's look with a
  third label on the same row; the highlighted choice pulses as today; the
  hint line names the keys.
- **Fewest possible words.** The proposed texts are ceilings.
- **Fits the minimum terminal** (139×31) with the map drawn underneath.
- **Sound.** Dismissing the victory notice plays the menu-select sound;
  Reset Everything and New Campaign confirms play what the current confirm
  plays.

## Acceptance criteria

- [ ] Beating the final opponent for the first time in a run, then
      acknowledging the game-over popup, lands on the map with the victory
      notice over it showing the run's numbers; Enter, Space or Esc
      dismisses it; no map key acts while it is up; the completed marker is
      on the header afterward.
- [ ] A rematch win (or loss) on a completed run raises no victory notice
      and behaves as before; completing a replayed campaign (after New
      Campaign) raises it again and increments completions again.
- [ ] The run-over notice shows the same summary block between its title and
      its reset note; Esc is still ignored; Enter or Space still resets.
- [ ] Credits won and lost this run count settled wins (net gain) and settled
      losses (forfeited stake) only; a stake forfeited by discarding a saved
      match counts toward neither; both zero with the run; both round-trip
      through `profile.json` and default to zero for a pre-024 document.
- [ ] The first-clear record is set only on the profile's first completion,
      to that run's matches played; a second completion leaves it unchanged;
      it round-trips and defaults to unset; the Records Campaign view shows
      it folded into the completions line and shows today's line when unset.
- [ ] The Records breakdown table starts at the same row in every view, as
      it does today.
- [ ] Quick Play deals the built deck: a Quick Play match's hand comes from
      the profile's deck (a test with a non-standard built deck); the
      opponent select line and the README no longer say the standard deck is
      dealt; an incomplete deck still diverts to the builder.
- [ ] Start Campaign shows Continue / New Campaign / Reset Everything whenever
      the run has progress or the pool differs from the starter, and opens
      the map directly for a fresh profile.
- [ ] New Campaign (confirmed) clears beaten opponents, the in-flight pointer
      and stake, the run tally, the saved match and the banner, and keeps
      credits, collection, deck, lifetime stats and onboarding marks; the
      confirm's No and Esc change nothing; the stake note appears when a
      match is in flight.
- [ ] Reset Everything (confirmed) resets to the starter exactly as New
      Campaign did before this spec; its confirm's No and Esc change nothing.
- [ ] After New Campaign with a balance below the fresh map's cheapest ante,
      the run-over notice shows.
- [ ] Both notices and the three-choice panel fit 139×31 over their screens,
      follow the breathing-room rule, and are monochrome.
- [ ] No code change in `src/game.rs`, `src/player.rs`, `src/card.rs`,
      `src/save.rs`, `src/economy.rs` or the AI — a doc comment that still
      says Quick Play deals the standard deck may be corrected, nothing else;
      `PROFILE_VERSION` and `SAVE_VERSION` are 1; `cargo test` is green.

## Resolved decisions

Ruled with the person on 2026-09-15, on the recommendations as proposed:

- **1 — The run continues after the win.** A notice once, then the map stays
  open with rematches and the shop. No new reset path. (Ending like a loss
  would throw away the deck built to win; New Game Plus with scaled
  difficulty stays deferred.)
- **2 — The award is a victory notice with a run summary, plus one lifetime
  record.** A credit bonus buys nothing after completion; a unique card is an
  engine and balance change.
- **3 — The run-summary backlog item folds in.** One summary, two notices.
- **4 — The map keeps a persistent completed marker.**
- **5 — Quick Play deals the built deck.** Supersedes spec 022's ruling
  (which accepted, on record, that the built deck only mattered in campaign
  matches). `DECISIONS.md` records the reversal at the merge.
- **6 — New Campaign keeps cards and credits; Reset Everything is the full
  wipe.** Supersedes spec 014's "New Campaign = full fresh start". The 014
  argument — keep the early game and the depth-gated economy meaningful — no
  longer holds: since spec 021 a first clear pays only progress and every
  match is even money, so a replay earns no more than rematches already can.
  Going broke becomes the only thing that takes the pool, which is the loss
  condition's intent. This also answers the open casual-versus-roguelike
  identity question (spec 021's "spec E") in the casual direction, with
  going broke as the one roguelike bite.
- **The principle, in the person's words:** the profile is a global pool of
  cards and credits between all modes; a new player has only the basic deck
  and no credits to buy anything, plays the campaign to earn, and once they
  win they keep everything for Quick Play as well; New Campaign keeps
  accumulating, with a separate option to reset everything.
- **Record flag (session's pick, pending the person's word).** The first-clear
  record counts the first completion only, so a starter-deck first run is
  never compared with a premium-deck replay. A "best completion" that
  replays could beat was the alternative.
- **Not retuning the curve for replays** — opt-in replay, noted in
  `docs/balance.md` at the merge, no constant moves.
