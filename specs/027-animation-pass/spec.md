# Spec: Animation pass — spec 027

**Status**: Approved (2026-09-18; Q7 added at planning; **Revision 1** at the
Phase 1 pause the same day — Q8–Q10; **Revision 2** at the Phase 1b pause,
2026-09-19 — Q11, the flip removed; see the end of this file). Rulings Q1 a–e;
Q2 A, Q3 A, Q4 A, Q5 A, Q6 A delegated to the session and taken as
recommended, under the person's standing constraint (Goal 1); Q2 A is
superseded for dealer cards by Q8.
**Depends on**: spec 002 (the selection pulse and the Motion rule in
`design/brief.md`), spec 004 (the settings overlay and its config file),
spec 017 (the frame-to-frame snapshot diff that banter and audio use),
spec 026 (the compact and wide board layouts)

## Summary

Today exactly one thing moves on a still screen: the current selection
breathes between two emphasis states every half second, on one shared
clock. Everything else on the match board appears instantly. A dealt card,
the opponent's whole move, a changed total and the round-outcome popup all
land on the same frame as the state change that caused them, so a fast
round reads as a jump cut: the player looks up and the board is simply
different.

This spec adds **sparse, one-shot transitions** on the match board that
guide the eye to what just changed: a card arriving on either side draws
bold for a beat before it settles; a total that changed draws bold for a
beat; the round-outcome and game-over popups arrive a beat after the last
card instead of covering it; and the opponent's thinking pause carries a
small ticking indicator so it reads as deliberate rather than as a hang.
Every transition is a change of **emphasis over time**, the vocabulary the
design brief already set for this pass: no sweeps, no particles, no
ambient motion. Everything is drawing only. The engine, the phase timing
and the keys are untouched, and no key is ever delayed by a transition.
An **Animations** row in Settings turns the whole layer off.

No engine, AI, economy, save-format or balance change. Portraits stay
static.

## Goals

1. **Noticeable, never in the way.** The person's standing constraint:
   each transition is visible enough to add to the game, and short enough
   that a player playing quickly never waits for one and never has a key
   ignored or deferred because of one. Every key that works today works on
   the same frame it works on today.
2. **The eye lands on what changed.** After any state change on the board
   the player can tell, without reading the whole board, which card just
   arrived, that a total moved, and that the opponent is deciding.
3. **One vocabulary.** Every transition is an emphasis change over a short,
   fixed time on an element that already exists at its final position.
   Emphasis here means the four text levels *and the border weight* — a
   card may land with the heavy border the hand cursor already uses and
   settle to its resting border (Revision 1). No element moves across the
   screen, nothing appears in stages and nothing is redrawn in a different
   shape (the face-down flip Revision 1 allowed here was withdrawn by Q11).
4. **Stillness stays the default.** Outside the moments listed under *Key
   behavior*, the board is exactly as still as it is today; the selection
   pulse stays the only continuous mover, and it keeps breathing while a
   transition runs.
5. **It can be turned off.** A player who wants no motion beyond the
   selection pulse switches Animations off in Settings and the board draws
   as it does before this spec, frame for frame — with one standing
   exception, Q7 below: the Score figure rests at normal weight from this
   spec on, On or Off, where it was bold before.

## Non-goals (explicitly deferred)

- **Anything outside the match board.** Menus, the Outfitter, the deck
  builder, the map, the wager prompt, the shop and the records screens do
  not change. No purchase flash, no credit-counter flash.
- **A face-down reveal** (a new card showing `?` for a beat and flipping to
  its value). Revision 1 (Q8) let a dealer card flip; Revision 2 (Q11)
  withdrew it after the person saw it — a card's content never changes
  after it is drawn, on either side. Q2 A stands for every card.
- **A stake flash** at game over; the presence panel's stake line, round
  pips and banter line are untouched.
- **Portrait animation.** Spec 016 deferred "swapping portrait frames on
  the pulse". The brief's portrait amendment says portraits are static, no
  animation ever; that stands, and the deferral is closed rather than
  reopened. (Q5 A; the roadmap notes it at merge.)
