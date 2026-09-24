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
- **The acted-on element stands apart.** In a modal, prompt, or panel,
  the one element the player acts on (a stake, a selection, a confirm
  line) gets an empty row above and below it; the rest of the text stays
  compact. This is the design brief's *Density and breathing room* rule
  (added 2026-09-13 after the wager prompt shipped dense; corrected the
  same day when the first application over-padded every row — the person
  ruled that only the acted-on line gets air). The modal box itself pads
  its content evenly — one empty row above and below, never a slab of
  empty rows under the text (`OverlayLayout`). A new or changed screen is
  checked against both at review, not after the person plays it.

## Architecture

- `App` (`app.rs`) owns a `Screen` enum and routes input to whichever
  screen is active. `Screen::StartMenu` and `Screen::InGame` each own
  their own state (`MenuState`, `GameState`). New top-level modes
  (campaign map, shop) should become new `Screen`
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
  `AwaitingNextRound` → `GameOver`). Extend it for new phases rather
  than adding parallel ad hoc flags.

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
Implementation pauses on the cadence set below, and whenever something
unexpected bears on spec adherence.

## Pause cadence

<!-- A separate question from the involvement level, asked at the same
time and answered here. Involvement level says who approves technical
work; this says how often the build stops for the person. Changing it
later is one word on this line plus a tier-log row, same as a role
table row — and it can change mid-spec. Adopted 2026-09-20, after the
skill made cadence its own setting on 2026-09-19; this project ran
"every phase" from its first spec until then, which is what specs
001-028 did. -->

**When there's something to try** (the skill's default; this project's
since 2026-09-20, from spec 029 onward).

The three values:

- **When there's something to try.** Pause after a phase the planner
  marked with a walkthrough, and run straight through the phases it
  marked `walkthrough: none`. A foundational phase that adds a type,
  a shared helper, or a test harness changes nothing the person can
  observe, so a pause there asks them to attest to something they
  cannot see — which is a gate in appearance only.
- **Every phase.** Pause after each one regardless. The most
  conservative setting, and the right one for a project whose phases
  are hard to tell apart from outside, or early in a project where
  nobody trusts the marking yet.
- **Only when blocked.** No phase pauses at all. The spec runs to the
  pre-merge sweep and the person does one walkthrough at the end,
  against the accumulated list. The most hands-off setting, and the
  one that costs the most to unwind if something went wrong early.

**The marking belongs to the planner.** From spec 029 onward the
`sdd-planner` marks **every** phase header `walkthrough: <what the
person can try>` or `walkthrough: none — <why>`, and marks honestly
rather than generously: marking everything as a walkthrough to be safe
reproduces exactly the problem the setting exists to fix. The call
belongs at planning time because the planner sees the whole spec at
once, while an orchestrator mid-phase cannot tell a phase with nothing
to show from one it has merely failed to describe. The test is
observability: does this phase change something the person could see
by using the app? A type, a shared helper, a test harness, an internal
refactor — no. A screen, a flow, different behavior — yes.

**Blockers pause under every cadence, including "only when blocked".**
These are not phase pauses and are never skipped:

- Either escalation trigger: something in the design turns out
  infeasible or needs real rework, or an unknown surfaces that would
  materially change the project's direction.
- A product question `spec.md` doesn't settle.
- The escape hatch firing twice on one task, which means the task
  list itself is wrong.
- A sweep or review finding that can't be resolved without changing
  what the spec promised.

**Nothing skipped is dropped.** A phase that runs without a pause
still gets its `skeptical-reviewer` phase review — what is skipped is
the person's attestation, not the check. And every unpaused phase
appends its walkthrough items — or, if it had none, its one-line
reason — to a running **walkthrough list in `tasks.md`**, and that
list is what the person walks through at the close-out. Skipping a
pause defers the person's check; it does not remove it. The tier log
records which phases ran unpaused, so a defect found late can be
traced to the phase that introduced it.

