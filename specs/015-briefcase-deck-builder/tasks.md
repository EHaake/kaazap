# Tasks: Two-panel "briefcase" deck-builder — spec 015

**Status**: Draft — pending review
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task should be completable (and
testable) on its own. If a session ends mid-list, resume by finding the first
unchecked task — don't re-verify everything above it unless something looks off.

<!-- WARNING: once implementation starts, this file gets written by more than one
party — whoever's steering adds scope and reshuffles tasks; the implementing
session (the orchestrator — never the sdd-implementer subagent) checks boxes and
adds findings. Never edit this file from a stale copy. Prefer small, targeted
edits over regenerating it wholesale — a full replacement silently discards
whatever the other party added since your copy was taken. -->

Per the constitution: every implementation task ends with an actual build and,
where tests exist for what changed, an actual test run — reported, not summarized.
Under the model policy, the orchestrator re-runs build + tests itself after an
implementer returns, and only the orchestrator commits.

---

## Phase 1 — Layout foundation

<!-- Foundational geometry; the whole two-panel approach rests on it fitting the
89×31 minimum. Per-task skeptical-reviewer pass. -->

- [x] **T001 (foundational)** — Add `BriefcaseLayout` to `src/layout.rs`: two
  side-by-side bordered panel `Rect`s (`collection`, `deck`) within `num_cols`,
  a shared `title_y`/`readout_y`/`hint_y`, per-panel `label` anchors, `cols` (=4),
  `visible_rows` derived from height, and `card_origin(panel, visible_index)`.
  Pure over `Config`. Add `briefcase_fits_the_minimum_terminal`.
  *Verify: at `Config(89,31)` the test asserts `cols == 4`, `visible_rows >= 3`,
  both panel borders within `num_cols`, `title_y < readout_y < hint_y` all
  on-frame, every `card_origin(panel, i)` (all visible slots) lands strictly
  inside its own panel's borders, the panels don't overlap (`collection.x1 <
  deck.x0`), and a full visible grid of cards clears the hint (mirrors
  `grid_layout_fits_the_minimum_terminal_for_the_full_universe`). `cargo build`
  clean; `cargo test` green.*

## Phase 2 — Screen reshape

- [x] **T002** — Reshape `src/deck_builder.rs` into the two-panel briefcase over
  `BriefcaseLayout`: `enum Panel { Collection, Deck }`; state `{ active,
  collection_cursor, deck_cursor, collection_scroll }`; rows split by location
  (`collection_by_type()` → available `owned-in_deck>0` on the left, `in_deck>0`
  on the right); input (`Tab`/`BackTab` switch, arrows+`wasd` move/wrap,
  `Enter`/`Space` → `Add` in Collection / `Remove` in Deck, `Esc`/`x` → `Back`,
  retire `Backspace`); draw two bordered labeled panels, `CardView` Heavy=cursor
  else Single (**drop Double**), `×N` caption, `Deck: N/10` readout (Alert while
  short), empty-side cue; keep `collection_scroll` following the cursor. Initial
  focus (and focus after a side empties) rests on a non-empty panel; **only the
  active panel's cursor is Heavy+pulse — the inactive panel's remembered cursor
  draws Single**; non-cursored cards keep today's Muted; source `cols`/`visible_rows`
  only from `BriefcaseLayout` (retire the old `COLS` const).
  *Verify: new `deck_builder` unit tests green — `Tab` flips `active`; `Enter` in
  Collection → `Add(cursored)`, in Deck → `Remove(cursored)`; a card with both
  `available>0` and `in_deck>0` appears in **both** panels' row lists with the
  right counts (the split-by-location behavior); initial focus rests on a non-empty
  panel (an all-decked profile opens on Deck); arrow move wraps; `collection_scroll`
  clamps and keeps the cursor visible; `Esc`/`x` → `Back`; unknown key → `None`.
  `cargo build` no new warnings; `cargo test` green. (Both panels rendering is
  confirmed by the T005 driver.)*

## Phase 3 — Return routing + entry points

<!-- T003 touches the shared Back arm (foundational). -->