- **Departure transitions.** Cards clearing at the end of a round and the
  popup closing are instant, as today. (A hand slot emptying is no longer
  instant: it shows the source ghost, Q9 — a landing cue, not a departure.)
- **Holding the selection pulse** while a transition runs (Q3 B declined).
- **A reduced-motion setting that also stops the selection pulse or the
  campaign map's starfield.** The Animations row governs this spec's
  transitions only.
- **Any change to the opponent's thinking time**, the phase machine, the
  save format, the profile, the economy, the AI or balance data.

## Entities

- **Transition** — a one-shot emphasis change on one board element,
  started by an observed change and running for a fixed **beat**, after
  which the element draws as it does today. Transitions are drawing state
  only: they are never saved, never affect the engine, and are discarded
  when the board leaves the screen.
- **Beat** — the duration of a transition. There are two lengths: the
  **arrival beat** (a card arriving, a total changing, a source ghost
  showing) and the **popup beat** (the wait before an outcome popup draws).
  Both are constants the planner picks within the bounds under *Timing*,
  and both may be tuned at the phase walkthrough within those bounds. (A
  third, the flip beat, existed between Revisions 1 and 2.)
- **Source ghost** — the hand slot a side card was just played from, drawn
  for the arrival beat as a full single-line outline with an empty face at
  Normal emphasis (Revision 1, Q9). Today an emptied slot draws nothing;
  the ghost is the "it came from there" half of a source-and-destination
  pair, with no motion between them.
- **Thinking indicator** — the opponent-turn status line during the
  opponent's thinking pause, with a small element that steps visibly
  through the pause.
- **Animations setting** — a third row in Settings, On or Off, saved with
  the other settings. Default On.

## Key behavior

### What transitions, and when

All of the following apply to **both sides** of the board, in **both
layouts** (wide and compact, spec 026), and only while the match board is
on screen.

1. **A dealer card arriving** (ruling a; Revision 1, Q8; Revision 2, Q11).
   When a card is dealt to a side's dealer row, that card draws with the
   **heavy border** and Strong emphasis from the first frame it exists,
   its value visible from that frame; when the arrival beat ends it
   settles to today's look (single border, Normal). The rest of the row is
   unchanged.
2. **A side card played** (ruling b; Revision 1, Q9). When a card lands in
   a side's played row, that card draws with the **heavy border** and
   Strong emphasis for the arrival beat, then settles to its resting look
   (the double border, Normal). At the same time the **hand slot it came
   from** draws the **source ghost** — a full single-line outline, empty
   face, Normal emphasis — for the arrival beat, then blanks as today.
   Both sides: the opponent's hidden `?` slot empties into the same ghost.
   When the opponent draws and plays in the same move, the dealt card, the
   played card and the ghost transition at once. Nothing moves between the
   hand and the board.
3. **A total changing** (ruling c). When a side's **Score** figure in the
   header changes value, the figure draws Strong for the arrival beat,
   then settles. The **Rounds won** figure and the slot counter do not
   transition. The over-20 alert is unchanged.
4. **The popup beat** (ruling d). When the round ends, the round-outcome
   popup is **not drawn** until the popup beat has elapsed since the round
   resolved, so the last card that decided the round is seen uncovered,
   with its own arrival highlight. The same holds for the game-over popup.
   The keys the popup announces work from the first frame regardless:
   pressing `n` during the beat advances to the next round at once and the
   popup simply never appears; `g` and `x` likewise at game over. Nothing
   else is held: the state is `AwaitingNextRound` or `GameOver` from the
   frame it is today, saves happen when they happen today, banter and
   audio fire when they fire today.
5. **The thinking indicator** (ruling e). During the opponent's thinking
   pause the status line reads `Opponent's Turn` as today with a trailing
   indicator that steps at least twice within the pause: `Opponent's Turn
   .`, `Opponent's Turn ..`, `Opponent's Turn ...`, cycling. The line keeps
   its Muted emphasis. It reverts to today's text the moment the opponent
   acts. On the compact layout the indicator stays clear of the stake at
   the right end of the band (the band's upper row carries the alert and
   the stake; the indicator is on the base row with the prompt, so they
   never meet).

### What does not transition

