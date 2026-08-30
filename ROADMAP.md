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
- **Cursor-selection interaction model** — replace direct-keypress card
  play with a unified selection model (arrow-key navigation, value
  toggling on ± cards, confirm-to-play), shared with the campaign
  screens (shop, pack opening, opponent select) once those exist.
  Deliberately deferred from the core-engine spec so it gets designed
  once, with all its use cases known.
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
