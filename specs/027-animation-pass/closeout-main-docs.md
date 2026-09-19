# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 027)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T005, ready to paste on `main` after the merge
(the same close-out shape specs 020–026 used). Nothing here is applied by the
spec branch.

Spec 027 amends no rule `CLAUDE.md` states — see §3. Nothing to apply there.

Line numbers below are **`main`'s** at the time of drafting (2026-09-19,
`main` at `395a646`, *Record spec 026 (compact layout below 139 columns) in
the roadmap and decisions log*). `main` already carries spec 026's close-out,
which is not touched here. Each edit quotes its anchor text verbatim, which is
what to match on; every anchor below was re-checked with `grep -c` against
`git show main:<file>` at `395a646` and is unique there.

Applying: never chain a file edit, a branch switch and a commit in one shell
command (spec 025's miss) — switch to `main`, edit, verify the anchors again,
then commit.

---

## 1. `ROADMAP.md` — three edits, the rest judged and left

`grep -n -i "animation" ROADMAP.md` on `main` returns two hits that are
about this spec: **574–576** (the spec 016 backlog bullet's "Light animation …
stayed deferred to a later spec") and **722–728** (the "Considered animation
pass" backlog bullet this spec ships). Read and judged: both get edits (1b,
1c). The spec 016 *Shipped* entry (lines 203–224) does not mention animation
and is left alone; spec 017's "no layout/engine/save/AI change" and spec 026's
entry do not either.

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Compact layout below 139 columns (spec 026)**
entry, which ends with these two lines (currently `ROADMAP.md` lines 463–464,
just before the blank line and `## Backlog`):

```markdown
  threshold and the too-small screen) attested. `Readme.md`'s terminal-size
  paragraph re-synced.
```

Insert after them:

```markdown
- **Animation pass** (spec 027) — the match board now has **sparse, one-shot
  transitions** that guide the eye to what just changed, in the vocabulary
  spec 002's Motion rule set: emphasis over time on an element already at
  its final position, no sweeps, no particles, no ambient motion. A **card
  arriving** on either side — a dealt card in the dealer row, a played card
  in the played row — lands with the **heavy border** and Strong for one
  arrival beat (600 ms) and settles to its resting look (single or double
  border, Normal); a played card's **emptied hand slot shows a source ghost**
  (a plain single-line outline, empty face, no number key) for the same beat,
  the "it came from there" half of a source-and-destination pair with nothing
  flying between them. A **Score that changed** draws Strong for the beat and
  rests Normal (it was bold at all times before, so the beat could not show —
  Q7; `Rounds won` and the slot counter never transition). The **round and
  game-over popups wait one popup beat** (800 ms) after the round resolves,
  so the deciding card is seen uncovered with its own highlight; `n`, `g` and
  `x` work from the first frame and the popup then simply never appears. The
  opponent's thinking pause carries a **stepping indicator** (`Opponent's
  Turn .` / `..` / `...`, every 300 ms) on the status line, Muted, reverting
  the moment the opponent acts. Both sides, both layouts (89 and 139); the
  first frame of a new match, a rematch and a resumed save is drawn settled;
  the selection pulse keeps breathing throughout (`design/brief.md`'s Motion
  section gained one sentence: a one-shot transition may run alongside the
  continuous pulse). All of it is drawing state: a new `motion.rs`
  (`BoardMotion`, a pure per-element countdown observed once per tick from
  the frame-to-frame diff, the same snapshot pattern banter and audio use),
  read by `board.rs`; the engine, `GamePhase`, the timing constants and every
  key are untouched, and no key is ever delayed. Settings gained a third row,
  **Animations On/Off** (saved with the volumes; a file without the key reads
  On), which turns the whole layer off — the Off board is the pre-spec board
  frame for frame apart from the Score resting Normal. Two revisions at the
  phase pauses: the first bold-only arrivals did not register on a thin
  box-drawn card (Revision 1: heavy landings, the source ghost, and a
  face-down `?` flip for dealt cards); the person then judged the flip as not
  making sense and it was withdrawn (Revision 2) — a card's face never
  changes after it is drawn. Portraits stay static (spec 016's "light
  animation" deferral is closed, not reopened). No engine, AI, economy,
  wager, save-format or balance change: `game.rs`, `main.rs`, `render.rs`,
  `layout.rs`, `portrait.rs`, `card.rs`, `player.rs`, `save.rs`,
  `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`, `campaign_map.rs`,
  `opponent.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are
  untouched, `PROFILE_VERSION` / `SAVE_VERSION` stay 1, no new crate, no
  color path. Driver walkthroughs after each phase attested the
  arrivals, the ghost, the popup beat, the dots and the settled first
  frames at 89×31 and 139×31, and the Off state at 89×31. `Readme.md`'s settings mention names the
  row.
```

### 1b. Close the spec 016 backlog bullet's animation deferral (currently lines 574–576)

The shipped-marked **Opponent portraits (monochrome)** backlog bullet ends by
deferring light portrait animation. Using the repo's inline convention,
replace:

```markdown
  by a more capable tool and validated/integrated by Claude Code. **Light animation**
  (swapping frames on the pulse/tick) stayed **deferred** to a later spec; **banter** (above)
  is the next slice and now has a face to attach to.
```

with:

```markdown
  by a more capable tool and validated/integrated by Claude Code. **Light animation**
  (swapping frames on the pulse/tick) stayed **deferred** to a later spec — **closed by
  spec 027 (Q5 A): portraits stay static**, as the brief's portrait amendment says, and the
  animation pass left the presence panel untouched; **banter** (above)
  is the next slice and now has a face to attach to.
```

### 1c. Mark the backlog entry this spec shipped (currently lines 722–728)

Under "### Onboarding, endgame & release readiness", replace the whole
**Considered animation pass** bullet:

```markdown
- **Considered animation pass** — deliberate, sparse animations that
  guide the eye during play: a dealt card arriving, a flip resolving,
  a total changing, round transitions. Builds on spec 002's selection
  pulse and its Motion rule in `design/brief.md` (motion is emphasis;
  one vocabulary — emphasis transitions over time — never ambient or
  decorative movement). Designed against the finished UI overhaul, not
  in advance of it.
```

with:

```markdown
- **Considered animation pass** — ✅ **Shipped (spec 027** — see Shipped
  above and `DECISIONS.md`). Deliberate, sparse animations that guide the
  eye during play: a card arriving on either side lands heavy for a beat
  (with a source ghost in the hand slot a played card left), a changed
  total draws Strong for a beat, the outcome popups wait a beat so the
  deciding card is seen, and the opponent's pause carries a stepping
  indicator. Built on spec 002's selection pulse and its Motion rule in
  `design/brief.md` (motion is emphasis; one vocabulary — emphasis
  transitions over time — never ambient or decorative movement), with the
  brief amended by one sentence so a one-shot transition may run alongside
  the pulse. "A flip resolving" was tried at Revision 1 and withdrawn at
  Revision 2: a card's face never changes after it is drawn. An Animations
  row in Settings turns the layer off. Designed against the finished UI
  overhaul, as intended.
```

**Leave alone** the spec 016 *Shipped* entry (203–224: it never claimed
animation), spec 017's and spec 026's entries, and every other backlog
bullet.

---

## 2. `DECISIONS.md` — one edit

`grep -n -i "animation" DECISIONS.md` on `main` returns one hit, **line 385**,
inside spec 016's *bounded pictorial exception* bullet: portraits are
"monochrome, static, opponent-only … no color, no animation". Read and
judged: it is exactly what Q5 A upholds and stays as written — the new
section below says so. No spec 016 ruling deferred portrait animation in
`DECISIONS.md` (that deferral lives only in the roadmap, 1b above).

### 2a. Append a new section at the end of the file

Append at the **end of the file** — `DECISIONS.md`'s last two lines are
currently the tail of the "## Compact layout below 139 columns (spec 026)"
section:

```
new crate. Monochrome by construction: the stake line is `Strong`, as on the
panel, and no new emphasis level.
```

(The last line is unique in the file — `grep -c` reads 1 — and it is also
the end of the file, which is the anchor.) Append after them:

```markdown

## Animation pass (spec 027)

Until this spec exactly one thing moved on a still screen: the selection
pulse. A dealt card, the opponent's whole move, a changed total and the
outcome popup all landed on the same frame as the state change behind them,
so a fast round read as a jump cut. This spec adds sparse, one-shot
transitions on the match board — emphasis over a short, fixed time on an
element already at its final position — and an Animations row in Settings
that turns them off. Drawing only: a new `motion.rs` and changes to
`board.rs`, `app.rs`, `settings.rs`, `lib.rs`, one doc line in `frame.rs`,
one test line in `audio.rs`, the README and the brief. Ruled by the person on
2026-09-18, with two revisions at the phase pauses (Revision 1 on 2026-09-18,
Revision 2 on 2026-09-19).

- **Q1 a–e in, f out — a dealer card arriving, a played card arriving, a
  total changing, the popup beat and the thinking indicator; no stake
  flash.** Exactly the roadmap's list plus the thinking indicator; the
  presence panel's stake line, pips and banter are untouched.
- **Q2 A — arrival is emphasis only.** A face-down beat hides information
  and costs more for a subtlety. (Superseded for dealer cards by Q8 at
  Revision 1, then restored for every card by Q11 at Revision 2 — see
  below.)
- **Q3 A — the selection pulse keeps breathing during a transition**, and
  `design/brief.md`'s Motion section is amended by one sentence: the
  one-thing-moves rule counts *continuous* motion, and a one-shot emphasis
  transition may run alongside the pulse because it ends on its own within a
  beat and never breathes. Holding the pulse (Q3 B) was declined.
- **Q4 A — an Animations On/Off row in Settings.** Reduced motion is a real
  need and the overlay was written to grow. Saved with the volumes; a
  settings file **without the key reads On**; a file with it Off starts Off;
  the toggle takes effect on the next frame, mid-match included. Off means
  the board draws settled from the first frame — no heavy border, no ghost,
  no Strong, popup on the resolving frame, static thinking line — with one
  standing exception (Q7). The pulse and the map's starfield are not
  governed by it.
- **Q5 A — portraits stay static.** Spec 016's "light portrait animation"
  deferral (swapping frames on the pulse) is closed at this merge rather
  than reopened; the brief's portrait amendment ("no color, no animation",
  spec 016 above) stands unchanged and the presence panel is untouched.
