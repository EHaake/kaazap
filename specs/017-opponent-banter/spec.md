# Spec 017 — Opponent banter

## Summary

Give each opponent a voice. Spec 016 gave them faces; this makes them *speak*.
Short, per-opponent flavor lines fire on match events in the presence panel's
reserved line beneath the portrait, and the panel's reserved round-win pips now
fill in — so the opponent reacts to the match as it happens, and how close they
are to taking it is glanceable on the face.

## Why

Personality and immersion — the highest personality-per-effort item on the
roadmap. Portraits (spec 016) gave the cast faces precisely so banter would have
a face to come from; reactions to busts, round wins, and the final blow are what
turn ten AI parameter-sets into ten opponents who feel like people. The event
plumbing already exists (the audio layer detects these same events by diffing
game state), so the real work is the *writing* and the panel wiring.

## User-facing behavior

### In-match banter

- The reserved line beneath the opponent's portrait shows a short line **in the
  opponent's voice**, reacting to what just happened. It fires on:
  - **match start** — a greeting or opening taunt;
  - **round outcome** — their round win (gloat), your round win (grudging), a tie;
  - **a bust** — the opponent busting (rueful) or the player busting (mocking);
  - **match end** — a closing line on the final win or loss.
- A line appears on its event and **stays through its reaction window** — the
  round-end / between-rounds pause, and the pre-first-move opening — then **clears
  when the next round's active play begins**: the match-start greeting clears on the
  player's first action, and each round's reaction clears once the following round is
  underway. During active play the panel's banter row is simply blank (the pips
  remain, anchoring the space). No wall-clock timer and no fade — clearing is tied to
  the round rhythm, so a line is on screen exactly when there's something to react to
  and gone once the player is playing again. *(Revised at the spec-017 attestation
  from the original "persists until the next event" default, which lingered a
  reaction through the whole next round — human-ruled: option B, phase-based clear.)*
- **In-match only.** The opponent-select and campaign-map previews still show just
  name + portrait — they're for *choosing* an opponent, not watching one react.
- Banter never competes with the board's mechanical prompts (turn / "OVER 20!" /
  play prompts). Those live in the board's status band and are unchanged; banter
  lives in the opponent panel's own reserved line.

### Round-win pips

- The panel shows the opponent's **round-win progress as pips** — first-to-3, so a
  glance at the face tells you they're one round from taking the match. Pips update
  as rounds resolve.
- *(Delegated default, revisitable at attestation: the panel is opponent-only per
  spec 016's intentional asymmetry, so it shows the **opponent's** pips; the
  player's mirror pips arrive with the future player-status panel. The board
  headers keep the numeric "Rounds won: N" for both sides, unchanged.)*

### Voice & variety

- All **10 roster opponents** get a distinct voice, consistent with their
  difficulty and blurb — the rookie eager and rattled, the veteran dry and calm,
  the boss cold and untouchable. The same event reads differently from two
  different opponents.
- Each event has **a few variant lines**, chosen so the same line doesn't repeat
  back-to-back within a match.
- Lines are **terse** — sized to fit the panel (~18–20 cells wide), matching the
  game's clipped voice. No speeches.
- The **generic fallback opponent** has a minimal, neutral set — never blank, never
  another opponent's voice, but deliberately characterless, like its portrait.
- Banter is **transient**: it's presentation, not game state. It is not written to
  the save file; a resumed match shows an appropriate line for the current state
  (or none), not the exact last quip.

## Acceptance criteria

- [x] Each of the 10 roster opponents speaks in a **distinct** voice; the same event
  from two opponents reads differently. *(T002/T005e: 10 authored voices; pairwise-distinct
  + no-line-shared tests; Phase 2 + T005e content reviews verified POV/character; person
  attested.)*
- [x] A line fires on **match start, each round win/loss/tie, each bust (player or
  opponent), and match end**, shown in the panel line beneath the portrait. *(banter_event +
  precedence, unit-tested; seeded fresh + on rematch (match_restarted); driver-observed.)*
- [x] Lines **vary** — a given event does not repeat the same line twice in a row
  within a match. *(pick avoids the last line via banter_last across the phase-clear;
  repeatable classes ≥3; unit-tested.)*
- [x] The panel's **round-win pips** reflect the opponent's round wins (0–3) and
  update as rounds resolve. *(draw_presence_extras reads opponent.rounds_won live; count/clamp
  tests; driver-observed ○ ○ ○ → ● ○ ○.)*
- [x] Banter and pips appear **only in the in-match panel**; opponent-select and the
  campaign map still show name + portrait with no banter/pips. *(draw_presence_extras called
  only from board.rs; grep-confirmed previews call only draw_presence_panel; driver-observed.)*
- [x] Banter never clips or overruns the panel, never overlaps the portrait / name /
  pips, and never disturbs the board's mechanical prompts (which are unchanged). *(fit ≤
  BANTER_MAX_WIDTH + border-untouched tests; the board status band is untouched; driver-observed.)*
- [x] The generic fallback opponent shows a neutral line (never blank, never another
  opponent's voice). *(GENERIC set; banter_for("default")/unknown → GENERIC; non-empty +
  distinct-from-roster tests.)*
- [x] Build and tests green; **no engine, save-format, or AI change**; monochrome
  preserved. Verified by playing a match and reading the lines, not only by tests. *(290 tests
  green; only banter.rs/portrait.rs/board.rs/app.rs touched; glyphs + Emphasis, no color;
  person attested by playing.)*

## Non-goals

- **Branching or interactive dialogue**, or any player dialogue/choices — one-way
  flavor only.
- **Voice / audio / text-to-speech** — banter is text; SFX are unchanged.
- **Portrait animation** tied to banter — animation stays deferred (spec 016 non-goal).
- **Banter outside a match** — the select-screen blurb and the campaign map already
  carry pre-match flavor.
- **Reacting to every micro-event** (individual hits, stands, each card played). v1
  reacts to the four event classes above; a "board-reversing card play" reaction is
  a possible later addition, not built here.
- **Persisting banter across save/resume.**
- **The player's own pips / player-status panel** — arrives with the stakes spec.

## Resolved decisions (from the design conversation)

- **Home: the portrait's panel line** (the face speaks), in-match only. [human-ruled]
- **Triggers: match start, round outcomes, busts, match end** — reusing the event
  detection the audio layer already performs by diffing game state. [human-ruled]
- **Lines authored in-repo by Claude Code** — unlike the portraits (escalated to
  Fable), the writing stays with Claude; the person edits what doesn't land.
  [human-ruled]
- **Scope includes the round-win pips** — fill the panel's reserved pips space now,
  not later. [human-ruled]
- **Delegated defaults** (mine to set, revisitable at the phase attestation):
  event-driven replace with no timers; opponent-only pips; the generic fallback
  minimal and neutral; banter transient (not saved).
- **Attestation ruling (spec 017 T005 pause):** the "persists until the next event"
  default was revised to **phase-based clearing** (a line clears when the next round's
  play begins; blank between; pips retained) — the lingering reaction read as stale.
  Pips are **spaced apart** rather than contiguous. [human-ruled]
