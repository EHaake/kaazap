# Decisions

Business, product, and process context that doesn't belong in `plan.md`
(technical design) — the reasoning behind naming, scope, and process
calls that a future session (or future you) would otherwise have to
reconstruct from chat history that's already gone.

## Naming

"Kaazap" is "Pazaak" spelled backwards.

## Purpose and bar for quality

This is a personal project, but the intent is for it to eventually be
presentable enough to share (portfolio piece, itch.io release). That
raises the bar somewhat versus a pure hobby throwaway: worth caring about
things like a real README, no crashes/panics in normal play, and
documenting platform-specific build requirements (e.g. ALSA dev libs on
Linux once sound lands) — without turning this into a project that needs
enterprise-grade process for a solo effort.

## Rule variants (intentional deviations from real Pazaak)

- **Main deck draws 0–10, not 1–10.** Real Pazaak's main deck only goes
  1–10; Kaazap's includes 0. This was set intentionally during earlier
  development — the original reasoning wasn't written down and isn't
  currently recalled, but it's deliberate, not a bug. Kept as-is; treat
  it as one of Kaazap's subtle rule changes rather than something to
  "fix" without a real design conversation first.
- **No per-turn side-card limit.** Real Pazaak allows exactly one
  side-deck card per turn; Kaazap allows any number in a turn,
  including chaining several recovery cards while over 20. Deliberate
  simplicity (no per-turn flag to track) — confirmed as a variant, not
  canon fidelity, during spec 001's review.
- **The draw key accepts the bust while over 20.** Going over 20
  doesn't bust by itself (that part is canon — you may play a card to
  recover before your turn ends); Kaazap's variant is that while over,
  `d` draws nothing and instead stands into the bust, same as `s`, so
  the draw key keeps a meaning in that state. Human-ruled during spec 001
  (T008b). (Spec 006 made Space "play the selected card" rather than
  draw, so for a while `d`/`s` were the bust-accepting keys and Space wasn't;
  **spec 023 gave Space back to drawing**, so Space, `d` and `s` all accept the
  bust now — the in-game controls overlay says "Over 20: Space, D or S accepts the bust." and the board's alert reads "OVER 20!  (Space/D/S: bust)")
- **Flip/bust edge rulings** (canon sources are thin here, spec 001):
  a *standing* player pushed over 20 by a flip card busts immediately;
  a *live* player pushed over gets their recovery window; any side
  still over 20 when the round ends for another reason is bust at
  resolution; and if both sides end up bust, the round is a tie.
- **Table cap of 12, filling it holds (not "filled table wins").** Each
  side's table holds at most 12 cards per round (dealer draws + played,
  combined). Real Pazaak caps at 9 and a side that *fills* the table
  without busting wins the round outright; Kaazap uses 12 and filling it
  simply **auto-stands** you on your current total (which then wins,
  loses, or ties by the normal rules). Deliberately simpler for now — it
  reuses the existing stand/resolve path instead of adding a special
  win condition. The over-20 recovery window still applies while a slot
  remains (an 11-card side over 20 can play its 12th as a recovery).
  Human-ruled for spec 003; revisit the canonical "fill wins" as a
  possible post-v1 variant.

## Why a campaign/progression system exists at all

In KOTOR, Pazaak's minus and special side-deck cards are gated by the
game's larger economy — you buy better cards from vendors using credits
earned across the whole RPG. Kaazap has no surrounding RPG to supply that
context. Rather than just handing the player every card type for free
from the start, Kaazap introduces its own lightweight progression:
currency and/or card packs earned from match wins, unlocking better cards
over a campaign of increasingly difficult opponents. This is what gives
the "distinct opponents," "full save/resume," and "card variety" pieces
of the project a shared reason to exist together, rather than being three
unrelated features.

## Campaign design (progression model & shape)

Direction set with the human in Sept 2026 (post-spec-006) and decomposed
into specs in `ROADMAP.md`'s Campaign epic. The load-bearing product calls:

- **Progression = credits + campaign depth.** The player accumulates
  **credits** (from wins) and **campaign depth** (how far core-ward they've
  reached). Depth **gates which cards are available**; credits **buy** from
  that available pool in a **shop**, and **each win also drops a random
  card** from the same depth-gated pool. Two acquisition paths (deterministic
  purchase + random reward), one scarcity mechanism (the depth gate). Chosen
  over a credits-only minimum for more texture, and over a full
  pack-opening/gacha economy to keep the tuning burden bounded. Scarcity is
  *distribution*, not new card types — the canon side-card pool is already
  fully implemented (`card.rs`).
- **Campaign shape = a planet map, Outer Rim → Core.** Nodes are Star Wars
  planets; you start on the Outer Rim against a few easier opponents and move
  core-ward into more, and more difficult, ones. A node may hold a single
  opponent or several ("depends on the planet"). Broadly linear rather than a
  branching web, for scope — branches stay possible later.
- **Roguelike is a separate, optional mode**, not the primary structure: go
  as far as you can, a fixed number of losses before you restart. A stretch
  layer over the same subsystems, not a reason to complicate the core
  campaign.
- **Personalities are AI + deck + flavor, not reskins.** Because
  `decide_opponent_move` is one isolated policy keyed off a single
  `STAND_THRESHOLD`, cheap per-opponent parameters (stand threshold, behavior
  flags) already play noticeably differently; bespoke decision logic is
  reserved for signature/boss opponents where it earns its cost.

## Side-deck customization (spec 008)

Subsystem B — the first spec to ship the **player profile** (`profile.json`),
the persistent player-owned document specs C and D extend. The load-bearing
product calls (all human-ruled this session):

- **Collection is a bag of copies, not a set of types.** You own N copies of a
  card and may run up to N in your deck (duplicates allowed, classic Pazaak).
  Chosen over a simpler owned-or-not set because spec C's per-win card drops
  and shop purchases grow *copies* — the counts model absorbs that with no
  later migration. (The whole side-card universe is only 15 types, so
  type-level scarcity was never the lever — copies + credits are, per "Campaign
  design" above.)
- **Modest starter collection, seeded by spec 008.** Since the economy (C)
  doesn't exist yet to grow it, B seeds a starting collection: the default
  10-card deck plus a few spare adjusters (`+1`, `-1`, `±2`). The exact list is
  tunable balance data, revisited in C's balance pass.
- **Decks are exactly 10 cards to play** (the classic size). The builder lets
  the deck sit under 10 while editing, but starting a match requires a full 10
  — an incomplete deck routes back to the builder rather than starting an
  under-strength match.
- **A match snapshots the deck it began with** (in the match save), so editing
  your deck from the menu changes future matches, not an in-progress saved one.

The **two-panel KOTOR-style "briefcase" builder** (collection left, deck right,
moving cards between panels) was considered and **deferred to a presentation
pass sequenced with/after spec C** — it's a visual overhaul, not new
capability, and a two-panel view earns its complexity only once the collection
grows large. **Shipped in spec 015** — which grew it past "presentation-only" into
a full-universe album with placeholders (see below). See `ROADMAP.md`.

## Campaign map (spec 009)

Subsystem D — the integration layer that gives the campaign its shape. Scoped
to navigation + progression *structure*; the economy (credits/rewards) is spec
C and is stubbed here (wins record progress only). The load-bearing calls
(human-ruled this session):

- **Front door = Continue + Start Campaign + Quick Play.** Continue resumes an
  in-progress match; Quick Play is the retained choose-any-opponent flow (the
  old Start Game); Start Campaign opens the full-screen map. The
  discard-a-saved-match confirm lives **at campaign entry** — choosing Start
  Campaign over a saved match prompts first, and Yes discards it then and there
  (trade-off, accepted during playtest: peeking at the map and backing out
  costs the save). So once on the map, a launch never has a save to overwrite.
- **First map = multi-opponent planets + a light branch**, using all five
  roster opponents: Cinder → the Ashfall/Drift fork (either order) → The Spindle
  (two opponents). A vertical slice; more worlds are gated on roster growth (see
  `ROADMAP.md`).
- **Full-screen, hybrid coordinate model** — a bird's-eye scatter that still
  trends Outer Rim → Core, with a bottom info panel. A deliberate departure from
  the centered fixed-block layout every other screen uses.
- **Twinkling starfield**, allowed by amending the Motion principle
  (`design/brief.md`, its own commit) — slow, low-contrast, background-only, so
  the functional surfaces stay calm.
- **A loss has no penalty** — it just returns you to the map with the node open
  to retry. Stakes (credits, roguelike lives) are specs C and E.
- **Original planet names, not Star Wars trademarks** (Cinder, Ashfall, Drift,
  The Spindle) — the same IP stance as opponents and music; tunable placeholders.
- **Campaign context lives in the profile, not the engine.** A persisted
  `in_progress` node pointer marks the current match as a campaign match (so a
  resumed match still routes to the map at game over); `GameState` stays
  campaign-agnostic. City-zoom (a planet expanding into a second-level map) is a
  deferred later spec.

## Smarter opponent AI (spec 010)

Board-aware opponents that play to win the round — extending "Personalities are
AI + deck + flavor" above from cheap parameters to real, distinct policies. The
product/process calls (the first two human-ruled this session):

- **Board-aware + per-opponent strategy archetypes**, not a single shared policy
  (opponents must feel individual) and not a near-optimal solver (that erases
  personality and is hard to tune into a fun curve). The AI reasons about the
  current board one action at a time — reacting precisely once the player has
  stood (their target is known), playing to threshold while they're live.
- **A dash of randomness over a deterministic core.** Each opponent has a
  per-turn `misplay` chance of a legal-but-suboptimal move, so a learned opponent
  isn't perfectly exploitable and matches feel a touch human. The decision fn
  stays a pure, unit-tested function of the board; the randomness is a thin,
  separately-tested `opponent_action(roll)` seam. The default opponent's rate is
  0, so the engine and its tests stay deterministic.
- **Strategy is `OpponentProfile` data** — a `Copy` `AiStrategy` enum plus a
  `misplay: f32` — so there's no engine plumbing (the decision fn already had the
  whole `GameState`) and no save change (the save persists the opponent id and
  rebuilds the profile from code). Consistent with spec 007's "personality lives
  in the profile"; the upgrade path logged there held with no rework.
