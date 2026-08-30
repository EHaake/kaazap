# Plan: TUI Overhaul

**Status**: Draft — pending review
**Implements**: spec.md in this directory (design reference:
`design/brief.md`)

Guiding constraints from `CLAUDE.md`: extend the custom Frame/renderer,
never replace it; simplest design that satisfies the spec; game logic
stays decoupled from rendering. The strongest structural claim in this
plan: **`game.rs` is not modified at all** — the cursor model translates
into the existing `GameAction` vocabulary, which is exactly the boundary
spec 001 promised would make this swap logic-free. Any task that finds
itself editing `game.rs` has gone off-plan and should stop for review.

## Data model / core types

### Styled cells (`frame.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emphasis {
    #[default]
    Normal,
    Strong, // bold      — the thing to look at
    Muted,  // dim       — inactive / secondary
    Alert,  // inverse   — interrupts only, rationed per the brief
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub emphasis: Emphasis,
}

pub type Frame = Vec<Vec<Cell>>; // still [x][y], still double-buffered
```

`Emphasis` is a single-axis enum, not bitflags — the brief mandates one
axis per element, so representable state matches allowed state. Cursor
selection is *not* an `Emphasis` variant: it's carried by border weight
(heavy glyphs), i.e. by characters, not attributes.

`frame.rs` also becomes the single home of the drawing vocabulary,
replacing the three identical `draw_text` helpers currently duplicated
across `board.rs`, `menu.rs`, and `overlay.rs`:

```rust
pub enum BorderWeight { Single, Heavy } // ┌─┐│└┘ vs ┏━┓┃┗┛