- **Q6 A — the match board only.** Menus, the Outfitter, the deck builder,
  the map, the wager prompt, the shop and the records screens do not change.
- **Q7 — the Score rests at Normal weight** (a session ruling at planning
  under the person's delegation — plan *Open questions* 1 — flagged in the
  spec-conformance summary). `Score: N` had been drawn bold at all times, so
  a bold-for-a-beat transition on it would have been invisible, and the only
  stronger level is the rationed inverse. The options were to rest the Score
  Normal so the beat shows, or keep it bold and drop the total-change
  transition; the first was taken because ruling c is the person's. The
  consequence is the one deviation from "the Off board is the pre-spec board
  frame for frame": with Animations Off the Score is no longer bold.
- **The standing constraint**, stated by the person with the rulings:
  transitions must be noticeable enough to add to the game and quick enough
  that they never get in the way of player actions. No key is ever delayed
  or deferred; the opponent's pause is still `OPPONENT_THINKING_TIME_MS`;
  the phase machine, saves, banter and audio fire exactly when they did.

**Revision 1** (the person, 2026-09-18, at the Phase 1 pause). With Phase 1
built, the popup beat and the thinking dots read well but the bold-only hit
and card-play arrivals did not register at all: the driver confirmed the bold
attribute reached the terminal on the right frames, and the cause was that
bold on a thin box-drawn card is barely distinguishable from normal, while
the hand cursor's heavy breathing border already owned the strongest look on
the board. The remedy was shape, not weight.

