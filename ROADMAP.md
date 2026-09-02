# Roadmap

Deliberately unordered — this is a backlog, not a schedule. Priorities get
set once there's a working, correct core engine to actually build on top
of, not guessed at here in advance.

## Shipped

- **Core Pazaak engine** (spec 001) — replaced the placeholder
  `LogicCard { value: i32 }` with a real card-type system: plus, minus,
  flip (±), special 2&4 / 3&6, and tiebreaker cards, plus scoring,
  round/game resolution, and opponent decision-making. The main deck's
  0–10 draw range is an intentional rule variant (see DECISIONS.md), not
  something to fix. Everything in the backlog depends on this existing.
- **Terminal UI overhaul** (spec 002) — monochrome visual identity,
  box-drawing borders, a layout layer that replaced magic numbers with
  computed regions, terminal-resize handling, and the two backlog items
  below folded in:
  - **Cursor-selection interaction model** — arrow-key navigation, ±
    sign toggling, confirm-to-play, translated into the existing engine
    actions so the game logic stayed untouched. Campaign screens (shop,
    pack opening, opponent select) will reuse this model once they exist.
  - **Wire up "How to Play"** — the previously dead menu item now opens
    the self-sizing rules overlay.
- **Board slot cap & vertical centering** (spec 003) — cap each side's
  table at 12 cards per round (dealer draws + played, combined); filling
  it auto-stands (holds its total — not the canonical "filled table
  wins", see DECISIONS.md). Merged the separate dealer/played zones into
  one fixed 4×3 grid per side and made the whole board a fixed-size block
  centered in both axes, fixing the tall-monitor spread and the
  short-terminal dealer-overflow artifact spec 002's review surfaced.
  First engine-touching spec since 001; the cap + auto-stand reuse the
  existing stand/resolve path, no new phase.
- **Audio & settings** (spec 004) — looping background music and generated
  retro sound effects via `rodio` (which replaced the unused `rusty_audio`
  dependency), gated by user preference. A **Settings** overlay over the
  start menu (per-channel Music/SFX volume sliders) and a global `m` mute,
  with preferences persisted to a JSON config file — the first slice of the
  save/persistence layer. The bundled track (Kevin MacLeod's CC-BY "Chipper
  Doodle") is a licensed placeholder; SFX are synthesized by
  `scripts/gen_sfx.py`, so nothing carries third-party encumbrance. The
  engine stays audio-free — SFX are derived at the app layer by diffing
  game state. An original cantina-vibe track is a follow-up (below).
- **Mid-match save & resume** (spec 005) — an in-progress match auto-saves
  to a versioned JSON file (`<data_dir>/saves/savegame.json`) and resumes
  via a **Continue** item on the start menu. A `SavedGame` projection stands
  in for `GameState` (which can't derive serde — `GamePhase` holds an
  `Instant`), carrying a schema version and re-arming the opponent's
  think-timer on load; there's no RNG/deck state to persist (already-drawn
  cards live in the saved `PlayerState`). Saves fire on state change and
  clear on match completion; Start Game over a save confirms first. The
  three menu modals were consolidated into one `Modal` enum along the way,
  and the engine stays untouched but for serde derives. Scope was the match
  only — campaign-level persistence waits for the campaign (see backlog).
- **Control & input polish** (spec 006) — three input quality-of-life
  changes that make **Space** the single "confirm / proceed" key: it plays
  the highlighted hand card on your turn (like Enter), advances at the
  round-end pause (like `n`), and starts a new game at game over (like `g`).
  Drawing moved to its own dedicated key, `D` (Space no longer draws).
  **Emacs nav keys** `Ctrl+P/N/B/F` mirror Up/Down/Left/Right everywhere the
  arrows navigate (start menu, settings, in-game hand cursor, discard-
  confirm), via a pure `resolve_key` translation at the input boundary in
  `main.rs`. The `?` help overlays document all of it. The engine was touched
  only at its input mapping (`game_action_from_key`); the in-play "play the
  selected card" reuses the existing `app.rs` cursor model.

## Backlog

### Campaign (epic — now being actively sequenced)

The campaign is an **integration layer** over several self-contained
subsystems, each its own spec, buildable and testable in isolation (behind
a debug hook) before the campaign map stitches them together — so the pieces
are proven working before the glue is written. The progression model and
campaign-shape decisions live in `DECISIONS.md` ("Campaign design"). Labels
A–E are design handles, not spec numbers (assigned when a spec is picked
up); the order is **dependency-driven, not a rigid schedule**. (This is the
"priorities get set once the engine works" trigger from the top of this
file — the engine works, so the campaign is now being planned in order.)

- **A · Opponent roster & personalities** — a named roster with distinct
  difficulties and play styles, replacing the single generic "Opponent".
  `decide_opponent_move` is already an isolated policy keyed off one
  `STAND_THRESHOLD`, so personalities scale from cheap (per-opponent
  thresholds + behavior flags) to bespoke (custom logic for signature/boss
  fights); each opponent also carries its own side deck and flavor (name,
  difficulty, optional taunts/portrait). No persistence required — testable
  via an opponent-select debug menu. **Self-contained; recommended start.**
- **B · Collection & side-deck customization** — a deck-builder screen that
  shows the cards you own and lets you swap them in/out of a 10-card side
  deck, classic-Pazaak style; matches then deal from your built deck.
  Introduces the **profile save** (`profile.json`) — a persistent document
  distinct from the match save (spec 005), which later specs extend rather
  than replace. Self-contained; shippable before C (arrange a starter
  collection), but most meaningful once C makes the collection grow.
- **C · Economy & progression** — **credits** from wins; a card pool that
  **unlocks by campaign depth**; a **shop** selling from the unlocked pool;
  and a **random card drop from each win** pulled from that same pool.
  Scarcity is the depth gate, not new card types (the canon pool is
  complete). Extends the profile save with credits + owned cards. Depends on
  B, and on a progression-depth input (stubbed until D).
- **D · Campaign map** — an overview map of **Star Wars planets**, Outer Rim
  → Core, difficulty and opponent-count rising core-ward; a node may hold one
  or several opponents. Tracks run progress, supplies the depth C reads,
  grants C's rewards on win, and reworks the main-menu new-game / continue
  semantics for a campaign. The integration spec — depends on A/B/C, and may
  split into more than one spec (map + navigation, then rewards/meta).
- **E · Roguelike mode** (stretch, optional) — an alternate run structure:
  go as far as you can, a fixed number of losses before you restart. Reuses
  A–D's infrastructure; last.

Cross-cutting, not their own specs: a **balance/tuning pass** once
progression exists (playtest-heavy, iterative), and **profile-save migration
discipline** as the schema grows (reuse spec 005's versioning).

- **Smarter / board-aware opponent AI** (builds on spec 007's roster) — 007's
  AI is deliberately minimal: deterministic, plays only its own hand, and
  personality is a per-opponent `stand_threshold` + side deck. A later spec can
  make opponents **board-aware** (e.g. stand once safely ahead of the player's
  visible total, vary aggression by position) and give signature/boss
  opponents **bespoke strategies**. The 007 design accommodates this **without
  rework** — captured now so the upgrade path isn't rediscovered later:
  - `decide_opponent_move(&self)` already has the whole `GameState` in scope,
    so `self.player`'s board is reachable with **no plumbing** to add; turns
    alternate one action at a time, so the player's current board is visible at
    decision time (exactly what positional play needs).
  - Personality lives entirely in `OpponentProfile`: grow it with a behavior
    flag or two, an `AiStrategy` enum the decision fn matches on, or even a
    `decide: fn(&GameState) -> OpponentAction` per profile — all `Copy` /
    const-roster-friendly.
  - The **stable seam** is `decide_opponent_move -> OpponentAction`; its caller
    (`play_opponent_turn`) and the action type don't change as the brain grows.
  - **No save change** — the save persists the opponent *id* and rebuilds the
    profile from code, so richer profiles resume for free.
  - The one concrete addition: a test helper that sets **both** boards (today's
    `opponent_at` sets only the opponent's).
  - Deferred on purpose: adding empty strategy scaffolding in 007 would be
    speculative generality (see `CLAUDE.md` → Simplicity); the extension is
    cheap when actually wanted.

### Other (not campaign-dependent)

- **Play log / move history** — a running record of every move both
  players make during a game (dealer draws, cards played with their
  chosen sign, flips, stands, busts, round outcomes), shown as it
  happens. Presentation is open for future discussion: a side panel, a
  toggleable pane, or a separate popup the player invokes with a
  keypress. Would build on spec 002's overlay frame and monochrome
  vocabulary. The engine already routes all state changes through
  apply_*_action, so those are the natural points to record from.
- **Considered animation pass** — deliberate, sparse animations that
  guide the eye during play: a dealt card arriving, a flip resolving,
  a total changing, round transitions. Builds on spec 002's selection
  pulse and its Motion rule in `design/brief.md` (motion is emphasis;
  one vocabulary — emphasis transitions over time — never ambient or
  decorative movement). Designed against the finished UI overhaul, not
  in advance of it.
- **Original cantina-vibe music** — the bundled track (Kevin MacLeod's
  CC-BY "Chipper Doodle", spec 004) is a good placeholder but not the
  target vibe. Generate or commission an original chiptune track closer
  to the *Star Wars* cantina-jazz feel — jazzy, swingy, a little exotic —
  **without** copying the copyrighted theme (which, along with the KOTOR
  Pazaak music, can't be used; see DECISIONS.md). Nothing in the CC0/CC-BY
  libraries surveyed got close to that specific flavor, so an original is
  the path. Human-requested during spec 004.
- **Post-v1 rule enhancements** — once the complete game exists as a
  baseline, consider Kaazap-specific rule variants that suit the TUI
  format (a mid-match hand-redraw mechanic is one candidate). Evaluated
  against the finished core game, not designed in advance.
