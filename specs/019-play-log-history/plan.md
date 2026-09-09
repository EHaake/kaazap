# Plan: Play log — full match history + scrollable window (spec 019)

> Amends spec 018. Read `src/play_log.rs`, `src/overlay.rs`, and the play-log
> parts of `src/app.rs` (field `play_log`, `Modal::PlayLog`, `update_play_log`,
> the `L`/`Esc` routing, the `Some(Modal::PlayLog)` draw arm) first — this plan
> changes those and nothing else. **No engine/save change** (`game.rs`,
> `player.rs`, `card.rs`, `save.rs` stay untouched; `PlayLog` is never
> serialized). Monochrome by construction (glyphs + `Emphasis`, no color path).

## 1. `src/play_log.rs` — group moves by round; render the full transcript

Replace the split `outcomes: Vec<RoundSummary>` + per-round `moves: Vec<Move>`
with a per-round grouping that keeps every round's moves:

```rust
#[derive(Default)]
struct RoundLog {
    moves: Vec<Move>,
    summary: Option<RoundSummary>, // Some once the round resolved
}

pub struct PlayLog {
    rounds: Vec<RoundLog>,   // all rounds this match, oldest first; last = in-progress
    opponent_name: String,
    prev: Option<PlayLogSnapshot>,
}
```

`Move`, `Resolution`, `RoundSummary`, `PlayLogSnapshot`, `moves_since`,
`moves_for_side`, `match_restarted`, `round_reset`, `summarize_round` are
**unchanged** (they already do the right per-diff work). Only the accumulation
in `observe` and the rendering change.