- **Q8 — a dealer card lands heavy** (and, until Revision 2's Q11 withdrew
  it, face down for a flip beat). The dealt card draws with the heavy border
  and Strong for the arrival beat, then settles to today's single border,
  Normal.
- **Q9 — a played card lands heavy, and its hand slot shows a source
  ghost.** No card flies from the hand to the board: at the loop's 50 ms
  frame and whole-cell positions a flight would read as a stutter. The
  ghost outline in the emptied slot (single border, empty face, Normal, no
  number key) plus the heavy landing gives the same "it came from there" cue
  without motion — the person's choice from the session's options. Both
  sides: the opponent's hidden `?` slot empties into the same ghost.
- **Q10 — the Score transition stays as built** (Strong for the arrival
  beat, resting Normal per Q7). Not raised by the person; noted so the
  revision's scope is explicit. If it proves as invisible as the card bold,
  it is a follow-up.
- Not in the revision: the selection pulse's own vocabulary (spec 002 — a
  heavy/thin border alternation was floated as more visible than
  bold/normal; the person has not ruled), the thinking indicator and the
  popup beat, which the person judged good.

**Revision 2** (the person, 2026-09-19, at the Phase 1b pause). Played with
Revision 1 built, the person judged the heavy landings and the source ghost
good and the dealer card's `?` flip as not making sense: "just remove the
initial `?` and call it good."

