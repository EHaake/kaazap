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
- **New Campaign = full fresh start** — wipe campaign progress, credits, and the
  collection/deck back to the starter (`Profile::reset_to_starter` = a fresh
  `Profile::default`), keeping audio/settings (a separate file). Chosen over an
  NG+-style replay that keeps your arsenal, so the early game and the depth-gated
  economy stay meaningful; NG+ stays deferred to spec E (see the amended economy
  note above). Destructive, so it sits behind a default-No confirm.
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
  launchable match.
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
