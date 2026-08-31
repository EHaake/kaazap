# Plan: Board Slot Cap & Vertical Centering

**Status**: Approved
**Implements**: spec.md in this directory (design reference:
`design/brief.md`)

Guiding constraints from `CLAUDE.md`: simplest design that satisfies the
spec; game logic stays decoupled from rendering; extend `GamePhase` and
the `apply_*_action` path rather than adding parallel ad-hoc flags. This
is the first engine-touching spec since 001, so the inverse of spec 002's
headline claim applies: `game.rs` and `player.rs` **are** edited here,
but minimally and behind tests — a card-count ceiling and an auto-stand
that reuses the existing `stood`/resolve machinery, no new phase, no new
flag. The single number `MAX_TABLE_CARDS = 12` is the one source of
truth shared by the rule (game.rs), the grid (layout.rs), and the
minimum-size check (config.rs).

## Data model / core types

### The cap constant (`lib.rs`)

```rust
/// Max cards one side may hold on the table in a round (dealer draws +
/// played cards). Reaching it auto-stands that side. One source of
/// truth: the rule, the board grid, and the minimum terminal height all
/// derive from this.
pub const MAX_TABLE_CARDS: usize = 12;
```

### Table count (`player.rs`)

`PlayerState` gains two small queries beside `score()` — no new fields,
the count is derived from the two vectors it already owns:

```rust
impl PlayerState {
    /// Cards on the table this round: dealer draws + played side cards.
    pub fn table_card_count(&self) -> usize {
        self.dealer_row.len() + self.played_row.len()
    }
    pub fn table_full(&self) -> bool {
        self.table_card_count() >= crate::MAX_TABLE_CARDS
    }
}
```

The two-vector model (`dealer_row`, `played_row`) is **retained** — this
spec does not merge them into one timeline. The single visual grid is a
rendering concern (below), not a data-model change.

### Auto-stand (`game.rs`)

Enforced at the existing single choke point, `resolve_after_action`,
which already runs after every player action *and* at the end of
`play_opponent_turn` — so one insertion covers both sides. A full table
sets `stood`; the over-20 checks that already live just below then bust a
full side that is over, or let a full side ≤ 20 hold:

```rust
// (inside resolve_after_action, after scores are computed, before the
//  existing over-20 bust checks)

// A side that has filled the table can hold no more cards: it stands on
// its current total. The over-20 checks below then bust it if it is
// over, exactly as a manual stand would (001's over-20-at-stand rule).
if self.player.table_full()   { self.player.stood = true; }
if self.opponent.table_full() { self.opponent.stood = true; }
```

This reuses `stood` deliberately: the existing `PlayerTurn` update
already flips to the opponent when `!player_can_act()` (stood ⇒ can't
act), the existing "both stood ⇒ `RoundEnd`" already fires, and the
existing `apply_game_action` guards (`!self.player.stood`) already refuse
any further hit or play once auto-stood. No new `GamePhase`, no parallel
"table full" flag — the constitution's state-machine rule, honored.

The over-20 **recovery window is preserved** while a slot remains: an
11-card side at 23 is not yet `table_full`, keeps its turn (001's
over-20 logic), and can play a recovery card as its 12th — which then
fills the table and auto-stands it at the recovered total. Only reaching
12 *while still over* busts with no recovery, because there is no 13th
slot for a recovery card. This falls out of the code above with no
special case.

### AI stands when full (`game.rs`)

`decide_opponent_move` gains one guard at the top, so the AI rule is
directly unit-testable in isolation (the auto-stand above is the actual
enforcement; this keeps the AI from ever *choosing* an impossible hit):

```rust
fn decide_opponent_move(&self) -> OpponentAction {
    if self.opponent.table_full() {
        return OpponentAction::Stand;
    }
    // ... existing logic unchanged
}
```

## Layout: one grid, one fixed centered block (`layout.rs`)

### SideLayout collapses dealer + played into one grid

```rust
pub struct SideLayout {
    pub header: Rect, // name / score / rounds
    pub grid:   Rect, // single capped card grid: dealer draws + played
    pub hand:   Rect, // hand row (numbers drawn just below)
}
```

