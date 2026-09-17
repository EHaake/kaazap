# Tasks: Locked cards in the Outfitter — spec 025

> **Status**: Signed off (skeptical-reviewer at opus, 2026-09-16)
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
Under the model policy, the orchestrator verifies after an implementer returns,
and only the orchestrator commits. **This spec runs on the fallback**: the
Fable allowance is low, so the session runs on Opus 5, **experiment 2 is
paused** — every task below is dispatched to the `sdd-implementer` at `opus`,
not to `sdd-implementer-fable` — and the planner and the sign-off run at `opus`
with the top-tier override dropped.

**Foundational phase:** none. The spec is one screen; Phase 1 is the whole
feature (T001 the code, T002 the docs and the driver walkthrough). No task
carries `review: per-task` — T001's helpers are used only inside `shop.rs`, so
a mistake there is caught by the one Phase 1 review before anything inherits it.

---

## Phase 1 — The grouped Outfitter

<!-- T001 is the whole code change; T002 documents it and ends the phase with
the review and the driver walkthrough. -->

- [x] **T001** — `src/economy.rs` + `src/shop.rs`: list every card, grouped by
  region, cursor over the unlocked prefix. In `economy.rs` add
  `impl RegionTier { pub fn region_name(self) -> &'static str }` exactly as plan
  §Design 1 and the test `region_name_is_the_inverse_of_region_tier`; nothing
  else in the file changes. In `shop.rs`, per plan §Design 2: add `TIERS`,
  `listing()`, `unlocked_count(depth)`, `heading(tier, depth)`,
  `row_text(card, cursored, owned)`, `LIST_ROWS`, and `list_left(center_x)`;
  replace `anchors(num_rows, n)` with `anchors(num_rows)`; make `handle_input`
  wrap over `n = unlocked_count(deepest_reached(..))` and buy
  `listing()[cursor]`; redraw the list as three groups (blank row, heading at
  `left + 3` in `Normal`, card rows at `left`; locked rows and unaffordable rows
  `Muted`, the cursored row `pulse`), title/balance/hint unchanged; update the
  module doc, the `ShopState.cursor` comment, `handle_input`'s and `draw`'s docs,
  and drop the stale "89×31" wording. Tests (plan §Tests):
  `the_listing_groups_every_card_by_tier`,
  `the_unlocked_prefix_is_the_available_pool_at_every_depth`,
  `headings_name_the_region_and_lock_until_reached`,
  `arrows_wrap_over_the_unlocked_cards_only` (replaces
  `arrows_move_over_the_pool_and_wrap`), the rewritten
  `enter_and_space_buy_the_highlighted_card` (fresh profile: Up then Enter →
  `Buy(Card::PlusMinus(1))`; then Down pressed 7 times, Enter after each, every
  `Buy(c)` has `card_tier(c) == RegionTier::Outer` and the 7th Down lands back
  on `Buy(Card::Plus(1))`; Core profile: Down then
  Enter/Space → `Buy(listing()[1])`),
  `a_reset_map_relocks_groups_but_keeps_owned_counts`, and
  `the_full_list_fits_the_minimum_terminal` (replaces
  `the_full_pool_fits_the_minimum_terminal`; dimensions from
  `Config::min_size()`, asserts `LIST_ROWS == 21`; headings measured only for
  the tiers that can lock — Mid and Core — since Outer's locked form is never
  drawn, and `list_left` measures the same set). Add a `mid_profile()` test
  helper beside `core_profile()` (beaten `cinder/greeb`, `scree/dax`). Do not
  run `cargo fmt`. (Copies: the existing `shop.rs` — its state/outcome/draw shape
  and tests; `economy.rs`'s `the_pool_grows_monotonically_with_depth` for the
  three depths.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green,
  reported verbatim, with the eight named tests passing and
  `esc_and_x_back_out_and_unknown_keys_are_ignored` unchanged; `git diff
  --stat` shows only `src/shop.rs` and `src/economy.rs`; `git diff --
  src/economy.rs` adds only `region_name` and its test (no line of `card_tier`,
  `deepest_reached`, `available_pool` or `card_price` changes); the
  implementer's report quotes `listing`, `unlocked_count`, `heading` and
  `handle_input` verbatim.*

- [x] **T002** — `docs/economy.md`: the shop section describes the new list.
  Rewrite § The shop and its reserve's listing sentence per plan §Design 3 (all
  15 cards in three region groups; unreached groups dimmed with prices and owned
  counts under `<Region>  ·  reach the <Region> to unlock`; the cursor visits
  unlocked cards only, so a locked card can't be bought; the grouping reads
  `card_tier` / `deepest_reached`, the same functions as `available_pool`).
  Touch § The depth-gated pool only if it now reads as if locked cards were
  hidden. In § Tuning & guards (~line 260), rename the guard
  `the_full_pool_fits_the_minimum_terminal` to
  `the_full_list_fits_the_minimum_terminal` and add the new guards to that
  inventory: the shop's `the_listing_groups_every_card_by_tier`,
  `the_unlocked_prefix_is_the_available_pool_at_every_depth`,
  `headings_name_the_region_and_lock_until_reached`,
  `arrows_wrap_over_the_unlocked_cards_only`,
  `a_reset_map_relocks_groups_but_keeps_owned_counts`, and `economy.rs`'s
  `region_name_is_the_inverse_of_region_tier` (plan §Design 3). (Copies the
  neighbouring prose voice and the inventory's existing `name` (`file.rs`)
  form.)
  *Verify: `cargo build --all-targets` + `cargo test -q` green verbatim
  (docs-only, the command still runs); `git diff --stat` shows only
  `docs/economy.md`; `grep -n "the_full_pool_fits" docs/economy.md` is empty.
  **PAUSE for the person** (after the Phase 1 review): the
  orchestrator drives the walkthrough in plan §Verification with the
  `run-kaazap` skill at 139×31 — real profile, settings and save backed up and
  checksummed first, restored and checksum-verified after — and reports what it
  saw: fresh profile shows three groups, Mid and Core dimmed under their lock
  headings, cursor on `+1`, ↑/↓ wrapping inside the Outer group; a buy and a
  refused buy behave as before; Mid Rim reached → Mid navigable, Core locked;
  Core reached → all navigable; after New Campaign on a profile owning Mid/Core
  cards → both locked again with owned counts intact; headings read clearly
  against the dimmed rows; nothing clips; on the Core profile (no long heading
  drawn), whether the 28-column rows sit visibly left of the centered title —
  reported as a layout finding. Then the person tries it.*

