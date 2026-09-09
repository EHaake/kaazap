# Plan: Opponent banter — spec 017

> **Status:** Approved — skeptical-reviewer signed off (notes applied), person approved (product-owner + technical-lead). Implementation started.
**Implements**: `spec.md` in this directory

## Context

Spec 016 gave every opponent a face in an always-visible in-match presence panel, and
**deliberately reserved** two rows beneath the portrait for exactly this spec's work
(`PANEL_H_INMATCH = 18` in `portrait.rs`, whose derivation comment ends "…gap, reserved").
This spec fills that reserved space: a short **banter line** in the opponent's voice that fires
on match events, and the panel's **round-win pips** (opponent's round wins, first-to-3).

This is presentation work over a settled state machine. **No engine, save-format, or AI change**
— scoring, round/game resolution, the AI, and the save schema are untouched. The banter line and
pips read game state that already exists (`GameState.round_outcome`, per-side `bust`,
`GamePhase::GameOver`, `opponent.rounds_won`); banter itself is transient App state, never saved.
The work is: a `banter.rs` module (event diff + line selection + the authored line tables), a
panel-extras drawer in `portrait.rs` that renders the banter line and pips **only** in the
in-match panel, and thin wiring on `App` and `board.rs`.

## What the code already gives us

- **The event-detection pattern is already established for audio.** `audio.rs` diffs two
  `AudioSnapshot`s (`AudioSnapshot::of(&GameState)` → `audio_cues(prev, curr)`), driven by
  `App::emit_audio_cues` (`app.rs:511`) each tick, seeded **silently** on fresh
  (`prev_audio = None`, `app.rs:502`) and resumed (`app.rs:972`) matches. Banter reacts to the
  **same events** — round outcome, per-side bust, game over — detected the same way, by diffing.
  The silent-seed-on-resume behavior is exactly what the spec's "a resumed match shows an
  appropriate line for the current state, not the exact last quip" needs.
- **Only two sites build an in-match `GameState`.** `Screen::InGame { … }` is constructed at
  exactly `app.rs:493` (fresh match — every entry point, Quick Play and campaign, routes through
  it via `GameState::with_opponent`) and `app.rs:965` (resume). Both already reset the audio
  seed to `None` on the next lines. These are the two — and only — banter seed sites: fresh →
  match-start greeting; resume → blank. Rounds advance *in place* within a match (no `GameState`
  rebuild), so per-round events flow through the tick diff with no reseed.
- **The opponent's identity and round wins are on `GameState`.** `opponent_profile`
  (`game.rs:58`, `Copy`, all-`&'static`) carries `id` — the key banter lines attach to.
  `state.opponent.rounds_won` (`player.rs:21`, 0–3, first-to-3 at `game.rs:347`) is the pip
  count, read live at draw time so pips are always current with no event plumbing.
- **The presence panel already reserves the rows.** `draw_presence_panel(frame, panel, name,
  art)` (`portrait.rs:39`) draws border + name + portrait and leaves the bottom two interior rows
  blank. Its three callers size their own `panel` Rect: in-match (`board.rs:267`), opponent-select
  preview, campaign rail. The banter line + pips must render **only** in the in-match panel — so
  they go in a **separate** in-match-only drawer, leaving `draw_presence_panel` (and its two
  preview callers) byte-for-byte unchanged.
- **Monochrome is structural.** `frame.rs` has no color path; every drawer takes an `Emphasis`
  (Normal/Strong/Muted/Alert), never a color. Banter text and pip glyphs render as characters +
  an `Emphasis`, so monochrome is preserved by construction.

## Design tensions resolved

These are technical choices (mine to make); each is stated with its reason.

### 1. Where the current banter line lives — on `App`, as transient state

The current line is presentation, not game state, and must not reach the save file. It lives on
`App` alongside `prev_audio`/`last_reward`: `banter: Option<&'static str>` (the line now shown,
`None` = blank). Because lines are authored in-repo as string literals, the line is a
`&'static str` — no allocation, no ownership churn. `board.rs` can't read it from `GameState`, so
`BoardView::draw` gains a `banter: Option<&str>` parameter (its only new input); `App::draw`
passes `self.banter`.

