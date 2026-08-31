# Spec: TUI Overhaul — Monochrome Identity, Cursor Selection, Layout Robustness

**Status**: Approved
**Depends on**: 001-core-engine (merged)
**Design reference**: `design/brief.md`

## Summary

Rework Kaazap's presentation and interaction layer around three ideas:
a deliberate monochrome visual identity (weight, brightness, and
inversion as the only emphasis — see `design/brief.md`), the
cursor-selection interaction model deferred from spec 001, and a layout
layer that is robust by construction — no panics from text placement,
no hand-counted magic numbers, and real terminal-resize handling. Also
absorbs two dangling UI items: wiring the dead "How to Play" menu entry
and updating the rules text for everything spec 001 added. Game logic
is untouched: the engine API already takes resolved values, so this
spec exercises that boundary rather than moving it.

## Goals

1. **Styled-cell rendering.** Extend the custom double-buffered `Frame`
   from a char grid to a styled-cell grid supporting monochrome
   emphasis attributes (bold, dim, inverse) — no color. The diff
   renderer and established render-thread pattern stay.
2. **Monochrome visual identity** per `design/brief.md`: sharp
   single-line box-drawing borders replace ASCII `+-|` everywhere;
   heavy borders mark cursor selection; bold/dim/inverse carry
   emphasis per the brief's token table; labeled board zones (dealer
   row, played row, hand) and a single status line that owns
   turn/prompt/alert text.
3. **Cursor-selection interaction model** (deferred from spec 001):
   navigate the hand with arrow keys, toggle a ± card's sign on the
   selected card before playing, confirm to play. Designed as a
   reusable selection vocabulary the future campaign screens (shop,
   pack opening, opponent select) and the start menu inherit — the
   start menu adopts it in this spec as proof.
4. **Direct keys keep working.** The existing fast path — `1`–`4`,
   `d`/space, `s`, the `h`/`l` sign prompt — remains exactly as
   shipped, coexisting with the cursor model. (A user setting choosing
   between them is a future settings-spec item, noted in `ROADMAP.md`;
   this spec ships both, always on.)
5. **Layout layer.** Screen regions computed from `Config` in one
   place; text drawing that clips instead of panicking; centered and
   edge-aligned placement helpers. The magic-number offsets in
   `board.rs`, `overlay.rs`, and `menu.rs` (both TODOs included) are
   replaced, not augmented.
6. **Terminal resize handling.** Resizing mid-game re-computes layout
   and redraws cleanly. Shrinking below the minimum shows a "terminal
   too small" state (game paused, nothing lost) and recovers when the
   terminal grows back — replacing today's behavior of garbling until
   restart. Startup below minimum still errors out as it does now.
7. **How to Play, wired and current.** The dead `MenuItem::HowToPlay`
   opens a rules overlay; rules text covers all spec-001 card kinds,
   the over-20 recovery rule, the tiebreaker, and match structure.
8. **Test coverage** for the new interaction and layout *logic*
   (cursor movement/state, sign toggling, region computation,
   clipping arithmetic), per `CLAUDE.md`. Rendered output is verified
   by running the game, not by asserting against a mock terminal.

## Non-goals (explicitly deferred)

- **Color** — ruled out for this spec by explicit decision: the
  monochrome starkness is the identity (see `design/brief.md`). A
  single accent hue MAY be revisited later; "a colorful game" is not
  the direction. The styled-cell architecture is the only concession
  to that future, and only because attributes need it anyway.
- **Input-model setting** (cursor-only vs keys-only toggle) — belongs
  to the future settings screen; both models simply coexist here.
- **Campaign screens** (shop, pack opening, opponent select) — not
  built here; the cursor vocabulary is designed so they can adopt it
  without new interaction invention.
- **Eye-guiding gameplay animations** (dealing, flips resolving, score
  changes, round transitions) — deferred to a dedicated animation pass,
  now on `ROADMAP.md` (human-requested). This spec ships exactly one
  piece of motion: the selection pulse below.
- **Sound** — separate roadmap item.
- **TUI frameworks** — per the constitution, the custom Frame/renderer
  is extended, never replaced.
- **Changing the minimum terminal size** — the current minimum stays;
  this spec only makes violating it mid-game safe.

## Entities

- **Cell** — one screen position: a character plus monochrome emphasis
  (normal, bold, dim, inverse). `Frame` becomes a grid of these.
- **Emphasis tokens** — the brief's four: selected (heavy border),
  strong (bold), muted (dim), alert (inverse). UI code speaks in
  tokens; only the renderer knows what they map to.
- **Layout** — named regions (player/opponent halves; dealer, played,
  hand zones; header; status line; overlay box) computed from terminal
  size in one place, consumed everywhere text or cards are placed.
- **Cursor** — the current selection within a selectable set (hand
  slots, menu items): position, wrap/clamp behavior, and — for ±/
  tiebreaker cards — a pending sign carried until confirm.
- **Status line** — the single region owning turn text, sign prompts,
  the over-20 alert, and advance hints; owns the precedence when two
  messages compete.

## Key user flows

### Playing a card with the cursor

Arrow keys (←/→) move a visible cursor across the hand's occupied
slots; the selected card shows the heavy border. On a ± or tiebreaker
card, ↑/↓ (or a toggle key) flips the card's pending sign in place —
the face text updates between `+3` and `-3` so the choice is visible
on the card itself, no sub-prompt needed. Enter commits the selected
card at its shown sign. The number keys and the `h`/`l` sub-prompt
flow keep working unchanged alongside this; Escape (or moving on)
never costs the player anything — a card is only spent on confirm.

