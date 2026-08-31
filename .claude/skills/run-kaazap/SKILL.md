---
name: run-kaazap
description: Launch and drive the kaazap TUI to see it running and verify changes in the real app. Use when asked to run the game, smoke-test a change, or check what a screen actually shows.
---

# Running kaazap

kaazap is a crossterm TUI. Its renderer cursor-positions every changed
cell individually (column-major, one char per `MoveTo`), so the raw
output stream never contains readable words — grepping a pty stream or
typescript is useless. You must interpret cursor movements into a grid.
On this machine tmux is not installed and macOS `screen` (4.00.03) has
flaky async `hardcopy`, so this skill ships its own driver.

## Run (interactive, for agents)

```bash
python3 .claude/skills/run-kaazap/driver.py [step ...]
```

Steps execute in order:

| Step | Meaning |
|---|---|
| `wait:TEXT[:SECONDS]` | Pump output until TEXT appears on screen (default timeout 15s) |
| `key:KEYS` | Send KEYS to the app (`\r` = Enter, `\e` = ESC — e.g. `\e[C` right arrow) |
| `pump:SECONDS` | Let the app run and absorb output for SECONDS |
| `resize:WxH` | Resize the pty (sends SIGWINCH); e.g. `resize:40x15`. Stay within 180x48. |
| `snap:LABEL` | Print the current screen grid, labeled |

With no args: waits for the menu and snapshots it. Exits nonzero if a
`wait` times out (prints a `*_TIMEOUT` snapshot first). The driver
builds/launches via `cargo run` from the repo root and kills the app
when the script ends.

Example — start a game, hit once, play hand card 1, look:

```bash
python3 .claude/skills/run-kaazap/driver.py 'wait:Start Game' 'key:\r' \
    'wait:Your Turn' snap:START key:d pump:3 snap:AFTER_HIT \
    key:1 pump:1.5 snap:AFTER_PLAY
```

Waiting on round resolution: outcome texts are `You won this round!`,
`Opponent won the round!`, `You Tied!` — `wait:round:20` catches the
first two; there is no common substring across all three.

## Key reference (in game)

| Key | Action |
|---|---|
| `1`–`4` | Play hand card at that slot |
| `\e[C` / `\e[D` | Cursor select next/prev hand card (right/left arrow) |
| `\e[A` / `\e[B` | Toggle a ±/tiebreaker card's sign (up/down arrow) |
| `\r` | Confirm the cursor-selected card (Enter) |
| `d` / space | Hit (draw a dealer card) |
| `s` | Stand |
| `n` | Next round (when prompted) |
| `g` | New game (after game over) |
| `x` | Back to menu |
| `?` | Toggle help overlay |

Menu: arrows/`w`/`s` navigate, Enter/space select.

## Quirks

- Opponent turns take ~1s each (`OPPONENT_THINKING_TIME_MS`); after an
  action that hands them the turn, `pump:3` before snapshotting.
- Minimum terminal 89×31 (`Config::from_terminal` errors below that at
  startup); the driver uses 180×48.
- The app handles live resize (spec 002): the `resize:WxH` step works
  mid-run, and below the minimum it shows a "too small" recovery screen,
  restoring when the terminal grows back.

## Run (direct, for humans)

```bash
cargo run
```

`x` to the menu; Ctrl-C to quit.