A second field, `banter_last: Option<&'static str>`, retains the **most recently shown** line
independently of whether it is currently displayed. Phase-based clearing (tension §8) blanks
`banter` (sets it `None`) while a round is played, but the no-back-to-back rule (§4) must still
avoid repeating the previous line when the *next* event fires — otherwise two consecutive round
wins, blanked apart, could show the identical quip and violate acceptance criterion 3. So `pick`
is fed `banter_last` (not `banter`), and `banter_last` is updated only when a new line is picked,
never cleared by the phase-clear. Both are transient `App` state, never saved.

### 2. Reuse-or-parallel the audio snapshot — **parallel**, keeping banter self-contained

Banter needs a prev/curr diff of a few game facts (per-side bust, round outcome, game over). It
could reuse `AudioSnapshot`, but that type's fields are private to `audio.rs` and shaped for
audio's needs. Rather than widen `audio.rs`'s surface (and couple flavor text to the audio
module — the acceptance criteria call out "no audio change" in spirit), banter **parallels** the
pattern with its own tiny `BanterSnapshot` in `banter.rs`. `audio.rs` is untouched. The
duplication is ~5 fields; the two observers stay decoupled, matching the constitution's
game-logic/rendering separation. `App` gains `prev_banter: Option<BanterSnapshot>` and a
`update_banter()` method mirroring `emit_audio_cues`, called right after it at the same two tick
sites (`app.rs:798`, `app.rs:1068`).

### 3. Where the lines attach — a dedicated `banter.rs` table, not a field on `OpponentProfile`

Portraits attach as a single `&'static str` field per opponent — one line each, minimal bloat.
Banter is a *set* of lines across seven event classes; inlining that per entry would dominate the
already-large `OPPONENTS` literal and bury the balance data (thresholds, decks). The person edits
lines "that don't land," so the voices want to be **read together** — all of one opponent's
lines in a block, comparable across opponents. So the lines live in `banter.rs` as `const
BanterSet` values keyed by `id`, resolved with `banter_for(id) -> &'static BanterSet` (roster
match, else the generic set). This keeps `opponent.rs` as the roster's balance-data source of
truth and collects the voice content in one legible, editable place — the same split portraits
took (art in `assets/portraits/`, not inline).

### 4. No back-to-back repeat — pick a variant ≠ the currently-shown line

The criterion: "a given event does not repeat the same line twice in a row." `pick` tracks the
currently-shown line (`self.banter`, the *globally* last-shown line regardless of which event
produced it) and requires the newly-picked line `!= that line`. So no per-class history is needed
— a single `pick(lines, last: Option<&str>, rng) -> &'static str` that avoids `last` when the
pool has >1 entry. Pure and unit-tested (`rand` is already a dependency).

One hole to close in authoring, not code: `pick` on a **single-line pool returns that sole
line**, so a *repeatable* event class (`RoundWin`/`RoundLoss`/`RoundTie` — the opponent can win
or lose rounds back-to-back — and `OpponentBust`/`PlayerBust` — the same side can bust in
consecutive rounds) authored with only one variant would repeat back-to-back and violate the
criterion. So the **five repeatable classes carry ≥2 lines each** for every voice (including
`GENERIC`); the non-repeatable classes (`MatchStart`, `MatchWin`, `MatchLoss` — each fires at
most once per match) may have one. A test asserts the ≥2 floor for the repeatable classes across
every set, so a one-line repeatable class can't ship green.

### 5. Event precedence — match-end > bust > round-outcome

A bust and the round outcome it causes surface in the same diff tick; the final blow surfaces the
bust *and* game-over together. One line shows per event, so precedence is: **match end** (the
closing line) beats a **bust** (the vivid reaction) beats a **round outcome** (the plain
resolution). So a bust-that-loses-the-match reads its closing line, not a rueful bust line; a
bust-that-ends-a-non-final-round reads the bust line, not the round-loss line. Encoded in
`banter_event`'s ordered checks; unit-tested.

