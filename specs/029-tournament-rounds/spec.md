# Spec: Tournament rounds — spec 029

**Status**: Approved (2026-09-20). Rulings A1, B1, C1, D2, E2, F1, G1, H1,
I1 (the person, 2026-09-20), then J1, K1, L1, M1, N1, O1, P1 and Q
out-of-scope on the consequences of D2 + E2 the same day. **Amended
2026-09-21** (rulings R1, R2, R3) after the person walked the venue — see
*Amendment, 2026-09-21*, which supersedes M1's art sizing and N1 entirely —
and **again the same day** (R4, R5, R6) after they walked the rebuilt
venue: see *Amendment, 2026-09-21 (second)*.
The dated ruling records under *Resolved decisions*, and the **Depends
on** line's reference to spec 012, keep the word "Outfitter": they record
what was decided or shipped then. Everything else — the summary, the
entities, the behavior and the acceptance criteria — says **Card Shop**.
**Depends on**: spec 009 (the campaign map and its `b`/`c` doors), spec 012
(the Outfitter and the depth-gated pool), spec 008 / 015 (the collection and
the briefcase deck builder), spec 016 (opponent portraits), spec 021 (the
per-match wager, escrow, settlement and the run-over condition), spec 022
(the tuned curve and the balance simulator), spec 023 (the first-campaign
primer and How to Play's campaign section), spec 024 (the endgame and the
run summary), spec 026 (the 89×31 minimum and the 139-column threshold)

## Summary

Today a single match win defeats a campaign opponent. Ten opponents, ten
matches, and the campaign is over — and one lucky match can take a world.

This spec makes each campaign opponent a **series**. Every opponent is
beaten by winning **2 matches out of 3**; the final opponent, The
Sovereign on Zenith, by winning **3 out of 5**. A match is unchanged —
still first to 3 round wins — and each match in a series is staked,
prompted and settled exactly as it is today.

Between the matches of a series the player is at the **venue**: a new
screen standing for the tournament hall on that planet, where they start
the next match, visit the Card Shop, open their collection, or quit to the
menu. The venue is where every match of a series begins. It reserves a region
for per-planet art **beside** the opponent's portrait — two separate
elements, so that one later spec can author the art and another can show
more than one opponent on a planet, neither needing a layout rework.

Starting a series **commits** the player to it. While one is in progress
the map is not reachable and no other planet can be played; entering the
campaign goes straight to the venue. Losing a series costs only the stakes
already lost: the score resets to 0–0, the opponent stays un-beaten, and
the player is returned to the map free to go anywhere again.

Rematches on cleared planets stay what they are — a single match, launched
from the map, for credits.

No engine, AI, save-format or economy-constant change. Monochrome by
construction.

## Goals

1. **The campaign is longer, and a world is earned rather than drawn.**
   The minimum path goes from 10 matches to 21, and the expected path to
   roughly 25. A single lucky match no longer clears a planet.
2. **The difficulty curve sharpens in the direction it was already
   pointed.** A best-of-three converts a per-match edge into a larger
   series edge and a per-match deficit into a larger deficit, which is
   what spec 022's gates were tuned to want.
3. **A series is a commitment.** Once started, it is played out or lost;
   there is no banking a win and going elsewhere.
4. **The between-match moment is a place, not a menu.** The venue reads as
   somewhere the player is standing, and is built so that per-planet art
   and a second opponent each drop into it later without a layout rework.
5. **Nothing about playing a match changes.** The board, the keys, the
   phases, the wager prompt and the settlement are what they are today.
6. **It all works at 89×31.**

## Non-goals (explicitly deferred)

- **Per-planet venue art.** This spec reserves the region and fills it
  with a placeholder; authoring the art is its own spec, run the way spec
  016's portraits were — a brief in the repo, the art drawn by a more
  capable tool, Claude Code validating and integrating.
- **Per-planet music.** Ruled out of scope by the person; its own spec.
- **Series-aware banter.** An opponent's match-start line will now fire
  two or three times in a row. Accepted; a roadmap follow-up.
- **New records or statistics.** The existing counters keep counting
  matches. No series-won counter, no series-lost counter, nothing new on
  the run summary or the Records screens.