- The first frame of a match, whether new, a rematch, or a **resumed
  save**, is drawn settled: nothing on it is in transition except the
  selection pulse, and a resumed match already at the round-outcome popup
  draws that popup at once. Transitions are started only by changes
  observed while the board is on screen.
- The hand: a new hand dealt at match start does not transition. The
  selected hand card keeps its heavy breathing border exactly as today.
- Cards clearing and the popup closing: instant. (A hand slot emptying
  shows the source ghost, Q9.)
- The presence panel (portrait, name, banter, pips, stake), the header's
  names and `Rounds won`, the divider, the ghost slots.

### Concurrency and interruption

- Transitions are independent. A player hitting twice in quick succession
  has two cards in transition, each settling on its own clock. A new
  transition never cuts an older one short, and an older one never delays
  a newer one.
- A state change during a transition is drawn on the frame it happens, as
  today; the transition just continues around it. If the element in
  transition ceases to exist (the round ends and the row clears), the
  transition is simply discarded.
- The selection pulse keeps its cadence throughout. `design/brief.md`'s
  Motion section gets one added sentence under this spec: the one-thing-
  moves rule counts continuous motion; a one-shot emphasis transition may
  run alongside the pulse. (Q3 A.)

### Timing

- The **arrival beat** is at least one selection-pulse period (500 ms) and
  at most one second.
- The **popup beat** is at least the arrival beat, so the deciding card's
  highlight is seen in full before the popup covers it, and at most one
  second.
- The **thinking indicator** steps at a cadence that shows at least two
  distinct states within the opponent's one-second pause.
- No transition ever lengthens the opponent's pause, delays a phase
  change, or defers a key.

### The Animations setting

- Settings gains a third row, **Animations**, below Sound FX, showing
  `On` or `Off`. `←`/`→` (and `a`/`d`) toggle it; `↑`/`↓` reach it; the
  overlay's other rows and keys are unchanged.
- It is saved in the existing settings file with the music and sound
  volumes. A settings file without the key reads as **On**. A file with
  it Off starts the game with it Off.