**The cost, stated plainly, because this is a trade.** A problem
introduced in phase one and found at the merge is more expensive to
unwind than the same problem caught in phase one. Fewer pauses buy
autonomy with late detection. Spec 028 is the evidence on both sides:
its Phase 1 walkthrough caught a garbled crash report in twelve runs
out of twelve, which a deferred check would have found four phases
later — and its Phases 2 and 3 were run straight through under the old
cadence anyway, because there was nothing to see, which is the case
this setting generalizes.

## Model policy

**Fable profile** (this project's until 2026-09-19). Top tier
`fable`; implementation tier `opus`; session tier `claude-fable-5-1`
at medium effort (the top and session tiers are the same model at
different effort). The fallback session model is `claude-opus-5-5`.
Settings from the skill's `project/.claude/settings.json`.

**Opus profile** — **this project's, since 2026-09-19**, when the
person said to use Opus for everything from now on; it stands until
they say otherwise, and nobody infers its end from an allowance reading
or from a sign-off that keeps finding problems. Opus for everything:
top tier `opus`; implementation tier `opus`; session tier
`claude-opus-5-5` at medium effort. Every dispatch runs at high effort
— the agent definitions' own default — and only orchestration runs at
medium. Nothing runs on Fable: the planner and sign-off dispatches
carry no override, the close-out goes to `sdd-implementer`, and the
top-tier fallback below never applies. Settings from the skill's
`project/.claude/settings.opus.json`. (The skill called these the
standard and economy profiles until 2026-09-23; renamed here on
2026-09-24 to match. `claude-opus-4-8` names no current role or
fallback — it appears only in the history below, or if the person asks
for it.)

These names are the only place a model is spelled out; everything
below refers to the roles.

### The role table

Every dispatch in this project resolves here. Cells hold tier names,
never model IDs, so switching profile re-points every row at once and
the three names above stay the only place a model is spelled out.
"Override" means the orchestrator passes a per-call model override on
that dispatch; without one, the agent's own frontmatter applies, and
every agent definition defaults to the implementation tier except
`sdd-implementer-fable`, which pins the top tier's model at medium.

| Role | Dispatched as | Model | Effort |
|---|---|---|---|
| Spec conversation | the spec session itself | session tier | high (raised per session) |
| Plan and tasks draft | `sdd-planner` | implementation tier (no override) | high |
| Plan and tasks sign-off | `skeptical-reviewer` | **top tier** (no override) | high |
| Decision review | `skeptical-reviewer` | **top tier** (no override) | high |
| Task implementation | `sdd-implementer` | implementation tier | high |
| Close-out task | `sdd-implementer` | top tier | high |
| Per-task and phase review | `skeptical-reviewer` | implementation tier | high |
| Pre-merge sweep | `skeptical-reviewer` | implementation tier | high |
| Orchestration and bookkeeping | the session itself | session tier | medium |

**Moving a role.** Edit its row, nothing else. To step a role down,
replace **top tier** (override) with "implementation tier (no
override)"; the dispatch then carries no override and the agent runs
at its own default. To change the implementer, change the agent name
in that row — both definitions stay installed, so it is a word, not a
reinstall. Under the Opus profile — where this project is now — the
top tier *is* the implementation tier, so every override column reads
"no override" and the close-out row reads `sdd-implementer` at high;
moving back to the Fable profile re-points those cells and nothing else
in the table changes. The plan-and-tasks draft row reads
"implementation tier (no override)" under both profiles: the planner
moved down as a **trial**, adopted here 2026-09-24 from the skill's
revision of the same date and starting with spec 031 — the sign-off
stays at the top tier and checks the draft. The trial's tier logs
record what the planner cost and what the sign-off found; the row
moves back up only if the person says so.

**A change the person asks for gets written here before it is acted
on.** If they say to move a role — for one window, for this project,
for good — edit the row, note it in the current spec's tier log with
the date and which spec it changed at, and commit, in the same turn,
*before* the next dispatch. Then do it. A model preference that lives
only in a session's context is gone at the next session boundary and
the next session will not know it ever existed, which is the one
failure this whole repo-as-interface arrangement exists to prevent.
If the person frames it as temporary ("while my allowance is low"),
write the row with the condition and the date in the comment, so
whoever reads it next knows when it stops applying and can ask. Never
infer the end of a temporary change and revert it unasked.