- **Strategy assignment follows each opponent's archetype; blurbs were rewritten
  to match.** A first-pass implementation reassigned strategies off the old
  spec-007 blurbs; the skeptical-review pass reverted to the spec's intent
  (Scrapper = Aggressive, Ace / Master = Calculating) and rewrote Vessa's and
  Rix's on-screen blurbs so the flavor matches how they now play — code, spec,
  plan, and docs agree.
- **Cautious means standing earlier, not conceding.** "Won't over-hit into an
  avoidable bust" is delivered by a lower effective threshold (it stops building
  its own hand sooner); behind a *stood* player it still chases, because
  conceding a winnable round would read as a broken opponent, not a cautious one.

## Roster expansion & new worlds (spec 011)

The campaign's first content-scaling spec — 10 opponents across 8 worlds (was
5 / 4), extending "Campaign map (spec 009)" and "Personalities are AI + deck +
flavor". The calls (first two human-ruled this session):

- **Substantial scale** — +5 opponents and +4 worlds, over a lighter top-up or a
  larger expansion: the point that meaningfully grows the campaign while staying
  legible on the star map (~8 worlds is the comfortable ceiling before the
  hand-authored node positions need real tuning) and quick to balance.
- **Raise the ceiling with a new final boss.** Since stand thresholds cap at the
  sensible 19 and Calculating / misplay-0 is already optimal play, "tougher than
  the Magistrate" is delivered by a strictly better **deck** — the one lever that
  needs no engine change. The Sovereign's deck is **fully playable**: it drops the
  flips (the AI provably never plays a flip — empty `playable_values`) that sit
  dead in the Magistrate's deck, for maximal ± range + recovery + the tiebreaker.
  A test pins the boss's playable multiset as dominating the Magistrate's.
- **Two contrasting personalities per difficulty tier**, differentiated by spec
  010's `AiStrategy` archetype, so the roster grows in breadth without pushing
  thresholds past 15–19.
- **Keep spec 009's light-branch-that-rejoins shape**, stretched to two-world
  lanes, rather than a more-branching "pick your route" map — familiar and
  legible. The graph stays derived data (one start, acyclic, all reachable, every
  opponent on exactly one planet), guarded by tests.
- **Map legibility is tested, not eyeballed.** The node/label positions are
  hand-authored with no runtime overlap check, so a test asserts unique node
  cells and non-colliding, non-clipping labels at the 89×31 minimum.
- **Deferred to the balance pass:** whether the boss *feels* tougher, and the
  mild oddity that the mid-tier decks (brakka/kesh, like the legacy rix/
  Magistrate) each carry a dead flip. Kept for now — consistent and per-plan —
  and logged for the tracked balance pass rather than re-tuned blind.

## Economy & progression (spec 012)

The last of the campaign subsystems — subsystem **C** — making wins pay out, over
"Campaign design (progression model & shape)" and "Side-deck customization (spec
008)". The credits + depth model (not gacha) was pre-ruled in "Campaign design";
the calls newly settled this spec (first two human-ruled this session):

- **Campaign-only earning** (human-ruled) — only campaign wins pay credits and
  drop a card; Quick Play stays a stakes-free practice sandbox. Free by
  construction: the reward hangs on the existing once-per-win campaign seam, which
  Quick Play (no `in_progress`) never reaches.
- **The shop lives on the campaign map** (human-ruled) — the between-worlds
  Outfitter, beside the campaign depth that gates its stock, with the credit
  balance in the map header — rather than a Start-menu store or a post-match panel.
- **Scarcity is distribution, not new card types** (pre-ruled, reaffirmed) — the
  15-card universe is complete, so progression gates *acquiring* the existing cards
  by campaign depth (three region tiers, Outer ⊆ Mid ⊆ Core). Both the win-drop
  and the shop draw from that one growing pool.
  **Clarified — not superseded — by spec 025**: the shop still *draws* from
  that one growing pool, so what you can buy and what it costs are unchanged,
  but it now *lists* all 15 cards with the un-reached groups dimmed and locked.
  The gate reads the same `card_tier` / `deepest_reached` pair `available_pool`
  does, so the list cannot drift from the pool. See *Locked cards in the
  Outfitter (spec 025)* below.
- **A single card drop per win, not a pack/gacha** (pre-ruled) — one roll, one
  card, to keep the tuning burden bounded.
- **Additive persistence, no version bump** — `credits` is a serde-defaulted
  `Profile` field (the `campaign`-field precedent), so old profiles load with
  `credits: 0`; the grown collection is just a longer `Vec<Card>`.
- **The reward is a deterministic, testable core** — credits and the drawn card
  are a pure `win_reward(threshold, pool, roll)`; only the roll is random (the
  spec-010 seam pattern). The thin `App::tick` wiring over it is a conscious
  untested accept: its pieces are all unit-tested and the ordering was
  reviewer-walked.
- **A finite economy per run** — each win pays once, so a full clear yields a
  bounded purse (~300 credits). A plain **campaign reset** (start over from the
  starter, discarding progress/credits/cards) ships in **spec 014**; only **NG+** —
  carrying your earned cards/credits *into* a fresh run — remains deferred to the
  roguelike mode (spec E). *(Amended by spec 014, which draws this line; the
  original phrasing lumped "reset" in with the deferred item.)*
- **Deferred to the balance pass:** whether the reward/price curve *feels* right
  (e.g. the first win's 10 credits can't yet afford a 20-credit Outer card) —
  logged for the tracked balance pass rather than tuned blind.

## Bounded misplays (spec 013)

A **correction to spec 010**, not new scope: spec 010 framed its per-turn misplay
as "the deliberate, **bounded** version," but the shipped code was **unbounded**, so
opponents stood on 0 and conceded from far behind (found in campaign playtest). The
calls:

- **Bounded, not zeroed** — a misplay stays a *believable* human error, never a
  *catastrophic* one; the weakness that makes early opponents beatable is kept, only
  the suicidal outcomes are removed. Lowering the rates was rejected — it makes
  suicide rarer, not gone; the model itself was wrong.
- **Two layers** (both in `game.rs`): (1) a misplay fires only while the position is
  *open* — the player is live and the opponent is ≤ 20 — so a resolved position
  (player stood, or a bust to recover) is always played straight (every deviation
  there is pure self-harm: concede a chase, throw a lead, fumble a save); (2) the
  timid `Hit → Stand` is capped to within `MISPLAY_TIMID_MARGIN` (2) of the
  threshold, so a "chicken out" is standing on ~15, never on 0.
- **Kept as flavor** — the greedy over-hit (`Stand → Hit`, the classic beginner
  bust) and the card fumble (`PlayHand → Hit`) while the position is open. They read
  as weakness without being suicidal, are self-limiting, and the roster is
  anti-correlated (the opponents that bust hardest from a greedy hit barely misplay).
- **Perturbed-threshold model weighed and rejected** — modelling a misplay as a
  noisy threshold re-running the deterministic policy is tidier (and gets "no misplay
  vs a stood player" for free), but it's outcome-equivalent on the catastrophic cases
  (a greedy over-hit busts the same either way) while being a bigger rewrite of an
  already-tested seam — it fails the simplicity mandate. The bounded action-flip is
  the smaller, correct change.
- **Rates unchanged** — the per-opponent misplay rates (0.25 → 0.0) stay the
  difficulty scalar; only the *outcome* is floored to competent. A balance pass on
  the rates stays a tracked cross-cutting item.
- **Two spec-010 tests re-authored**, surfaced not silent (per `CLAUDE.md`): they
  encoded the old unbounded contract, so a pointer note was added to spec 010's
  acceptance evidence. Both fix layers are mutation-checked.

## New Campaign / start over (spec 014)

"Start Campaign" always resumed; there was no way to begin again. A playtester
picked Campaign expecting a fresh start, saw prior progress, and mistook it for a
miscounted loss. The calls (both human-ruled):

- **Offer the choice at Campaign entry** — when cleared progress exists, a
  Continue / New Campaign panel (a `Modal` over the menu), not a separate top-level
  menu item. With no progress the map opens directly (the choice would be a no-op).
  **Spec 024 widened both**: the panel is three choices (Continue / New Campaign /
  Reset Everything) and shows whenever the run has progress *or* the pool differs
  from the starter, so only a truly fresh profile opens the map directly.
- **New Campaign = full fresh start** — wipe campaign progress, credits, and the
  collection/deck back to the starter (`Profile::reset_to_starter` = a fresh
  `Profile::default`), keeping audio/settings (a separate file). Chosen over an
  NG+-style replay that keeps your arsenal, so the early game and the depth-gated
  economy stay meaningful; NG+ stays deferred to spec E (see the amended economy
  note above). Destructive, so it sits behind a default-No confirm.
  **Superseded by spec 024**: New Campaign now resets the **map only** and keeps
  the pool, with a separate **Reset Everything** choice doing the wipe described
  here (same `reset_to_starter`, same default-No confirm). The 014 argument no
  longer holds — since spec 021 every match is even money and a first clear pays
  only progress, so a replay earns no more than rematches already can; going
  broke is now the only thing that takes the pool. See *Endgame, victory & what
  you keep (spec 024)* below.
- **The irreversible decision is a tested seam** — the wipe fires only via a pure
  `confirm_choice(on_yes, key)` (Commit iff Enter/Space + Yes), unit-tested and
  mutation-checked, so a future refactor can't silently flip it (the `cursor_confirm`
  precedent). The remaining menu-input effect wiring stays thin over the tested
  reset, following the existing untested-confirm-handler precedent.

