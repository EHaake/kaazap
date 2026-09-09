# Spec 018 — Play log

## Summary

An in-match, on-demand **play log**: a toggleable overlay that records the
moves of the current round as they happen — both players' dealer draws, cards
played (with the chosen sign / flip resolution), stands, and busts, each with
the resulting total — plus a running list of this match's round outcomes.
Opened and dismissed with a keypress, drawn over the board in the project's
monochrome overlay vocabulary.

## Why

The board shows the current *state* but not the *sequence* that produced it.
After a flurry of dealer draws and hand-card plays it's easy to lose track of
what just happened — especially the opponent's moves during its turn, which
resolve on a timer. A reviewable move history makes the round legible and the
game feel more transparent, without adding or changing any mechanic. It's a
cheap, self-contained feature: state changes already funnel through the
engine's `apply_*_action` methods (the natural recording points), and the
bordered overlay frame from spec 002 already exists.

## User-facing behavior

### Opening and dismissing

- In-match, **`L`** (mnemonic "Log") **toggles** the play log overlay
  open/closed; `Esc` also closes it. Lowercase `l` is already the sign-minus
  key during a sign choice, so the toggle uses the capital.
- The overlay is available in any in-match phase.
- Opening it **does not pause** the game: the opponent's think timer keeps
  running and the log **updates live** while open, so a move made while the
  overlay is up appears immediately.
- Consistent with the existing overlays (How to Play, the `?` helps):
  bordered, monochrome, self-sizing/centered.

### What it shows

Two sections:

1. **Round outcomes this match** — a running list of completed rounds in
   order. Each entry is a compact summary, not just the winner: which side
   won (or a tie), **both sides' final totals**, and **how the round
   resolved** — a bust (and which side busted), a stand, or a filled-table
   auto-stand. This is the only content that carries across rounds.
2. **This round's moves** — a chronological list (oldest → newest) of every
   visible move since the current round began:
   - **Dealer draw** — which side, the value drawn, the resulting total.
   - **Hand card played** — which side, the card, the committed `+`/`−` sign
     or flip resolution, the resulting total.
   - **Stand** — which side, at what total.
   - **Bust** — which side, at what total.

The two players are clearly distinguished (you vs. the opponent, by name).

### Lifecycle

- The **move list is per-round**: it clears when a new round begins.
- The **round-outcomes list accumulates** across the match and clears at
  match start (a new match or rematch).
- The log is **ephemeral**: nothing about it is written to the save file. A
  resumed mid-match save opens with an empty round-outcomes list and logs
  only the moves made from resume onward.

### Applies to

Both **Quick Play** and **Campaign** matches — it's in-match and
opponent-agnostic.

## Acceptance criteria

- [ ] A dedicated key opens the play log overlay in-match; the same key and
      `Esc` close it.
- [ ] While open, the overlay lists the current round's moves in order, each
      tagged with the acting side and showing the value/card and the resulting
      total; a move made while it's open appears without reopening it.
- [ ] Dealer draws, hand-card plays (with the chosen sign / flip resolution),
      stands, and busts are all recorded, for both the player and the opponent.
- [ ] A separate section lists completed rounds this match, in order; each
      entry shows the winner (or tie), both sides' final totals, and how the
      round resolved (bust and which side, stand, or filled-table auto-stand).
- [ ] When a new round begins, the move list is empty while the
      round-outcomes list still shows prior rounds.
- [ ] Starting a new match / rematch clears both sections.
- [ ] Nothing about the log is written to or read from the save file; a
      resumed match opens with an empty log and logs from resume onward.
- [ ] The log never reveals an opponent hand card before it is played — it
      records moves as they happen, i.e. once already visible on the board.
- [ ] `cargo build --all-targets` and `cargo test` are green.

## Non-goals

- Not persisted, not exported, not a post-match summary screen.
- Not a full match transcript at move-level detail across rounds — only the
  current round's moves; completed rounds collapse to their outcome.
- No always-on panel: the log does **not** claim the reserved left margin,
  which stays earmarked for the future player-status panel (spec 016).
- No filtering, search, or heavy scrolling. The board caps a round at 12
  cards per side, so the current round's move count is bounded; if it ever
  exceeds the overlay height, showing the most recent moves that fit is
  acceptable.
- No color — monochrome per spec 002.
- No new engine phase, no AI change, no new or changed card behavior.

## Resolved decisions (from the design conversation)

- **Presentation**: toggleable popup overlay (not an always-on side panel) —
  keeps it simple and leaves the reserved left margin for the player-status
  panel.
- **Event scope**: everything visible — draws, plays with sign, stands, busts
  — with running totals.
- **Lifecycle**: ephemeral; move detail is per-round, only round *outcomes*
  carry to the match level; not saved.
- **Round-outcome detail**: each entry carries the winner/tie, both final
  totals, and how it resolved (bust/stand/filled-table) — not just the winner.
- **Applies to** both Quick Play and Campaign.
- **Toggle key**: `L` (avoids the lowercase-`l` sign-minus binding). Settled.
- **No pause**: opening the log never pauses the (turn-based) game; it updates
  live under the overlay. Settled.