- `reset(&mut self, opponent_name)`: `self.rounds.clear()`, set name, `prev = None`.
- `observe(&mut self, gs)`:
  - `prev = None` → seed only (store snapshot), emit nothing.
  - else, with `prev`:
    - `match_restarted(prev, curr)` → `self.rounds.clear()`.
    - else `round_reset(prev, gs)` → `self.rounds.push(RoundLog::default())` (a
      new round has begun; the finished round's log stays in `rounds`).
    - else:
      - `let ms = moves_since(prev, gs);`
      - `let newly_resolved = !prev.outcome_present && curr.outcome_present;`
      - if `!ms.is_empty() || newly_resolved`: ensure a current round exists —
        if `self.rounds` is empty **or** its last round already has a
        `summary` (defensive; shouldn't happen before a `round_reset`), push a
        fresh `RoundLog`; then extend the current round's `moves` with `ms` and,
        if `newly_resolved`, set its `summary = Some(summarize_round(gs))`.
  - always store `self.prev = Some(curr)`.

  The one-card invariant, the reset signals, and the precedence classification
  are all as in spec 018 — this only changes *where* the moves and the summary
  are stored (onto the current `RoundLog` instead of two flat vecs).

- **Rendering** — split title from the scrollable body so the draw layer can
  pin the title and scroll the body:

  `pub fn render_body(&self) -> Vec<String>` — the full transcript, **no title,
  no trimming** (the draw layer scrolls it). For each round (1-based `n`):
  - a header: `Round {n}` plus, if `summary` is `Some`, ` — {result}` where
    `{result}` is the winner/tie + both totals + resolution (reuse the existing
    `outcome_line` content, reworded to fit a header); if `None`,
    ` (in progress)`.
  - each move line, **indented two spaces** (reuse `move_line`).
  - a blank separator line after each round except the last.
  When `rounds` is empty (fresh match, pre-first-move), return a single
  placeholder line (e.g. `"(no moves yet)"`).

  Keep `side_label`, `move_line`, and the outcome/result formatting helpers;
  drop the old `render_lines` (its title/section/overflow logic moves to the
  draw layer). Tune punctuation/wording for the header form; tests assert
  substrings, not exact strings.

- **Tests** (adjust the existing ones, add where noted):
  - The spec-018 `moves_since` / `summarize_round` tests are unchanged (those
    fns didn't change) — keep them.
  - Replace the `observe`/lifecycle tests that reached into `log.moves` /
    `log.outcomes` with equivalents over `log.rounds`: after a completed round
    then a `setup_next_round`-style empty state, **the prior round's moves are
    still present** (a new empty current round was pushed; the prior round's
    `moves` and `summary` are intact) — this is the headline behavioral change,
    so pin it explicitly. A `game_over` true→false clears all rounds. A first
    `observe` from `prev = None` emits nothing. A round resolving sets the
    current round's `summary` exactly once.
  - Replace the `render_lines` tests with `render_body` tests: multiple rounds
    each render their header (with result once resolved, "in progress" while
    live) followed by their own moves; the player is "You" and the opponent is
    named; an empty log renders the placeholder. No trimming happens in
    `render_body` (a 3-round body returns all rounds' lines).

## 2. `src/overlay.rs` — a scrollable, padded overlay draw path

> **Shipped divergence (recorded in DECISIONS.md):** the box is sized *directly*
> to ~38% wide × ~58% tall (not via `OverlayLayout::new` at ~70% as sketched
> below) — `OverlayLayout`'s fixed padding ballooned a percentage-sized box
> toward full-screen, and the owner then halved the width. The overflow/scroll
> contract below is as built; only the box-sizing paragraph is superseded.

`draw_text_overlay` (spec 018) stays for the three static overlays — **do not
change it**. Add a second public function for the play log:

```rust
/// Draw a fixed, larger, padded overlay with a pinned title and a vertically
/// scrolled body. Returns the clamped scroll offset (so the caller can store
/// the corrected value). `scroll` is the top body line shown; `usize::MAX`
/// means "pin to bottom".
pub fn draw_scrollable_overlay(
    config: Config,
    title: &str,
    body: &[String],
    scroll: usize,
    frame: &mut Frame,
) -> ScrollResult
```

where `ScrollResult { scroll: usize, at_top: bool, at_bottom: bool }` (the
caller uses `at_bottom` to keep "follow" pinned). Behavior:

- **Size — reuse `OverlayLayout::new`.** Don't hand-roll centering/clamping:
  compute a large *content* size and let `OverlayLayout::new(config, w, h)`
  center and clamp it (it adds padding and clamps to the frame, exactly as the
  static overlays do). Target ~70% of the screen: `w = (config.num_cols * 7 /
  10).max(MIN_W)`, `h = (config.num_rows * 7 / 10).max(MIN_H)` with a sensible
  `MIN_W` (≥ the longest header, ~40) and `MIN_H` (~10). Draw into
  `layout.inner`; `clear_rect(frame, layout.outer)` then
  `draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal)`.
- **Interior rows** (all via `draw_text_in` on `layout.inner`, `Align`/inset as
  noted): row 0 = title (`Align::Center`); row 1 = a horizontal rule (a
  `String` of `─` `inner.width()` wide, `Align::Left`); rows `2 ..
  inner.height()-1` = the body viewport (`Align::Left`); the last inner row =
  a hint (`↑/↓ · PgUp/PgDn`, shown when scrollable). `draw_text_in` clips a line
  wider than the rect, so no manual truncation is needed.
- **Body viewport height** `vh = inner.height().saturating_sub(3)` (title, rule,
  hint). `max_off = body.len().saturating_sub(vh)`. Clamp `scroll` to
  `[0, max_off]` (treat `usize::MAX` as `max_off`). Draw `body[scroll ..
  (scroll+vh).min(body.len())]` at successive viewport rows. Set
  `at_top = scroll == 0`, `at_bottom = scroll == max_off`. Optionally append
  `▲`/`▼` to the rule/hint rows when more lies above/below.
- Everything drawn with `Emphasis::Normal` (monochrome). Reuse the existing
  `OverlayLayout`/`draw_box`/`clear_rect`/`draw_text_in` primitives — new layout
  math, not a new rendering mechanism. Add a small `overlay.rs` unit test that
  `draw_scrollable_overlay`'s clamp returns `at_bottom` for `usize::MAX` and
  clamps an over-large `scroll` (drive it with a throwaway `Frame` + `Config`).

## 3. `src/app.rs` — scroll state, keys, follow, draw arm

- **State**: add `play_log_scroll: usize` and `play_log_follow: bool` (grouped
  with the `play_log` field). On opening the log (`L` when no modal, in
  `Screen::InGame`): set `play_log_follow = true` (and `play_log_scroll` is
  recomputed by the draw fn while following). `Modal::PlayLog` stays unit-like.
- **Routing** (`handle_key`, the `Modal::PlayLog`-open arm): `L`/`Esc` close (as
  now). Add: `Up`/`Ctrl+P` → scroll up one line (`follow = false`;
  `scroll = scroll.saturating_sub(1)`); `Down`/`Ctrl+N` → down one line
  (`scroll += 1`, clamped next draw; if it reaches bottom, `follow = true`);
  `PageUp` → up by a page (`follow = false`); `PageDown` → down by a page. Use
  the same key vocabulary the rest of the app uses (arrows + the emacs mirrors
  already resolved in `main.rs`); the modal captures these, so there is no
  conflict with in-game bindings. Other keys are still swallowed by the modal.
  (A page = the current viewport height; since only the draw fn knows it,
  approximate with a constant page like 8, or store the last viewport height
  from the draw result — a small `play_log_page: usize` updated each draw. Keep
  it simple; a fixed reasonable page step is acceptable.)
- **Draw arm** (`Some(Modal::PlayLog)`, `Screen::InGame`): build
  `let body = self.play_log.render_body();`, choose the scroll to pass
  (`if self.play_log_follow { usize::MAX } else { self.play_log_scroll }`), call
  `overlay::draw_scrollable_overlay(self.config, "Play Log", &body, scroll,
  frame)`, and store back the returned clamped `scroll` (and update `follow` from
  `at_bottom` if you scrolled via keys — keep the follow logic in the key
  handler, just persist the clamped `scroll` here so the next key press starts
  from a real value).
- **No resize arm** still needed (content rebuilt each draw; the box is sized
  from `self.config`, updated in `resize`).
- Reset `play_log_scroll`/`play_log_follow` sensibly at match entry is optional
  (they're recomputed on open); at minimum ensure opening the log sets
  `follow = true` so it shows the latest.

## Files

- `src/play_log.rs` — `RoundLog`, `PlayLog.rounds`, `observe` accumulation,
  `render_body` (replaces `render_lines`); tests updated.
- `src/overlay.rs` — add `draw_scrollable_overlay` (+ `ScrollResult`);
  `draw_text_overlay` and the static overlays unchanged.
- `src/app.rs` — `play_log_scroll`/`play_log_follow` (+ optional page), scroll
  keys in the `Modal::PlayLog` arm, the new draw arm.
- Docs (orchestrator, at close-out): `DECISIONS.md` play-log entry amended;
  `ROADMAP.md` note; this spec dir.
- **No change**: `game.rs`, `player.rs`, `card.rs`, `save.rs`, `board.rs`,
  `render.rs`, `frame.rs`, `layout.rs`, `banter.rs`, `audio.rs`, `opponent.rs`.

## Verification

`cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
— reported verbatim, no new warnings. Then a driver pass (back up +
checksum-restore the real profile/saves first): a multi-round match shows every
round's moves; scrolling reaches the earliest round and clamps; the log opens on
the latest and follows live; larger padded window; monochrome; static overlays
unchanged. Product-owner attestation.