### 6. Pips — opponent-only, first-to-3, read live

`opponent.rounds_won` (0–3) drives the pips, read at draw time so they update as rounds resolve
with no event wiring. Rendered as three glyphs in the pip reserved row, **each separated by a
single blank cell** (`● ○ ○`, span `ROUND_PIPS * 2 − 1 = 5`, centered) — revised at the T005
attestation, the contiguous `●○○` read as cramped: `rounds_won` filled + the rest empty.
**Opponent-only** per spec 016's intentional asymmetry and the spec's delegated
default (the player's mirror pips arrive with the future player-status panel) — surfaced here for
attestation, revisitable. Glyphs are monochrome: filled pips at `Emphasis::Strong` (bold,
still no color) for glanceability, empty at `Emphasis::Muted`. Proposed glyphs `●`/`○` (see
Tests: a fit test bounds the pip string; the exact glyph is confirmable at attestation and
trivially swappable — the block/dot glyph family shares the East-Asian *Ambiguous* width caveat
already carried by the portraits and box art, a pre-existing project-wide assumption, not a new
one).

### 7. The reserved rows — banter above pips, both centered

The in-match panel's interior is 16 rows (`PANEL_H_INMATCH − 2`): name (row 0), portrait (rows
1–12), gap (row 13), **banter (row 14)**, **pips (row 15)**. `draw_presence_extras` computes the
interior the same way `draw_presence_panel` does and draws banter via `draw_text_in` (which clips
to the interior width, so an over-long line can never overrun) and the pips on the row
below. Banter width is bounded by `BANTER_MAX_WIDTH = PANEL_W − 2 = 20`.

### 8. Phase-based clearing (attestation revision) — a line clears when the next round's play begins

*(Revised at the T005 attestation from the original "persists until the next event"; human-ruled
option B.)* A reaction line should be on screen while there is something to react to and gone once
the player is playing again — not linger through a whole subsequent round. The clear is driven by
the **round rhythm**, not a wall clock (no timer, no idle-fade), which is deterministic and
unit-testable and needs no per-frame expiry bookkeeping.

The unifying signal is the player's **first action of a round**. `BanterSnapshot` gains
`player_engaged: bool` = "the player has drawn a dealer card, played a side card, or stood this
round" (`!player.dealer_row.is_empty() || !player.played_row.is_empty() || player.stood`). It is
`false` at every round's pristine start (each round deals no card until the player hits) and at
match start. A pure `play_resumed(prev, curr) -> bool` returns `curr.player_engaged &&
!prev.player_engaged` — the false→true transition. `update_banter` clears `self.banter` to `None`
on that transition (when no new event fires the same tick). This one rule covers both cases: the
match-start greeting clears on the player's first hit/stand of round 1, and each round-end
reaction (which shows through `AwaitingNextRound`, the "next round" pause) clears when the next
round's play begins. The match-end line has no following round, so it persists on the game-over
screen (desired). `banter_last` is untouched by the clear, preserving §4.

**Rematch greeting (found at the T005 review).** A rematch (`new_game` in place, after game over)
does not run the fresh-match seed path (`app.rs:493`) — it resets `GameState` inside the existing
`Screen::InGame`, so without help the match-end line would linger into the new match and no
match-start greeting would fire, missing the "a line fires on match start" criterion for the
rematch case. A rematch is the *only* in-game `game_over` true→false transition, so it is a clean
diff signal: `pub fn match_restarted(prev, curr) -> bool = prev.game_over && !curr.game_over`.
`update_banter` checks it **first** (a match start outranks the round-level branches) and seeds a
greeting: `pick(banter_for(id).match_start, self.banter_last, rng)` into both `banter` and
`banter_last`. No input-path or borrow changes — it rides the existing per-tick diff. (Leaving to
the menu exits `InGame`, where `update_banter` early-returns, so no false trigger.)

## Design