- **Q11 — the flip is withdrawn.** A dealt card lands heavy with its value
  visible from its first frame; the flip beat, its constant and its bounds
  are gone (`grep FLIP src/` is empty). A card's content never changes after
  it is drawn, on either side; Q2 A stands for every card. Everything else in
  Revision 1 (Q9, Q10) stands.

**Process note.** Both revisions were written into `spec.md` in the
implementation session at the person's ruling ("revise the spec now so that
we finish it here") — a stated deviation from the constitution's rule that
spec conversations happen in a spec session of their own. Each revision had
its own planner amendment, sign-off at the top tier, a phase (1b) with its
own review, and a walkthrough.

Design tensions resolved during planning:

- **One struct, observed in `tick`, read by `draw`; `None` is the settled
  draw.** `BoardMotion` (`src/motion.rs`, pure logic and tests, no rendering
  import, like `banter.rs`) is a field on `App`. `App::tick` calls
  `observe(&game_state, dt)` while the screen is `InGame` and resets it to
  `default()` otherwise — that one `match` is the spec's "discarded when the
  board leaves the screen" and its "first frame drawn settled" (the first
  observation after entering a match seeds silently and starts nothing).
  `App::draw` passes `settings.animations.then_some(&motion)` to
  `BoardView::draw`, so `None` — no motion at all — is the settled draw and
  the Off state is the same code path as "nothing is arriving". Rejected: a
  per-element `Instant` map (untestable without sleeping), a `GamePhase`
  extension (the engine must not know), and an `Animations` flag inside
  `BoardMotion` (the board would have two ways to be settled). Clocks count
  down in the tick's `dt`, so every test is a sequence of `observe` calls
  with chosen durations, no sleeping.
- **A row that shrinks is a clear, never a change.** `setup_next_round` and
  `new_game` empty both rows and drop the total to 0 — a "change" by value,
  but the spec wants a rematch's first frame settled and the eye guided to
  what *arrived*. Rule in `observe`: if a side's dealer or played count fell
  since the last observation, discard that side's arrivals (the source ghost
  included) and start nothing for it; otherwise start an arrival per new
  index and a Score arrival if the total differs. A second change to the
  same Score restarts its beat (one figure, one clock); a card index cannot
  arrive twice without a clear. The `g` rematch's settled board falls out of
  this rule rather than the seed.
- **Settings: three rows, one `adjust`, no disk in tests.**
  `SettingsAction::Louder`/`Quieter` became **`Right`/`Left`** — on a
  volume row louder/quieter, on the Animations row a toggle; the old names
  were about to be wrong — and the value change moved out of
  `App::handle_settings_input` into **`Settings::adjust(row, right)`**, which
  owns the volume step (`VOLUME_STEP`, clamped to 0..=1) and the toggle, so
  both are unit-tested without the `save()` that writes the real config
  file; `App` still calls `set_settings`, `save` and the cue as before. The
  rows are a `const ROWS: [SettingRow; 3]`; the hint reads `←/→ change` (was
  `←/→ volume`). The Animations row is **padded to the volume rows' width**
  (T004a, the person at the Phase 2 pause, after the reviewer noted the
  shorter centred row shifted its marker and label ~6 columns right): the
  row's marker and label now sit in the volume rows' columns, with the value
  one space after the label.
