# Spec: New Campaign option (start over) — spec 014

## Summary

"Start Campaign" always **resumes** the player's existing progress — there is no
way to start a fresh campaign. A player who has cleared some worlds and wants to
begin again is stuck (a playtester hit exactly this: they picked Campaign expecting
a fresh start, saw their prior progress, and assumed a loss had been miscounted).

This spec adds the choice: when the player selects Campaign and **cleared progress
already exists**, offer **Continue** or **New Campaign**. New Campaign is a **full
fresh start** — it wipes campaign progress, credits, and the collection/deck back
to the starter, keeping audio/settings. Because it's destructive, it's behind an
explicit confirmation.

No engine, board-renderer, or save-format change — a menu-flow addition plus a
profile reset over shipped patterns.

## Goals

1. **Offer the choice.** Selecting Campaign with existing cleared progress presents
   Continue (resume where you are) vs New Campaign (start over). With no progress
   yet, Campaign behaves exactly as today (opens the map directly).
2. **New Campaign = full fresh start.** It resets campaign progress, credits, and
   the collection/deck to the starter, so the depth-gated economy stays meaningful
   (you re-earn cards). Audio/other settings are untouched.
3. **Guard the destructive action.** New Campaign requires a confirmation
   defaulting to the safe choice, and clearly states that progress, credits, and
   cards will be erased.
4. **No regression.** Continue resumes exactly as before (including the existing
   mid-match-save discard on entry); every other screen, rule, and save behaves as
   before; the save *format* is unchanged.

## Key behavior

### Choosing Campaign

- **No cleared progress** (a fresh or never-won campaign) → the map opens directly,
  as today. (Continue and New Campaign would be identical, so no prompt.)
- **Cleared progress exists** → a small panel offers **Continue** (default) and
  **New Campaign**, dismissible with Esc.
  - **Continue** → resume: the campaign map at your current position (unchanged
    behavior, including discarding a stray in-progress match save on entry).
  - **New Campaign** → a confirmation ("this erases your progress, credits, and
    collection", default **No**). Confirming wipes the profile to the starter and
    opens a fresh map (0 cleared, only the first world unlocked); declining returns
    without changing anything.

### What a fresh start resets

Campaign progress, credits, and the collection/deck return to the starter (the
same state a brand-new profile has). Audio and other settings live outside the
profile and are kept. Any in-progress match save is discarded.

## Design requirements

- **The reset is a single, testable profile operation** — resetting to the starter
  profile, not an ad-hoc field-by-field clear, so it can't drift from what a new
  profile actually is.
- **Follow the modal convention.** The choice and the confirm are `Modal`s over the
  menu (one open at a time), mirroring the existing discard-a-save confirmation —
  ←/→ to choose, Enter to commit, Esc to cancel, the safe option default.
- **The board and engine stay untouched.** This is menu/profile only.

## Acceptance criteria

- [x] Selecting Campaign with cleared progress shows a Continue / New Campaign
      choice; with no progress it opens the map directly (unchanged). *(driver: the
      real (has-progress) profile showed the Continue/New panel; a wiped no-progress
      profile opened the map directly with no modal. `has_progress` gate +
      `has_progress_is_false_until_an_opponent_is_beaten`.)*
- [x] Continue resumes the campaign at the current position exactly as before
      (including the mid-match-save discard on entry). *(`enter_campaign_continue`
      is the extracted pre-014 body — reviewer-verified byte-identical, incl. the
      `has_save` discard confirm.)*
- [x] New Campaign requires a confirm (default No); confirming resets campaign
      progress, credits, and collection/deck to the starter and opens a fresh map;
      declining changes nothing. *(default No — `draw_confirm_new_campaign` +
      `confirm_choice_commits_only_on_enter_with_yes` (mutation-checked: only
      Enter+Yes wipes); driver: Yes → 0/8 map, credits ◈ 0, collection back to the
      13-card starter; Esc left the profile intact (checksum).)*
- [x] Audio/settings are preserved across a reset; the save format is unchanged
      (no `PROFILE_VERSION` bump). *(`reset_to_starter_wipes_everything_back_to_a_new_profile`
      — serialized-equality with `Profile::default()`; settings live in a separate
      struct/file; `PROFILE_VERSION` stays 1.)*
- [x] `cargo test` green (the reset, the progress check, and the modal-flow decision
      test); `cargo build` no new warnings; legible at 89×31; no panics. *(249
      passed, 0 failed; build 0 warnings; driver `CONFIRM_MIN` snapshot at 89×31; no
      panics across the sweep.)*

## Resolved decisions

- **Full fresh start** (human-ruled) — New Campaign wipes progress, credits, and
  collection/deck to the starter, rather than an NG+-style replay that keeps your
  arsenal. Keeps the early game and the depth-gated economy meaningful.
- **Offered as a choice at Campaign entry** (human-ruled) — a Continue / New
  Campaign panel when progress exists, not a separate top-level menu item.
- **Not NG+** — keeping cards/credits across a reset stays with the roguelike mode
  (spec E).