- **Off** means: cards and totals draw settled from the first frame — no
  heavy border, no source ghost, no Strong — the outcome
  popups draw on the same frame as today, and the thinking line is
  today's static `Opponent's Turn`. The board's frames are identical to
  the game's frames before this spec, apart from the Score figure resting
  at normal weight (Q7). The selection pulse and the map's starfield are
  unaffected.
- The setting takes effect immediately, including on a match in progress.

## Acceptance criteria

1. [x] **Dealer card arrival** (Revisions 1 and 2). On a hit, the new card in
   the player's dealer row is drawn with the heavy border and Strong
   emphasis on the frame it appears, its face reading its value from that
   frame, and it is drawn single-border Normal once the arrival beat has
   elapsed; the cards already in the row are single-border Normal
   throughout. The same for a card dealt to the opponent.
2. [x] **Played card arrival** (Revision 1). On a play, the card in the played
   row is drawn heavy-border Strong for the arrival beat, then
   double-border Normal, on both sides, and its face shows its value from
   the first frame; the hand slot it came from draws the source ghost (a
   full single-line outline, empty face, Normal) for the arrival beat and
   is blank after; when the opponent draws and plays in one move, the
   dealt card, the played card and the ghost transition together.
3. [x] **Total change.** When a side's Score changes, that figure is Strong for
   the arrival beat, then Normal; a Score that did not change is never
   Strong; `Rounds won` never transitions; the over-20 alert is unchanged.
4. [x] **Popup beat.** On the frame a round resolves the popup is absent and
   the deciding card is visible and Strong; once the popup beat has
   elapsed the popup is drawn; `n` pressed during the beat starts the next
   round on that frame. The game-over popup behaves the same with `g` and
   `x`. The phase, the save and the banter/audio events are exactly as
   before this spec (existing tests untouched).
5. [x] **Thinking indicator.** During the opponent's thinking pause the status
   line shows at least two different indicator states across the pause,
   keeps Muted emphasis, and reverts to the plain text once the opponent
   has acted. On the compact layout the line never reaches the stake.
6. [x] **Keys are never delayed.** Every key handled on the board today has
   the same effect on the same frame with a transition running; no
   `GamePhase` variant, timing constant or `apply_*` method changes
   meaning. The opponent's pause is still `OPPONENT_THINKING_TIME_MS`.
7. [x] **Settled first frame.** The first drawn frame of a new match, a
   rematch and a resumed save has no element in transition, and a resumed
   match at `AwaitingNextRound` shows its popup on that first frame.
8. [x] **The Animations setting.** Settings shows an Animations row that
   toggles On/Off with `←`/`→`, is saved, defaults to On, reads On from a
   file without the key and Off from a file with it Off; with it Off the
   board's frames are identical to a settled draw (no heavy-border or
   Strong card, no source ghost, no Strong
   score, popup on the resolving frame, static thinking line);
   toggling it mid-match takes effect on the next frame.
9. [x] **Both layouts.** Criteria 1–5 hold at 89×31 and at 139×31; the
   presence panel is unchanged at 139.
10. [x] **The pulse keeps breathing.** The selection's emphasis alternates at
    its cadence while a transition runs; `design/brief.md`'s Motion
    section carries the one-sentence amendment.
11. [x] **Timing bounds.** The arrival beat and the popup beat are named
    constants within *Timing*'s bounds, and the popup beat is not shorter
    than the arrival beat.
12. [x] **No forbidden change.** No change to `card.rs`, `player.rs`,
    `save.rs`, `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`,
    `campaign_map.rs`, `opponent.rs`, `tests/balance.rs`, `Cargo.toml` or
    `Cargo.lock`; `game.rs` has no behavior change and its existing tests
    pass untouched; `SAVE_VERSION` and `PROFILE_VERSION` stay 1. No new
    crate. No color path: the only attributes emitted are the four
    existing emphasis levels.
13. [x] **README.** The settings mention in `Readme.md` names the Animations
    row.

**Checked off at T005 (2026-09-19)** — evidence: 1, 2, 3 and 9 by the
board and motion tests (`board.rs`, `motion.rs`, both layouts) and the Phase
1 and Phase 1b driver walkthroughs at 89×31 and 139×31 (heavy landing and
value visible on the first frame, settled by 0.75 s; the source ghost with
no number key, blank after; the opponent's card and Score landing together;
the Score plain when a dealt 0 left it unchanged; `Rounds won` never bold;
the panel unchanged at 139); 4 by the popup and app tests and the Phase 1
walkthrough (no popup at two 0.4 s samples, popup by 0.8 s, `n` on the
resolving frame with no popup ever drawn, game-over popup absent at 0.1 s
and present at 1.1 s); 5 by the app tests and the walkthrough (dots
stepping, plain line once the opponent acted, at 89 the dots on the band's
lower row under the alert/stake row); 6 and 12 by the mechanical checks in
`closeout-main-docs.md` §4 (the forbidden files untouched in `git diff
main...HEAD`, `game.rs` untouched, both versions 1, no `Color` in the diff,
no new crate, 0 warnings, 443 + 6 tests green three times); 7 by the app
tests and the walkthrough (Continue on a save left at the popup drew it on
the first frame with nothing bold, Continue mid-match and the `g` rematch
drew settled); 8 by the settings tests (default, missing key, Off) and the
Phase 2 walkthrough (`→` flipped Off and the file gained `"animations":
false`; a hit then drew thin with its value, a plain `Opponent's Turn`, a
double-bordered play with the slot blank and the popup on the resolving
frame; a file with the key deleted read On); 10 by the walkthroughs (the
cursor kept breathing through every landing) and the brief's Motion
amendment on the branch; 11 by `beats_are_named_constants_within_bounds`
(600 / 800 / 300 ms; `grep FLIP src/` empty); 13 by the README diff on the
branch.

## Resolved decisions (the person, 2026-09-18)

- **Q1 a–e — dealer-card arrival, played-card arrival, total change, the
  popup beat and the thinking indicator are in; the stake flash (f) is
  out.** Exactly the roadmap's list plus the thinking indicator.
- **Q2 A — arrival is emphasis only** (delegated; taken as recommended). A
  face-down beat hides information and costs more for a subtlety.
