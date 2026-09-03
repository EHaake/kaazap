# Spec: Campaign Map

## Summary

Today you reach a match either by resuming (Continue) or Start Game →
opponent-select. This spec adds the **campaign** itself: a full-screen,
node-based **star map** where planets are nodes, laid Outer Rim → Core, and you
travel core-ward beating the opponents each planet holds. It's the integration
layer that stitches the roster (spec 007) and your built deck (spec 008) into a
progression you move through.

This is subsystem **D** of the campaign epic, scoped to the **navigation and
progression-structure layer**: you travel the map, play a planet's opponents
with your side deck, and clearing a planet unlocks the next. The **economy is
deliberately stubbed** — wins record progress but grant no credits or cards yet
(that's spec C). The map gives the campaign its shape; the rewards layer plugs
into that shape later.

## Goals

1. **A campaign you travel.** A full-screen map of planets, Outer Rim → Core,
   mostly-linear with one light branch (a fork you can take in either order that
   rejoins). You move a cursor between unlocked planets and choose where to play.
2. **Planets hold opponents.** A planet holds one or more opponents (from the
   spec-007 roster); beating all of a planet's opponents **clears** it, which
   **unlocks** the planets that depend on it. Reaching and clearing the Core
   world completes the run.
3. **Play from the map with your deck.** Selecting a planet launches a match
   against its next un-beaten opponent, dealt from the side deck you built
   (spec 008). Winning records progress and returns you to the map; losing
   returns you to the map with the planet still open to retry.
4. **A distinct front door.** The start menu offers **Start Campaign** and
   **Quick Play** (Quick Play is the existing choose-any-opponent flow, kept).
   **Continue** still resumes an in-progress match. Your campaign run persists
   across sessions.
5. **Atmosphere within the terminal.** The map has real ornament — a header, a
   bottom info panel for the highlighted planet, connecting routes, and a
   **slowly twinkling starfield** backdrop (sanctioned by the amended Motion
   principle) — while staying legible.
6. **Test coverage** per `CLAUDE.md` for the new logic (the map graph:
   clear/unlock/next-opponent derivation; the run's persistence; map
   navigation).

## Non-goals (explicitly deferred)

- **The economy** — credits, a shop, card rewards, depth-gated card unlocks.
  All spec C. In this spec, a win only records that the opponent was beaten.
- **City-zoom** — a planet expanding into a second-level map of city/cantina
  sub-nodes. A later spec; planets ship as a flat, ordered opponent list.
- **Roguelike lives / run-loss stakes** — spec E. A lost match here has no
  penalty beyond staying on that planet; you retry freely.
- **The two-panel "briefcase" deck-builder** — a separate presentation pass.
- **New opponents or map content beyond the first map** — the first map uses
  the existing 5-opponent roster; it grows as the roster and economy do.

## Key user flows

### Entering and traveling the campaign

From the start menu you activate **Start Campaign**. The map fills the screen:
planets as nodes with connecting routes over a twinkling starfield, a header
band (the run's progress, the Outer Rim → Core axis), and a bottom info panel
describing the highlighted planet. You move the cursor between **unlocked**
planets (arrows / `w`·`a`·`s`·`d` / the emacs nav keys); the panel updates to
the highlighted planet — its name, region, its opponents (beaten / current /
locked), and a line of flavor. **Esc**/`x` returns to the menu. Choosing Start
Campaign while a match is saved prompts to discard it first; once on the map,
moving around and backing out are free.

### Playing a planet

Pressing **Enter/Space** on an unlocked, not-yet-cleared planet starts a match
against its **next un-beaten opponent**, dealt from your built side deck. (If
your deck isn't a legal 10, you're sent to the deck-builder first, as when
starting any match.) You play the match normally. On a **win**, the opponent is
recorded beaten; acknowledging the result returns you to the map, where the
planet reflects its new state — cleared if that was its last opponent, which
unlocks whatever depended on it. On a **loss**, you return to the map and the
planet stays open to try again. Clearing the Core world completes the run.

### The first map

Cinder (Outer Rim) → a fork of Ashfall and Drift (Mid Rim, either order) →
The Spindle (Core, two opponents). Beating Cinder's opponent unlocks both
Ashfall and Drift; clearing both unlocks The Spindle; clearing The Spindle
completes the run. (Planet names are original placeholders, tunable.)

### Resuming

**Continue** resumes an in-progress match board (campaign or quick-play),
exactly as today; on a resumed campaign match, winning still records progress
and returns you to the map. **Start Campaign** re-opens the map at your saved
run position. A profile from before this spec (or none) starts a fresh run.

## Design requirements

- **Full-screen and responsive.** The map fills the whole terminal (a
  deliberate departure from the centered fixed-block layout the board and menus
  use) and re-lays-out on resize. The global minimum terminal size still holds.
- **Consistent vocabulary.** Monochrome; node states use the existing emphasis
  tiers (cleared bright, current highlighted with the selection pulse, open
  normal, locked dim); the same nav synonyms; the same bordered/centered feel
  for the info panel.
- **Ornament stays background.** The twinkling starfield is slow, low-contrast,
  and strictly behind the scene — never on a node, route, or the panel — per
  the amended Motion rule.
- **No regression.** Quick Play reproduces today's Start Game → opponent-select
  behavior; Continue is unchanged; a player who never opens the campaign sees
  no change to the existing flows.
- **The engine stays campaign-agnostic.** Campaign context lives in the profile
  and the app, not in `GameState`; the map reuses the existing match-launch and
  win-detection paths.

## Acceptance criteria

- [x] The start menu shows **Start Campaign** and **Quick Play**; Quick Play
      behaves exactly like today's Start Game (opponent-select, discard-confirm
      over a save), and Continue is unchanged. *(driver: menu shows both; Quick
      Play → discard-confirm → the roster select screen.)*
- [x] Start Campaign opens a **full-screen** map: header band, planet nodes,
      connecting routes, a twinkling starfield, and a bottom info panel for the
      highlighted planet. *(driver, 180×48: fills the terminal; a star flipped
      `✦`→`·` between frames — the twinkle is live.)*
- [x] The cursor moves between **unlocked** planets (arrows / `wasd` / emacs);
      the info panel tracks the highlight; **Esc**/`x` returns to the menu.
      *(driver: cursor Ashfall→Drift, panel updated Vessa Korr→Old Toran; `x`
      returned to the menu.)*
- [x] Selecting an unlocked, un-cleared planet launches a match against its
      next un-beaten opponent, dealt from the built deck; an incomplete deck
      diverts to the deck-builder first. *(driver: Cinder → a match vs Greeb;
      the deck-valid guard mirrors `open_opponent_select`.)*
- [x] Winning records the opponent as beaten and returns to the map; clearing a
      planet (all its opponents beaten) unlocks its dependents. The first map's
      fork (Ashfall/Drift unlock after Cinder; The Spindle after both) works.
      *(unit: the fork/unlock/clear derivation; driver: a seeded Cinder-beaten
      profile rendered Cinder ● + the fork unlocked, "1/4 cleared". A live
      full-match win is a manual playtest — see the close-out note.)*
- [x] Losing returns to the map with the planet still open to retry. *(driver:
      lost a campaign match → back to the map, "0/4 cleared", Cinder still
      open, `in_progress` cleared.)*
- [x] The campaign run persists: travel/clear a planet, quit, relaunch — the
      run is as left. **Continue** resumes an in-progress campaign match and, on
      win, still records progress and returns to the map. *(unit: campaign
      round-trip; the `in_progress` pointer is persisted, so a resumed match
      routes to the map at game over; Continue path unchanged.)*
- [x] A pre-spec / missing profile begins a fresh run with only the start
      planet unlocked. *(unit: `a_fresh_run_unlocks_only_the_start`; the
      `campaign` field is serde-defaulted.)*
- [x] Unit tests cover the map graph (clear/unlock/next-opponent derivation and
      the fork), the run's persistence round-trip, and map navigation; the
      layout stays in-bounds at the minimum terminal. `cargo test` green (207
      passed), `cargo build` no new warnings.

## Resolved decisions

- **Front door = Continue + Start Campaign + Quick Play** (human-ruled).
  Continue resumes the in-progress match board; Quick Play is the retained
  opponent-select. **The discard-confirm lives at campaign entry** (human-ruled
  during playtest): choosing Start Campaign over a saved match prompts first,
  and Yes discards it then and there — so once on the map a launch never has a
  save to overwrite. Trade-off, accepted: peeking at the map and backing out
  still costs the saved match.
- **First map = multi-opponent planets + a light branch** using all 5 roster
  opponents (human-ruled). Cinder → {Ashfall, Drift} → The Spindle (two
  opponents). Grows with the roster/economy.
- **Full-screen, hybrid coordinate model** (human-ruled): a bird's-eye scatter
  that still trends rim→core, with a bottom info panel — legible where the eye
  follows a route, and a canvas that fills honestly.
- **Twinkling starfield allowed** via the Motion-principle amendment
  (`design/brief.md`, its own commit) — slow, low-contrast, background-only.
- **The map is a `Screen`, not an overlay** (per the constitution) — a full mode
  you navigate to; it supersedes opponent-select as *a* way opponents are chosen
  (opponent-select lives on as Quick Play).
- **Economy stubbed; a loss has no penalty** (human-ruled). Wins record progress
  only; credits/rewards are spec C, run-loss stakes are spec E.
- **Campaign context lives in the profile, not the engine.** A profile-side
  "in-progress node" pointer (persisted, so it survives Continue) marks the
  current match as a campaign match; `GameState` stays campaign-agnostic.
