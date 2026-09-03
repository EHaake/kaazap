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
  draw, so `d`/`s` are the bust-accepting keys now — Space isn't.)
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
grows large. See `ROADMAP.md`.

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
