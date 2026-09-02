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

- **Opponent personalities** — distinct opponents with different
  strategies/decks, not just one generic AI.
- **Campaign & progression** — currency earned from wins, card packs that
  unlock better side-deck cards, an opponent ladder of increasing
  difficulty.
- **Campaign-level persistence** — *mid-match save/resume shipped in spec
  005 (above).* What remains: persisting campaign progress, currency, and
  unlocked cards between sessions — deferred until the campaign exists to
  produce that state. The spec-005 save file is versioned so this extends
  it rather than replacing it.
- **Full side-deck customization** — collecting/building your own 10-card
  side deck, KOTOR-vendor style. Explicitly deferred out of v1 in favor
  of a simple default deck; revisit once the core campaign loop exists.
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
