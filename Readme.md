# Kaazap

### A familiar game for a solar system very, very near

A terminal-based Rust implementation of **Pazaak**, the card game from
*Star Wars: Knights of the Old Republic* — mostly faithful to the
original rules, with its own lightweight campaign layer standing in for
the RPG economy those rules originally leaned on.

Face a series of opponents with distinct personalities across a campaign
of increasing difficulty. Play matches to three round-wins — draw cards
toward 20 without busting, using your side-deck hand to push your total
up or down. Wins earn currency and card packs that unlock better
side-deck cards for tougher matchups ahead.

> **Status:** actively being built. The core Pazaak engine, a terminal
> UI overhaul, audio with a settings menu (music/SFX volume, a global
> mute), mid-match save/resume (a **Continue** on the menu), a **ten-opponent**
> roster with distinct difficulties, decks, and **board-aware AI**
> (they read your board and play to beat the round, each with its own
> strategy — up to a flawless final boss), a deck-builder for
> assembling your own side deck from a card collection, **an eight-world
> campaign map** — travel a node-based star map, Outer Rim → Core, clearing
> each world's opponents to unlock the next — and **a campaign economy**
> (wins earn credits and drop cards; a shop on the map sells from a pool that
> unlocks by how far core-ward you've reached) are in place; the balance pass
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

## How to play

The **Side Deck** menu item opens a deck-builder: browse the cards you own
and add or remove copies to assemble your **10-card side deck**, the deck
your hand is dealt from each match. Move over the grid with the arrows
(or `w`/`a`/`s`/`d`), **Enter** to add a copy of the highlighted card and
**Backspace** to remove one; your deck must be a full 10 cards to start a
match.

From the start menu, **Start Campaign** opens a full-screen star map: travel
between planets (arrows / `w`·`a`·`s`·`d`), and at each you play its opponents
to clear it and unlock the way core-ward. **Quick Play** instead lets you pick
any opponent from the roster directly (each has its own difficulty, side
deck, and play style; see `docs/opponents.md` for how difficulty is tuned). In-game, press `?`
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
