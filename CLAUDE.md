# Project Constitution

This file is the standing contract for how this codebase is built. It loads
into every Claude Code session automatically. Specs and plans must not
contradict it; if a spec needs to, the constitution gets amended first,
explicitly, in its own commit.

## What this project is

Kaazap is a terminal-based Rust implementation of Pazaak, the card game
from *Star Wars: Knights of the Old Republic* — mostly faithful to the
original rules, with a bespoke campaign layer standing in for the RPG
context those rules originally leaned on. The core loop: the player faces
a series of opponents with distinct personalities and increasing
difficulty, playing first-to-3-round-wins Pazaak matches (draw dealer cards
toward 20 without busting, using side-deck cards to adjust your total).
Wins earn currency and/or card packs that unlock better side-deck cards
for future matches. It's a personal project, built with the intent of
eventually being presentable enough to share (portfolio, itch.io).

## Simplicity

This is a small personal project, not an enterprise production codebase.
Prefer the simplest design that satisfies the spec: fewer types, fewer
abstractions, no speculative generality. Readability and maintainability
beat cleverness and future-proofing. If a plan or implementation grows
indirection the spec doesn't demand, that's a smell — cut it. This is a
standing instruction to Claude Code as much as a note to the human.

## Platform

- **Target**: Linux, macOS, Windows terminals via `crossterm`. No specific
  minimum terminal size beyond what `Config::from_terminal` already
  enforces (errors out if the terminal is smaller than the layout needs).
- **Rendering**: a custom double-buffered `Frame` (`Vec<Vec<char>>`),
  diffed and drawn through `crossterm` on a dedicated render thread
  separate from the game loop thread. This is an established pattern in
  `main.rs`/`render.rs` — keep it. Do not introduce a TUI framework
  (ratatui, etc.); that would mean rewriting the whole rendering layer
  for a mostly-cosmetic win.
- **Language**: Rust, 2024 edition, stable toolchain.

## Architecture

