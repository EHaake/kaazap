# Tasks: TUI Overhaul

**Status**: Approved
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task is completable and
testable on its own; resume at the first unchecked task.

<!-- WARNING, worth leaving in: once implementation starts, this file
gets written by more than one party. Never edit it from a stale copy;
prefer small targeted edits over regenerating it wholesale. -->

Per the constitution: every task ends with `cargo build` and
`cargo test`, both green, actual output reported. *Verify:* lines are
the task-specific checks on top of that baseline. One commit per task,
referencing the task ID; push after each commit — draft PR #2 tracks
the running diff.

**Review cadence**: Phases 1–3 (rendering foundation, layout, cursor +
pulse) are foundational — stop for review after **every task**. Phases
4–5 (resize/help, acceptance) are mechanical — stop after **each
phase**.

**Standing tripwire** (from plan.md): `game.rs` and `player.rs` are not
modified in this spec. Every task's verify implicitly includes
`git diff main -- src/game.rs src/player.rs` staying empty; a task that
needs to touch them has gone off-plan and stops for review.

---

## Phase 1 — Rendering foundation (`frame.rs`, `render.rs`, `card.rs`)

Review: after every task.

- [x] **T001 — Styled cells: `Cell` + `Emphasis`, renderer attributes**
  *(The forced-background item flagged here was resolved in T001a.)*