- **Q3 A — the selection pulse keeps breathing during a transition**, and
  the brief's Motion section is amended by one sentence to say a one-shot
  transition may run alongside the continuous pulse (delegated; taken as
  recommended).
- **Q4 A — an Animations On/Off row in Settings** (delegated; taken as
  recommended). Reduced motion is a real need, the overlay was written to
  grow, and a missing key reads as On.
- **Q5 A — portraits stay static**; spec 016's "light portrait animation"
  deferral is closed at merge (delegated; taken as recommended).
- **Q6 A — the match board only** (delegated; taken as recommended).
- **Q7 — the Score rests at normal weight** (session ruling at planning,
  2026-09-18, under the delegation; flagged to the person in the
  spec-conformance summary). Today `Score: N` is drawn bold at all times,
  so a bold-for-a-beat transition on it would be invisible, and the only
  stronger level is the rationed inverse. The planner offered two ways
  out: rest the Score at Normal so the beat shows, or keep it bold and
  drop the total-change transition (ruling c). The session took the
  first: ruling c is the person's, and the Score keeps its place in the
  header either way. Goal 5, the Off bullet and criterion 8 carry the
  exception.
- **The standing constraint**, stated by the person with the rulings:
  transitions must be noticeable enough to add to the game and quick enough
  that they never get in the way of player actions. It is Goal 1, the
  *Timing* bounds and criterion 6.

## Revision 1 (the person, 2026-09-18, at the Phase 1 pause)

**Finding.** With Phase 1 built and walked through, the person reported
that the popup beat and the thinking dots read well but the hit and
card-play arrivals did not register at all. The driver confirmed the bold
attribute reaches the terminal on the right frames; the cause is that
bold on a thin box-drawn card is barely distinguishable from normal in a
terminal, and the hand cursor's heavy breathing border already owns the
strongest look on the board. The remedy is shape, not weight: border
weight, a face-down beat, and a source-and-destination pair.

- **Q8 — a dealer card arrives heavy and face down.** The dealt card
  draws with the heavy border for the arrival beat and shows `?` for the
  flip beat before its value. Supersedes Q2 A for dealer cards only; the
  played card keeps its value visible (the person's ruling; the flip's
  cost was a moment's hidden information, which the person accepts for a
  card the dealer, not the player, chose).
- **Q9 — a played card lands heavy, and its hand slot shows a source
  ghost.** No card flies from the hand to the board: at the loop's 50 ms
  frame and whole-cell positions a flight would read as a stutter. The
  ghost outline in the emptied slot plus the heavy landing gives the same
  "it came from there" cue without motion (the person's choice from the
  session's options).
- **Q10 — the Score transition stays as built** (Strong for the arrival
  beat, resting Normal per Q7). Not raised by the person; noted so the
  revision's scope is explicit. If it proves as invisible as the card
  bold, it is a follow-up, not part of this revision.
- **Not in this revision**: the selection pulse's own vocabulary (spec
  002 — a heavy/thin border alternation was floated as more visible than
  bold/normal; the person has not ruled), and any change to the thinking
  indicator or the popup beat, which the person judged good.
- **Bounds and process**: the flip beat's bounds are under *Timing*; the
  Off state and criteria 1, 2, 8 and 11 carry the new looks; Goal 3, the
  Non-goals, Entities and Key behavior 1–2 are amended in place above.
  The revision was made in the implementation session at the person's
  ruling ("revise the spec now so that we finish it here"), a stated
  deviation from the constitution's spec-session rule.

## Revision 2 (the person, 2026-09-19, at the Phase 1b pause)

**Finding.** Played with Revision 1 built, the person judged the heavy
landings and the source ghost good and the dealer card's `?` flip as not
making sense: "just remove the initial `?` and call it good."

- **Q11 — the flip is withdrawn.** A dealt card lands heavy with its value
  visible from its first frame. The flip beat, its constant and its bounds
  go; Q8 now reads "a dealer card arrives heavy". The Non-goals' face-down
  bullet is restored for every card, Goal 3 loses its staged-appearance
  exception, and criteria 1, 8 and 11 and the Off state are amended in
  place above. Everything else in Revision 1 (Q9, Q10) stands.