- `App` (`app.rs`) owns a `Screen` enum and routes input to whichever
  screen is active. `Screen::StartMenu` and `Screen::InGame` each own
  their own state (`MenuState`, `GameState`). New top-level modes
  (campaign map, shop/pack-opening) should become new `Screen`
  variants — don't bolt them onto existing ones. *Menu sub-panels*
  (How to Play, Settings), by contrast, are overlays shown over
  `StartMenu` — `App` holds their transient state and routes input to
  them while open, and the underlying menu (and its selection) is
  preserved. The line: a full mode the player navigates *to* is a
  `Screen`; a panel opened *over* the menu and dismissed back to it is
  an overlay. (Settings was a `Screen` in spec 004 and moved to an
  overlay in spec 004's UI pass, for consistency with How to Play.)
  Each `Screen` module follows the same shape: a small state struct (its
  cursor/selection), a `handle_input(key, …) -> Option<Outcome>` returning
  **one** owned outcome enum (not an action/event split), and a
  `draw(frame, config, …, pulse)` — wired into the three `match &self.screen`
  arms in `app.rs` (input, `?`-help, draw). `opponent_select.rs` is the
  reference; a new screen copies it rather than inventing a new shape.
- Game logic (`game.rs`, `player.rs`, `card.rs`) stays decoupled from
  rendering (`board.rs`, `frame.rs`, `render.rs`). State mutation always
  goes through `apply_*_action` methods that centralize validation;
  drawing code never mutates state. Preserve this boundary as campaign,
  save/load, and audio are added.
- `GamePhase` already drives the core state machine
  (`PlayerTurn` → `OpponentThinking` → `OpponentTurn` → `RoundEnd` →
  `AwaitingNextRound` → `GameOver`). Extend it for new phases (pack
  opening, shop) rather than adding parallel ad hoc flags.

## Testing

- Game logic — scoring, round/game resolution, card effects, opponent
  decision-making — needs unit test coverage going forward. This is new
  discipline for the project, not a retroactive requirement for every
  existing line, but new or changed logic in this area ships with tests.
- Rendering/frame code (`board.rs`, `render.rs`, `frame.rs`) is lower
  priority for unit tests, since it's mostly terminal side-effect code —
  verify that by actually running it, not by writing tests that mostly
  assert against a mock terminal.
- A task is not complete until `cargo test` passes — run it, report the
  actual output, don't paraphrase or assume.

## Dependencies

- Default: no new third-party crates without discussing it first. The
  existing set (`crossterm`, `rand`, `anyhow`) is already earning its place.
  (`strum`/`strum_macros` were dropped in spec 005 when the menu refactor
  removed the only `EnumIter` use.)
- Audio uses `rodio` (spec 004). `rusty_audio` was dropped: it's
  fire-and-forget SFX with no looping or pause, and running it alongside
  rodio would mean two output streams contending for the device — a
  single rodio backend (one output stream, a looping music sink, detached
  sinks for overlapping SFX) is cleaner. Linux builds need ALSA dev
  libraries installed to compile rodio — documented in the README.
- Persistence (save/resume, campaign state, currency, unlocked cards):
  `serde` + `serde_json` for the save format — human-inspectable while
  debugging — and the `directories` crate for a proper cross-platform
  save location, rather than hand-rolling either.

## Project file safety

- `Cargo.lock` is committed as-is (already the case) and only
  regenerated via `cargo build`/`cargo update` — never hand-edited.

## Involvement level

**Product owner.** The person owns `spec.md`, attests to behavior by
using the app at phase pauses, and decides escalations. They do not
approve technical work: `plan.md` and `tasks.md` are signed off by Plan
Mode plus the `skeptical-reviewer`, foundational tasks are reviewed by
the `skeptical-reviewer` rather than the person, and what reaches the
person is a spec-conformance summary, not an architecture review.
Implementation pauses after each phase unless the person says to run
further, and whenever something unexpected bears on spec adherence.

## Spec-driven workflow

This project follows spec → plan → tasks → implement, gated by review
between each phase — at the **product-owner** involvement level above,
those gates are the `skeptical-reviewer`'s, not the person's. Artifacts
live in `specs/<NNN>-<slug>/`:

- `spec.md` — what and why, user-facing behavior, acceptance criteria,
  explicit non-goals. No implementation detail.
- `plan.md` — technical design: types, data flow, what changes where.
- `tasks.md` — ordered, small, independently verifiable tasks.

Authorship: `spec.md` is written in the chat design conversation; since
this project already has shipped code, Claude Code drafts `plan.md` and
`tasks.md` in Plan Mode, against the actual codebase, committed to the
spec branch with the PR still in draft. Both are signed off before any
implementation task starts by Plan Mode plus the `skeptical-reviewer`
(blocking findings fixed and re-reviewed); the person receives a
spec-conformance summary to approve, not the plan itself.

Do not begin implementation on a feature without an approved spec and
plan in that feature's directory. When resuming a session, check
`specs/<feature>/tasks.md` for current state before doing anything else.

## Collaboration workflow

If the `spec-driven-development` skill is installed
(`~/.claude/skills/spec-driven-development/` or a project-level
`.claude/skills/`), its collaboration workflow applies automatically —
routine tasks proceed normally, real decisions resolve via Plan Mode and
the `skeptical-reviewer` subagent, and beyond the phase pauses defined
above, the person is looped in only when something in the design turns
out infeasible or needs real rework, or a previously-unknown
consideration surfaces that would materially change the project's
direction. Nothing needs to be repeated here.

## Verification

After any implementation task, Claude Code must:

1. Build the project (`cargo build`).
2. Run the test suite (`cargo test`).
3. Report the actual pass/fail output, not a paraphrase.

A task is not complete until steps 1–2 are green. Do not weaken, skip, or
delete a test to make it pass — if a test seems wrong, flag it and ask.

## Git conventions

- **One branch per spec, not per task or phase.**
- **Never commit directly to `main`** for spec-specific work. Repo-wide
  files (`CLAUDE.md`, `ROADMAP.md`, `DECISIONS.md`) commit straight to
  `main`; spec-specific files ride into `main` only when the spec merges.
- Open the PR as a draft immediately after pushing the branch, for a
  running diff. Only mark it ready and merge once every task in the
  spec's `tasks.md` is complete and verified.
- Keep AI co-authorship attribution on commits — accurate, and worth
  keeping for a project meant to demonstrate this workflow.
- Never force-push.

## Commits

- One commit per completed task where practical, referencing the task ID.
- Commit messages describe what changed and why, not "implement task 3".
