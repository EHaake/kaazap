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
  **Superseded by spec 026**: the minimum is **89×31 again** and 139 is the *threshold*
  above which the panel draws; `IN_MATCH_MIN_WIDTH` is now `WIDE_LAYOUT_MIN_WIDTH` (same
  value, same derivation from the portrait size). Below 139 the same board draws without
  the panel and a staked match's stake moves onto the status band. The right-margin
  placement, the reserved left margin and the asymmetry are unchanged at 139 and above.
  See *Compact layout below 139 columns (spec 026)* below.
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
  139×31 minimum — since **spec 026** 139 is the wide layout's threshold, and
  below it the stake rides on the board's status band instead, so the ruling
  holds at every width); a bust counter is a cheap additive field if the balance pass
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

## Compact layout below 139 columns (spec 026)

Since spec 016 the game needed a terminal 139 columns wide, because the
opponent-presence panel sits beside the fixed 89-column board with an equal,
empty margin on the other side — large for a general audience, when the game
had played at 89 before. This spec brings the minimum back to **89×31** and
makes 139 a **threshold**: at 139 and wider nothing changes; narrower, the
match board draws without the panel and the stake at risk moves onto the board.
A presentation-only spec: `layout.rs`, `config.rs`, `board.rs`, `portrait.rs`,
`overlay.rs`, `records.rs`, `app.rs`, fit tests and the README. Ruled by the
person on 2026-09-17.

- **Q1 B — the compact board drops the portrait and banter but keeps the
  stake.** Money at risk stays on screen (the wager-warning chore of
  2026-09-17 shows the same priority); banter without a face reads oddly in a
  status band spec 017 deliberately kept mechanical. The header's `Rounds won`
  line already carries what the pips showed, so the pips go too.
- **Q2 A — chosen by width alone, live, no setting.** The resize path already
  rebuilds the board view from the size; a setting would be a second way to
  reach a state width already reaches.
- **Q3 B — the select preview and the map rail keep their portraits at every
  width.** The person wants the portraits to stay part of the game even where
  the match window can't hold one; both surfaces physically fit at 89. (The
  recommended one-rule-everywhere option was declined.)
- **Q4 A with C's rule — the stake goes on the status band's upper row,
  right-aligned; the requirement is visibility for the whole staked match
  without growing the board block.** The planner could move it if the
  walkthrough found a collision; it found none.
- **Q5 A — the threshold stays 139.** No third layout between 114 and 138;
  the wide layout is exactly as spec 016 shipped it.
- **Q6 A — the settled stake shows at game over, on both layouts.** Found by
  the Phase 1 walkthrough (finding F1): at the game-over popup the compact
  band's `Stake ◈ N` was gone, because the match settles on the tick that
  draws the game-over frame and `stake_at_risk()` is already cleared — and the
  wide panel had been blank there since spec 021 for the same reason. Ruled
  the same day: `App::stake_to_show()` returns `stake_at_risk()` or, at
  `GamePhase::GameOver`, the settled amount from the banner; `src/app.rs`
  only, pinned by an App-level test at 89×31 and 139×31. The wide layout's
  one deliberate change.

Design tensions resolved during planning:

- **The panel is an `Option<Rect>`, chosen where the geometry is built.**
  `BoardLayout.opponent_panel` is `Some` iff `cols >= WIDE_LAYOUT_MIN_WIDTH`,
  decided in `BoardLayout::new` — the one place the board geometry is built
  (startup via `App::new`, resize via `App::resize`) — so there is no second
  flag to keep in sync and no width check in `board.rs` beyond `if let
  Some(panel)`. Rejected: a `wide: bool` beside a still-computed rect (two
  fields saying one thing, and the rect would be off-frame at 89), and a
  `Config::is_wide()` helper (a second place to evaluate the rule).
  `main.rs` and `App::resize` are untouched: `Config::fits` reads
  `min_size()` and `resize` already rebuilds `BoardView::new(config)`, so the
  switch across 139 falls out of the existing resize path.
- **`IN_MATCH_MIN_WIDTH` is renamed, and one test helper carries both
  widths.** The name would be false after this spec (the in-match minimum is
  89). It is `WIDE_LAYOUT_MIN_WIDTH` — same value, same derivation, new doc —
  and `Config::min_size()` returns `(BOARD_WIDTH, BOARD_BLOCK_HEIGHT)`. The
  tests that used the old name for "the minimum" switched to
  `Config::fit_sizes()`, a `#[cfg(test)]` helper returning `[89×31, 139×31]`,
  so every fit test loops both without spelling numbers. `wager.rs` is on the
  spec's no-change list: its fit test reads `min_size()` and so measures 89
  without an edit, and the 139 case follows from centering (a content-sized
  box that fits 89 fits any wider frame) — the spec's AC 3 records exactly
  this.
- **The play log's width floor became the wide box's width.** `SCROLL_MIN_W`
  40 → 52 (`139 · 38 / 100`), so the compact play log is the same box as the
  wide one instead of a 40-column one that clips more transcript lines; 52 plus
  margins fits 89. The change is confined to widths below 139 — widths the game
  has never run at — and at 139 and up the clamp never binds, so the wide box
  is untouched. The sizing moved into a pure `scroll_box(config) -> Rect` so
  the equality is a test, not a walkthrough observation. Pre-existing and
  **not** fixed here: round headers with a long opponent name (`Round 1: The
  Magistrate wins — You 20 / The Magistrate 19 (stand)`, 66 columns) already
  clip at 139's 48-column interior; the spec asks only that the compact log
  show what the wide one shows. Not observed in the walkthroughs.
- **The stake line is the panel's string.** `portrait::stake_line(stake)`
  (`Stake ◈ N`) is used by the panel and by the compact arm — one string, so
  the "panel's own form" claim can't drift. Banter is accepted and ignored in
  compact, keeping `BoardView::draw`'s signature unchanged.
- **The density rule was checked — no conflict.** The stake shares the over-20
  alert's row (the alert is 27 characters, `Align::Left`; a six-digit stake is
  14, `Align::Right`, on the 81-column band — ≥ 40 blank cells between them
  by arithmetic; the test pins the alert left of the stake with every cell
  between blank), adds no row, and the board block keeps its fixed 31-row height. The
  acted-on element on the board (the cursored hand card) is unchanged.

Attested by driver walkthroughs at both sizes. At 89×31: every screen (menu,
How to Play, opponent select with the preview beside the list, map clear of the
rail, Outfitter, deck builder with the hint whole and centered, records, play
log as a 52-column box, wager prompt) on frame with nothing clipped; a staked
match showed `Stake ◈ 30` from the first frame through the over-20 alert, a
round popup and the game-over popup; Quick Play showed no stake line; the
too-small screen quoted `Need at least 89 x 31` at 60×20. At 139×31: spec
016's panel with portrait, banter, pips and stake, no band stake, and the stake
on the panel at game over. Resizing 139 → 138 → 139 mid-match kept the match,
the moved cursor and the open `?` overlay and toggled the panel.

No engine, AI, economy, wager, save-format or balance change: `main.rs`,
`card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`, `economy.rs`,
`wager.rs`, `campaign.rs`, `campaign_map.rs`, `tests/balance.rs`, `Cargo.toml`
and `Cargo.lock` are untouched; `portrait.rs` gained only `stake_line`, its
call and one doc line; `PROFILE_VERSION` and `SAVE_VERSION` both stay 1; no
new crate. Monochrome by construction: the stake line is `Strong`, as on the
panel, and no new emphasis level.

## Animation pass (spec 027)

Until this spec exactly one thing moved on a still screen: the selection
pulse. A dealt card, the opponent's whole move, a changed total and the
outcome popup all landed on the same frame as the state change behind them,
so a fast round read as a jump cut. This spec adds sparse, one-shot
transitions on the match board — emphasis over a short, fixed time on an
element already at its final position — and an Animations row in Settings
that turns them off. Drawing only: a new `motion.rs` and changes to
`board.rs`, `app.rs`, `settings.rs`, `lib.rs`, one doc line in `frame.rs`,
one test line in `audio.rs`, the README and the brief. Ruled by the person on
2026-09-18, with two revisions at the phase pauses (Revision 1 on 2026-09-18,
Revision 2 on 2026-09-19).

- **Q1 a–e in, f out — a dealer card arriving, a played card arriving, a
  total changing, the popup beat and the thinking indicator; no stake
  flash.** Exactly the roadmap's list plus the thinking indicator; the
  presence panel's stake line, pips and banter are untouched.
- **Q2 A — arrival is emphasis only.** A face-down beat hides information
  and costs more for a subtlety. (Superseded for dealer cards by Q8 at
  Revision 1, then restored for every card by Q11 at Revision 2 — see
  below.)
- **Q3 A — the selection pulse keeps breathing during a transition**, and
  `design/brief.md`'s Motion section is amended by one sentence: the
  one-thing-moves rule counts *continuous* motion, and a one-shot emphasis
  transition may run alongside the pulse because it ends on its own within a
  beat and never breathes. Holding the pulse (Q3 B) was declined.
- **Q4 A — an Animations On/Off row in Settings.** Reduced motion is a real
  need and the overlay was written to grow. Saved with the volumes; a
  settings file **without the key reads On**; a file with it Off starts Off;
  the toggle takes effect on the next frame, mid-match included. Off means
  the board draws settled from the first frame — no heavy border, no ghost,
  no Strong, popup on the resolving frame, static thinking line — with one
  standing exception (Q7). The pulse and the map's starfield are not
  governed by it.
- **Q5 A — portraits stay static.** Spec 016's "light portrait animation"
  deferral (swapping frames on the pulse) is closed at this merge rather
  than reopened; the brief's portrait amendment ("no color, no animation",
  spec 016 above) stands unchanged and the presence panel is untouched.
- **Q6 A — the match board only.** Menus, the Outfitter, the deck builder,
  the map, the wager prompt, the shop and the records screens do not change.
- **Q7 — the Score rests at Normal weight** (a session ruling at planning
  under the person's delegation — plan *Open questions* 1 — flagged in the
  spec-conformance summary). `Score: N` had been drawn bold at all times, so
  a bold-for-a-beat transition on it would have been invisible, and the only
  stronger level is the rationed inverse. The options were to rest the Score
  Normal so the beat shows, or keep it bold and drop the total-change
  transition; the first was taken because ruling c is the person's. The
  consequence is the one deviation from "the Off board is the pre-spec board
  frame for frame": with Animations Off the Score is no longer bold.
- **The standing constraint**, stated by the person with the rulings:
  transitions must be noticeable enough to add to the game and quick enough
  that they never get in the way of player actions. No key is ever delayed
  or deferred; the opponent's pause is still `OPPONENT_THINKING_TIME_MS`;
  the phase machine, saves, banter and audio fire exactly when they did.

