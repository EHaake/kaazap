# Spec: First-run onboarding & controls refinement — spec 023

**Status**: Approved (2026-09-13)
**Depends on**: spec 021 (wager & loss, run-over), spec 012 (economy, shop),
spec 009 (campaign map), spec 006 (control & input polish — partly superseded
here), spec 022 (Quick Play deals the standard deck)

## Summary

The campaign loop is complete but tells a new player nothing: a fresh profile
lands on the galaxy map with no idea that every match is staked, that a loss
forfeits the stake, that going broke ends the run, or that the shop is how you
get past the Mid Rim — and the first match opens with no idea what the keys
do. This spec adds the two ends of onboarding, both shown **once per profile**
and dismissed with one key, both **in the fewest possible words**:

1. **A first-campaign primer** over the galaxy map, the first time a profile
   enters it.
2. **A first-match popup** over the board, the moment a profile's first match
   actually starts.

The existing How to Play overlay stays the full reference and gains a short
campaign section so the primer's content can be found again. The opponent
select screen gets the one line spec 022 deferred: Quick Play deals the
standard deck.

Because the first-match popup lists the keys, this spec also carries a
**controls refinement pass** the person asked for at the same time, so the
popup describes the controls the game actually has:

- **1–4 select a hand card** (like ←/→) instead of playing it outright; a
  second key plays it. The separate "+ or −?" prompt that 1–4 used to open on
  a ± card is gone — the sign shown on the card (flipped with ↑/↓) is the
  sign played, as it already is for the cursor.
- **Space draws** on the player's turn. Playing the selected card is
  **Enter**, with **P** as its secondary key. Space keeps every other role it
  already has: confirm in menus and prompts, next round, new game,
  acknowledge a notice.

This supersedes spec 006's "Space plays the selected card" (its goal 2): the
most common in-play input is drawing from the dealer deck and moving on, so
that is what the spacebar does.

## Goals

1. **A first-campaign primer.** The first time a profile's galaxy map opens, a
   short overlay says what to pay attention to and nothing else: matches are
   staked and a loss forfeits the stake; going broke ends the run and resets
   you; the Outfitter sells the cards that get you past the Mid Rim; the ante
   rises as you go deeper. One key dismisses it; it never shows again for that
   profile.
2. **A first-match popup.** When a profile's first match starts, a popup
   explains the mechanics and the basic controls: draw toward 20 without going
   over, stand to lock your total, play a side card from your hand of four to
   adjust it, first to three rounds wins; the keys for draw, stand, pick a
   card, play it, and flip a ± sign. One key dismisses it; the match does not
   advance underneath it; it never shows again for that profile.
3. **Once per profile, surviving resets.** Both "seen" marks persist in the
   profile without a format version bump, and survive a run-over reset and
   New Campaign the way lifetime stats do. An existing profile from before this
   spec sees each piece once.
4. **Quick Play says it deals the standard deck.** One line on the opponent
   select screen.
5. **How to Play covers the campaign layer** in a few lines, so the primer's
   content remains findable after it is dismissed, and its controls line
   matches the new keys.
6. **Controls refinement.** 1–4 select; Enter or P plays; Space draws on the
   player's turn (and, like D, accepts a bust when over 20); the ± sign prompt
   after a direct key no longer exists; every on-screen hint and the controls
   overlay describe the new keys.
7. **No regression.** No change to scoring, round or match resolution, the
   AI, the economy, the wager prompt, settlement, or any save format; an
   existing profile and an existing mid-match save load as before.

## Non-goals (explicitly deferred)

- **An interactive tutorial** (a guided first match, step-by-step prompts).
  Two static texts and one key each.
- **Reopening the primer or the popup**, or a "show hints again" setting. How
  to Play is the reference; the once-only pieces are not menu items.
- **Any other onboarding surface**: no first-launch welcome on the start menu,
  no shop or deck-builder primer, no hint on the wager prompt beyond what it
  already shows.
- **A configurable / rebindable keymap**, vim-style keys, or mouse input
  (spec 006's non-goals stand).
- **Other control changes** than the two named: the emacs synonyms, Esc/X,
  Q, N, G, S, D, the play log key, and the arrow keys are unchanged.
- **The endgame award, the run summary, the run archive** — their own roadmap
  items.
- **Wording tuning by playtest.** The texts below ship as written unless the
  person changes them in this spec; later wording changes are chores.

## Entities

- **Primer** — the first-campaign overlay: a static text shown once over the
  galaxy map. Content in an asset text file like the existing overlays.
- **First-match popup** — the mechanics-and-controls text shown once over the
  board at a match's start. Static text, same treatment.
- **Seen marks** — two booleans in the profile: primer seen, first-match popup
  seen. Additive, default false, no version bump. Set when the piece is
  *dismissed* (not when it is shown), saved immediately.
- **Hand cursor** — the existing selection over the four hand slots, with its
  pending ± sign. 1–4 now move it; Enter and P commit it; Space no longer
  touches it.

## Key behavior

### The primer

- **When.** The galaxy map screen opens from the start menu — Start
  Campaign's no-progress path, Continue, a confirmed discard-and-enter, or a
  confirmed New Campaign — and the profile's primer mark is unset. (New
  Campaign is only reachable from the start menu; there is no map-side New
  Campaign.) Not on Back from the shop or the deck-builder — those return to
  a map the player has already seen — and not from a match's game-over
  acknowledgement.
