# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 026)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T005, ready to paste on `main` after the merge
(the same close-out shape specs 020–025 used). Nothing here is applied by the
spec branch.

Spec 026 amends no rule `CLAUDE.md` states — see §3. Nothing to apply there.

Line numbers below are **`main`'s** at the time of drafting (2026-09-18,
`main` at `6115fcf`, *Record the wager-warning chore in the roadmap and
decisions log*). `main` already carries spec 025's close-out and the
wager-warning chore's entries, neither of which is touched here. Each edit
quotes its anchor text verbatim, which is what to match on; every anchor below
was re-checked with `grep -c` against `git show main:<file>` at `6115fcf` and
is unique there.

Applying: never chain a file edit, a branch switch and a commit in one shell
command (spec 025's miss) — switch to `main`, edit, verify the anchors again,
then commit.

---

## 1. `ROADMAP.md` — four edits, one judged and left

`grep -n "139" ROADMAP.md` on `main` returns four hits (lines 210, 420, 526,
645). Read and judged: **210 and 526 describe 139×31 as the minimum** in the
present tense and get the repo's inline supersession annotation; **420 is a fit
claim** (spec 025's Outfitter "fit 139×31, pinned by a test") that stays true —
it now fits 89×31 too, and the spec 026 Shipped entry says so — left alone;
**645 is the backlog entry this spec ships**.

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Locked cards in the Outfitter (spec 025)**
entry, which ends with these two lines (currently `ROADMAP.md` lines 420–421,
just before the blank line and `## Backlog`):

```markdown
  and hint fit 139×31, pinned by a test and by a driver walkthrough at all
  three depths and across a New Campaign. `docs/economy.md` re-synced.
```

Insert after them:

```markdown
- **Compact layout below 139 columns** (spec 026) — the minimum terminal is
  **back to 89×31**, where it was before spec 016. **139 is now a threshold,
  not the minimum**: at 139 columns and wider the match draws exactly as spec
  016 shipped it (the board centered, the opponent-presence panel — portrait,
  name, banter, round pips, stake — in the right margin); from 89 to 138 the
  **same board draws alone**, centered, with no panel, no portrait, no banter
  and no pips (the header's `Rounds won` line already carries what the pips
  showed). The one piece of panel information that matters to play, the
  **stake at risk**, moves onto the board: a staked campaign match shows
  `Stake ◈ N` right-aligned on the status band's upper row, drawn `Strong`,
  from the first frame through the game-over popup, sharing the over-20
  alert's row (≥ 40 blank cells between them, pinned by a test) so the
  31-row board block does not grow; Quick Play shows no stake line. The
  layout is **chosen by width alone, live, with no setting** — `BoardLayout`'s
  panel is an `Option<Rect>`, `Some` iff `cols >= WIDE_LAYOUT_MIN_WIDTH` (the
  renamed `IN_MATCH_MIN_WIDTH`, same value and derivation), so a resize from
  139 to 138 mid-match redraws the next frame compact and a resize back
  restores the panel, keeping the match, the hand cursor and an open help
  overlay (pinned on `App::resize`). The opponent-select preview and the
  campaign map's portrait rail **keep their portraits at every width**; at
  89 every planet and label is on frame and clear of the rail (pinned). Along
  the way: the stake row on the wide panel now **stays through the game-over
  popup** on both layouts (Q6, `App::stake_to_show` reads the settled amount
  at `GameOver`; it had been blank there since spec 021), and the play log's
  width floor rose 40 → 52 so the compact log is the same box as the wide one.
  Every "fits the minimum terminal" test now loops `Config::fit_sizes()` —
  89×31 and 139×31 — and the too-small screen and the startup error both quote
  `89 x 31`. No engine, AI, economy, wager, save-format or balance change:
  `main.rs`, `card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`,
  `economy.rs`, `wager.rs`, `campaign.rs`, `campaign_map.rs`,
  `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are untouched and
  `PROFILE_VERSION` / `SAVE_VERSION` stay 1. Driver walkthroughs at 89×31 and
  139×31 (every screen, a staked match, Quick Play, the resize across the
  threshold and the too-small screen) attested. `Readme.md`'s terminal-size
  paragraph re-synced.
```

### 1b. Inline annotation on the spec 016 Shipped entry (currently lines 209–211)

The **Opponent portraits (spec 016)** entry describes the grown minimum in the
present tense. Using the repo's inline-supersession convention, replace:

```markdown
  logic, with the generic as the never-blank fallback). The always-on panel **grew the
  minimum terminal 89×31 → 139×31** — the board is unchanged and still centered, the panel
  sits in the right margin, and the equal left margin is reserved empty for a future
```

with:

```markdown
  logic, with the generic as the never-blank fallback). The always-on panel **grew the
  minimum terminal 89×31 → 139×31** — **superseded by spec 026**, which makes 139 a
  threshold rather than the minimum (the minimum is 89×31 again; below 139 the board
  draws without the panel and a staked match's stake moves onto the status band) — the
  board is unchanged and still centered, the panel
  sits in the right margin, and the equal left margin is reserved empty for a future
```

### 1c. Inline annotation on the backlog's shipped-marked *Opponent portraits* bullet (currently lines 525–527)

Replace:

```markdown
  in opponent-select and on the campaign map. As anticipated it needed a layout region and
  a grown minimum terminal (**89×31 → 139×31**), and stayed monochrome by construction (no
  color path), sanctioned by a `design/brief.md` bounded-exception amendment. The mandated
```

with:

```markdown
  in opponent-select and on the campaign map. As anticipated it needed a layout region and
  a grown minimum terminal (**89×31 → 139×31** — **superseded by spec 026**, which brought
  the minimum back to 89×31 and made 139 the threshold above which the panel draws), and
  stayed monochrome by construction (no
  color path), sanctioned by a `design/brief.md` bounded-exception amendment. The mandated
```

### 1d. Mark the backlog entry this spec shipped (currently lines 645–651)

Under "### Onboarding, endgame & release readiness", replace the whole
**A compact layout below 139 columns** bullet:

```markdown
- **A compact layout below 139 columns.** The minimum terminal grew to 139×31
  with the portrait panel (spec 016), which is large for a general audience.
  A layout that drops the presence panel (portrait + banter) when the terminal
  is narrower — back to the pre-016 89-column board — would widen who can play
  at all. Its own spec: a second `BoardLayout` arm and the panel-less draw
  path, no engine change; the portraits and banter stay as they are above the
  threshold.
```

with:

```markdown
- **A compact layout below 139 columns** — ✅ **Shipped (spec 026** — see
  Shipped above and `DECISIONS.md`). The minimum terminal had grown to 139×31
  with the portrait panel (spec 016), which is large for a general audience.
  Below 139 columns the match now drops the presence panel (portrait, banter,
  pips) and draws the pre-016 89-column board alone, with a staked match's
  stake moved onto the status band; the minimum is 89×31 again. Chosen by
  width alone, live across a resize, no setting; the portraits and banter stay
  exactly as they were at 139 and above, and the select preview and the map
  rail keep their portraits at every width. No engine change.
```

**Leave alone** line 420 (spec 025's "fit 139×31, pinned by a test" — still
true, and a fit claim rather than a minimum) and the **wager-warning chore**
entry (line 571).

---

## 2. `DECISIONS.md` — three edits

`grep -n "139\|89×31\|IN_MATCH_MIN_WIDTH" DECISIONS.md` on `main` returns five
hits (lines 228, 406, 409, 729, 1224). Read and judged: **228** (spec 009's map
labels "at the 89×31 minimum") is true again and stays; **1224** (spec 025's
`list_left` at 139 columns) is a measurement, not a minimum, and stays;
**406–411** is the ruling spec 026 supersedes (2a); **729** (spec 021's stake
"inside the 139×31 minimum") describes 139×31 as the minimum and gets a short
inline note (2b).

### 2a. The spec 016 minimum-terminal ruling is superseded (currently lines 406–411)

Replace:

```markdown
- **The always-visible in-match panel grew the minimum terminal, 89×31 → 139×31.** The
  board keeps its exact layout and stays centered; the opponent-presence panel is drawn in
  the **right margin**, and the equal **left margin is reserved empty** for a future
  player-status panel (the layout is intentionally asymmetric for now). `IN_MATCH_MIN_WIDTH`
  and the panel width both derive from the portrait size, so they move together; below the
  minimum the existing too-small machinery errors with the required size, unchanged.
```

with:

```markdown
- **The always-visible in-match panel grew the minimum terminal, 89×31 → 139×31.** The
  board keeps its exact layout and stays centered; the opponent-presence panel is drawn in
  the **right margin**, and the equal **left margin is reserved empty** for a future
  player-status panel (the layout is intentionally asymmetric for now). `IN_MATCH_MIN_WIDTH`
  and the panel width both derive from the portrait size, so they move together; below the
  minimum the existing too-small machinery errors with the required size, unchanged.
  **Superseded by spec 026**: the minimum is **89×31 again** and 139 is the *threshold*
  above which the panel draws; `IN_MATCH_MIN_WIDTH` is now `WIDE_LAYOUT_MIN_WIDTH` (same
  value, same derivation from the portrait size). Below 139 the same board draws without
  the panel and a staked match's stake moves onto the status band. The right-margin
  placement, the reserved left margin and the asymmetry are unchanged at 139 and above.
  See *Compact layout below 139 columns (spec 026)* below.
```

### 2b. Inline note on spec 021's stake-on-screen bullet (currently lines 727–730)

Replace:

```markdown
- **The stake is shown in-match; no "runs ended broke" counter.** The wager's
  tension belongs on screen (it rides in the existing presence panel, inside the
  139×31 minimum); a bust counter is a cheap additive field if the balance pass
  wants it.
```

with:

```markdown
- **The stake is shown in-match; no "runs ended broke" counter.** The wager's
  tension belongs on screen (it rides in the existing presence panel, inside the
  139×31 minimum — since **spec 026** 139 is the wide layout's threshold, and
  below it the stake rides on the board's status band instead, so the ruling
  holds at every width); a bust counter is a cheap additive field if the balance pass
  wants it.
```

### 2c. Append a new section at the end of the file

Append at the **end of the file** — `DECISIONS.md`'s last two lines are
currently the tail of the "## Chore: warn on the wager prompt when a loss would
end the run (2026-09-17)" section:

```
`tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are untouched, and
`PROFILE_VERSION` / `SAVE_VERSION` stay 1.
```

(The pair is unique in the file — `grep -c` on each line reads 1 — and it is
also the end of the file, which is the anchor.) Append after them:

```markdown
## Compact layout below 139 columns (spec 026)

Since spec 016 the game needed a terminal 139 columns wide, because the
opponent-presence panel sits beside the fixed 89-column board with an equal,
empty margin on the other side — large for a general audience, when the game
had played at 89 before. This spec brings the minimum back to **89×31** and
makes 139 a **threshold**: at 139 and wider nothing changes; narrower, the
match board draws without the panel and the stake at risk moves onto the board.
A presentation-only spec: `layout.rs`, `config.rs`, `board.rs`, `portrait.rs`,
`overlay.rs`, `records.rs`, `app.rs`, fit tests and the README. Ruled by the
person on 2026-09-17.

- **Q1 B — the compact board drops the portrait and banter but keeps the
  stake.** Money at risk stays on screen (the wager-warning chore of
  2026-09-17 shows the same priority); banter without a face reads oddly in a
  status band spec 017 deliberately kept mechanical. The header's `Rounds won`
  line already carries what the pips showed, so the pips go too.
- **Q2 A — chosen by width alone, live, no setting.** The resize path already
  rebuilds the board view from the size; a setting would be a second way to
  reach a state width already reaches.
- **Q3 B — the select preview and the map rail keep their portraits at every
  width.** The person wants the portraits to stay part of the game even where
  the match window can't hold one; both surfaces physically fit at 89. (The
  recommended one-rule-everywhere option was declined.)
- **Q4 A with C's rule — the stake goes on the status band's upper row,
  right-aligned; the requirement is visibility for the whole staked match
  without growing the board block.** The planner could move it if the
  walkthrough found a collision; it found none.
- **Q5 A — the threshold stays 139.** No third layout between 114 and 138;
  the wide layout is exactly as spec 016 shipped it.
- **Q6 A — the settled stake shows at game over, on both layouts.** Found by
  the Phase 1 walkthrough (finding F1): at the game-over popup the compact
  band's `Stake ◈ N` was gone, because the match settles on the tick that
  draws the game-over frame and `stake_at_risk()` is already cleared — and the
  wide panel had been blank there since spec 021 for the same reason. Ruled
  the same day: `App::stake_to_show()` returns `stake_at_risk()` or, at
  `GamePhase::GameOver`, the settled amount from the banner; `src/app.rs`
  only, pinned by an App-level test at 89×31 and 139×31. The wide layout's
  one deliberate change.

Design tensions resolved during planning:

- **The panel is an `Option<Rect>`, chosen where the geometry is built.**
  `BoardLayout.opponent_panel` is `Some` iff `cols >= WIDE_LAYOUT_MIN_WIDTH`,
  decided in `BoardLayout::new` — the one place the board geometry is built
  (startup via `App::new`, resize via `App::resize`) — so there is no second
  flag to keep in sync and no width check in `board.rs` beyond `if let
  Some(panel)`. Rejected: a `wide: bool` beside a still-computed rect (two
  fields saying one thing, and the rect would be off-frame at 89), and a
  `Config::is_wide()` helper (a second place to evaluate the rule).
  `main.rs` and `App::resize` are untouched: `Config::fits` reads
  `min_size()` and `resize` already rebuilds `BoardView::new(config)`, so the
  switch across 139 falls out of the existing resize path.
- **`IN_MATCH_MIN_WIDTH` is renamed, and one test helper carries both
  widths.** The name would be false after this spec (the in-match minimum is
  89). It is `WIDE_LAYOUT_MIN_WIDTH` — same value, same derivation, new doc —
  and `Config::min_size()` returns `(BOARD_WIDTH, BOARD_BLOCK_HEIGHT)`. The
  tests that used the old name for "the minimum" switched to
  `Config::fit_sizes()`, a `#[cfg(test)]` helper returning `[89×31, 139×31]`,
  so every fit test loops both without spelling numbers. `wager.rs` is on the
  spec's no-change list: its fit test reads `min_size()` and so measures 89
  without an edit, and the 139 case follows from centering (a content-sized
  box that fits 89 fits any wider frame) — the spec's AC 3 records exactly
  this.
- **The play log's width floor became the wide box's width.** `SCROLL_MIN_W`
  40 → 52 (`139 · 38 / 100`), so the compact play log is the same box as the
  wide one instead of a 40-column one that clips more transcript lines; 52 plus
  margins fits 89. The change is confined to widths below 139 — widths the game
  has never run at — and at 139 and up the clamp never binds, so the wide box
  is untouched. The sizing moved into a pure `scroll_box(config) -> Rect` so
  the equality is a test, not a walkthrough observation. Pre-existing and
  **not** fixed here: round headers with a long opponent name (`Round 1: The
  Magistrate wins — You 20 / The Magistrate 19 (stand)`, 66 columns) already
  clip at 139's 48-column interior; the spec asks only that the compact log
  show what the wide one shows. Not observed in the walkthroughs.
- **The stake line is the panel's string.** `portrait::stake_line(stake)`
  (`Stake ◈ N`) is used by the panel and by the compact arm — one string, so
  the "panel's own form" claim can't drift. Banter is accepted and ignored in
  compact, keeping `BoardView::draw`'s signature unchanged.
- **The density rule was checked — no conflict.** The stake shares the over-20
  alert's row (the alert is 27 characters, `Align::Left`; a six-digit stake is
  14, `Align::Right`, on the 81-column band — ≥ 40 blank cells between them,
  pinned), adds no row, and the board block keeps its fixed 31-row height. The
  acted-on element on the board (the cursored hand card) is unchanged.