**Revision 1** (the person, 2026-09-18, at the Phase 1 pause). With Phase 1
built, the popup beat and the thinking dots read well but the bold-only hit
and card-play arrivals did not register at all: the driver confirmed the bold
attribute reached the terminal on the right frames, and the cause was that
bold on a thin box-drawn card is barely distinguishable from normal, while
the hand cursor's heavy breathing border already owned the strongest look on
the board. The remedy was shape, not weight.

- **Q8 — a dealer card lands heavy** (and, until Revision 2's Q11 withdrew
  it, face down for a flip beat). The dealt card draws with the heavy border
  and Strong for the arrival beat, then settles to today's single border,
  Normal.
- **Q9 — a played card lands heavy, and its hand slot shows a source
  ghost.** No card flies from the hand to the board: at the loop's 50 ms
  frame and whole-cell positions a flight would read as a stutter. The
  ghost outline in the emptied slot (single border, empty face, Normal, no
  number key) plus the heavy landing gives the same "it came from there" cue
  without motion — the person's choice from the session's options. Both
  sides: the opponent's hidden `?` slot empties into the same ghost.
- **Q10 — the Score transition stays as built** (Strong for the arrival
  beat, resting Normal per Q7). Not raised by the person; noted so the
  revision's scope is explicit. If it proves as invisible as the card bold,
  it is a follow-up.
- Not in the revision: the selection pulse's own vocabulary (spec 002 — a
  heavy/thin border alternation was floated as more visible than
  bold/normal; the person has not ruled), the thinking indicator and the
  popup beat, which the person judged good.

**Revision 2** (the person, 2026-09-19, at the Phase 1b pause). Played with
Revision 1 built, the person judged the heavy landings and the source ghost
good and the dealer card's `?` flip as not making sense: "just remove the
initial `?` and call it good."

- **Q11 — the flip is withdrawn.** A dealt card lands heavy with its value
  visible from its first frame; the flip beat, its constant and its bounds
  are gone (`grep FLIP src/` is empty). A card's content never changes after
  it is drawn, on either side; Q2 A stands for every card. Everything else in
  Revision 1 (Q9, Q10) stands.

**Process note.** Both revisions were written into `spec.md` in the
implementation session at the person's ruling ("revise the spec now so that
we finish it here") — a stated deviation from the constitution's rule that
spec conversations happen in a spec session of their own. Each revision had
its own planner amendment, sign-off at the top tier, a phase (1b) with its
own review, and a walkthrough.

Design tensions resolved during planning:

- **One struct, observed in `tick`, read by `draw`; `None` is the settled
  draw.** `BoardMotion` (`src/motion.rs`, pure logic and tests, no rendering
  import, like `banter.rs`) is a field on `App`. `App::tick` calls
  `observe(&game_state, dt)` while the screen is `InGame` and resets it to
  `default()` otherwise — that one `match` is the spec's "discarded when the
  board leaves the screen" and its "first frame drawn settled" (the first
  observation after entering a match seeds silently and starts nothing).
  `App::draw` passes `settings.animations.then_some(&motion)` to
  `BoardView::draw`, so `None` — no motion at all — is the settled draw and
  the Off state is the same code path as "nothing is arriving". Rejected: a
  per-element `Instant` map (untestable without sleeping), a `GamePhase`
  extension (the engine must not know), and an `Animations` flag inside
  `BoardMotion` (the board would have two ways to be settled). Clocks count
  down in the tick's `dt`, so every test is a sequence of `observe` calls
  with chosen durations, no sleeping.
- **A row that shrinks is a clear, never a change.** `setup_next_round` and
  `new_game` empty both rows and drop the total to 0 — a "change" by value,
  but the spec wants a rematch's first frame settled and the eye guided to
  what *arrived*. Rule in `observe`: if a side's dealer or played count fell
  since the last observation, discard that side's arrivals (the source ghost
  included) and start nothing for it; otherwise start an arrival per new
  index and a Score arrival if the total differs. A second change to the
  same Score restarts its beat (one figure, one clock); a card index cannot
  arrive twice without a clear. The `g` rematch's settled board falls out of
  this rule rather than the seed.
- **Settings: three rows, one `adjust`, no disk in tests.**
  `SettingsAction::Louder`/`Quieter` became **`Right`/`Left`** — on a
  volume row louder/quieter, on the Animations row a toggle; the old names
  were about to be wrong — and the value change moved out of
  `App::handle_settings_input` into **`Settings::adjust(row, right)`**, which
  owns the volume step (`VOLUME_STEP`, clamped to 0..=1) and the toggle, so
  both are unit-tested without the `save()` that writes the real config
  file; `App` still calls `set_settings`, `save` and the cue as before. The
  rows are a `const ROWS: [SettingRow; 3]`; the hint reads `←/→ change` (was
  `←/→ volume`). The Animations row is **padded to the volume rows' width**
  (T004a, the person at the Phase 2 pause, after the reviewer noted the
  shorter centred row shifted its marker and label ~6 columns right): the
  row's marker and label now sit in the volume rows' columns, with the value
  one space after the label.
- **Revision 1: the arrivals change shape on the clocks Phase 1 already
  runs.** The heavy landing costs no new clock: a card draws
  `BorderWeight::Heavy` + `Strong` while its arrival is counting down and its
  resting border after. **The source ghost is one more `Elem`**,
  `Hand(side, slot)`, started when a hand slot went `Some → None` since the
  last observation, on the arrival beat, and dropped by the same shrink rule
  as the side's cards; the board draws it by reusing `CardView` with an
  empty face rather than a bespoke box, so its rect and interior blanking are
  the card's. **The heavy landing is the recorded exception to the brief's
  "distinct weights, distinct meanings" rule** (spec 003): Heavy was
  reserved for cursor selection, and for one arrival beat it now also marks
  a card landing. `frame.rs`'s `BorderWeight` doc names the exception (the
  file's one comment-only touch); `card.rs`'s `// Heavy marks cursor
  selection (T007)` comment is **knowingly left stale**, because `card.rs`
  is on the spec's no-change list (AC 12) — noted here so the next touch of
  that file fixes it. The brief's Motion section itself carries only the
  Q3 A sentence. **The flip was built as a read of the arrival's countdown**
  — `?` while the remaining time was still inside the flip window, no second
  clock, no `face_down` flag — and **withdrawn** by Q11 after the person saw
  it: the constant, the guard and the `?` branch are gone.
- **The beats live in `lib.rs`, the bounds in a test.** As shipped, with no
  tuning at the pauses: `ARRIVAL_BEAT_MS = 600`, `POPUP_BEAT_MS = 800`,
  `THINKING_STEP_MS = 300`. A motion test pins the spec's bounds
  (`SELECTION_PULSE_MS ≤ arrival ≤ 1000`, `arrival ≤ popup ≤ 1000`,
  `2 · step ≤ OPPONENT_THINKING_TIME_MS`), so a retune outside them fails
  the build.

Attested by driver walkthroughs at 89×31 and 139×31 after Phases 1 and 1b and
at 89×31 after Phase 2, with the
person's own play at the Phase 1, 1b and 2 pauses. Phase 1 (bold-only): a
dealt card bold on its frame and settled by ~1 s, the Score bold only when it
changed (a dealt 0 left it plain), `Rounds won` never bold, the cursor still
breathing; the dots stepping through the pause and the plain line once the
opponent acted; a round resolved with no popup at two 0.4 s samples and the
popup by 0.8 s; `n` on the resolving frame started the next round with no
popup ever drawn; the game-over popup absent at 0.1 s and present at 1.1 s;
Continue on a save left at the popup drew the popup on the first frame with
nothing bold, and Continue mid-match drew settled; at 89 the dots sat on the
band's lower row with the alert/stake row above. Phase 1b (heavy landings): a
hit landed heavy, still heavy at 0.35 s, settled to the thin border by
0.75 s; a play landed heavy in the grid while the emptied hand slot kept a
plain outline and lost its number key, then settled to the double border
with the slot blank; the cursor moved to the next card and kept breathing;
the opponent's card landed the same way; the presence panel unchanged at 139.
Phase 2 (the setting): Settings showed Music, Sound FX, Animations On; `→`
flipped it Off and the file on disk gained `"animations": false`; a hit with
it Off drew the thin card with its value, a plain `Opponent's Turn`, a
double-bordered play with the slot blank and the popup on the resolving
frame; `→` again read On and the file said `true`; a file with the key
deleted read On.

No engine, AI, economy, wager, save-format or balance change: `game.rs`,
`main.rs`, `render.rs`, `layout.rs`, `portrait.rs`, `card.rs`, `player.rs`,
`save.rs`, `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`,
`campaign_map.rs`, `opponent.rs`, `tests/balance.rs`, `Cargo.toml` and
`Cargo.lock` are untouched; `frame.rs` changed one doc comment; `audio.rs`
changed one test's struct literals for the new field; `PROFILE_VERSION` and
`SAVE_VERSION` both stay 1; no new crate; the build has no warnings. No color
path: the only attributes emitted are the four existing emphasis levels and
the existing border weights.

## Chore: a path seam for the profile, match save and settings (2026-09-19)

Closes the "known gap, deferred" noted in the 2026-09-13 chore entry above and
the backlog's *A path-injection seam for the profile and save locations*.
`Profile::path`, the match-save path and `Settings::config_path` each resolved
`ProjectDirs` for themselves, with no override — so no `App`-level flow could
be tested without reading and writing the real data folder, and every driver
session had to back up the human's profile first.

- **One module owns the locations.** A new `paths.rs` exposes `data_dir()` and
  `config_dir()`; the three call sites go through it and `ProjectDirs` is
  imported nowhere else in the crate.
- **The root resolves once, in three steps:** an explicit `paths::set_root`,
  then the `KAAZAP_DATA_DIR` environment variable, then the platform
  directories. With neither override present the paths are exactly where they
  have always been — the same `ProjectDirs::from("", "", "kaazap")`, the same
  `data_dir()`/`config_dir()`, the same filenames and `saves/` subfolder.
  Nothing moves, and no existing file is migrated.
- **An override collapses the two directories into one.** `data_dir()` and
  `config_dir()` both return the override root; `profile.json`,
  `saves/savegame.json` and `settings.json` don't collide, and one temp
  directory holding all three is the point. Only the platform case keeps them
  distinct, as the OS wants.
- **`KAAZAP_DATA_DIR` is the lever a driver uses.** `set_root` is an in-process
  call, so a session driving the built binary can't reach it; the environment
  variable can, and it applies as long as nothing in-process has resolved the
  root first. This
  supersedes the roadmap line saying every driver session must back up the
  real profile first — pointing the run at a scratch directory is now enough.
- **`set_root` is not `cfg(test)`-gated**, since integration tests call it from
  outside the crate. It returns whether the override took: once the root is
  resolved it stays put, so a late call is a truthful `false` rather than a
  silent relocation mid-run.