### Menu navigation with the same vocabulary

The start menu highlights its selected item using the same emphasis
language (and gains How to Play as a working entry). Up/down and
Enter/space behave as today; visually the selection reads the same way
the selected card does, one vocabulary across screens.

### Resizing the terminal

Mid-game, growing or shrinking the terminal re-lays-out the board
cleanly on the next frame. Shrinking below the minimum swaps to a
"Terminal too small — need at least COLSxROWS" screen with the game
paused underneath; restoring the size returns to the game exactly
where it was. No panic, no garbled frame, no lost state at any size.

### Reading the rules

Menu → How to Play opens a rules overlay (same overlay frame as the
controls help): card kinds and what each does, the over-20 recovery
rule and that `d`/`s` accept the bust, tiebreaker resolution, match
structure. Dismissed the same way the controls overlay is.

## Design requirements

- `design/brief.md` is binding: monochrome only; single-line default
  borders; heavy = selection; bold/dim/inverse per the token table;
  inverse rationed to genuine alerts.
- Every interactive state must be discoverable in the moment: the
  cursor teaches itself by being visible; sign toggling and confirm
  keys appear on the status line while a card is selected; overlays
  continue listing their keys.
- The status line owns message precedence (alert beats prompt beats
  turn text) so competing messages never overdraw each other.
- **Selection breathes**: the selected card and the selected menu item
  carry a gentle two-phase pulse between emphasis states — the one
  moving thing on an otherwise still screen, marking "you are here".
  Nothing else animates in this spec.

## Acceptance criteria

- [x] `Frame` cells carry monochrome emphasis; the renderer emits
      attribute changes only when a cell's emphasis differs — and no
      color escape codes ever. *(T001/T001a; `grep` for color setters in
      `src/` is empty.)*
- [x] All cards, dividers, and overlay frames render with single-line
      box-drawing; no ASCII `+-|` chrome remains. *(T003.)*
- [x] A full game is playable with only arrows + Enter: navigating the
      hand, toggling a ± card's pending sign (shown in the status line
      per T007a, not on the card face), and committing — verified in the
      running game.
- [x] Direct-key play (`1`–`4`, `d`/space, `s`, `h`/`l` prompt, `n`,
      `g`, `x`, `?`) works exactly as before — regression-checked
      against the spec-001 behavior. *(Keys-only round to outcome +
      h/l sub-prompt verified in-app.)*
- [x] The selected card (and selected menu item) is visually distinct
      per the brief (heavy border / `▸` marker); unselected cards are
      unchanged (unselected hand cards dim per T007a). *(T007.)*
- [x] The selected card and selected menu item pulse gently between
      emphasis states; nothing else on screen animates — verified by
      running. *(T007; real-terminal eyeball still recommended.)*
- [x] No text-drawing code can panic from placement: drawing at any
      position, any length, any terminal size ≥ minimum clips safely
      — covered by unit tests on the layout/clipping logic. *(T002
      `clip_`/`box_` tests.)*
- [x] `board.rs`, `overlay.rs`, and `menu.rs` place content via the
      layout layer; the hand-tuned numeric offsets (and both
      magic-number TODOs) are gone. *(T004/T005.)*
- [x] Resize during play re-lays-out cleanly; below-minimum shows the
      recovery state and restores without losing game state; verified
      by resizing a live game. *(T008; shrink→recover→reflow with score
      intact.)*
- [x] Menu → How to Play opens the rules overlay; its text covers all
      card kinds, over-20/bust behavior, and the tiebreaker. *(T009.)*
- [x] `cargo test` green (114), with new tests covering cursor logic,
      sign toggling, region computation, and clip arithmetic; `cargo
      build` introduces no new warnings (0, down from main's 4).

## Resolved decisions

- **Monochrome with attributes** (human-ruled at scoping): bold, dim,
  and inverse are in — they are brightness/emphasis, in the spirit of
  the identity — and color stays out. The one-accent-hue door is
  recorded in the brief; nothing in this spec anticipates it beyond
  cells existing.
- **Sharp single-line borders; heavy for selection** (human-ruled at
  scoping, from presented options): rounded, double-default, and
  keep-ASCII were considered and declined. Double-line is the approved
  fallback if heavy glyphs render poorly in real terminal fonts.
- **Both input models always on** in this spec; the choose-one setting
  is deferred to the settings spec (roadmapped) rather than shipping a
  toggle without a settings screen to live in.
- **Start menu adopts the cursor vocabulary now** — it's the cheapest
  second consumer and proves the model generalizes before the campaign
  screens need it.
- **Startup below minimum size still errors** (unchanged); only the
  mid-game transition is being made safe. Revisiting the minimum
  itself is out of scope.
- **Space stays Hit.** Enter is the cursor model's confirm key; space
  is not overloaded, avoiding any conflict with the existing draw
  binding.
- **Selection animation kept** (amended at plan review, human-ruled):
  the plan's original call to retire the menu blink was vetoed in favor
  of a unified selection pulse — the menu's existing animation evolves
  into the shared vocabulary rather than disappearing. The broader
  "guide the eye" animation work is a roadmapped future spec, not this
  one.