- **Rematches as series.** A rematch stays a single match (the grinding
  path is meant to be low-friction).
- **Abandoning a series.** There is no forfeit action. Quitting to the
  menu and returning puts the player back at the venue, mid-series.
- **Re-tuning the curve.** This spec measures the new series-level rates
  and records them; changing an economy constant, a roster value or the
  starter deck in response is a later spec.
- **Any change to the match engine, the opponent AI, the save format, the
  wager arithmetic, or the balance data.**

## Entities

- **Series** — the unit that defeats a campaign opponent: a run of matches
  against one opponent on one planet, won by the first side to reach the
  required number of match wins. **Best of three** (first to 2) for every
  opponent except The Sovereign, which is **best of five** (first to 3).
  A match always produces a winner, so a series always resolves.
- **Series score** — the two match-win tallies of the series in progress,
  the player's and the opponent's. Held in the profile beside the existing
  campaign progress, additive and defaulted, and cleared by the same reset
  paths that clear the rest of a run.
- **The lock** — at most one series is in progress at a time, and while
  one is, it is the only campaign match the player can play.
- **The venue** — a new top-level screen: the tournament hall on the
  planet where the series is being played. It is where a series begins,
  where the player returns after every match that does not end the series,
  and the only door to the Card Shop and the collection while locked.
- **The planet art region** — the venue's **dominant element**: a region
  reserved for art of the venue itself, as wide and as tall as the screen
  allows once the text has its rows, drawn at **every** width. In this
  spec it holds a plain placeholder; a later spec replaces its contents
  without changing the layout around it. (Amended by the person
  2026-09-21, superseding rulings M1 and N1 — see *Amendment, 2026-09-21*.)
- **The opponent portrait** — the existing portrait art (spec 016), drawn
  at the venue **beside** the planet art region as its own element, not
  inside it. The two are separate because they answer to different things:
  the art is the planet's and does not change with the opponent, and a
  later spec may show more than one opponent on a planet.

## Key behavior

### Starting a series

- From the map, Enter on an unlocked planet that still has an un-beaten
  opponent opens the **venue** for that planet's next opponent, with the
  series at 0–0. No match starts and nothing is staked by arriving.
- The map's planet detail says what a launch commits the player to —
  **Best of 3**, or **Best of 5** for The Sovereign — so the commitment is
  visible before it is made.
- A series is in progress from the moment the venue is opened this way.

### The venue

- It names the planet and the opponent, and shows the **series score and
  its length** — the player's match wins, the opponent's, and how many are
  needed.
- It offers four actions, cursor-selected with Enter to confirm: **play
  the next match**, **the Card Shop**, **the collection**, and **quit to
  the main menu**. `b` and `c` also open the Card Shop and the collection,
  as they do on the map.
- The Card Shop and the collection **return to the venue**, not to the
  map.
- The constitution's density rule applies: the action the player is about
  to take gets an empty row above and below it; the rest stays compact.
- The venue draws the **planet art region** with the **opponent's
  portrait beside it**, as two distinct elements with room between them,
  at **every** width. The art takes the height left over once the text has its rows, and most
  of the width left over once the portrait has its column — **most**, not
  all, since ruling **R4** trimmed it (see *Amendment, 2026-09-21
  (second)*). It still dominates the screen. The art region holds a plain
  placeholder in this spec. (Amended by the person 2026-09-21 — see
  *Amendment, 2026-09-21*.)
- The rows sit **above and below the art**, not beside it: the place, the
  planet, the opponent, the series score and the **credit balance** above;
  the action row and the controls hint below. Every one of them centres on
  the **art region's** horizontal centre rather than the terminal's
  (ruling R5), because the portrait's column makes those two different
  points and text centred on the terminal reads as shifted off the art.
- At 89×31 the venue is fully legible, with no horizontal overflow and
  nothing clipped.

### Playing a match from the venue

- Choosing to play opens the **wager prompt** as it is today: the same
  ante floor for that opponent, the same player-chosen stake up to the
  full balance, the same escrow at launch. Declining the prompt returns to
  the venue.
- The match itself is unchanged in every respect.
- During a series match the **series score is visible** at both layout
  widths. A rematch match shows no series score, because it is not part of
  a series.