- **The seam is added, not yet used by the existing tests.** Seven `app.rs`
  unit tests construct an `App`, which reads the real profile, settings and
  save; taking them off the real folder changes those tests' behaviour and is
  its own follow-up. The seam is what that follow-up needed to exist.
- **Why a chore and not a spec.** One new module and three one-line call-site
  changes, with no engine, AI, save-format, balance-data or dependency change
  and no new screen or mode.

## Crash & data safety (spec 028)

An audit found three ways kaazap failed badly, all confirmed in the code. The
terminal was restored in exactly one place, reachable only by `break
'gameloop` on `q`, so any panic or input error dropped the player into a shell
with no cursor and no echo. All three writers replaced a file in place with
`fs::write`, so an interrupted write destroyed the progress it was meant to
preserve. And `Profile::load` collapsed missing, unreadable, malformed and
wrong-version into `unwrap_or_default()`, after which the first `save()` wrote
a starter profile over the campaign it could not read — silently. This is a
repair spec: one new module (`src/crash.rs`), one shared function
(`paths::write_whole`), one modal variant, one enum and one struct on
`Profile`. No new screen, no new phase, no new crate, no engine, AI, economy
or balance change. Ruled by the person on 2026-09-19, with Q6 added at plan
sign-off the same day.

- **Q1 a — an unreadable profile is moved aside under a dated name, and the
  notice says where it went.** The alternatives were to leave it and refuse to
  save (nothing persists, and the player has to know that), or to leave it and
  let the next save overwrite it after a warning. Setting it aside is the only
  one where a recoverable campaign is actually recoverable, and it is the same
  instinct as the *archive the last run at reset* backlog item — which stays
  separate and still open.
- **Q2 a — the notice is a modal on the start menu, dismissed with a key.** A
  persistent line, and a banner that clears on the first navigation, were both
  offered. Losing a run deserves a stop, not a line the player scrolls past.
- **Q3 b — the notice says why.** "Couldn't be read" and "saved by a different
  version of kaazap" are separate messages, because the version case is the
  only one where the player can act, by finding the build that wrote it. The
  two internal cases the player *can't* act on differently — an I/O or
  permission failure and a malformed document — are deliberately one message
  (`ProfileProblem::Unreadable`).
- **Q4 a — a bad match save gets a notice too, but no file is kept.** Before
  this spec **Continue** simply disappeared with no explanation. The accepted
  consequence: the unreadable save is removed so the notice doesn't repeat
  every launch, so that one match is gone for good. A match is not a run.
- **Q5 a — the crash report is one kaazap line plus the panic's message and
  location**, rather than raw panic output alone, so the player knows it was
  kaazap and a bug report has something in it.
- **Q6 a — `q` and `m` still act while the notice is up** (the person,
  2026-09-19, at plan sign-off). The sign-off raised this as blocking rather
  than deciding it: the plan kept both keys live against the spec's original
  "no other key does anything while it is up". Both are handled before the
  modal chain — `q` in the game loop itself, before `App` sees the key; `m`
  ahead of modal routing since spec 004 — so suppressing them would have made
  this the one modal in the game you cannot quit or mute under, and would have
  meant the game loop asking `App` which modal is open. **`spec.md` was
  amended** in two places and carries a dated Q6 section naming them as the
  standing exception, so the acceptance criterion and the design now agree.
  Enter, Space and Esc dismiss; nothing else acts.

Design calls made during planning:

- **A panic hook that records, and a guard that restores and prints** (§1).
  Two mechanisms, neither doing its usual job. The **guard** (`TerminalGuard`
  in `main.rs`) is the only way to reach the restore on the non-panic paths —
  `q`, and an `Err` from `event::poll`/`event::read` leaving `main` through
  `?` — and its `Drop` also runs while a panic unwinds, so the restore has
  exactly one home and the three teardown lines left the end of `main`. The
  **hook** prints nothing: the default hook prints *before* unwinding reaches
  any guard, i.e. while the alternate screen is still up, and that output is
  thrown away when we leave it. So the hook records the first panic's message
  and location into a `OnceLock` in `crash.rs`, and the guard's `Drop` prints
  the report **after** the restore. Consequences, each deliberate: **first
  panic wins**, so a render-thread panic (the cause) is what gets reported and
  the `join().unwrap()` that surfaces it on the main thread (the symptom) is a
  no-op against an already-set `OnceLock`, and the process still exits 101;
  the hook is installed **inside `TerminalGuard::enter`, before raw mode**, so
  no window exists where the default hook prints onto the alternate screen
  with nothing recorded to reprint; `restore_terminal` swallows every error
  (`let _ = …`) rather than unwrapping, because a panic inside a `Drop` during
  unwinding aborts the process, and because restoring twice must change
  nothing. The whole design depends on `Drop` running: `Cargo.toml` sets no
  `panic = "abort"` in any profile (checked at planning and re-checked at
  T002), so a panic unwinds in release as well as debug. If that ever changes,
  the crash path has to move into the hook.
  - **The guard joins the render thread before restoring**, and that was not
    the first design. The plan shipped with one flag check at the top of the
    render loop (`if crash::report().is_some() { break; }`) and named a
    deterministic fallback in case it wasn't enough. The Phase 1 walkthrough
    found `KAAZAP_CRASH_AT=tick` and `=draw` garbled the crash report **6 runs
    out of 6** — the forced first render before the loop never reaches the
    check at all — so the fallback was taken (T002a): the guard carries
    `render: Option<JoinHandle<()>>` and joins it at the top of its `Drop`.
    The re-walkthrough was clean in all 33 runs.
  - **The join is only bounded because `render_tx` is declared *after* the
    guard.** Locals drop in reverse declaration order, so the channel sender
    goes first, the render thread's `recv` fails, and the thread ends — then
    the join returns. Hoisting the channel above the guard would deadlock
    every ending. Two comments in `main.rs` say so; it cannot be unit-tested,
    so it is checked at review and in the walkthrough.
  - **`main` keeps its own `join().unwrap()` on the `q` path**, taking the
    handle back out of the guard. That `unwrap`, not the guard's ignored
    `Err`, is what makes a panicked render thread exit 101.
  - **Scope: the endings `spec.md` enumerates.** `q`, an input error, and a
    panic on either thread. A `SIGTERM` or `kill -9` still leaves the terminal
    unrestored; a signal handler is not in this spec. That is a boundary, not
    a gap — but nothing should describe the guarantee as "however kaazap
    ends".
  - Rejected: `catch_unwind` around the loop (the spec's own non-goal — no
    recovery — and more code for the same teardown); printing from the hook
    (the report lands on the alternate screen); a guard owning the channel as
    well as the handle (the sender has to drop *before* the join, which
    reverse drop order already gives for free).
- **`KAAZAP_CRASH_AT` ships in the binary, and is deliberately undocumented**
  (§2). Four acceptance criteria are about what a crash looks like, and a
  shipped binary cannot otherwise panic on purpose. `main` reads the variable
  **once** before the loop; `key`, `tick`, `draw` and `render` panic at those
  points, and `input` returns `Err` from `main` at the same place
  `event::poll`'s `?` would — the no-panic path of AC 4. Nothing in the game
  sets it, and no key, file or menu reaches it. It matches the existing
  `KAAZAP_DATA_DIR` idiom, and it is **not** in `Readme.md`: it is a review
  and walkthrough seam, not a player feature.
- **One `write_whole` in `paths.rs`, a fixed temp name, and no `fsync`** (§3).
  `pub(crate) fn write_whole(path: &Path, contents: &str) -> bool` writes to
  `path.with_extension("tmp")` and `fs::rename`s it over the target. Stated no
  more strongly than it is true: the bytes are complete before anything
  replaces the file the game reads, so an interrupted write leaves the
  previous file byte-for-byte unchanged and never leaves a half-written file
  *under the real name*. On POSIX the replacement is atomic; on Windows
  `std::fs::rename` prefers `FileRenameInfoEx` and falls back to `MoveFileEx`,
  which Microsoft does not guarantee atomic in every case — safe either way
  for this purpose, with one Windows-only consequence: if another process
  holds the target open the rename fails and the save is a silent no-op, which
  is exactly the best-effort contract all three callers already had. The temp
  name is **fixed, not unique**, so repeated interrupted writes leave one
  piece of debris rather than a growing pile, and it is never `*.json`, so no
  loader looks at it. It lives in `paths.rs` — "where kaazap's files live"
  becomes "…and how they are written" — rather than in a module whose whole
  content would be one function, and it takes an explicit `&Path`, so its own
  unit tests need no data root and can't race the `OnceLock`.
  - **No `fsync`, and here is the reason, so a later power-cut question finds
    it.** `File::create` + `write_all` + `sync_all` would additionally survive
    a *machine* crash, but Rust's `sync_all` is `F_FULLFSYNC` on macOS — a
    full device flush — and the match save is written on **every state
    change**, i.e. effectively every key press. The acceptance criteria are
    about a write that fails partway, which rename alone covers completely;
    the power cut appears only in the spec's narrative. A durable version
    would also have to fsync the *directory*. This is one line to change if
    the person ever wants to pay for it; `write_whole`'s doc comment names
    `fsync` and cites plan §Design tension 3 so the trade is findable from the
    code.
  - **`write_whole` is a convention, not an enforced rule.** All three writers
    go through it, and nothing in the test suite would catch a fourth writer
    calling `fs::write` directly — or one built from `File::create` +
    `write_all`, or `serde_json::to_writer`. The greps in T009 are a one-shot
    check at merge, not a regression guard. **The next writer of a kaazap file
    goes through `paths::write_whole`.**
  - **Two kaazap processes writing at once is the one case the fixed temp name
    does not cover.** Both would write the same `<stem>.tmp` and an
    interleaving could splice them before either renames. This was bought
    deliberately: unique temp names would trade a rare two-instance corruption
    for certain debris accumulation across crashes. A known limit, not an
    oversight.
  - **Write-then-rename changes two things `fs::write` did not.** The file
    gets fresh permission bits at the default umask rather than keeping the
    ones it had, and a symlink at the target is replaced rather than written
    through. Neither matters for a game's saves; both are now inherited by
    every future writer.
- **The save suspension is a process-scoped flag, not a field** (§4). When an
  unreadable profile cannot be moved aside, nothing may overwrite it for the
  rest of the launch — and "the rest of that launch" *is* the process, so
  `static SAVES_SUSPENDED: AtomicBool` in `profile.rs`, set by `load` when the
  move fails and read as the first line of `save`. With twelve `save()` call
  sites the guard has to live behind `save` itself. Rejected: a
  `#[serde(skip)]` field on `Profile`, because `reset_to_starter` does `*self
  = Profile::default()`, which would silently clear the flag and let **Reset
  Everything** write over the very file the flag exists to protect; and a flag
  on `App`, because twelve call sites and every future one would have to
  remember it. The static is the same shape as `paths::ROOT`.
