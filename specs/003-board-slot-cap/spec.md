# Spec: Board Slot Cap & Vertical Centering

**Status**: Approved
**Depends on**: 001-core-engine (merged), 002-ui-overhaul (merged)
**Design reference**: `design/brief.md`

## Summary

Two coupled changes — one rules, one layout — that finish bounding the
game board.

- **Rules:** cap each side's table at **12 cards per round** (dealer
  draws + played cards, counted together). A side that fills all 12
  slots without going over 20 **auto-stands** — it holds its total and
  can act no further this round. This is deliberately *not* the
  canonical "fill the table to win"; holding is simpler and reuses the
  existing stand/resolve path.
- **Layout:** merge today's separate dealer-draw and played-card zones
  into a **single fixed-height 12-slot grid per side**, and lay the
  whole board out as one bounded block **centered vertically** in the
  terminal.

Bounding the card count is what lets the board be a fixed block, and the
fixed block is the visible payoff of the cap — hence one spec. Together
they fix the tall-monitor "spread" (header pinned top, hand pinned
bottom, a stretched gap between) and the short-terminal dealer-overflow
artifact surfaced by spec 002's review (a 4th+ dealer card wrapping out
of the one-row dealer zone into the played area at ~24–29 rows).

This is the first spec since 001 to touch the engine (`game.rs`). The
change there is deliberately small — a card-count ceiling and an
auto-stand that reuses the existing `stood`/resolve machinery, no new
phase or parallel flag — and ships fully tested per the constitution.

## Goals

1. **Board slot cap.** No side may hold more than 12 cards on the table
   in a round, counting dealer draws and played cards together. Once a
   side reaches 12, it cannot draw or play again that round. Enforced
   symmetrically for the player and the AI.
2. **Auto-stand on fill.** Reaching 12 cards ends the side's turn by
   standing on its current total: if that total is ≤ 20 it holds; if it
   is over 20 it busts — consistent with the existing "over 20 at stand
   time busts" rule (001). The over-20 recovery window still works while
   a slot remains: an 11-card side at 23 can still play a recovery
   card as its 12th and hold. Filling the table is never itself a win.
3. **Single per-side card grid.** Replace the separate dealer and played
   zones with one shared 12-slot grid per side that both dealer draws
   and played cards fill. Dealer-drawn and hand-played cards are
   **visually differentiated without color** (weight/emphasis from the
   spec-002 vocabulary); card faces continue to identify kind.
4. **Fixed-height, vertically centered board.** The board reserves the
   full grid height and renders as one fixed block, centered vertically
   as a unit (header, grid, hand, status keep their relative spacing).
   On a tall or vertical terminal the board is a centered island, not a
   top-and-bottom spread.
5. **Raised minimum terminal height.** The bounded block needs more rows
   than today's minimum; the minimum height rises from 24 to roughly 30
   rows (exact value derived in the plan from the grid geometry; width
   is unchanged). Spec 002's below-minimum handling already covers this
   gracefully — startup errors out, mid-game shows the "too small"
   recovery state — so only the threshold changes.
6. **Test coverage** for the new logic per `CLAUDE.md`: the cap ceiling,
   auto-stand-at-fill resolving as hold vs bust, the AI standing when
   full, and the grid/board region computation. Rendered output is
   verified by running the game, not against a mock terminal.

## Non-goals (explicitly deferred)

- **Canonical "fill the table wins."** Explicitly declined for now in
  favor of auto-stand/hold; a post-v1 rule-variant candidate, not this
  spec.
- **A variable cap tuned to terminal size.** The cap is a fixed rule
  (12), never scaled to how tall the terminal is. Fitting 12 slots is a
  minimum-size requirement, not a per-session adjustment.
- **A combined/chronological card model.** The engine keeps its two
  per-side vectors (dealer draws, played cards); this spec does not
  merge them into one timeline or track true placement order. The single
  grid is a rendering concern; how the two sources fill it is a plan
  detail, not a data-model change.
- **Scoring, draw range, flips, tiebreaker, hand mechanics** — all
  untouched. This spec adds a ceiling and an auto-stand; it changes no
  existing resolution rule.
- **Dealing/among-slots animation.** Motion for cards arriving in the
  grid belongs to the roadmapped animation pass, not here.
- **Other rule variants** (mid-match hand redraw, etc.) — separate
  roadmap items.

## Entities

- **Board grid** — the single per-side region that holds a round's cards
  (dealer draws + played), capped at 12 slots, laid out at fixed height.
- **Slot count / cap** — a side's current card total (dealer + played)
  and its hard ceiling of 12; the value the cap and auto-stand test
  against.
