# Spec: Compact layout below 139 columns — spec 026

**Status**: Approved (2026-09-17; AC 3 corrected at sign-off, B1). Rulings Q1 B, Q2 A, Q3 B, Q4 A (with C's
rule), Q5 A.
**Depends on**: spec 002 (the fixed, centered board block), spec 016 (the
opponent presence panel and the 139-column minimum), spec 017 (the banter
line in the panel), spec 021 (the escrowed stake shown in the panel), spec
009 (the campaign map's full-terminal layout)

## Summary

The game needs a terminal at least 139 columns wide, because the opponent
presence panel — portrait, name, banter line, round pips, stake — sits
beside the fixed 89-column board with an equal, empty margin on the other
side. That is large for a general audience; before spec 016 the game played
at 89 columns.

This spec brings the minimum back down to **89×31**. At **139 columns or
wider nothing changes**. Narrower than that, the match board draws
**without the panel** — the same board, only the panel is gone — and the one
piece of panel information that matters to play, the **stake at risk**,
moves onto the board itself. The opponent select preview and the campaign
map's portrait rail **keep their portraits at every width**; the portraits
stay part of the game even when the live match window has no room for one.
The layout is chosen by width alone, switches live across a resize, and
has no setting.

No engine, AI, economy, save-format, or balance change. The wide layout
is untouched.

## Goals

1. **A terminal between 89 and 138 columns wide plays the whole game.**
   Every screen — menu, overlays, opponent select, map, Outfitter, deck
   builder, records, play log, wager prompt, the match itself — fits
   89×31 with nothing clipped.
2. **The compact board is the same board.** Cards, hand, header, status
   band and popups are exactly as they are at 139; only the panel is
   absent. The board stays centered in whatever width there is.
3. **A staked match always shows its stake.** On the compact board the
   stake sits at the right end of the status band's upper row (the row
   the over-20 alert uses, left-aligned and short, so the two never
   collide), in the panel's own `Stake ◈ N` form. The **rule** is that the
   stake is visible for the whole staked match without growing the board
   block; the exact spot is the proposal and may move if the walkthrough
   finds a collision. Quick Play shows no stake line, as today.
4. **Portraits survive where they fit.** The opponent select screen keeps
   its preview panel and the campaign map keeps its portrait rail below
   139 columns, and the map still reads cleanly at 89: no planet or label
   clipped or drawn over the rail.
5. **A live resize across 139 switches layouts mid-match, both ways,**
   with the match untouched. The too-small screen, the startup error and
   the README quote the new minimum.

## Non-goals (explicitly deferred)

- **A third layout between 114 and 138 columns** (panel on the right only,
  board shifted left). Two layouts, one threshold.
- **Any portrait, name panel or banter line on the compact board.** The
  opponent's name is already in the board header; banter events still
  fire (audio, log) but no line is shown below 139.
