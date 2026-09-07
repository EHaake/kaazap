# Tasks: Opponent portraits — spec 016

**Status**: **All tasks complete (T001–T008).** Person attested the panels; pre-merge whole-spec sweep came back **merge-clean** (0 blocking, non-blocking cleanups applied). 267 tests green, no warnings; branch pushed to draft PR #19. **Awaiting the person's go to mark the PR ready and merge to `main`.** The spike-gate escalation (below) is resolved: portraits were externally authored (Fable 5.1) and integrated in T002.
**Implements**: plan.md in this directory
**Person approval:** plan + tasks approved; minimum 139×31 accepted (portraits always-on); the `design/brief.md` bounded exception approved (lands at T001b).

**Spike outcome (T001 go/no-go, person ruling):** the render path + approach are **accepted**
— portraits render correctly and read as faces — but the person escalated the **art authoring**
to a more capable tool (Claude Design / Fable 5.1) to get better faces than the Claude Code
baseline, per the spec's human-ruled escape hatch. The hand-off spec is
`portrait-art-brief.md` in this directory (same 18×12 monochrome block-grid format, so results
drop straight into the render path). **Consequence for the task list:** T002 is no longer
"Claude Code authors the 9" — it becomes "**integrate + validate** the 11 externally-authored
portraits" (drop into `assets/portraits/`, run the dimension/distinctness tests, render for the
person's sign-off). T001's two baseline faces (`generic.txt`, `greeb.txt`) stay as placeholders
until the replacements arrive. Implementation is **halted here** until the art comes back; the
render-path code (T001) is committed and green.

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