New geometry constant (in `portrait.rs`, with the panel geometry it derives from):

```
BANTER_MAX_WIDTH = PANEL_W - 2   // = 20: the panel interior width lines must fit
ROUND_PIPS       = 3             // first-to-3 (matches game.rs's round-win target)
```

### 1. `src/banter.rs` — event diff, line selection, and the voices (foundational)

New module (added to `lib.rs`), pure logic + content, no dependency on rendering:

- `pub enum BanterEvent { MatchStart, RoundWin, RoundLoss, RoundTie, OpponentBust, PlayerBust,
  MatchWin, MatchLoss }` — from the **opponent's** point of view (RoundWin = the opponent won
  the round; MatchWin = the opponent won the match).
- `pub struct BanterSnapshot { o_bust, p_bust, outcome: Option<RoundOutcome>, game_over: bool,
  opp_won_game: bool, player_engaged: bool }` + `pub fn of(gs: &GameState) -> BanterSnapshot`,
  mirroring `AudioSnapshot::of` (reads `opponent.bust`, `player.bust`, `round_outcome`,
  `GamePhase::GameOver { winner }`; `player_engaged` = `!player.dealer_row.is_empty() ||
  !player.played_row.is_empty() || player.stood`, per tension §8).
- `pub fn play_resumed(prev: &BanterSnapshot, curr: &BanterSnapshot) -> bool` — `curr.player_engaged
  && !prev.player_engaged`, the player's first action of a round (tension §8), the phase-clear
  signal.