- **A dated name, hand-rolled, in UTC** (§6). `profile-YYYYMMDD-HHMMSS.json`,
  beside the file it replaces. The spec wants a name the player can find, and
  the toolbox has no date crate and is not getting one, so `profile.rs` gained
  about twenty pure lines: seconds since the epoch split into a day count and
  a time of day, plus the standard civil-from-days conversion (verified
  longhand against all three test vectors at review, including a pre-epoch
  one). **UTC**, because local time needs the platform's zone database, which
  needs a crate — so a player well east or west of UTC may see a name a day
  off their local clock. If the name is taken — two bad launches in the same
  second, or a hand-made copy — `-2` … `-9` follow; past that the move is not
  attempted, which lands in the suspended branch, the safe end. The caller
  takes the first name whose path doesn't exist, because `fs::rename` would
  otherwise *replace* an existing set-aside file. The integration test
  produced `profile-20260920-023255{,-2,-3}.json` with all three collisions in
  one second, so the numbered branch is what actually ran. Rejected:
  `profile-<unix-seconds>.json` (no date math, but unreadable to a player) and
  the file's own mtime (the same formatting problem).
- **Two keys still act under the notice, and the `?` ordering is now a
  comment** (§8). See Q6 above for the ruling. The plan's own claim — that the
  set is just `q` and `m` — was treated as a claim, not a reading: the Phase 4
  review **enumerated** the keys from the top of `handle_key` down and
  confirmed it (`q` in `main.rs` before `App`; `m` ahead of the modal ladder;
  Enter/Space/Esc in the arm; everything else a no-op, including `?`, `L`,
  `Q`, `M`, the arrows, the digits and Ctrl+P/N/B/F, which arrive as arrows).
  One ordering is load-bearing and invisible: **`?` is handled inside the
  `else` branch that runs only when no modal is open**, so it cannot reach the
  notice — but if it ever moved ahead of the modal chain, opening and closing
  help would set `self.modal = None` and the notice would be gone for good,
  with nothing to bring it back. That constraint was carried out of the review
  and into the code as a comment at the `?` site (T008a), so it outlives the
  review that found it. For the record, `m` under the notice **mutes
  silently** — it is not a no-op, it simply has nothing to draw.
- **The notice is a `Modal` carrying its lines** (§7). `Modal::DataNotice(
  Vec<String>)`, built once in `App::new` by a pure `data_notice_lines(profile,
  save_unreadable) -> Option<Vec<String>>` whose `None` *is* the "raise no
  modal" answer, so there is no separate predicate. The other notices rebuild
  their lines each draw because their content is live run state; this content
  is fixed at launch and its inputs are gone by the second frame, so it is
  carried and `App` gains no field. Dismissal reuses the existing
  `notice_dismissed` and `draw_notice`, so there is no new drawing code and no
  `resize` arm. One line of the drafted wording was **replaced at T008**
  on a Phase 3 review finding: "nothing from this session is kept" was
  literally false, because only `Profile::save` no-ops under suspension while
  `save::save` and `Settings::save` still write. Both documents carried the
  claim — `plan.md`'s §Design 7 code block, and `spec.md`'s *Key behavior*
  line "nothing from that session persists, and the notice says that too" —
  and the notice's shipped wording is the correction for both: "kaazap won't
  save over it, so the campaign you play this session won't be kept." The
  match save and the settings file still persist under profile suspension.
  `plan.md` was amended to match; `spec.md` was left as the person wrote it,
  and no acceptance criterion repeats the claim.

Two things a future reader should know that have no other home:

- **`payload_as_str` pins the crate to Rust ≥ 1.91.** The panic hook uses
  `PanicHookInfo::payload_as_str`, stable from 1.91, and nothing in the tree
  says so — AC 17 forbids adding `rust-version` to `Cargo.toml`. The toolchain
  in use is 1.91.1. The two-arm `&str` / `String` downcast is the equivalent
  with no version floor if that ever bites.
- **`eprintln!` inside `TerminalGuard::drop` can panic on a closed stderr**
  (`kaazap 2>&1 | head -1`), and a panic inside a `Drop` during unwinding
  aborts the process. `restore_terminal` is unwrap-free by design; this is the
  one unguarded panic site left on that path. No acceptance criterion covers
  it, so it is recorded here rather than fixed under this spec.

Attested by driver walkthroughs at 89×31 with `KAAZAP_DATA_DIR` pointed at a
scratch directory throughout — the real profile, save and settings were never
in play. **Phase 1** (T002, then T002a after the fix): 33 runs — 3 quits plus
6 each of `=tick`, `=draw`, `=input`, `=key` and `=render` — all clean, the
cursor visible every time, exit codes 0 for the quit, 101 for each panic and 1
for the input error, with the `input` run printing `Error: kaazap couldn't
read the terminal` and **no panic text**. **Phase 4** (T008), ten runs: (a) a
clean launch showed no notice and created nothing; (b) a malformed profile
raised the notice naming the dated file, the kept bytes were identical, and
playing a **whole match** afterwards left the kept file md5-identical with
exactly one set-aside file and a fresh `profile.json`; (c) a `"version": 2`
profile gave the wrong-version wording; (d) a malformed match save gave its
line, no **Continue**, the file removed, and a **silent next launch**; (e)
both damaged at once gave **one** notice, box 73 wide, nothing clipped, and
the move-failed case gave the widest box — 87 columns with one column of
margin each side, exactly the review's arithmetic — with the unreadable file
still there unchanged; (f) `?`, `L`, `m`, an arrow, `3` and `x` left the
screen byte-identical with the menu selection unmoved, Enter, Space and Esc
each dismissed to the menu with the selection where it starts, and `q` quit
with exit 0 and a clean terminal.

No engine, AI, economy, wager, balance-data or dependency change: `card.rs`,
`game.rs`, `player.rs`, `opponent.rs`, `economy.rs`, `campaign.rs`,
`wager.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are untouched;
`PROFILE_VERSION` and `SAVE_VERSION` both stay 1; the version comparisons and
every `#[serde(default)]` are unchanged — `Profile::from_json` became a
`#[cfg(test)]` wrapper over a new `classify(text) -> Result<Self,
ProfileProblem>` precisely so every existing profile test stands unedited and
the gate's behaviour is visibly the same; no new crate; the build has no
warnings, the same count as `main`. Monochrome by construction: the notice
reuses `draw_notice`'s existing emphasis levels and adds none.

## Tournament rounds (spec 029)

A single match win used to defeat a campaign opponent: ten opponents, ten
matches, and one lucky match could take a world. Spec 029 puts a **series**
between the match and the opponent, a **venue** between the map and the match,
and a **lock** that makes the series the only campaign match playable while it
runs. It adds one `Screen` (`Venue`), one module (`src/venue.rs`), one layout
struct (`VenueLayout`), two persisted fields (`CampaignRun::series`,
`NodeRef::settled`), both `#[serde(default)]` with no version bump, and — by
ruling R9 — sixteen drawings of the planets in `assets/planets/`, with a second
bounded exception for them in `design/brief.md`. No engine, AI, save-format,
economy-constant, balance-data or dependency change. Ruled by the person on
2026-09-20 (A1–I1, then J1–Q on the consequences of D2 and E2), amended three
times after they walked the venue (R1–R3 on 2026-09-21, R4–R6 the same day, R7
and R8 on 2026-09-22), and extended at the merge pause the same day (R9), which
brought the art itself into the spec.

### The rulings (the person, 2026-09-20)

- **A1 — the new unit is a "series."** "Round" is already the within-match
  unit, and "tournament round" would collide with it.
- **B1 — each match is staked separately**, exactly as spec 021 has it. Keeps
  escrow, settlement and the broke check intact, and makes the between-match
  shop visit worth something: a win pays before the next match starts.
- **C1 — losing a series costs only the stakes already lost.** The score resets
  to 0–0, the opponent stays un-beaten, nothing further is taken.
- **D2 — a venue screen between matches**, rather than returning to the map.
  The person's reason: it is a place, and an opportunity for art that raises
  the immersion of the campaign.
- **E2 — the player is locked into a series once it starts.** No abandon
  action; quitting to the menu leaves it in progress.
- **F1 — rematches stay single matches** (the grinding path is meant to be
  low-friction).
- **G1 — best of five for the final opponent only**, not for the Core
  generally.
- **H1 — no new records or statistics.** Match counters count matches;
  campaign completion counts once. The lifetime first-clear record stops being
  comparable with anything set before this spec (the campaign takes roughly
  2.4× as many matches) — accepted, not reset.
- **I1 — the series is visible on the map and in the match**, which with E2 and
  P1 resolves as: the map shows the series **length** before a launch (the
  player is never on the map while a series is live), and the venue and the
  match board show the **score**. Confirmed by the person.
- **J1 — the map launches to the venue at 0–0**, staking nothing, and every
  match of the series starts from the venue. **K1 — the venue offers play, the
  shop, the collection and quit**, the shop and collection returning to the
  venue. **L1 — rematches keep today's flow**: map → wager → match → map.
- **M1 — the art region is reserved now and filled with a plain placeholder;
  authoring it is its own spec.** Amended **twice**. The same day: the
  opponent's portrait sits **beside** the region rather than inside it, because
  the art belongs to the planet and a planet may later hold more than one
  opponent, so the two are laid out as separate elements now and neither later
  spec reworks the layout. Then on 2026-09-21 (R3, below): the art region
  becomes the screen's dominant element, with the text above and below it.
  **Its placeholder half was superseded on 2026-09-22 by R9** (below): the
  region now holds the planet's art, and the art was authored outside the
  codebase from this spec's brief rather than in a spec of its own.
- **N1 — the art region draws at 139 columns and wider only.** **Superseded
  2026-09-21 by R3**: it draws at every width.
- **O1 — while locked, broke is judged against the locked opponent's ante
  floor**, and the shop reserves that floor. The cheapest ante on the map may
  be on a planet the player is not allowed to play.
- **P1 — entering the campaign mid-series goes straight to the venue**; the map
  is not reachable while locked.
- **Q — per-planet music is out of scope**, its own spec (on `ROADMAP.md`).

### The first amendment (the person, 2026-09-21, after walking the venue)

- **R1 — the series length reads `Best of 3` / `Best of 5`**, the same words
  the map's planet detail uses, so the player meets one phrase for one idea.
  "first to 2" had been a session choice, never a ruling. (When R1 was ruled
  the map did not say it yet — T007 landed it later in the spec — and the
  amendment sign-off made the pause reports say so rather than claim it early.)
- **R2 — the shop is the "Card Shop" everywhere the player reads it**: the
  venue's action, the shop screen's own header, the key hints, How to Play,
  `Readme.md` and `docs/economy.md`. The person found "Outfitter" unclear, and a
  button that says one thing and opens a screen titled another reads as a
  defect. **Earlier specs' documents, `DECISIONS.md` and `ROADMAP.md` keep the
  old word on purpose** — they record what those specs did, and rewriting them
  would falsify history. Spec 029's own `spec.md` is the live contract and was
  corrected (caught at the amendment sign-off, which found the carve-out had
  been widened to all of `specs/**`). No symbol was renamed — `ShopState`,
  `open_shop`, `shop.rs` were never false — and `shop::TITLE` became the one
  string the venue's button and the shop's header both read.
