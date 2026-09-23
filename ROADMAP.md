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
  changes that made **Space** the single "confirm / proceed" key: it played
  the highlighted hand card on your turn (like Enter), advances at the
  round-end pause (like `n`), and starts a new game at game over (like `g`).
  Drawing moved to its own dedicated key, `D`. **The in-play half is superseded
  by spec 023**: on the player's turn Space now *draws* (D still does too),
  Enter or P plays the selected card, and 1–4 select rather than play. The
  round-end and game-over roles are unchanged.
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
  owned counts. Matches deal the player's hand from the built deck (campaign
  matches only, between spec 022 and spec 024 — **superseded by spec 024**,
  which gives **every** match, Quick Play included, the built deck) — the player
  deck moved from the `DEFAULT_SIDE_DECK`
  const onto `GameState`, mirroring how spec 007 moved the opponent's deck — and
  each match **snapshots its deck into the save**, so editing your deck never
  rewrites an in-progress match; resume falls back to the default for a pre-spec
  or malformed deck. Decks must be exactly 10 to play (an incomplete deck routes
  Start Game to the builder). The whole 15-card side-card universe is
  `card::ALL_SIDE_CARDS`; the starter was the default 10 + a few spares —
  **superseded by spec 022**, which gave a fresh profile its own Outer-tier
  `profile::STARTER_SIDE_DECK` plus lateral spares and left
  `card::DEFAULT_SIDE_DECK` as the opponent baseline and Quick Play's deal — the
  Quick Play half **superseded in turn by spec 024**, which leaves
  `DEFAULT_SIDE_DECK` the opponent baseline *only*. A
  two-panel "briefcase" builder is a logged follow-up (below).
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
  counts, and a live balance (also in the map header), and listing only that
  unlocked pool — **superseded by spec 025**, which lists all 15 cards grouped
  by region with the un-reached groups dimmed and locked; what the shop sells,
  and what it costs, is unchanged. Everything grows the
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
  Campaign offers **Continue** vs **New Campaign**; New Campaign was a full fresh
  start (`Profile::reset_to_starter` — wipes progress, credits, and collection/deck
  to the starter, keeping settings) behind a default-No confirm. A no-progress
  profile opens the map directly — **superseded by spec 024**, which made New
  Campaign reset the **map only** (you keep credits, collection and deck), added
  a third **Reset Everything** choice for the old full wipe, and widened the
  panel's trigger to "the run has progress *or* the pool differs from the
  starter". The default-No confirm and its `confirm_choice` seam are unchanged. Menu/profile only — no engine, board,
  or save-format change; rendering DRYed into a shared two-choice overlay helper
  — **generalized by spec 024** to `draw_choice_panel`, which takes any number of
  labels and serves the three-choice entry panel too — and the irreversible wipe
  guarded by a mutation-checked `confirm_choice` seam, which still guards both of
  spec 024's reset scopes.
- **Two-panel briefcase deck-builder** (spec 015) — the side-deck builder is now a
  two-panel "briefcase": **Collection** (left) and built **Deck** (right), each a fixed
  album of every card type — owned shown as solid card frames with a copy count, the rest
  as faint corner-tick "ghosted slot" placeholders — where building is moving a copy
  across (`Tab` switch, `Enter` move; the cursor skips placeholders). Also openable from
  the campaign map via **`c`** for between-match retooling (a small capability bump beyond
  the roadmap's "presentation-only" framing, human-ruled), which also fixed a pre-existing
  bug: the incomplete-deck divert returned to the menu, now the map. Content-sized panels
  (no scrolling); a nav that can't move plays a distinct "declined" cue.
  `deck_builder`/`layout`/`app`/`campaign_map` only — no engine, save, or economy change.
- **Opponent portraits** (spec 016) — every opponent (plus a generic fallback) now has a
  face: a low-resolution **monochrome** portrait shown in an always-visible
  **opponent-presence panel** beside the board in-match, and as a preview in Quick Play
  opponent-select and on a focused campaign-map node. A new `portrait.rs` render path
  (`draw_portrait` + a shared `draw_presence_panel`), one authored `.txt` art asset per
  opponent (`assets/portraits/`), and a `portrait` field on `OpponentProfile` (data, not
  logic, with the generic as the never-blank fallback). The always-on panel **grew the
  minimum terminal 89×31 → 139×31** — **superseded by spec 026**, which makes 139 a
  threshold rather than the minimum (the minimum is 89×31 again; below 139 the board
  draws without the panel and a staked match's stake moves onto the status band) — the
  board is unchanged and still centered, the panel
  sits in the right margin, and the equal left margin is reserved empty for a future
  player-status panel; the panel also reserves rows for the coming banter line + round pips.
  Monochrome by construction (no color path), blessed by a `design/brief.md` bounded-
  exception amendment. Portraits are original block art (no trademarked species), authored
  to an in-repo brief and validated/integrated by Claude Code after the mandated art-format
  spike. Banter (below) follows — it now has a face to come from.
- **Opponent banter** (spec 017) — every opponent now has a **voice**: short flavor lines
  in the opponent's own register, fired on match events (match start, each round
  won/lost/tied, each bust, match end) in the presence panel's reserved line beneath the
  portrait, and the panel's reserved **round-win pips** now fill in (opponent-only,
  first-to-3). Ten distinct roster voices plus a neutral generic fallback, several variant
  lines per event class, chosen not to repeat back-to-back. A line lives through its
  reaction window and **clears when the next round's play begins** (phase-based, no timer —
  revised from "persist until the next event" at the person's attestation), blank between
  with the pips anchoring the space. A new `banter.rs` (event-diff mirroring the audio
  snapshot pattern + the authored line tables, keyed by opponent `id`) and an in-match-only
  `draw_presence_extras` filling spec 016's reserved rows; `App` holds the transient line
  (never saved). **No engine, save-format, or AI change; monochrome preserved.** Lines
  authored in-repo by Claude Code (the person edits what doesn't land). Reactions to
  individual card plays and a player-side pips panel stay deferred.
- **Play log / move history** (spec 018) — an in-match, on-demand **play log**: a
  toggleable overlay (**`L`** to open, `L`/`Esc` to close) recording the current
  round's moves as they happen — both sides' dealer draws, hand plays (with the
  resolved `+`/`−` sign or flip), stands, and busts, each with the resulting total —
  plus a running list of this match's round outcomes (winner/tie, both totals, and
  how it resolved: bust / filled-table / stand). It **does not pause** the game — the
  opponent's think timer keeps running and the log updates live under the overlay.
  **Ephemeral**: transient `App` state, never saved — a resumed match opens with an
  empty log and logs from resume on. Built as the project's first **dynamic** overlay
  (the static-text `draw_overlay` machinery was extracted into a public
  `draw_text_overlay`); capture is a per-tick delta diff of `GameState` (the
  audio/banter pattern), so **no engine, save-format, AI, or card-behavior change**;
  monochrome by construction. Applies to both Quick Play and Campaign. Filtering,
  search, cross-round move detail, and an always-on side panel stayed **out of scope**
  (the reserved left margin is still earmarked for the future player-status panel).
