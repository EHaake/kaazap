# Roadmap

Deliberately unordered — this is a backlog, not a schedule. Priorities get
set once there's a working, correct core engine to actually build on top
of, not guessed at here in advance.

- **Core Pazaak engine completion** — give the card system real types
  instead of the current placeholder `LogicCard { value: i32 }`: minus
  cards, flip (±) cards, and special cards (2&4, 3&6, tiebreaker-style).
  The main deck's 0–10 draw range is an intentional rule variant (see
  DECISIONS.md), not something to fix. Everything else below depends on
  this existing first.
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
- **Wire up "How to Play"** — currently a dead menu item
  (`MenuItem::HowToPlay => {}` in `app.rs`); the help-overlay text assets
  already exist and just need to be reachable from the menu too.
- **Full side-deck customization** — collecting/building your own 10-card
  side deck, KOTOR-vendor style. Explicitly deferred out of v1 in favor
  of a simple default deck; revisit once the core campaign loop exists.
- **Post-v1 rule enhancements** — once the complete game exists as a
  baseline, consider Kaazap-specific rule variants that suit the TUI
  format (a mid-match hand-redraw mechanic is one candidate). Evaluated
  against the finished core game, not designed in advance.