### Ending a match

- The stake settles exactly as it does today — even money on a win, lost
  on a loss — and the match is recorded in the lifetime and run tallies as
  it is today.
- The winner's series tally goes up by one.
- **If the series is not yet decided**, acknowledging the game-over popup
  returns the player to the **venue**, which shows the updated score.
- **If the match decided the series**, acknowledging the popup returns the
  player to the **map**, whose banner names the series result alongside
  the stake settlement it already reports.

### Ending a series

- **Won** — the opponent is marked beaten for the first time at that
  moment. The planet clears when all its opponents are beaten, the next
  planets unlock, and campaign completion is counted once, all exactly as
  they are today. The lock is released.
- **Lost** — the series score is discarded, the opponent stays un-beaten,
  and the planet does not clear. Nothing further is taken: the cost of a
  lost series is the stakes lost in its matches. The lock is released and
  the player may start the series again, from 0–0, or go anywhere else.

### The lock

- While a series is in progress the **map is not reachable**. Entering the
  campaign from the main menu goes straight to the venue, and no other
  planet — including a cleared one offering a rematch — can be played.
- Quitting to the main menu from the venue leaves the series in progress.
  Returning to the campaign lands back at the venue with the same score.
- The lock is released only by the series resolving, or by a reset (New
  Campaign, Reset Everything, or the run ending), which clears it with the
  rest of the run.

### Going broke while locked

Spec 021 ends a run when the player cannot cover the cheapest ante on the
map. While locked into a series that rule would be wrong — the cheapest
ante may be on a planet the player is not allowed to play. So:

- **While a series is in progress, "broke" means the player cannot cover
  the ante floor of the opponent they are locked against**, checked at the
  same two seams as today: after a campaign match settles, and on entering
  the campaign.
- **The Card Shop reserves that same floor** while locked, so shopping can
  never strand the player, exactly as it can never strand them today.
- Everything else about the run-over flow is unchanged: the modal, the
  reset, the preserved lifetime records.

### Rematches

Unchanged. A cleared planet still offers its final opponent as a single
match, launched from the map, staked and settled as today, with no venue
and no series. The opponent stays beaten either way.

### Saving and resuming

- The series score and which series is locked live in the profile with the
  rest of the campaign run, and survive quitting and returning.
- A match saved mid-play resumes as it does today; finishing it then
  routes by the rules above.
- A profile saved before this spec loads with every series at 0–0.
  Opponents already beaten stay beaten, cleared planets stay cleared, and
  a match left in flight resolves as the first match of a fresh series
  against that opponent.

### What the player is told

- The **first-campaign primer** (spec 023) gains a line: an opponent is
  beaten by winning two matches out of three, and the final opponent three
  out of five.
- **How to Play**'s campaign section says the same, and that the player is
  committed to a series once it starts.

## Acceptance criteria

> All twenty checked at the close-out (2026-09-22). The evidence for each —
> the walkthrough rows, the tests, T010's simulator run, and four recorded
> caveats (AC 3, 6, 11 and 16) — is in `closeout-main-docs.md` §6.

1. [x] **Series length.** Every campaign opponent except The Sovereign is
   defeated by 2 match wins and no fewer; The Sovereign by 3. Winning
   fewer leaves the opponent un-beaten and the planet uncleared.
2. [x] **Starting a series.** Enter on a playable planet opens the venue at
   0–0 without staking anything; the map's planet detail names the series
   length before the launch.
3. [x] **The venue.** It shows the planet, the opponent, the series
   score and length, and the **credit balance**; it offers play / Card Shop
   / collection / quit, with Enter confirming and `b` and `c` working as on
   the map; the Card Shop and the collection each return to the venue.
   (Renamed from "Outfitter" by ruling R2 — see *Amendment, 2026-09-21*.
   The credit balance added by ruling **R6** — see *Amendment, 2026-09-21
   (second)*; caught at that amendment's planning, which found R6 changing
   behavior with no criterion naming it.)
4. [x] **Every match of a series starts at the venue**, through today's
   wager prompt, with today's floor, stake range and escrow; declining
   returns to the venue.