- **R3 — the art dominates the screen, at every width.** The person's words: a
  much larger canvas allows more detailed, immersive art, and makes it feel like
  you are in the venue, on the planet. The venue became horizontal bands — the
  header rows above, the art and the portrait beside it, the action row and hint
  below. Two consequences the person ruled on when asked: **the portrait keeps
  its own column** (M1's reason survives intact), and **it draws at 89 columns
  too**, superseding N1 — a text-only venue at the minimum size is the version
  they least wanted. The cost, named at the time because it would land on a
  deferred art spec (R9 later brought the art into this one):
  **each planet's art must work at two quite different sizes.** R3 also cost
  one struct and one constant — `VenueRail` (which made "both regions or
  neither" a fact of the type) and `VENUE_ART_W` — **deleted** rather than kept
  as unconditional wrappers: a one-variant wrapper whose doc describes a
  condition that no longer exists is the false-name defect this spec renamed
  four symbols to avoid.

### The second amendment (the person, 2026-09-21, after walking the rebuilt venue)

- **R4 — the art about 15–20 % smaller.** Landed at **17.5 %** smaller at 89
  columns (58×23 → 50×22) and **16.8 %** at 139 (108×23 → 94×22) by giving the
  art **seven eighths** of the columns left of the portrait's gap. That is the
  **one scale factor** the plan takes, and a stated divergence from its own
  "the art's size is a subtraction, not a ratio" (plan §Design tension 7):
  R4 asks for a percentage, and no fixed column inset lands inside 15–20 % at
  both fit sizes (the workable insets, 7–9 and 12–17, do not overlap). The
  height is still a pure subtraction; there is still no minimum-art-size
  constant and no breakpoint. **R9 deleted the fraction** (below): the box now
  follows the drawings, whose sizes are R4's result, so R4 survives as those
  sizes and its band test still pins it; and the narrow and wide drawings now
  switch at 139 columns, the layout's existing threshold.
- **R5 — every text row centres on the art's centre, not the terminal's.** The
  person's eye caught it: the portrait's column puts the band's centre well
  right of the art box, so text centred on the terminal read as shifted off the
  thing it labels. Header and footer alike — aligning only the header would
  trade one mismatch for another. The consequence the person was warned of —
  at 89 columns the 63-character controls hint cannot fit over a 50-column art
  box and would land flush against column 0 — was resolved by **shortening the
  hint to 55 characters** with the board's own single-space ` · ` separators.
  Rejected, each for a stated reason: aligning only the header (`spec.md`
  forbids it — one mismatch traded for another); clamping the hint's column to
  a minimum (the row would be centred on nothing, bringing back for one row the
  off-centre look R5 removes); splitting the hint over two rows (it costs a row
  the art needs and puts a second text row in the footer band, where only the
  action row may have air); and widening the art to contain the hint
  (impossible at 89 columns — the widest art there is 58, and R4 wants about
  50). The shortened hint still overhangs the art box by 3 columns left and 2
  right at 89, which `spec.md` sanctions. `spec.md`'s "not flush against column 0" became
  an assertion: the fit test had been satisfying it silently, because
  `saturating_sub` clamps a left overflow to column 0 and the test checked only
  the right edge. The assertion now makes **61 characters** the ceiling for any
  future venue row at 89 columns.
- **R6 — the venue shows the credit balance**, in the Card Shop's own words:
  `shop::credits_label` is the one function both screens call, so the wording
  cannot drift. It is the screen where the player chooses between playing and
  shopping, which is the choice a balance informs (asked at the Phase 2 pause,
  answered here). Checked by the existing drawn-frame breathing test — which
  requires the row to be non-blank — rather than by a new test; the row's
  *content* is not asserted, which the plan sanctioned.
- **Plan §Open questions 2, closed by the person: the placeholder keeps the
  planet's name** until there is real art. (It had been asked at the Phase 2
  pause and re-put at the first amendment's walkthrough.) Since R9 the name is
  only the fallback for a planet with no art, which a validated delivery never
  has.
- **A deliverable, not a behavior: the per-planet art brief** ships on the
  branch as `specs/029-tournament-rounds/planet-art-brief.md`, in spec 016's
  shape. It asks for **two grids per planet — 48×20 and 92×20, sixteen files**
  under `assets/planets/` — because the art region's interior is a different
  width at the two layout sizes, which is the cost R3 named. It records the
  **single-asset alternative** (one 92-wide grid whose central 48 columns stand
  alone) in case the person preferred eight drawings to sixteen, and it priced
  four options for the **loading rule between the fit sizes** and chose none
  (T005e). The count was put to the person at the second amendment's pause;
  they answered with R7 and continued, so the brief stood at sixteen with the
  alternative recorded. There is no deferred art spec now: **R9** took both
  questions into this spec — sixteen drawings were delivered, and the person
  chose a rule the brief had not priced — **the box fits the art** — over all
  four it had.

### Two more rulings (the person, 2026-09-22)

- **R7 — a consistent, slight gap between the art and the portrait.** The
  person's words: the portrait sat a little too far to the right. The gap is
  now exactly `PANEL_GAP` (3 columns) at every width and the art-plus-portrait
  group is centred; the art's size (R4) and the text's alignment (R5) did not
  move. The outer margins are equal to within one column (at an odd leftover
  the right is one wider). Specified as a **class** of sanctioned assertion
  changes rather than a list — see *The gate pattern* below — and it landed
  first time.
- **R8 — How to Play says "matches."** How to Play opens with a rule about
  *rounds*, so "Opponents are Best of 3" could be read as rounds. The campaign
  lines now read "Each opponent is Best of 3 matches, the last / Best of 5. A
  started series is played out." (asked at the Phase 4 pause).

### R9 — the art, integrated (the person, 2026-09-22, at the merge pause)

- **R9 — the per-planet art is integrated in this spec**, rather than merging
  on the placeholder and leaving it to a spec of its own. The person's words:
  *let's add integrating the art into this spec.* **Authoring stays outside**:
  the sixteen drawings were made from `planet-art-brief.md` by the tool the
  person used (**Opus 5.5**, credited in `assets/CREDITS.md`), and this spec
  validates them and draws them. It added acceptance criterion **21** and a
  phase of its own (Phase 6), and narrowed the spec's art non-goal to
  *authoring*.
- **Between the fit sizes, the box fits the art** — the person's choice over
  the four options the brief priced (letterbox the drawing in a larger box,
  stretch it, tile or extend it, commission a third size). The box is always
  exactly the drawing plus its border — the narrow drawing below 139 columns,
  the wide one from 139 up, 20 rows tall — and space the drawing does not use
  is margin *around* the art-and-portrait group, never blank space inside the
  frame. At exactly 89×31 and 139×31 this is the layout R4, R5 and R7 had
  already approved, so nothing moved at the fit sizes (the orchestrator
  captured the venue at both before and after the geometry change, and the
  captures were identical). **The planet's name survives only as the fallback**
  for a planet with no art.
- **Three plan calls, each with its reason.** (1) **A taller terminal centres
  the whole venue vertically** (plan §Open questions 8): the 31-row
  composition keeps its internal spacing and the spare rows go above and below
  it, the odd one below, as the board's block does — so the text keeps its
  relationship to the picture it labels. The cost: above 31 rows the controls
  hint is no longer on the last row. (2) **The drawing is drawn at full
  strength** — `Emphasis::Normal`, the portraits' weight, inside a muted
  border (§Open questions 7): the brief told the artist the picture is drawn at
  one uniform emphasis with depth carried by glyph density, which dimming would
  compress. (3) **R4's fraction is deleted**: the box follows the drawings now,
  R4's result survives as their size, and its 15–20 % band test still pins it.
- **AC 21's checklist test derives its canvas from the venue's layout**
  (`VenueLayout::new(c).art` at each fit size) rather than restating 48, 92 and
  20 — which closes close-out note 37's concern that nothing tied the brief to
  the code. The test runs the brief's items 1–6 (exact set of visible files, 20 lines, one
  trailing newline, exact width in characters, the closed 23-character
  palette with its count asserted, distinct drawings) and checks for `\r`
  *before* the palette, so a CRLF file fails on the right assertion. Two
  mutation checks confirmed it bites (one extra space; one file converted to
  CRLF). Three more tests: every planet's art fills its box at all 660 sizes
  the venue's every-size tests measure (every width from 89 to 220, at 31, 32,
  33, 40 and 60 rows), the drawn frame's interior equals the drawing, and a
  planet without art shows its name.
- **`.gitattributes` marks `assets/planets/*.txt -text`.** Windows is a target
  and Git for Windows defaults to `core.autocrlf=true`. The first reason given
  for it was half wrong, and corrected at the Phase 6 review: a CRLF checkout
  would **not** mis-size the art, because `str::lines` strips `\r\n` and the
  drawer iterates `lines()` — but it **would** fail the new `\r` assertion, so
  `cargo test` would break on such a checkout. That alone justifies the rule.
  The portraits have no such protection, which is fine only because they have
  no `\r` test.
