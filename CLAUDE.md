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
Every campaign match is played for a stake the player chooses: a win
pays it back double, a loss forfeits it, and going broke ends the run
and resets it. Credits buy better side-deck cards in the shop. It's a
personal project, built with the intent of eventually being presentable
enough to share (portfolio, itch.io).

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
approve technical work: `plan.md` and `tasks.md` are drafted by the
`sdd-planner` and signed off by the `skeptical-reviewer`, each phase
(and any task the planner marked for its own review) is reviewed by
the `skeptical-reviewer` rather than the person, and what reaches the
person is a spec-conformance summary, not an architecture review.
Implementation pauses after each phase unless the person says to run
further, and whenever something unexpected bears on spec adherence.

## Model policy

- **Tiers by name**: top tier `fable`; implementation tier `opus`;
  session tier `fable` at medium effort (experiment 1 — the top and
  session tiers are the same model at different effort; the fallback
  session model is `claude-opus-4-8`, the full ID, since a
  previous-generation model has no short alias). These names are the
  only place a model is spelled out; everything below refers to the
  roles.
- **The session runs at the session tier, at medium effort**, set in
  this repo's `.claude/settings.json` — written at project setup from
  the skill's `assets/settings-template.json` (`"model":
  "claude-fable-5-1"`, `"effortLevel": "medium"`, and a level under
  `"modelSettings"` for each tier's full model ID). If that file is missing or lacks these
  keys, recreate it from the template and commit it before dispatching
  anything; nobody creates it by hand. Project settings outrank user
  settings, so a model picked in the app's picker only affects the
  session it was picked in — new sessions in this repo start here
  regardless. The app's effort indicator may show the model's default
  rather than the level in effect; `/effort status` inside the session
  is the authoritative check. The orchestrating session takes many
  bookkeeping turns and re-sends its whole context on each one — the
  dominant cost of the workflow — and it makes no design decisions: it
  assembles bundles, dispatches, verifies, commits, and reports. The
  role never needs the top tier; it sits on the top tier's model
  under experiment 1 because Fable 5.1's cache-read rate makes the
  seat's re-sends cost about what they would on Opus, and this spec
  measures the allowance draw and the readability of the reports.
  If it drops the protocol (a skipped review, a stale `tasks.md`
  edit, a task done by hand), the first fix is high effort, one line
  in the same file.
- **The session tier never resolves a design question.** When triage
  finds a task that isn't routine, the session frames the question in
  Plan Mode — so nothing is touched meanwhile — and dispatches the
  `skeptical-reviewer` at the top tier on a decision bundle: the task
  line, the plan section, the acceptance criteria, and the options as
  the session sees them. It transcribes the recommendation into
  `plan.md` and dispatches what remains. A product question `spec.md`
  doesn't settle goes to the person instead.
- **Everything the person reads is plain language.** Pause reports,
  spec-conformance summaries, and questions use short sentences and
  everyday words — no task IDs, agent names, tier names, or internal
  shorthand unless the person asks — and assume the reader won't open
  `plan.md`. Say what can now be tried, where execution deviated from
  the spec and why, and what needs a decision. Technical detail
  belongs in `plan.md` and the commit log, not in the report.
- **What the person's walkthrough finds is a finding, not a task
  line.** When the person reports at a phase pause that something is
  wrong, the session restates it — which acceptance criterion, what
  they saw, what the spec says — and dispatches a diagnosis bundle to
  the `sdd-implementer` (the report, the restatement, the task line,
  the plan section, the acceptance criterion, the files). The
  implementer finds the cause and fixes it if the fix is routine and
  inside the footprint; otherwise it returns the diagnosis and
  options, which go to a decision review at the top tier. A fix is
  logged as a sub-lettered task; a finding that is really the spec
  being ambiguous goes back to the person as a product question. The
  session never diagnoses in place.
- **The top tier runs only inside the decisions**: the `sdd-planner`
  (one dispatch per spec) and the `skeptical-reviewer` on plan/tasks
  sign-off and on decision reviews — each dispatched with an explicit
  per-call override to the top tier's name. The three agent
  definitions carry `effort: high`, which overrides the session's
  medium, so reasoning stays at full strength where it matters.
- **Spec conversations happen in a Claude Code spec session of their
  own**, at the top tier, never inside an implementation session. The
  spec session also runs planning once `spec.md` is approved — the
  planner dispatch, the sign-off, the spec-conformance summary — and
  ends when `plan.md` and `tasks.md` are final, with a new session
  (not `/clear`, which keeps the model) whose opening prompt is the
  spec session's last message. A session in this repo opens at the
  session tier — the top tier's model at medium — so a spec session
  states its model and effort first (`/effort status`) and asks the
  person to raise effort to high for this session (`/effort high`)
  before continuing. The next session opens at medium again from
  `.claude/settings.json`. (The project's very first spec, with no
  codebase yet, happened in chat.)
- **The `skeptical-reviewer` runs at the implementation tier by
  default** (its definition says `opus`) for per-phase reviews, the
  per-task reviews the planner marks, and the pre-merge sweep. Each
  review gets a single bundle file assembled with shell — diff, task
  lines, plan sections, acceptance criteria; for the sweep, the
  documents and the spec's full diff — and reads nothing else.
- **Review loop cap**: one review and at most one re-review per
  invocation — task, phase, sign-off, or sweep. The re-review sees the
  findings and the fix diff only. Blocking
  means it would fail an acceptance criterion or a test, or contradicts
  `plan.md` or `CLAUDE.md`; nothing else blocks. Anything open after
  the re-review goes to the tier log and the sweep.
- **Implementation runs at the implementation tier**, in the
  `sdd-implementer` subagent (its definition says `opus`), one task
  per dispatch, sequentially. The orchestrating
  session triages each task, dispatches routine ones on a task bundle
  assembled with shell (task line, plan section, acceptance criteria,
  files, the pattern file to copy), and on return verifies with the
  verification command below — re-run by the orchestrator for tasks
  marked `review: per-task`, taken from the implementer's verbatim
  output otherwise — never by re-reading the diff. Only the
  orchestrator edits `tasks.md` or commits, and the orchestrator never
  implements second-look notes or does device or browser checks by
  hand.
- **One implementation session per spec.** It opens when `plan.md` and
  `tasks.md` are final and ends at the merge; a phase pause is a pause
  in it, not a boundary — the person attests and says continue.
  `/compact` if the context grows large; never clear or compact
  mid-task. `/clear` is not part of the workflow: both session
  boundaries are new sessions.
- **Every session-ending pause ends with a continuation prompt.** When
  the next step belongs in a fresh session — plan and tasks final, a
  merge with the next spec waiting on `ROADMAP.md`, or a phase pause
  the person is stopping at — the report's last item is the exact
  prompt to paste there, in its own fenced block. It names the spec directory,
  the files to read, where to resume, the involvement level, the
  pause cadence, and any effort switch the next session needs. Write
  anything the next session needs to a file first; the prompt points
  at files. If nothing can proceed until the person decides
  something, say so instead.
- **Batch the bookkeeping**: commit, checkbox, and tier-log row in one
  shell command; bundle assembly and dispatch back to back. Every turn
  saved is one fewer re-send of the whole context.
- **Fallback**: if the top tier's usage budget runs out, dispatch the
  planner and sign-off at the implementation tier for the rest of the
  window (drop the override; both definitions default to `opus`), and
  switch the session itself to `claude-opus-4-8` mid-session
  (`/model claude-opus-4-8` — one cache re-write, then continue).
  Nothing else changes; the tier log records what ran and when the
  switch happened, which is a result of experiment 1 in itself.
- **Escape hatch**: two failed verifications on one task, or a "stopped
  on a judgment call" the orchestrator considers well-specified, and
  the orchestrator does that task itself, noting the
  miss in `tasks.md`.
- **Lighter implementer**: off. <!-- Turn on per project once the
  first spec's tier log justifies it: "the session tier for tasks with
  an automated Verify check, a named pattern file, and a small
  footprint." -->
- **Log token usage per implementer run and per reviewer invocation**,
  plus tier misses, in `tasks.md`'s tier log for the first spec under
  this policy, and compare against a previous spec before treating the
  policy as settled.
- **History (this project)**: the session tier was `sonnet` until
  2026-09-07 (orchestrator at the step-down tier, third tier off), moved
  to `claude-opus-4-8` on 2026-09-09, and to `fable` at medium under
  experiment 1 on 2026-09-11; the `sdd-implementer` has always run at
  `opus`. Spec 021 is the first spec under experiment 1 — its tier log
  in `specs/021-wager-and-loss/tasks.md` carries the allowance reading
  and the per-invocation token counts to compare against specs 019–020.

## Spec-driven workflow

This project follows spec → plan → tasks → implement, gated by review
between each phase — the person's or the `skeptical-reviewer`'s, per
the involvement level above. Artifacts live in `specs/<NNN>-<slug>/`:

- `spec.md` — what and why, user-facing behavior, acceptance criteria,
  explicit non-goals. No implementation detail.
- `plan.md` — technical design: types, data flow, what changes where.
- `tasks.md` — ordered, small, independently verifiable tasks.

Authorship: `spec.md` is written in the chat design conversation.
Until this project has shipped code, `plan.md` and `tasks.md` are too;
once shipped code is what plans extend, the `sdd-planner` subagent
drafts them instead — at the top tier, from a planning bundle, against
the actual codebase — and the orchestrator commits them to the spec
branch with the PR still in draft. Both are signed off before any
implementation task starts: at the product-owner level by the
`skeptical-reviewer` (blocking findings fixed and re-reviewed), with
the person receiving a spec-conformance summary to approve; at the
technical-lead level by the person directly.

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

The verification command for this project is:

    cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25

After any implementation task, Claude Code must run that command and
report its actual output, not a paraphrase.

A task is not complete until that output is green. Do not weaken, skip,
or delete a test to make it pass — if a test seems wrong, flag it and
ask. When the task was dispatched to the `sdd-implementer`, its verbatim
output is the verification; for a task marked `review: per-task` the
orchestrator re-runs the command itself before committing.

## Git conventions

- **One branch per spec, not per task or phase.**
- **Never commit directly to `main`** for spec-specific work. Repo-wide
  files (`CLAUDE.md`, `ROADMAP.md`, `DECISIONS.md`) commit straight to
  `main`; spec-specific files ride into `main` only when the spec merges.
- Open the PR as a draft immediately after pushing the branch, for a
  running diff. Only mark it ready and merge once every task in the
  spec's `tasks.md` is complete and verified.
- **Keep merged branches — never delete them.** A merged spec or chore
  branch stays as a historical artifact: merge without `--delete-branch`
  (the repo does not auto-delete head branches), so each branch's own
  commit history is preserved for a project meant to demonstrate the
  workflow. (Spec branches 010–015 were deleted at their merges before
  this rule, then restored from the merge commits' second parents, so
  the full 001–015 branch history is intact; the rule holds going
  forward.)
- Keep AI co-authorship attribution on commits — accurate, and worth
  keeping for a project meant to demonstrate this workflow.
- Never force-push.

## Commits

- One commit per completed task where practical, referencing the task ID
  — made by the orchestrating session after its own verification, never
  by the implementer subagent.
- Commit messages describe what changed and why, not "implement task 3".
