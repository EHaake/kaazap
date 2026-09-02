# Kaazap

### A familiar game for a solar system very, very near

A terminal-based Rust implementation of **Pazaak**, the card game from
*Star Wars: Knights of the Old Republic* — mostly faithful to the
original rules, with its own lightweight campaign layer standing in for
the RPG economy those rules originally leaned on.

Face a series of opponents with distinct personalities across a campaign
of increasing difficulty. Play best-of-3-round matches — draw cards
toward 20 without busting, using your side-deck hand to push your total
up or down. Wins earn currency and card packs that unlock better
side-deck cards for tougher matchups ahead.

> **Status:** actively being built. The core Pazaak engine, a terminal
> UI overhaul, audio with a settings menu (music/SFX volume, a global
> mute), and mid-match save/resume (a **Continue** on the menu) are in
> place; opponent personalities, campaign progression, and campaign-level
> persistence are next — see `ROADMAP.md` for what's shipped and what's
> ahead.

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

In-game, press `?` for a rules and controls overlay. If you know real
Pazaak, Kaazap plays close to the source material with a few intentional
tweaks — see `DECISIONS.md` for what's changed and why.

## Development

This project is built using spec-driven development with Claude:

- `CLAUDE.md` — architecture, conventions, and workflow constitution
- `ROADMAP.md` — the (unordered) feature backlog
- `DECISIONS.md` — the reasoning behind naming, scope, and process calls
- `specs/` — per-feature spec → plan → tasks docs

## Acknowledgments

Kaazap is an unofficial, non-commercial fan project inspired by Pazaak
as it appears in *Star Wars: Knights of the Old Republic*. It is not
affiliated with, endorsed by, or sponsored by Lucasfilm, Disney, BioWare,
or Aspyr.

Bundled music is licensed under Creative Commons Attribution — see
`assets/CREDITS.md` for the required attribution. Sound effects are
generated from scratch (`scripts/gen_sfx.py`) and carry no third-party
licensing.
