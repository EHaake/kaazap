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
- **Opponent roster & personalities** (spec 007) — the first campaign
  subsystem (A). A named roster of opponents, each with its own difficulty
  (an AI **stand threshold**) and its own **side deck**, chosen from a new
  **opponent-select `Screen`** reached from Start Game. The opponent AI reads
  the per-opponent threshold (was a global const) and each side deals its own
  deck; mid-match save/resume persists which opponent you face (a
  serde-defaulted opponent id — no save-version bump, old saves resume against
  the default). The engine's decision function and phase machine were reused
  unchanged — only their inputs moved from globals to an `OpponentProfile` on
  `GameState`. Roster + tuning are documented in `docs/opponents.md`;
  deterministic threshold+deck difficulty by design, with board-aware/bespoke
  AI a logged follow-up (below).
- **Collection & side-deck customization** (spec 008) — the second campaign
  subsystem (B). A persistent **player profile** (`profile.json`, modeled on
  the settings file plus the match save's version-discard discipline) holding a
  **card collection** (a bag of owned copies) and a **built 10-card side deck**,
  plus a **deck-builder `Screen`** reached from a new **Side Deck** menu item
  where you add/remove copies (arrows/`wasd`/emacs, Enter/Backspace) against the
  owned counts. Matches deal the player's hand from the built deck — the player
  deck moved from the `DEFAULT_SIDE_DECK` const onto `GameState`, mirroring how
  spec 007 moved the opponent's deck — and each match **snapshots its deck into
  the save**, so editing your deck never rewrites an in-progress match; resume
  falls back to the default for a pre-spec or malformed deck. Decks must be
  exactly 10 to play (an incomplete deck routes Start Game to the builder). The
  whole 15-card side-card universe is `card::ALL_SIDE_CARDS`; the starter is the
  default 10 + a few spares (tunable in C's balance pass). A two-panel
  "briefcase" builder is a logged follow-up (below).
- **Campaign map** (spec 009) — the campaign's integration layer (subsystem D),
  scoped to navigation + progression structure with the **economy stubbed**
  (wins record progress; credits/rewards are C). A full-screen, node-based
  **star map** (`campaign.rs` const graph + a `CampaignMap` `Screen`): planets
  Outer Rim → Core, mostly-linear with one fork that rejoins, each holding one
  or more roster opponents; beating a planet's opponents clears it and unlocks
  its dependents — all derived from a `beaten` set + the graph, no stored
  redundancy. The menu gains **Start Campaign** (which discards a saved match
  first, on confirm) + **Quick Play** (the retained opponent-select); Continue
  unchanged. Campaign progress lives in the profile — an `in_progress` node
  pointer marks a match as a campaign match, so a resumed match still routes
  back to the map at game over — and the engine stays campaign-agnostic.
  Ornament: a twinkling starfield behind the nodes (the Motion principle was
  amended for a bounded ambient backdrop, `design/brief.md`). The map is `const`
  data, so more worlds are a one-entry-each addition, gated only on roster
  growth (below).
- **Smarter / board-aware opponent AI** (spec 010) — opponents now **read the
  player's board** instead of playing solitaire to a private threshold. Once you
  stand, the opponent plays to beat your final total: stands the moment it's
  ahead (the headline fix — it used to grind its threshold and could bust a won
  round), plays a hand card that lands a winning total when behind, hits to chase
  otherwise, and resolves ties by the lone-tiebreaker rule. Each opponent gains
  an **`AiStrategy`** archetype (Basic / Aggressive / Cautious / Calculating —
  the ±1 threshold shifters, the min/max winning-total pickers, and the
  Calculating **tiebreaker tie-steal**) plus a per-turn **misplay rate** (the
  rookie slips ¼ of the time, the Master never). The decision fn stays the
  **deterministic core** (unit-tested against both boards); randomness is a thin,
  seam-tested `opponent_action(roll)` wrapper. No engine plumbing and no save
  change — the decision fn already had the whole `GameState`, and strategy is
  `Copy` const data on `OpponentProfile` (rebuilt from the saved id). Roster +
  strategies documented in `docs/opponents.md`; the spec-007 upgrade path held
  with no rework.
- **Roster expansion & new campaign worlds** (spec 011) — the campaign roughly
  doubled: **10 opponents** (was 5) and **8 worlds** (was 4), ending in a new
  final boss beyond the Magistrate. The roster gained a second, contrasting
  personality per difficulty tier (leaning on spec 010's `AiStrategy` archetypes
  so difficulty rises by threshold while play style varies within a tier); the
  boss (The Sovereign) matches the Magistrate's flawless play but carries a
  fully-playable, flip-free deck. The map is a stretched version of spec 009's
  light-branch-that-rejoins diamond — a fork into two two-world lanes that rejoin
  at a Mid-Rim hub, then a linear Core run to the boss. Pure `const` content over
  the spec 007/009/010 patterns — no engine, AI, save, or new-card-type change —
  plus new tests for graph integrity (one start, acyclic, all reachable), map
  legibility at the minimum terminal, roster coverage, and difficulty
  monotonicity along every edge. A tuning/balance pass is still tracked below.
- **Economy & progression** (spec 012) — the last big campaign subsystem: wins
  now **pay out**. Beating a campaign opponent awards **difficulty-scaled
  credits** (10–50, by stand threshold) and **drops one random card**, both drawn
  from a **depth-gated pool** — the three map regions (Outer → Mid → Core)
  progressively unlock more of the 15-card universe. A **shop** on the campaign
  map (the Outfitter, `b`) spends credits on that same pool, showing prices, owned
  counts, and a live balance (also in the map header). Everything grows the
  collection the spec-008 deck-builder already reads, so won/bought cards are
  immediately usable. Pure content + persistence over shipped seams: `credits` is
  an additive `#[serde(default)]` profile field (no `PROFILE_VERSION` bump), the
  reward hangs on the existing once-per-win campaign seam (Quick Play stays
  stakes-free), and the reward math is a pure, roll-seam-testable `win_reward`.
  Board and engine stay economy-free. The economy is **finite per run** (~300
  credits for a full clear). A plain campaign **reset** ships in spec 014 (New
  Campaign); only **NG+** — keeping your arsenal across a replay — stays with the
  roguelike mode (E).
  Mechanics + tuning snapshot in `docs/economy.md`; a dedicated balance pass on
  the reward/price curve remains tracked below.
- **Bounded misplays** (spec 013) — a **correction to spec 010**, from campaign
  playtest: the misplay seam was unbounded, so opponents stood on 0 and conceded
  from far behind. Now a misplay is a *believable* error, never suicidal — it fires
  only while the position is open (no slip once the player has stood or the opponent
  is over 20) and the timid `Hit → Stand` is capped to within `MISPLAY_TIMID_MARGIN`
  (2) of the threshold. The believable open-position errors (greedy over-hit bust,
  card fumble) and the per-opponent rates are unchanged; masters (misplay 0) are
  untouched. Two spec-010 tests re-authored (surfaced), both fix layers
  mutation-checked. Details in `docs/opponents.md` / `DECISIONS.md`.
- **New Campaign / start over** (spec 014) — from playtest: "Start Campaign" only
  ever resumed, with no way to begin again. Now, when cleared progress exists,
  Campaign offers **Continue** vs **New Campaign**; New Campaign is a full fresh
  start (`Profile::reset_to_starter` — wipes progress, credits, and collection/deck
  to the starter, keeping settings) behind a default-No confirm. A no-progress
  profile opens the map directly (unchanged). Menu/profile only — no engine, board,
  or save-format change; rendering DRYed into a shared two-choice overlay helper,
  and the irreversible wipe guarded by a mutation-checked `confirm_choice` seam.

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

- **A · Opponent roster & personalities** — ✅ **Shipped (spec 007** — see
  Shipped above and `docs/opponents.md`). A named roster, each opponent with
  its own stand threshold + side deck, chosen from a real opponent-select
  `Screen`; save/resume persists the chosen opponent. Scoped to deterministic
  threshold+deck difficulty; board-aware / bespoke signature-opponent AI is a
  logged follow-up ("Smarter / board-aware opponent AI", below).
- **B · Collection & side-deck customization** — ✅ **Shipped (spec 008** —
  see Shipped above). A deck-builder `Screen` showing the cards you own and
  letting you swap copies in/out of a 10-card side deck, classic-Pazaak style;
  matches deal from the built deck. Introduced the **profile save**
  (`profile.json`) — the persistent player-owned document, distinct from the
  match save (spec 005), that C and D extend rather than replace. Collection is
  a bag of copies (spec C grows it); the starter is modest. The two-panel
  KOTOR-style "briefcase" layout is a deferred UI follow-up (below).
- **C · Economy & progression** — ✅ **Shipped (spec 012** — see Shipped above
  and `docs/economy.md`). Credits from wins; a card pool that **unlocks by
  campaign depth**; a **shop** selling from the unlocked pool; and a **random
  card drop from each win** pulled from that same pool. Scarcity is the depth
  gate, not new card types (the canon pool is complete). Extended the profile
  save with credits (additive, no version bump); B supplied the collection it
  grows and D the run-progress structure it reads and hangs rewards on. As
  predicted, it needed no engine, board, or save-format change. A dedicated
  balance pass on the reward/price curve remains a tracked cross-cutting item
  (below).
- **D · Campaign map** — ✅ **Shipped (spec 009** — see Shipped above). A
  full-screen node map (original planet names, not trademarks), Outer Rim →
  Core, mostly-linear with a light branch; planets hold roster opponents and
  clear/unlock by a derived graph; Start Campaign + Quick Play front door.
  Scoped to navigation + progression structure — the **economy is stubbed**, so
  it did **not** depend on C; the "rewards/meta" half of the original D-split
  (granting C's credits/cards on win) rides with **C**. Wins record progress
  only; a loss just returns to the map.
- **E · Roguelike mode** (stretch, optional) — an alternate run structure:
  go as far as you can, a fixed number of losses before you restart. Reuses
  A–D's infrastructure; last.
- **More campaign worlds / roster expansion** — ✅ **Shipped (spec 011** — see
  Shipped above). Grew the campaign to 10 opponents across 8 worlds, ending in a
  new final boss. As predicted, it was pure `const` content: `campaign::PLANETS`
  and `OPPONENTS` entries with no engine/layout/save impact, gated only on the
  roster growth that landed with it. A dedicated **balance pass** on the wider
  spread (does the boss feel tougher than the Magistrate? the mid-tier decks'
  dead flips?) remains a tracked cross-cutting item (below).

Cross-cutting, not their own specs: a **balance/tuning pass** once
progression exists (playtest-heavy, iterative), and **profile-save migration
discipline** as the schema grows (reuse spec 005's versioning).

- **Smarter / board-aware opponent AI** — ✅ **Shipped (spec 010** — see Shipped
  above and `docs/opponents.md`). Opponents read the player's board and play to
  win the round, each with an `AiStrategy` archetype + a per-turn misplay rate,
  over a deterministic, unit-tested decision core with a seam-tested randomness
  wrapper. The spec-007 upgrade path held exactly as it had been captured here —
  `decide_opponent_move` already had the whole `GameState`, personality lives on
  `OpponentProfile`, the `decide_opponent_move -> OpponentAction` seam and its
  caller were unchanged, and the save (opponent id only) needed no change — so it
  shipped with **no rework**, plus the one anticipated test helper that seeds
  both boards (`board_at`).

- **Two-panel "briefcase" deck-builder** (builds on spec 008's subsystem B) —
  008's builder is a single grid of owned cards with in-deck/owned count
  badges. A later, **presentation-only** pass can adopt the classic KOTOR
  layout: your **collection on the left, your built deck on the right**, moving
  cards across between the two panels (`Tab` to switch panels, Enter/←→ to move
  a card). It's a visual overhaul, not new capability, so it's **sequenced
  with/after spec C** (Economy) — a two-panel "curate from a big pile" view
  earns its complexity once C's economy grows the collection, whereas today's
  modest starter fits one grid. Also reinforces the campaign's "manage your
  briefcase between matches" feel, and pairs with the spec-D map as the
  between-nodes retooling screen. Human-requested during spec 008.

### Other (not campaign-dependent)

- **Stats & records** — lightweight persistence of play history: win/loss
  record per opponent, longest win streak, matches played, cards collected /
  collection completion %. A "mastery" layer that makes the game feel finished
  and gives a reason to keep playing after the campaign — cheap to build on the
  existing profile save (`profile.json`). Presentation open (a menu screen or a
  panel on the campaign map; monochrome, per spec 002's vocabulary). Suggested
  during the post-spec-009 review.
- **Difficulty setting** (easy / normal / hard) — a global option (in the
  Settings overlay) that nudges how sharply opponents play and/or the player's
  starting resources. Widens the audience for a public / itch.io release at low
  cost. **Now unblocked:** the board-aware AI shipped (spec 010), so difficulty
  can scale how well opponents actually *think* — e.g. globally nudging the
  misplay rate and/or the effective threshold, not merely the raw stand
  thresholds. Suggested during the post-spec-009 review.
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