5. [x] **Between matches.** After a match that does not decide the series,
   acknowledging the game-over popup lands on the venue with the score
   updated by one for the winner.
6. [x] **Deciding match.** After a match that decides the series,
   acknowledging the popup lands on the map, whose banner names the series
   result as well as the stake settlement.
7. [x] **Winning a series** marks the opponent beaten exactly then; the
   planet clears, the next planets unlock and a campaign completion counts
   once, unchanged from today.
8. [x] **Losing a series** resets the score to 0–0, leaves the opponent
   un-beaten and the planet uncleared, takes nothing beyond the stakes
   already lost, and returns the player to the map.
9. [x] **The lock.** With a series in progress, entering the campaign goes
   to the venue, the map cannot be reached, and no other planet — cleared
   or not — can be launched.
10. [x] **Broke while locked** is judged against the locked opponent's ante
    floor, at the same two seams as today, and the Card Shop reserves that
    floor; the run-over modal and reset are unchanged.
11. [x] **In-match score.** The series score is visible during every series
    match at 89×31 and at 139×31, and is absent during a rematch.
12. [x] **Rematches** on a cleared planet launch from the map as a single
    staked match, with no venue, and settle as they do today.
13. [x] **Save and resume.** Quitting at the venue and returning lands at
    the venue with the same score; a match saved mid-play resumes and then
    routes by criteria 5 and 6; a pre-029 profile loads with all series at
    0–0 and every beaten opponent still beaten.
14. [x] **Reset paths.** New Campaign, Reset Everything and the run-over
    reset each clear the series score and the lock along with the rest of
    the run.
15. [x] **Records and statistics are unchanged.** Match counters still count
    matches, campaign completion still counts once, and no new counter
    appears anywhere.
16. [x] **Both layouts.** The venue is legible at 89×31 with no overflow and
    nothing clipped, and at 139×31 likewise. At **both** widths it draws the
    planet art region holding its placeholder, with the opponent's portrait
    beside it as a separate element, neither overlapping nor clipped. The
    art region is the largest element on the screen at both widths, and is
    strictly larger at 139 than at 89. (Amended by the person 2026-09-21,
    superseding the original wide-only form.)
17. [x] **Density.** The venue's acted-on row has an empty row above and
    below it and the rest of its text stays compact; any modal it shows
    pads evenly.
18. [x] **What the player is told.** The first-campaign primer and How to
    Play's campaign section both state the two-of-three rule, the
    three-of-five final, and the commitment.
19. [x] **Balance measured, not changed.** The simulator is re-run and
    `docs/balance.md` records the series-level win rates implied by the
    measured per-match rates, for best-of-three and for the best-of-five
    final. No economy constant, roster value or starter-deck entry changes
    in this spec.
20. [x] **No forbidden change.** No behavior change in `game.rs`, `card.rs`,
    `player.rs` or `save.rs`; `SAVE_VERSION` and `PROFILE_VERSION` both
    stay 1; no new crate; no color path.

## Amendment, 2026-09-21 — the venue's wording and its art

The person walked the venue at the Phase 2 pause, approved the flow, and
ruled three changes to what it shows. Recorded here before any of it was
built, because the third supersedes two of this spec's own rulings and
rewrites an acceptance criterion.

### R1 — the series length reads "Best of 3", not "first to 2"

