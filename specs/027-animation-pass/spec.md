# Spec: Animation pass — spec 027

**Status**: Approved (2026-09-18). Rulings Q1 a–e; Q2 A, Q3 A, Q4 A, Q5 A,
Q6 A delegated to the session and taken as recommended, under the person's
standing constraint (Goal 1).
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
   fixed time on an element that already exists at its final position. No
   element moves across the screen, nothing appears in stages, nothing is
   redrawn in a different shape.
4. **Stillness stays the default.** Outside the moments listed under *Key
   behavior*, the board is exactly as still as it is today; the selection
   pulse stays the only continuous mover, and it keeps breathing while a
   transition runs.
5. **It can be turned off.** A player who wants no motion beyond the
   selection pulse switches Animations off in Settings and the board draws
   as it does before this spec, frame for frame.

## Non-goals (explicitly deferred)

- **Anything outside the match board.** Menus, the Outfitter, the deck
  builder, the map, the wager prompt, the shop and the records screens do
  not change. No purchase flash, no credit-counter flash.
- **A face-down reveal** (a new card showing `?` for a beat and flipping to
  its value). Arrival is emphasis only; a card's content never changes
  after it is drawn.
- **A stake flash** at game over; the presence panel's stake line, round
  pips and banter line are untouched.
- **Portrait animation.** Spec 016 deferred "swapping portrait frames on
  the pulse". The brief's portrait amendment says portraits are static, no
  animation ever; that stands, and the deferral is closed rather than
  reopened. (Q5 A; the roadmap notes it at merge.)
- **Departure transitions.** Cards clearing at the end of a round, a hand
  slot emptying when a card is played, and the popup closing are instant,
  as today.
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
  **arrival beat** (a card arriving, a total changing) and the **popup
  beat** (the wait before an outcome popup draws). Both are constants the
  planner picks within the bounds under *Timing*, and both may be tuned at
  the phase walkthrough within those bounds.
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

1. **A dealer card arriving** (ruling a). When a card is dealt to a side's
   dealer row, that card draws **Strong** from the first frame it exists,
   and settles to its normal look when the arrival beat ends. The rest of
   the row is unchanged.
2. **A side card played** (ruling b). When a card lands in a side's played
   row, that card draws Strong for the arrival beat, then settles. When the
   opponent draws and plays in the same move, both cards transition at
   once. The hand slot it came from empties instantly, as today.
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
- Cards clearing, hand slots emptying, the popup closing: instant.
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
- **Off** means: cards and totals draw settled from the first frame, the
  outcome popups draw on the same frame as today, and the thinking line
  is today's static `Opponent's Turn`. The board's frames are identical to
  the game's frames before this spec. The selection pulse and the map's
  starfield are unaffected.
- The setting takes effect immediately, including on a match in progress.

## Acceptance criteria

1. **Dealer card arrival.** On a hit, the new card in the player's dealer
   row is drawn Strong on the frame it appears and Normal once the
   arrival beat has elapsed; the cards already in the row are Normal
   throughout. The same for a card dealt to the opponent.
2. **Played card arrival.** On a play, the card in the played row is drawn
   Strong for the arrival beat, then Normal, on both sides; when the
   opponent draws and plays in one move, both cards are Strong together.
3. **Total change.** When a side's Score changes, that figure is Strong for
   the arrival beat, then Normal; a Score that did not change is never
   Strong; `Rounds won` never transitions; the over-20 alert is unchanged.
4. **Popup beat.** On the frame a round resolves the popup is absent and
   the deciding card is visible and Strong; once the popup beat has
   elapsed the popup is drawn; `n` pressed during the beat starts the next
   round on that frame. The game-over popup behaves the same with `g` and
   `x`. The phase, the save and the banter/audio events are exactly as
   before this spec (existing tests untouched).
5. **Thinking indicator.** During the opponent's thinking pause the status
   line shows at least two different indicator states across the pause,
   keeps Muted emphasis, and reverts to the plain text once the opponent
   has acted. On the compact layout the line never reaches the stake.
6. **Keys are never delayed.** Every key handled on the board today has
   the same effect on the same frame with a transition running; no
   `GamePhase` variant, timing constant or `apply_*` method changes
   meaning. The opponent's pause is still `OPPONENT_THINKING_TIME_MS`.
7. **Settled first frame.** The first drawn frame of a new match, a
   rematch and a resumed save has no element in transition, and a resumed
   match at `AwaitingNextRound` shows its popup on that first frame.
8. **The Animations setting.** Settings shows an Animations row that
   toggles On/Off with `←`/`→`, is saved, defaults to On, reads On from a
   file without the key and Off from a file with it Off; with it Off the
   board's frames are identical to a settled draw (no Strong card, no
   Strong score, popup on the resolving frame, static thinking line);
   toggling it mid-match takes effect on the next frame.
9. **Both layouts.** Criteria 1–5 hold at 89×31 and at 139×31; the
   presence panel is unchanged at 139.
10. **The pulse keeps breathing.** The selection's emphasis alternates at
    its cadence while a transition runs; `design/brief.md`'s Motion
    section carries the one-sentence amendment.
11. **Timing bounds.** The arrival beat and the popup beat are named
    constants within *Timing*'s bounds, and the popup beat is not shorter
    than the arrival beat.
12. **No forbidden change.** No change to `card.rs`, `player.rs`,
    `save.rs`, `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`,
    `campaign_map.rs`, `opponent.rs`, `tests/balance.rs`, `Cargo.toml` or
    `Cargo.lock`; `game.rs` has no behavior change and its existing tests
    pass untouched; `SAVE_VERSION` and `PROFILE_VERSION` stay 1. No new
    crate. No color path: the only attributes emitted are the four
    existing emphasis levels.
13. **README.** The settings mention in `Readme.md` names the Animations
    row.

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
- **The standing constraint**, stated by the person with the rulings:
  transitions must be noticeable enough to add to the game and quick enough
  that they never get in the way of player actions. It is Goal 1, the
  *Timing* bounds and criterion 6.