- **`design/brief.md` records the art as a second bounded exception** to its
  *Skeuomorphism boundary*, beside the portraits' (Phase 6 plan sign-off, B3:
  the portraits had been the *single* exception, amended before spec 016
  shipped them, and R9 needed the same amendment). Bounded the same way:
  static, venue-only, inside one single-weight box sized to the drawing, no
  colour, the portraits' glyphs plus four ASCII marks (`. ' * +`), no
  lettering, no discernible figures, original places. Written by the
  orchestrator on the branch and shown to the person at the Phase 6 pause.
- **The person's answers at the Phase 6 pause (2026-09-22).** The art tool was
  Opus 5.5. The person had **Cinder's and Scree's art redrawn "with right
  angles only"** and committed it themselves (`bc2598c`), then said to
  continue — read as the go on the art (the brief's checklist item 7, the
  product owner's look). The redraw was validated like the first delivery:
  items 2–5 by shell on the four files, and AC 21's test over all sixteen.
  Three things were shown and **not ruled on**; see *Asked, and not answered*.

### Asked, and not answered

- **Plan §Open questions 1 — the deciding match's game-over frame shows no
  series score.** The in-match score is derived from the live series, and the
  deciding match ends the series during settlement, one tick before the
  game-over popup; so after a non-deciding match the band shows the updated
  score, and after the deciding one (won or lost) it shows nothing, while the
  map banner that follows names the result. The plan sign-off ruled it **not an
  AC 11 violation** (the score is on every frame the player can act in). Put to
  the person at the Phase 3 pause, with both frames; **they continued without
  ruling. Left as built, and not treated as a ruling either way.** Holding the
  final score is a one-shot `App` field in the `victory_due` idiom, a
  sub-lettered task rather than a redesign, if they later want it.
- **Three things shown at the Phase 6 pause (2026-09-22)**: the art's weight
  (plan §Open questions 7 — drawn at full plain weight; dimming it is a
  one-word change), vertical centring on terminals taller than 31 rows (§Open
  questions 8 — the hint leaves the last row), and the text of the
  `design/brief.md` amendment. **The person continued without changing any of
  them. Left as built**; the amendment text stands as written.

### The design calls (plan, signed off 2026-09-20)

- **The return target is derived from the lock, never remembered.** Spec 015
  shipped a bug where a *remembered* origin was not set on one path; a second
  remembered target (a venue origin on the deck builder and the shop) would
  have doubled the places that can forget. Instead `App::open_campaign_home` is
  the **only** place that assigns `Screen::CampaignMap` or `Screen::Venue`, and
  it reads whether a series exists. Every door — Continue, New Campaign, Reset
  Everything, the game-over acknowledgement, Back from the shop and the deck
  builder, the map's launch — goes through it, and the deck builder's
  invalid-deck divert gets the venue return for free. **The invariant is held by
  a reviewed grep, not by the type system**:
  `grep -nE "self\.screen = Screen::(CampaignMap|Venue)" src/app.rs` must return
  exactly two lines, both in `open_campaign_home`. It is written as two
  assignment statements rather than one `if` expression precisely so the grep
  can see them — in the expression form neither line names a variant and the
  gate would pass while checking nothing (T006's deviation, upheld at the
  Phase 2 review, which also widened the grep three ways). A `CampaignHome`
  enum with a mapping test was drafted and dropped: the test would have been a
  tautology aimed at the wrong risk.
- **One floor, one predicate: `economy::reserve_floor`.** O1 changes what the
  floor *is* while locked, not how many there are. `cheapest_floor` kept its
  body and became private; `reserve_floor` returns the locked opponent's ante
  while a series runs and the cheapest launchable ante otherwise, and all of
  `Profile::is_broke` and `Profile::can_afford` (and through it the shop's
  purchases, dimming and *spendable* readout) read it. While locked, the
  reserve **is** the venue opponent's own ante, so a player at the venue who is
  not broke can always cover the match it offers and the venue draws no banner
  — **but only because the map's launch refuses a series the balance cannot
  cover**, which the pre-merge sweep found missing (its blocking B1): the floor
  rises at the moment a series starts, and without that check a player with 15
  credits could launch a 20-ante series, be locked into it with no playable
  match, and lose the run at the next campaign entry. One helper,
  `App::refuse_uncovered`, now guards both the map's series launch and every
  wager. The wager prompt's warning reads `economy::reserve_after_a_loss`
  instead — see *O1's side effect* below.
- **Settling exactly once stays a data property, and now covers the series.**
  Spec 021 made the payout idempotent by zeroing the escrow; a series tally
  increment has no such property, and `mark_beaten` on the wrong match would
  clear a planet early. So the in-flight `NodeRef` carries a `settled` flag and
  `CampaignRun::take_settlement` takes the node and its stake **once** — a
  second call returns `None`, pays nothing, moves no tally and beats nobody.
  `take_stake` was deleted: two ways to empty one escrow is one too many. **What
  it still does not cover**: `Profile::record_match`, which `resolve_match` runs
  *before* settlement, so a second `resolve_match` on one match would still
  double-count lifetime and run statistics — pre-existing, still guarded only by
  the `phase_changed && GameOver` edge, and stated in `take_settlement`'s doc
  rather than claimed away. One visible consequence in tests only: a second
  settlement now returns `None` where it returned `Some(Won(0))`; the claim that
  this changes nothing in production rests on inspection of that edge, not on a
  test.
- **The series is stored beside the in-flight pointer, not inside it**, because
  the two have different lifetimes: a Quick Play match and the kill-with-no-save
  forfeit both clear the pointer, and neither may end a series. So a Quick Play
  match started mid-series cannot end it, and the board's score line is gated on
  the in-flight node *matching* the series, not on a series existing.
- **How many wins a series needs is derived from the opponent id**
  (`wins_needed`, `FINAL_OPPONENT = "sovereign"`), never stored: a stored count
  is a second source of truth a hand-edited or older save could contradict, and
  derived, a pre-029 profile gets the right length for free. One constant
  rather than a roster field, because `opponent.rs` is balance data this spec
  must not touch.
- **The migration rule: `is_opponent_beaten`, not the presence of a series,
  decides whether a settled match can beat its opponent** (plan sign-off B1).
  The first draft settled a match left in flight across the upgrade as a
  non-series match, so a player mid-match against Greeb when they upgraded would
  have cleared Cinder in one win — the outcome this spec exists to remove. Now
  `record_series_match` **begins a series** when none is running and the
  opponent is un-beaten, and credits the match to it, exactly as `spec.md`'s
  "a match left in flight resolves as the first match of a fresh series" says.
  So a beaten opponent is a rematch and an un-beaten one is always in a series,
  and **no settled match can beat an opponent who has not lost a series**. The
  `NotInSeries && player_won` clause that remains in the beat condition can
  only ever re-mark an already-beaten opponent — a no-op kept to preserve the
  rematch path literally.
- **Four renames, each because the old name would assert something false**:
  `take_stake` → `take_settlement`; `cheapest_floor` → `reserve_floor` (the old
  one kept, private, as its implementation); `BuilderOrigin::Map` /
  `BackTo::Map` → `…::Campaign` (the builder no longer necessarily returns to
  the map); `map_entry_modal` → `campaign_entry_modal`. `enter_campaign_map` →
  `enter_campaign` and `open_campaign_map` → `open_campaign_home` followed from
  the first design call. `src/wager.rs`'s reserve doc was corrected with them
  (plan sign-off B6): exempting it from the rename's grep would have satisfied
  the gate and left the false claim standing.

### O1's side effect, which no walkthrough could reach

The wager prompt's warning row — **"Lose this and the run is over."**, added by
the 2026-09-17 chore — is driven by the prompt's reserve, which is now
`reserve_floor`. While locked against a deep opponent that reserve rises from
the map's cheapest (10, Cinder's rematch, in every run state) to **that
opponent's own ante** — 50 against Rix and up — so the warning fires at far
lower stakes than it did. That is correct under O1 and probably desirable: the
warning still reads the same floor the broke check reads. But it is a real
change in how often a player sees that row, it needs a Core-depth run that no
walkthrough in this spec reached, and so it is recorded here rather than
attested. It also **supersedes, while a series is locked, the chore's bullet
"The predicate is the run's cheapest ante, not the prompt's own floor"**: while
locked the two are the same number. The chore's own section above is left as
written.

**Fixed on the branch by the pre-merge sweep (its N1).** A loss that decides a
series releases the lock, so the broke check after it reads the map's cheapest
ante again; the warning had been reading the locked opponent's ante and could
say "Lose this and the run is over" when the run would continue. The prompt now
takes `economy::reserve_after_a_loss` — the floor the broke check reads after
this match is lost, found by recording the loss on a copy of the run so the
series' own rule decides it — and so again predicts exactly the check that ends
the run, as the 2026-09-17 chore ruled. It only ever over-warned; it never
under-warned.

### Coverage, stated honestly

- AC 14 is two-thirds instrumented: `every_reset_clears_the_lock` covers New
  Campaign and Reset Everything; the run-over reset is covered by the fact — in
  a comment — that it is the same call.
- AC 16's "the largest element on the screen" is asserted only as art area >
  portrait area, which is sufficient while every other element is a single row
  of text; a later spec adding a second panel would not be caught.
- The venue's credit row (R6) is pinned by being non-blank and by the shared
  `credits_label`, not by an assertion on its text.
- `the_deciding_match_and_the_series_agree` plays only all-win and all-loss
  series; `wins_needed_is_two_except_for_the_final_opponent`'s loop computes its
  expectation with the function's own expression and is rescued by its other two
  assertions — don't trim it to the loop.
- `no_settled_match_beats_an_unbeaten_opponent_outright` reads
  `is_opponent_beaten` *after* the call, which is sound only because
  `record_series_match` marks nobody beaten; if `mark_beaten` ever moves into
  it, that assertion silently changes meaning.
- A non-deciding match's `MapBanner::Settled` is set and never shown as a
  banner (the venue draws none); nothing is lost, because `stake_to_show` puts
  the settled amount on the game-over frame, but the case was un-discussed
  rather than decided. A contradictory banner ("Series won · Lost N credits")
  is unreachable in play — only a hand-edited save could produce it.
- The winning series banner (`★  Series won · …`) was never seen on screen —
  six auto-played attempts lost — and is pinned by an exact-string test.
- AC 21's checklist test is the whole of the validation a machine can do.
  Nothing can test `assets/CREDITS.md`'s originality claims — original places,
  no franchise imagery — which rest on the person's look at the art.
- `T003` renamed the `sweep_run` test helper to `sweep_run_in_series` and
  changed its behaviour where the task line said helpers should *gain* a
  series form; the rename is total, so every call site shows in the diff, which
  is what that bar protected.

### The gate pattern (process, recorded so the next spec plans against it)

This spec hit, repeatedly, **a Verify gate or an "exhaustive" list of changing
assertions that could not be satisfied or was incomplete**: T001's
one-exception list (three assertions had to move; a decision review ruled it a
planner enumeration error), plan sign-off B2/B3/B6 (Verify gates unsatisfiable
because call sites were under-enumerated — B6 was `src/wager.rs`, listed under
*No change* while it named the renamed function), T006 (the plan's code listing
would have made its own grep gate return zero lines), T005b (a gate spelling the
file `README.md` when it is tracked as `Readme.md` — harmless on darwin,
silently unsatisfiable on a case-sensitive filesystem), T004a and T005c (each
changed an assertion or a doc its "anything else is a stop-and-report" list
omitted, and neither implementer stopped), T007 (an unlisted construction site),
and the second amendment sign-off's B1 (a `git diff --stat` gate for a task
whose whole product is one untracked file, which that command cannot list — a
lesson specs 022 and 023 had already written down). **What worked**: T005f
stated the sanctioned change as a **class** ("any assertion whose subject is
the portrait's x position, the art–portrait gap, or the right-hand margin")
rather than a list, and landed first time; T008 enumerated its call sites by
grep before editing and found no gap. Related orchestrator misses: review
bundles showed `cargo test -q --lib` plus a warning-count grep instead of the
constitution's full command — three recurrences before it stuck — and the grep
returns 0 for a failed build as readily as a clean one; and two bundles echoed a
gate's *label* rather than the command actually run. Echo the command, don't
retype it.

Phase 6 added two more of the same kind at its plan sign-off, both caught
before dispatch: **B1**, a grep gate for deleted constants that substring-matched
`CampaignMapLayout`'s `FIELD_MARGIN_X` and so could never come back empty (fixed
with `grep -w`); and **B2**, a "clean working tree" check that an implementer
who never commits cannot satisfy (scoped to `assets/planets`).

**And one orchestrator miss of a different kind: the art was committed before
it was validated.** The art session wrote the sixteen files into
`assets/planets/` while the orchestrator was committing the Phase 6 plan with
`git add -A`, which swept them into that commit (`7445c68`) unvalidated and
unmentioned in its message. History was not rewritten (never force-push); the
delivery checkpoint, T013, validated the files already committed instead of
committing them, and they passed first time. The Phase 6 review checked the
commit: exactly the sixteen art files plus the three spec documents, nothing
else swept in. From then on the orchestrator stages **explicit paths, never
`git add -A`** — a checkpoint that validates before committing only works if
nothing else can commit first.

### What didn't change

`game.rs`, `card.rs`, `player.rs`, `save.rs`, `opponent.rs`, `Cargo.toml` and
`Cargo.lock` are untouched; `PROFILE_VERSION` and `SAVE_VERSION` stay 1; the two
new persisted fields are `#[serde(default)]`, the pattern spec 021 used for
`NodeRef::stake`; `economy.rs` lost only two doc lines and the `pub` on
`cheapest_floor` — no constant moved; no new crate; the build has no warnings,
the same count as `main`. Monochrome by construction: the venue, the series line
and the banners use the existing emphasis levels and add none, and the art is
drawn at the portraits' plain weight from a closed 23-character palette that
admits no escape character. The art adds sixteen text files, a
`.gitattributes` line and a `CREDITS.md` section — no crate, and no engine or
save file.