The venue's series line says `Series  1 – 0   ·   Best of 3`, and `Best of
5` against The Sovereign — the same words the map's planet detail uses, so
the player meets one phrase for one idea rather than two. "first to 2" was
a session choice, never a ruling.

### R2 — the shop is the "Card Shop" everywhere, not the "Outfitter"

The person found "Outfitter" unclear. The rename is **not** cosmetic to one
button: it covers the venue's action label, the shop screen's own header,
the map's and the venue's key hints where they name it, How to Play, the
README and `docs/economy.md`. A button that says one thing and opens a
screen titled another reads as a defect, which is the same reasoning that
made this spec rename four symbols rather than comment them.

Earlier specs' own documents (`specs/012-*`, `DECISIONS.md` entries) keep
the word: they are the record of what those specs did, and rewriting them
would falsify history rather than correct it. **This spec's own `spec.md`
is not in that category** — it is the live contract, so every one of its
live-prose mentions is corrected rather than preserved: the summary, the
venue entity, the venue's four actions, the return-to-the-venue bullet,
the Card Shop's reserve under *Going broke while locked*, and acceptance
criteria 3 and 10. Leaving them would have the person tick a criterion
naming a button the build does not have, which is the document-level form
of exactly the defect R2 exists to remove. (Caught at the amendment
sign-off, 2026-09-21: the plan revision had generalized this carve-out
from *earlier* specs to all of `specs/**`.)

### R3 — the art dominates the screen, at every width

The reason, in the person's words: a much larger canvas allows more
detailed, immersive art, and makes it feel like you are actually in the
venue, on the planet. So the venue becomes **horizontal bands** rather
than text-with-a-rail:

- the place, the planet, the opponent and the series score, above the art;
- the **art region** and the **opponent's portrait beside it**, taking
  the rows and most of the columns the text does not need (ruling **R4**
  trimmed the width; see *Amendment, 2026-09-21 (second)*);
- the action row and the controls hint, below the art.

Two consequences the person ruled on directly when asked:

- **The portrait keeps its own column.** It is not inset into the art and
  it is not dropped. M1's reason survives intact — the art is the
  planet's, the portrait is the opponent's, a planet may later hold more
  than one opponent — and the art takes most of what the portrait
  does not — trimmed by ruling **R4**, below.
- **It draws at 89 columns too**, superseding N1. A text-only venue at the
  minimum size is the version this amendment least wants. The cost, named
  because it lands on the deferred art spec rather than here: each planet's
  art must work at two quite different sizes.

**Still not in scope**: authoring the art. The region holds a plain
placeholder in this spec. Whether it carries the planet's name was the open
choice this amendment left standing; the person **closed it the same day** —
keep the label until there is real art. The *brief* for that art is a
deliverable of this spec; see *Amendment, 2026-09-21 (second)*.

## Amendment, 2026-09-21 (second) — the venue's proportions, its alignment, and its balance

The person walked the rebuilt venue and ruled three refinements plus one new
deliverable. R3 stands; these tune it.

### R4 — the art is about 15–20% smaller

Dominant, but not quite this dominant. The exact numbers are the plan's to
work out and the person's to eyeball at the next pause; the brief is
"about 15–20% smaller overall" than R3's first attempt (58×23 at 89
columns, 108×23 at 139).

### R5 — the venue's text aligns to the art, not to the terminal

The person's words: *the text should be centered above the art, not
centered generally. The way it is now looks off-centered.* And they are
right about the cause — the portrait's column sits to the right of the
art, so the band's centre is well right of the art box's centre, and text
centred on the terminal reads as shifted off the thing it labels.

So **every** row of the venue's text — the header rows above the art and
the action row and hint below it — centres on the **art region's**
horizontal centre. Above and below alike: aligning only the header would
trade one visible mismatch for another.

One consequence the plan must handle rather than discover: at 89 columns
the art can never be as wide as the controls hint, because the portrait's
column and its gap take 25 of the 89. So either the hint overhangs the art
or it gets shorter. It must not end up flush against column 0, which reads
as a rendering fault.

### R6 — the venue shows the credit balance

It is the screen where the player chooses between playing and shopping,
which is the choice a balance informs. (Asked at the Phase 2 pause,
answered here.)

### The art placeholder stays as it is

Also answered: the placeholder keeps the planet's name until there is real
art. Plan §Open questions 2 is **closed** — keep the label.

### R7 — a consistent, slight gap between the art and the portrait (2026-09-22)

After walking the result: *the portrait is just a little bit too far to the
right of the venue art … it would be better to have a consistent margin
between the two so that it always sits the same way away (and not too far).*
So the gap between the art region and the portrait is **the same small
number of columns at every width**, and the art-plus-portrait group sits
centred on the screen with equal outer margins. R4's art size and R5's
text alignment are unchanged — only the portrait moves.

### R8 — How to Play says "matches" (2026-09-22)

At the Phase 4 pause the person chose to spell out the unit, since How to
Play opens with a rule about *rounds*: the campaign lines read "Each opponent
is Best of 3 matches, the last / Best of 5. A started series is played out."

### A new deliverable: the per-planet art brief

Authoring the art remains a **non-goal** of this spec — but the **brief**
for it rides this branch, so the work can be handed to a design agent and
folded back in. It specifies what each planet's art must be, the exact
dimensions and character constraints it must satisfy, how the files are
named, where they live in the project, and how Claude Code validates and
integrates them. Written the way spec 016's portrait brief was.

This is documentation, not behavior: no acceptance criterion changes for
it, and the spec's *Non-goals* entry for per-planet art stands.

## Resolved decisions (the person, 2026-09-20)

- **A1 — the new unit is a "series."** "Round" is already the within-match
  unit; "tournament round" would collide with it.
- **B1 — each match is staked separately**, exactly as spec 021 has it.
  Keeps escrow, settlement and the broke check intact, and makes the
  between-match Outfitter visit worth something: a win pays before the
  next match starts.
- **C1 — losing a series costs only the stakes already lost.** The score
  resets, the opponent stays un-beaten, nothing further is taken.
- **D2 — a venue screen between matches**, rather than returning to the
  map. The person's reason: it is a place, and an opportunity for art that
  raises the immersion of the campaign.
- **E2 — the player is locked into a series once it starts.**
- **F1 — rematches stay single matches.**
- **G1 — best of five applies to The Sovereign only**, not to the Core
  opponents generally.
- **H1 — no new records or statistics.**
- **I1 — the series is visible on the map and in the match.** With E2 and
  P1 the map never shows a live score, so this resolves as: the map shows
  the series **length** before a launch, and the venue and the match board
  show the **score**.
- **J1 — the map launches to the venue at 0–0**, and every match of the
  series starts from the venue.
- **K1 — the venue offers play, Outfitter, collection and quit**, with the
  Outfitter and collection returning to the venue. No abandon action.
- **L1 — rematches keep today's flow**: map → wager → match → map.
- **M1 — the art region is reserved now and filled with a plain
  placeholder**; authoring per-planet art is its own spec. **Amended by
  the person the same day**: the placeholder stays plain, and the
  opponent's portrait sits **next to** the region rather than inside it.
  Their reason: the eventual venue will show planet art *and* an opponent
  portrait side by side, and a planet may later hold more than one
  opponent — so the two elements are laid out separately now, and neither
  later spec has to rework the layout. **Amended again 2026-09-21** after
  the person saw the venue running: side by side stands, but the art is
  **much larger** and the text moves above and below it. See *Amendment,
  2026-09-21*.
- **N1 — the art region draws at 139 columns and wider only**, matching the
  presence panel's rule. **Superseded 2026-09-21**: it draws at every
  width, because the art is the point of the screen and a text-only venue
  at 89 columns is the version the person least wants. See *Amendment,
  2026-09-21*.
- **O1 — while locked, broke is judged against the locked opponent's ante
  floor**, and the Outfitter reserves that floor.
- **P1 — entering the campaign mid-series goes straight to the venue**; the
  map is not reachable while locked.
- **Q — per-planet music is out of scope** and becomes its own spec.

### Flagged to the person with the rulings

- **Best-of-three amplifies the curve spec 022 tuned.** A 45 % per-match
  rate becomes a 42.5 % series; 60 % becomes 64.8 %. The Sovereign at 33 %
  per match becomes a **20.5 %** best-of-five series. This is the
  direction the gates were tuned to want, and it is a real sharpening —
  measured and recorded here (criterion 19), re-tuned later if it plays
  badly.
- **The lifetime first-clear record stops being comparable** with anything
  set before this spec, since it counts matches to clear the campaign and
  the campaign now takes roughly 2.4× as many. Accepted; the record is not
  reset.
- **Banter will repeat** two or three times per opponent. A non-goal here,
  logged to the roadmap.

### Session calls within a delegated ruling

- **The map shows the series length, not a score.** I1 asked for the
  series to be visible "on the map and in the match," but E2 and P1 mean
  the player is never on the map while a series is live, so there is no
  live score for it to show. Resolved as the length before a launch
  ("Best of 3"), with the score on the venue and the board. Confirmed by
  the person, 2026-09-20.
