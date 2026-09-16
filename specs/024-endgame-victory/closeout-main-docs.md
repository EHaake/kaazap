# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 024)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T010, ready to paste on `main` after the merge
(the same close-out shape specs 020–023 used). Nothing here is applied by the
spec branch.

No `CLAUDE.md` amendment is needed: spec 024 changed no rule the constitution
states. Line numbers below are `main`'s at the time of drafting (2026-09-16);
each edit also quotes its anchor text, which is what to match on.

---

## 1. `ROADMAP.md` — eight edits

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **First-run onboarding & controls refinement
(spec 023)** entry (which ends "…No rules, AI, economy, wager or settlement
change.", currently `ROADMAP.md` line ~330, just before `## Backlog`):

```markdown
- **Endgame, victory & what you keep** (spec 024) — the win finally has an
  ending, and one principle is now explicit across the game: **what you earn is
  yours.** Acknowledging the game-over popup of the match that **completes the
  run** lands on the galaxy map with a **victory notice** over it — title, a
  keep-your-pool note, the run summary, one dismiss line — raised once per
  completion (a rematch win on a complete run raises nothing; a replayed
  campaign's completing win raises it again), dismissed with Enter, Space or
  Esc, and holding every map key while it is up. The run does **not** end: the
  map stays open with rematches and the Outfitter. The **run summary** —
  matches played / won / lost, credits won and lost this run, best streak,
  worlds cleared — is one pure builder shown on **both** notices, so going broke
  now reads as a score too (folding in the backlog's *run summary on the
  run-over notice* item). The run tally gained two serde-defaulted counters
  (`credits_won` the net gain on each settled win, `credits_lost` each forfeited
  stake; a stake forfeited by discarding a saved match counts toward neither),
  and the lifetime stats gained one optional **first-clear record** — matches
  played in the run that produced the profile's first completion, set on the
  0 → 1 completions edge, never overwritten, folded into the Records Campaign
  line (`Campaign completions: 2  ·  first clear in 14 matches`) so the
  breakdown table keeps its fixed offset. The map header's axis label gives way
  to **`★  Campaign complete`** until the map is reset. **Quick Play now deals
  the deck you built** (superseding spec 022's ruling): `player_deck_for` is
  gone, `start_match` has one deal for both modes, and the opponent select line
  reads "Quick Play deals your deck. Nothing is staked." **New Campaign keeps
  your cards and credits** (superseding spec 014's full-wipe New Campaign): it
  resets the map only — beaten opponents, the in-flight match and its escrowed
  stake, the run tally — while a third entry choice, **Reset Everything**, is
  the old full wipe. Start Campaign's panel is therefore three choices, shown
  whenever the run has progress *or* the pool differs from the starter.
  Structurally, `App::tick`'s two profile calls became one
  `Profile::resolve_match` that owns the record-then-settle order (the
  first-clear number depends on it), `draw_two_choice` became an N-label
  `draw_choice_panel` that gives the acted-on choice row its blank row above
  even when a note is showing (which also corrected the spec-021 discard
  confirm), and `draw_run_over` became a shared `draw_notice`. No engine, AI,
  wager, settlement-math or save-format change; `tests/balance.rs`, `Cargo.toml`
  and `Cargo.lock` untouched; `PROFILE_VERSION` and `SAVE_VERSION` stay 1.
  `Readme.md`, `docs/economy.md` and `docs/balance.md` re-synced (the last with
  a short *Replays* note: a replay starts premium, deliberately not retuned).
```

### 1b. Inline annotation on the spec 014 entry (currently lines 173–176)

The **New Campaign / start over (spec 014)** entry describes New Campaign as a
full fresh start in the present tense. Using the repo's inline-supersession
convention (ROADMAP lines 95–98), replace:

```markdown
  Campaign offers **Continue** vs **New Campaign**; New Campaign is a full fresh
  start (`Profile::reset_to_starter` — wipes progress, credits, and collection/deck
  to the starter, keeping settings) behind a default-No confirm. A no-progress
  profile opens the map directly (unchanged).
```

with:

```markdown
  Campaign offers **Continue** vs **New Campaign**; New Campaign was a full fresh
  start (`Profile::reset_to_starter` — wipes progress, credits, and collection/deck
  to the starter, keeping settings) behind a default-No confirm. A no-progress
  profile opens the map directly — **superseded by spec 024**, which made New
  Campaign reset the **map only** (you keep credits, collection and deck), added
  a third **Reset Everything** choice for the old full wipe, and widened the
  panel's trigger to "the run has progress *or* the pool differs from the
  starter". The default-No confirm and its `confirm_choice` seam are unchanged.
```

The sentence immediately after the replaced text (lines 176–178) also dates — the
two-choice helper is now an N-label one. Replace:

```markdown
  or save-format change; rendering DRYed into a shared two-choice overlay helper,
  and the irreversible wipe guarded by a mutation-checked `confirm_choice` seam.
```

with:

```markdown
  or save-format change; rendering DRYed into a shared two-choice overlay helper
  — **generalized by spec 024** to `draw_choice_panel`, which takes any number of
  labels and serves the three-choice entry panel too — and the irreversible wipe
  guarded by a mutation-checked `confirm_choice` seam, which still guards both of
  spec 024's reset scopes.
```

### 1c. Inline annotation on the spec 022 entry (currently lines 284–285)

In the **Difficulty & economy balance pass (spec 022)** entry, replace:

```markdown
  constant, with lateral spares), the **standard deck** keeps its role as the
  opponent baseline and is what **Quick Play** deals, the roster was retuned
```

with:

```markdown
  constant, with lateral spares), the **standard deck** keeps its role as the
  opponent baseline and was what **Quick Play** dealt — **superseded by spec
  024**, which gives Quick Play the deck you built; the standard deck's
  opponent-baseline role is unchanged — the roster was retuned
```

### 1d. Inline annotations on the two spec 023 mentions of the old line

The spec 023 entry's "Quick Play deals the standard deck." claim appears in
**two places with different wording**, so each needs its own anchor. (An earlier
draft of this file said the sentence appeared twice identically; it does not.)

**1d-i — the Shipped entry** (currently lines 327–329), replace:

```markdown
  re-synced, and the opponent select screen finally says **"Quick Play deals the
  standard deck."** (spec 022's deferred line). No rules, AI, economy, wager or
  settlement change.
```

with:

```markdown
  re-synced, and the opponent select screen finally said **"Quick Play deals the
  standard deck."** (spec 022's deferred line) — **superseded by spec 024**,
  which made Quick Play deal the built deck and rewrote that line as **"Quick
  Play deals your deck. Nothing is staked."** No rules, AI, economy, wager or
  settlement change.
```

**1d-ii — the backlog's shipped-marked *First-run onboarding* bullet**
(currently lines 488–489), replace:

```markdown
  section; the cheap extra shipped too ("Quick Play deals the standard deck." on
  the opponent select screen).
```

with:

```markdown
  section; the cheap extra shipped too ("Quick Play deals the standard deck." on
  the opponent select screen — **superseded by spec 024**, which made Quick Play
  deal the built deck and rewrote the line as "Quick Play deals your deck.
  Nothing is staked.").
```

### 1e. Inline annotation on the spec 008 deck-builder entry (currently lines 87–88)

The **side-deck customization (spec 008)** entry still restricts the built deck
to campaign matches in the present tense. Replace:

```markdown
  owned counts. Matches deal the player's hand from the built deck (campaign
  matches, since spec 022) — the player deck moved from the `DEFAULT_SIDE_DECK`
```

with:

```markdown
  owned counts. Matches deal the player's hand from the built deck (campaign
  matches only, between spec 022 and spec 024 — **superseded by spec 024**,
  which gives **every** match, Quick Play included, the built deck) — the player
  deck moved from the `DEFAULT_SIDE_DECK`
```

### 1f. Inline annotation on that entry's spec-022 supersession note (currently lines 95–97)

The existing "**superseded by spec 022**" annotation a few lines below ends by
naming the standard deck as Quick Play's deal. Replace:

```markdown
  **superseded by spec 022**, which gave a fresh profile its own Outer-tier
  `profile::STARTER_SIDE_DECK` plus lateral spares and left
  `card::DEFAULT_SIDE_DECK` as the opponent baseline and Quick Play's deal. A
```

with:

```markdown
  **superseded by spec 022**, which gave a fresh profile its own Outer-tier
  `profile::STARTER_SIDE_DECK` plus lateral spares and left
  `card::DEFAULT_SIDE_DECK` as the opponent baseline and Quick Play's deal — the
  Quick Play half **superseded in turn by spec 024**, which leaves
  `DEFAULT_SIDE_DECK` the opponent baseline *only*. A
```

### 1g. Replace the two backlog bullets this spec shipped

Under "### Onboarding, endgame & release readiness (suggested 2026-09-13, after
spec 022)", replace the whole **endgame / victory award** bullet (currently
lines 493–499, beginning "- **The endgame / victory award** — what beating the
Sovereign gives.") *and* the whole **run summary** bullet that follows it
(currently lines 500–505, beginning "- **A run summary on the run-over
notice.**") — the two ship together — with:

```markdown
- **The endgame / victory award** — ✅ **Shipped (spec 024** — see Shipped
  above). The win awards a **victory notice with a run summary** plus one
  lifetime **first-clear record** on the Records Campaign view; the map shows
  **`★  Campaign complete`** afterward; and the run **continues** — rematches
  and the Outfitter stay open, with New Campaign replaying the map with the deck
  you built. A credit bonus and a unique card were ruled out (nothing to buy
  after completion; a new card is an engine and balance change), and New Game
  Plus scaling stays deferred.
- **A run summary on the run-over notice.** — ✅ **Shipped (spec 024**, folded
  into the endgame spec: one `run_summary_lines` builder, two notices). The
  run-over notice now carries matches played / won / lost, credits won and lost
  this run, best streak and worlds cleared between its title and its reset note.
  "Deepest planet reached" shipped as **worlds cleared out of the total**, which
  reads better on a map whose tiers unlock by clears.
```

---

## 2. `DECISIONS.md` — five edits, plus the three small ones in §2f

### 2a. The spec 014 "full fresh start" bullet is superseded (currently lines 315–320)

In the **New Campaign / start over (spec 014)** section, replace:

```markdown
- **New Campaign = full fresh start** — wipe campaign progress, credits, and the
  collection/deck back to the starter (`Profile::reset_to_starter` = a fresh
  `Profile::default`), keeping audio/settings (a separate file). Chosen over an
  NG+-style replay that keeps your arsenal, so the early game and the depth-gated
  economy stay meaningful; NG+ stays deferred to spec E (see the amended economy
  note above). Destructive, so it sits behind a default-No confirm.
```

with:

```markdown
- **New Campaign = full fresh start** — wipe campaign progress, credits, and the
  collection/deck back to the starter (`Profile::reset_to_starter` = a fresh
  `Profile::default`), keeping audio/settings (a separate file). Chosen over an
  NG+-style replay that keeps your arsenal, so the early game and the depth-gated
  economy stay meaningful; NG+ stays deferred to spec E (see the amended economy
  note above). Destructive, so it sits behind a default-No confirm.
  **Superseded by spec 024**: New Campaign now resets the **map only** and keeps
  the pool, with a separate **Reset Everything** choice doing the wipe described
  here (same `reset_to_starter`, same default-No confirm). The 014 argument no
  longer holds — since spec 021 every match is even money and a first clear pays
  only progress, so a replay earns no more than rematches already can; going
  broke is now the only thing that takes the pool. See *Endgame, victory & what
  you keep (spec 024)* below.
```

### 2b. The spec 022 Quick Play ruling is reversed (currently lines 783–786)

In the **Difficulty & economy balance pass (spec 022)** section, replace:

```markdown
- **Quick Play deals the standard (premium) deck** [human ruling, against the
  recommendation to deal the built deck]. Accepted downside, on record: the deck
  you build only matters in campaign matches, so Quick Play is no longer a place
  to test a build. Reversible in one line if that proves annoying.
```

with:

```markdown
- **Quick Play deals the standard (premium) deck** [human ruling, against the
  recommendation to deal the built deck]. Accepted downside, on record: the deck
  you build only matters in campaign matches, so Quick Play is no longer a place
  to test a build. Reversible in one line if that proves annoying.
  **Reversed by spec 024** — the accepted downside is exactly what proved
  annoying, and "what you earn is yours" made the one profile pool the rule in
  every mode. Quick Play deals the built deck; the standard deck keeps its
  opponent-baseline role. The consequence, accepted: a fresh profile's Quick
  Play deals the starter deck, so Quick Play against the Core is hard until the
  player has shopped. See *Endgame, victory & what you keep (spec 024)* below.
```

### 2c. The spec 022 deck-validity-divert bullet is doubly wrong now (currently lines 854–858)

In the same spec 022 section, the divert bullet's reasoning has inverted — Quick
Play *does* use the built deck now, so the divert is no longer a "consistency
nudge" but a genuine precondition. Replace:

```markdown
- **Quick Play keeps today's deck-validity divert.** `open_opponent_select`
  still sends an under-filled built deck to the builder before Quick Play even
  though Quick Play no longer *uses* that deck; the spec is silent, so the plan
  kept the behavior as a consistency nudge rather than adding a behavior change.
  Dropping it is a two-line change if it ever annoys.
```

with:

```markdown
- **Quick Play keeps today's deck-validity divert.** `open_opponent_select`
  still sends an under-filled built deck to the builder before Quick Play even
  though Quick Play no longer *uses* that deck; the spec is silent, so the plan
  kept the behavior as a consistency nudge rather than adding a behavior change.
  Dropping it is a two-line change if it ever annoys. **Both halves superseded
  by spec 024**: Quick Play deals the built deck, so the divert is no longer a
  nudge but the thing that upholds `start_match`'s deck-valid precondition for a
  deck that *is* dealt — and dropping it would now be a bug, not a two-line
  tidy.
```

### 2d. The spec 022 closing paragraph names Quick Play's deal (currently lines 865–866)

At the end of the spec 022 section's **Supersedes the spec 008 bullet above**
paragraph, replace:

```markdown
already hold), so the measured starter rates describe the deck a fresh player
actually fields, and `card::DEFAULT_SIDE_DECK` is the opponent baseline and
Quick Play's deal. The "exact list is tunable balance data" part of that bullet
```

with:

```markdown
already hold), so the measured starter rates describe the deck a fresh player
actually fields, and `card::DEFAULT_SIDE_DECK` is the opponent baseline and
Quick Play's deal — the latter **superseded by spec 024**, which leaves it the
opponent baseline only. The "exact list is tunable balance data" part of that bullet
```

### 2e. Append a new section at the end of the file

Append after the "## First-run onboarding & controls refinement (spec 023)"
section's last paragraph ("…`PROFILE_VERSION` and `SAVE_VERSION` both stay 1.
Monochrome by construction."):

```markdown
## Endgame, victory & what you keep (spec 024)

Beating the Sovereign did almost nothing: the ordinary win popup, a settled
banner, one muted line, a lifetime counter. Losing had a full notice and a reset
behind it; winning had nothing. Ruled with the human on 2026-09-15, on the
recommendations as proposed, around one principle **in the person's words**:
the profile is a global pool of cards and credits between all modes; a new
player has only the basic deck and no credits to buy anything, plays the
campaign to earn, and once they win they keep everything for Quick Play as
well; New Campaign keeps accumulating, with a separate option to reset
everything.

- **1 — The run continues after the win.** A notice once, then the map stays
  open with rematches and the shop. No new reset path. Ending like a loss would
  throw away the deck built to win; New Game Plus with scaled difficulty stays
  deferred.
- **2 — The award is a victory notice with a run summary, plus one lifetime
  record.** A credit bonus buys nothing after completion; a unique card is an
  engine and balance change.
- **3 — The run-summary backlog item folds in.** One summary, two notices.
- **4 — The map keeps a persistent completed marker** (`★  Campaign complete`
  in place of the rim→core axis label) until the map is reset.
- **5 — Quick Play deals the built deck.** **Supersedes spec 022's ruling**
  ("Quick Play deals the standard (premium) deck", quoted and annotated above),
  which had accepted on record that the deck you build only matters in campaign
  matches. That was the one line spec 022 said would be reversible in one line;
  it was.
- **6 — New Campaign keeps cards and credits; Reset Everything is the full
  wipe.** **Supersedes spec 014's "New Campaign = full fresh start"** (quoted
  and annotated above). The 014 argument — keep the early game and the
  depth-gated economy meaningful — no longer holds: since spec 021 a first clear
  pays only progress and every match is even money, so a replay earns no more
  than rematches already can. Going broke becomes the only thing that takes the
  pool, which is the loss condition's intent. This also answers the open
  casual-versus-roguelike identity question (spec 021's "spec E") in the
  **casual** direction, with going broke as the one roguelike bite.
- **Record flag (session's pick).** The first-clear record counts the **first**
  completion only, so a starter-deck first run is never compared with a
  premium-deck replay. A "best completion" that replays could beat was the
  alternative. A profile whose completions are already above zero never gains a
  record — accepted: this is a record for runs from here on.
- **Not retuning the curve for replays** — a replay with a premium deck is
  easier than the spec 022 curve assumed. It is opt-in and deliberate; **no
  constant moved**, and `docs/balance.md` gained a short *Replays* note saying
  so. Reset Everything is what returns a profile to the run those numbers
  describe.

Design tensions resolved during planning:

- **One `Profile::resolve_match` owns the record-then-settle order.** The
  first-clear number is "the run's matches played *including* the completing
  match", captured on the completion edge inside settlement — but the run tally
  is bumped by `record_match`, which `App::tick` called *after* settling, while
  `profile.rs`'s own tests recorded first. A latent app-versus-tests
  disagreement is exactly what spec 021 removed when it moved the completion
  edge into one method, so `App::tick`'s two profile calls became one
  `resolve_match(opponent_id, player_won, player_rounds, opp_rounds) ->
  Option<Settlement>` that derives the `Mode` itself, records, then settles;
  `record_match` and `settle_campaign_match` are now private to `profile.rs`.
  **Settlement's `Option<StakeOutcome>` signature deliberately stayed put**:
  flipping it would have broken the app's call site and nine `assert_eq!`s in
  the settling tests in the *same* task that added the new method, so the
  data-model task could not have built and tested green on its own (the
  constitution's rule that every task ends green). The cost, accepted and
  written down: **the completion edge is evaluated twice** — once inside
  settlement for the completions counter, once in `resolve_match` for the
  victory signal — pinned by a test asserting the two always agree
  (`campaign_completions` increases on exactly the resolutions that return
  `completed_run: true`, and on no others). The rejected alternatives were a
  `+ 1` inside the edge (correct only under the app's order, silently wrong
  under the tests') and a third app-level `record_first_clear` call (an ordering
  rule spanning three calls, unverifiable without an `App` that writes to disk).
- **The completion signal reaches the notice as a transient App flag.** The
  notice is raised on the *acknowledgement* of the game-over popup, one key
  event after the settlement that completed the run, so `Settlement.completed_run`
  is stored as `App::victory_due` and `mem::take`n by the next
  `enter_campaign_map`. **Not persisted, by design** — a flag on disk would be a
  save-format change for a transient. Consequences, all spec'd: a rematch win
  sets nothing; a replayed campaign's completing win sets it again; quitting
  before acknowledging loses the notice but neither the completion nor the
  payout, both already persisted.
- **The choice panel's breathing room was corrected while it was open.**
  `draw_two_choice` became `draw_choice_panel(title, note, labels, selected,
  hint, pulse)` taking N labels, and its row placement moved into a pure
  `choice_rows(note_present) -> (note_row, choice_row, hint_row, height)`:
  without a note the rows are unchanged (title 0, choices 2, hint 4), with a
  note the choices move to row 3 so the note never sits flush against the
  acted-on row — the design brief's *Density and breathing room* rule, which the
  panel had been quietly missing. **This also corrects the spec-021
  discard-a-save confirm**, the only other note-carrying panel. A correction
  inside a screen this spec already changed, not new scope.
- **`player_deck_for` was deleted rather than kept with one branch.** With Quick
  Play dealing the built deck there is one answer for both modes, so the fn, its
  `is_campaign` argument and the `DEFAULT_SIDE_DECK` / `stats::Mode` imports in
  `app.rs` all went; `start_match` has one deal. Keeping a one-line wrapper
  would be indirection the spec doesn't demand (constitution: *Simplicity*). The
  accepted cost: the claim loses its pure-fn test, because `start_match` writes
  the profile and the save and no App test may touch disk. It is pinned
  **structurally** instead — `app.rs` no longer names `DEFAULT_SIDE_DECK` at
  all, so it *cannot* deal the standard deck — plus a driver run with a
  deliberately non-standard built deck (a ten-card +1/−1 deck dealt
  `-1 +1 +1 -1`).
- **The Reset Everything confirm was retitled.** `spec.md` quoted only the tail
  (`… Erases progress, credits & cards.`); under the three-choice panel the old
  head ("New campaign?") would have named the wrong choice, so the title is
  **"Reset everything? Erases progress, credits & cards."** and New Campaign's
  is "New campaign? Resets the map; you keep your cards and credits." A wording
  decision, not a behavior change.
- **A pre-economy profile document shows the entry panel.** A `profile.json`
  without a `credits` key loads with 0 credits (spec 021's deliberate serde
  default, diverging from `SEED_PURSE`), so `differs_from_starter` is true for
  it and Start Campaign offers the three choices; Continue then meets the
  run-over notice as spec 021 intended. Only a document that *is* the starter —
  a serialized fresh profile — opens the map directly, which is what the
  acceptance criterion says. Ruled by the orchestrator on 2026-09-16 as what the
  spec's own predicate specifies, and asserted by the test.
- **The Records *This Run* view was not extended.** It keeps today's lines; the
  new credit counters and the worlds-cleared figure appear on the two notices
  only. `spec.md` was corrected to say so.

No engine change (`game.rs`, `player.rs`, `save.rs`, `economy.rs`, `wager.rs`
untouched; `card.rs` moved only in **doc comments**, by an acceptance criterion
amended for exactly that), no AI change, no balance data moved,
`tests/balance.rs` / `Cargo.toml` / `Cargo.lock` untouched, and no new crate.
The profile gained three additive `#[serde(default)]` fields
(`RunStats::credits_won`, `RunStats::credits_lost`,
`LifetimeStats::first_clear_matches`); `PROFILE_VERSION` and `SAVE_VERSION`
both stay 1, so a pre-024 profile loads with zero counters and no record.
Monochrome by construction.
```

### 2f. Three small present-tense corrections (sweep note N8)

Cheap, same convention, same paste:

**2f-i — `DECISIONS.md` lines 312–314**, the spec 014 panel-trigger bullet. replace:

```markdown
- **Offer the choice at Campaign entry** — when cleared progress exists, a
  Continue / New Campaign panel (a `Modal` over the menu), not a separate top-level
  menu item. With no progress the map opens directly (the choice would be a no-op).
```

with:

```markdown
- **Offer the choice at Campaign entry** — when cleared progress exists, a
  Continue / New Campaign panel (a `Modal` over the menu), not a separate top-level
  menu item. With no progress the map opens directly (the choice would be a no-op).
  **Spec 024 widened both**: the panel is three choices (Continue / New Campaign /
  Reset Everything) and shows whenever the run has progress *or* the pool differs
  from the starter, so only a truly fresh profile opens the map directly.
```

**2f-ii — `DECISIONS.md` lines 977–979**, the spec 023 primer tension, where the
confirm variant was renamed. replace:

```markdown
  New Campaign is **menu-only** (no `MapOutcome` variant; `ConfirmNewCampaign`
  is raised only from `CampaignEntry`), so no origin flag beyond `from_menu` is
  needed.
```

with:

```markdown
  New Campaign is **menu-only** (no `MapOutcome` variant; the confirm —
  `ConfirmNewCampaign`, renamed `ConfirmReset` and given a scope by spec 024 —
  is raised only from `CampaignEntry`), so no origin flag beyond `from_menu` is
  needed.
```

**2f-iii — `ROADMAP.md` lines 506–507**, the *Archive the last run at reset*
backlog bullet. This one is in `ROADMAP.md`, not `DECISIONS.md`; replace:

```markdown
- **Archive the last run at reset.** Before the run-over (or New Campaign)
  reset wipes the profile, write the outgoing `profile.json` to a dated
```

with:

```markdown
- **Archive the last run at reset.** Before the run-over (or **Reset
  Everything**) reset wipes the profile — since spec 024 New Campaign keeps the
  pool and wipes nothing worth archiving but the run tally — write the outgoing
  `profile.json` to a dated
```

---

## 3. Not drafted here (deliberately)

- **`Readme.md`, `docs/economy.md` and `docs/balance.md` are spec files on the
  branch** — the Quick Play sentence, the Start Campaign panel's trigger and
  choices, the settling seam and the new *Replays* subsection all ride into
  `main` with the merge and need no close-out edit. `docs/economy.md` was also
  **re-synced on the run-over reset** at the sweep: its *Going broke* section had
  said Enter runs `start_new_campaign` — `reset_to_starter` "then a fresh map …
  exactly as for New Campaign" — which is wrong on all three counts now. It now
  says the acknowledgement runs `reset_run` (the `reset_to_starter` wipe plus the
  saved match cleared — **Reset Everything**'s operation since spec 024) and
  lands on the **start menu** per the 2026-09-13 chore, while New Campaign is the
  map-only reset that keeps the pool. (The README is tracked as
  `Readme.md`; a case-insensitive filesystem hides it from `git diff --
  README.md`.)
- **`docs/opponents.md` is untouched** — its "standard deck" references are the
  opponent baseline, which this spec did not change.
- **No `CLAUDE.md` amendment.** Spec 024 changed no rule the constitution
  states; the three-choice panel is still a `Modal` over the menu, the
  acted-on-row rule was *applied* rather than changed, and the verification
  command is unchanged.
- **Tier-log observations stay in `specs/024-endgame-victory/tasks.md`** (the
  Phase 1 N3 test-name note, the Phase 2 N3 accessor note): process evidence for
  the model-policy experiments, not project decisions.