- **The session runs at the session tier, at medium effort**, set in
  this repo's `.claude/settings.json` — written at project setup from
  the profile's settings file in the skill's `project/.claude/`
  (`settings.json` for the Fable profile, `settings.opus.json` for
  Opus) (`"model"` set to the session tier's full ID, `"effortLevel":
  "medium"`, and a level under `"modelSettings"` for each tier's full
  model ID). If that file is missing or lacks these
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
  role never needs the top tier. Under the Fable profile it sits on
  the top tier's model; measured in this seat, Fable 5.1 at medium ran
  close to Opus 5.5 per task while taking about a third of the turns
  (the skill's design record has the numbers). Under the Opus profile
  it sits on Opus 5.5, whose reports read clearly to the person, and
  the same discipline about turns applies.
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
  line.** The person reports something wrong, at a phase pause or at
  the close-out walking the list the unpaused phases left. The session
  restates it — which acceptance criterion, what they saw, what the
  spec says — and dispatches a diagnosis bundle to
  the implementer the role table names (the report, the restatement,
  the task line, the plan section, the acceptance criterion, the
  files). The implementer finds the cause and fixes it if the fix is
  routine and inside the footprint; otherwise it returns the diagnosis
  and options, which go to a decision review at the top tier. A fix is
  logged as a sub-lettered task; a finding that is really the spec
  being ambiguous goes back to the person as a product question. The
  session never diagnoses in place.
- **The top tier runs only where the role table says it does**: by
  default the `skeptical-reviewer` on plan/tasks sign-off and on
  decision reviews, and the close-out dispatch (the `sdd-planner` sat
  here too until the 2026-09-24 trial moved it down). Under the Fable
  profile the first two carry an explicit per-call override to the
  top tier's name; drop the override and the definition's own
  implementation tier applies, which is exactly what stepping one of
  those rows down means. The agent definitions carry
  `effort: high`, which overrides the session's medium, so reasoning
  stays at full strength where it matters.
- **Spec conversations happen in a Claude Code spec session of their
  own**, at the top tier, never inside an implementation session. The
  spec session also runs planning once `spec.md` is approved — the
  planner dispatch, the sign-off, the spec-conformance summary — and
  ends when `plan.md` and `tasks.md` are final, with a new session
  (not `/clear`, which keeps the model) whose opening prompt is the
  spec session's last message. A session in this repo opens at the
  session tier at medium, so a spec session states its model and
  effort first (`/effort status`) and asks the person to raise effort
  to high for this session (`/effort high`) before continuing. The
  next session opens at medium again from `.claude/settings.json`.
  (The project's very first spec, with no codebase yet, happened in
  chat.)
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
- **Implementation runs in the subagent the role table names** — the
  task implementation row for ordinary tasks, the close-out row for
  close-out — one task per dispatch, sequentially. Neither is
  re-decided per spec: the table is the answer until the person
  changes a row. The orchestrating
  session triages each task, dispatches routine ones on a task bundle
  assembled with shell (task line, plan section, acceptance criteria,
  files, the pattern file to copy), and on return verifies with the
  verification command below — re-run by the orchestrator for tasks
  marked `review: per-task`, taken from the implementer's verbatim
  output otherwise — never by re-reading the diff. Only the
  orchestrator edits `tasks.md` or commits, and the orchestrator never
  implements second-look notes or does device or browser checks by
  hand.
- **Where a device or browser pass runs** (adopted 2026-09-24 from the
  skill's collaboration workflow). Not in the implementer, which has no
  simulator or browser access. A task whose Verify criterion needs one
  gets it from the person's walkthrough at the phase pause, or from a
  general-purpose agent the orchestrator dispatches for that check
  alone — never a task whose verification the implementer cannot
  perform, read as if it had. A dispatched pass costs mostly re-read
  context, so three habits keep it bounded: fold waits into the next
  call (`sleep 2 && <screenshot>`, or wait for a condition) rather than
  spending a turn on a bare `sleep`; one dispatch per checklist
  section, each given only its own section and returning a short
  pass/fail list, so context resets between sections; and leave the
  pass only what needs eyes — a check with a deterministic answer (an
  element exists, a label reads right, a key reaches the right screen)
  goes into the automated UI tests the plan names, where it runs every
  build for nothing, and the pass keeps layout, motion and feel. A
  smaller model or lower effort is not the lever.
- **Close-out runs on its own bundle, and the bundle carries the
  evidence** (adopted 2026-09-24 from the skill's collaboration
  workflow). The orchestrator assembles it with shell, as for a review
  bundle: each acceptance criterion with the test names or Done notes
  that satisfied it, the walkthrough list and what the person said at
  each pause, the tier log, the spec's summary and its decided lines,
  the `ROADMAP.md` entries this spec touches, and the previous spec's
  `DECISIONS.md` section as the shape to copy. The dispatch says not to
  read `spec.md`, `plan.md` or `tasks.md` in full; the close-out never
  goes looking for evidence itself. It drafts the `ROADMAP.md` and
  `DECISIONS.md` text into a file on the branch, to be applied to `main`
  after the merge, and updates the README if user-facing behavior
  changed. Only then does the pre-merge sweep run, so it verifies the
  close-out instead of pre-dating it. (Measured on the same model: a
  close-out allowed full reads cost $12.38 over 111 turns; one handed
  the bundle with full reads forbidden cost $0.99 over 24 turns.)
- **One implementation session per spec.** It opens when `plan.md` and
  `tasks.md` are final and ends at the merge; a phase pause is a pause
  in it, not a boundary — the person attests and says continue.
  `/compact` if the context grows large; never clear or compact
  mid-task. `/clear` is not part of the workflow: both session
  boundaries are new sessions.
- **Two pauses end with a continuation prompt, and only two**: plan
  and tasks final, and the merge with the next spec waiting on
  `ROADMAP.md`. At those the report's last item is the exact prompt to
  paste into the next session, in its own fenced block. It names the
  spec directory, the files to read, where to resume, the involvement
  level, the pause cadence, and any effort switch the next session
  needs. Write anything the next session needs to a file first; the
  prompt points at files. If nothing can proceed until the person
  decides something, say so instead.
- **A phase pause never ends with a continuation prompt.** The phase
  report ends with what to check in the app and how to say continue —
  nothing else. The session cannot know whether the person is about to
  stop, so a rule conditioned on that produces a prompt at every phase,
  which is what this line exists to prevent: the spec runs in one
  implementation session, and a prompt offered unasked invites a
  `/clear` that costs a re-read and buys nothing. If the person says
  they are stopping, or asks for a prompt, write one then, as the next
  message — the resume form from the first unchecked task. Asked for,
  it costs one turn; volunteered, it costs the session.
- **Batch the bookkeeping**: commit, checkbox, and tier-log row in one
  shell command; bundle assembly and dispatch back to back. Every turn
  saved is one fewer re-send of the whole context.
- **Fallback** (Fable profile): if the top tier's usage budget runs
  out, dispatch the planner and sign-off at the implementation tier for
  the rest of the window (drop the override; both definitions default
  to `opus`), switch the session itself to the fallback session model
  named in the Fable profile above (`/model claude-opus-5-5` — one
  cache re-write, then continue), and dispatch `sdd-implementer` for
  any row that names `sdd-implementer-fable`, including close-out.
  This is the whole role table stepped down at once, and it is the
  automatic form: it fires on the allowance, for the rest of the
  window, and the rows are not edited. A step-down the person *asks*
  for is the other form — it edits the rows and persists until they
  say otherwise. Don't confuse them, and don't silently revert one the
  person asked for. Either way the tier log records what ran and when
  the switch happened.
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
  to `claude-opus-4-8` on 2026-09-09, and to `fable` at medium on
  2026-09-11; the `sdd-implementer` has always run at `opus`. Spec 021
  is the first spec with the session on `fable` at medium — its tier log
  in `specs/021-wager-and-loss/tasks.md` carries the allowance reading
  and the per-invocation token counts to compare against specs 019–020.
  **Spec 025 (2026-09-16/17) ran entirely on the session fallback**:
  the person chose `claude-opus-5` at high effort with Fable's allowance
  low, so the whole spec — conversation, planning, sign-off,
  implementation, reviews and sweep — ran at `opus`/`claude-opus-5` with
  the top-tier override dropped. Its tier log is in
  `specs/025-outfitter-locked-cards/tasks.md`.
  **2026-09-19: the project moved to the Opus profile** (then called
  the economy profile) — the person said to use Opus for everything
  from now on, so every row resolves to `opus`, the close-out row names
  `sdd-implementer`, and `.claude/settings.json` puts new sessions on
  the Opus session tier at medium. Recorded here rather than in a tier
  log because no spec was in flight; the next spec's tier log opens
  with it. It holds until the person says otherwise.
  **2026-09-24: reconciled with the skill's 2026-09-23/24 revision** —
  profiles renamed Fable/Opus, the Opus model is `claude-opus-5-5`
  everywhere (`.claude/settings.json` rewritten from
  `settings.opus.json`), the close-out bundle rule and the
  device-or-browser-pass guidance adopted above. **Same day, the
  plan-and-tasks draft row moved to the implementation tier (no
  override) as a trial, starting with spec 031** — the first spec whose
  tier log opens with it; the sign-off stays at the top tier.

## Spec-driven workflow

This project follows spec → plan → tasks → implement, gated by review
between each phase — the person's or the `skeptical-reviewer`'s, per
the involvement level above. Every phase gets the reviewer's review;
*Pause cadence* decides which of them also stop for the person.
Artifacts live in `specs/<NNN>-<slug>/`:

- `spec.md` — what and why, user-facing behavior, acceptance criteria,
  explicit non-goals. No implementation detail.
- `plan.md` — technical design: types, data flow, what changes where.
- `tasks.md` — ordered, small, independently verifiable tasks. From
  spec 029 onward every phase header carries a `walkthrough:` marking
  (see *Pause cadence*), and the file carries the running walkthrough
  list that the unpaused phases append to.

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

### Chores (no spec)

Small, self-contained fixes the person rules do not need a spec — a
one-line flow change, a layout spacing fix, a wording fix — run as a
**chore**: a `chore/<slug>` branch off `main`, a draft PR, each fix
dispatched to the `sdd-implementer` with the person's words as the task
line, one `skeptical-reviewer` pass on the combined diff, then merge
(branch kept). No `specs/` directory. What makes something a chore is
the person's ruling plus the footprint: no engine, AI, save-format,
balance-data, or dependency change, and no new screen or mode. If a fix
supersedes a line in an earlier spec, the chore's merge gets a short
`DECISIONS.md` entry saying so. (Ruled 2026-09-13 for the first chore:
run-over returns to the menu; wager prompt spacing. The branch name was
`chore-<slug>` until 2026-09-19, when it was reconciled with the git
conventions' `chore/<short-description>`; branches already merged keep
the names they merged under.)

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
- **A spec's implementation never goes to `main` directly.** Its code,
  its files under `specs/<NNN>-<slug>/`, and any README change
  describing its behavior live on that spec's branch and reach `main`
  through its PR.
- **Work that isn't a spec's implementation commits straight to
  `main`, and doesn't need to ask.** Roadmap grooming, a
  `DECISIONS.md` entry, a constitution amendment, a docs or README
  correction unrelated to a spec in flight, the written residue of a
  conversation that didn't become a spec, tooling or config no spec
  touches. None of this has a spec branch, and none of it deserves
  one — a branch and a PR for a roadmap paragraph cost more than they
  protect, and stopping to ask costs a turn and interrupts the
  conversation that produced the change. The test is whether the
  change implements part of some spec's `tasks.md`, not whether the
  file appears on a list. If it doesn't: commit it to `main`, push,
  and say so in the report. This line is the permission; don't ask for
  it again.
- **Branch anyway when the change wants a diff someone will look at** —
  a dependency bump, a refactor with no spec behind it, anything where
  being wrong is expensive or awkward to unwind. Use
  `fix/<short-description>` or `chore/<short-description>`, not the
  spec `<NNN>-<slug>` convention, and open a PR. Size and risk decide
  this, not whether the work counts as "a spec."
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