## Series-aware banter, spoken word by word (spec 030)

Since spec 029 a campaign opponent is played as a series, but its lines
didn't know it: every match opened on a greeting, and a match that took the
series ended like any other. Spec 030 gives each voice six series pools and
has the opponent speak at the venue. It also makes every line **spoken**,
word by word, with a synthesized burble per word, and gives the burble a
**Voices** volume. And it holds the final series score on the deciding
match's game-over frame. It adds two types (`banter::Speech`,
`banter::SeriesState`), one sound (`assets/sfx/burble.wav`), one settings
field (`voices_volume`), two constants (`WORD_STEP_MS` 200, `EVENT_BEAT_MS`
400), and 146 lines of dialogue. No engine, AI, save-format, economy,
balance-data or dependency change. The person ruled 1B–6A in the spec
conversation (2026-09-23) and 7A and 8A at planning. They amended the spec at
the Phase 1 pause (9A–11A), after listening, and at the Phase 3 pause (12A,
13A), after walking the venue.

### The rulings (the person, 2026-09-23)

- **1B — a match started mid-series has lines for Leading, Trailing and
  Decider**, plus All square for the best of 5 at 1–1, rather than folding
  the decider into "all square". Decider means both sides are one win from
  the series (1–1 in a best of 3, 2–2 in a best of 5), so only The Sovereign
  reaches All square, and only its voice carries those lines. Every other
  voice's pool is empty, and no test pins that emptiness (unreachable, and
  harmless).
- **2A — the match that decides a series ends on series won / series lost
  lines.** Other match ends keep today's lines.
- **3B — the opponent speaks one line at the venue on arrival**, reacting to
  the series score.
- **4A — about 0.2 s per word, the first word at once** (`WORD_STEP_MS` =
  200, bounds pinned at 150–250). Every line has at most five words, so every
  line finishes within a second of its first word. That is pinned for every
  line of every pool.
- **5B — a soft burble per word.** The person's words were "soft and not
  louder than the music … sort of a 'soft burble' if possible." 9A later
  dropped the "not louder than the music" half.
- **6A — the deciding match's game-over frame holds the final series
  score**, which "should not follow the player to the galaxy screen." This
  closes spec 029's *Asked, and not answered* item (plan §Open questions 1
  there) in the direction that section predicted: a one-shot `App` field.
- **7A (asked at planning) — the compact board draws no line, so no burble
  plays there.** "If the banter isn't visible, it makes no sense to include
  the speech burble." The line on the compact board is a roadmap item.
- **8A (asked at planning) — a resumed match stays blank**, as spec 017
  shipped it, and no line is spoken on resume.

### Defaults set in the spec conversation, not separately ruled

Words appear in place. A new line interrupts the old one. Animations Off
shows the line whole, with one burble. A resumed match's line is whole and
silent (superseded by 8A: a resumed match shows no line). Quick Play keeps
today's lines but is spoken. The venue and match-start lines share a pool, with
no repeat across the two. The Card Shop / collection round trip is not an
arrival.

### The Phase 1 amendment (the person, 2026-09-23, after the first listen)

- **9A — the burble is as loud as the other sound effects**, and "never
  louder than the music" is dropped. The first build met that rule to the
  letter: its peak (0.08) sat under a ceiling of the music's RMS × default
  music volume / default SFX volume (0.1025). The person found it "really,
  really quiet" and had to turn the music off to hear it. Measured by peak,
  a soft-enveloped voice with tremolo is far quieter than a square-wave blip
  with the same peak, so the rule had been measuring the wrong thing.
- **10A — a Voices volume in Settings**, which the burble follows instead of
  Sound FX. `m` mutes it like every other sound.
- **11A — a line answering an event waits a beat.** A line said on a round,
  bust, tie or match/series event is chosen when it is today, but its first
  word waits `EVENT_BEAT_MS` (400 ms), so the event's own sound plays first.
  Popups, sounds, phases and keys keep their timing. The value is the top of
  the ruled 0.3–0.4 s. The opponent's bust sound at `OPPONENT_PITCH` (0.92)
  runs 380 ms, which rules out 350.
- **The Voices default is 50%** (T002c, at the re-listen). The person's
  words: "I have the volume set to 50% where it sounds well mixed with the
  rest of the sounds, so maybe we should make that the default." The plan had
  made it equal to Sound FX's 80%, so that the default settings compared like
  with like. The ear overruled the arithmetic. At 50% the loudness test still
  passes unchanged, but only just (below).
- **The approval (AC 9).** At the re-listen: "Murmur is good now and is
  correctly reflecting the settings." That was the only tweak asked for
  (T002c). No `BURBLE` number moved after T002b.

### The Phase 3 rulings (the person, 2026-09-23, after walking the venue)

- **12A — the venue line waits the same beat.** "Add a slight delay to the
  line so that the sounds don't overlap." The acknowledgement's menu click
  and the venue line's first burble had landed on the same tick. The venue
  arrival now passes `EVENT_BEAT_MS` too, reusing the one beat constant, so
  only the match-start and rematch greetings start at once. `spec.md`'s AC 8
  was amended to say so. The Phase 3 re-review caught that the amendment had
  first been missing, and the orchestrator transcribed it.
- **13A — an arrival under the run-over notice says no line.** The notice
  covers the presence panel at 89 columns and clips its first character at
  139, yet the line was spoken with its burbles and the run then reset.
  `arrive_at_campaign` now returns early when `profile.is_broke()` (the check
  `campaign_entry_modal` uses for the notice) and sets `speech = None`. The
  `None` is required, because otherwise the match's closing line, possibly
  still in its beat, keeps burbling at the venue. The person would prefer
  the opponent to taunt that the run is over, but not if it needs more
  design, and it does. It is on `ROADMAP.md`.

### The burble as shipped

`scripts/gen_sfx.py`'s `BURBLE` block, as it ships:

    "length_s": 0.12, "pitch_hz": 150, "glide": -0.12,
    "vibrato_hz": 18, "vibrato_depth": 0.04,
    "tremolo_hz": 24, "tremolo_depth": 0.35,
    "formants": ((500, 750, 90), (1100, 1400, 140)),
    "harmonics": 18, "attack_s": 0.015, "release_s": 0.06,
    "peak": 0.79

The synthesis is additive: `harmonics` harmonics of a gliding, vibrato'd
150 Hz fundamental, each weighted `1/k` times the resonance of two gliding
formants (a "wo→a" vowel), with a raised-cosine envelope and a tremolo. It
uses no `random`, so `bust`'s seeded noise is undisturbed. Per-word pitch is
`audio::BURBLE_PITCHES` = `[1.0, 0.94, 1.05, 0.97, 1.02]`, and `burble_cue`
wraps past its end. `python3 scripts/gen_sfx.py burble` regenerates that one
file and leaves the other thirteen byte-identical.

**`peak` went 0.08 → 0.79** (T002b), chosen so the burble's whole-clip RMS
(0.1668) lands at the mean RMS of the four board move sounds (0.1660). Those
are `CardDraw` 0.1629, `CardPlay` 0.2137, `Flip` 0.1882 and `Stand` 0.0992.
At default volumes the band is 0.0794–0.1710 and the music floor is 0.0820
(music RMS 0.1641 over its first 60 s × 0.5). At Voices 50% the burble reads
**0.0834**: inside the band, and 0.0014 over the music floor. That margin is
thin. A later default below about 49% fails the floor, and so does a quieter
burble. Either way the change is a product question (plan §Open questions 3),
not a looser test. A peak of 0.79 is near full scale at Voices 100%, which is
for the ear to judge. The test decodes a 60 s MP3 window and takes about 4 s
in a debug build.

### The design calls (plan, signed off 2026-09-23; amended the same day)

- **One `Speech` replaces `banter`.** The shown line and how much of it has
  been said live in one `Option<Speech>`, the renamed spec 017 field
  (`App::banter` → `App::speech`; `banter_last` unchanged). Replacing the line
  replaces its clock, and clearing it drops the clock, so interruption (AC 10)
  is a property of the data, not something each call site remembers.
- **Reveal by blanking, not slicing.** `revealed(line, n)` returns the line
  at its full length with each unsaid word's characters turned to spaces. Any
  centring drawer then puts each said word at its finished column (AC 8) with
  no new geometry. Passing an offset and a prefix would have duplicated
  `draw_text_in`'s centring in a second place.
- **Burbles come only from advancing the one `Speech`, at most one per
  step, and never two within `BURBLE_GAP_MS` (150 ms).** Nothing is queued in
  the audio thread, so a replaced or cleared line cannot burble again. Every
  burble goes through `App::burble`, the only caller of `burble_cue` (one
  call), which plays only if `audio::burble_clear` holds. Two burbles at once
  play louder than one, and after 9A that pushed the burble past the other
  effects and garbled it. Two bounds are pinned: the burble at its slowest
  pitch ends within the gap, and the gap is at most three loop ticks. **The
  gap drops a burble in exactly two cases**, both to keep the sound soft:
  (1) a new line whose first word arrives within 150 ms of the old line's
  last burble shows that word silently, which since the beat (11A, 12A) can
  happen only to a greeting; and (2) a stalled frame that crosses two word
  boundaries shows both words and owes one burble. A third caller arrived
  with 10A: the Voices row's preview. The gap keeps a held ←/→ from stacking
  previews. `self.burble(` has exactly three callers (`say`,
  `advance_speech`, the Settings arm), and the Verify gates counted them in
  every task after T002a.