`dealer` and `played` are gone; both sources now fill `grid`. Its width
is the side's horizontal span (unchanged from 002: `H_PAD..divider-H_PAD`
and the mirror), so `cards_per_row(grid)` is known before any vertical
math.

### Grid geometry is single-sourced — Erik's reflow requirement

The number of grid rows is a pure function of width, computed **once** and
used for card placement, ghost placement, block height, and the
minimum-size check alike — so filled cards, empty ghosts, and the
reserved height always agree and reflow together when the terminal
resizes:

```rust
/// Grid rows needed to show all MAX_TABLE_CARDS slots at this width.
pub fn board_grid_rows(cols: usize) -> usize {
    let per_row = /* cards_per_row over one side's span at `cols` */;
    MAX_TABLE_CARDS.div_ceil(per_row)   // 4/row → 3 rows; 6/row → 2; etc.
}
```

At the minimum width (~89 cols ⇒ 4 cards/row) this is 3 rows; wider
terminals pack more per row and need fewer. `board_grid_rows` is
monotonically non-increasing in width, which is what makes the
minimum-height check below a single worst-case constant.

### Fixed block, centered vertically

The board becomes one composite block whose height is derived from the
grid rows, then centered — killing the header-top / hand-bottom spread:

```
  header      2
  (gap)       1
  grid        grid_rows*CARD_HEIGHT + (grid_rows-1)     // 3 rows → 17
  (gap)       1
  hand        CARD_HEIGHT + 1  (numbers below)          // 6
  (gap)       1
  status      2                                         // alert / prompt
  ----------------------------------------------------
  block_h  =  grid_h + 13     → 30 at min width (3 rows), less when wider
```

```rust
pub fn board_block_height(cols: usize) -> usize { /* grid_h + 13 */ }
```

`BoardLayout::new` computes `top = (rows - board_block_height(cols)) / 2`
(saturating) and lays the bands out from `top` downward, mirrored across
the divider. On resize, spec 002's `App::resize` already recomputes the
whole layout from the new `Config`, so a width change that moves the grid
from 3 rows to 2 re-centers a shorter block automatically — no new resize
plumbing.

Status keeps spec 002's conditional placement (`status_right` beside the
hand on wide terminals, `status_below` under it when narrow, chosen by
`status_fits_right`); the block always reserves the 2-row `status_below`
band, so a right-placed status simply leaves it as margin. Carrying 002's
decision forward rather than regressing it.

### Minimum terminal height (`config.rs`)

`min_size()` height term becomes `board_block_height(min_cols)` — the
worst case, at minimum width — instead of today's
`CARD_HEIGHT * MIN_CARD_SIZE_HEIGHT + V_PAD`:

```rust
pub fn min_size() -> (usize, usize) {
    let half = H_PAD + HAND_SIZE * (CARD_WIDTH + 1);
    let min_cols = 2 * half + 1;                 // ~89, unchanged
    (min_cols, crate::layout::board_block_height(min_cols))  // ~30
}
```

So the minimum rises from **24 → ~30 rows**, width unchanged. Because
`board_grid_rows` only shrinks as width grows, any terminal that passes
`rows >= board_block_height(min_cols)` also fits its own (equal or
shorter) block — one constant is a sound gate.

**Deliberately simple:** a *very* wide, *very* short terminal (≥ ~130
cols so the grid needs only 2 rows, yet < 30 rows) is rejected even
though its shorter block could technically fit. A width-aware
`fits(cols, rows)` that checks `rows >= board_block_height(cols)` would
admit it, but that terminal shape is exotic and the reject is safe (the
spec-002 "too small" screen, not a garble). Not worth the added
complexity now; noted as a future refinement if anyone ever hits it.

Spec 002's below-minimum handling is otherwise untouched: startup errors
out, mid-game shows the recovery screen and restores. Only the threshold
number changes, and every place it surfaces (startup error, too-small
screen) reads it from `min_size()`, so they update for free.

## Rendering (`frame.rs`, `card.rs`, `board.rs`)