## Final phase — Spec close-out

- [ ] **T003** — Close-out. Draft
  `specs/025-outfitter-locked-cards/closeout-main-docs.md` in spec 024's shape:
  **ROADMAP** — mark "Show locked cards in the Outfitter" (~line 584) shipped as
  spec 025, and annotate any shipped entry that still says the shop lists only
  the unlocked pool (`grep -n "shop\|Outfitter" ROADMAP.md`, read and judge —
  e.g. ~152–153, ~417) with the repo's inline "superseded by spec 025, which …"
  convention; **DECISIONS** — the three rulings (A2 price shown dimmed, revised
  from A3; B2 cursor skips locked rows; C1 heading names the region), plan
  tension §1 (the unlocked prefix keeps the cursor a plain index), §2 (the
  block-centered list column), §3 (Normal headings), and a supersession line for
  any earlier decision that the shop shows only the available pool (`grep -n
  "available pool\|shop" DECISIONS.md`, read and judge) — to apply on `main`
  after the merge, never on the branch. Run `cargo test -q` three consecutive
  times and paste the tails. Mechanical checks: `git diff main --stat` lists no
  `src/app.rs`, `src/card.rs`, `src/game.rs`, `src/player.rs`, `src/save.rs`,
  `src/profile.rs`, `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`; `git diff
  main -- src/economy.rs` adds only `region_name` and its test; `cargo build
  --all-targets` warning count equals `main`'s. Check off `spec.md`'s acceptance
  criteria with evidence. Request the pre-merge sweep; apply
  `closeout-main-docs.md` on `main` after the merge.
  **Carried from the Phase 1 review** (apply in the close-out, text-only): N1 the
  T001 line and plan §Tests say the 7th Down "lands back on `Buy(Card::Plus(1))`"
  — correct both to "the 1st Down wraps to `+1`, the 7th lands back on `±1`";
  N5 `docs/economy.md` "The shop's dimming reads the same predicate…" → "The
  shop's *affordability* dimming reads…"; N6 the comment above `core_profile()`
  in `shop.rs` (cosmetic, optional).
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/025-outfitter-locked-cards/{spec,plan,tasks}.md`,
then implement from the first unchecked task. Involvement level is **product
owner**. Dispatch each task to the `sdd-implementer` (opus, high) —
**experiment 2 is paused for this spec** (the Fable allowance is low), so no
dispatch goes to `sdd-implementer-fable` unless the person says the allowance
has reset, and even then not mid-spec. Verify from the implementer's verbatim
output (no task is `review: per-task`), then commit. **No foundational phase.**
One `skeptical-reviewer` pass (opus) at the end of Phase 1, after T002, on a
shell-assembled bundle — the phase diff, the task lines, plan §Design and
§Tests, the acceptance criteria — one review plus at most one re-review.
**Pause cadence**: pause after Phase 1 for the person to play the Outfitter,
per the constitution, unless the person says to run straight through; the
orchestrator drives the walkthrough in T002 first and reports it in plain
language. Back up + checksum-restore the real profile, settings and save before
and after every driver session. Repo-wide docs (`ROADMAP.md`, `DECISIONS.md`)
change only via `closeout-main-docs.md` on `main` after the merge;
`docs/economy.md` rides in on the branch (T002). Never run `cargo fmt`.

Model & effort: the session runs on `claude-opus-5` (Opus 5) because the Fable
allowance is low — the person's choice for this fallback; `CLAUDE.md`'s
Fallback clause names `claude-opus-4-8`, and the constitution is not changed
for it; the planner and
the sign-off ran at `opus` with the top-tier override dropped (the
constitution's Fallback clause). Implementation, the phase review and the sweep
run at `opus`. A task that isn't routine goes to a decision review, never
resolved by the session; a product question `spec.md` doesn't settle goes to
the person. Never infer the Fable budget from a successful dispatch — ask if
unsure, and log which tier actually ran. Every session-ending pause ends with a
continuation prompt (spec directory, files to read, where to resume,
involvement level, pause cadence, any model switch) in its own fenced block.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs
and reviewer invocations), any escape-hatch miss (a task the orchestrator had to
redo, and why). -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| **Fallback in effect** — the Fable allowance is low, so the session runs on **`claude-opus-5`** (Opus 5, the person's choice; `CLAUDE.md`'s Fallback clause names `claude-opus-4-8` — recorded here, constitution unchanged) and **experiment 2 is paused**: every implementer dispatch goes to `sdd-implementer` (opus, high), and the planner / sign-off run at `opus` with the top-tier override dropped. | — | — | header |
| Planning: draft (sdd-planner) | opus (override dropped, experiment 2 paused) | ~90K (planner's own estimate) | drafted; 3 tasks in 2 phases, no foundational phase, no per-task review; no product fork |
| plan + tasks sign-off (skeptical-reviewer) | opus (override dropped) | ~70K (measured return; reviewer's own ~65K in / 4K out) | signed off, 0 blocking, 5 notes N1–N5 |
| Planning: sign-off notes (sdd-planner, same context) | opus | ~20K (planner's own estimate of the delta) | N1 (headings measured for Mid/Core only; Core-profile centering eyeball added to the walkthrough), N2 (economy.md guard inventory added to T002), N3 (density-rule tension §5), N4 (model recorded as `claude-opus-5`), N5 (wrap-at-7 assertion) applied; both files marked signed off |
| T001 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~51K (measured return; implementer's own ~45K) | done first try; 408 unit + 6 integration passing, diff = shop.rs + economy.rs only, economy.rs adds only `region_name` + its test; **task-text off-by-one**: after Up+Enter (cursor at ±1) the 1st Down wraps to `+1` and the 7th lands back on `±1`, not `+1` — the test asserts `bought == listing()[..7]`, stricter than the wording |
| T002 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~29K (measured return; implementer's own ~25K) | done first try; docs-only diff (docs/economy.md, +18/−6), 408 unit + 6 integration passing, `the_full_pool_fits` grep empty; § The depth-gated pool left as is (nothing there says locked cards are hidden) |
| Phase 1 review (skeptical-reviewer) | opus | ~48K (measured return; reviewer's own ~40K) | signed off, 0 blocking, 6 notes — N1 task/plan text off-by-one confirmed (test right, stricter) → T003; N2 `LIST_ROWS` is hand-counted, not read by the draw loop (accepted by the plan; driver is the check); N3 draw doesn't clamp the cursor (unreachable; pre-existing); N4 draw re-filters listing per tier (fine at 15); N5 economy.md "dimming reads the same predicate" now half the story → T003; N6 stale `core_profile()` comment → T003 optional |
| Phase 1 driver walkthrough (orchestrator, scratch profiles, 139×31; real profile/settings/save backed up to scratchpad, checksummed) | — | — | Fresh profile: three groups, headings `Mid Rim  ·  reach the Mid Rim to unlock` / `Core  ·  reach the Core to unlock`, prices and owned counts on every row, cursor on `+1`; ↑ wraps to `±1`, six ↓ from `+1` land on `±1` (never into Mid); two buys of `+1` took 50 → 10 credits (owned ×4 → ×6), a third Enter refused at spendable 0 with nothing changed. Mid profile (Cinder, Scree beaten): Mid heading bare, ↑ wraps to `3&6`, Core still locked with owned ×1 showing. Core profile: all headings bare, ↑ wraps to `±1T`. Core profile → Start Campaign → New Campaign → Yes: `beaten` empty, credits 500 kept; Outfitter shows Mid and Core locked again with `+4`/`±6`/`±1T` owned ×1 intact, ↑ wraps to `±1`. Nothing clips (list rows 6–25, hint 27). **Not verifiable by the driver:** dim/bold attributes (the snapshot is text only) — the person checks the dimming. **Layout finding:** the list block starts at column 48 (labels and headings at 51) while the title centers near column 69; unlocked rows span ~48–79, so on the Core profile (no long heading) the list sits about 5–6 columns left of the title and balance. Real data restored twice (after each driver script); SHA-256 of profile.json, settings.json, saves/savegame.json match the pre-driver sums (81bb34d1…, 6fbd0816…, 59866b15…) |