- **Revision 1: the arrivals change shape on the clocks Phase 1 already
  runs.** The heavy landing costs no new clock: a card draws
  `BorderWeight::Heavy` + `Strong` while its arrival is counting down and its
  resting border after. **The source ghost is one more `Elem`**,
  `Hand(side, slot)`, started when a hand slot went `Some → None` since the
  last observation, on the arrival beat, and dropped by the same shrink rule
  as the side's cards; the board draws it by reusing `CardView` with an
  empty face rather than a bespoke box, so its rect and interior blanking are
  the card's. **The heavy landing is the recorded exception to the brief's
  "distinct weights, distinct meanings" rule** (spec 003): Heavy was
  reserved for cursor selection, and for one arrival beat it now also marks
  a card landing. `frame.rs`'s `BorderWeight` doc names the exception (the
  file's one comment-only touch); `card.rs`'s `// Heavy marks cursor
  selection (T007)` comment is **knowingly left stale**, because `card.rs`
  is on the spec's no-change list (AC 12) — noted here so the next touch of
  that file fixes it. The brief's Motion section itself carries only the
  Q3 A sentence. **The flip was built as a read of the arrival's countdown**
  — `?` while the remaining time was still inside the flip window, no second
  clock, no `face_down` flag — and **withdrawn** by Q11 after the person saw
  it: the constant, the guard and the `?` branch are gone.
- **The beats live in `lib.rs`, the bounds in a test.** As shipped, with no
  tuning at the pauses: `ARRIVAL_BEAT_MS = 600`, `POPUP_BEAT_MS = 800`,
  `THINKING_STEP_MS = 300`. A motion test pins the spec's bounds
  (`SELECTION_PULSE_MS ≤ arrival ≤ 1000`, `arrival ≤ popup ≤ 1000`,
  `2 · step ≤ OPPONENT_THINKING_TIME_MS`), so a retune outside them fails
  the build.

Attested by driver walkthroughs at 89×31 and 139×31 after Phases 1 and 1b and
at 89×31 after Phase 2, with the
person's own play at the Phase 1, 1b and 2 pauses. Phase 1 (bold-only): a
dealt card bold on its frame and settled by ~1 s, the Score bold only when it
changed (a dealt 0 left it plain), `Rounds won` never bold, the cursor still
breathing; the dots stepping through the pause and the plain line once the
opponent acted; a round resolved with no popup at two 0.4 s samples and the
popup by 0.8 s; `n` on the resolving frame started the next round with no
popup ever drawn; the game-over popup absent at 0.1 s and present at 1.1 s;
Continue on a save left at the popup drew the popup on the first frame with
nothing bold, and Continue mid-match drew settled; at 89 the dots sat on the
band's lower row with the alert/stake row above. Phase 1b (heavy landings): a
hit landed heavy, still heavy at 0.35 s, settled to the thin border by
0.75 s; a play landed heavy in the grid while the emptied hand slot kept a
plain outline and lost its number key, then settled to the double border
with the slot blank; the cursor moved to the next card and kept breathing;
the opponent's card landed the same way; the presence panel unchanged at 139.
Phase 2 (the setting): Settings showed Music, Sound FX, Animations On; `→`
flipped it Off and the file on disk gained `"animations": false`; a hit with
it Off drew the thin card with its value, a plain `Opponent's Turn`, a
double-bordered play with the slot blank and the popup on the resolving
frame; `→` again read On and the file said `true`; a file with the key
deleted read On.