### Double border + ghost slot (`frame.rs`)

`BorderWeight` gains the third weight the design already sanctioned:

```rust
pub enum BorderWeight { Single, Heavy, Double } // ┌┐ / ┏┓ / ╔╗
```

`draw_box` learns the `╔═╗ ║ ╚═╝` glyph set. Three distinct weights now
carry three distinct meanings with no collision: **Single** = dealer
draw, **Double** = played card, **Heavy** = the hand's selected card
(unchanged from 002). A small `draw_ghost_slot(frame, rect)` draws an
empty slot as a `Muted` (dim) dashed outline (`┌╌╌┐ ╎ └╌╌┘`); if the
dashed glyphs render poorly anywhere, a dim `Single` box is the fallback
(one call to change).

### One grid draw (`board.rs`)

`board.rs` stops drawing two separate zones and instead fills `grid`
from the two vectors, then fills the remaining slots with ghosts —
indices mapped to `(col, row)` by the same `cards_per_row(grid)` the
geometry above uses:

- **Dealer draws** fill from the front: card `i` → grid index `i`,
  `Single` border, bare value (`display_text`).
- **Played cards** fill from the back: card `j` → grid index
  `MAX_TABLE_CARDS-1-j`, `Double` border, signed value.
- **Empty slots** — the indices between the two — draw as ghosts.