- **Play log: full match history + scrollable window** (spec 019) — a
  product-owner amendment to spec 018 after using it: the log now keeps **every
  round's full moves** for the whole match (reversing 018's collapse-to-outcome —
  completed rounds no longer shrink to a one-line result), grouped by round with
  each round's result on its header. Because a full transcript overflows the
  window, it's **scrollable** (`↑/↓` line, `PgUp/PgDn` page), opening pinned to
  the latest and following live moves until you scroll up. `PlayLog` moved to
  `rounds: Vec<RoundLog>`; a dedicated `overlay::draw_scrollable_overlay` draws a
  larger, roomier (~38%×58%, centered) box beside the static overlays'
  `draw_text_overlay`. Still ephemeral/never-saved; no engine/AI/save change;
  monochrome.
- **Stats & records** (spec 020) — a persistent "mastery" layer: per-opponent
  match/round W–L, a lifetime win streak (current + longest), campaign
  completions, collection completion ("N of 15"), and a separate current-run
  tally, shown on a new read-only **Records** `Screen` off the start menu (four
  paged views — Overall, Quick Play, Campaign, This Run — reusing the spec-019
  scroll interaction). Persisted as additive `#[serde(default)]` fields on
  `profile.json` (`stats`) and `CampaignRun` (`run_stats`) — **no
  `PROFILE_VERSION` bump**. Recording happens once at the match-end (GameOver)
  seam, deriving round W–L from `rounds_won`, so abandoned matches record nothing;
  totals/win-rate/combined/collection are derived, not stored. `reset_to_starter`
  now preserves the lifetime `stats` field (the one behavior change). Per-mode
  streaks were ruled out; menu-wide start-menu selection preservation was ruled in
  (every screen's Back now restores the menu cursor). No engine/save-format change.
- **Wager & loss condition** (spec 021) — the campaign economy turned
  two-directional. Every campaign match is played for a **stake** chosen in a
  wager prompt over the map (opens at a per-opponent **ante floor** of
  `(threshold − 14) × 10`, walks in steps of 5 up to the full balance, no cap);
  the stake is **escrowed at launch**, a win pays it back **double**, a loss
  keeps it. Spec 012's free card drop is gone — cards come only from the shop —
  and a fresh or reset profile now starts with a **seed purse of 50**. Cleared
  planets stay launchable as **rematches** against their final opponent (the
  grinding lever the balance pass tunes against), which change no progress and
  can't re-count a campaign completion. Dropping below the cheapest launchable
  ante ends the run: a modal notice on the map, acknowledged into spec 014's
  `reset_to_starter` (starter deck, no progress, seed purse; settings and
  lifetime records survive). The shop holds that cheapest ante back as a
  **reserve**, so shopping can never end a run. Persisted as an additive
  `#[serde(default)] stake` on `NodeRef` beside the in-flight pointer — **no
  `PROFILE_VERSION` bump, no `SAVE_VERSION` bump**, and no engine change
  (`game.rs` / `player.rs` / `card.rs` / `save.rs` untouched). Every number is a
  tunable constant in `economy.rs`; see `docs/economy.md`.
- **Difficulty & economy balance pass** (spec 022) — the difficulty curve is now
  **measured, not guessed**. A headless **balance simulator**
  (`tests/balance.rs`, an ignored integration test run as
  `KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table --
  --ignored --nocapture`) plays a **scripted player** — the AI's own
  deterministic core, no misplays — against all ten opponents with five named
  decks (the starter, the standard deck, and a best deck per card pool), then
  prints the win-rate table plus every spec target and economy bound with
  PASS/FAIL. Tuning moved **data only** (no AI logic, no new mechanics): the
  **starter deck is now Outer-tier only** (`profile::STARTER_SIDE_DECK`, its own
  constant, with lateral spares), the **standard deck** keeps its role as the
  opponent baseline and was what **Quick Play** dealt — **superseded by spec
  024**, which gives Quick Play the deck you built; the standard deck's
  opponent-baseline role is unchanged — the roster was retuned
  (the Outer Rim's weakness moved out of `misplay` and into weak all-1s decks
  (mostly `+1`/`−1`, and no `±` card) —
  Greeb's slip rate fell 0.25 → 0.18 as shipped; Nima 16/Cautious →
  17/Basic; Rix 18 → 19; Kesh 18/Aggressive → 19/Basic), and the **Mid and Core
  card prices rose 50 → 100 and 120 → 200**. Final curve at N = 10 000: all
  eight targets and all five economy bounds pass — the starter wins about
  70 % against Greeb, stays under 41 % across the Mid Rim and under 30 %
  across the Core, while a full-pool deck takes the finale 51 % of the time.
  Three ordinary sampled tests guard points on that curve at a small sample,
  plus one exact
  test that a fresh profile's starter is Outer-tier, so a future data edit that
  breaks it fails `cargo test`. **No engine, save-format or UI change**, and
  existing profiles keep their cards and credits. Method, measurements and
  re-run instructions in `docs/balance.md`; `docs/opponents.md` and
  `docs/economy.md` re-synced.
- **First-run onboarding & controls refinement** (spec 023) — the two ends of
  onboarding, plus the control pass the second one depends on. **A first-campaign
  primer** (`assets/primer_text.txt`) is raised over the galaxy map the first
  time a profile enters it *from the menu* — Start Campaign, Continue, the
  discard-and-enter confirm, or a confirmed New Campaign — and says only what a
  new player must know: every match is staked, a loss forfeits it, going broke
  resets the run, the Outfitter (`b`) sells the cards that get you past the Mid
  Rim. **A first-match popup** (`assets/first_match_text.txt`) is raised over the
  dealt board the moment a profile's first match *starts* — Quick Play or
  campaign, whichever comes first, never on a resume — and names the rules and
  the keys, holding the match (including an opponent-first deal) until it is
  dismissed. Both are `Modal` variants drawn through the existing overlay seam,
  shown **once per profile**, dismissed with Enter, Space or Esc, and blocking
  every key underneath. The marks are two serde-defaulted `bool`s in
  `profile.json` (`primer_seen` / `first_match_seen`) that **survive a run-over
  reset and New Campaign** like the lifetime stats — `PROFILE_VERSION` stays 1
  and `save.rs` is untouched, so existing profiles load unset and see each piece
  once. **Controls refinement** (supersedes spec 006's goal 2): **1–4 now
  *select* a hand card** like ←/→ instead of playing it, **Enter or P plays**
  the selected card at the sign shown on it, and **Space draws** on the player's
  turn (accepting the bust while over 20, like D) while keeping every other
  "proceed" role it had. The separate "+ or −?" prompt 1–4 used to open on a ±
  card is **retired** — ↑/↓ on the card is the only answer — though the engine's
  `AwaitingSignChoice` phase and its actions stay as an unobservable transient
  so `save.rs`, `tests/balance.rs` and every `sign_*` engine test are unchanged.
  How to Play gained a short **campaign section** so the primer's content is
  findable afterward, the in-game `?` overlay and the board's turn hint were
  re-synced, and the opponent select screen finally said **"Quick Play deals the
  standard deck."** (spec 022's deferred line) — **superseded by spec 024**,
  which made Quick Play deal the built deck and rewrote that line as **"Quick
  Play deals your deck. Nothing is staked."** No rules, AI, economy, wager or
  settlement change.

- **Endgame, victory & what you keep** (spec 024) — the win finally has an
  ending, and one principle is now explicit across the game: **what you earn is
  yours.** Acknowledging the game-over popup of the match that **completes the
  run** lands on the galaxy map with a **victory notice** over it — title, a
  keep-your-pool note, the run summary, one dismiss line — raised once per
  completion (a rematch win on a complete run raises nothing; a replayed
  campaign's completing win raises it again), dismissed with Enter, Space or
  Esc, and holding every map key while it is up. The run does **not** end: the
  map stays open with rematches and the Outfitter. The **run summary** —
  matches played / won / lost, credits won and lost this run, best streak,
  worlds cleared — is one pure builder shown on **both** notices, so going broke
  now reads as a score too (folding in the backlog's *run summary on the
  run-over notice* item). The run tally gained two serde-defaulted counters
  (`credits_won` the net gain on each settled win, `credits_lost` each forfeited
  stake; a stake forfeited by discarding a saved match counts toward neither),
  and the lifetime stats gained one optional **first-clear record** — matches
  played in the run that produced the profile's first completion, set on the
  0 → 1 completions edge, never overwritten, folded into the Records Campaign
  line (`Campaign completions: 2  ·  first clear in 14 matches`) so the
  breakdown table keeps its fixed offset. The map header's axis label gives way
  to **`★  Campaign complete`** until the map is reset. **Quick Play now deals
  the deck you built** (superseding spec 022's ruling): `player_deck_for` is
  gone, `start_match` has one deal for both modes, and the opponent select line
  reads "Quick Play deals your deck. Nothing is staked." **New Campaign keeps
  your cards and credits** (superseding spec 014's full-wipe New Campaign): it
  resets the map only — beaten opponents, the in-flight match and its escrowed
  stake, the run tally — while a third entry choice, **Reset Everything**, is
  the old full wipe. Start Campaign's panel is therefore three choices, shown
  whenever the run has progress *or* the pool differs from the starter.
  Structurally, `App::tick`'s two profile calls became one
  `Profile::resolve_match` that owns the record-then-settle order (the
  first-clear number depends on it), `draw_two_choice` became an N-label
  `draw_choice_panel` that gives the acted-on choice row its blank row above
  even when a note is showing (which also corrected the spec-021 discard
  confirm), and `draw_run_over` became a shared `draw_notice`. No engine, AI,
  wager, settlement-math or save-format change; `tests/balance.rs`, `Cargo.toml`
  and `Cargo.lock` untouched; `PROFILE_VERSION` and `SAVE_VERSION` stay 1.
  `Readme.md`, `docs/economy.md` and `docs/balance.md` re-synced (the last with
  a short *Replays* note: a replay starts premium, deliberately not retuned).
- **Locked cards in the Outfitter** (spec 025) — the depth gate is now
  **visible**. The Outfitter lists all **15 cards** on every visit, in three
  region groups — Outer Rim 7, Mid Rim 6, Core 2 — each row carrying its price
  and owned count exactly as before. A group whose region the run hasn't
  reached is **locked**: its rows are dimmed *with their prices still showing*
  (ruling A2, revised from A3 the same day, so a player can see what they're
  saving toward), its heading reads `Mid Rim  ·  reach the Mid Rim to unlock`
  in the map's own words (C1), and the cursor **passes over it** (B2), so no
  key can buy a locked card. Because the groups are drawn in tier order and a
  group is unlocked iff `tier <= deepest_reached`, the unlocked cards are
  always the **first `n`** of the listing (7, 13 or 15) — so the cursor stayed
  what it was, a `usize` wrapping over `0..n`, with no skip logic and no
  per-row lookup; a test pins that prefix against `available_pool` as a
  multiset at all three depths. The grouping reads `economy::card_tier` and
  `economy::deepest_reached`, the same pair `available_pool` uses, so the list
  and what's buyable **cannot disagree**. The list is centered **as one block**
  (`list_left`, off the widest line the list can ever draw, headings three
  columns in), so a row no longer re-centers when an owned count gains a digit
  and the cursor never makes the list jitter. Buying is untouched —
  affordability, the ante reserve, prices, the bought/refused sounds and the
  save all behave as specs 012, 021 and 022 left them — and after **New
  Campaign** (spec 024) the Mid and Core groups lock again with their owned
  counts intact. `economy.rs` gained exactly one function,
  `RegionTier::region_name` (the inverse of `region_tier`, with a test);
  everything else is `shop.rs` (`TIERS`, `listing`, `unlocked_count`,
  `heading`, `row_text`, `LIST_ROWS`, `list_left`, and `anchors(num_rows)` in
  place of `anchors(num_rows, n)`). No engine, AI, economy, balance-data or
  save-format change: `card.rs`, `game.rs`, `player.rs`, `save.rs`,
  `profile.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are
  untouched, the tier table and prices stand as spec 022 left them, and
  `PROFILE_VERSION` / `SAVE_VERSION` stay 1. The full list, headings, balance
  and hint fit 139×31, pinned by a test and by a driver walkthrough at all
  three depths and across a New Campaign. `docs/economy.md` re-synced.
- **Compact layout below 139 columns** (spec 026) — the minimum terminal is
  **back to 89×31**, where it was before spec 016. **139 is now a threshold,
  not the minimum**: at 139 columns and wider the match draws exactly as spec
  016 shipped it (the board centered, the opponent-presence panel — portrait,
  name, banter, round pips, stake — in the right margin); from 89 to 138 the
  **same board draws alone**, centered, with no panel, no portrait, no banter
  and no pips (the header's `Rounds won` line already carries what the pips
  showed). The one piece of panel information that matters to play, the
  **stake at risk**, moves onto the board: a staked campaign match shows
  `Stake ◈ N` right-aligned on the status band's upper row, drawn `Strong`,
  from the first frame through the game-over popup, sharing the over-20
  alert's row (a test pins the alert ending left of a six-digit stake with
  every cell between blank; the 81-column band leaves ≥ 40 of them) so the
  31-row board block does not grow; Quick Play shows no stake line. The
  layout is **chosen by width alone, live, with no setting** — `BoardLayout`'s
  panel is an `Option<Rect>`, `Some` iff `cols >= WIDE_LAYOUT_MIN_WIDTH` (the
  renamed `IN_MATCH_MIN_WIDTH`, same value and derivation), so a resize from
  139 to 138 mid-match redraws the next frame compact and a resize back
  restores the panel, keeping the match, the hand cursor and an open help
  overlay (pinned on `App::resize`). The opponent-select preview and the
  campaign map's portrait rail **keep their portraits at every width**; at
  89 every planet and label is on frame and clear of the rail (pinned). Along
  the way: the stake row on the wide panel now **stays through the game-over
  popup** on both layouts (Q6, `App::stake_to_show` reads the settled amount
  at `GameOver`; it had been blank there since spec 021), and the play log's
  width floor rose 40 → 52 so the compact log is the same box as the wide one.
  Every "fits the minimum terminal" test now measures 89×31 as well as 139×31:
  the tests in `layout.rs`, `overlay.rs`, `shop.rs`, `app.rs`,
  `opponent_select.rs`, `board.rs` and `records.rs` loop `Config::fit_sizes()`,
  while the wager prompt's test reads `Config::min_size()` and so measures 89
  with `wager.rs` untouched (its 139 case follows from a centered, content-
  sized box — spec AC 3). The too-small screen and the startup error both
  quote `89 x 31`. No engine, AI, economy, wager, save-format or balance change:
  `main.rs`, `card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`,
  `economy.rs`, `wager.rs`, `campaign.rs`, `campaign_map.rs`,
  `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are untouched and
  `PROFILE_VERSION` / `SAVE_VERSION` stay 1. Driver walkthroughs at 89×31 and
  139×31 (every screen, a staked match, Quick Play, the resize across the
  threshold and the too-small screen) attested. `Readme.md`'s terminal-size
  paragraph re-synced.
- **Animation pass** (spec 027) — the match board now has **sparse, one-shot
  transitions** that guide the eye to what just changed, in the vocabulary
  spec 002's Motion rule set: emphasis over time on an element already at
  its final position, no sweeps, no particles, no ambient motion. A **card
  arriving** on either side — a dealt card in the dealer row, a played card
  in the played row — lands with the **heavy border** and Strong for one
  arrival beat (600 ms) and settles to its resting look (single or double
  border, Normal); a played card's **emptied hand slot shows a source ghost**
  (a plain single-line outline, empty face, no number key) for the same beat,
  the "it came from there" half of a source-and-destination pair with nothing
  flying between them. A **Score that changed** draws Strong for the beat and
  rests Normal (it was bold at all times before, so the beat could not show —
  Q7; `Rounds won` and the slot counter never transition). The **round and
  game-over popups wait one popup beat** (800 ms) after the round resolves,
  so the deciding card is seen uncovered with its own highlight; `n`, `g` and
  `x` work from the first frame and the popup then simply never appears. The
  opponent's thinking pause carries a **stepping indicator** (`Opponent's
  Turn .` / `..` / `...`, every 300 ms) on the status line, Muted, reverting
  the moment the opponent acts. Both sides, both layouts (89 and 139); the
  first frame of a new match, a rematch and a resumed save is drawn settled;
  the selection pulse keeps breathing throughout (`design/brief.md`'s Motion
  section gained one sentence: a one-shot transition may run alongside the
  continuous pulse). All of it is drawing state: a new `motion.rs`
  (`BoardMotion`, a pure per-element countdown observed once per tick from
  the frame-to-frame diff, the same snapshot pattern banter and audio use),
  read by `board.rs`; the engine, `GamePhase`, the timing constants and every
  key are untouched, and no key is ever delayed. Settings gained a third row,
  **Animations On/Off** (saved with the volumes; a file without the key reads
  On), which turns the whole layer off — the Off board is the pre-spec board
  frame for frame apart from the Score resting Normal. Two revisions at the
  phase pauses: the first bold-only arrivals did not register on a thin
  box-drawn card (Revision 1: heavy landings, the source ghost, and a
  face-down `?` flip for dealt cards); the person then judged the flip as not
  making sense and it was withdrawn (Revision 2) — a card's face never
  changes after it is drawn. Portraits stay static (spec 016's "light
  animation" deferral is closed, not reopened). No engine, AI, economy,
  wager, save-format or balance change: `game.rs`, `main.rs`, `render.rs`,
  `layout.rs`, `portrait.rs`, `card.rs`, `player.rs`, `save.rs`,
  `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`, `campaign_map.rs`,
  `opponent.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are
  untouched, `PROFILE_VERSION` / `SAVE_VERSION` stay 1, no new crate, no
  color path. Driver walkthroughs after each phase attested the
  arrivals, the ghost, the popup beat, the dots and the settled first
  frames at 89×31 and 139×31, and the Off state at 89×31. `Readme.md`'s settings mention names the
  row.
- **Crash & data safety** (spec 028) — three ways kaazap failed badly, found
  by an audit of the existing code and all three fixed. **The terminal now
  comes back at every ending `spec.md` enumerates**: quitting with `q`, an
  error reading a terminal event, and a panic in key handling, in the
  per-frame update, in drawing, or on the render thread. A `TerminalGuard`
  declared first in `main` (so it drops last) owns the restore — its `Drop`
  runs while a panic unwinds — and a panic hook installed before raw mode
  **records** the first panic instead of printing it, so the **crash report**
  (one kaazap line, then the panic's own message and location — Q5 a) lands on
  the terminal the player came from rather than on the alternate screen being
  torn down. Signals are deliberately outside the spec: `SIGTERM` and
  `kill -9` still leave the terminal unrestored. **Every file is written whole
  or not at all**: one `paths::write_whole` (bytes to a fixed temp file beside
  the target, then `fs::rename` over it) now serves the profile, the settings
  and the match save, so an interrupted write leaves the previous file
  byte-for-byte unchanged and leaves at most one `*.tmp` of debris, which is
  never `*.json` and so is never loaded and never accumulates. **A profile
  that can't be read is no longer mistaken for a first launch**: missing — or
  no data directory at all — stays silent with a starter profile, but
  unreadable, malformed and wrong-version each keep the file aside as
  `profile-YYYYMMDD-HHMMSS.json` beside where it was (Q1 a) and raise a **data
  notice** modal over the start menu (Q2 a) saying which failure it was
  ("couldn't be read", or "saved by a different version of kaazap" — Q3 b) and
  where the file went. If the move itself fails, the notice says so and a
  process-wide flag **suspends every profile save for the rest of that
  launch**, so nothing overwrites the file that couldn't be read. A match save
  that can't be read gets its own line in the same notice and is **removed**
  (Q4 a): **Continue** is absent, as before, and the notice does not repeat on
  the next launch. One notice per launch carries both failures. Enter, Space
  and Esc dismiss it to the untouched start menu; `q` and `m` still quit and
  mute as they do under every other modal (Q6, ruled by the person at sign-off
  and written into `spec.md`); nothing else acts. `KAAZAP_CRASH_AT`
  (`key`/`tick`/`draw`/`render`/`input`) ships in the binary as the only way
  to demonstrate a crash, following the `KAAZAP_DATA_DIR` idiom and
  deliberately **not** documented in `Readme.md`. No engine, AI, economy,
  wager, balance-data or dependency change, and no new screen or mode:
  `card.rs`, `game.rs`, `player.rs`, `opponent.rs`, `economy.rs`,
  `campaign.rs`, `wager.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock`
  are untouched, `PROFILE_VERSION` / `SAVE_VERSION` stay 1, and the version
  gates and every `#[serde(default)]` are unchanged. Driver walkthroughs
  against a scratch `KAAZAP_DATA_DIR` attested 33 crash-and-quit runs at 89×31
  after Phase 1 (the first pass found `=tick` and `=draw` garbling the report
  6/6, which is why the guard now joins the render thread before restoring)
  and all six data scenarios at 89×31 after Phase 4. `Readme.md`'s **Saved
  data** paragraph names the set-aside file.
- **Tournament rounds** (spec 029) — a campaign opponent is no longer beaten by
  one lucky match. Every opponent is a **series**: **Best of 3** (two match
  wins), and **Best of 5** (three) for the final opponent, The Sovereign on
  Zenith. A match is unchanged — first to 3 round wins — and each match of a
  series is staked, prompted and settled exactly as spec 021 has it. The minimum
  path through the campaign goes from 10 matches to 21. Between matches the
  player stands at the **venue**, a new `Screen` — the tournament hall on that
  planet — which names the planet and the opponent, shows the series score and
  length and the credit balance, and offers **Play**, the **Card Shop**, the
  collection and quit to the menu (`b` and `c` as on the map); the Card Shop
  and the collection return to it. The venue is drawn as horizontal bands:
  its text above and below a dominant **planet art region** that shows **each
  planet's own art** — sixteen monochrome drawings, a 48×20 narrow and a 92×20
  wide one per planet, authored outside the codebase from an in-repo brief and
  validated by a test — with the opponent's portrait in its own column three
  columns beside it, at **every** width. The box is sized exactly to the
  drawing: the narrow drawing below 139 columns (a 50×22 box), the wide one
  from 139 up (94×22), and at every other size the spare space is margin
  around the art and the portrait, never blank space inside the frame; a
  taller terminal centres the whole venue. Every text row is centred on the
  art rather than the terminal. `design/brief.md` gained a second bounded
  exception for the art, beside the portraits'. Starting a series **commits** the player to it: the map is
  unreachable, entering the campaign lands on the venue, and only the series
  resolving or a reset releases the lock. A lost series costs only the stakes
  already lost and resets to 0–0; a won one beats the opponent exactly then,
  and clearing, unlocking and completion counting are unchanged. While locked,
  "broke" is judged against the **locked opponent's ante**, and the Card Shop
  reserves the same floor — one `economy::reserve_floor` behind both. The map's
  planet detail names `Best of 3` / `Best of 5` before a launch; the board's
  status band carries `Series 1 – 0` during every series match at both widths;
  the map banner after the deciding match names the series result beside the
  settlement. Rematches on cleared planets stay single matches launched from
  the map. The shop is renamed **Card Shop** everywhere the player reads it.
  The first-campaign primer and How to Play state the rule and the commitment.
  The balance simulator was re-run with **no lever moved** and
  `docs/balance.md` gained *Series rates*: the curve sharpens in both
  directions (starter vs Greeb 70.3 % a match → 78.8 % a series, starter vs Rix
  28.1 → 19.3), and the starter's chance against the final opponent falls from
  23.2 % to **8.6 %**. No new records or statistics. The **per-planet art
  brief** the drawings were made from ships at
  `specs/029-tournament-rounds/planet-art-brief.md`. No engine, AI, save-format,
  economy-constant, balance-data or dependency change: `game.rs`, `card.rs`,
  `player.rs`, `save.rs`, `opponent.rs`, `Cargo.toml` and `Cargo.lock` are
  untouched, `PROFILE_VERSION` / `SAVE_VERSION` stay 1, and the two new
  persisted fields (`NodeRef::settled`, `CampaignRun::series`) are additive and
  `#[serde(default)]` — a pre-029 profile loads with no series running and every
  beaten opponent still beaten. The person amended the spec mid-implementation
  after walking the venue (R1–R8), then at the merge pause brought the art
  itself into the spec (R9), and attested at every paused phase; driver
  walkthroughs against a scratch `KAAZAP_DATA_DIR` covered the venue, the lock,
  the routing and the board at 89×31 and 139×31, and all eight planets' art at
  both sizes, plus 120×31 and 139×40.
  `Readme.md`'s campaign paragraph names the series, the venue and the lock.

## Backlog

**What's left for v1 (the person, 2026-09-23).** The game is basically
complete. Two things remain before it counts as done: **series-aware
banter** (below, taken as spec 030 with a word-by-word speaking animation)
and **music** — *Original cantina-vibe music* and/or *Per-planet music*
(below). Everything else here is optional polish or post-v1. The
**roguelike mode** is low priority, because the campaign already plays like
one: going broke ends the run and wipes it. The **difficulty curve** is in a
good place; another pass is possible, but nothing changes unless it is
clearly an improvement (see *Re-tune the curve for series play*).

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
  campaign depth**; a **shop** selling from the unlocked pool (and, until
  **spec 025**, listing only that pool — spec 025 supersedes the listing half:
  all 15 cards, grouped by region, the un-reached groups locked and skipped by
  the cursor, while what the shop *sells* is unchanged); and a **random
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
- **E · Roguelike mode** — **low priority (the person, 2026-09-23):** the
  campaign already works like a roguelike, since going broke ends the run and
  resets it, so a separate mode adds little. Original framing: an alternate
  run structure — go as far as you can, a fixed number of losses before you
  restart. Reuses A–D's infrastructure; last.
- **More campaign worlds / roster expansion** — ✅ **Shipped (spec 011** — see
  Shipped above). Grew the campaign to 10 opponents across 8 worlds, ending in a
  new final boss. As predicted, it was pure `const` content: `campaign::PLANETS`
  and `OPPONENTS` entries with no engine/layout/save impact, gated only on the
  roster growth that landed with it. A dedicated **balance pass** on the wider
  spread (does the boss feel tougher than the Magistrate? the mid-tier decks'
  dead flips?) remains a tracked cross-cutting item (below).

Cross-cutting, not their own specs: a **balance/tuning pass** once
progression exists (playtest-heavy, iterative — now homed as the **Difficulty
& economy balance pass** under "Stakes, loss condition & difficulty balance"
below), and **profile-save migration discipline** as the schema grows (reuse
spec 005's versioning).

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

- **Two-panel "briefcase" deck-builder** — ✅ **Shipped (spec 015** — see Shipped
  above). Grew beyond the original "presentation-only" framing into a fixed
  full-universe **album** with placeholders, a **`c`** map launch key, and a
  return-path bug fix. Human-requested during spec 008.

- **F · Tournament rounds** — ✅ **Shipped (spec 029** — see Shipped above and
  `DECISIONS.md`). Every campaign opponent is a Best of 3 series, the final
  opponent Best of 5. The spec questions this entry listed were settled as:
  **each match is staked separately** (spec 021's escrow, settlement and loss
  condition untouched); a lost match inside a series costs its stake and a lost
  series costs **only** the stakes already lost; the map shows the series
  **length** before a launch and the venue and the board show the **score**;
  the records are unchanged; and a save mid-series resumes at the venue, which
  is where the Card Shop and the collection are visited between matches. The
  person's longer-range wish to lean further into the tournament theme stays a
  direction, not an item — per-planet venue art shipped with this spec (ruling
  R9), and per-planet music and series-aware banter (below) are its next
  concrete pieces.

### Immersion & personality (now being sequenced)

Making the opponents feel like people, not just AI parameters — two
independent slices; **portraits shipped first** (spec 016 — see Shipped) —
banter is a voice and needs a face to come from, and now has one — and neither
touches the stakes work below, so they can interleave freely.

- **Opponent banter (monochrome)** — ✅ **Shipped (spec 017** — see Shipped above and
  `DECISIONS.md`). Event-driven flavor lines per opponent on match start, each round
  won / lost / tied, each bust, and match end — plus the panel's opponent round-win pips.
  The home turned out to be **not** the board's status band (that stays mechanical) but the
  presence panel's own reserved line beneath the portrait (spec 016). Ten distinct voices +
  a neutral fallback in a new `banter.rs`, event detection mirroring the audio snapshot
  diff, no layout/engine/save/AI change. As anticipated the real cost was the *writing*.
  The "board-reversing card play" reaction stayed **deferred** (reacts to the four event
  classes only). Opponents still also carry the static `blurb` (spec 007), shown outside
  matches.
- **Opponent portraits (monochrome)** — ✅ **Shipped (spec 016** — see Shipped above
  and `DECISIONS.md`). A monochrome character-art portrait per opponent (plus a generic
  fallback), shown in an always-visible presence panel beside the board and as a preview
  in opponent-select and on the campaign map. As anticipated it needed a layout region and
  a grown minimum terminal (**89×31 → 139×31** — **superseded by spec 026**, which brought
  the minimum back to 89×31 and made 139 the threshold above which the panel draws), and
  stayed monochrome by construction (no
  color path), sanctioned by a `design/brief.md` bounded-exception amendment. The mandated
  art-format spike proved the approach; the portraits were then authored to an in-repo brief
  by a more capable tool and validated/integrated by Claude Code. **Light animation**
  (swapping frames on the pulse/tick) stayed **deferred** to a later spec — **closed by
  spec 027 (Q5 A): portraits stay static**, as the brief's portrait amendment says, and the
  animation pass left the presence panel untouched; **banter** (above)
  is the next slice and now has a face to attach to.
- **Series-aware banter** (deferred by spec 029, 2026-09-20). Since spec 029 a
  campaign opponent is a Best of 3 series (Best of 5 for the final opponent), so
  each opponent's **match-start line now fires two or three times in a row** —
  up to five against The Sovereign — and the match-end lines know nothing about
  whether that match took or lost the series. Accepted as a non-goal there. The
  work is lines, not machinery: `banter.rs`'s event classes would gain a
  series-aware variant of match start (first match, a match with the series
  level, a match that can decide it) and perhaps of match end, reading the
  series the board already shows (`board_series_line`'s inputs). As with spec
  017, the real cost is the writing — ten voices and the fallback.

### Stakes, loss condition & difficulty balance (now being sequenced)

Give failure teeth and make a better deck actually matter. Reworks spec 012's
one-directional economy; near-term scope deliberately trimmed (human-ruled) —
the endgame and the mode-identity question are deferred, below.

- **Wager & loss condition** — ✅ **Shipped (spec 021** — see Shipped above and
  `docs/economy.md`). The economy is two-directional: a player-chosen stake per
  campaign match (ante floor by opponent threshold, even money, no cap),
  escrowed at launch; going broke ends the run through spec 014's full reset
  (starter deck *and* seed purse, settings surviving). The free card drop was
  removed outright rather than made rare, so cards come only from the shop,
  bought with wagered credits; rematches on cleared planets were added in the
  same spec so safe grinding actually exists. The softer *keep-your-cards*
  restart remains a noted future difficulty-lever if full reset playtests too
  punishing. Still **deferred to a later spec/discussion (human-ruled):** what
  "beating the game" awards (the endgame/victory) and the
  casual-campaign-vs-roguelike-mode identity question (see **E · Roguelike
  mode**).
- **Difficulty & economy balance pass** — ✅ **Shipped (spec 022** — see Shipped
  above and `docs/balance.md`). The three coupled levers (opponents, card
  prices, economy constants) were tuned together against measured win rates, and
  the invariant landed as a set of explicit targets: the starter deck clears the
  Outer Rim, **walls at the Mid Rim** (under 50 %) and **cannot credibly take
  the Core** (under 33 %), while funding a Core card by safe floor-stake
  grinding takes **43** expected matches against **11** by betting in the Mid
  Rim with a bought deck — so progress needs better cards or bigger bets. The
  human-flagged caveat stands: "cards required" is probabilistic and the *feel*
  only settles by playtest, so every lever remained a tunable constant and
  `docs/balance.md` records how to re-measure after a change. Distinct from the
  global **Difficulty setting** (easy / normal / hard) in *Other* — that is a
  player-facing selector layered on this curve, which is now the baseline it
  moves relative to.
- **Re-tune the curve for series play, if it plays badly** (raised by spec 029,
  2026-09-22). **The person, 2026-09-23:** the difficulty is in a good place
  overall. Another pass is possible, but change nothing unless we are sure it
  is an objective improvement. Spec 029 turned every opponent into a Best of 3 series and the
  final opponent into a Best of 5, and **measured, it did not change**:
  `docs/balance.md`'s *Series rates* section converts the per-match rates into
  series rates, and the sharpening is real in both directions — the starter
  deck against Greeb goes from 70.3 % a match to 78.8 % a series, against Rix
  from 28.1 % to 19.3 %, and against The Sovereign's best of five from 23.2 %
  to **8.6 %**, while the full-pool deck's 51.3 % there becomes 52.4 %. That is
  the direction spec 022's gates were tuned to want, but the campaign is also
  longer — 21 matches at minimum rather than 10, roughly 25 expected — and
  nobody has played it end to end yet. If playtesting
  says the Mid Rim wall or the final is now too steep, the levers are the same
  three spec 022 tuned (opponents, prices, economy constants), plus two new
  ones: the series length itself (`campaign::wins_needed`, one expression) and
  best of five for the final opponent only. Re-measure with the simulator first
  (`KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table --
  --ignored --nocapture`, as `docs/balance.md` gives it; about 9 s in release). One note for whoever runs it: the
  largest per-match drift between the 2026-09-13 and 2026-09-22 runs, starter
  vs Rix 29.9 → 28.1, is about 2.8 standard errors — plausible as the largest
  of 50 cells with no fixed seed, but if a later run drifts the same way again,
  look at the engine before blaming chance.

### Onboarding, endgame & release readiness (suggested 2026-09-13, after spec 022)

Raised at the spec 022 close-out, once the loss condition had teeth and the
curve was tuned: the loop is complete except for its two ends — what a new
player is told at the start, and what a finished player gets at the end — and
the project is not yet presentable to strangers. Items are independent; the
human's stated priority is the first-run onboarding.

- **First-run onboarding (human-prioritized).** — ✅ **Shipped (spec 023** — see
  Shipped above). Both pieces landed as specified: the first-campaign primer on
  a fresh profile's first menu entry to the galaxy map, and the first-match
  popup the moment its first match starts, each shown once per profile, each
  dismissed with one key, each holding everything underneath while it is up. The
  marks are serde-defaulted profile flags with no version bump, and they survive
  a reset — a player who has read the rules is not re-taught on the way back in.
  How to Play kept its role as the full reference and gained the campaign
  section; the cheap extra shipped too ("Quick Play deals the standard deck." on
  the opponent select screen — **superseded by spec 024**, which made Quick Play
  deal the built deck and rewrote the line as "Quick Play deals your deck.
  Nothing is staked."). The spec also carried the **controls refinement**
  the popup's key list depends on (1–4 select, Enter/P play, Space draws, the ±
  prompt retired), which the backlog had not anticipated.
- **The endgame / victory award** — ✅ **Shipped (spec 024** — see Shipped
  above). The win awards a **victory notice with a run summary** plus one
  lifetime **first-clear record** on the Records Campaign view; the map shows
  **`★  Campaign complete`** afterward; and the run **continues** — rematches
  and the Outfitter stay open, with New Campaign replaying the map with the deck
  you built. A credit bonus and a unique card were ruled out (nothing to buy
  after completion; a new card is an engine and balance change), and New Game
  Plus scaling stays deferred.
- **A run summary on the run-over notice.** — ✅ **Shipped (spec 024**, folded
  into the endgame spec: one `run_summary_lines` builder, two notices). The
  run-over notice now carries matches played / won / lost, credits won and lost
  this run, best streak and worlds cleared between its title and its reset note.
  "Deepest planet reached" shipped as **worlds cleared out of the total**, which
  reads better on a map whose tiers unlock by clears.
- **Archive the last run at reset.** Before the run-over (or **Reset
  Everything**) reset wipes the profile — since spec 024 New Campaign keeps the
  pool and wipes nothing worth archiving but the run tally — write the outgoing
  `profile.json` to a dated
  `runs/` file beside it. Costs nothing, changes no rule, and is insurance if
  the full reset ever feels too punishing in play — the softer
  *keep-your-cards* restart noted in spec 021 stays a separate lever.
- **Warn on the wager screen when a loss would end the run** — ✅ **Shipped
  (chore 2026-09-17**, PR #31; see `DECISIONS.md`). Raised by the person
  2026-09-16 while attesting spec 025: stakes are uncapped, so a player with a
  big purse and a full collection could bet nearly all of it, drop below the
  cheapest ante on a loss, and take spec 021's full reset — cards, deck and
  credits — with only "Lose −N" on the prompt beforehand. The wager prompt now
  carries one more line, **"Lose this and the run is over."**, whenever the
  chosen stake would leave `credits − stake` under `economy::cheapest_floor`.
  Display only: stakes stay uncapped, and the grid, the clamp to the balance,
  commit, cancel, the payout math, the reset itself and every sound are
  untouched. The softer *keep-your-cards* restart (spec 021) and *archive the
  last run at reset* (above) stay separate levers, both still open.
- **A path-injection seam for the profile and save locations** — ✅ **Shipped
  (chore 2026-09-19**; see `DECISIONS.md`). `Profile::path`, the match-save path
  and `Settings::config_path` each resolved `ProjectDirs` for themselves, so no
  `App`-level flow (e.g. "acknowledging the run-over lands on the start menu",
  chore 2026-09-13) could be tested without touching the real data folder, and
  every driver session had to back up the human's profile first. A new
  `paths.rs` now owns all three locations behind a root resolved once —
  `paths::set_root` for tests, the `KAAZAP_DATA_DIR` environment variable for a
  run driving the built binary, the platform directories otherwise (unchanged,
  nothing migrated). **Still open, the follow-up the seam exists for:** seven
  `app.rs` unit tests construct an `App` and so still read the real profile,
  settings and save — pointing them at a scratch root changes those tests'
  behaviour and was left outside the chore's footprint. **Spec 028 raised the
  stakes on it.** `App::new` no longer merely *reads* the real data folder, it
  repairs it: `Profile::load` **moves** an unreadable profile aside to a dated
  name, and `save::check_at_launch` **deletes** an unreadable match save. On
  healthy files nothing happens; on a machine whose data folder is damaged, a
  `cargo test` run now *changes* it — doing what the next launch would have
  done, so nothing recoverable is lost, but doing it from a test run rather
  than from the game. Some of those tests also call `draw` without
  overwriting `app.modal`, and since spec 028 `App::new` can set it, so on such
  a machine the launch notice would draw over the board and fail their row
  assertions (before spec 028 the modal was unconditionally `None` at
  construction). Related trap for anyone testing by hand: **never run
  `cargo test` from a shell with `KAAZAP_DATA_DIR` exported at a fixture
  directory** — it repairs the scenario you were about to test.
  **Spec 029 widened the surface again.** A campaign screen is now chosen from
  the profile (`open_campaign_home` reads the series lock), so any `App` test
  that asserts on a campaign screen passes or fails on whoever's save is on
  disk unless it first sets `app.profile = Profile::default()` — which
  `the_primer_swallows_map_keys` had to, after failing against the person's own
  mid-series profile. And `Profile::save()` has no `cfg(test)` guard, so "replace
  the profile with a default, then drive input" is one save away from
  overwriting the developer's real profile; the one test doing it today is safe
  only because its keys cannot dismiss the modal it sits under.
- **Release readiness** (chores, not a spec): a **CI workflow** running
  `cargo build --all-targets` and `cargo test` on push (none exists);
  **README screenshots** of the menu, a match, the map and the shop;
  a **packaging check** (release binaries for the three targets, the music
  track's CC-BY attribution in the distributed files); a license check on
  every bundled asset before anything goes to itch.io.
- **Show locked cards in the Outfitter** — ✅ **Shipped (spec 025** — see
  Shipped above and `docs/economy.md`). Raised by the person 2026-09-16,
  playing early in a campaign: the shop offered only +1 to +3, −1 to −3 and
  ±1, and nothing said the rest exist — it hid every card above the deepest
  region reached, so the depth gate read as missing cards. Taken ahead of the
  compact layout: the Outfitter now lists the whole 15-card universe grouped by
  region, the locked rows dimmed with their prices showing under a heading that
  names the region that opens them, and the cursor passing over them. No
  economy, price or pool change; **new card types** stayed their own backlog
  item (below).
- **A compact layout below 139 columns** — ✅ **Shipped (spec 026** — see
  Shipped above and `DECISIONS.md`). The minimum terminal had grown to 139×31
  with the portrait panel (spec 016), which is large for a general audience.
  Below 139 columns the match now drops the presence panel (portrait, banter,
  pips) and draws the pre-016 89-column board alone, with a staked match's
  stake moved onto the status band; the minimum is 89×31 again. Chosen by
  width alone, live across a resize, no setting; the portraits and banter stay
  exactly as they were at 139 and above, and the select preview and the map
  rail keep their portraits at every width. No engine change.

### Other (not campaign-dependent)

- **Stats & records** — ✅ **Shipped (spec 020** — see Shipped above). Per-opponent
  match/round W–L, lifetime win streak, campaign completions, and collection
  completion on a read-only Records `Screen` off the start menu, with a separate
  current-run view. Built on the existing `profile.json` save (additive fields, no
  version bump); presentation resolved as a full `Screen` (not a campaign-map
  panel), monochrome. Suggested during the post-spec-009 review.
- **Difficulty setting** (easy / normal / hard) — **on hold (human-ruled
  2026-09-13):** a single well-tuned default curve suits this game better than
  a selector; revisit only if a public release shows the tuned curve losing
  players. Original framing kept below for the record. A global option (in the
  Settings overlay) that nudges how sharply opponents play and/or the player's
  starting resources. Widens the audience for a public / itch.io release at low
  cost. **Now unblocked, and now with a baseline:** the board-aware AI shipped
  (spec 010), so difficulty can scale how well opponents actually *think* —
  e.g. globally nudging the misplay rate and/or the effective threshold, not
  merely the raw stand thresholds — and spec 022 measured the **normal** curve
  those nudges would move relative to (`docs/balance.md`). Any proposed offset
  can be re-measured against the same targets with the balance simulator before
  it ships, rather than being playtested blind. Suggested during the
  post-spec-009 review.
- **Considered animation pass** — ✅ **Shipped (spec 027** — see Shipped
  above and `DECISIONS.md`). Deliberate, sparse animations that guide the
  eye during play: a card arriving on either side lands heavy for a beat
  (with a source ghost in the hand slot a played card left), a changed
  total draws Strong for a beat, the outcome popups wait a beat so the
  deciding card is seen, and the opponent's pause carries a stepping
  indicator. Built on spec 002's selection pulse and its Motion rule in
  `design/brief.md` (motion is emphasis; one vocabulary — emphasis
  transitions over time — never ambient or decorative movement), with the
  brief amended by one sentence so a one-shot transition may run alongside
  the pulse. "A flip resolving" was tried at Revision 1 and withdrawn at
  Revision 2: a card's face never changes after it is drawn. An Animations
  row in Settings turns the layer off. Designed against the finished UI
  overhaul, as intended.
- **Original cantina-vibe music** — the bundled track (Kevin MacLeod's
  CC-BY "Chipper Doodle", spec 004) is a good placeholder but not the
  target vibe. Generate or commission an original chiptune track closer
  to the *Star Wars* cantina-jazz feel — jazzy, swingy, a little exotic —
  **without** copying the copyrighted theme (which, along with the KOTOR
  Pazaak music, can't be used; see DECISIONS.md). Nothing in the CC0/CC-BY
  libraries surveyed got close to that specific flavor, so an original is
  the path. Human-requested during spec 004.
- **Per-planet venue art** — ✅ **Shipped (spec 029** — see Shipped above and
  `DECISIONS.md`). Deferred by spec 029 on 2026-09-20 and brought back into it
  by the person's ruling R9 at its merge pause, 2026-09-22. Each planet's venue
  shows its own art: **sixteen** monochrome drawings, a 48×20 narrow and a
  92×20 wide one per planet, in `assets/planets/`, loaded with `include_str!`
  like the portraits. The path spec 016 established was reused: a brief in the
  repo (`specs/029-tournament-rounds/planet-art-brief.md`), the art drawn by a
  more capable tool, and Claude Code validating it against the brief's
  checklist — now a test, whose canvas sizes are read from the venue's own
  layout rather than restated. The questions this entry listed were settled as:
  **one piece per planet**, two sizes each (the art region is a different width
  at the two layouts); **static**, as the portraits are; and the art draws at
  **every** width, below 139 columns too. Between the two sizes the **box fits
  the drawing** — the person's choice over letterboxing, stretching, extending
  or a third size — so spare space is margin around the art and the portrait, never blank
  space inside the frame. `design/brief.md` records the art as a second bounded
  exception, beside the portraits'.

- **Per-planet music** (raised by the person 2026-09-20, during the spec 029
  conversation, and ruled out of scope there as its own spec). Today one
  looping track plays throughout (spec 004). The direction: a distinct track
  per planet — or per region, which may be the better granularity for eight
  worlds — so travelling core-ward *sounds* different, alongside the venue
  art that spec 029 defers. The two are the same immersion push and were
  raised in the same breath; they are separate specs because one is an audio
  change and the other is art plus layout.

  Not a data-only change, and worth knowing before it is picked up: spec 004's
  audio layer is a **single looping music sink** (see CLAUDE.md's dependency
  note), so playing a different track per location means teaching that layer to
  swap what is playing on a screen transition — start, stop, and the question of
  whether a swap cuts or crossfades. The existing Music volume slider and the
  global `m` mute must keep working across a swap.

  Questions for its spec conversation, not here: per planet or per region; what
  plays on the map, at the venue, and during a match (three surfaces, and the
  match may want to stay neutral); whether the track follows the planet or the
  opponent; and where the tracks come from. That last one is the real cost and
  it compounds — **Original cantina-vibe music** below is already the unsolved
  sourcing problem for *one* track, and this item multiplies it by the number of
  regions, with the bundled repo size growing to match.
- **More side-card types** (raised by the person 2026-09-16). The
  collectible universe is 15 types (spec 001), and specs 012 and 022 treated
  it as complete, but the original game also has **+5, +6, −5, −6, ±4 and ±5**.
  Adding them is new card types in `card.rs`, a tier and price for each in
  `economy.rs`, the album and shop growing (since spec 025 the Outfitter's
  list is 21 rows — 15 cards plus a blank and a heading per region — against
  the 31-row minimum, and `LIST_ROWS` guards it), opponent decks possibly
  using them, and a re-measure of the spec 022 curve with the balance
  simulator — its own spec, with a balance pass inside it.
- **Post-v1 rule enhancements** — once the complete game exists as a
  baseline, consider Kaazap-specific rule variants that suit the TUI
  format (a mid-match hand-redraw mechanic is one candidate). Evaluated
  against the finished core game, not designed in advance.