/// Clip-safe: writes the chars that fit inside the frame, drops the
/// rest. Cannot panic at any position or length.
pub fn draw_text(frame, x, y, text, emphasis);
/// Clip-safe placement within a Rect: Left / Center / Right.
pub fn draw_text_in(frame, rect, row, align, text, emphasis);
/// One box-drawer for cards, overlays, and any future frame.
pub fn draw_box(frame, rect, weight);
```

### Renderer (`render.rs`)

Diffing compares `Cell`s (char + emphasis). The render loop tracks the
currently-emitted attribute; when a changed cell's emphasis differs, it
emits `SetAttribute(Reset)` + the mapped attribute (Bold / Dim /
Reverse) before printing. Only `Emphasis` maps to escape codes — there
is no code path that can emit color. A dimension mismatch between
`last_frame` and `curr_frame` forces the full clear + redraw path
(this is what makes resize rendering safe).

### Layout (`layout.rs`)

Extends the existing module (home of `Rect`/`OverlayLayout`) with
computed region sets — pure functions of terminal size, unit-testable:

```rust
pub struct BoardLayout {
    pub divider_x: usize,
    pub player: SideLayout,
    pub opponent: SideLayout,
    pub status: Rect, // player-half bottom strip: one message, owned
}
pub struct SideLayout {
    pub header: Rect, // name / score / rounds / stood-bust
    pub dealer: Rect, // card grid zone, labeled
    pub played: Rect,
    pub hand: Rect,
}
```

plus `MenuLayout` (title + items) and card-slot math
(`slot index -> Rect` within a zone, wrapping by `cards_per_row`) so
`board.rs` never does coordinate arithmetic again. `OverlayLayout`
stays, sized in one place. Every hand-tuned offset in `board.rs`,
`menu.rs`, and `overlay.rs` (both magic-number TODOs) is replaced by
reads from these structs.

### Cursor (`app.rs`)

```rust
pub struct HandCursor {
    index: usize,           // hand slot under the cursor
    pending_positive: bool, // sign shown/committed for ± & tiebreaker
}
```

Owned by `Screen::InGame` alongside the boxed `GameState` — interaction
state lives with the screen, not in game logic.

### Selection pulse (`app.rs`)

```rust
pub struct SelectionPulse {
    acc: Duration,
    on: bool, // phase A (Strong) vs phase B (Normal)
}
```

One pulse, owned by `App`, ticked in `App::tick`, passed read-only into
draws — a single cadence for every selection on every screen (the
existing `MENU_ANIMATION_TIME_MS` constant becomes
`SELECTION_PULSE_MS`). The selected element's *emphasis* breathes
between `Strong` and `Normal`; the heavy border weight stays constant
as the stable anchor, so the pulse reads as breathing, not flicker. Behavior: ←/→ move
across occupied slots (wrapping), ↑/↓ toggle `pending_positive` (reset
to `+` on every move; only meaningful on sign-choice cards), Enter
confirms. **Confirm emits existing actions only**: a fixed card or flip
sends `PlayHand { index }`; a ± / tiebreaker sends `PlayHand { index }`
immediately followed by `ChooseSign { positive }` — the engine passes
through `AwaitingSignChoice` and back within one input event, invisible
to the player, with all validation still centralized in
`apply_game_action`. Arrow/Enter arrive as `KeyCode` variants, not
chars, so the existing char-key routing is untouched by construction.

## Architecture / flow changes

- **`board.rs`**: draws from `BoardLayout`; zone labels ("Dealer",
  "Played", "Hand") in `Muted`; the selected hand card renders via
  `draw_box(.., Heavy)` and — if it's a sign-choice card — its face
  shows the pending signed value (`+3`/`-3`) instead of `±3`, so the
  choice is visible on the card itself; the selected card's border
  emphasis breathes with the shared pulse. `BoardView::draw` gains
  cursor and pulse parameters (read-only); rendering never mutates. The status line becomes one function choosing exactly one
  message by precedence: alert ("OVER 20 …", rendered `Alert`) >
  sign-prompt > cursor hint ("←/→ card  ↑/↓ sign  Enter play") > turn
  text. Outcome/BUSTED banners render `Alert`; scores `Strong`.
- **`card.rs`**: `CardView` gains `weight: BorderWeight` and an
  emphasis for its text; its hand-rolled ASCII border is replaced by
  `draw_box` + centered clip-safe text. Card size constants unchanged.
- **`menu.rs`**: selection rendered in the shared vocabulary — the
  selected item breathes `Strong`/`Normal` on the shared pulse,
  unselected items `Normal`. The `++ item ++` text decoration is
  retired, but its *animation* survives as the pulse (human-ruled at
  plan review); `MenuState`'s private timer is replaced by the shared
  `SelectionPulse`. Layout from `MenuLayout`.
- **`overlay.rs`**: borders via `draw_box(Single)`; new
  `OverlayKind::HowToPlay` backed by `assets/how_to_play_text.txt`
  (card kinds, over-20/bust recovery incl. d/s accepting the bust,
  tiebreaker, first-to-3). `app.rs` routes `MenuItem::HowToPlay` to
  open it; dismissal mirrors the existing help overlay.
- **`app.rs`**: intercepts `KeyCode::Left/Right/Up/Down/Enter` for the
  in-game cursor before the char-key path; owns `App::resize(Config)`
  (below) and the too-small state.
- **`main.rs` (resize)**: the event loop matches `Event::Resize(c, r)`.
  At or above minimum: rebuild `Config`, call `App::resize` (recompute
  layouts, re-instantiate any open overlay), allocate frames at the new
  size — the renderer's dimension-mismatch check forces a clean full
  redraw. Below minimum: `App` enters a `TooSmall` presentation state —
  draw shows "Terminal too small — need at least WxH" and input is
  ignored except quit; game state is untouched underneath and play
  resumes exactly where it was once a compliant resize arrives.
  Startup below minimum still errors out (unchanged).

## Known limitations

- **Heavy box-drawing glyphs vary by terminal font.** First rendering
  task includes an on-screen check in a real terminal; if heavy reads
  badly, the approved fallback is double-line (`╔═╗`) per the brief —
  one constant to flip.
- **The pty driver can't assert emphasis.** Its grid parser ignores
  SGR codes (so it keeps working unchanged for text/structure checks),
  but attribute verification is by eye in a real terminal — consistent
  with the constitution's "verify rendering by running it".
- **Driver can't send arrow keys yet.** `key:` handles literal chars;
  cursor-path smoke tests need escape sequences (`\x1b[C` etc.) — a
  small driver/skill update, done when the cursor tasks land.
- **The pulse can't be captured by the pty driver** — snapshots are
  instants, so the breathe is verified by eye in a real terminal; the
  driver still asserts static structure (which card is heavy-bordered).

## Testing strategy

Unit tests target the pure logic, not terminal output (constitution):

- Clip-safety: `draw_text`/`draw_text_in` at every boundary — text
  overrunning the frame edge, x/y out of bounds, rect-relative
  clipping; property: no input panics.
- `BoardLayout`/`MenuLayout`: regions stay in-bounds and non-
  overlapping at minimum size, typical size, and asymmetric sizes;
  card-slot math wraps rows correctly.
- `HandCursor`: movement skips empty slots and wraps; sign resets on
  move; toggle only affects sign-choice cards; confirm emits the right
  action sequence (incl. the ± double-action); empty-hand behavior.
- Renderer emphasis transitions: cell diff yields attribute changes
  only when emphasis changes (testable on the queued-command level or
  via a pure "what attribute should be active" helper — keep it to
  logic, not a mock terminal).
- Status-line precedence: one pure function, all orderings tested.
- `SelectionPulse`: accumulation toggles the phase at the cadence and
  carries remainder time (the same arithmetic the menu timer has
  today, now shared and tested).
- Rendering itself (glyphs, emphasis appearance, resize behavior):
  verified by running — driver for structure, real terminal + manual
  resize for attributes and the too-small state.

## File structure

No new source modules; no new dependencies (box-drawing is Unicode,
attributes are already in `crossterm`). Touched: `frame.rs` (Cell,
Emphasis, drawing vocabulary), `render.rs` (cell diff + attributes,
dimension-mismatch force), `layout.rs` (region sets), `card.rs`
(CardView restyle), `board.rs`, `menu.rs`, `overlay.rs`, `app.rs`
(cursor, resize state, How-to-Play routing), `main.rs` (resize event),
`assets/how_to_play_text.txt` (new), `assets/game_overlay_text.txt`
(gains cursor keys). **Untouched: `game.rs`, `player.rs`,
`config.rs`** (config keeps its startup minimum check; resize reuses
the same constants).

## Resolved decisions

- **Cursor emits existing `GameAction`s** — `PlayHand` then
  `ChooseSign` for sign-choice cards — rather than a new
  play-with-value action. Zero engine surface change, validation stays
  centralized, and the spec-001 claim that the prompt swap would touch
  no game logic is honored literally.
- **`Emphasis` is a one-axis enum**, not bitflags — the brief's "one
  axis at a time" rule enforced by the type.
- **Selection is glyph weight, not an attribute** — heavy border is
  drawn state, so a pure-characters fallback remains possible and
  selection survives even if attributes render oddly somewhere.
- **`HandCursor` lives on `Screen::InGame` in `app.rs`** — interaction
  state with the screen; `BoardView::draw` receives it read-only.
- **`draw_text` deduplicated into `frame.rs`** — three identical
  copies today; one clip-safe implementation replaces them all.
- **Selection pulse unifies the animation** (human-ruled at plan
  review, vetoing this plan's original blink-retirement): one shared
  two-phase breathe — emphasis `Strong` ↔ `Normal` on the selected
  element, heavy border constant — at the former menu-blink cadence,
  across menu and board. The broader eye-guiding animation pass is
  roadmapped as its own future spec, and its design should build on
  this pulse rather than invent a second motion vocabulary.
- **Sign resets to `+` on cursor move** — predictable over sticky.
- **Resize below minimum pauses rather than exits**, and startup
  behavior is deliberately unchanged.