- `pub fn banter_event(prev: &BanterSnapshot, curr: &BanterSnapshot) -> Option<BanterEvent>` —
  ordered precedence (§5): game-over-new → `MatchWin`/`MatchLoss`; opponent-bust-new →
  `OpponentBust`; player-bust-new → `PlayerBust`; outcome-newly-set → `RoundWin`
  (`OpponentWon`) / `RoundLoss` (`PlayerWon`) / `RoundTie` (`Tied`); else `None`. (`MatchStart`
  is never produced here — it's set at seeding, §What-the-code-gives.)
- `pub struct BanterSet { match_start, round_win, round_loss, round_tie, opponent_bust,
  player_bust, match_win, match_loss: &'static [&'static str] }` and
  `pub fn lines_for(set: &'static BanterSet, ev: BanterEvent) -> &'static [&'static str]`.
- `const GENERIC: BanterSet` (neutral, characterless — never blank, never a roster voice) and
  one `const` per roster id; `pub fn banter_for(id: &str) -> &'static BanterSet` returns the
  roster set or `GENERIC` for `"default"`/unknown ids.
- `pub fn pick(lines: &[&'static str], last: Option<&str>, rng: &mut impl rand::Rng) ->
  &'static str` — a random line `!= last` when `lines.len() > 1` (§4).

### 2. `src/portrait.rs` — the in-match panel-extras drawer

- `pub const BANTER_MAX_WIDTH`, `const ROUND_PIPS`.
- `pub fn draw_presence_extras(frame, panel: Rect, banter: Option<&str>, opponent_rounds_won:
  usize)` — computes the panel interior, draws `banter` (if `Some`) centered on interior row 14
  via `draw_text_in` (clip-safe), and the pips on interior row 15: `opponent_rounds_won` filled
  glyphs (`Emphasis::Strong`) + `ROUND_PIPS − rounds_won` empty (`Emphasis::Muted`), drawn
  **per-glyph at stride 2** (one blank cell between pips, §6) over a centered span of
  `ROUND_PIPS * 2 − 1`. Clip-safe like the other portrait drawers. **`draw_presence_panel` is unchanged** —
  this is a separate function called only by the in-match board, so the two preview callers show
  name + portrait only (acceptance criterion).

### 3. `src/board.rs` — pass the line + pips into the in-match panel

`BoardView::draw` gains a `banter: Option<&str>` parameter. After its existing
`draw_presence_panel(…)` call (`board.rs:267`), it calls
`draw_presence_extras(frame, self.layout.opponent_panel, banter, state.opponent.rounds_won)`.
Nothing else in the board changes; the board's own status band / mechanical prompts are untouched
(acceptance criterion: banter never disturbs them).

### 4. `src/app.rs` — banter state, seeding, and the per-tick update

- Fields: `banter: Option<&'static str>` (current line shown), `banter_last: Option<&'static str>`
  (last line shown, for `pick`'s no-repeat — retained across the phase-clear, §1), and
  `prev_banter: Option<BanterSnapshot>` (the diff seed), grouped with `prev_audio` as transient
  in-game state.
- `fn update_banter(&mut self)` — mirrors `emit_audio_cues`: snapshot the current `GameState`;
  if `prev_banter` is `Some(prev)`, then **on an event** (`banter_event(&prev, &curr)` yields
  `ev`) set `let line = pick(lines_for(banter_for(id), ev), self.banter_last, &mut rand::rng());
  self.banter = Some(line); self.banter_last = Some(line);` (where `id =
  game_state.opponent_profile.id`); **else on `play_resumed(&prev, &curr)`** set `self.banter =
  None` (the phase-clear, §8 — `banter_last` untouched). Then `self.prev_banter = Some(curr)`.
  Called right after `emit_audio_cues` at both tick sites (`app.rs:798`, `app.rs:1068`).
- **Seeding** (`app.rs:493` fresh / `app.rs:965` resume, beside the existing `prev_audio = None`):
  fresh match → `self.prev_banter = None;` and seed the greeting into **both** fields:
  `let line = pick(banter_for(opponent.id).match_start, None, &mut rand::rng()); self.banter =
  Some(line); self.banter_last = Some(line);`; resume → `self.prev_banter = None; self.banter =
  None; self.banter_last = None;` (blank — the spec's "an appropriate line for the current state,
  or none"; the next event replaces it).
- `App::draw` passes `self.banter` into `board_view.draw` (`app.rs:1081`).

## Files

- `src/banter.rs` — **new**: `BanterEvent`, `BanterSnapshot` + `of`, `banter_event`, `BanterSet`
  + `lines_for`, the `GENERIC` + 10 roster line tables + `banter_for`, `pick`; logic + content
  tests. Added to `src/lib.rs`.
- `src/portrait.rs` — add `BANTER_MAX_WIDTH`, `ROUND_PIPS`, `draw_presence_extras`; its tests.
  `draw_presence_panel` unchanged.
- `src/board.rs` — `BoardView::draw` gains the `banter` param and the `draw_presence_extras` call.
- `src/app.rs` — `banter` / `prev_banter` fields, `update_banter`, two seed sites, two tick
  calls, the `draw` call-site param.
- `DECISIONS.md`, `ROADMAP.md` — close-out (banter design record; roadmap item shipped).
- **No change:** `game.rs`, `player.rs`, `card.rs`, `audio.rs`, `save.rs` (banter never saved),
  `opponent.rs` (voices live in `banter.rs`, roster data unchanged), `opponent_select.rs`,
  `campaign_map.rs`, `render.rs`, `frame.rs`, `config.rs`, `layout.rs`.

## Tests

Each behavioral claim names the task that owns its check. Two claims are human-attested (marked).

- **Event precedence & mapping** (T001): `banter_event` returns `MatchWin`/`MatchLoss` on new
  game-over (over a co-occurring bust and outcome), `OpponentBust`/`PlayerBust` on a new bust
  (over the round outcome it causes), and the right `RoundWin`/`RoundLoss`/`RoundTie` on a newly
  set outcome; a repeated/stale snapshot yields `None`. Guards "a line fires on match start, each
  round win/loss/tie, each bust, and match end" and the precedence design.
- **No back-to-back repeat** (T001): for a ≥2-line pool, `pick(pool, Some(prev), rng)` never
  returns `prev` across many draws; for a 1-line pool it returns that line. Paired with the
  **repeatable-class floor** (T001 generic; T002 roster; raised to ≥3 in T005e): every set's five
  repeatable classes (`round_win`, `round_loss`, `round_tie`, `opponent_bust`, `player_bust`) carry
  enough lines that `pick` always has an alternative when the same event fires in consecutive
  rounds. Together these
  guard "a given event does not repeat the same line twice in a row."
- **Every event class is non-empty for every voice** (T001 generic; T002 roster): `banter_for`
  for `"default"`, every roster id, and an unknown id returns a set whose eight classes are each
  non-empty. Guards "never blank"; the unknown-id case guards the generic fallback.
- **Every line fits the panel** (T001 generic; T002 roster): every line of every class of every
  `BanterSet` has `chars().count() <= BANTER_MAX_WIDTH`. Guards "banter never clips or overruns."
- **Distinct voices (proxy)** (T002): the 10 roster `BanterSet`s are pairwise distinct and each
  differs from `GENERIC` (concatenated-lines proxy). Guards "each of the 10 speaks in a distinct
  voice" — the machine-checkable proxy; "reads as a distinct voice" is the human part below.
- **Phase-based clear** (T005a): `play_resumed` is `true` exactly on the `player_engaged`
  false→true transition and `false` otherwise (both false, both true, true→false); a
  `BanterSnapshot::of` on a pristine round start has `player_engaged == false`, and after a
  dealt/played/stood player it is `true`. Guards tension §8. **No-repeat survives the clear**
  (T005a): with `banter` cleared to `None` but `banter_last` retained, a subsequent same-event
  `pick` still avoids the previous line — guards acceptance criterion 3 across the blank.
- **Pips are spaced** (T005b): the pip row spans `ROUND_PIPS * 2 − 1` cells with a blank between
  each glyph, still exactly `ROUND_PIPS` markers with `rounds_won` filled, centered, within the
  interior (updates the T003 pip test for the stride-2 layout).
- **Panel extras render correctly** (T003): `draw_presence_extras` is clip-safe off-frame; the
  pip row for `rounds_won` in `0..=3` has exactly `ROUND_PIPS` markers with `rounds_won` filled;
  a banter line at `BANTER_MAX_WIDTH` lands within the interior and one longer is clipped (no
  cell past the panel border). Guards "never overlaps the portrait / name / pips," pip
  correctness, and the width bound.
- **Previews unchanged** (structural, confirmed at T003/close-out): banter/pips live only in
  `draw_presence_extras`, called only from `board.rs`; `draw_presence_panel` and the
  opponent-select / campaign-map callers are untouched. Guards "banter and pips appear only in
  the in-match panel."
- **Distinct, in-character voices read as intended; monochrome preserved** — *not unit-testable.*
  Verified by playing a match and reading the lines per the spec ("verified by playing a match…
  not only by tests"). **Marked needing verification** (close-out driver / person attestation).
- **No engine/save/AI change** — structural: no file under `game.rs`/`player.rs`/`card.rs`/
  `save.rs`/the AI is edited; banter is never serialized. **Confirmed by the diff at the sweep.**

## Verification

- `cargo build --all-targets` (no new warnings) + `cargo test -q` (reported verbatim, per the
  constitution).
- **Driver / person attestation** (back up + checksum-restore the real profile first, per the
  standing data-safety practice): start a Quick Play match (default opponent → the generic
  neutral line) and a campaign match — a greeting shows at match start; a line in the opponent's
  voice replaces it on each round win/loss/tie, each bust, and the final blow, and persists
  between events; the pips fill as the opponent wins rounds. Confirm the lines fit and read as
  ten distinct voices, nothing renders in color, the board's own prompts are undisturbed, and
  opponent-select and the campaign map still show name + portrait only. Snapshot at 139×31 and
  wider.

## Non-goals (from spec)

No branching/interactive or player dialogue; no audio/TTS; no portrait animation; no banter
outside a match; no reaction to micro-events (individual hits/stands/card plays); no persisting
banter across save/resume; no player-side pips / player-status panel (opponent-only for now).
Engine, AI, save format, and audio are untouched.