- [x] **T001a — Drop the forced background; inherit terminal default**
  *(human-ruled: the background should be the terminal's own)* Removed
  `render.rs`'s `SetBackgroundColor` grey-clear/black lines — `src/` now
  emits zero color codes (acceptance criterion #1), and cleared cells
  show the terminal's default background (brief). Added a per-frame
  `SetAttribute(Reset)` baseline so an attribute left active at frame end
  can't leak into the next frame's redrawn cells once Alert lands.
  *Verify: `grep -rn "Set(Fore|Back)groundColor\|Color::" src/` empty;
  `cargo test` green (71); driver renders the board without regression;
  real-terminal background confirmed default by the human.*
  `frame.rs`: `Emphasis { Normal, Strong, Muted, Alert }` (one-axis
  enum, `Default = Normal`), `Cell { ch, emphasis }`,
  `Frame = Vec<Vec<Cell>>`; all writing sites migrate mechanically
  (chars become `Normal` cells — visuals unchanged this task).
  `render.rs`: diff compares `Cell`s; a changed cell whose emphasis
  differs from the currently-emitted attribute gets
  `SetAttribute(Reset)` + Bold/Dim/Reverse before printing; a
  dimension mismatch between last and current frame forces the full
  clear + redraw path. No color code paths exist.
  Tests: `cell_` — default cell, emphasis-to-attribute mapping helper.
  *Verify: `cargo test` green; driver smoke — board renders
  character-identical to pre-task (same snapshot greps pass);
  `grep -rn "SetForegroundColor\|SetBackgroundColor" src/` shows only
  the pre-existing force-clear background lines or less.*

- [x] **T002 — Drawing vocabulary: clip-safe text + one box-drawer**
  *Minor plan refinement: `draw_box` takes an `emphasis` param (plan
  listed `draw_box(frame, rect, weight)`) so T007's selected-card border
  can breathe without re-touching every call site; borders pass
  `Normal`. `draw_text_in`/`draw_box` use `layout::Rect` (one-way dep,
  no cycle). Not yet wired into CardView/overlay borders — that's T003,
  which is why the driver still shows ASCII `+-|`.*
  `frame.rs` gains the single implementations: `draw_text` (clip-safe:
  writes what fits, drops the rest, cannot panic at any x/y/length),
  `draw_text_in(rect, row, align)` (Left/Center/Right within a Rect),
  `draw_box(rect, weight)` with `BorderWeight { Single, Heavy }` and
  the box-drawing glyph constants. The three identical `draw_text`
  helpers in `board.rs`, `menu.rs`, `overlay.rs` are deleted; call
  sites switch to the shared versions (positions unchanged, borders
  still ASCII this task).
  Tests: `clip_` — out-of-bounds x/y, text overrunning the frame edge,
  rect-relative clipping and alignment; `box_` — corners/edges/weights
  asserted on a small in-memory frame.
  *Verify: `cargo test clip_ box_` green; driver smoke unchanged;
  `grep -c "fn draw_text" src/*.rs` finds exactly one definition.*

- [x] **T003 — Box-drawing restyle**
  *Single-line confirmed in-app (cards `┌───┐`/`│`/`└───┘`, divider `│`,
  overlay framed). Heavy-glyph font check deferred to its real use: no
  Heavy weight renders until T007's selected card, so the definitive
  real-terminal check happens there; double-line (`╔═╗`) is the ready
  fallback (one `BorderWeight::glyphs` arm) if heavy reads badly. Human
  asked to eyeball the heavy sample in the meantime.*
  `CardView` draws via `draw_box` (gains `weight` + text emphasis
  params, defaults Single/Normal); the board divider becomes `│`; the
  overlay border uses `draw_box(Single)`. ASCII `+-|` chrome is gone.
  Includes the plan's font check: render heavy `┏━┓` in a real
  terminal alongside single-line; if heavy reads badly, flip the
  approved fallback (double-line) and record the outcome here.
  *Verify: driver snapshot shows `┌`/`─`/`│` card borders and divider;
  `grep -n '"+-"\|'\''+'\''' src/card.rs src/board.rs src/overlay.rs`
  finds no border ASCII; manual real-terminal check of heavy glyphs
  reported.*

- [x] **T003a — Played tiebreaker stays legible on the table**
  *(human-reported "bug": lost a 20–20 tie inexplicably)* Diagnosed as
  correct rules behavior made invisible: the opponent's tiebreaker (a
  lone tiebreaker wins an otherwise-tied round) rendered as a plain
  "+1", indistinguishable from a Plus(1), so the tie-loss looked
  arbitrary. Fixed the display, not the logic: `PlayedCard::display_text`
  now shows a played tiebreaker as "+1T"/"-1T", matching its "±1T" hand
  label. The spec-001 test that had locked in the misleading "+1" is
  corrected (not weakened) to assert the marked form, both signs.
  `game.rs`/`player.rs` untouched — resolution logic was already right.
  *Verify: `cargo test` green (80); a played tiebreaker shows "+1T" on
  the table.*

## Phase 2 — Layout layer (`layout.rs`, `board.rs`, `menu.rs`, `overlay.rs`)

Review: after every task.

- [x] **T004 — Board regions**
  *`BoardLayout`/`SideLayout` + `cards_per_row`/`card_slot` compute all
  board geometry from `Config`; board.rs holds no coordinate arithmetic
  or magic offsets — the old `PlayArea`/`cards_per_row` field and the
  `dealer_y=4`/`hand_y`/`played_y` constants are gone. Headers place via
  the header Rect with alignment (scores `Strong`, right-aligned); zone
  labels "Dealer"/"Played"/"Hand" render `Muted` above each zone. Card
  positions preserved; the score readouts now right-align to the region
  instead of hand-tuned `mid-12`/`mid-17` offsets (intentional, cleaner).
  Turn/outcome text left for T005's status line.*
  `layout.rs`: `BoardLayout`/`SideLayout` (header, dealer, played,
  hand, status, divider) as pure functions of terminal size, plus
  card-slot math (`slot index -> Rect`, wrapping by cards-per-row).
  `board.rs` places everything through it: zone labels ("Dealer",
  "Played", "Hand") drawn `Muted`; scores `Strong`; no coordinate
  arithmetic or magic offsets remain in `board.rs`.
  Tests: `layout_` — regions in-bounds and non-overlapping at minimum
  (67x24), typical (180x48), and odd sizes; slot math wraps rows
  correctly.
  *Verify: `cargo test layout_` green; driver — zones labeled, cards
  and headers land in their regions at 180x48; a second driver run at
  the minimum size renders without artifacts.*

- [x] **T005 — Status line, menu/overlay layout, self-sizing overlays**
  *`status_message` is one pure precedence function (over-20 Alert >
  sign-prompt Strong > turn text; opponent-turn Muted), rendered
  right-aligned in the status strip — the manual `saturating_sub`
  placements are gone. BUSTED and outcome banners now render Alert,
  Stood/hints Muted. Menu positions via `MenuLayout` (items tightened
  from the old ~9-row gap to a computed 3-row gap below the title —
  flagged, vetoable). Overlays self-size via `measure()` (widest line ×
  line count); both magic-number TODOs deleted. Status strip inset 2
  cols from the divider (small T004-layout refinement) so right-aligned
  text isn't flush. Cursor-hint precedence slot noted for T007.*
  One pure precedence function chooses the single status message:
  alert ("OVER 20 …", rendered `Alert`) > sign-prompt > cursor hint
  (slot exists now, wired in T007) > turn text; `board.rs` renders its
  result right-aligned in the status rect (the manual
  `saturating_sub` placements go away). Outcome/BUSTED banners render
  `Alert`. `menu.rs` positions via `MenuLayout`. `overlay.rs` computes
  `content_width`/`content_height` from the loaded text's measured
  lines — both magic-number TODOs die here.
  Tests: `status_` — every precedence ordering; `overlay_` — measured
  sizing from sample text.
  *Verify: `cargo test status_ overlay_` green; driver — prompt,
  over-20 alert, outcome text, and both overlays render correctly
  placed; the TODO comments are gone.*

## Phase 3 — Cursor + pulse (`app.rs`, `board.rs`, `menu.rs`)

Review: after every task.

- [x] **T006 — Hand cursor: logic, routing, action emission**
  *`HandCursor` on `Screen::InGame`; pure movement/toggle/normalize over
  the hand; `cursor_confirm` emits `PlayHand` (+`ChooseSign` for
  sign-choice cards). Arrow/Enter routing gated to `PlayerTurn`, ahead
  of the char path; number-key + h/l path untouched. Driver extended on
  main (`\e` → ESC) and merged back. Verified in-app: right-arrow moved
  to slot 2 and Enter played the 2&4; down-arrow on a flip correctly
  no-oped and Enter played it. `game.rs`/`player.rs` untouched. Rendering
  the selected card (heavy border) is T007 — the cursor drives play now
  but isn't yet visible on screen.*
  `HandCursor` on `Screen::InGame`: ←/→ move across occupied slots
  (wrapping), ↑/↓ toggle pending sign (reset to `+` on move; only
  meaningful on sign-choice cards), Enter confirms — emitting existing
  actions only (`PlayHand`, then `ChooseSign` for ±/tiebreaker).
  Cursor methods are pure over `&[Option<Card>]` so they unit-test
  without an `App`. Slot normalization after a play; empty-hand
  no-ops. Arrow/Enter routing in `app.rs` ahead of the char path,
  active only during `PlayerTurn`. Rides along: extend the run-kaazap
  driver's `key:` to send escape sequences (`\e` → ESC) so arrows are
  scriptable — committed to main per the skill's home, merged back.
  Tests: `cursor_` — movement/wrap/skip, sign toggle + reset, confirm
  action sequences (fixed, flip, ±, tiebreaker), empty hand,
  normalization after play.
  *Verify: `cargo test cursor_` green; driver plays a full card via
  arrows + Enter (no number keys), including a ± card at both signs;
  `git diff main -- src/game.rs src/player.rs` empty.*

- [x] **T007 — Selection pulse + selection rendering**
  *`SelectionPulse` on `App`, ticked once in `App::tick`, its emphasis
  passed read-only into both draws. Selected hand card renders a heavy
  border breathing Strong↔Normal, with a sign-choice card's face showing
  its pending signed value (`+1T`↔`-1T` verified in-app). Menu selection
  keeps its animation as the shared pulse — `++ item ++` retired for a
  constant `▸` marker (its always-on anchor, mirroring the card's heavy
  border) plus the breathing emphasis; `MenuState`'s private timer/
  fields and `MenuState::tick` are gone, `Screen::draw` (now dead)
  removed with them (clears the last pre-existing warning). Cursor hint
  joins the status line: PlayerTurn now shows "←/→ card ↑/↓ sign Enter
  play" instead of "Your Turn" — flagged, vetoable.*
  *Deviation note: menu selection uses a `▸` marker anchor because a menu
  item has no border to anchor the breathe; without it the item would be
  indistinguishable at the Normal phase.*

- [x] **T007a — Selected ± keeps its face; unselected cards recede**
  *(human-reported)* A selected sign-choice card no longer mutates its
  face to `+3`/`-3` (the ± "minus" appeared to vanish); it keeps its
  `±N`/`±1T` label, and the pending sign moved to the status line
  ("Play +3? (↑/↓ flip · Enter play)", flipping with the toggle) — so
  the sign is visible without the card losing its ±. `Card::selected_face`
  removed. Also: unselected hand cards now render `Muted` so the heavy
  breathing selection stands out more. Verified in-app: selected ±3
  kept its face, status flipped +3↔-3. `game.rs`/`player.rs` untouched.

- [x] **T007b — OVER 20 alert stacks above the prompt, doesn't replace it**
  *(human-reported)* The over-20 warning was replacing the whole status
  line, hiding the sign/cursor instructions — so while over 20 you
  couldn't see how to flip a ± to minus. The status strip is now two
  rows: `over_twenty_alert` renders on the upper row only while over 20,
  and the base prompt (cursor hint / selected-± "Play -3?") always
  renders below it. Over-20 logic split out of `status_message`. Verified
  in-app: "OVER 20!  (d/s: bust)" on the row above "←/→ card …", both
  visible. `game.rs`/`player.rs` untouched.
  `SelectionPulse` owned by `App`, ticked in `App::tick`
  (`MENU_ANIMATION_TIME_MS` renamed `SELECTION_PULSE_MS`); passed
  read-only into draws. Board: selected card renders heavy border,
  border emphasis breathing `Strong`/`Normal` on the pulse; a
  sign-choice card's face shows its pending signed value (`+3`/`-3`)
  instead of `±3`; the cursor hint joins the status line. Menu: the
  `++ item ++` decoration is retired; the selected item breathes on
  the shared pulse; `MenuState`'s private timer is removed.
  Tests: `pulse_` — accumulation toggles phase at the cadence and
  carries remainder (the old menu-timer arithmetic, now shared).
  *Verify: `cargo test pulse_` green; driver snapshot shows `┏` heavy
  border exactly on the selected card and the pending sign on its
  face; manual real-terminal check: card and menu selections visibly
  breathe, nothing else moves.*

## Phase 4 — Resize + How to Play (`main.rs`, `app.rs`, `overlay.rs`, assets)

Review: after the phase.

- [x] **T008 — Terminal resize handling**
  *`Event::Resize` in the game loop tracks the new size for frame
  allocation and either `App::resize` (rebuilds board layout + any open
  overlay, game state untouched) or `App::set_too_small` (paused
  recovery screen). `render()` detects a frame dimension change and
  forces a full clear+redraw, with a `force ||` short-circuit so the
  old-shaped `last_frame` is never indexed (the resize panic that would
  otherwise happen). Startup-below-min still errors. Driver gained a
  `resize:WxH` step (SIGWINCH via TIOCSWINSZ) on main, merged back.
  Verified in-app: shrink to 40x15 → "Terminal too small / Need at least
  67 x 24 / Now 40 x 15"; grow back → game resumed with score intact and
  reflowed to the new width; overlay re-centered on resize; no panic.
  `game.rs`/`player.rs` untouched.*
  `main.rs` handles `Event::Resize`: at/above minimum → rebuild
  `Config`, `App::resize` (layouts + any open overlay), frames
  reallocated, renderer's dimension-mismatch force gives a clean full
  redraw. Below minimum → `App` presents "Terminal too small — need at
  least WxH" with the game paused and state untouched; a compliant
  resize returns to play exactly where it was. Startup below minimum
  still errors (unchanged).
  *Verify: manual — resize a live game larger, smaller, below minimum,
  and back: clean re-layout each time, recovery screen shown and
  dismissed, game resumes mid-round with state intact; no panic at any
  step.*

- [ ] **T009 — How to Play, wired and current**
  `OverlayKind::HowToPlay` backed by new
  `assets/how_to_play_text.txt`: card kinds and effects, over-20
  recovery (incl. d/s accepting the bust), tiebreaker resolution,
  first-to-3 match structure. `MenuItem::HowToPlay` opens it;
  dismissal mirrors the help overlay. `game_overlay_text.txt` gains
  the cursor keys.
  *Verify: driver — menu → How to Play shows the rules text inside an
  intact border; dismiss returns to menu; controls overlay lists the
  cursor keys.*

## Phase 5 — Acceptance

Review: after the phase.

- [ ] **T010 — Acceptance sweep + skeptical review**
  Walk every spec.md acceptance box with evidence: full build (no new
  warnings vs `main`), full test suite, a complete game played with
  arrows-only and another with direct-keys-only, the no-color check
  (no color escape emission paths in `src/`), monochrome/emphasis
  behavior eyeballed in a real terminal, resize exercised. Then run
  the `skeptical-reviewer` subagent over the branch (as spec 001 did)
  and address its findings — sub-lettered tasks for anything real.
  Mark PR #2 ready only on the human's word.
  *Verify: all spec.md acceptance boxes checked with evidence;
  reviewer findings resolved or explicitly ruled; build/test output
  reported verbatim.*

---

## Handoff note

Read `CLAUDE.md`, `design/brief.md`, then this spec's `spec.md`,
`plan.md`, and this file. Implement in order from T001. **Stop for
review after every task in Phases 1–3; after each full phase for
Phases 4–5.** One commit per task referencing the task ID; push after
each commit — draft PR #2 tracks the diff. Sub-letter (`T00Xa`) any
genuinely new scope and flag it — never silently absorb it, never
renumber. The `game.rs`/`player.rs` tripwire applies to every task.