## Two-panel briefcase deck-builder (spec 015)

The spec-008 single-grid side-deck builder became a two-panel "briefcase". The calls
(all human-ruled, several across three visual-review rounds):

- **A full-universe album, superseding "owned cards only"** — both panels show every card
  type in fixed canonical slots; a type absent from a panel (unowned, or all-decked on the
  Collection side / not-decked on the Deck side) is a faint placeholder that fills in once
  present. This deliberately supersedes the first cut's "owned cards only" view — you now
  see the whole set and your gaps — while acquiring cards stays the shop's job (the builder
  never sells or grants). It also made the panels **content-sized** and **removed scrolling
  entirely** (the fixed grid always shows the bounded set at once).
- **Reachable from the campaign map (`c`)** — a small new capability beyond the roadmap's
  "presentation-only" framing, taken because it delivers the "retool between nodes" feel the
  roadmap itself calls for. `c`, since `d` is wasd-movement on the map.
- **Return-path correction (a bug fix)** — routing the builder's Back to its launching
  screen fixed a pre-existing bug: the campaign "incomplete deck" divert returned to the
  menu instead of the map. The divert *mechanism* is unchanged; surfaced during planning.
- **Placeholders are not navigable** — the selection cursor only lands on present (solid)
  cards; it skips placeholders and won't focus an all-placeholder panel. Selecting a card
  you can't act on carried no meaning.
- **Placeholder style: a sparse "ghosted slot"** — faint corner ticks + the card's dimmed
  face, not a dashed frame (which read as "almost a real card"). A `BorderWeight::Dashed`
  weight added mid-spec was removed once corner ticks won.
- **A distinct no-op nav cue** — a nav key that can't move plays the "declined" cue
  (`MenuBack`, the same sound a blocked add/remove makes), not the move cue and not silence,
  so the input registers without implying something changed.

No engine, `Profile`-model, save-format, or economy change — a screen + layout reshape plus
the one map launch key.

## Opponent portraits (spec 016)

Every opponent (plus a generic fallback) got a face — a low-resolution monochrome
portrait shown beside the board in-match and as a preview in opponent-select and on
a focused campaign-map node. The calls:

- **A bounded pictorial exception to the monochrome brief, amended before the code
  landed.** `design/brief.md` ruled out block-art flourishes and made cards the only
  physical metaphor. Rather than quietly break that, the brief gained an amendment (its
  own commit, right after the art-format spike): opponent portraits are the single
  pictorial element — **monochrome, static, opponent-only, one fixed portrait frame
  distinct from the card frame, no color, no animation**. The card frame stays the only
  card-shaped box; the portrait frame is a clearly-non-card element. Human-ruled at plan
  sign-off.
- **Glyph vocabulary (spike-confirmed).** A fixed **18×12** grid, one glyph per cell,
  drawn at uniform plain weight — depth comes from **glyph density**, not attribute
  layers or color (the render model has no color path). The palette is the full/shade
  blocks `█ ▓ ▒ ░`, the half blocks `▀ ▄ ▌ ▐`, the quadrant blocks
  `▖ ▗ ▘ ▝ ▙ ▟ ▛ ▜ ▚ ▞`, and space.
- **Authored assets, not a generation script — and who authored them.** Procedurally
  synthesizing a *recognizable face* isn't feasible, so the spec's "script and/or authored
  assets" took the authored branch. The mandated **art-format spike** (one portrait, looked
  at in the running game) confirmed the approach reads as a face; at the go/no-go the person
  **escalated the actual authoring to a more capable tool** (Fable 5.1) against an in-repo
  specification (`specs/016-opponent-portraits/portrait-art-brief.md`), with Claude Code
  **validating** (12 lines × ≤18 cols, palette-only, pairwise-distinct) and **integrating**
  them. Original alien designs, no trademarked species (the planet-names / no-copyrighted-
  music stance). Recorded in `assets/CREDITS.md`.
- **`OpponentProfile.portrait` is data, not logic** — a `&'static str` embedded with
  `include_str!`, like `blurb`. An unknown or absent id resolves to `DEFAULT_OPPONENT`,
  whose portrait is the generic, so the panel is **never blank** (the Quick Play default and
  any unmapped opponent show the generic).
- **The always-visible in-match panel grew the minimum terminal, 89×31 → 139×31.** The
  board keeps its exact layout and stays centered; the opponent-presence panel is drawn in
  the **right margin**, and the equal **left margin is reserved empty** for a future
  player-status panel (the layout is intentionally asymmetric for now). `IN_MATCH_MIN_WIDTH`
  and the panel width both derive from the portrait size, so they move together; below the
  minimum the existing too-small machinery errors with the required size, unchanged.
- **Reserved, not built.** The in-match panel reserves rows below the portrait for the
  coming banter line + round pips (the next personality spec); the two preview panels use a
  snug rect with no reserved rows. One shared `draw_presence_panel` drawer serves all three
  surfaces; the campaign map reserves a right-side rail (the node field reflows to clear it,
  no planet-position edits).

No engine, AI, save-format, or campaign-logic change — a render path, authored assets, one
data field, and a layout/minimum-terminal change.

## Explicitly deferred out of v1

- **Full side-deck customization** (building your own 10-card deck from a
  collection, KOTOR-vendor style) shipped a simple default deck in v1 instead.
  **✅ Shipped as spec 008** (subsystem B — see "Side-deck customization" above
  and `ROADMAP.md`); the deck-builder replaced the fixed default deck.
- **Mid-run side-deck redraw.** Real Pazaak doesn't redraw your hand
  within a given match, and v1 of Kaazap won't either. Deferred rather
  than rejected: once the full game exists, rule enhancements that fit
  the TUI format get considered as a deliberate post-v1 pass (see
  `ROADMAP.md`), and a redraw mechanic is a candidate there.

## Audio & music (spec 004)

- **No copyrighted Star Wars / KOTOR music, even 8-bit covers.** The
  Cantina Band theme and the KOTOR Pazaak music are copyrighted
  (Lucasfilm/Disney and John Williams; BioWare/LucasArts), and a fan
  chiptune *cover* of them is still a derivative work — not licensable for
  a project meant to be shared. Attribution is not a license: only a
  license grants the right to use, and none exists for fan use. So Kaazap
  ships none of it.
- **Bundled track is CC-BY, credited.** v1 ships Kevin MacLeod's "Chipper
  Doodle" (CC-BY 4.0) as the background loop — properly licensed for reuse
  with attribution (`assets/CREDITS.md`). It's a placeholder: the target
  vibe (the *Star Wars* cantina-jazz feel, evoked not copied) isn't well
  matched by anything in the CC0/CC-BY libraries surveyed, so generating
  an original is roadmapped (see `ROADMAP.md`).
- **Sound effects are generated, not sourced.** All SFX are synthesized by
  `scripts/gen_sfx.py` (square/triangle blips), so they carry no
  third-party licensing.

## Testing

No tests exist in the codebase as of the project's pickup (last commit
Jan 2026). Going forward, game logic gets unit test coverage as it's
written or touched — not retroactively applied to every existing line on
day one, but treated as real discipline from here on, not aspirational.

## Opponent banter (spec 017)

- **Home: the presence panel's reserved line, in-match only.** Spec 016's
  panel deliberately reserved two rows beneath the portrait; banter fills the
  upper one and the round-win pips the lower. The board's own status band and
  round-outcome popup stay mechanical — banter never competes with the play
  prompts. The opponent-select and campaign-map previews keep showing name +
  portrait only (a separate `draw_presence_extras`, in-match callers only; the
  shared `draw_presence_panel` is unchanged).
- **Presentation, not game state.** Banter is transient `App` state
  (`banter` / `banter_last` / `prev_banter`), never written to the save. A
  resumed match shows an appropriate line for the current state (or none),
  not the exact last quip. No engine, save-format, or AI change; monochrome
  preserved by construction (glyphs + `Emphasis`, no color path).
- **Events reuse the audio-layer pattern.** A `banter.rs` `BanterSnapshot` is
  diffed each tick (mirroring `AudioSnapshot`/`audio_cues`) into an event with
  precedence **match-end > bust > round-outcome**, so one line shows per tick.
  The engine itself stays silent; `App` observes it from outside, exactly like
  the SFX layer.
- **Voices live in `banter.rs`, keyed by `id` — not on `OpponentProfile`.**
  A set of lines across eight event classes per opponent would bury the roster's
  balance data (thresholds, decks). The voices are collected where they can be
  read and edited together — the same split portraits took (art in
  `assets/portraits/`, not inline). Ten distinct roster voices + a neutral,
  characterless `GENERIC` fallback (never blank, never another opponent's voice).
- **No back-to-back repeat.** `pick` avoids the last-shown line; `banter_last`
  retains it across a blank so the rule holds even when the line clears between
  events. Repeatable event classes carry ≥3 lines (≥2 for once-per-match) so
  `pick` always has an alternative. Lines authored in-repo by Claude Code — the
  person edits what doesn't land (unlike the portraits, escalated to a design
  tool). [human-ruled]
- **Clearing is phase-based, not timed (revised at attestation).** The initial
  delegated default — a line persists until the next event — read as stale: a
  reaction lingered through the whole next round. Ruled at the T005 attestation
  to **option B**: a line lives through its reaction window (the round-end /
  between-rounds pause and the pre-first-move opening) and clears when the next
  round's play begins — detected by the player's first action of a round
  (`play_resumed`), no wall-clock timer. Blank between, with the pips anchoring
  the space. A rematch (`new_game` in place, the only in-game game-over→playing
  transition) re-seeds a greeting via `match_restarted`. [human-ruled]
