# Kaazap

### A familiar game for a solar system very, very near

A terminal-based Rust implementation of **Pazaak**, the card game from
*Star Wars: Knights of the Old Republic* — mostly faithful to the
original rules, with its own lightweight campaign layer standing in for
the RPG economy those rules originally leaned on.

Face a series of opponents with distinct personalities across a campaign
of increasing difficulty. Play matches to three round-wins — draw cards
toward 20 without busting, using your side-deck hand to push your total
up or down. Every campaign match is played for a **stake** you choose: a
win pays it back double, a loss costs it, and winnings buy better
side-deck cards for tougher matchups ahead. Run out of credits and the
run is over.

> **Status:** actively being built. The core Pazaak engine, a terminal
> UI overhaul, audio with a settings menu (music/SFX volume, a global
> mute), mid-match save/resume (a **Continue** on the menu), a **ten-opponent**
> roster with distinct difficulties, decks, and **board-aware AI**
> (they read your board and play to beat the round, each with its own
> strategy — up to a flawless final boss), a deck-builder for
> assembling your own side deck from a card collection, **an eight-world
> campaign map** — travel a node-based star map, Outer Rim → Core, clearing
> each world's opponents to unlock the next — and **a campaign economy**
> (stake credits on every campaign match — a win pays double, a loss costs the
> stake, and going broke ends the run in a full reset; a shop on the map sells
> from a pool that unlocks by how far core-ward you've reached), and **opponent portraits** (a
> low-resolution monochrome face for every opponent, shown beside the board
> in-match and in the select/map previews), and **a Records overlay** (a
> read-only "mastery" popup off the start menu — like How to Play / Settings —
> showing per-opponent match/round win-loss, win streak, campaign completions,
> and collection progress, split Quick Play / Campaign / overall / this-run) are
> in place; the balance pass
> and stretch features are what's ahead — see `ROADMAP.md` for the full picture.

## Building & running

```
cargo build
cargo run
```

Requires a stable Rust toolchain (2024 edition). On Linux, the ALSA
development libraries are required to build (a dependency of `rodio`, the
audio library used for the music and sound effects):

```
# Debian/Ubuntu
sudo apt install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel
```

**Terminal size:** Kaazap needs a terminal at least **139 × 31** (columns ×
rows) — the fixed board plus the always-visible opponent-portrait panel beside
it. Below that it shows the required size and exits rather than rendering
broken; enlarge the terminal and run again.

## How to play

The **Side Deck** menu item (or **`c`** from the campaign map) opens a two-panel
"briefcase" deck-builder: your **Collection** on the left, your built **Deck** on
the right, each an album of every card type — the ones you have shown as solid
cards with a copy count, the rest — those absent from that panel — as faint
placeholders. Move the
selection with the arrows (or `w`/`a`/`s`/`d`), **Tab** to switch panels, and
**Enter** to move a copy across — adding one from the Collection or returning one
from the Deck. Your side deck (the 10 cards your hand is dealt from each match)
must be a full 10 to start a match.

From the start menu, **Start Campaign** opens a full-screen star map: travel
between planets (arrows / `w`·`a`·`s`·`d`), and at each you play its opponents
to clear it and unlock the way core-ward. Launching a match opens a **wager
prompt** — pick your stake (arrows) above the opponent's minimum ante and
**Enter** to commit; win and it comes back doubled, lose and it's gone. A world
you've already cleared stays open for **rematches** against its final opponent,
so you can grind small, safe bets to fund a card. If your credits ever drop
below the cheapest ante on the map, the **run is over**: a full reset to the
starter deck and a fresh purse. Once you've cleared any world, Start
Campaign first asks whether to **Continue** your run or begin a **New Campaign** —
a fresh start that resets your progress, credits, and collection (your settings
are kept). **Quick Play** instead lets you pick
any opponent from the roster directly (each has its own difficulty, side
deck, and play style; see `docs/opponents.md` for how difficulty is tuned). The **Records** menu item
opens a read-only popup of your play history — matches and rounds won/lost per
opponent, your win streak, campaign completions, and collection progress —
paged left/right across Overall, Quick Play, Campaign, and This Run views. In-game, press `?`
for a rules and controls overlay. If you know real Pazaak, Kaazap plays close
to the source material with a few intentional tweaks — see `DECISIONS.md` for
what's changed and why.

## Development

This project is built using spec-driven development with Claude:

- `CLAUDE.md` — architecture, conventions, and workflow constitution
- `ROADMAP.md` — the (unordered) feature backlog
- `DECISIONS.md` — the reasoning behind naming, scope, and process calls
- `specs/` — per-feature spec → plan → tasks docs
- `docs/` — reference notes (e.g. `docs/opponents.md` — the opponent
  roster and difficulty tuning)

## Acknowledgments

Kaazap is an unofficial, non-commercial fan project inspired by Pazaak
as it appears in *Star Wars: Knights of the Old Republic*. It is not
affiliated with, endorsed by, or sponsored by Lucasfilm, Disney, BioWare,
or Aspyr.

Bundled music is licensed under Creative Commons Attribution — see
`assets/CREDITS.md` for the required attribution. Sound effects are
generated from scratch (`scripts/gen_sfx.py`) and carry no third-party
licensing.
