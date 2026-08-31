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

## Backlog

- **Opponent personalities** — distinct opponents with different
  strategies/decks, not just one generic AI.
- **Campaign & progression** — currency earned from wins, card packs that
  unlock better side-deck cards, an opponent ladder of increasing
  difficulty.
- **Save/resume persistence** — full mid-game save/resume, plus
  persisting campaign progress, currency, and unlocked cards between
  sessions.
- **Sound** — wire up the already-present `rusty_audio` dependency.
- **Settings screen** — volume and any other runtime-configurable options
  that come out of the above.
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
- **Board slot cap & vertical centering** — cap the cards per side per
  round (dealer draws + hand cards) at 12 slots; a side that fills all
  slots without busting auto-stands (holds at its total — not the
  canonical "filled table wins", deliberately simpler for now). This
  bounds the board's height so it can be laid out as a fixed block and
  centered vertically in the terminal — fixing the spread on tall /
  vertical monitors where the header pins to the top and the hand to the
  bottom. (Also fixes a min/short-terminal artifact surfaced by spec
  002's review: with no dealer cap, a 4th+ dealer card wraps out of
  the one-row-tall dealer zone into the Played area at ~24-29 rows.) A small rules + layout spec: it changes the engine (game.rs),
  so it was kept out of the UI-overhaul spec (002), which held the engine
  untouched. Decisions (12 slots, auto-stand) confirmed with the human.
- **Considered animation pass** — deliberate, sparse animations that
  guide the eye during play: a dealt card arriving, a flip resolving,
  a total changing, round transitions. Builds on spec 002's selection
  pulse and its Motion rule in `design/brief.md` (motion is emphasis;
  one vocabulary — emphasis transitions over time — never ambient or
  decorative movement). Designed against the finished UI overhaul, not
  in advance of it.
- **Post-v1 rule enhancements** — once the complete game exists as a
  baseline, consider Kaazap-specific rule variants that suit the TUI
  format (a mid-match hand-redraw mechanic is one candidate). Evaluated
  against the finished core game, not designed in advance.