- **The Animations setting is read at say-time.** `Speech::new(line,
  animated)` decides how many burbles the line is owed, which is settled when
  it is chosen. The setting can only change from the Settings overlay over
  the start menu, where no line is on screen, so reading it at say-time is
  never stale.
- **Arrival is the caller's word; returns settle in `tick`.**
  `open_campaign_home` keeps its single job and stays the only place either
  campaign screen is assigned (spec 029's grep still returns exactly two
  lines). The new `arrive_at_campaign` calls it and then says the venue's
  line. Its callers are `enter_campaign` (every menu entry and the game-over
  acknowledgement) and `launch_from_map`. The Card Shop's and the deck
  builder's Back arms keep calling `open_campaign_home`, so a return says
  nothing new. On any screen other than the board or the venue, `tick`
  **settles** the current `Speech` (every word shown, silently), the way it
  resets `BoardMotion`, so coming back draws the whole line. Two alternatives
  were rejected: a flag on `VenueState` (rebuilt on every return) and an
  `arrival: bool` on `open_campaign_home` (spec 015's bug was a caller
  forgetting what a return is).
- **The series state lives in `banter.rs` and is read through
  `match_series`.** `SeriesState` is a from-the-opponent's-side reading used
  only to pick lines, so `campaign.rs` stays untouched. At match start the
  series comes from `match_series(in_progress, series)`, which is
  `board_series_line`'s node-matches-series rule pulled out into its own
  function. Quick Play, a rematch and a match left in flight with no series
  therefore all read as "no series", exactly as the board does. At the venue
  it comes from the locked series, because the pointer is already cleared by
  then.
- **`banter_last` is fed to the match start only inside a series**
  (`let last = state.and(self.banter_last)`; sign-off B1). The first draft
  fed it always, which would have changed Quick Play's pick, against AC 4.
  Now a series match start avoids the venue's line (AC 6), and every
  no-series match passes `None` exactly as before.
- **`final_series` follows the motion idiom rather than `victory_due`'s
  take-on-entry.** It is set in `tick`'s resolution block from the match's
  series cloned *before* `resolve_match` (settlement clears the live series
  one tick before the game-over frame and the closing line need it). It is
  cleared in `tick`'s `_` screen arm beside the `BoardMotion` reset, so it is
  gone on every exit from the match, not just the one that exists today. The
  map never reads it. **`decided_series` repeats the tally rule under a
  test** (`decided_series_agrees_with_the_series_rule`, which drives
  `record_series_match` through both series lengths) rather than changing
  `SeriesOutcome` to carry the score. That change would have touched
  `campaign.rs`, `profile.rs` and every `MapBanner` match.
- **Loudness as a number.** As signed off it was a ceiling: peak ≤ music RMS
  × default music / default SFX. 9A dropped the sentence it tested, and the
  first listen showed that peak was the wrong measure anyway. It was
  **replaced, not deleted**. `the_burble_is_as_loud_as_the_other_sounds` took
  `the_burble_is_softer_than_the_music`'s place in the same diff. It checks a
  **band** (the burble's whole-clip RMS × default Voices lies between the
  quietest and loudest of `CardDraw`, `CardPlay`, `Flip`, `Stand` × default
  Sound FX) and a **floor** (at least the music's first-60-s RMS × default
  Music), plus a peak under 1.0. Whole-clip RMS is the simplest measure the
  repo can take without a crate. It does not weight frequencies the way the
  ear does, which is why the person's approval is the other half of AC 9.
- **The event beat lives inside `Speech`, not at the `App`.** `Speech` gains
  `wait` and `after(wait)`. While waiting it shows nothing and owes nothing,
  and the step that ends the wait owes the first burble. A pending line held
  beside `speech` was rejected: it would be a second field that
  interruption, clearing and settling must each remember, which is the split
  the first design call refused. With the wait inside, every existing rule
  covers the beat as it stands. **The value is 400 ms.** It outlasts every
  round, tie and bust sound (pinned by
  `the_event_beat_outlasts_the_round_sounds`: `RoundWin` 210 ms, `RoundLoss`
  270, `RoundTie` 210, `Bust` at 0.92 pitch 380). It does **not** outlast
  the match-end jingles (`GameWin` 450 ms, `GameLoss` 520), so a match or
  series line's first word lands on their last note. Waiting out the whole
  jingle, about 0.55 s, would be outside the ruled range, and the person
  heard it at the re-listen and asked for no change.
- **Voices: one field and one routing function.** `Settings.voices_volume`
  is `#[serde(default)]`, so an older file loads with Voices at its default
  and keeps every other value. Without the serde default,
  `from_json_or_default` would have reset the whole file. One pure
  `audio::sfx_level(muted, sfx, settings)` gives each effect's volume or
  `None`: the burble on Voices, every other effect on Sound FX through the
  unchanged `should_play_sfx`. `AudioState::play` calls it. The default was
  set equal to Sound FX's (80%) so that the defaults compared like with like,
  and **T002c moved it to 50%** by ear (above). The Voices row **previews one
  burble** at the new level instead of the `MenuMove` tick, because a tick at
  the Sound FX volume tells the player nothing about Voices.
- **`voices_volume` is declared last in the struct** (T002a's decision
  review). serde's derived `Deserialize` also accepts a struct as a
  positional JSON array. `settings_malformed_or_empty_json_falls_back_to_default`
  relies on `[1,2,3]` failing when its third element lands on the
  `animations` bool, and declaring the volume third, as the task line said,
  let that array parse. Three options were weighed: declare it last (the test
  is untouched, and the only visible effect is key order in `settings.json`);
  edit the test (a test changed to pass); or add a custom deserializer (code
  nobody asked for). The review chose declaring it last. **Any later
  `Settings` field should also be declared last.** The field's doc says why.
- **The venue panel grew two rows** (`VENUE_PANEL_H` 15 → 17: border, name,
  portrait, a gap row, the line row, border). The venue line sits on the
  panel's interior row 14, exactly where the board draws it, through one
  shared drawer, `portrait::draw_banter_line`. The rejected alternative was a
  line floating under the 15-row panel, which reads as a caption *under* the
  opponent rather than the opponent speaking. **The class of layout
  assertions that moved**, sanctioned as a class in T007: six pinned venue
  portrait `Rect`s in `layout.rs`, each `y1` + 2; one area comment (330 →
  374); and two `VenueState::draw` calls gaining their `line` argument. That
  is a stated deviation from AC 11's parenthetical "existing tests
  untouched", which is about phases, popups and timings, none of which these
  assertions pin.
- **Tests never build an `App`.** `App::new` reads the real settings and
  profile, and `Profile::save()` has no `cfg(test)` guard. Every decision
  sits behind a pure function and is tested there. The `App` wiring (`say`,
  `advance_speech`, `arrive_at_campaign`, the settle-on-leave) is checked by
  the driven walkthroughs and the counted-call greps.

### Coverage, stated honestly

- **An opponent bust's line never shows now.** It is replaced one tick later
  by the round-outcome line (`game.rs` ~282–285 against ~399–400). Before the
  beat it showed for one tick, about 50 ms; with the beat it never shows.
  Nobody could see the difference. It is recorded so it isn't taken for a
  bug.
- **The beat lands at 400 ms plus up to one loop tick** (≈ 0.40–0.45 s), at
  the top of AC 8's range. Round-end timing was driven. Bust and match-end
  timing were not, since they take the same `update_banter` branch. The
  person heard the match end at the re-listen.
- **The silent cases rest on construction and the counted-call greps**
  (Animations Off with one burble, 89 columns silent, the resumed match, a
  replaced or cleared line, a return from the Card Shop). A driver cannot
  hear. The person's ear was the check.
- **A line still revealing can burble under something that covers it**, for
  under a second: under the `?` help overlay on the board, and under the
  wager modal at 89 columns, which covers the venue's line row.
- **`the_burble_follows_voices_and_nothing_else_does` lists the sounds by
  hand**, so a later `Sfx` variant is not caught automatically.
- **AC 4 rests on the board's own rule.** `match_series` returns a series
  only when the in-flight node is the one the locked series is played
  against. Three callers rely on that during Quick Play: `series_state_now`,
  `tick`'s `before` and the draw arm.
- **`banter_last` isn't persisted**, so the first venue line after a relaunch
  can repeat the one said before quitting (1 in 3). That is no regression:
  spec 017's no-repeat rule was always within a session.
- **Whether the menu-select sound ends inside 400 ms**, so that the venue's
  first burble truly follows it (12A), is for the ear. It is not measured.
  `the_event_beat_outlasts_the_round_sounds` could be extended to it.
- **`the_venue_rows_breathe_only_around_the_action_row` still draws with no
  line.** The geometry holds either way. Drawing a line there would be
  optional hardening.

### Process notes

- **A driver session touched the real data directory once.** In the Phase 1
  walkthrough, an Animations-Off run hit a zsh `nomatch` error that skipped
  the chained `export KAAZAP_DATA_DIR=…`. The run re-saved the person's
  profile, wrote a Quick Play `savegame.json` and set Animations Off in their
  `settings.json`. It was reported to the person and nothing further was
  touched. The lesson: set the data directory on the command itself
  (`KAAZAP_DATA_DIR=… cmd`), not in an earlier link of a chain that can fail.
- **Verify gates that could not be read as written**: T003's
  `grep -nE '^\s*banter:'` also matches the `banter::{` import, which matched
  before the task too. The amendment sign-off's B1 was a bare
  `EVENT_BEAT_MS` grep that would also match the import and `say`'s doc
  (fixed to `from_millis(EVENT_BEAT_MS)`). And T001 found that `tail -n 25`
  of `cargo test -q` shows only the last test binaries' summaries, not the
  lib's 519, so reports add `cargo test -q 2>&1 | grep "test result"`.
- **What worked**: sanctioned changes stated as a **class** (T004's widened
  "every line" iterators; T007's venue panel assertions) landed first time,
  as they did in spec 029. So did call sites enumerated by grep before
  editing (T003, T007).

### What didn't change

`game.rs`, `card.rs`, `player.rs`, `campaign.rs`, `profile.rs`, `save.rs`,
`economy.rs`, `opponent.rs`, `board.rs`, `Cargo.toml` and `Cargo.lock` are
untouched. `PROFILE_VERSION` and `SAVE_VERSION` stay 1. The one new
persisted field is in the settings file, not a save or the profile, and is
`#[serde(default)]`. No existing banter line or pool changed. The thirteen
existing sounds are byte-identical. No new crate. The build has no warnings,
the same count as `main`. Monochrome by construction: the spoken line is the
existing banter line drawn through the existing drawer, and no colour path
was added.