No engine, AI, economy, wager, save-format or balance change: `game.rs`,
`main.rs`, `render.rs`, `layout.rs`, `portrait.rs`, `card.rs`, `player.rs`,
`save.rs`, `profile.rs`, `economy.rs`, `wager.rs`, `campaign.rs`,
`campaign_map.rs`, `opponent.rs`, `tests/balance.rs`, `Cargo.toml` and
`Cargo.lock` are untouched; `frame.rs` changed one doc comment; `audio.rs`
changed one test's struct literals for the new field; `PROFILE_VERSION` and
`SAVE_VERSION` both stay 1; no new crate; the build has no warnings. No color
path: the only attributes emitted are the four existing emphasis levels and
the existing border weights.
```

---

## 3. Not drafted here (deliberately)

- **`Readme.md` and `design/brief.md` are on the branch** — the README's
  settings mention names the Animations row and the brief's Motion section
  carries the Q3 A sentence (T004). Both ride into `main` with the merge and
  need no close-out edit.
- **`docs/economy.md`, `docs/balance.md` and `docs/opponents.md` are
  untouched** — none of them describes the board's drawing or the settings
  overlay, and no number moved.
- **No `CLAUDE.md` amendment.** The rendering pattern (`Frame`, render
  thread), the `Screen` shape, the draw-never-mutates boundary (the motion
  is observed in `tick`, and `draw` only reads it), the `GamePhase` machine
  and the verification command are unchanged. The *acted-on element stands
  apart* rule was checked on the Settings overlay and read as not applying
  to a cursored list, matching the start menu (plan tension 8). The spec's
  two in-session revisions are a stated, logged deviation from the
  spec-session rule, not an amendment to it.
- **`DECISIONS.md` line 385** (spec 016's "no color, no animation" for
  portraits) is left as written: it is what Q5 A upholds.
- **`src/card.rs:161`'s `// Heavy marks cursor selection (T007)`** is stale
  after this spec (Heavy also marks an arrival for one beat) and is left,
  because `card.rs` is on the spec's no-change list (AC 12); recorded in the
  DECISIONS entry above so it does not die with the tier log. A wording
  chore can take it with the two stale comments spec 026's close-out named
  (`src/wager.rs:497`, `src/config.rs`'s `fit_sizes` doc).
- **Reviewer notes left for the sweep**, all non-blocking, stay in
  `specs/027-animation-pass/tasks.md`'s tier log: the dot clock not
  restarting on a back-to-back pause (AC 5 holds either way), no board test
  distinguishing the sides' arrivals, the key→tick→draw ordering in
  `main.rs` pinned only by the walkthrough, no board assertion that a
  sibling dealer card stays single/Normal while another lands,
  `ROWS`/`SettingRow` as two lists. (`row_text`'s doc was corrected by
  T004a; the redundant `FLIP_BEAT_MS` test import went with T003d.)
  Process evidence for the model-policy experiments, not project decisions.

---

## 4. Mechanical checks (T005, run on the branch at `d647c00`, 2026-09-19)

`cargo test -q 2>&1 | tail -n 25`, three consecutive runs — identical tails:

```
=== run 1 ===

running 443 tests
....................................................................................... 87/443
....................................................................................... 174/443
....................................................................................... 261/443
....................................................................................... 348/443
....................................................................................... 435/443
........
test result: ok. 443 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
i......
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.98s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

=== run 2 ===

running 443 tests
....................................................................................... 87/443
....................................................................................... 174/443
....................................................................................... 261/443
....................................................................................... 348/443
....................................................................................... 435/443
........
test result: ok. 443 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
i......
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.97s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

=== run 3 ===

running 443 tests
....................................................................................... 87/443
....................................................................................... 174/443
....................................................................................... 261/443
....................................................................................... 348/443
....................................................................................... 435/443
........
test result: ok. 443 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
i......
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.98s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The forbidden files are untouched (`git diff main...HEAD --stat` lists none
of them; the explicit query is empty):

```
$ git diff main...HEAD --stat
 Readme.md                         |   3 +-
 design/brief.md                   |   6 +-
 specs/027-animation-pass/plan.md  | 782 ++++++++++++++++++++++++++++++++++++++
 specs/027-animation-pass/spec.md  | 362 ++++++++++++++++++
 specs/027-animation-pass/tasks.md | 500 ++++++++++++++++++++++++
 src/app.rs                        | 209 +++++++++-
 src/audio.rs                      |   9 +-
 src/board.rs                      | 308 +++++++++++++--
 src/frame.rs                      |   3 +-
 src/lib.rs                        |   9 +
 src/motion.rs                     | 500 ++++++++++++++++++++++++
 src/settings.rs                   | 216 +++++++++++--
 12 files changed, 2818 insertions(+), 89 deletions(-)

$ git diff main...HEAD --stat -- src/game.rs src/main.rs src/render.rs src/layout.rs src/portrait.rs src/card.rs src/player.rs src/save.rs src/profile.rs src/economy.rs src/wager.rs src/campaign.rs src/campaign_map.rs src/opponent.rs tests/balance.rs Cargo.toml Cargo.lock
(empty)
```

`src/frame.rs` is one doc comment — the sentence was rewrapped, so `--stat`
counts 3 lines (2 in, 1 out) for one comment edit, no code:

```
$ git diff main...HEAD -- src/frame.rs
@@ -72,7 +72,8 @@ pub enum Align {
 /// Border line weight. Single is the default chrome everywhere; Heavy
-/// is reserved for cursor selection; Double marks a played side card on
+/// is reserved for cursor selection (and, for one arrival beat, a card
+/// landing on the board — spec 027); Double marks a played side card on
 /// the board grid (dealer draws stay Single). Distinct weights, distinct
```

`src/audio.rs` touches only `mod tests` (the module opens at line 357; the
single hunk is at 377, inside it, and only adds `..Settings::default()` to
three struct literals in one test):

```
$ git diff main...HEAD -- src/audio.rs | grep -n "^@@"
5:@@ -377,14 +377,17 @@ mod tests {
$ grep -n "mod tests" src/audio.rs
357:mod tests {
```

The save and profile versions still read 1:

```
$ grep -n "VERSION" src/save.rs src/profile.rs
src/save.rs:31:const SAVE_VERSION: u32 = 1;
src/save.rs:92:        version: SAVE_VERSION,
src/save.rs:173:    (saved.version == SAVE_VERSION).then(|| from_saved(saved))
src/save.rs:375:        val["version"] = serde_json::json!(SAVE_VERSION + 1);
src/profile.rs:31:const PROFILE_VERSION: u32 = 1;
src/profile.rs:92:    PROFILE_VERSION
src/profile.rs:136:            version: PROFILE_VERSION,
src/profile.rs:207:        (profile.version == PROFILE_VERSION).then_some(profile)
src/profile.rs:506:            version: PROFILE_VERSION,
src/profile.rs:628:        assert_eq!(PROFILE_VERSION, 1, "the seed purse is no on-disk shape change");
src/profile.rs:656:        assert_eq!(PROFILE_VERSION, 1, "the seen-marks are no on-disk shape change");
src/profile.rs:820:        assert_eq!(PROFILE_VERSION, 1, "no version bump for the additive stats field");
src/profile.rs:955:        assert_eq!(PROFILE_VERSION, 1, "the counters and the record are no shape change");
src/profile.rs:1237:        assert_eq!(PROFILE_VERSION, 1, "no save-format change in this spec");
src/profile.rs:1268:        val["version"] = serde_json::json!(PROFILE_VERSION + 1);
```

No color path added (the branch's `src/` diff has no `Color`):

```
$ git diff main...HEAD -- src/ | grep -n "Color"
(empty, exit 1)
```

Warning count: **0 on this branch**. `main`'s count was not re-measured here
(no worktree); spec 026's tier log (T005 row, `specs/026-compact-layout/
tasks.md`) recorded "warning count 0 on main and branch" at spec 026's merge,
and every task's verification on this branch reported 0 warnings.

```
$ cargo build --all-targets 2>&1 | grep -c warning
0
```

The three beat constants, their bounds test, and the withdrawn flip:

```
$ grep -n "ARRIVAL_BEAT_MS\|POPUP_BEAT_MS\|THINKING_STEP_MS\|FLIP" src/lib.rs
74:// SELECTION_PULSE_MS <= ARRIVAL_BEAT_MS <= 1000, ARRIVAL_BEAT_MS <= POPUP_BEAT_MS
75:// <= 1000, THINKING_STEP_MS * 2 <= OPPONENT_THINKING_TIME_MS.
76:pub const ARRIVAL_BEAT_MS: u64 = 600; // a card arriving / a total changing draws Strong this long
77:pub const POPUP_BEAT_MS: u64 = 800; // the round/game popup waits this long after the round resolves
78:pub const THINKING_STEP_MS: u64 = 300; // the thinking indicator steps . / .. / ... at this cadence

$ cargo test -q beats_are_named_constants_within_bounds 2>&1 | grep -B1 -A2 "running 1 test"
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 442 filtered out; finished in 0.00s

$ grep -rn "FLIP\|face_down" src/
(empty, exit 1)
$ grep -n FLIP src/
(empty, exit 1)
```