- **Round-win pips are opponent-only, spaced (revised at attestation).** The
  panel shows the opponent's round wins (0–3, first-to-3), read live from
  `opponent.rounds_won`. Opponent-only per spec 016's intentional asymmetry —
  the player's mirror pips arrive with the future player-status panel; the board
  headers keep the numeric "Rounds won: N" for both sides. Drawn with one blank
  cell between glyphs (`● ○ ○`) after the contiguous form read as cramped at
  attestation. [human-ruled]
- **The pip count (`ROUND_PIPS = 3`) is layout geometry, deliberately not
  coupled to the engine's first-to-3 threshold.** The panel reserves a fixed
  physical slot; changing match length would force a panel redesign regardless,
  so the pip count can't silently drift into a wrong-but-rendering state without
  a human touching the panel. Per the simplicity rule, the rendering layer does
  not import a game-layer constant for a value this stable. (Pre-merge sweep,
  spec 017 — resolving a carried T001-review note.)

## Play log (spec 018)

An in-match, toggleable **play log** overlay (`L` to open, `L`/`Esc` to close):
the current round's moves in order (dealer draws, hand plays with the resolved
sign/flip, stands, busts — each with the resulting total) plus a running list
of this match's round outcomes. The calls:

- **Presentation, never game state — a `PlayLog` on `App`, never serialized.**
  Like banter (spec 017), the log is transient `App` state; nothing about it
  reaches `save.rs`. A resumed mid-match save opens with an empty log and logs
  only from resume onward — the restored board is never back-logged (the first
  `observe` after `reset` seeds silently). No engine, save-format, AI, or
  card-behavior change; monochrome by construction (glyphs + `Emphasis`, no
  color path).
- **Capture is a per-tick delta diff, not engine hooks.** The spec's "the
  `apply_*_action` methods are the natural recording points" is a why-it's-cheap
  note, *not* a license to mutate from `game.rs` — the constitution forbids
  observing code that mutates, and this spec forbids engine change. So the log
  **observes from the outside**, diffing successive `GameState`s exactly as
  `audio.rs` and `banter.rs` do. Banter fires *one* event per transition; the
  log reconstructs *every* discrete move as an ordered list, resting on a
  test-pinned invariant: **at most one card is added to one side per capture**,
  so a side's `score()` after the diff is exactly its post-move total.
- **The twin-call discipline is load-bearing.** `update_play_log` is called at
  **both** sites `update_banter` is — after input in `handle_key` and every
  frame in `tick`. The `tick` call is what captures the opponent's timer-driven
  moves (the very moves the spec says are hardest to follow); a keypress-only
  call site would miss them.
- **Round-outcome resolution is a precedence classification: bust >
  filled-table > stand.** A round can end with mixed causes (one side stands,
  the other auto-stands on a full table), so "how it resolved" needs a single
  defined rule: both busted → both-bust (a tie); one busted → that side's bust;
  else a filled table → filled-table auto-stand; else a plain stand. This
  precedence is a design decision, not spec-settled, so it is pinned by a test.
- **The first *dynamic* overlay — `draw_text_overlay` extracted.** The three
  prior overlays are static `include_str!` text; the play log renders live
  per-frame content. Rather than shoehorn dynamic content into `Overlay`, the
  box machinery (measure → layout → clear → border → per-line text) was
  extracted into a public free `overlay::draw_text_overlay(config, &[String],
  frame)`; `Overlay::draw_overlay` delegates to it and the static overlays are
  behaviorally unchanged. The log builds its `Vec<String>` from
  `PlayLog::render_lines` and calls it directly — content rebuilt each frame
  from live state, so `Modal::PlayLog` needs no resize-rebuild arm.