- [ ] **T003 (foundational)** — Origin tracking in `src/deck_builder.rs` +
  `src/app.rs`: `enum BuilderOrigin { Menu, Map }` (`Copy`) on `DeckBuilderState`
  with `origin()`; `open_deck_builder(origin)`; `BuildOutcome::Back` branches
  `Menu => start_menu()`, `Map => open_campaign_map()`; update the three existing
  call sites — menu `SideDeck` → `Menu`, opponent-select divert (`app.rs:374`) →
  `Menu`, **campaign-launch divert (`app.rs:445`) → `Map`** (fixes the pre-existing
  bug where fixing an incomplete deck mid-campaign returned to the menu). Express
  the routing as a pure `back_destination(BuilderOrigin)` seam mirroring the
  existing `confirm_choice`/`ConfirmChoice` pattern (`app.rs:281`), so the branch is
  unit-testable without an `App`.
  *Verify: origin routing tests — `new(Map).origin() == Map`, and the pure
  `back_destination` maps `Menu`→menu / `Map`→map (mutation-checkable, like
  `confirm_choice`); the campaign-divert-returns-to-map fix stated in the report;
  `cargo build`/`cargo test` green.*
- [ ] **T004** — Map entry point in `src/campaign_map.rs` + `src/app.rs`:
  `MapOutcome::OpenDeckBuilder`; `KeyCode::Char('c')` arm (**`c`; `d` is taken by
  wasd movement**); extend the hint to `"↑/↓ move  ·  Enter play  ·  b shop  ·  c
  deck  ·  Esc menu"` (keep the existing double-space `·` style, `campaign_map.rs:268`);
  app CampaignMap arm → `open_deck_builder(BuilderOrigin::Map)`.
  *Verify: `campaign_map` test — `Char('c')` → `OpenDeckBuilder`; hint contains
  `deck`. `cargo build`/`cargo test` green. (Map `c` → builder → `Esc` → map
  round-trip confirmed by the T005 driver.)*

## Final phase — Spec close-out

- [ ] **T005** — Driver verification + close-out. Back up + checksum-restore the
  real profile (standing data-safety practice); stage a duplicate-rich profile so
  both panels populate. Capture 89×31 and ~120-wide snapshots of: both panels, a
  move-across (counts + readout updating), and map `c` → builder → `Esc` → back to
  the **map**. Check off `spec.md` acceptance criteria with evidence. Update
  `ROADMAP.md` (mark the briefcase shipped; drop it from future) and `DECISIONS.md`
  (the `c` key choice, the map-entry scope bump beyond "presentation-only", the
  campaign-divert bug fix); README only if entry wording changed. Remove the
  now-orphaned `GridLayout` + its fit test from `src/layout.rs` (T002 left it with
  no caller — `BriefcaseLayout` supersedes it; verify no references remain first)
  and note the supersession in `DECISIONS.md`. Request the pre-merge whole-spec
  sweep.
  *Verify: `cargo build`/`cargo test` green, reported verbatim; legible 89×31
  snapshots; `ROADMAP.md` no longer lists this as future; sweep clean or findings
  resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/015-briefcase-deck-builder/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product owner**.
Dispatch each routine task to the `sdd-implementer` per the model policy; verify by
running the build and tests yourself, then commit. **Foundational tasks (T001,
T003):** run the `skeptical-reviewer` after the task, scoped to that task's diff,
its plan section, and its acceptance criteria (a shell-assembled bundle). **T002,
T004:** review at phase end. Pause for the person after each phase, and whenever
something unexpected bears on spec adherence.

Every pause produces a report in this shape, in this order:

1. **Why this pause** — a phase boundary, a spec-adherence question, or an
   escalation trigger. One line.
2. **What you can now do** — behavior that exists and can be tried, as a user would
   experience it, so attestation is possible.
3. **Where execution deviated from the spec, and why** — every place, per "never
   silently," not just the interesting ones.
4. **What needs your decision** — product questions only; technical detail lives in
   plan.md and the commit log.

## Tier log (first spec under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs
and reviewer invocations), any escape-hatch miss (a task the orchestrator had to
redo at the top tier, and why). Compare the spec total against a previous spec of
similar size before treating the policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| plan + tasks sign-off (skeptical-reviewer) | opus (decision) | ~163k | signed off first pass; 0 blocking; 6 second-looks folded into T001–T004 + spec goal-7 |
| T001 (sdd-implementer) | opus | ~106k | done first pass; build + 250 tests green |
| T001 review (skeptical-reviewer) | opus | ~48k | signed off; 0 blocking; geometry + non-vacuous test verified |
| T002 (sdd-implementer) | opus | ~156k | done; scroll deviation flagged + accepted; build + 254 tests |
| T002 review (skeptical-reviewer) | opus | ~98k | signed off; 0 blocking; scroll accepted; 3 second-looks folded (comment fix, guard test, GridLayout→T005) |