Filling from opposite ends means neither group's cards move when the
other grows (drawing a dealer card never shifts an already-played card),
and it clusters dealer draws (top-left) apart from played cards
(bottom-right) as a free reinforcement of the border distinction. The
alternative — dealer-then-played contiguous with ghosts trailing — reads
as a cleaner "fills top-down" but shifts played cards one slot right on
every subsequent draw; rejected for that instability. (Open to a
different call at review; it's a localized rendering choice.)

The per-zone "Dealer"/"Played" labels from 002 retire with the zones; the
two border styles plus the card faces self-identify. A small `n/12`
slot-count hint in the header is an optional nicety, left to
implementation.

`CardView` already carries `weight` and `emphasis` (002), so played
cards are just `weight: Double`; no structural change there.

## Architecture / flow changes

- **`player.rs`**: add `table_card_count` / `table_full`. No field or
  behavior change to `score`, `has_tiebreaker_in_play`, etc.
- **`game.rs`**: auto-stand in `resolve_after_action`; `table_full`
  guard in `decide_opponent_move`. Nothing else — scoring, flips,
  tiebreaker, sign-choice, next-round/new-game all unchanged.
- **`layout.rs`**: `SideLayout` grid collapse; `board_grid_rows` /
  `board_block_height`; `BoardLayout::new` centers the block. `Rect`,
  `card_slot`, `cards_per_row`, `OverlayLayout`, `MenuLayout` unchanged.
- **`config.rs`**: `min_size` height from `board_block_height`.
- **`frame.rs`**: `BorderWeight::Double` + `draw_box` glyphs;
  `draw_ghost_slot`.
- **`card.rs`**: no change beyond passing `Double` (verify double renders
  cleanly at 9×5).
- **`board.rs`**: single-grid fill (dealer front / played back / ghost
  middle); reads `grid`, not `dealer`/`played`.
- **Untouched**: `render.rs` (cell diff already handles any glyph +
  emphasis), `app.rs`, `main.rs`, `menu.rs`, `overlay.rs`, `screen.rs`,
  all assets.

## Known limitations

- **Double-line glyphs vary by terminal font**, like the heavy glyphs
  before them. The first rendering task eyeballs `╔═╗` in a real
  terminal; dim `Single` is the ghost fallback and a solid `Heavy`-vs-
  `Double` retest is cheap if either reads badly.
- **The pty driver can't assert dim/emphasis**, so "dealer dim vs played
  full-weight" and the ghost dimming are eye-checks. It *can* assert
  structure: that played cards carry `╔`/`═` glyphs, that the grid never
  exceeds its rows, and — the concrete 002-review artifact — that a
  many-low-draw round no longer wraps a dealer card into a lower band.
- **Very-wide/very-short terminals rejected** by the simple constant
  min-height (above) — intentional, revisit only if hit.

## Testing strategy

Engine logic (constitution requires coverage for changed game logic):

- **Cap ceiling**: a side never exceeds `MAX_TABLE_CARDS`; the 12th card
  sets `stood`.
- **Hold branch**: reaching 12 at ≤ 20 auto-stands and holds (not bust,
  turn passes).
- **Bust branch**: reaching 12 at > 20 busts (`RoundEnd`, that side
  loses).
- **Recovery-as-12th**: an 11-card side over 20 plays a recovery card as
  its 12th and holds; mutation-check that removing the auto-stand lets a
  13th card land (guards the test can fail).
- **AI full table**: `decide_opponent_move` returns `Stand` when full;
  and end-to-end through `play_opponent_turn` the AI stands rather than
  drawing a 13th.
- **Both sides fill without bust**: round still resolves by the existing
  totals/tiebreaker rules (a full-but-not-over table is just a stand).

Layout logic (pure, unit-testable):

- `board_grid_rows`: 3 at min width, non-increasing as width grows,
  ≥ 1 always.
- `board_block_height` monotonic; `min_size` height equals
  `board_block_height(min_cols)`; `Config::fits` accepts the new minimum
  and rejects one row short.
- `BoardLayout`: regions in-bounds and vertically disjoint
  (header/grid/hand/status) at min, typical, tall, and wide-short sizes;
  block vertically centered (top margin ≈ bottom margin); grid holds 12
  slot positions with no overlap; slots reflow with width (4/row → 3
  rows vs a width giving 6/row → 2 rows), filled + ghost indices
  covering exactly `0..MAX_TABLE_CARDS`.

Rendering (by running, per constitution): dealer `Single` / played
`Double` / ghost dim reads clearly; the board sits as one centered block
with no tall-terminal spread; the dealer-overflow artifact is gone near
the minimum height — driver for structure, real terminal for emphasis.

## Suggested phasing (detailed in tasks.md)

1. **Engine** — `MAX_TABLE_CARDS`, `table_full`, auto-stand, AI guard,
   tests. Foundational (rules correctness): review every task.
2. **Layout** — grid collapse, `board_grid_rows`/`board_block_height`,
   centering, `min_size`, tests. Foundational (reshapes the board,
   changes min size): review every task.
3. **Rendering** — `Double` border + ghost slot, single-grid draw.
   Mechanical against the approved mockup: review per phase.
4. **Verification** — acceptance sweep (in-app centering, artifact-gone
   check via the driver, `cargo test`/`cargo build`), README/ROADMAP
   close-out.

## Resolved decisions

- **Auto-stand reuses `stood` at the `resolve_after_action` choke
  point** — one insertion covers both sides, the existing turn-pass /
  both-stood / over-20-bust logic all flows unchanged, and no new phase
  or flag is introduced (constitution's state-machine rule). The
  over-20 recovery window survives for free while a slot remains.
- **`MAX_TABLE_CARDS` is the single source of truth** for the rule, the
  grid slot count, and the minimum height — no second `12` to drift.
- **AI full-table stand is an explicit guard** in `decide_opponent_move`
  (directly testable) backed by the auto-stand as real enforcement.
- **One shared grid; Single/Double/Heavy = dealer/played/selected**
  (human-ruled). Double is the sanctioned third border weight; the
  selected-card heavy border keeps its sole meaning.
- **Empty slots are dim ghost outlines** (human-ruled), part of the same
  width-reflowed grid so filled and empty slots always agree (human's
  reflow requirement).
- **Fixed block centered vertically; height derived from width-driven
  grid rows; minimum height 24 → ~30 via a single worst-case constant**
  (human-ruled: fixed block over a grow-and-recenter board). Width-aware
  min-height refinement noted but declined for simplicity.
- **Dealer fills front, played fills back, ghosts between** — stability
  (no card shifts when the other group grows) over a contiguous
  "fills top-down" look; a localized rendering choice, open at review.
- **Two-vector card model retained** — no chronological merge; the grid
  is a rendering view over the two vectors.