- **`Modal::PlayLog` captures input but does not pause the game.** By the
  constitution's own line, a panel opened over `Screen::InGame` and dismissed
  back to it is a modal — the same category as How to Play. `tick` never
  consults `self.modal`, so the board keeps advancing and the opponent's timer
  keeps running while the log is up; the overlay is rebuilt from live state each
  frame, so a move made while it's open appears immediately. Like every other
  modal it captures input (you can't play a card until it closes), but the
  *game* is not paused.
- **Known non-issue: the fixed section yields to the box on an impossibly short
  terminal.** `render_lines` never trims the fixed part (title, outcomes,
  headers, placeholders) — only the move lines, keeping the most recent that
  fit (the spec's stated acceptable degradation). On a terminal short enough
  that `fixed_count > inner_height_budget` (~rows ≤ 10 with several outcomes —
  below what `Config::from_terminal` admits for an in-match layout, so
  unreachable in practice) the fixed lines clip off the bottom rather than
  trim. Logged as a known non-issue; no code change. (Phase-2 review.)
  *(Superseded by spec 019 — the trim/fixed-section split is gone; the window
  scrolls instead.)*

## Play log: full match history + scrollable window (spec 019)

A product-owner amendment to spec 018, ruled after using the shipped log:

- **Reverses spec 018's collapse-to-outcome.** 018 deliberately kept only the
  *current* round's moves and collapsed completed rounds to a one-line outcome
  (its non-goal "not a full match transcript at move-level detail across
  rounds"). The owner found that too lossy — you couldn't review earlier moves.
  The log now keeps **every round's full moves** for the whole match, grouped by
  round with each round's result on its header. `PlayLog` changed from a split
  `outcomes` + per-round `moves` to `rounds: Vec<RoundLog>` (each round its own
  moves + `Option<RoundSummary>`); `round_reset` now *starts a new round* rather
  than clearing. The rest of the observation machinery (the delta-diff, the
  one-card invariant, the precedence classification, the ephemeral/never-saved
  lifecycle, clear-on-match-start/rematch) is unchanged.
- **Scrollable window** was the chosen overflow answer (over
  auto-most-recent-that-fits or grow-only), since a full transcript routinely
  exceeds the window: `↑/↓` by line, `PgUp/PgDn` by page. It opens pinned to the
  latest and follows live moves until you scroll up, re-pinning at the bottom; a
  `▲/▼` hint shows only when it overflows. This **replaces** spec 018's
  fixed-section-plus-most-recent-trim rule (and with it the short-terminal clip
  non-issue above) — `render_body` now emits the untrimmed transcript and the
  draw layer scrolls a viewport over it.
- **A dedicated `draw_scrollable_overlay`** beside the static overlays'
  `draw_text_overlay` — a fixed, larger box (not content-sized) with a pinned
  title, a rule, a breathing row, and a scrolled body. It sizes the box
  **directly** rather than via `OverlayLayout` (whose fixed padding, tuned for
  the small static overlays, ballooned a percentage-sized box toward
  full-screen). Final size ~**38% wide × ~58% tall**, centered — width halved
  from a first ~72% cut at the owner's request. Monochrome preserved.
- **Follow/scroll contract lives in the draw fn, not the key handler**: the draw
  fn is the single source of truth for the clamped offset and the bottom re-pin
  (`app.rs` key handlers only nudge the offset and unset follow). Worth keeping
  in mind so a later change doesn't re-add follow logic to the handler and
  double-manage it.
- **Known tradeoff (accepted):** the result header repeats the opponent name
  (`Round N: <name> wins — You x / <name> y (stand)`), so at the narrower width
  a long opponent name clips gracefully at the edge. Accepted for now; the
  header wording can be shortened later if it grates. [product owner]

No engine/AI/save-format change; `PlayLog` still never serialized.

## Stats & records (spec 020)

A persistent "mastery" layer: per-opponent match/round W–L, a win streak,
campaign completions, and a current-run tally, shown on a new read-only
**Records** `Screen` off the start menu. No mechanic changes.

- **Additive serde fields on `profile.json`, no `PROFILE_VERSION` bump.**
  `LifetimeStats` is a `#[serde(default)]` field on `Profile` and `RunStats` a
  `#[serde(default)]` field on `CampaignRun`, exactly as `campaign`/`credits`
  were added — an older profile (no `stats`/`run_stats` keys) loads all-zero,
  pinned by a test mirroring the credits one. (Note: the new binary always
  *writes* the `stats` block on any save, defaulted-empty, since the field has no
  `skip_serializing_if` — harmless and expected.)
- **Recording happens only at the match-end seam, deriving round W/L from
  `rounds_won`** — a deliberate deviation from the spec's sketch of a separate
  round-resolution recording point. At the GameOver tick the final
  `player.rounds_won` / `opponent.rounds_won` already *are* the per-opponent round
  tally (ties were replayed, never counted), so one once-per-match block records
  match W/L, round W/L, streak, run tally, and completion in one shot. Observable
  results are identical, and this makes "an abandoned match records nothing"
  exactly true (a quit-before-GameOver never reaches the seam), with resume and
  rematch needing no special handling. Guarded by `phase_changed && GameOver`,
  which fires on exactly the entering tick (GameOver is only ever entered inside
  `tick`'s `update()`); driver-confirmed end-to-end (play → quit → relaunch → the
  Records screen shows the persisted record).
- **Derived vs. persisted split — one source of truth.** Only per-opponent
  records, the two streaks, and campaign completions persist. Overall/mode totals,
  win rate, the combined (Quick Play + Campaign) per-opponent record, and
  collection completion ("N of 15") are computed on demand for the screen, so
  nothing can drift.
- **`reset_to_starter` preserves the lifetime `stats` field** (take-reset-restore)
  — the one deliberate behavior change to existing code. Career records are the
  point of a mastery layer, so New Campaign must not wipe them (analogous to
  Settings surviving a reset by living in their own file). The current-run tally
  rides on `CampaignRun`, which reset already clears, so the run scope wipes for
  free. The existing "identical to a fresh profile" reset test was amended to a
  field-wise contract (stats preserved; everything else starter).
- **No per-mode streaks** [Erik-ruled, 2026-09-09]. Only two streaks exist — the
  lifetime **overall** streak and the current-**run** streak, the two that read as
  meaningful. So the Quick Play and Campaign views show *no* streak line; a streak
  appears only on Overall and This Run. Additive to add later if ever wanted.
- **Campaign-completion detection via `run_complete()`, ordered after
  `mark_beaten`.** `Profile::record_match` increments completions on
  `campaign && player_won && run_complete()`; the recording block is placed *after*
  the existing campaign-win block in `tick`, so `mark_beaten` has run and
  `run_complete()` is accurate. It can't double-count: a completed run exposes no
  launchable match. *Superseded by spec 021:* rematches made a completed run
  launchable again, so completion detection moved into
  `Profile::settle_campaign_match` as an edge (`!was_complete && run_complete()`)
  — see "Wager & loss condition (spec 021)", design tensions.
- **Menu-wide selection preservation** [Erik-ruled, 2026-09-09]. The spec
  criterion "Esc returns to the menu with its selection preserved" surfaced that
  *every* screen's Back rebuilt the menu at the top (Continue). Rather than a
  Records-only fix or a spec-wording change, the ruling was to preserve selection
  menu-wide: `App` remembers the last-activated `MenuItem`, and `start_menu()` —
  the single choke point every Back funnels through — restores the cursor to it
  (falling back to the top when that item is absent, e.g. Continue with no save).
  A small cross-cutting UX win that spec 020 paid for once.

- **Records is an overlay, not a full `Screen`** [product-owner call,
  2026-09-09, after implementation]. It shipped as a full-screen `Screen` (spec
  decision C), but a full-screen page read as inconsistent beside How to Play /
  Settings, so it was converted to a centered popup `Modal::Records(RecordsState)`
  with block-centered text (~55%×75% of the terminal, content scrolls). This
  fits `CLAUDE.md`'s existing line — a read-only panel opened over the menu and
  dismissed back to it is an *overlay*, not a mode you navigate *to* — so the
  spec decision was revised, not the constitution. A side benefit: the menu is
  never left, so its selection is preserved without the T007 machinery (which
  still serves the real Screens). `Screen::Records` was removed; the state,
  input, builders, and spec-019 scroll clamp are unchanged.

No `game.rs`/`player.rs`/`card.rs`/`save.rs` change; the mid-match save format is
untouched (stats live in `profile.json`). Monochrome by construction.

## Wager & loss condition (spec 021)

The campaign economy becomes two-directional: every campaign match is played for
a stake, a loss costs it, and going broke ends the run. All of the following were
ruled with the human on 2026-09-10, on the recommendations as proposed.

- **Rematches on cleared planets, against the final opponent.** The balance
  pass's invariant ("safe minimum-wager grinding can't fund the next tier")
  presumes grinding exists — and it didn't: a cleared planet was a no-op. The
  final opponent is the simplest rule; a per-opponent picker can come later.
- **Seed purse of 50.** A fresh profile had 0 credits, which can't stake
  anything. 50 is one Outer-tier card or a few minimum antes.
- **The ante floor reuses the threshold scalar** — the same `(threshold − 14) × 10`
  the old win reward used, so difficulty keeps living in one number.
- **Player-chosen stake, floor to full balance, no cap.** "Bigger, riskier bets"
  has to be a real lever; a cap is a later balance knob, not a launch rule.
- **Even money, and the old win credit removed.** Difficulty now lives in the
  floor, so scaling the payout by threshold too would double-count it. A first
  clear earns progress only.
- **The free card drop is removed outright** rather than made rare. Cards come
  from the shop, bought with wagered credits; a rare drop is cheap to add back if
  playtest wants it. (This also removed the only randomness in `economy.rs` — the
  injected `roll` seam went with it.)
- **Escrow at launch.** The stake leaves the balance when the match starts, so
  the map header is honest mid-match and discarding a saved match can't dodge a
  loss.
- **Broke = can't cover the cheapest launchable floor, checked after a loss and
  at campaign entry; the shop reserves that floor.** The floor stays a real floor
  (no "last stand" below it), and shopping can never end a run. The entry-time
  check doubles as the migration path for a pre-021 profile with no credits: it
  meets the run-over flow rather than a dead map. Accepted for a personal project
  over a progress-preserving top-up.
- **Run-over is a modal on the map, reset via spec 014's path** — one reset
  operation, with lifetime records preserved per spec 020. There is no decline:
  the reset *is* the loss condition.
- **The stake is shown in-match; no "runs ended broke" counter.** The wager's
  tension belongs on screen (it rides in the existing presence panel, inside the
  139×31 minimum); a bust counter is a cheap additive field if the balance pass
  wants it.
- **A campaign completion counts once.** Spec 020's "a completed run exposes no
  launchable match" assumption no longer holds with rematches, so completion is
  tied explicitly to the win that completes the run.

Design tensions resolved during planning:

- **The stake lives on `NodeRef`, not on `CampaignRun` or the match save.**
  `NodeRef { planet, opponent, stake }` with `#[serde(default)] stake: u32`: the
  stake is *part of* the in-flight match context — it exists iff a campaign match
  is in flight, persists in `profile.json` beside the pointer, and is discarded
  with it. A `CampaignRun.stake` field would need its own clear on every path
  that clears the pointer; a `SavedGame` field would put economy data in the
  engine save and force a save-format change.
- **Settlement and completion move into one `Profile::settle_campaign_match` —
  superseding spec 020's `record_match` clause.** Spec 020 counted a completion
  inside `record_match` as `campaign && player_won && run_complete()`, relying on
  "a completed run exposes no launchable match"; rematches void that. The check
  moved into `settle_campaign_match`, which owns `mark_beaten` and detects
  completion as the **edge** `!was_complete && run_complete()` around it — a
  rematch leaves `beaten` unchanged so the edge never fires, and a first clear of
  the final node fires it exactly once. This also dissolves spec 020's "the
  record block must run after the win block" ordering requirement: the ordering
  is now internal to one method, and `record_match` does lifetime + run-tally
  recording only.
- **Exactly-once payout is a data property, not an ordering rule.**
  `settle_campaign_match` calls `CampaignRun::take_stake()` (returns the stake
  and zeroes the escrow) before paying, and `mark_beaten` is idempotent — so even
  a double-fired seam would pay `win_payout(0) == 0`. Consequence: the escrow
  reads 0 from the game-over tick on, so the in-match stake line disappears there
  while the outcome popup and the map banner carry the result. Quitting at the
  game-over screen without acknowledging leaves a stale `in_progress` with stake
  0 — harmless, and replaced by the next launch.
- **One resolution block on the `phase_changed` edge.** The spec-012 every-tick
  campaign-win block and the spec-020 edge-based record block became a single
  block in `App::tick`: `settle_campaign_match` → `record_match` → `save`. The
  `!is_opponent_beaten` once-guard is gone (rematches make it meaningless); the
  once-guarantee is the `phase_changed` edge plus the zeroed escrow above.
- **The broke check is a pure predicate run at two app seams.**
  `Profile::is_broke()` = `credits < economy::cheapest_floor(run)`, evaluated on
  the game-over acknowledgement of a campaign match and at campaign entry, both
  through one `App::enter_campaign_map()` helper. It is deliberately *not* run on
  every `open_campaign_map` (shop / deck-builder Back): the shop reserve makes
  those paths unable to create a broke state, and keeping the check at the two
  spec'd seams keeps the intent legible. A staked win can never leave the player
  broke (`credits ≥ win_payout(stake) ≥ 2 × floor`), so no "only after a loss"
  guard is needed; the one exception is a stake-0 pointer (a pre-021 campaign
  save on a 0-credit profile), where the accepted migration path simply arrives
  one match later.

No `game.rs`/`player.rs`/`card.rs`/`save.rs` change; `PROFILE_VERSION` and
`SAVE_VERSION` both stay 1 (the stake is an additive `#[serde(default)]` field).
Monochrome by construction.

## Difficulty & economy balance pass (spec 022)

The difficulty curve stops being a guess. A headless simulator measures it, the
tuning moves data only, and `docs/balance.md` records the method, the numbers
and how to re-run them. Ruled with the human on 2026-09-12 — on the
recommendations as proposed except **C**.

- **Measure with a headless simulator, then playtest.** The roadmap assumed
  playtest-only tuning; a simulator makes the invariant a number and the
  playtest attests the feel. Test-side code, no new crates, no seed.
- **The starter deck becomes Outer-tier only.** The discovery that drove the
  spec: the starter *was* the premium standard deck, so buying cards was
  optional. The standard deck keeps its role as the opponent baseline; the
  starter is a new, separate constant whose composition the tuning settled
  (`+1 +1 +1 +1 +2 +2 −1 −1 −1 −2`, spares `±1 +2 −2`).
- **Quick Play deals the standard (premium) deck** [human ruling, against the
  recommendation to deal the built deck]. Accepted downside, on record: the deck
  you build only matters in campaign matches, so Quick Play is no longer a place
  to test a build. Reversible in one line if that proves annoying.
  **Reversed by spec 024** — the accepted downside is exactly what proved
  annoying, and "what you earn is yours" made the one profile pool the rule in
  every mode. Quick Play deals the built deck; the standard deck keeps its
  opponent-baseline role. The consequence, accepted: a fresh profile's Quick
  Play deals the starter deck, so Quick Play against the Core is hard until the
  player has shopped. See *Endgame, victory & what you keep (spec 024)* below.
- **Targets, not vibes**: the starter beats Greeb at least two matches in three,
  is under half against the Mid Rim and under a third against the Core; each
  better pool's best deck wins at least as often as the one below it; a finished
  deck has a roughly even finale (45–55 % against the Sovereign).
- **Data levers only.** Opponent data, card tiers and prices, economy constants,
  the starter deck. No AI logic, no new mechanics; an unreachable target would
  have been a question for the human rather than a code change.
- **Existing profiles keep their cards.** Only new and reset profiles get the
  new starter; no migration, no top-up, no version bump.
- **A short balance doc plus re-synced tables** (`docs/balance.md`, and
  `docs/opponents.md` / `docs/economy.md` re-synced), so the next pass starts
  from data.
- **The two spec 021 close-out notes were fixed on `main` before this spec was
  drafted**: the constitution no longer names pack-opening as a future mode, and
  the spec 020 completion bullet points at its spec 021 supersession.

Ruled at the **Phase 2 pause on 2026-09-13**, after the human played the tuned
build:

- **The Outer Rim is retuned to weak decks, not high slip rates.** At this
  pause the human was playing the mid-pass tuning, in which Greeb, Dax and
  Vessa carried their weakness in `misplay` at 0.44 / 0.36 / 0.34 — peaks
  reached inside this pass, well above the 0.25 / 0.22 / 0.15 that shipped
  before it — which read as *random* rather than *weak* in play. Moving it
  into the decks — all-1s hands (mostly `+1`/`−1`, and no `±` card) with
  almost no exact-20 coverage — held the same win-rate curve
  while the slip rates fell to 0.18 / 0.16 / 0.15, so the early opponents now
  play their best line most turns and still lose. The re-measured table is the
  one the docs record.
- **Nima's and Kesh's blurbs were reworded** — a deliberate, **limited
  exception** to the spec's "no roster-text rewrite" non-goal. Their tuned
  `strategy` no longer matched what their old blurbs promised (Nima had been
  described as folding early, Kesh as hair-trigger), so two sentences changed to
  keep the roster text honest. Nothing else in the roster's prose moved.
- **`strategy` is a tuning knob, independent of blurb text.** Nima went
  Cautious → Basic and Kesh Aggressive → Basic for measured reasons (holding an
  effective threshold while the raw threshold moved, and opening the Mid Rim as
  a wall). The archetype is difficulty data first and characterization second;
  where the two diverge, the text follows the data — as it did above — rather
  than the data being held hostage to the text.

Design tensions resolved during planning:

- **The simulator is an integration test, not a `#[cfg(test)]` module in
  `game.rs`.** `tests/balance.rs` uses only the public API, keeps ~300 lines of
  harness out of the engine, is built by `cargo build --all-targets` and run by
  `cargo test`, and is nowhere near the binary. The table run is `#[ignore]`d
  (and `--release`, for 500 000 matches in seconds) so `cargo test` stays fast;
  the guards are ordinary tests in the same file at a small sample size.
- **The scripted player is the AI's own deterministic core, in a fixed order**
  — recover from a bust, land exactly 20, beat a stood opponent (including the
  tiebreaker steal), else play to a stand-at-17 threshold. A deterministic
  function of the board with no randomness and no memory, so a measured
  difference is the *data's*, not the proxy's. Two stated limitations, kept as
  is and documented: it never plays a flip (neither does the AI, which is why no
  candidate deck carries one), and it stands on a tie at ≥ 17 even when the
  opponent alone holds a tiebreaker — a sure loss a human would chase.
- **The simulator does the bound arithmetic and prints PASS/FAIL**, from the
  measured table and the live constants, so a tuning iteration is one command
  and a failing curve prints rather than panics. The ignored run never asserts;
  the separate guards do.
- **"Best deck from a pool" is a hand-built deck, not a search**: the strongest
  of at most three human-legible candidates per pool, measured against that
  pool's region and fixed as a constant. Searching the multisets would need a
  different tool and would answer a question the doc isn't asking ("what would a
  player build?"). The alternatives and their rates are recorded in
  `docs/balance.md`.
- **Quick Play keeps today's deck-validity divert.** `open_opponent_select`
  still sends an under-filled built deck to the builder before Quick Play even
  though Quick Play no longer *uses* that deck; the spec is silent, so the plan
  kept the behavior as a consistency nudge rather than adding a behavior change.
  Dropping it is a two-line change if it ever annoys. **Both halves superseded
  by spec 024**: Quick Play deals the built deck, so the divert is no longer a
  nudge but the thing that upholds `start_match`'s deck-valid precondition for a
  deck that *is* dealt — and dropping it would now be a bug, not a two-line
  tidy.

**Supersedes the spec 008 bullet above**, "Modest starter collection, seeded by
spec 008": the starter is no longer the default 10-card deck plus `+1`, `−1`,
`±2` spares. It is its own Outer-tier constant (`profile::STARTER_SIDE_DECK`)
with **lateral** spares (`±1 +2 −2` — only `±1` is a type the deck doesn't
already hold), so the measured starter rates describe the deck a fresh player
actually fields, and `card::DEFAULT_SIDE_DECK` is the opponent baseline and
Quick Play's deal — the latter **superseded by spec 024**, which leaves it the
opponent baseline only. The "exact list is tunable balance data" part of that bullet
still holds — this pass is what tuned it.

No `game.rs`/`player.rs`/`card.rs`/`save.rs` logic change and no UI change;
`PROFILE_VERSION` and `SAVE_VERSION` both stay 1. Monochrome by construction.

## Chore: run-over returns to the start menu; wager prompt spacing (2026-09-13)

The first fixes run under the constitution's *Chores (no spec)* lane, ruled by
the human after playing the spec 022 build (PR #26; branch kept).

- **Losing the run ends at the start menu.** Acknowledging the run-over notice
  still resets the profile to the starter through the one reset path, but the
  player now lands on the start menu instead of a fresh campaign map. This
  supersedes spec 021's "On acknowledgement … a fresh map opens" — a lost run
  should read as an ending, not an instant restart. New Campaign from the
  map's own panel still resets and opens the map.
- **The wager prompt breathes** *(superseded the same day — see the chore
  entry dated 2026-09-13 below: only the stake row gets air).* An empty row under the title and above and
  below the stake row and the win/lose line; text unchanged; emphasis chosen by
  each row's role rather than a hardcoded index. This is the first application
  of the design brief's *Density and breathing room* rule, added the same day
  because dense modals had come up in review three times; the constitution
  now checks new or changed screens against it at review.
- **Why a chore and not a spec.** Two one-file flow/layout fixes with no
  engine, AI, save-format, balance-data or dependency change and no new
  screen. The lane exists so that fixes of this size are recorded and
  reviewed without the spec → plan → tasks ceremony; anything larger, or
  anything touching the excluded areas, is still a spec.
- **Known gap, deferred.** No unit test covers "after acknowledging, the
  screen is the start menu": `Profile::path` and the match-save path go
  through `ProjectDirs` with no injection seam, so an `App`-level test would
  read and write the real profile. A path-injection seam is a candidate chore
  of its own; until then the run-over destination is verified by playing.

## Chore: only the acted-on row gets air; modals pad evenly (2026-09-13)

The human's correction to the wager-prompt spacing above (PR #27; branch kept).

- **The rule was over-read.** The first application padded every row of the
  wager prompt, which made the box taller without making the stake any easier
  to find. The human's intent is narrower and now stands as the rule in
  `CLAUDE.md` and `design/brief.md`: the one element the player acts on gets an
  empty row above and below; everything else stays compact. The wager prompt
  is back to its original rows plus one blank above and one below the stake.
- **Modal boxes pad evenly.** `OverlayLayout` sized every small modal with four
  rows of padding but placed the content two rows from the top, so each showed
  one empty row above the text and about five below. The box is now border,
  one empty row, content, one empty row, border — for the wager, run-over,
  confirm, settings and help overlays alike. `V_PAD` now means the total
  vertical padding; `H_PAD` is still per side.

## First-run onboarding & controls refinement (spec 023)

A new player is now told the two things the loop never told them — what a stake
costs and what the keys do — each once per profile, in the fewest possible
words. Ruled with the human on 2026-09-13, on the recommendations as proposed.

- **A — The seen marks survive resets.** A run-over and New Campaign wipe the
  run but keep the marks, as they keep lifetime stats; a player who has read the
  rules is not re-taught on the way back in.
- **B — The first match of either mode** shows the popup: the mechanics and
  controls are identical in Quick Play and campaign, so whichever comes first
  gets it.
- **C — Existing profiles see both pieces once.** The profile format could have
  treated a missing mark as seen; the human chose to show them, so an existing
  profile can try them without a wipe, at the cost of one key each.
- **D — The Quick Play line lives on the opponent select screen**, where Quick
  Play is chosen, not in How to Play.
- **E — How to Play gains a short campaign section**, so it is genuinely the
  full reference once the primer is gone.
- **Timing (the human's words).** The primer shows the first time through the
  campaign, on first entering the galaxy screen; the gameplay popup once the
  first game actually starts.
- **F — The controls refinement rides in this spec**, not a separate one or a
  chore: the popup's key list depends on it, and retiring the ± prompt touches
  the engine's phase list, which the chore lane excludes. **Spec 006's goal 2 is
  superseded** — the most common in-play input is drawing and moving on, so that
  is what the spacebar does.
- **G — P is the secondary play key.** Free on the player's turn, mnemonic, and
  away from the draw/stand hand (D, S, Space). Enter stays primary.
- **H — The ± prompt is retired outright** rather than kept for the direct keys:
  with 1–4 selecting, every play goes through the cursor, whose ↑/↓ sign is
  already the answer; a second way to answer the same question would be the
  indirection the constitution tells us to cut.

Design tensions resolved during planning:

- **The engine keeps the sign-choice pass-through; only the player-facing prompt
  goes.** `GameAction::{PlayHand, ChooseSign, CancelSignChoice}`,
  `GamePhase::AwaitingSignChoice`, `play_card`, `commit_sign_choice` and every
  `sign_*` engine test are unchanged; what was removed is the
  `AwaitingSignChoice` branch of `game_action_from_key` (h/l/+/−/1/2/c), the
  `'1'..'4' → PlayHand` arm, and the board's sign prompt. Reasons: the spec
  requires the engine's card tests unchanged and `save.rs` untouched, and
  `SavedPhase::AwaitingSignChoice` exists on disk; `tests/balance.rs` and the
  two headless loops are written against it; and after this spec the only
  producer of `PlayHand` in the binary is `cursor_confirm`, which answers the
  phase inside the same key event, so the phase is unobservable — no key maps
  into it, `update()` leaves it alone, `status_message` returns `None` for it.
  The cost is one transient phase the player can never see; the alternative (a
  signed `PlayHand`) would rewrite `cursor_confirm`, nine tests, the simulator
  and the save format for no visible gain.
- **Where the primer is raised: the three menu-entry sites, not the game-over
  path.** `enter_campaign_map` gained a `from_menu: bool`, and
  `map_entry_modal(broke, primer_due)` is the pure precedence seam — run-over
  first, then the primer, else nothing. The menu-entry callers pass `true`:
  `enter_campaign_continue` (both the no-progress path and `CampaignEntry`'s
  Continue), the `PendingStart::Campaign` confirm arm, and `start_new_campaign`
  — which switched from `open_campaign_map()` to `enter_campaign_map(true)`,
  and is never broke at that moment because the reset just left the seed purse.
  New Campaign is **menu-only** (no `MapOutcome` variant; the confirm —
  `ConfirmNewCampaign`, renamed `ConfirmReset` and given a scope by spec 024 —
  is raised only from `CampaignEntry`), so no origin flag beyond `from_menu` is
  needed. The game-over acknowledgement passes `false` — the spec says the
  primer is not raised from a match's game-over path, which is reachable with
  the primer unseen only by a pre-023 profile resuming a saved campaign match.
  The shop and deck-builder Backs keep plain `open_campaign_map` and never raise
  it: they return to a map already seen.
- **A pre-023 mid-match save can be sitting in `AwaitingSignChoice`.** Pressing
  `1` on a ± card used to save the game in that phase (`save_game` fires on every
  `game_changed`), and after this spec no key maps to `ChooseSign`, so such a
  save would resume soft-locked. Continue therefore applies
  `GameAction::CancelSignChoice` immediately after `save::load()` — a no-op in
  every other phase, and in that one it returns the card to hand at `PlayerTurn`
  by the engine's existing cancel semantics
  (`sign_cancel_restores_turn_with_card_unspent`). `save.rs` is untouched;
  `CancelSignChoice` stays for exactly this.
- **Invariant, recorded at the sweep:** `enter_campaign_map` and `start_match`
  now *assign* the modal (`map_entry_modal(...)` / `Modal::FirstMatch`), so
  every caller clears its own modal *before* entering the map or starting a
  match — the wager Commit, the discard-and-enter confirm, the campaign-entry
  panel and the New Campaign confirm all do. A future caller that raises a
  modal first would lose it silently; the invariant is verified by reading and
  by the driver, not by a test (plan tension §4: App tests never touch disk).
- **Process note:** the experiment-2 amendment to `CLAUDE.md` (commit
  `d0eb6e8`) landed on the 023 branch by the person's one-commit ruling of
  2026-09-14 and rode into `main` at the merge; never force-pushing outranked
  the commit-straight-to-`main` convention for repo-wide files.

**Supersedes spec 006's goal 2** ("Space plays the selected card", `ROADMAP.md`
under *Control & input polish*): Space draws on the player's turn again. Its
round-end, game-over, menu, prompt and notice roles are unchanged, and the
parenthetical on the spec 001 draw-key bullet above was corrected to match.

No `game.rs` rules change (card effects, scoring, resolution), no AI, economy,
wager-prompt or settlement change, and no `save.rs` change; `game.rs` moved only
at `game_action_from_key`, a new `restart_opponent_pause` (so the opponent's
thinking pause runs from the popup's dismissal rather than through it), and its
own tests. `PROFILE_VERSION` and `SAVE_VERSION` both stay 1. Monochrome by
construction.

## Endgame, victory & what you keep (spec 024)

Beating the Sovereign did almost nothing: the ordinary win popup, a settled
banner, one muted line, a lifetime counter. Losing had a full notice and a reset
behind it; winning had nothing. Ruled with the human on 2026-09-15, on the
recommendations as proposed, around one principle **in the person's words**:
the profile is a global pool of cards and credits between all modes; a new
player has only the basic deck and no credits to buy anything, plays the
campaign to earn, and once they win they keep everything for Quick Play as
well; New Campaign keeps accumulating, with a separate option to reset
everything.

- **1 — The run continues after the win.** A notice once, then the map stays
  open with rematches and the shop. No new reset path. Ending like a loss would
  throw away the deck built to win; New Game Plus with scaled difficulty stays
  deferred.
- **2 — The award is a victory notice with a run summary, plus one lifetime
  record.** A credit bonus buys nothing after completion; a unique card is an
  engine and balance change.
- **3 — The run-summary backlog item folds in.** One summary, two notices.
- **4 — The map keeps a persistent completed marker** (`★  Campaign complete`
  in place of the rim→core axis label) until the map is reset.
- **5 — Quick Play deals the built deck.** **Supersedes spec 022's ruling**
  ("Quick Play deals the standard (premium) deck", quoted and annotated above),
  which had accepted on record that the deck you build only matters in campaign
  matches. That was the one line spec 022 said would be reversible in one line;
  it was.
- **6 — New Campaign keeps cards and credits; Reset Everything is the full
  wipe.** **Supersedes spec 014's "New Campaign = full fresh start"** (quoted
  and annotated above). The 014 argument — keep the early game and the
  depth-gated economy meaningful — no longer holds: since spec 021 a first clear
  pays only progress and every match is even money, so a replay earns no more
  than rematches already can. Going broke becomes the only thing that takes the
  pool, which is the loss condition's intent. This also answers the open
  casual-versus-roguelike identity question (spec 021's "spec E") in the
  **casual** direction, with going broke as the one roguelike bite.
- **Record flag (session's pick).** The first-clear record counts the **first**
  completion only, so a starter-deck first run is never compared with a
  premium-deck replay. A "best completion" that replays could beat was the
  alternative. A profile whose completions are already above zero never gains a
  record — accepted: this is a record for runs from here on.
- **Not retuning the curve for replays** — a replay with a premium deck is
  easier than the spec 022 curve assumed. It is opt-in and deliberate; **no
  constant moved**, and `docs/balance.md` gained a short *Replays* note saying
  so. Reset Everything is what returns a profile to the run those numbers
  describe.

Design tensions resolved during planning:

- **One `Profile::resolve_match` owns the record-then-settle order.** The
  first-clear number is "the run's matches played *including* the completing
  match", captured on the completion edge inside settlement — but the run tally
  is bumped by `record_match`, which `App::tick` called *after* settling, while
  `profile.rs`'s own tests recorded first. A latent app-versus-tests
  disagreement is exactly what spec 021 removed when it moved the completion
  edge into one method, so `App::tick`'s two profile calls became one
  `resolve_match(opponent_id, player_won, player_rounds, opp_rounds) ->
  Option<Settlement>` that derives the `Mode` itself, records, then settles;
  `record_match` and `settle_campaign_match` are now private to `profile.rs`.
  **Settlement's `Option<StakeOutcome>` signature deliberately stayed put**:
  flipping it would have broken the app's call site and nine `assert_eq!`s in
  the settling tests in the *same* task that added the new method, so the
  data-model task could not have built and tested green on its own (the
  constitution's rule that every task ends green). The cost, accepted and
  written down: **the completion edge is evaluated twice** — once inside
  settlement for the completions counter, once in `resolve_match` for the
  victory signal — pinned by a test asserting the two always agree
  (`campaign_completions` increases on exactly the resolutions that return
  `completed_run: true`, and on no others). The rejected alternatives were a
  `+ 1` inside the edge (correct only under the app's order, silently wrong
  under the tests') and a third app-level `record_first_clear` call (an ordering
  rule spanning three calls, unverifiable without an `App` that writes to disk).
- **The completion signal reaches the notice as a transient App flag.** The
  notice is raised on the *acknowledgement* of the game-over popup, one key
  event after the settlement that completed the run, so `Settlement.completed_run`
  is stored as `App::victory_due` and `mem::take`n by the next
  `enter_campaign_map`. **Not persisted, by design** — a flag on disk would be a
  save-format change for a transient. Consequences, all spec'd: a rematch win
  sets nothing; a replayed campaign's completing win sets it again; quitting
  before acknowledging loses the notice but neither the completion nor the
  payout, both already persisted.
- **The choice panel's breathing room was corrected while it was open.**
  `draw_two_choice` became `draw_choice_panel(title, note, labels, selected,
  hint, pulse)` taking N labels, and its row placement moved into a pure
  `choice_rows(note_present) -> (note_row, choice_row, hint_row, height)`:
  without a note the rows are unchanged (title 0, choices 2, hint 4), with a
  note the choices move to row 3 so the note never sits flush against the
  acted-on row — the design brief's *Density and breathing room* rule, which the
  panel had been quietly missing. **This also corrects the spec-021
  discard-a-save confirm**, the only other note-carrying panel. A correction
  inside a screen this spec already changed, not new scope.
- **`player_deck_for` was deleted rather than kept with one branch.** With Quick
  Play dealing the built deck there is one answer for both modes, so the fn, its
  `is_campaign` argument and the `DEFAULT_SIDE_DECK` / `stats::Mode` imports in
  `app.rs` all went; `start_match` has one deal. Keeping a one-line wrapper
  would be indirection the spec doesn't demand (constitution: *Simplicity*). The
  accepted cost: the claim loses its pure-fn test, because `start_match` writes
  the profile and the save and no App test may touch disk. It is pinned
  **structurally** instead — `app.rs` no longer names `DEFAULT_SIDE_DECK` at
  all, so it *cannot* deal the standard deck — plus a driver run with a
  deliberately non-standard built deck (a ten-card +1/−1 deck dealt
  `-1 +1 +1 -1`).
- **The Reset Everything confirm was retitled.** `spec.md` quoted only the tail
  (`… Erases progress, credits & cards.`); under the three-choice panel the old
  head ("New campaign?") would have named the wrong choice, so the title is
  **"Reset everything? Erases progress, credits & cards."** and New Campaign's
  is "New campaign? Resets the map; you keep your cards and credits." A wording
  decision, not a behavior change.
- **A pre-economy profile document shows the entry panel.** A `profile.json`
  without a `credits` key loads with 0 credits (spec 021's deliberate serde
  default, diverging from `SEED_PURSE`), so `differs_from_starter` is true for
  it and Start Campaign offers the three choices; Continue then meets the
  run-over notice as spec 021 intended. Only a document that *is* the starter —
  a serialized fresh profile — opens the map directly, which is what the
  acceptance criterion says. Ruled by the orchestrator on 2026-09-16 as what the
  spec's own predicate specifies, and asserted by the test.
- **The Records *This Run* view was not extended.** It keeps today's lines; the
  new credit counters and the worlds-cleared figure appear on the two notices
  only. `spec.md` was corrected to say so.

No engine change (`game.rs`, `player.rs`, `save.rs`, `economy.rs`, `wager.rs`
untouched; `card.rs` moved only in **doc comments**, by an acceptance criterion
amended for exactly that), no AI change, no balance data moved,
`tests/balance.rs` / `Cargo.toml` / `Cargo.lock` untouched, and no new crate.
The profile gained three additive `#[serde(default)]` fields
(`RunStats::credits_won`, `RunStats::credits_lost`,
`LifetimeStats::first_clear_matches`); `PROFILE_VERSION` and `SAVE_VERSION`
both stay 1, so a pre-024 profile loads with zero counters and no record.
Monochrome by construction.

## Locked cards in the Outfitter (spec 025)

A new player saw seven cards in the Outfitter and nothing saying the other
eight exist, so the depth gate read as missing cards rather than as a promise.
The person hit it in play on 2026-09-16 and ruled the same day. The spec
changes what the shop **shows** and nothing about what it sells, what that
costs, or when a region opens.

- **A2 — locked rows show their price, dimmed.** First ruled **A3** (a
  `locked` word in place of the price), reversed the same day so a player can
  see what they are saving toward. The cost, accepted on record: in a
  monochrome UI a locked row and an unaffordable row look alike, so the
  difference is carried by the group heading and by the cursor (which stops on
  an unaffordable card but never on a locked one).
- **B2 — the cursor skips locked rows.** They are there to be seen, not
  selected, so Enter on a locked card cannot happen — which is what lets the
  buy path stay exactly as it was.
- **C1 — the heading names the region** (`Mid Rim  ·  reach the Mid Rim to
  unlock`), in the map's own labels, rather than naming the planet that opens
  the group. One vocabulary for depth across the map and the shop.
- **Taken ahead of the compact layout**, the other queued shop-adjacent item,
  and **new card types (+5, +6, −5, −6, ±4, ±5) were logged as their own
  roadmap item** rather than folded in: they would move `card.rs`, the tier
  table, the prices and the spec 022 curve, and this spec moves none of those.

Design tensions resolved during planning:

- **Unlocked cards are always a prefix of the list — so the cursor stays a
  plain index.** Groups are drawn in tier order and a group is unlocked iff
  `tier <= deepest_reached`, so the unlocked cards are always the **first `n`**
  cards of the grouped listing (7, 13 or 15). The cursor therefore stayed a
  `usize` wrapping over `0..n`, and a drawn row `i` is cursored iff `i ==
  cursor` (counting cards only). **B2 falls out of the ordering** — no skip
  logic, no per-row lookup. The claim is pinned by a test that the prefix
  equals `available_pool` as a multiset and that every card after it is deeper
  than `deepest_reached`, at all three depths. Rejected: keeping
  `available_pool`'s order for the cursor — it is `ALL_SIDE_CARDS` order (`+4`
  between `+3` and `−1`), not the grouped display order, so the cursor index
  and the drawn row would diverge.
- **The list is one block, centered as a block.** Headings are wider than rows
  (`Mid Rim  ·  reach the Mid Rim to unlock` is 39 columns; a row is 28), so
  centering each line on its own would put headings and rows on different left
  edges and leave the list ragged. Instead the whole list shares one left
  column, `list_left(center_x)`, computed from the widest line it can ever
  draw, with headings three columns in, aligned with the card labels. Title,
  balance and hint stay centered as before. Two consequences, both accepted:
  a row no longer re-centers when its owned count gains a digit (`×9` → `×10`),
  so buying can't nudge a row sideways; and because the block is sized for the
  *widest possible* line (the locked Mid Rim heading at its indent, 42 columns),
  the rows themselves sit left of centre — at 139 columns `list_left` is 48, a
  row is 27–28 columns wide, so the row block centres on column 62 against a
  title centred on 69: **about seven columns left**. It is most visible on a
  profile that has unlocked everything, where no long locked heading is on
  screen to fill the width. Measured at the driver walkthrough and reported at
  the phase pause, where the person attested the screen looked good; left as is.
- **Headings are drawn Normal, locked or not.** The spec dims locked *rows* and
  says nothing about heading emphasis. Drawing every heading Normal keeps the
  lock sentence — the thing that tells locked from unaffordable — at full
  weight while the rows under it recede, and adds no new emphasis level.
- **No defensive tier check on Buy.** `try_purchase` does not look at tier, and
  `handle_input` can only emit `Buy(listing()[cursor])` with `cursor < n`. The
  guarantee is the prefix property plus the cursor clamp, both tested in
  `shop.rs`; a tier check in `app.rs` would be a second rule for a state the
  screen cannot produce — cut per the constitution's *Simplicity*.
- **The density rule was checked — no conflict.** The *acted-on element stands
  apart* rule gives the acted-on line an empty row above and below. The list
  stays compact with one empty row above each heading, matching today's shop
  list: padding a moving cursor row would shift every row below it on each
  keypress, which is exactly the jitter the spec forbids. So the list's only
  air is the heading gaps.

No engine change (`card.rs`, `game.rs`, `player.rs`, `save.rs` untouched), no
AI change, no balance data moved (`economy.rs`'s tier table and prices are as
spec 022 left them; the file gained only `RegionTier::region_name`, the inverse
of `region_tier`, plus its test), no profile-format change,
`tests/balance.rs` / `Cargo.toml` / `Cargo.lock` untouched, and no new crate.
`PROFILE_VERSION` and `SAVE_VERSION` both stay 1. Monochrome by construction:
Normal, Muted and the existing cursor pulse, no new emphasis level.

## Chore: warn on the wager prompt when a loss would end the run (2026-09-17)

Raised by the person on 2026-09-16 while attesting spec 025, and ruled a chore:
one line on an existing screen, no engine, save-format, balance-data or
dependency change, and no new screen or mode.

Stakes are uncapped (spec 021), so a player with a big purse and a full
collection could stake nearly all of it and, on a loss, fall under the cheapest
ante and take the full reset — cards, deck and credits — having seen nothing on
the prompt but `Lose −N`. The prompt now adds **"Lose this and the run is
over."** whenever the chosen stake would leave the balance under the run's
cheapest ante.

- **Additive, not a supersession.** Spec 021 says the prompt shows the
  opponent, its ante floor, the balance and what a win pays; it does not
  enumerate the rows as a closed list, and nothing it rules is reversed here.
  Stakes stay uncapped, the reset behaves exactly as spec 021 left it, and the
  softer *keep-your-cards* restart stays a separate, still-open lever.
- **The predicate is the run's cheapest ante, not the prompt's own floor.**
  `economy::cheapest_floor` is the same value `Profile::is_broke` tests, with
  the same strict `<`, so the warning cannot disagree with the condition that
  actually ends the run. This opponent's ante can sit well above the cheapest
  node still launchable — warning against the prompt's own floor would fire far
  too early on a deep planet. A test with Rix (ante 50) against a reserve of 10
  pins the distinction: it fails if the two are confused, and fails again if the
  comparison is relaxed to `<=`.
- **The snapshot cannot go stale.** The balance and the reserve are read
  together at the one call site, and `settle_campaign_match` touches the
  campaign only on the *win* branch — so a loss leaves `cheapest_floor`
  unchanged and the number the prompt warned on is the number the broke check
  uses.
- **`Strong`, not `Alert`.** `frame.rs` documents Alert as inverse and rationed
  for interrupts; this is a line read while choosing, not an interrupt. The
  person attested the wording and the weight in play before the merge.
- **No spacer of its own.** The constitution's *Density and breathing room*
  rule gives air to the acted-on element, which is the stake row; the warning
  stays compact against the Win/Lose row above it. The box grows by one row on
  its own, since it sizes itself from the row list.

Touched `src/wager.rs` and the one call site in `src/app.rs`, nothing else:
`card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`, `economy.rs`,
`tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are untouched, and
`PROFILE_VERSION` / `SAVE_VERSION` stay 1.