- **Precedence.** If the map opens broke (an existing profile from before this
  spec could), the run-over notice shows instead and the primer waits for the
  next time the map opens. One modal at a time.
- **While up.** The map is visible underneath. No node can be launched, no
  wager prompt, shop or deck-builder opens until it is dismissed.
- **Dismissal.** Enter, Space or Esc; every other key is ignored, matching the
  run-over notice. Dismissing sets and saves the mark. Quitting the app with the
  primer up leaves the mark unset, so it shows again next time.
- **Text**, verbatim (the title row centered like the other overlays):

```
=====  Your first campaign  =====

Every match is played for a stake you choose.
Win and it comes back doubled. Lose and it's gone.
Go broke and the run ends: starter deck, seed purse.

The Outfitter (b) sells the cards that get you
past the Mid Rim. The ante rises as you go deeper.

                 Enter to continue
```

### The first-match popup

- **When.** A match *starts* — Quick Play or campaign, whichever the profile
  plays first — and the profile's first-match mark is unset. The board is
  drawn underneath with the opening hands dealt. A match resumed from a save
  is not a start and never shows it.
- **While up.** The match does not advance: no player key reaches the game,
  and if the opponent is to act first it does not act until the popup is
  dismissed. (Once dismissed, the opponent's usual thinking pause runs from
  then.)
- **Dismissal.** Enter, Space or Esc; every other key ignored. Dismissing sets
  and saves the mark. Quitting with the popup up leaves the mark unset.
- **Text**, verbatim, describing the keys *as refined below*:

```
=====  How a match works  =====

Draw toward 20 without going over.
Stand to lock in your total.
Play a side card from your hand of four to adjust it.
First to 3 rounds wins the match.

 Space  draw          S  stand
 1-4 or ←/→  pick a card    Enter  play it
 ↑/↓  flip a ± card's sign

                   Enter to begin
```

### Seen marks and persistence

- Two additive, serde-defaulted booleans on the profile, default false, saved
  with the profile. `PROFILE_VERSION` stays 1; the mid-match save format is
  untouched.
- A profile from before this spec loads with both unset and sees each piece
  once (ruling C). A fresh profile is the same.
- The run-over reset and New Campaign preserve both marks, alongside lifetime
  stats (ruling A). Nothing else in the reset changes.

### Quick Play line and How to Play

- **Opponent select** gains one line above its controls hint:
  `Quick Play deals the standard deck.` It fits the minimum terminal with the
  full roster, blurb and hint.
- **How to Play** gains a campaign section at the end and its controls line
  changes to the new keys. Proposed text (the person may edit):

```
Campaign: every match is staked. Win and it
comes back doubled; lose and it's gone. Go
broke and the run resets. The Outfitter sells
cards; the ante rises as you go deeper.

1-4 or ←/→ pick a card, Enter plays it,
Space draws, S stands. ? closes.
```

### Controls refinement

On the player's turn:

| Key | Before | After |
|---|---|---|
| 1 2 3 4 | Play that hand card (a ± card then asked + or −) | Select that hand card (empty slot: nothing) |
| ← → | Select a hand card | unchanged |
| ↑ ↓ | Flip the selected ± card's sign | unchanged |
| Enter | Play the selected card | unchanged |
| Space | Play the selected card | Draw (over 20: accept the bust, like D) |
| P | — | Play the selected card |
| D | Draw | unchanged |
| S | Stand | unchanged |
| h / l / + / − / c | Answer or cancel the ± prompt | gone (the prompt no longer exists) |

- A ± card plays with the sign shown on it; ↑/↓ flips it before playing. There
  is no separate sign prompt anywhere, so no player-facing state waits for a
  + or − answer.
- Everywhere else Space is unchanged: menus and modals confirm, round end
  advances, game over starts a new game or acknowledges, the run-over notice
  and the two new pieces dismiss.
- The board's turn hint, the in-game controls overlay (`?`) and How to Play
  name the new keys; nothing on screen still says Space plays a card or that
  1–4 play one.