- **Card-source distinction** — dealer-drawn vs hand-played, carried
  into rendering so the two read differently in the shared grid using
  monochrome weight/emphasis only.
- **Auto-stand** — reaching the cap ends the side's ability to act,
  resolving through the existing stand/resolve path as a hold (≤ 20) or
  a bust (> 20).

## Key user flows

### Filling your table

As you draw and play, cards fill your grid in order. When your 12th card
lands you can no longer draw or play: you auto-stand on your total. If
that total is 20 or under you hold and the turn passes to the opponent;
if it is over 20 you bust and lose the round (there is no free slot left
for a recovery card). The status line tells you the table is full and
that you have stood — the same way the over-20 alert already announces a
forced state.

### The opponent filling its table

The AI, on a full table, stands rather than attempting to draw — the
mirror of the player's auto-stand. A round where both sides fill without
busting resolves by the normal totals-and-tiebreaker rules.

### Reading the board

Dealer draws and the cards you played sit together in one per-side grid,
but are visually distinct — you can tell at a glance which cards the
dealer dealt you and which you played, without color. The running total
reads exactly as before.

### A tall or vertical terminal

The whole board is a fixed-height block centered vertically. Instead of
the header hugging the top and the hand hugging the bottom with a large
empty gap between them, the board sits as one centered composition with
balanced margins above and below.

## Design requirements

- `design/brief.md` is binding: monochrome only. The dealer-vs-played
  distinction uses the existing weight/emphasis vocabulary, never color.
- **The differentiation must not collide with the hand's existing
  "selected = heavy border" meaning.** One visual cannot carry two
  meanings; the plan resolves how the grid distinguishes dealer/played
  without reusing the selection treatment (a small mockup for approval
  is expected before implementation).
- Whether unfilled slots show as faint placeholders (a Pazaak-style
  table of empty slots filling up) or stay blank within the reserved
  block is an open design detail for the plan, resolved with a mockup.
- The full-table / auto-stand state is discoverable on the status line,
  consistent with the over-20 alert pattern — the player is told why
  their turn ended.
- The board is centered as a unit; relative spacing of header, grid,
  hand, and status is preserved, only the block's vertical position
  changes with terminal height.

## Acceptance criteria

- [ ] A side can never exceed 12 cards (dealer + played) in a round; the
      12th card ends that side's ability to draw or play. Unit-tested.
- [ ] Reaching 12 cards with a total ≤ 20 auto-stands and holds; with a
      total > 20 busts. The over-20 recovery still works while a free
      slot remains. Unit-tested, including the hold and bust branches.
- [ ] The AI stands when its table is full instead of drawing.
      Unit-tested.
- [ ] Dealer draws and played cards render together in one per-side grid
      and are visually differentiated without color; card faces still
      identify kind. Verified in the running game.
- [ ] The board renders as one fixed-height block centered vertically;
      on a tall terminal there is no top/bottom spread. Verified by
      running at a tall size.
- [ ] The dealer-overflow artifact (a 4th+ dealer card wrapping into the
      played area at ~24–29 rows) no longer occurs. Verified by driving
      a many-low-draw round near the minimum height.
- [ ] The minimum terminal height is updated to fit the fixed block;
      below-minimum still shows the spec-002 recovery state mid-game and
      still errors at startup. Any user-facing size text (help/README)
      reflects the new minimum.
- [ ] `cargo test` green with the new coverage; `cargo build` introduces
      no new warnings.

## Resolved decisions

- **12-card combined cap; auto-stand/hold on fill, not filled-table-
  wins** (human-ruled; captured on `ROADMAP.md`). Holding reuses the
  existing stand path and keeps the rules change minimal.
- **Single shared grid over two separate zones** (human-ruled). The
  combined-12 cap is what makes one bounded grid the natural layout;
  keeping two zones would each need their own reserved height and work
  against the goal.
- **Dealer/played cards must be visually differentiated** in the shared
  grid, monochrome only (human-added at scoping; border-weight
  difference suggested). Exact treatment set in the plan, avoiding the
  selection-heavy-border collision, with a mockup for approval.
- **Fixed block with a raised minimum height** (human-ruled), chosen
  over keeping the ~24-row minimum with a board that grows and
  re-centers during play — the fixed block is stable and robustly fixes
  the overflow, at the cost of a taller minimum terminal.
- **Engine change stays minimal** — a card-count ceiling plus an
  auto-stand that reuses `stood` and the existing resolve path, no new
  `GamePhase` and no parallel ad-hoc flag, per the constitution's state-
  machine rule. The two-vector card model is retained.