- **A setting** to force the compact board on a wide terminal.
- **Changing the wide layout** — the board's position, the panel, the
  reserved left margin — in any way (the one exception, ruled at the Phase 1
  pause: the panel's stake row now stays through the game-over popup, Q6 A).
- **A player-status panel** in the reserved left margin (still reserved).
- **Engine, AI, economy, wager, save-format or balance-data changes.**

## Entities

- **Wide layout** — the layout that exists today: the board centered with
  the presence panel in the right margin, from 139 columns up.
- **Compact layout** — the board alone, centered, from 89 to 138 columns.
- **Threshold** — 139 columns, the width the wide layout needs. The layout
  is wide iff the terminal is at least that wide.

## Key behavior

### Choosing the layout

- The layout is a function of the terminal width, evaluated wherever the
  board geometry is built today (startup and resize). At or above 139
  columns: wide. Below, down to 89: compact. Below 89 columns or 31 rows:
  the existing too-small screen, now quoting `89 x 31`.
- A resize from 139 to 138 mid-match redraws the next frame as the compact
  board; a resize back restores the panel. Game state, the hand cursor, an
  open help overlay and the play log are untouched either way — the
  existing resize path already rebuilds the presentation from the size.

### The compact board

Sketch at 89×31, staked match (the board is unchanged; the one new line is
the stake at the right of the status band's upper row):

```
  Player: You                Score: 14 │  Opponent: Rix              Score: 9
                          Rounds won: 1 │                          Rounds won: 0
  …                                     │  …
  [hand]                                │  [hidden hand]

  Over 20!                                                        Stake ◈ 40
  ↑/↓ flip · Enter play · Space draw · S stand
```

- No panel, no portrait, no banter line, no pips. The header's `Rounds
  won` line already carries what the pips showed.
- The stake line appears only on a staked campaign match, for the whole
  match (from the first frame to the game-over popup), in `Stake ◈ N`,
  drawn `Strong` as in the panel.
- The centered round-outcome and game-over popups draw as today.

### The other portrait surfaces

- **Opponent select**: the preview panel draws at every width, at its
  current position right of the list.
- **Campaign map**: the portrait rail draws at every width; the node
  field shrinks to clear it as today. At 89 columns every planet and
  label is on frame and clear of the rail.

## Design requirements

- Monochrome only; no new emphasis level. The stake line reuses the
  panel's `Strong`.
- The *Density and breathing room* rule: the compact board adds no rows;
  the stake shares an existing row.
- The board block keeps its fixed size and its centering in both modes,
  so crossing the threshold never moves a card.

## Acceptance criteria

- [x] The minimum terminal is 89×31: `Config::min_size()` returns it, the
      startup error and the too-small screen quote `89 x 31`, and a test
      pins the figure.
- [x] The board layout at 138 columns has no panel and at 139 has the
      panel at spec 016's position, pinned by a test; the board's own
      rects are identical in both, apart from the centering offset.
- [x] Every screen fits 89×31: each existing "fits the minimum terminal"
      test (board, overlays, Outfitter, deck builder, opponent select, map,
      records) runs at 89×31 as well as 139×31 — the wager prompt's test
      reads `Config::min_size()` and so measures 89, and its 139 case
      follows from a centered, content-sized box, with `wager.rs` itself
      untouched (see the last criterion) — and a driver
      walkthrough at 89×31 shows menu, How to Play, opponent select, map,
      Outfitter, deck builder, records, play log, wager prompt and a match
      with nothing clipped.
- [x] On a staked campaign match at 89 columns the `Stake ◈ N` line is on
      the board from the first frame through the game-over popup, without
      the board block growing; a Quick Play match at 89 shows no stake line.
- [x] The over-20 alert and the stake line never overlap: a test draws both
      on the compact board and checks the cells.
- [x] Resizing from 139 to 138 columns mid-match and back keeps the match,
      the hand cursor and any open help overlay, and shows the panel only at
      139 — pinned by a test on the app's resize path and by the driver
      walkthrough.
- [x] Opponent select and the campaign map draw their portraits at 89
      columns; a map fit test at 89×31 shows every planet and label on frame
      and clear of the rail.
- [x] The wide layout at 139×31 and larger is unchanged: the existing
      board-layout tests pass untouched and the 139×31 driver walkthrough
      matches spec 016's screen — except that the panel's stake row now stays
      through the game-over popup (Q6 A).
- [x] `Readme.md`'s terminal-size paragraph describes the new minimum and
      the two layouts.
- [x] No change to `card.rs`, `game.rs`, `player.rs`, `save.rs`,
      `profile.rs`, `economy.rs`, `wager.rs`, `tests/balance.rs`,
      `Cargo.toml` or `Cargo.lock`; `PROFILE_VERSION` and `SAVE_VERSION`
      stay 1.

## Resolved decisions (the person, 2026-09-17)

- **Q1 B — the compact board drops the portrait and banter but keeps the
  stake.** Money at risk stays on screen (the wager-warning chore of
  2026-09-17 shows the same priority); banter without a face reads oddly in
  a status band spec 017 deliberately kept mechanical.
- **Q2 A — chosen by width alone, live, no setting.** The resize path
  already rebuilds the board view from the size; a setting would be a
  second way to reach a state width already reaches.
- **Q3 B — the select preview and the map rail keep their portraits at
  every width.** The person wants the portraits to stay part of the game
  even where the match window can't hold one; both surfaces physically fit
  at 89. (The recommended one-rule-everywhere option was declined.)
- **Q4 A with C's rule — the stake goes on the status band's upper row,
  right-aligned; the requirement is visibility for the whole staked match
  without growing the board block.** The planner may move it if the
  walkthrough finds a collision.
- **Q5 A — the threshold stays 139.** No third layout between 114 and 138;
  the wide layout is exactly as spec 016 shipped it.
- **Q6 A — the settled stake shows on the game-over frame, both layouts**
  (ruled at the Phase 1 pause, after finding F1). The match settles on the
  tick that draws the game-over popup, so the escrow reads zero there; the
  board is handed the settled amount instead, on the compact band and on the
  wide panel alike.

## Acceptance evidence (T005 close-out, 2026-09-18)

Checked off by the orchestrator against `cargo test -q` (three consecutive
runs, 424 + 6 passed, 0 failed, 1 ignored, warning count 0 on both `main` and
the branch) and the tier log's walkthrough rows in `tasks.md`.

1. **Minimum 89×31** — `config_min_size_is_the_board_block`,
   `the_wide_threshold_is_the_board_plus_panel_margins`,
   `the_too_small_screen_quotes_the_new_minimum`; the bail's `{} x {}` seen in
   the T001 diff and at the Phase 1 walkthrough (a 60×20 start).
2. **Panel at 139, none at 138, same board rects** —
   `the_panel_appears_at_the_threshold_and_not_below`,
   `board_rects_are_identical_across_the_threshold_apart_from_centering`.
3. **Every screen fits 89×31** — every fit test loops `Config::fit_sizes()`
   (T001, T003); `wager.rs`'s test reads `min_size()`, file untouched (diff
   check below); Phase 2 driver walkthrough at 89×31 (tier log row): menu,
   help, How to Play, Settings, Records, opponent select, map, Outfitter, deck
   builder (hint line whole and centered), play log, wager prompt, a match.
4. **The stake for the whole staked match** —
   `the_compact_board_carries_the_stake_clear_of_the_alert`,
   `quick_play_shows_no_stake_line`, the T002a App-level test; the Phase 1
   walkthrough found the stake missing at the game-over popup (finding F1,
   fixed as T002a), and the Phase 2 walkthrough saw `Stake ◈ 30` from the
   first frame through the game-over popup, and Quick Play without it.
5. **Alert and stake never overlap** — the cells between are asserted blank in
   `the_compact_board_carries_the_stake_clear_of_the_alert`; the over-20
   moment was watched at 89 in both walkthroughs.
6. **Resize across 139 keeps the match** —
   `a_resize_across_the_threshold_keeps_the_match_and_toggles_the_panel`;
   the Phase 2 walkthrough resized 139→138→139 mid-match with a moved cursor
   and an open `?` overlay.
7. **Portraits at 89** — `preview_panel_is_on_frame_and_clear_of_the_list_at_the_minimum`,
   `campaign_map_layout_fits_the_minimum_terminal`,
   `the_campaign_map_is_legible_at_the_minimum_terminal` at both sizes; seen at
   89 in both walkthroughs.
8. **Wide layout unchanged** — the existing layout tests pass untouched,
   `the_wide_board_keeps_the_stake_in_the_panel`; the 139×31 walkthrough
   matched spec 016's screen, with the panel stake staying through the
   game-over popup (Q6 A).
9. **README** — the terminal-size paragraph replaced in T004.
10. **No forbidden change** — `git diff main...HEAD --stat` over `src/main.rs
    src/card.rs src/game.rs src/player.rs src/save.rs src/profile.rs
    src/economy.rs src/wager.rs src/campaign.rs src/campaign_map.rs
    tests/balance.rs Cargo.toml Cargo.lock` is empty; `git diff main...HEAD --
    src/portrait.rs` adds only `stake_line`, its call and one doc line;
    `SAVE_VERSION` and `PROFILE_VERSION` both read 1.
