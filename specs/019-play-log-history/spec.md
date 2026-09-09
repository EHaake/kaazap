# Spec 019 — Play log: full match history + scrollable window

## Summary

Two adjustments to the shipped play log (spec 018), requested by the product
owner. Neither adds a mechanic; both are to the log's presentation/observation.

1. **Keep every round's full move detail for the whole match.** This *reverses*
   spec 018's decision to collapse completed rounds to their outcome and show
   only the current round's moves. The log now shows a full match transcript:
   each round's dealer draws, hand plays (with the resolved sign/flip), stands,
   and busts — with running totals — grouped by round, oldest → newest, with
   each round's result shown on its header once resolved.
2. **A larger, scrollable, roomier log window.** Because a full transcript
   routinely exceeds the window, the overlay is **scrollable** (`↑`/`↓` by line,
   `PgUp`/`PgDn` by page). The window is also made notably larger with more
   interior padding and slightly tightened formatting, so it reads more easily.

## Why

The owner found the per-round clearing too lossy: after a flurry of moves you
could no longer review what happened earlier in the round or in prior rounds —
only the one-line outcome survived. Keeping the full transcript makes the whole
match legible; scrolling + a bigger window keep that legible when it's long.

## User-facing behavior

- The log still toggles with `L` (`L`/`Esc` close), still draws over the board
  in the monochrome overlay vocabulary, still does **not** pause the game and
  **updates live** while open.
- **Content is the full match transcript**, grouped by round:
  - Each round is a block: a header naming the round and — once the round has
    resolved — its result (winner or tie, both final totals, and how it
    resolved: bust and which side / filled-table auto-stand / stand); the
    in-progress round's header marks it as such.
  - Beneath each header, that round's moves in order (dealer draws, hand plays
    with the resolved `+`/`−` sign or flip, stands, busts), each with the acting
    side (you vs. the opponent by name) and the resulting total.
  - Rounds are ordered oldest → newest.
- **Scrolling**: when the transcript is taller than the window, `↑`/`↓` scroll
  by a line and `PgUp`/`PgDn` by a page. The log **opens showing the latest**
  moves (pinned to the bottom) and stays pinned as live moves arrive, until you
  scroll up; scrolling back to the bottom re-pins it. A brief on-screen hint
  indicates scrollability.
- **Lifecycle (unchanged intent, adjusted for full history)**: the transcript
  accumulates across the match and **clears at match start / rematch**. It is
  still **ephemeral** — nothing about the log is written to or read from the
  save file; a resumed mid-match match opens with an empty transcript and logs
  only from resume onward.
- Applies to both Quick Play and Campaign, as before.

## Acceptance criteria

- [ ] The log shows every round of the current match, each with its own moves in
      order (not collapsed to an outcome line); prior rounds' moves remain
      visible (via scrolling) after new rounds begin.
- [ ] Each round's header shows its result once resolved — winner/tie, both
      totals, and how it resolved (bust/which side, filled-table, or stand); the
      in-progress round is marked as such.
- [ ] `↑`/`↓` scroll by a line and `PgUp`/`PgDn` by a page when the transcript
      exceeds the window; scrolling is clamped at both ends (no panic, no blank
      overscroll).
- [ ] The log opens showing the latest moves and follows live moves while pinned
      to the bottom; after scrolling up it holds position until scrolled back
      down.
- [ ] The window is visibly larger than the spec-018 content-sized box, with
      interior padding, and remains centered, bordered, and monochrome.
- [ ] Starting a new match / rematch clears the transcript; a resumed match opens
      empty and logs from resume onward; nothing about the log touches the save.
- [ ] The static overlays (How to Play, the `?` helps) are unchanged.
- [ ] `cargo build --all-targets` and `cargo test` are green.

## Non-goals

- No change to any game mechanic, engine phase, AI, card behavior, or the save
  format. Still a read-only observer over the settled state machine.
- No color — monochrome per spec 002.
- No filtering or search. No per-round collapse/expand controls (the whole
  transcript is shown; scrolling is the only navigation).
- No horizontal scrolling; lines that exceed the inner width are handled as the
  overlay already handles long lines.

## Resolved decisions

- **Full per-round move history, reversing spec 018's collapse-to-outcome.**
  [product owner]
- **Scrollable window** over auto-most-recent-that-fits or grow-only, chosen so
  the whole transcript stays reachable in long matches. [product owner]
- **Opens pinned to the latest, follows live until scrolled up.** Continuous
  with spec 018's "what just happened" default. [delegated — attested]
- **Window size, padding, and exact formatting are a delegated UX detail**,
  built to a sensible interpretation and confirmed by the owner's attestation.