- Whether the engine keeps an internal pass-through for sign-choice cards is a
  plan decision; the acceptance criteria only require that no player-facing
  prompt exists and that the engine's rules and tests for card effects are
  unchanged.

## Design requirements

- **Overlay convention.** Both pieces are modals over their screen — one modal
  at a time, neutral default, the same bordered box the other text overlays
  use, monochrome.
- **Density and breathing room** (design brief; `CLAUDE.md`). The dismiss line
  is the acted-on element: it gets an empty row above and below it; every
  other row stays compact. The box pads its content evenly — one empty row
  above and below — and never a slab of empty rows under the text. The box's
  own bottom padding row may serve as the row below the dismiss line; do not
  add a second one.
- **Fewest possible words.** The texts above are the ceiling, not a floor to
  pad out.
- **Fits the minimum terminal** (139×31) with the map or board drawn
  underneath.
- **Sound.** Dismissal plays the menu-select sound, like acknowledging the
  other notices.

## Acceptance criteria

- [x] A fresh profile's first entry to the galaxy map shows the primer with
      the text above; Enter, Space or Esc dismisses it; no node, wager, shop
      or deck-builder opens while it is up; it does not show
      on any later map open, including after a run-over reset and after New
      Campaign.
      *Evidence: `map_entry_modal` raises `Modal::Primer` only when
      `from_menu && !profile.primer_seen()` and dismissal marks the profile —
      `map_entry_modal_prefers_run_over_then_primer`,
      `the_primer_swallows_map_keys`,
      `onboarding_texts_are_the_spec_text_and_fit`; the mark survives the reset
      (`reset_to_starter_wipes_the_run_but_preserves_lifetime_stats_and_onboarding_marks`),
      so neither a run-over nor New Campaign shows it again.*
- [x] A fresh profile's first match — Quick Play or campaign — shows the
      popup with the text above over the dealt board; the match does not
      advance while it is up, including when the opponent acts first; Enter,
      Space or Esc dismisses it; it does not show for any later match, a
      resumed match, or after a reset.
      *Evidence: the popup is raised at the one match-start seam, guarded by
      `!profile.first_match_seen()` (`src/app.rs:821`), which Continue's resume
      path never reaches; `tick` freezes the engine while it is up (`let held =
      matches!(self.modal, Some(Modal::FirstMatch))`) — tested by
      `the_first_match_popup_holds_the_match_and_swallows_play_keys`,
      `onboarding_dismissed_on_enter_space_or_esc_only`,
      `onboarding_texts_are_the_spec_text_and_fit`.*
- [x] Both marks round-trip through `profile.json`; a profile document without
      them loads with both unset; `PROFILE_VERSION` is 1 and the match save
      format is untouched; quitting with either piece up leaves its mark unset.
      *Evidence:
      `onboarding_marks_default_unset_round_trip_and_load_unset_from_older_documents`
      (which also asserts `PROFILE_VERSION == 1`); `git diff main --stat` shows
      no `src/save.rs`, and `SAVE_VERSION` is 1; the marks are written only in
      `handle_onboarding_input`, on dismissal, so quitting under either leaves
      them unset.*
- [x] If the map opens broke, the run-over notice shows and the primer does
      not; the primer shows on the next map open.
      *Evidence: `map_entry_modal_prefers_run_over_then_primer` — `(true,
      true)` and `(true, false)` both give `Modal::RunOver`, `(false, true)`
      gives `Modal::Primer`, and the mark is set only on dismissal, so the
      primer is still due at the next open.*
- [x] The opponent select screen shows "Quick Play deals the standard deck."
      above its hint, on-frame at the minimum terminal with the full roster.
      *Evidence: `QUICK_PLAY_NOTE` drawn `Muted` at `y + 4` with the hint moved
      to `y + 5` and the layout footer widened 6 → 7;
      `the_full_roster_and_footer_fit_the_minimum_terminal` and
      `preview_panel_is_on_frame_and_clear_of_the_list_at_the_minimum`.*
- [x] How to Play contains the campaign section and the new controls line;
      the in-game controls overlay and the board's turn hint name Space as
      draw, Enter/P as play, and 1–4 as select; no on-screen text says Space
      plays a card or that 1–4 play one.
      *Evidence: `help_texts_name_the_new_keys_and_nothing_old`,
      `help_texts_fit_the_minimum_terminal_unclamped`,
      `turn_hints_fit_the_status_band`, `status_never_shows_a_sign_prompt`;
      and the T009 greps of
      `assets`, `src/board.rs`, `src/app.rs` return only "Space draw", "Space /
      D  Draw", "Over 20: Space, D or S accepts the bust" and 1–4 "pick" /
      "Select" lines — no line says Space plays a card or that 1–4 play one.*