**Foundational phases:** Phase 1 (T001) and Phase 3 (T003, T004) are foundational —
the new shared render path, the shared data-model field, and the layout/minimum-terminal
change everything else rests on. Under the constitution's current review cadence the default
is a **per-phase** `skeptical-reviewer` pass; only the two tasks whose mistake is genuinely
expensive to unwind carry `review: per-task` — **T001** (the `portrait.rs` render primitive
plus the `OpponentProfile.portrait` data-model field a dozen roster entries depend on) and
**T003** (the grown minimum terminal + in-match panel geometry that re-shapes every screen's
layout constraint). Every other task — T002, T004, T005, T006, T007, and the close-out — is
reviewed **at phase end** (T004 and T005, foundational/shared but contained, ride their
phase's review rather than their own). T001 is additionally the spec-mandated **art-format
spike**, its person pause a go/no-go; **T001b** lands the `design/brief.md` bounded exception
immediately after the spike confirms the vocabulary (sign-off finding 1).

---

## Phase 1 — Portrait render path + format spike

<!-- Foundational: the new render primitive + the shared OpponentProfile field. Also
the spec-mandated spike — prove one portrait reads as a face before authoring the rest.
Per-task skeptical-reviewer pass, then a person pause (go/no-go). -->

- [x] **T001 (foundational, spike, review: per-task)** — Create `src/portrait.rs` (add to `src/lib.rs`):
  `pub const PORTRAIT_WIDTH = 18` / `PORTRAIT_HEIGHT = 12`; `draw_portrait(frame, x, y,
  art, emphasis)` drawing each art line via `draw_text`, clip-safe like `CardView::draw`.
  Author **two** original monochrome art files — `assets/portraits/generic.txt` and
  `assets/portraits/greeb.txt` — each `PORTRAIT_HEIGHT` lines of ≤ `PORTRAIT_WIDTH` cells,
  block/shade + plain-char vocabulary (plan §Crux 1). Add `portrait: &'static str` to
  `OpponentProfile` (`opponent.rs`); set `DEFAULT_OPPONENT` → `generic.txt`, `greeb` →
  `greeb.txt`, the other 9 roster entries → `generic.txt` **temporarily** (T002 repoints
  them). Wire the cursored opponent's portrait into `opponent_select.rs`'s draw (raw
  `draw_portrait` + name, placed right of the list) so it is viewable.
  *Verify: `cargo build` no new warnings; `cargo test` green (reported verbatim) — new
  tests: `draw_portrait` is clip-safe off-frame; `greeb`/`generic` each have
  `lines().count() == PORTRAIT_HEIGHT` and every line `<= PORTRAIT_WIDTH`. Then the
  **spike gate**: run the game, open opponent-select, cursor on Greeb — confirm the
  portrait reads as an alien face with eyes and expression, in monochrome. **PAUSE for
  the person** (go/no-go): if it does not read as a face, STOP and escalate the approach
  (the spec's human-ruled escape hatch) before authoring the rest — do not proceed.*

- [x] **T001b (governance doc, its own commit)** — Amend `design/brief.md` with the bounded
  portraits exception, now that T001's spike has confirmed the vocabulary. Add an "Amendment
  (spec 016 — opponent portraits)" section mirroring the Motion/starfield amendment: opponent
  portraits are the single pictorial element — monochrome, static, opponent-only, one fixed
  portrait frame distinct from the card frame, no color, no animation — a bounded exception to
  "avoid Unicode block-art flourishes" and the skeuomorphism boundary, naming the confirmed
  glyph vocabulary. Pulled ahead of Phase 2 (sign-off finding 1) so the standing contract
  sanctions portraits before the bulk of contradicting code lands. No code; the exception was
  already blessed at plan sign-off (plan Crux 1), so no per-task review.
  *Verify: `design/brief.md` carries the amendment; build/tests unaffected (doc-only).*

## Phase 2 — The cast (author all portraits)

- [x] **T002** — Author the remaining 9 roster portraits (`assets/portraits/{dax, vessa,
  nima, toran, brakka, rix, kesh, magistrate, sovereign}.txt`), each original (no
  trademarked species), monochrome, dimensions-valid, expression fitting the opponent's
  blurb/difficulty; repoint each `OpponentProfile.portrait` to its own file (copies T001's
  `include_str!` pattern). Add data-invariant tests in `opponent.rs`.
  *Verify: `cargo build`/`cargo test` green — tests: every portrait (`OPPONENTS[*]` +
  `DEFAULT_OPPONENT`) has `lines().count() == PORTRAIT_HEIGHT` and each line `<=
  PORTRAIT_WIDTH`; the 10 roster `.portrait` strings are pairwise distinct;
  `DEFAULT_OPPONENT.portrait == generic` and non-empty. Driver: cursor the whole roster in
  opponent-select — 10 distinct faces, none blank.*

## Phase 3 — Layout foundation: presence panel + grown minimum

<!-- Foundational: adds the opponent panel to the board layout, raises the enforced
minimum terminal, and re-shapes the campaign-map field. T003 gets a per-task
skeptical-reviewer pass; T004 is reviewed at phase end. -->

- [x] **T003 (foundational, review: per-task)** — In-match panel geometry + the grown minimum. Add the panel
  size consts to `src/portrait.rs` (`PANEL_W`, `PANEL_H_INMATCH`, `PANEL_GAP`, derived from
  `PORTRAIT_WIDTH/HEIGHT` — plan §Design). In `src/layout.rs`: add `pub opponent_panel: Rect`
  to `BoardLayout` (anchored `left + BOARD_WIDTH + PANEL_GAP`, top-aligned with the board
  block, `PANEL_W × PANEL_H_INMATCH`) and `pub const IN_MATCH_MIN_WIDTH = BOARD_WIDTH +
  2*(PANEL_GAP + PANEL_W)`. In `src/config.rs`: `min_size` → `(IN_MATCH_MIN_WIDTH,
  BOARD_BLOCK_HEIGHT)`; fix the doc comment. Board centering and every existing board Rect
  stay byte-for-byte the same. (Sign-off finding 3: repointing `min_size` makes the
  `BOARD_WIDTH` import in `config.rs` dead — drop it and add `IN_MATCH_MIN_WIDTH` in the same
  edit so the no-new-warnings gate stays green.)
  *Verify: `cargo build`/`cargo test` green — new `board_and_panel_fit_the_minimum_terminal`:
  at `(IN_MATCH_MIN_WIDTH, 31)` `opponent_panel` is in-bounds, `opponent_panel.x0 >
  l.opponent.hand.x1`, does not overlap the board, and its bottom `<= 30`; the renamed
  min-size test asserts `min_size() == (139, 31)` and `== (IN_MATCH_MIN_WIDTH,
  BOARD_BLOCK_HEIGHT)`. Report the new minimum (139×31) in the task note.*
- [x] **T004 (foundational)** — Campaign-map portrait rail. In `src/layout.rs`
  `CampaignMapLayout::new`: add `pub portrait_panel: Rect` on the right of the field band
  (`PANEL_W` wide, `2 + 1 + PORTRAIT_HEIGHT` tall, near the field top) and shrink `field.x1`
  to `portrait_panel.x0 - 2`. Move the two map guard tests to the new minimum.
  *Verify: `cargo build`/`cargo test` green — `campaign_map_layout_fits_the_minimum_terminal`
  and `the_campaign_map_is_legible_at_the_minimum_terminal` run at `(IN_MATCH_MIN_WIDTH, 31)`
  and additionally assert every planet node + label stays inside the reduced `field` and
  clear of `portrait_panel` (no node/label lands on the rail). Nodes reflow into the
  reduced-but-wider field — no `PLANETS` position edits. Sign-off finding 2: far-right
  cursored-label clearance is genuinely tight (~1 cell) and is NOT guaranteed by the
  "more room" framing — treat this assertion as a real check and bump `FIELD_MARGIN_X` or the
  field/rail gap if it fails.*

## Phase 4 — Presence panel on every surface

- [x] **T005 (shared component)** — Add `draw_presence_panel(frame, panel: Rect, name, art)`
  to `src/portrait.rs`: `BorderWeight::Single` box, `name` (`Emphasis::Strong`, centered top
  interior), `draw_portrait` centered below, remaining height left blank (reserved
  banter/pips). Replace T001's raw opponent-select preview with a call to it over a pure
  `preview_rect(config) -> Rect` (right of the centered list). This is the shared drawer
  referenced by T006 and T007.
  *Verify: `cargo build`/`cargo test` green — `preview_rect(min)` is on-frame and its `x0 >`
  the widest roster row's right edge (no overlap with the list). Driver: opponent-select
  shows a bordered portrait + name panel beside the list.*
- [x] **T006** — In-match panel. In `src/board.rs`, `BoardView::draw` calls
  `draw_presence_panel(self.layout.opponent_panel, state.opponent_profile.name,
  state.opponent_profile.portrait)` after the board. (Copies the `draw_side`/layout-driven
  drawing style already in `board.rs`.)
  *Verify: `cargo build`/`cargo test` green. Driver: start a Quick Play match (default
  opponent → generic portrait) and a campaign match — the opponent portrait + name panel is
  visible beside the board throughout the match; the board itself is unchanged and still
  centered. Snapshot at 139×31 and wider.*
- [x] **T007** — Campaign-map preview. In `src/campaign_map.rs`, `draw` resolves the focused
  planet's shown opponent (`run.next_opponent(planet)`, else the planet's last opponent,
  else `DEFAULT_OPPONENT`) and calls `draw_presence_panel(layout.portrait_panel, name,
  portrait)`. (Uses `opponent_by_id`, already imported.)
  *Verify: `cargo build`/`cargo test` green. Driver: on the campaign map, focus each node —
  the next opponent's face shows in the rail; a cleared planet shows its resident face; no
  node or label is clipped or covered. Snapshot at 139×31 and wider.*

## Final phase — Spec close-out

- [x] **T008** — Docs, driver, sweep. (The `design/brief.md` bounded-exception amendment
  already landed in its own commit at **T001b** — sign-off finding 1.) `DECISIONS.md`: the
  portrait vocabulary, the grown minimum
  (89×31 → 139×31), the `OpponentProfile.portrait` field, authored-assets-not-a-script, the
  campaign-map rail, the reserved (empty) mirror margin. `assets/CREDITS.md`: portraits are
  original in-repo authored art (no license encumbrance). `Readme.md`: document the new
  minimum terminal size. `ROADMAP.md`: mark opponent portraits shipped; drop from future.
  Check off `spec.md` acceptance criteria with evidence. Request the pre-merge whole-spec
  sweep.
  *Verify: `cargo build`/`cargo test` green, reported verbatim; legible snapshots at 139×31
  and wider of the in-match panel, opponent-select preview, and campaign-map rail; every
  existing screen (board, start menu, opponent-select, campaign map, deck-builder, shop)
  confirmed fitting at the new minimum and erroring cleanly below it; `ROADMAP.md` no longer
  lists this as future; `design/brief.md` amended (in T001b); sweep clean or findings
  resolved. Sign-off finding 4: correct the now-stale "89 = minimum" naming/comments in
  `briefcase_fits_the_minimum_terminal`,
  `layout_regions_are_in_bounds_and_stacked_at_several_sizes`, and
  `the_full_roster_and_footer_fit_the_minimum_terminal` (they still pass — only their
  "minimum" wording misdescribes 139) and add a 139-wide case to the multi-size board test.*

---

## Handoff note

Read `CLAUDE.md`, `design/brief.md`, and `specs/016-opponent-portraits/{spec,plan,tasks}.md`,
then implement from the first unchecked task. Involvement level is **product owner**.
Dispatch each routine task to the `sdd-implementer` per the model policy; verify by running
the build and tests yourself, then commit. **Tasks marked `review: per-task` (T001, T003):**
run the `skeptical-reviewer` after the task, scoped to that task's diff, its plan section, and
its acceptance criteria (a shell-assembled bundle), one review plus at most one re-review, and
re-run the verification command yourself before committing. **Every other task (T002, T004,
T005–T007) and the close-out:** review at phase end. Pause for the person after each phase,
and whenever something unexpected bears on spec adherence.

**T001 is a hard gate:** it is the spec-mandated art-format spike. After building it, the
person must *look* and confirm the portrait reads as a face (eyes + expression) in monochrome.
If it does not, STOP — do not author the remaining portraits — and escalate the approach
(e.g. human-supplied source art) per the spec's human-ruled escape hatch.

Every pause produces a report in this shape, in this order:

1. **Why this pause** — a phase boundary, a spec-adherence question, or an escalation
   trigger. One line.
2. **What you can now do** — behavior that exists and can be tried, as a user would
   experience it, so attestation is possible.
3. **Where execution deviated from the spec, and why** — every place, per "never silently,"
   not just the interesting ones.
4. **What needs your decision** — product questions only; technical detail lives in plan.md
   and the commit log.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs and
reviewer invocations), any escape-hatch miss (a task the orchestrator had to redo at the top
tier, and why). Compare the spec total against a previous spec of similar size before
treating the policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| Planning: draft (sdd-planner) | opus (top, inherit) | 162,098 | drafted; no escalation |
| plan + tasks sign-off (skeptical-reviewer) | opus (decision) | 110,054 | **signed off**, 0 blocking; 5 second-look items folded (brief timing → T001b, T003 import, T004 rail clearance, T005 shared-path review, T008 stale-minimum wording) |
| T001 impl — code scaffolding (sdd-implementer) | opus (one down) | 55,663 | done; flagged that `game.rs:1688` is a full `OpponentProfile` literal (not `..DEFAULT_OPPONENT`), so it needed the new field — a minor plan §2 inaccuracy, handled. Art files authored by the orchestrator (spike creative core), not dispatched. |
| T001 review (skeptical-reviewer) | opus (per-task) | 39,472 | **signed off**, 0 blocking. Non-blocking notes (logged, deferred to sweep): (1) block/shade glyphs `█▀▄▓░` are East-Asian *Ambiguous* width — a pre-existing project-wide assumption shared with the box borders + title art, not a T001 regression; (2) plan §2 game.rs claim (above); (3) clip test omits direct right-edge horizontal-overrun case (covered transitively via `draw_text`); (4) placement test uses ASCII not a multibyte glyph (sound via `draw_text`); (5) temporary opponent-select preview could graze a long blurb — verified no overlap (blurb row ≫ portrait rows) and T005 replaces it anyway. |
| T001 spike gate (person) + T002 art (external, Fable 5.1 session per `portrait-art-brief.md`) | fable (art session); orchestrator integrated | n/a (art session not metered here) | spike **go**; all 11 portraits (incl. re-authored `generic`/`greeb`) delivered, validated 12×≤18 / palette-only / pairwise distinct, signed off by the person; roster repointed, T002 invariant tests added; 265 tests green. T001b still open. |
| T003 impl — panel geometry + grown minimum (sdd-implementer) | opus (one down) | 37,501 | done, no deviations; board Rects/centering byte-for-byte unchanged; flagged that campaign-map tests hardcode `(89,31)` → T004 moves them. 266 tests green. |
| T003 review (skeptical-reviewer) | opus (per-task) | 24,410 | **signed off**, 0 blocking (arithmetic re-derived: panel 117..138 at 139-min, symmetric 25-col margins). Non-blocking notes (logged, sweep): (1) fit test's overlap guard asserts `x0 > opponent.hand.x1` (board *content* edge 109) not the board-*block* edge 113 — the plan's specified assertion, panel at 117 clears both; (2) test hardcodes rows `31` not `BOARD_BLOCK_HEIGHT`; (3) zero right-edge slack at the minimum (intentional, symmetric-margin design); (4) other-screen fit at 139 not checked here — satisfied by construction (every other screen is a centered block that already fits 89, so it fits wider) and confirmed by the T008 sweep/driver. |
| T004 impl — campaign-map portrait rail (sdd-implementer) | opus (one down) | 38,888 | done, no deviations; `FIELD_RAIL_GAP=2`, no bump needed — far-right cursored label (planet "zenith") clears the rail by 14 cells at the wider 139-min field (sign-off finding 2's ~1-cell tightness was an artifact of the old 89 field). Stale "89×31" comments in the moved tests corrected. |
| T004 review (skeptical-reviewer, phase-end) | opus (default) | 25,891 | **signed off**, 0 blocking; strong second-look: rail *vertical* bounds were untested (reviewer lacked the consts to confirm) → orchestrator added `portrait_panel` to the fits test's in-bounds loop + `portrait_panel.y1 < panel.y0` (computed clear: y1=16 vs bottom info band at row 26). Minor non-blocking: `FIELD_RAIL_GAP=2` leaves one clear column (cosmetic). |
| T005 impl — draw_presence_panel + opponent-select preview (sdd-implementer) | opus (one down) | 29,244 | done, no deviations; shared bordered drawer + pure `preview_rect` + preview fit test; replaced T001's raw preview. 267 tests. |
| T006 in-match panel (orchestrator) | opus (top) | trivial | 2-line wiring (import + one `draw_presence_panel` call in `board.rs`); done at the orchestrator tier as sub-dispatch-threshold, not an escape-hatch miss. |
| T007 campaign-map rail (orchestrator) | opus (top) | trivial | import + next→last→default resolution chain + one call in `campaign_map.rs`; done at the orchestrator tier as sub-dispatch-threshold. |
| Phase 4 review (skeptical-reviewer, phase-end, T005–T007) | opus (default) | 25,108 | **signed off**, 0 blocking. Non-blocking, all resolved: (1) name overflow into the 20-col interior — confirmed longest name "The Magistrate" (14) fits; (2) T003/T004 Rects taken on faith — confirmed clear of live content by the driver render (panels sit clean beside board and on the map); (3) `next_opponent=Some(bad id)` skips the last-opponent fallback → still `DEFAULT_OPPONENT`, never blank (invariant violation anyway). Driver-attested at 180-wide: opponent-select bordered panel, in-match right-margin panel, campaign rail all render; board/nodes unchanged. |
| T008 — close-out docs + stale-minimum fixes (orchestrator) | opus (top) | n/a | DECISIONS/CREDITS/README/ROADMAP updated; spec.md criteria checked with evidence; stale "89×31" test wording fixed + (139,31) case added; 267 tests green. |
| Pre-merge whole-spec sweep (skeptical-reviewer) | opus (default) | 58,192 | **merge-clean**, 0 blocking. Non-blocking cleanups applied: removed 9 stale "temporary generic — T002 repoints" comments in `opponent.rs` (sat above the already-correct per-opponent `include_str!`); `CREDITS.md` now names Fable 5.1 as the art author (matching DECISIONS). Noted-no-action: `plan.md`'s "Claude-authored" is historical drift (DECISIONS is the record of record), `portrait-art-brief.md` confirmed committed, the `layout`↔`portrait` mutual import is the intended single-source design. |