Attested by driver walkthroughs at both sizes. At 89×31: every screen (menu,
How to Play, opponent select with the preview beside the list, map clear of the
rail, Outfitter, deck builder with the hint whole and centered, records, play
log as a 52-column box, wager prompt) on frame with nothing clipped; a staked
match showed `Stake ◈ 30` from the first frame through the over-20 alert, a
round popup and the game-over popup; Quick Play showed no stake line; the
too-small screen quoted `Need at least 89 x 31` at 60×20. At 139×31: spec
016's panel with portrait, banter, pips and stake, no band stake, and the stake
on the panel at game over. Resizing 139 → 138 → 139 mid-match kept the match,
the moved cursor and the open `?` overlay and toggled the panel.

No engine, AI, economy, wager, save-format or balance change: `main.rs`,
`card.rs`, `game.rs`, `player.rs`, `save.rs`, `profile.rs`, `economy.rs`,
`wager.rs`, `campaign.rs`, `campaign_map.rs`, `tests/balance.rs`, `Cargo.toml`
and `Cargo.lock` are untouched; `portrait.rs` gained only `stake_line`, its
call and one doc line; `PROFILE_VERSION` and `SAVE_VERSION` both stay 1; no
new crate. Monochrome by construction: the stake line is `Strong`, as on the
panel, and no new emphasis level.
```

---

## 3. Not drafted here (deliberately)

- **`Readme.md` is on the branch** — its terminal-size paragraph now
  describes the 89×31 minimum and the two layouts (T004). It rides into `main`
  with the merge and needs no close-out edit.
- **`docs/economy.md`, `docs/balance.md` and `docs/opponents.md` are
  untouched** — none of them describes the terminal size or the board layout,
  and no number moved.
- **No `CLAUDE.md` amendment.** The constitution's *Platform* section says the
  game has "no specific minimum terminal size beyond what
  `Config::from_terminal` already enforces" — still true, the figure it
  enforces just moved. The rendering pattern (`Frame`, render thread), the
  `Screen` shape, the draw-never-mutates boundary and the verification command
  are unchanged; the *acted-on element stands apart* rule was checked and found
  not to apply (recorded above).
- **The spec 025 Shipped entry's "fit 139×31" (ROADMAP line 420)** is left as
  history: it is a fit claim, not a minimum, and the Outfitter's fit test now
  loops 89 as well.
- **Tier-log observations stay in `specs/026-compact-layout/tasks.md`** (the
  Phase 1 and Phase 2 review notes, the walkthrough rows, finding F1's route
  to T002a): process evidence for the model-policy experiments, not project
  decisions.