- [x] On the player's turn: 1–4 move the selection to that slot (an empty slot
      changes nothing) and play nothing; Enter or P plays the selected card
      with the sign shown on it, for fixed, ±, flip and tiebreaker cards
      alike; Space draws, and over 20 accepts the bust like D; no key opens a
      + or − prompt. Space at round end, game over, menus, modals and notices
      behaves as before.
      *Evidence:
      `cursor_select_lands_on_an_occupied_slot_and_resets_the_sign_like_a_move`,
      `cursor_select_ignores_an_empty_or_out_of_range_slot`,
      `turn_key_binds_the_spec_023_keys`,
      `cursor_confirm_plays_a_fixed_card_immediately`,
      `cursor_confirm_plays_sign_card_at_the_pending_sign`,
      `cursor_confirm_tiebreaker_commits_as_pending_sign`,
      `cursor_confirm_flip_card_applies_and_does_not_prompt`,
      `space_draws_on_the_players_turn_and_advances_at_the_pauses`,
      `space_over_twenty_accepts_the_bust_like_d`,
      `number_and_sign_keys_map_to_nothing_on_the_players_turn`,
      `sign_phase_maps_no_keys`.*
- [x] Unit tests cover: the seen marks' defaults, round trip and survival of
      the reset; the primer and popup showing exactly once and blocking input
      underneath; 1–4 selecting; Enter/P playing each card kind at the shown
      sign; Space drawing on the player's turn and advancing at the pauses;
      the existing card-effect and resolution tests unchanged.
      *Evidence: the tests named above, all in the 386-test suite; `git diff
      main -- src/game.rs | grep '^@@'` shows engine hunks only in
      `game_action_from_key`, the added `restart_opponent_pause`, and `mod
      tests` — every `sign_*`, card-effect and resolution test is untouched.*
- [x] No change to `game.rs` rules (card effects, scoring, resolution), the
      AI, the economy, the wager prompt, settlement, or `save.rs`; `cargo
      build` has no new warnings; `cargo test` is green.
      *Evidence: `git diff main --stat` touches no `save.rs`, `player.rs`,
      `card.rs`, `economy.rs`, `wager.rs`, `campaign_map.rs`, `tests/balance.rs`,
      `Cargo.toml` or `Cargo.lock`; `cargo build --all-targets 2>&1 | grep -c
      warning` is 0 on this branch and 0 on `main`; three consecutive `cargo
      test -q` runs were green (386 + 6 passed, 0 failed, 1 ignored).*
- [x] Attested in play by the person: a fresh profile sees the primer once on
      the map and the popup once at its first match; the new keys feel right;
      an existing profile sees each piece once and keeps everything else.
      *Evidence: attested by the person in play on 2026-09-14/15 — Phase 1
      (controls) and Phase 2 (onboarding, including the T006a centering fix);
      the Phase 2 driver walkthrough is logged in this spec's `tasks.md` tier
      log.*

## Resolved decisions

Ruled with the person on 2026-09-13, on the recommendations as proposed:

- **A — The seen marks survive resets.** A run-over and New Campaign wipe the
  run but keep the marks, as they keep lifetime stats; a player who has read
  the rules is not re-taught on the way back in.
- **B — The first match of either mode** shows the popup: the mechanics and
  controls are identical in Quick Play and campaign, so whichever comes first
  gets it.
- **C — Existing profiles see both pieces once.** The profile format could
  treat a missing mark as seen; the person chose to show them, so an existing
  profile can try them without a wipe, at the cost of one key each.
- **D — The Quick Play line lives on the opponent select screen**, where Quick
  Play is chosen, not in How to Play.
- **E — How to Play gains a short campaign section**, so it is genuinely the
  full reference once the primer is gone.
- **Timing (person's words).** The primer shows the first time through the
  campaign, on first entering the galaxy screen; the gameplay popup once the
  first game actually starts.

Ruled the same day, on the recommendations as proposed:

- **F — The controls refinement rides in this spec**, not a separate one or
  a chore: the popup's key list depends on it, and retiring the ± prompt
  touches the engine's phase list, which the chore lane excludes. Spec 006's
  goal 2 is superseded; `DECISIONS.md` records it at the merge.
- **G — P is the secondary play key.** Free on the player's turn, mnemonic,
  and away from the draw/stand hand (D, S, Space). Enter stays primary.
- **H — The ± prompt is retired outright** rather than kept for the direct
  keys: with 1–4 selecting, every play goes through the cursor, whose ↑/↓
  sign is already the answer; a second way to answer the same question would
  be the indirection the constitution tells us to cut.
