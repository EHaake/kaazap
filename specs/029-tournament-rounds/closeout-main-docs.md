# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 029)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T011, ready to paste on `main` after the merge
(the same close-out shape specs 020–028 used). Nothing here is applied by the
spec branch.

Spec 029 amends no rule `CLAUDE.md` states — see §3. Nothing to apply there.

Line numbers below are **`main`'s** at the time of drafting (2026-09-22,
`main` at `a5a854c`, *Roadmap: per-planet venue art as its own spec*; the
branch at `7d323eb`, merge base `6877585`). Each edit quotes its anchor text
verbatim, which is what to match on; every anchor below was re-checked with
`grep -c` against `git show main:<file>` at `a5a854c` and is unique there
(`ROADMAP.md` 937 lines, `DECISIONS.md` 1921).

**Refreshed after Phase 6 (ruling R9), 2026-09-22**, with the branch at
`7e5fe11` and `main` still at `a5a854c`, so every line number and anchor above
still holds — the anchors were re-checked with `grep -cF` against
`git show main:ROADMAP.md` for the refresh (each 1; 937 and 1921 lines). The
refresh changes only what R9 made false or added: §1a, §1b, §1c, the R9
material in §2a, §3's `design/brief.md` bullet, the triage of notes 49–56
(§4), §5 re-run in full, and §6's criterion 16 and new criterion 21.

Applying: never chain a file edit, a branch switch and a commit in one shell
command (spec 025's miss) — switch to `main`, edit, verify the anchors again,
then commit.

**One wording rule this close-out is held to — ruling R2.** The shop is the
**Card Shop** everywhere the player reads it. New prose below says Card Shop.
But the records of earlier specs keep the word they were written with: the
`ROADMAP.md` Shipped entries and backlog bullets for specs 012, 024 and 025, and
every earlier `DECISIONS.md` section, say "Outfitter" and **stay as written** —
rewriting them would falsify what those specs did (`spec.md`, *Amendment,
2026-09-21*, R2). Where the new text below refers to what an earlier spec
shipped, it may name the Outfitter; it never renames it after the fact.

**A second rule: one phrase for one idea (ruling R1).** The series length is
`Best of 3` / `Best of 5` — never "first to 2" — in everything below.

---

## 1. `ROADMAP.md` — six edits, the rest judged and left

`grep -n -i "series\|tournament\|venue\|best of" ROADMAP.md` on `main` returns
the **F · Tournament rounds** backlog bullet (638–654, the item this spec
ships), the **Per-planet venue art** bullet (873–900, written 2026-09-20 against
the *pre-amendment* design as a deferred item — which this spec now ships, by
ruling R9), and the
**Per-planet music** bullet (902–924, which names the venue once and stays
accurate).

A wider read was done and judged: `grep -n -iE "outfitter|cheapest_floor|banter|balance pass"`
returns the Outfitter in earlier Shipped entries and backlog bullets (the record
of specs 012, 024, 025 — **left**, per R2), the **Warn on the wager screen**
bullet (765–776), which names `economy::cheapest_floor` — **left** as the record
of what that chore shipped; its supersession-while-locked is recorded in
`DECISIONS.md` (§2a, *O1's side effect*), not by rewriting the chore's bullet —
and the shipped **Opponent banter** and **Difficulty & economy balance pass**
bullets, which stay shipped and gain neighbours (1d, 1e) rather than edits.

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Crash & data safety (spec 028)** entry, which
ends with these two lines (currently `ROADMAP.md` lines 553–554, just before the
blank line and `## Backlog`):

```markdown
  and all six data scenarios at 89×31 after Phase 4. `Readme.md`'s **Saved
  data** paragraph names the set-aside file.
```

Insert after them:

```markdown
- **Tournament rounds** (spec 029) — a campaign opponent is no longer beaten by
  one lucky match. Every opponent is a **series**: **Best of 3** (two match
  wins), and **Best of 5** (three) for the final opponent, The Sovereign on
  Zenith. A match is unchanged — first to 3 round wins — and each match of a
  series is staked, prompted and settled exactly as spec 021 has it. The minimum
  path through the campaign goes from 10 matches to 21. Between matches the
  player stands at the **venue**, a new `Screen` — the tournament hall on that
  planet — which names the planet and the opponent, shows the series score and
  length and the credit balance, and offers **Play**, the **Card Shop**, the
  collection and quit to the menu (`b` and `c` as on the map); the Card Shop
  and the collection return to it. The venue is drawn as horizontal bands:
  its text above and below a dominant **planet art region** that shows **each
  planet's own art** — sixteen monochrome drawings, a 48×20 narrow and a 92×20
  wide one per planet, authored outside the codebase from an in-repo brief and
  validated by a test — with the opponent's portrait in its own column three
  columns beside it, at **every** width. The box is sized exactly to the
  drawing: the narrow drawing below 139 columns (a 50×22 box), the wide one
  from 139 up (94×22), and at every other size the spare space is margin
  around the art and the portrait, never blank space inside the frame; a
  taller terminal centres the whole venue. Every text row is centred on the
  art rather than the terminal. `design/brief.md` gained a second bounded
  exception for the art, beside the portraits'. Starting a series **commits** the player to it: the map is
  unreachable, entering the campaign lands on the venue, and only the series
  resolving or a reset releases the lock. A lost series costs only the stakes
  already lost and resets to 0–0; a won one beats the opponent exactly then,
  and clearing, unlocking and completion counting are unchanged. While locked,
  "broke" is judged against the **locked opponent's ante**, and the Card Shop
  reserves the same floor — one `economy::reserve_floor` behind both. The map's
  planet detail names `Best of 3` / `Best of 5` before a launch; the board's
  status band carries `Series 1 – 0` during every series match at both widths;
  the map banner after the deciding match names the series result beside the
  settlement. Rematches on cleared planets stay single matches launched from
  the map. The shop is renamed **Card Shop** everywhere the player reads it.
  The first-campaign primer and How to Play state the rule and the commitment.
  The balance simulator was re-run with **no lever moved** and
  `docs/balance.md` gained *Series rates*: the curve sharpens in both
  directions (starter vs Greeb 70.3 % a match → 78.8 % a series, starter vs Rix
  28.1 → 19.3), and the starter's chance against the final opponent falls from
  23.2 % to **8.6 %**. No new records or statistics. The **per-planet art
  brief** the drawings were made from ships at
  `specs/029-tournament-rounds/planet-art-brief.md`. No engine, AI, save-format,
  economy-constant, balance-data or dependency change: `game.rs`, `card.rs`,
  `player.rs`, `save.rs`, `opponent.rs`, `Cargo.toml` and `Cargo.lock` are
  untouched, `PROFILE_VERSION` / `SAVE_VERSION` stay 1, and the two new
  persisted fields (`NodeRef::settled`, `CampaignRun::series`) are additive and
  `#[serde(default)]` — a pre-029 profile loads with no series running and every
  beaten opponent still beaten. The person amended the spec mid-implementation
  after walking the venue (R1–R8), then at the merge pause brought the art
  itself into the spec (R9), and attested at every paused phase; driver
  walkthroughs against a scratch `KAAZAP_DATA_DIR` covered the venue, the lock,
  the routing and the board at 89×31 and 139×31, and all eight planets' art at
  both sizes, plus 120×31 and 139×40.
  `Readme.md`'s campaign paragraph names the series, the venue and the lock.
```

### 1b. Mark the **F · Tournament rounds** backlog bullet shipped (currently lines 638–654)

Replace the whole bullet — from

```markdown
- **F · Tournament rounds: beat each opponent best-of-three, the final
```

through its last line

```markdown
  commitment in this entry.
```

with:

```markdown
- **F · Tournament rounds** — ✅ **Shipped (spec 029** — see Shipped above and
  `DECISIONS.md`). Every campaign opponent is a Best of 3 series, the final
  opponent Best of 5. The spec questions this entry listed were settled as:
  **each match is staked separately** (spec 021's escrow, settlement and loss
  condition untouched); a lost match inside a series costs its stake and a lost
  series costs **only** the stakes already lost; the map shows the series
  **length** before a launch and the venue and the board show the **score**;
  the records are unchanged; and a save mid-series resumes at the venue, which
  is where the Card Shop and the collection are visited between matches. The
  person's longer-range wish to lean further into the tournament theme stays a
  direction, not an item — per-planet venue art shipped with this spec (ruling
  R9), and per-planet music and series-aware banter (below) are its next
  concrete pieces.
```

### 1c. Mark the **Per-planet venue art** bullet shipped (currently lines 873–900)

The bullet was written on 2026-09-20 as a deferred item, against the
pre-amendment venue (a 30×15 region at 139 columns and wider, text only below).
Ruling R9 (2026-09-22) brought the art into this spec, so the item ships here.
It is **marked shipped in the roadmap's own form** (✅ **Shipped (spec NNN** —
…), like every other shipped backlog bullet and like 1b — not removed, which
would drop the record that it was once a deferred item, and not left, which
would leave `main` promising a spec that will not be written. Replace the whole
bullet — from

```markdown
- **Per-planet venue art** (deferred by spec 029, 2026-09-20). Spec 029 builds
```

through its last line

```markdown
  stays text-only there.
```

with:

```markdown
- **Per-planet venue art** — ✅ **Shipped (spec 029** — see Shipped above and
  `DECISIONS.md`). Deferred by spec 029 on 2026-09-20 and brought back into it
  by the person's ruling R9 at its merge pause, 2026-09-22. Each planet's venue
  shows its own art: **sixteen** monochrome drawings, a 48×20 narrow and a
  92×20 wide one per planet, in `assets/planets/`, loaded with `include_str!`
  like the portraits. The path spec 016 established was reused: a brief in the
  repo (`specs/029-tournament-rounds/planet-art-brief.md`), the art drawn by a
  more capable tool, and Claude Code validating it against the brief's
  checklist — now a test, whose canvas sizes are read from the venue's own
  layout rather than restated. The questions this entry listed were settled as:
  **one piece per planet**, two sizes each (the art region is a different width
  at the two layouts); **static**, as the portraits are; and the art draws at
  **every** width, below 139 columns too. Between the two sizes the **box fits
  the drawing** — the person's choice over letterboxing, stretching, extending
  or a third size — so spare space is margin around the art and the portrait, never blank
  space inside the frame. `design/brief.md` records the art as a second bounded
  exception, beside the portraits'.
```

### 1d. New bullet: **series-aware banter**, in *Immersion & personality*

Insert immediately after the **Opponent portraits (monochrome)** bullet, which
ends with this line (currently line 686, followed by a blank line and
`### Stakes, loss condition & difficulty balance`):

```markdown
  is the next slice and now has a face to attach to.
```

Insert after it:

```markdown
- **Series-aware banter** (deferred by spec 029, 2026-09-20). Since spec 029 a
  campaign opponent is a Best of 3 series (Best of 5 for the final opponent), so
  each opponent's **match-start line now fires two or three times in a row** —
  up to five against The Sovereign — and the match-end lines know nothing about
  whether that match took or lost the series. Accepted as a non-goal there. The
  work is lines, not machinery: `banter.rs`'s event classes would gain a
  series-aware variant of match start (first match, a match with the series
  level, a match that can decide it) and perhaps of match end, reading the
  series the board already shows (`board_series_line`'s inputs). As with spec
  017, the real cost is the writing — ten voices and the fallback.
```

### 1e. New bullet: **re-tune the curve if the series rates play badly**, in *Stakes, loss condition & difficulty balance*

Insert immediately after the **Difficulty & economy balance pass** bullet, which
ends with this line (currently line 720, followed by a blank line and
`### Onboarding, endgame & release readiness`):

```markdown
  moves relative to.
```

Insert after it:

```markdown
- **Re-tune the curve for series play, if it plays badly** (raised by spec 029,
  2026-09-22). Spec 029 turned every opponent into a Best of 3 series and the
  final opponent into a Best of 5, and **measured, it did not change**:
  `docs/balance.md`'s *Series rates* section converts the per-match rates into
  series rates, and the sharpening is real in both directions — the starter
  deck against Greeb goes from 70.3 % a match to 78.8 % a series, against Rix
  from 28.1 % to 19.3 %, and against The Sovereign's best of five from 23.2 %
  to **8.6 %**, while the full-pool deck's 51.3 % there becomes 52.4 %. That is
  the direction spec 022's gates were tuned to want, but the campaign is also
  longer — 21 matches at minimum rather than 10, roughly 25 expected — and
  nobody has played it end to end yet. If playtesting
  says the Mid Rim wall or the final is now too steep, the levers are the same
  three spec 022 tuned (opponents, prices, economy constants), plus two new
  ones: the series length itself (`campaign::wins_needed`, one expression) and
  best of five for the final opponent only. Re-measure with the simulator first
  (`KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table --
  --ignored --nocapture`, as `docs/balance.md` gives it; about 9 s in release). One note for whoever runs it: the
  largest per-match drift between the 2026-09-13 and 2026-09-22 runs, starter
  vs Rix 29.9 → 28.1, is about 2.8 standard errors — plausible as the largest
  of 50 cells with no fixed seed, but if a later run drifts the same way again,
  look at the engine before blaming chance.
```

### 1f. Amend the path-injection seam bullet's "Still open" follow-up (currently lines 786–802) — **only if the pre-merge sweep did not already fix it**

Spec 029 widened this follow-up (close-out note 21). If the sweep resolved it
on the branch, skip this edit. Otherwise, after the bullet's last line

```markdown
  directory** — it repairs the scenario you were about to test.
```

append, as a continuation of the same bullet:

```markdown
  **Spec 029 widened the surface again.** A campaign screen is now chosen from
  the profile (`open_campaign_home` reads the series lock), so any `App` test
  that asserts on a campaign screen passes or fails on whoever's save is on
  disk unless it first sets `app.profile = Profile::default()` — which
  `the_primer_swallows_map_keys` had to, after failing against the person's own
  mid-series profile. And `Profile::save()` has no `cfg(test)` guard, so "replace
  the profile with a default, then drive input" is one save away from
  overwriting the developer's real profile; the one test doing it today is safe
  only because its keys cannot dismiss the modal it sits under.
```

**Leave alone** every other roadmap bullet, including **Per-planet music**
(902–924, still accurate), **Warn on the wager screen** (765–776, the chore's
record — see §2a), **Show locked cards in the Outfitter** (809–818) and every
other Outfitter mention (R2), and **Archive the last run at reset** (758–764).

---

## 2. `DECISIONS.md` — one edit

`grep -n -iE "series|tournament|venue|best of" DECISIONS.md` on `main` returns
**no hits**: no earlier ruling is about series play. `grep -n "cheapest_floor"`
returns three: **line 777** (spec 021's `is_broke`) and **lines 1281, 1290**
(the 2026-09-17 wager-warning chore). All three are the record of what those
specs ruled and stay as written; spec 029's changes to them are stated in the
new section (the *One floor* and *O1's side effect* bullets), which is where a
reader following the function name will land next.

### 2a. Append a new section at the end of the file

Append at the **end of the file** — `DECISIONS.md`'s last three lines are
currently the tail of the "## Crash & data safety (spec 028)" section:

```
the gate's behaviour is visibly the same; no new crate; the build has no
warnings, the same count as `main`. Monochrome by construction: the notice
reuses `draw_notice`'s existing emphasis levels and adds none.
```

(The last line is unique in the file — `grep -c` reads 1 — and it is also the
end of the file, which is the anchor; the file ends with a newline.) Append
after it:

```markdown

## Tournament rounds (spec 029)

A single match win used to defeat a campaign opponent: ten opponents, ten
matches, and one lucky match could take a world. Spec 029 puts a **series**
between the match and the opponent, a **venue** between the map and the match,
and a **lock** that makes the series the only campaign match playable while it
runs. It adds one `Screen` (`Venue`), one module (`src/venue.rs`), one layout
struct (`VenueLayout`), two persisted fields (`CampaignRun::series`,
`NodeRef::settled`), both `#[serde(default)]` with no version bump, and — by
ruling R9 — sixteen drawings of the planets in `assets/planets/`, with a second
bounded exception for them in `design/brief.md`. No engine, AI, save-format,
economy-constant, balance-data or dependency change. Ruled by the person on
2026-09-20 (A1–I1, then J1–Q on the consequences of D2 and E2), amended three
times after they walked the venue (R1–R3 on 2026-09-21, R4–R6 the same day, R7
and R8 on 2026-09-22), and extended at the merge pause the same day (R9), which
brought the art itself into the spec.

### The rulings (the person, 2026-09-20)

- **A1 — the new unit is a "series."** "Round" is already the within-match
  unit, and "tournament round" would collide with it.
- **B1 — each match is staked separately**, exactly as spec 021 has it. Keeps
  escrow, settlement and the broke check intact, and makes the between-match
  shop visit worth something: a win pays before the next match starts.
- **C1 — losing a series costs only the stakes already lost.** The score resets
  to 0–0, the opponent stays un-beaten, nothing further is taken.
- **D2 — a venue screen between matches**, rather than returning to the map.
  The person's reason: it is a place, and an opportunity for art that raises
  the immersion of the campaign.
- **E2 — the player is locked into a series once it starts.** No abandon
  action; quitting to the menu leaves it in progress.
- **F1 — rematches stay single matches** (the grinding path is meant to be
  low-friction).
- **G1 — best of five for the final opponent only**, not for the Core
  generally.
- **H1 — no new records or statistics.** Match counters count matches;
  campaign completion counts once. The lifetime first-clear record stops being
  comparable with anything set before this spec (the campaign takes roughly
  2.4× as many matches) — accepted, not reset.
- **I1 — the series is visible on the map and in the match**, which with E2 and
  P1 resolves as: the map shows the series **length** before a launch (the
  player is never on the map while a series is live), and the venue and the
  match board show the **score**. Confirmed by the person.
- **J1 — the map launches to the venue at 0–0**, staking nothing, and every
  match of the series starts from the venue. **K1 — the venue offers play, the
  shop, the collection and quit**, the shop and collection returning to the
  venue. **L1 — rematches keep today's flow**: map → wager → match → map.
- **M1 — the art region is reserved now and filled with a plain placeholder;
  authoring it is its own spec.** Amended **twice**. The same day: the
  opponent's portrait sits **beside** the region rather than inside it, because
  the art belongs to the planet and a planet may later hold more than one
  opponent, so the two are laid out as separate elements now and neither later
  spec reworks the layout. Then on 2026-09-21 (R3, below): the art region
  becomes the screen's dominant element, with the text above and below it.
  **Its placeholder half was superseded on 2026-09-22 by R9** (below): the
  region now holds the planet's art, and the art was authored outside the
  codebase from this spec's brief rather than in a spec of its own.
- **N1 — the art region draws at 139 columns and wider only.** **Superseded
  2026-09-21 by R3**: it draws at every width.
- **O1 — while locked, broke is judged against the locked opponent's ante
  floor**, and the shop reserves that floor. The cheapest ante on the map may
  be on a planet the player is not allowed to play.
- **P1 — entering the campaign mid-series goes straight to the venue**; the map
  is not reachable while locked.
- **Q — per-planet music is out of scope**, its own spec (on `ROADMAP.md`).

### The first amendment (the person, 2026-09-21, after walking the venue)

- **R1 — the series length reads `Best of 3` / `Best of 5`**, the same words
  the map's planet detail uses, so the player meets one phrase for one idea.
  "first to 2" had been a session choice, never a ruling. (When R1 was ruled
  the map did not say it yet — T007 landed it later in the spec — and the
  amendment sign-off made the pause reports say so rather than claim it early.)
- **R2 — the shop is the "Card Shop" everywhere the player reads it**: the
  venue's action, the shop screen's own header, the key hints, How to Play,
  `Readme.md` and `docs/economy.md`. The person found "Outfitter" unclear, and a
  button that says one thing and opens a screen titled another reads as a
  defect. **Earlier specs' documents, `DECISIONS.md` and `ROADMAP.md` keep the
  old word on purpose** — they record what those specs did, and rewriting them
  would falsify history. Spec 029's own `spec.md` is the live contract and was
  corrected (caught at the amendment sign-off, which found the carve-out had
  been widened to all of `specs/**`). No symbol was renamed — `ShopState`,
  `open_shop`, `shop.rs` were never false — and `shop::TITLE` became the one
  string the venue's button and the shop's header both read.
- **R3 — the art dominates the screen, at every width.** The person's words: a
  much larger canvas allows more detailed, immersive art, and makes it feel like
  you are in the venue, on the planet. The venue became horizontal bands — the
  header rows above, the art and the portrait beside it, the action row and hint
  below. Two consequences the person ruled on when asked: **the portrait keeps
  its own column** (M1's reason survives intact), and **it draws at 89 columns
  too**, superseding N1 — a text-only venue at the minimum size is the version
  they least wanted. The cost, named at the time because it would land on a
  deferred art spec (R9 later brought the art into this one):
  **each planet's art must work at two quite different sizes.** R3 also cost
  one struct and one constant — `VenueRail` (which made "both regions or
  neither" a fact of the type) and `VENUE_ART_W` — **deleted** rather than kept
  as unconditional wrappers: a one-variant wrapper whose doc describes a
  condition that no longer exists is the false-name defect this spec renamed
  four symbols to avoid.

### The second amendment (the person, 2026-09-21, after walking the rebuilt venue)

- **R4 — the art about 15–20 % smaller.** Landed at **17.5 %** smaller at 89
  columns (58×23 → 50×22) and **16.8 %** at 139 (108×23 → 94×22) by giving the
  art **seven eighths** of the columns left of the portrait's gap. That is the
  **one scale factor** the plan takes, and a stated divergence from its own
  "the art's size is a subtraction, not a ratio" (plan §Design tension 7):
  R4 asks for a percentage, and no fixed column inset lands inside 15–20 % at
  both fit sizes (the workable insets, 7–9 and 12–17, do not overlap). The
  height is still a pure subtraction; there is still no minimum-art-size
  constant and no breakpoint. **R9 deleted the fraction** (below): the box now
  follows the drawings, whose sizes are R4's result, so R4 survives as those
  sizes and its band test still pins it; and the narrow and wide drawings now
  switch at 139 columns, the layout's existing threshold.
- **R5 — every text row centres on the art's centre, not the terminal's.** The
  person's eye caught it: the portrait's column puts the band's centre well
  right of the art box, so text centred on the terminal read as shifted off the
  thing it labels. Header and footer alike — aligning only the header would
  trade one mismatch for another. The consequence the person was warned of —
  at 89 columns the 63-character controls hint cannot fit over a 50-column art
  box and would land flush against column 0 — was resolved by **shortening the
  hint to 55 characters** with the board's own single-space ` · ` separators.
  Rejected, each for a stated reason: aligning only the header (`spec.md`
  forbids it — one mismatch traded for another); clamping the hint's column to
  a minimum (the row would be centred on nothing, bringing back for one row the
  off-centre look R5 removes); splitting the hint over two rows (it costs a row
  the art needs and puts a second text row in the footer band, where only the
  action row may have air); and widening the art to contain the hint
  (impossible at 89 columns — the widest art there is 58, and R4 wants about
  50). The shortened hint still overhangs the art box by 3 columns left and 2
  right at 89, which `spec.md` sanctions. `spec.md`'s "not flush against column 0" became
  an assertion: the fit test had been satisfying it silently, because
  `saturating_sub` clamps a left overflow to column 0 and the test checked only
  the right edge. The assertion now makes **61 characters** the ceiling for any
  future venue row at 89 columns.
- **R6 — the venue shows the credit balance**, in the Card Shop's own words:
  `shop::credits_label` is the one function both screens call, so the wording
  cannot drift. It is the screen where the player chooses between playing and
  shopping, which is the choice a balance informs (asked at the Phase 2 pause,
  answered here). Checked by the existing drawn-frame breathing test — which
  requires the row to be non-blank — rather than by a new test; the row's
  *content* is not asserted, which the plan sanctioned.
- **Plan §Open questions 2, closed by the person: the placeholder keeps the
  planet's name** until there is real art. (It had been asked at the Phase 2
  pause and re-put at the first amendment's walkthrough.) Since R9 the name is
  only the fallback for a planet with no art, which a validated delivery never
  has.
- **A deliverable, not a behavior: the per-planet art brief** ships on the
  branch as `specs/029-tournament-rounds/planet-art-brief.md`, in spec 016's
  shape. It asks for **two grids per planet — 48×20 and 92×20, sixteen files**
  under `assets/planets/` — because the art region's interior is a different
  width at the two layout sizes, which is the cost R3 named. It records the
  **single-asset alternative** (one 92-wide grid whose central 48 columns stand
  alone) in case the person preferred eight drawings to sixteen, and it priced
  four options for the **loading rule between the fit sizes** and chose none
  (T005e). The count was put to the person at the second amendment's pause;
  they answered with R7 and continued, so the brief stood at sixteen with the
  alternative recorded. There is no deferred art spec now: **R9** took both
  questions into this spec — sixteen drawings were delivered, and the person
  chose a rule the brief had not priced — **the box fits the art** — over all
  four it had.

### Two more rulings (the person, 2026-09-22)

- **R7 — a consistent, slight gap between the art and the portrait.** The
  person's words: the portrait sat a little too far to the right. The gap is
  now exactly `PANEL_GAP` (3 columns) at every width and the art-plus-portrait
  group is centred; the art's size (R4) and the text's alignment (R5) did not
  move. The outer margins are equal to within one column (at an odd leftover
  the right is one wider). Specified as a **class** of sanctioned assertion
  changes rather than a list — see *The gate pattern* below — and it landed
  first time.
- **R8 — How to Play says "matches."** How to Play opens with a rule about
  *rounds*, so "Opponents are Best of 3" could be read as rounds. The campaign
  lines now read "Each opponent is Best of 3 matches, the last / Best of 5. A
  started series is played out." (asked at the Phase 4 pause).

### R9 — the art, integrated (the person, 2026-09-22, at the merge pause)

- **R9 — the per-planet art is integrated in this spec**, rather than merging
  on the placeholder and leaving it to a spec of its own. The person's words:
  *let's add integrating the art into this spec.* **Authoring stays outside**:
  the sixteen drawings were made from `planet-art-brief.md` by the tool the
  person used (**Opus 5.5**, credited in `assets/CREDITS.md`), and this spec
  validates them and draws them. It added acceptance criterion **21** and a
  phase of its own (Phase 6), and narrowed the spec's art non-goal to
  *authoring*.
- **Between the fit sizes, the box fits the art** — the person's choice over
  the four options the brief priced (letterbox the drawing in a larger box,
  stretch it, tile or extend it, commission a third size). The box is always
  exactly the drawing plus its border — the narrow drawing below 139 columns,
  the wide one from 139 up, 20 rows tall — and space the drawing does not use
  is margin *around* the art-and-portrait group, never blank space inside the
  frame. At exactly 89×31 and 139×31 this is the layout R4, R5 and R7 had
  already approved, so nothing moved at the fit sizes (the orchestrator
  captured the venue at both before and after the geometry change, and the
  captures were identical). **The planet's name survives only as the fallback**
  for a planet with no art.
- **Three plan calls, each with its reason.** (1) **A taller terminal centres
  the whole venue vertically** (plan §Open questions 8): the 31-row
  composition keeps its internal spacing and the spare rows go above and below
  it, the odd one below, as the board's block does — so the text keeps its
  relationship to the picture it labels. The cost: above 31 rows the controls
  hint is no longer on the last row. (2) **The drawing is drawn at full
  strength** — `Emphasis::Normal`, the portraits' weight, inside a muted
  border (§Open questions 7): the brief told the artist the picture is drawn at
  one uniform emphasis with depth carried by glyph density, which dimming would
  compress. (3) **R4's fraction is deleted**: the box follows the drawings now,
  R4's result survives as their size, and its 15–20 % band test still pins it.
- **AC 21's checklist test derives its canvas from the venue's layout**
  (`VenueLayout::new(c).art` at each fit size) rather than restating 48, 92 and
  20 — which closes close-out note 37's concern that nothing tied the brief to
  the code. The test runs the brief's items 1–6 (exact set of visible files, 20 lines, one
  trailing newline, exact width in characters, the closed 23-character
  palette with its count asserted, distinct drawings) and checks for `\r`
  *before* the palette, so a CRLF file fails on the right assertion. Two
  mutation checks confirmed it bites (one extra space; one file converted to
  CRLF). Three more tests: every planet's art fills its box at all 660 sizes
  the venue's every-size tests measure (every width from 89 to 220, at 31, 32,
  33, 40 and 60 rows), the drawn frame's interior equals the drawing, and a
  planet without art shows its name.
- **`.gitattributes` marks `assets/planets/*.txt -text`.** Windows is a target
  and Git for Windows defaults to `core.autocrlf=true`. The first reason given
  for it was half wrong, and corrected at the Phase 6 review: a CRLF checkout
  would **not** mis-size the art, because `str::lines` strips `\r\n` and the
  drawer iterates `lines()` — but it **would** fail the new `\r` assertion, so
  `cargo test` would break on such a checkout. That alone justifies the rule.
  The portraits have no such protection, which is fine only because they have
  no `\r` test.
- **`design/brief.md` records the art as a second bounded exception** to its
  *Skeuomorphism boundary*, beside the portraits' (Phase 6 plan sign-off, B3:
  the portraits had been the *single* exception, amended before spec 016
  shipped them, and R9 needed the same amendment). Bounded the same way:
  static, venue-only, inside one single-weight box sized to the drawing, no
  colour, the portraits' glyphs plus four ASCII marks (`. ' * +`), no
  lettering, no discernible figures, original places. Written by the
  orchestrator on the branch and shown to the person at the Phase 6 pause.
- **The person's answers at the Phase 6 pause (2026-09-22).** The art tool was
  Opus 5.5. The person had **Cinder's and Scree's art redrawn "with right
  angles only"** and committed it themselves (`bc2598c`), then said to
  continue — read as the go on the art (the brief's checklist item 7, the
  product owner's look). The redraw was validated like the first delivery:
  items 2–5 by shell on the four files, and AC 21's test over all sixteen.
  Three things were shown and **not ruled on**; see *Asked, and not answered*.

### Asked, and not answered

- **Plan §Open questions 1 — the deciding match's game-over frame shows no
  series score.** The in-match score is derived from the live series, and the
  deciding match ends the series during settlement, one tick before the
  game-over popup; so after a non-deciding match the band shows the updated
  score, and after the deciding one (won or lost) it shows nothing, while the
  map banner that follows names the result. The plan sign-off ruled it **not an
  AC 11 violation** (the score is on every frame the player can act in). Put to
  the person at the Phase 3 pause, with both frames; **they continued without
  ruling. Left as built, and not treated as a ruling either way.** Holding the
  final score is a one-shot `App` field in the `victory_due` idiom, a
  sub-lettered task rather than a redesign, if they later want it.
- **Three things shown at the Phase 6 pause (2026-09-22)**: the art's weight
  (plan §Open questions 7 — drawn at full plain weight; dimming it is a
  one-word change), vertical centring on terminals taller than 31 rows (§Open
  questions 8 — the hint leaves the last row), and the text of the
  `design/brief.md` amendment. **The person continued without changing any of
  them. Left as built**; the amendment text stands as written.

### The design calls (plan, signed off 2026-09-20)

- **The return target is derived from the lock, never remembered.** Spec 015
  shipped a bug where a *remembered* origin was not set on one path; a second
  remembered target (a venue origin on the deck builder and the shop) would
  have doubled the places that can forget. Instead `App::open_campaign_home` is
  the **only** place that assigns `Screen::CampaignMap` or `Screen::Venue`, and
  it reads whether a series exists. Every door — Continue, New Campaign, Reset
  Everything, the game-over acknowledgement, Back from the shop and the deck
  builder, the map's launch — goes through it, and the deck builder's
  invalid-deck divert gets the venue return for free. **The invariant is held by
  a reviewed grep, not by the type system**:
  `grep -nE "self\.screen = Screen::(CampaignMap|Venue)" src/app.rs` must return
  exactly two lines, both in `open_campaign_home`. It is written as two
  assignment statements rather than one `if` expression precisely so the grep
  can see them — in the expression form neither line names a variant and the
  gate would pass while checking nothing (T006's deviation, upheld at the
  Phase 2 review, which also widened the grep three ways). A `CampaignHome`
  enum with a mapping test was drafted and dropped: the test would have been a
  tautology aimed at the wrong risk.
- **One floor, one predicate: `economy::reserve_floor`.** O1 changes what the
  floor *is* while locked, not how many there are. `cheapest_floor` kept its
  body and became private; `reserve_floor` returns the locked opponent's ante
  while a series runs and the cheapest launchable ante otherwise, and all of
  `Profile::is_broke` and `Profile::can_afford` (and through it the shop's
  purchases, dimming and *spendable* readout) read it. While locked, the
  reserve **is** the venue opponent's own ante, so a player at the venue who is
  not broke can always cover the match it offers and the venue draws no banner
  — **but only because the map's launch refuses a series the balance cannot
  cover**, which the pre-merge sweep found missing (its blocking B1): the floor
  rises at the moment a series starts, and without that check a player with 15
  credits could launch a 20-ante series, be locked into it with no playable
  match, and lose the run at the next campaign entry. One helper,
  `App::refuse_uncovered`, now guards both the map's series launch and every
  wager. The wager prompt's warning reads `economy::reserve_after_a_loss`
  instead — see *O1's side effect* below.
- **Settling exactly once stays a data property, and now covers the series.**
  Spec 021 made the payout idempotent by zeroing the escrow; a series tally
  increment has no such property, and `mark_beaten` on the wrong match would
  clear a planet early. So the in-flight `NodeRef` carries a `settled` flag and
  `CampaignRun::take_settlement` takes the node and its stake **once** — a
  second call returns `None`, pays nothing, moves no tally and beats nobody.
  `take_stake` was deleted: two ways to empty one escrow is one too many. **What
  it still does not cover**: `Profile::record_match`, which `resolve_match` runs
  *before* settlement, so a second `resolve_match` on one match would still
  double-count lifetime and run statistics — pre-existing, still guarded only by
  the `phase_changed && GameOver` edge, and stated in `take_settlement`'s doc
  rather than claimed away. One visible consequence in tests only: a second
  settlement now returns `None` where it returned `Some(Won(0))`; the claim that
  this changes nothing in production rests on inspection of that edge, not on a
  test.
- **The series is stored beside the in-flight pointer, not inside it**, because
  the two have different lifetimes: a Quick Play match and the kill-with-no-save
  forfeit both clear the pointer, and neither may end a series. So a Quick Play
  match started mid-series cannot end it, and the board's score line is gated on
  the in-flight node *matching* the series, not on a series existing.
- **How many wins a series needs is derived from the opponent id**
  (`wins_needed`, `FINAL_OPPONENT = "sovereign"`), never stored: a stored count
  is a second source of truth a hand-edited or older save could contradict, and
  derived, a pre-029 profile gets the right length for free. One constant
  rather than a roster field, because `opponent.rs` is balance data this spec
  must not touch.
- **The migration rule: `is_opponent_beaten`, not the presence of a series,
  decides whether a settled match can beat its opponent** (plan sign-off B1).
  The first draft settled a match left in flight across the upgrade as a
  non-series match, so a player mid-match against Greeb when they upgraded would
  have cleared Cinder in one win — the outcome this spec exists to remove. Now
  `record_series_match` **begins a series** when none is running and the
  opponent is un-beaten, and credits the match to it, exactly as `spec.md`'s
  "a match left in flight resolves as the first match of a fresh series" says.
  So a beaten opponent is a rematch and an un-beaten one is always in a series,
  and **no settled match can beat an opponent who has not lost a series**. The
  `NotInSeries && player_won` clause that remains in the beat condition can
  only ever re-mark an already-beaten opponent — a no-op kept to preserve the
  rematch path literally.
- **Four renames, each because the old name would assert something false**:
  `take_stake` → `take_settlement`; `cheapest_floor` → `reserve_floor` (the old
  one kept, private, as its implementation); `BuilderOrigin::Map` /
  `BackTo::Map` → `…::Campaign` (the builder no longer necessarily returns to
  the map); `map_entry_modal` → `campaign_entry_modal`. `enter_campaign_map` →
  `enter_campaign` and `open_campaign_map` → `open_campaign_home` followed from
  the first design call. `src/wager.rs`'s reserve doc was corrected with them
  (plan sign-off B6): exempting it from the rename's grep would have satisfied
  the gate and left the false claim standing.

### O1's side effect, which no walkthrough could reach

The wager prompt's warning row — **"Lose this and the run is over."**, added by
the 2026-09-17 chore — is driven by the prompt's reserve, which is now
`reserve_floor`. While locked against a deep opponent that reserve rises from
the map's cheapest (10, Cinder's rematch, in every run state) to **that
opponent's own ante** — 50 against Rix and up — so the warning fires at far
lower stakes than it did. That is correct under O1 and probably desirable: the
warning still reads the same floor the broke check reads. But it is a real
change in how often a player sees that row, it needs a Core-depth run that no
walkthrough in this spec reached, and so it is recorded here rather than
attested. It also **supersedes, while a series is locked, the chore's bullet
"The predicate is the run's cheapest ante, not the prompt's own floor"**: while
locked the two are the same number. The chore's own section above is left as
written.

**Fixed on the branch by the pre-merge sweep (its N1).** A loss that decides a
series releases the lock, so the broke check after it reads the map's cheapest
ante again; the warning had been reading the locked opponent's ante and could
say "Lose this and the run is over" when the run would continue. The prompt now
takes `economy::reserve_after_a_loss` — the floor the broke check reads after
this match is lost, found by recording the loss on a copy of the run so the
series' own rule decides it — and so again predicts exactly the check that ends
the run, as the 2026-09-17 chore ruled. It only ever over-warned; it never
under-warned.

### Coverage, stated honestly

- AC 14 is two-thirds instrumented: `every_reset_clears_the_lock` covers New
  Campaign and Reset Everything; the run-over reset is covered by the fact — in
  a comment — that it is the same call.
- AC 16's "the largest element on the screen" is asserted only as art area >
  portrait area, which is sufficient while every other element is a single row
  of text; a later spec adding a second panel would not be caught.
- The venue's credit row (R6) is pinned by being non-blank and by the shared
  `credits_label`, not by an assertion on its text.
- `the_deciding_match_and_the_series_agree` plays only all-win and all-loss
  series; `wins_needed_is_two_except_for_the_final_opponent`'s loop computes its
  expectation with the function's own expression and is rescued by its other two
  assertions — don't trim it to the loop.
- `no_settled_match_beats_an_unbeaten_opponent_outright` reads
  `is_opponent_beaten` *after* the call, which is sound only because
  `record_series_match` marks nobody beaten; if `mark_beaten` ever moves into
  it, that assertion silently changes meaning.
- A non-deciding match's `MapBanner::Settled` is set and never shown as a
  banner (the venue draws none); nothing is lost, because `stake_to_show` puts
  the settled amount on the game-over frame, but the case was un-discussed
  rather than decided. A contradictory banner ("Series won · Lost N credits")
  is unreachable in play — only a hand-edited save could produce it.
- The winning series banner (`★  Series won · …`) was never seen on screen —
  six auto-played attempts lost — and is pinned by an exact-string test.
- AC 21's checklist test is the whole of the validation a machine can do.
  Nothing can test `assets/CREDITS.md`'s originality claims — original places,
  no franchise imagery — which rest on the person's look at the art.
- `T003` renamed the `sweep_run` test helper to `sweep_run_in_series` and
  changed its behaviour where the task line said helpers should *gain* a
  series form; the rename is total, so every call site shows in the diff, which
  is what that bar protected.

### The gate pattern (process, recorded so the next spec plans against it)

This spec hit, repeatedly, **a Verify gate or an "exhaustive" list of changing
assertions that could not be satisfied or was incomplete**: T001's
one-exception list (three assertions had to move; a decision review ruled it a
planner enumeration error), plan sign-off B2/B3/B6 (Verify gates unsatisfiable
because call sites were under-enumerated — B6 was `src/wager.rs`, listed under
*No change* while it named the renamed function), T006 (the plan's code listing
would have made its own grep gate return zero lines), T005b (a gate spelling the
file `README.md` when it is tracked as `Readme.md` — harmless on darwin,
silently unsatisfiable on a case-sensitive filesystem), T004a and T005c (each
changed an assertion or a doc its "anything else is a stop-and-report" list
omitted, and neither implementer stopped), T007 (an unlisted construction site),
and the second amendment sign-off's B1 (a `git diff --stat` gate for a task
whose whole product is one untracked file, which that command cannot list — a
lesson specs 022 and 023 had already written down). **What worked**: T005f
stated the sanctioned change as a **class** ("any assertion whose subject is
the portrait's x position, the art–portrait gap, or the right-hand margin")
rather than a list, and landed first time; T008 enumerated its call sites by
grep before editing and found no gap. Related orchestrator misses: review
bundles showed `cargo test -q --lib` plus a warning-count grep instead of the
constitution's full command — three recurrences before it stuck — and the grep
returns 0 for a failed build as readily as a clean one; and two bundles echoed a
gate's *label* rather than the command actually run. Echo the command, don't
retype it.

Phase 6 added two more of the same kind at its plan sign-off, both caught
before dispatch: **B1**, a grep gate for deleted constants that substring-matched
`CampaignMapLayout`'s `FIELD_MARGIN_X` and so could never come back empty (fixed
with `grep -w`); and **B2**, a "clean working tree" check that an implementer
who never commits cannot satisfy (scoped to `assets/planets`).

**And one orchestrator miss of a different kind: the art was committed before
it was validated.** The art session wrote the sixteen files into
`assets/planets/` while the orchestrator was committing the Phase 6 plan with
`git add -A`, which swept them into that commit (`7445c68`) unvalidated and
unmentioned in its message. History was not rewritten (never force-push); the
delivery checkpoint, T013, validated the files already committed instead of
committing them, and they passed first time. The Phase 6 review checked the
commit: exactly the sixteen art files plus the three spec documents, nothing
else swept in. From then on the orchestrator stages **explicit paths, never
`git add -A`** — a checkpoint that validates before committing only works if
nothing else can commit first.

### What didn't change

`game.rs`, `card.rs`, `player.rs`, `save.rs`, `opponent.rs`, `Cargo.toml` and
`Cargo.lock` are untouched; `PROFILE_VERSION` and `SAVE_VERSION` stay 1; the two
new persisted fields are `#[serde(default)]`, the pattern spec 021 used for
`NodeRef::stake`; `economy.rs` lost only two doc lines and the `pub` on
`cheapest_floor` — no constant moved; no new crate; the build has no warnings,
the same count as `main`. Monochrome by construction: the venue, the series line
and the banners use the existing emphasis levels and add none, and the art is
drawn at the portraits' plain weight from a closed 23-character palette that
admits no escape character. The art adds sixteen text files, a
`.gitattributes` line and a `CREDITS.md` section — no crate, and no engine or
save file.
```

---

## 3. Not drafted here (deliberately)

- **`Readme.md` is on the branch** — the campaign paragraph names the series,
  the venue, the lock and the locked-opponent floor (T009), and the status
  blockquote says **Card Shop** (T005b, reflowed whole in T009). It rides into
  `main` with the merge. The tracked name is `Readme.md`.
- **`docs/economy.md` and `docs/balance.md` are on the branch** — the reserve
  floor, the settlement and the Card Shop wording (T002, T003, T005b, T007), and
  *Series rates* (T010). No close-out edit.
- **`design/brief.md` is on the branch** — its *Skeuomorphism boundary* gained a
  second bounded exception for the venue art (R9; Phase 6 plan sign-off B3),
  written by the orchestrator and shown to the person at the Phase 6 pause. It
  rides into `main` with the merge. The question the first draft of this
  close-out left open here — whether the art needed its own amendment — is
  **closed**: it did, and it has one. Otherwise the venue uses the existing
  emphasis vocabulary and the constitution's *acted-on element stands apart*
  rule as written (an empty row above and below the action row; the modals it
  can raise pad evenly through `OverlayLayout`).
- **`assets/CREDITS.md` and `.gitattributes` are on the branch** — the *Venue
  art* section naming Opus 5.5 as the tool, and the `-text` rule for the art
  files (T014). They ride into `main` with the merge.
- **No `CLAUDE.md` amendment.** The venue is a new `Screen` — a full mode the
  player navigates *to* — which is exactly what the Architecture section asks
  for, copied from `opponent_select.rs`'s shape (one state struct,
  `handle_input` returning one owned `VenueOutcome`, `draw`) and wired into the
  three `match &self.screen` arms. `GamePhase` is untouched; the
  draw-never-mutates boundary holds; the verification command is unchanged; no
  new crate. The session-model change of 2026-09-22 (the person switched this
  session to `claude-opus-5-5` with `/model`) was session-only by the
  constitution's own rule and edited no role-table row; it is in the tier log.
- **`ROADMAP.md`'s Outfitter mentions stay** (R2). A one-line chore if the
  person ever wants the roadmap's *open* items to say Card Shop.

---

## 4. The close-out notes, triaged

All 56 notes gathered in `tasks.md` (*Notes for the close-out*), plus the items
the tier log carried to the sweep outside that list. Each is **resolved**
(nothing owed), **DECISIONS** (recorded in §2a), **ROADMAP** (a follow-up in
§1), or **sweep** (listed in §4b for the first pre-merge sweep, or §4c for the
second, to rule on — the sweep decides code; this close-out changes none).
Notes 1–51 were triaged before R9 and went to the first sweep; notes 52–56 come
from the Phase 6 review and were triaged at the refresh, with 49–51 re-checked
alongside them.

### 4a. The 56 notes

| # | Subject | Disposition | Where / why |
|---|---|---|---|
| 1 | `resolve_match` / `settle_campaign_match` docs said `None` = Quick Play only | resolved | Both docs now name the already-settled case (`src/profile.rs` ~331, ~370) |
| 2 | `app.rs` test `NodeRef` carries `settled: false` after settlement | sweep | S1 |
| 3 | T001 evidence was `--lib` + a warning count | DECISIONS | *The gate pattern* (orchestrator misses); T003's review did get the full command |
| 4 | `wins_needed` test loop is self-referential | DECISIONS | *Coverage, stated honestly* |
| 5 | `no_settled_match_…` reads `is_opponent_beaten` after the call | DECISIONS | *Coverage, stated honestly* (the name's overclaim is sweep N4) |
| 6 | `Some(Won(0))` → `None` "nowhere in production" rests on inspection | DECISIONS | *Settling exactly once* bullet. (The note asked for a walkthrough-list line; none was added, and there is nothing a person could see — recorded instead) |
| 7 | plan §Tests' `can_afford` numbers impossible | resolved | Plan corrected in place; test asserts the real claim |
| 8 | `docs/economy.md` can't name `cheapest_floor_is_the_min_…` because of the grep gate | sweep | S2 |
| 9 | `docs/economy.md` named the T006 functions ahead of T006 | resolved | T006 landed those exact names; T007 fixed the stale paragraph; no "later in this spec" parenthetical remains |
| 10 | No mixed win–loss–win series test | sweep | S3 |
| 11 | Double-resolve test infers "beats once" | sweep | S4 |
| 12 | `docs/economy.md` step 1 / step 2 split muddled | sweep | S5 |
| 13 | `docs/economy.md` test list lacks the series tests | sweep | S6 |
| 14 | Double-resolve still double-counts `matches_played` | DECISIONS | *Settling exactly once* — what it does not cover |
| 15 | Phase 1 walkthrough attested a path Phase 2 stops using | resolved | The Phase 2 walkthrough re-attested the two-wins rule through the venue (AC 6, 7) |
| 16 | `is_broke_reads_…_cheapest_launchable_ante` name | sweep | S7 |
| 17 | AC 14 two-thirds instrumented | DECISIONS | *Coverage, stated honestly* |
| 18 | `sweep_run` → `sweep_run_in_series` changed behaviour | DECISIONS | *Coverage, stated honestly* (for the record) |
| 19 | Mid-series replacement confinement | resolved | CLOSED at the Phase 2 review, grep run and widened |
| 20 | Non-deciding `Settled` banner set, never shown as a banner | DECISIONS | *Coverage, stated honestly* |
| 21 | Tests read the real profile; `save()` has no `cfg(test)` guard | sweep | S8 — and §1f drafts the roadmap text if the sweep leaves it |
| 22 | `ACTION_GAP` duplicates `CHOICE_GAP` | sweep | S9 |
| 23 | Venue deck-guard divert untested and un-walked | resolved | Attested by the person (tier log) after two reviews verified it by inspection |
| 24 | `open_wager` silent on a lookup miss while `draw` falls back | sweep | S10 |
| 25 | Bundle echoed a gate label, not the command | DECISIONS | *The gate pattern* |
| 26 | T004a under-enumeration | DECISIONS | *The gate pattern* |
| 27 | `portrait_y1`'s no-op clamp / no clamp against `rows` | sweep | S11 |
| 28 | Bands test hardcodes `3` / `cols − 4` beside `MARGIN_X` | sweep | S12 (re-check against T005f's rewrite first) |
| 29 | "Dominates" asserted only against the portrait | DECISIONS | *Coverage, stated honestly* |
| 30 | `row_text` copied from `board.rs` (now in three modules) | sweep | S13 |
| 31 | `Readme.md` line ~86 columns after the rename | resolved | T009 reflowed the blockquote whole; the one remaining 86-column line (21) is identical on `main` |
| 32 | T005a's gate output not in the bundle | DECISIONS | *The gate pattern* |
| 33 | `--lib` + warning grep, third recurrence | DECISIONS | *The gate pattern* |
| 34 | T005c under-enumeration | DECISIONS | *The gate pattern* |
| 35 | Credit row content untested | DECISIONS | *Coverage, stated honestly* |
| 36 | Binding-case fit assertion pinned to a literal `89×31` | sweep | S14 |
| 37 | Three soft spots in the art brief | resolved | Fixed in T005e before hand-off; and since R9, AC 21's checklist test derives the canvas from `VenueLayout`, so the brief is tied to the code (§2a, *R9*) |
| 38 | "Equal outer margins" only to within one column | sweep | S15 (the DECISIONS text already says "to within one column") |
| 39 | Two badly-reading doc sentences from T005f | sweep | S16 |
| 40 | `MARGIN_X` no longer names an on-screen margin | sweep | S17 |
| 41 | In-flight-across-upgrade match shows no score; doc lists three cases | sweep | S18 |
| 42 | `launchable_opponent`'s doc contract false for `series_length_detail` | sweep | S19 |
| 43 | Contradictory banner unreachable | DECISIONS | *Coverage, stated honestly* |
| 44 | Banner/tally test passes `NotInSeries` | sweep | S20 |
| 45 | plan §Open questions 1 unanswered | DECISIONS | *Asked, and not answered* |
| 46 | How to Play's "Best of 3" names no unit | resolved | Ruling R8, T009a |
| 47 | `Readme.md`'s rematch / lock / floor sentences | sweep | S21 |
| 48 | `help_texts_name_the_new_keys_and_nothing_old` undersells | sweep | S22 |
| 49 | `docs/balance.md` series cells differ by 0.1 when recomputed | sweep | S23 |
| 50 | Balance-value gate half missing from the bundle | resolved | Closed by the orchestrator; re-checked in §5 |
| 51 | Starter vs Rix drift ~2.8 SE | ROADMAP | §1e, the re-tuning follow-up |
| 52 | `assets/CREDITS.md` named no art tool (a visible marker) | resolved | The person named **Opus 5.5** at the Phase 6 pause and it replaced the marker (`7e5fe11`); `grep -n "ART TOOL" assets/CREDITS.md` is empty (§5). Was a merge gate for the second sweep; no longer |
| 53 | `.DS_Store` trap in AC 21's `read_dir` exact-match check | resolved | P1 (§4c) — the second sweep ruled fix-now; **fixed on the branch** (T014a): hidden files are skipped, a misnamed file still fails |
| 54 | `spec.md`'s brief paragraph still said the art non-goal "stands" | resolved | Corrected by the orchestrator with a pointer to R9 (`spec.md`, *A new deliverable*, "Superseded the next day by R9") |
| 55 | The `.gitattributes` rationale half wrong in the record | DECISIONS | §2a, *R9* — the corrected reason (a CRLF checkout fails the `\r` assertion; it does not mis-size the art) |
| 56 | `CREDITS.md`'s originality claims are untestable | DECISIONS | *Coverage, stated honestly* |

**Counts: resolved 12, DECISIONS 19, ROADMAP 1, sweep 24** (56 in all).
Resolved: 1, 7, 9, 15, 19, 23, 31, 37, 46, 50, 52, 54. DECISIONS: 3, 4, 5, 6,
14, 17, 18, 20, 25, 26, 29, 32, 33, 34, 35, 43, 45, 55, 56. ROADMAP: 51. Sweep:
2, 8, 10, 11, 12, 13, 16, 21, 22, 24, 27, 28, 30, 36, 38, 39, 40, 41, 42, 44,
47, 48, 49 (the first sweep), and 53 (the second).

Re-checked at the refresh: **49** stays with the first sweep, which ruled it
leave-and-record — `docs/balance.md` still carries no "unrounded" clause
(`grep -n unrounded docs/balance.md` is empty), so the recomputation caveat
lives only in S23 below; **50** holds after Phase 6 — `src/opponent.rs` still
untouched and `src/economy.rs`'s removed lines unchanged (§5); **51** stays in
§1e.

### 4b. The pre-merge sweep list

Everything here is non-blocking as recorded; the sweep decides whether each is
fixed on the branch, recorded, or left. None is a spec-conformance failure
except possibly **N1**, which is new and the one to look at first.

> **The sweep's rulings (2026-09-22).** It found one **blocking** defect not on
> this list — **B1**, a series could start against an opponent the balance could
> not cover — and ruled **fix-now** on B1, **N1**, **N2** and **N8**; all four
> were fixed on the branch as T011a, re-reviewed clean, and driven in the game.
> **Every other item is leave-and-record**, with the reasons in the sweep's
> report as summarised in `tasks.md`'s tier log; **nothing needs the person.**

**New, found while drafting this close-out:**

- **N1 — the "run is over" warning can be wrong on a match that could lose the
  series.** The wager prompt's warning (`WagerState::loss_ends_the_run`,
  `src/wager.rs` ~131) compares `credits − stake` against the reserve taken at
  launch, which while locked is the locked opponent's ante. But a **deciding
  loss releases the lock** — `record_series_match` sets `series = None` on
  `Lost` (`src/campaign.rs` ~388) — and the broke check that follows, in
  `enter_campaign` on the game-over acknowledgement (`src/app.rs` ~833), reads
  `reserve_floor` again, now the map's cheapest (10). Example: locked against
  Rix (ante 50) at 0–1, 80 credits, stake 40 → the prompt says "Lose this and
  the run is over."; lose, and the series is lost, the lock released, 40 ≥ 10,
  and the run continues. The chore of 2026-09-17 ruled that the warning "cannot
  disagree with the condition that actually ends the run"; on the potentially
  deciding match of a series it now can (it over-warns; it never under-warns —
  a non-deciding loss keeps the lock and the floors agree). AC 10 is about the
  broke check, which is right; this is the warning. Options as seen, not
  chosen: make the reserve at launch account for a loss that would decide the
  series (a small change to what `launch`/`open_wager` passes), or accept and
  record it. A product-visible string is involved, so if the fix is not
  obviously what the chore already ruled, it is a question for the person.
- **N2 — `src/wager.rs`'s own docs still describe the pre-O1 rule.** The
  `reserve` field doc (~36–38: "The run's cheapest ante … Not this prompt's
  `floor`, which is *this* opponent's ante and can sit above the cheapest node")
  and `loss_ends_the_run`'s doc (~128–129: "the run's cheapest ante") are false
  while locked, when the reserve *is* this opponent's ante. B6 corrected
  `WagerState::new`'s doc and the test helper; these two survived because the
  gate grepped for `cheapest_floor`, not "cheapest ante". Doc only.

**Carried from the tier log, outside the 51:**

- **N3 — Start Campaign skips the Continue / New Campaign / Reset Everything
  panel with a live 0–0 series** (Phase 2 walkthrough, incidental finding):
  `Profile::differs_from_starter` reads `campaign.has_progress()`, and a series
  is not progress by that definition, so a player who starts a series and
  quits has no visible door to New Campaign until they play a match. Not a
  spec violation (AC 9, AC 14 hold). For the sweep to rule on.
- **N4 — sign-off second-look (8)**: `no_settled_match_beats_an_unbeaten_opponent_outright`
  lives in `campaign.rs` but the property it names is pinned jointly with
  T003's test in `profile.rs`, so the name overclaims for where it sits.
- **N5 — sign-off second-look (9)**: the `|| (NotInSeries && player_won)` clause
  in `settle_campaign_match`'s `beats` is documented as provably a no-op; a
  branch whose own comment says it does no work is the mild smell CLAUDE.md's
  *Simplicity* names.
- **N6 — first amendment sign-off S4**: `FOOTER_H` / `action_y` double encoding
  (deliberately left for the sweep by T005c); also T004a's finding that
  `FOOTER_H` is `pub` with no reader outside `new`.
- **N7 — first amendment sign-off S5**: nothing asserts that the art region
  *draws* at 89 columns — R3's most load-bearing change — beyond the layout
  test's Rects.
- **N8 — first amendment sign-off S7**: `shop.rs` and `app.rs` doc staleness
  predating the amendment.
- **N9 — T004a finding**: `the_art_region_dominates_at_both_widths` destructures
  `fit_sizes()` as `[narrow, wide]` and would silently invert if that order
  changed (pairs with S14).
- **N10 — T005c finding**: the plan's dictated doc text uses British "centre"
  where the surrounding code says "center", so `layout.rs` carries both within
  a few lines.

**From the 51 (numbers in brackets are the note numbers):**

- **S1** [2] `src/app.rs` ~2877: a test `NodeRef` with `settled: false` in a
  test whose header says the match has settled — models an impossible state.
- **S2** [8] `docs/economy.md` cannot name `cheapest_floor_is_the_min_over_launchable_nodes`
  while T002's grep gate stands; rename the test or relax the gate, or leave.
- **S3** [10] Add the two-line mixed case (win–loss–win, and won the match /
  lost the series) to `the_deciding_match_and_the_series_agree`; T007's
  `banner_line` leans on the claim.
- **S4** [11] Add `assert!(!q.campaign().is_opponent_beaten("cinder", "greeb"))`
  to `resolving_a_match_twice_pays_beats_and_counts_once`.
- **S5** [12] `docs/economy.md` step 1 attributes the `settled` check that
  step 2's `take_settlement()` owns.
- **S6** [13] `docs/economy.md`'s test list lacks the new series tests.
- **S7** [16] `is_broke_reads_the_balance_against_the_cheapest_launchable_ante`
  now reads `reserve_floor`; the name is accurate only for its no-series half.
- **S8** [21] Test isolation: a scratch data dir for tests, and/or a
  `cfg(test)` guard on `Profile::save()`. If left, apply §1f.
- **S9** [22] `venue.rs`'s `ACTION_GAP = 6` / `action_row_width()` duplicate
  `app.rs`'s private `CHOICE_GAP` / `choice_row_width`, untested for equality.
- **S10** [24] `open_wager` silently does nothing on an unknown id while the
  venue's `draw` falls back — the two sites disagree about tolerance.
- **S11** [27] `src/layout.rs` `portrait_y1`: the `.max(band_y0)` is a no-op
  clamp, and it is the one derived edge with no clamp against `rows`.
- **S12** [28] The bands test hardcodes the margin rule as `3` / `cols − 4`
  though `VenueLayout::MARGIN_X` is visible (check what T005f left).
- **S13** [30] `row_text` test helper now copied into three modules.
- **S14** [36] The binding-case fit assertion is a literal `Config { 89, 31 }`
  outside the `fit_sizes()` loop; read it from `fit_sizes()[0]`.
- **S15** [38] Plan note, `MARGIN_X`'s doc and the bands-test comment say
  "equal outer margins" unqualified; add "to within one column" (160 columns:
  11 and 12).
- **S16** [39] `text_x`'s and `ART_W_NUM`'s rewritten docs read badly and
  overrun the wrap.
- **S17** [40] `MARGIN_X` is now only a sizing input, not a visible margin —
  rename or leave.
- **S18** [41] `board_series_line`'s doc lists three no-score cases and misses
  the fourth: a match left in flight across the upgrade (no series exists until
  it settles).
- **S19** [42] `launchable_opponent`'s doc says callers gate on
  `planet_unlocked`; `series_length_detail` does not (by design — locked Zenith
  shows `Best of 5`).
- **S20** [44] `the_banner_and_the_run_tally_report_the_same_net_gain` passes
  `NotInSeries` rather than `settled.series`.
- **S21** [47] `Readme.md`: the rematch sentence doesn't say rematches are
  single matches from the map with no venue; "the map stays closed until you
  win it or lose it" omits New Campaign / Reset Everything; the locked-floor
  clause reads as a second check rather than a replacement.
- **S22** [48] `help_texts_name_the_new_keys_and_nothing_old` also carries the
  series assertions; its name undersells it.
- **S23** [49] `docs/balance.md` *Series rates*: add "computed from the
  unrounded rates of the 2026-09-22 run" so a hand recomputation from the
  one-decimal per-match column (or from the older table) is not reported as a
  discrepancy.

**23 S-items from the 51, and 10 N-items: 33 sweep items in all.**

**Mooted since by Phase 6.** T012 deleted `VenueLayout::MARGIN_X`, `ART_W_NUM`,
`ART_W_DEN` and `span_w` (`grep -nwE "MARGIN_X|ART_W_NUM|ART_W_DEN|span_w"
src/layout.rs` is empty), so **S12** (the bands test's `3` beside a visible
`MARGIN_X` — the literal `3` is now the only form of that bound), **S17**
(`MARGIN_X`'s name) and the `ART_W_NUM` half of **S16** refer to code that no
longer exists, and **S15**'s "`MARGIN_X`'s doc" is gone with it. All four were
leave-and-record; nothing is owed.

### 4c. The second pre-merge sweep list (scoped to Phase 6)

The first sweep's findings are closed (T011a, re-reviewed). The second sweep is
the orchestrator's to dispatch, on the Phase 6 diff; this is what the close-out
hands it from the notes:

- **P1** [53] **A `.DS_Store` trap.** *(Ruled fix-now by the second sweep and
  fixed on the branch as T014a.)* `every_planets_art_passes_the_briefs_checklist`
  lists `assets/planets/` with `read_dir` and demands an exact match with the
  sixteen expected names, and `.gitignore` hides `.DS_Store` — so if Finder
  ever opens that folder, `cargo test` fails on this machine while
  `git status` shows nothing. Skipping dotfiles in the listing would remove the
  trap. (No `.DS_Store` is there today: `ls -a assets/planets` shows only the
  sixteen files.) **The sweep to rule.**
- **Note 52 is no longer a merge gate**: `assets/CREDITS.md` names the tool
  (§4a, §5).
- The seam named in the task line, for the sweep itself to check: nothing from
  Phases 1–5 — the brief and the tests' comments — still assumes the art box
  grows with the terminal.

**1 sweep item from the notes (P1), plus the seam check.**

---

## 5. Mechanical checks (T011 refresh, run on the branch at `7e5fe11`, 2026-09-22)

**Every check below was re-run fresh after Phase 6**; the first run's outputs
(at `7d323eb`) predate the art and are not evidence for it, so they are
replaced rather than kept. `main` was at `a5a854c` for every comparison below.
Nothing was applied to `main` and no branch was switched: `main`'s content was
read with `git show main:<path>`, and `main` was built in a throwaway
`git worktree` with its own `CARGO_TARGET_DIR`, removed afterwards. The working
tree was clean (`git status --short` empty, `HEAD` = `7e5fe11`) when the checks
ran; this document's refresh is the only uncommitted change.

### `cargo test -q`, three consecutive runs

Preceded by `cargo build --all-targets 2>&1 | tail -n 20` →
``    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s``.
`tail -n 25` reaches only the last few of the **eleven** test binaries. The
tails, verbatim (each run exited 0):

```
=== run 1 ===

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

=== run 2 ===

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

=== run 3 ===

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Because that tail hides the counts, the same three runs filtered to every
`running` / `test result` line — **488 + 7 + 7 = 502 passing, 0 failing, 1
ignored** (the long balance simulation, ignored since spec 022), identical
across the three runs. The library's 488 is the first run's 482 plus the one
test T011a added, T012's one new layout test and T014's four new venue tests
(counted by `#[test]` in `src/` at `7d323eb`, `19810a7` and `HEAD`: 482, 483,
488):

```
=== run 1 ===
running 488 tests
test result: ok. 488 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 8 tests
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
=== run 2 ===
running 488 tests
test result: ok. 488 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 8 tests
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.99s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
=== run 3 ===
running 488 tests
test result: ok. 488 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 8 tests
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.99s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### AC 21's tests, by name

```
$ cargo test --lib -- every_planets_art_passes_the_briefs_checklist every_planets_art_fills_its_box_at_every_size the_venue_draws_the_planets_art_inside_its_box a_planet_without_art_shows_its_name the_art_box_is_the_drawing_plus_its_border_at_every_size 2>&1 | grep -E "^test |^test result"
test venue::tests::a_planet_without_art_shows_its_name ... ok
test layout::tests::the_art_box_is_the_drawing_plus_its_border_at_every_size ... ok
test venue::tests::every_planets_art_passes_the_briefs_checklist ... ok
test venue::tests::the_venue_draws_the_planets_art_inside_its_box ... ok
test venue::tests::every_planets_art_fills_its_box_at_every_size ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 483 filtered out; finished in 0.03s
```

T014's four (the checklist, the fill at every size, the drawn frame, the
fallback) and T012's every-size geometry test.

### The branch's whole diff against `main` (three-dot, since `main` may move)

```
$ git diff main...HEAD --stat=120
 .gitattributes                                    |    1 +
 Readme.md                                         |   40 +-
 assets/CREDITS.md                                 |   12 +
 assets/how_to_play_text.txt                       |    4 +-
 assets/planets/ashfall-narrow.txt                 |   20 +
 assets/planets/ashfall-wide.txt                   |   20 +
 assets/planets/cinder-narrow.txt                  |   20 +
 assets/planets/cinder-wide.txt                    |   20 +
 assets/planets/drift-narrow.txt                   |   20 +
 assets/planets/drift-wide.txt                     |   20 +
 assets/planets/karrus-narrow.txt                  |   20 +
 assets/planets/karrus-wide.txt                    |   20 +
 assets/planets/scree-narrow.txt                   |   20 +
 assets/planets/scree-wide.txt                     |   20 +
 assets/planets/the-anvil-narrow.txt               |   20 +
 assets/planets/the-anvil-wide.txt                 |   20 +
 assets/planets/the-spindle-narrow.txt             |   20 +
 assets/planets/the-spindle-wide.txt               |   20 +
 assets/planets/zenith-narrow.txt                  |   20 +
 assets/planets/zenith-wide.txt                    |   20 +
 assets/primer_text.txt                            |    6 +-
 design/brief.md                                   |   16 +
 docs/balance.md                                   |   47 +-
 docs/economy.md                                   |  125 +++-
 specs/029-tournament-rounds/closeout-main-docs.md | 1369 +++++++++++++++++++++++++++++++++++
 specs/029-tournament-rounds/plan.md               | 2189 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 specs/029-tournament-rounds/planet-art-brief.md   |  354 +++++++++
 specs/029-tournament-rounds/spec.md               |  575 +++++++++++++++
 specs/029-tournament-rounds/tasks.md              | 2378 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 src/app.rs                                        |  294 ++++++--
 src/board.rs                                      |   79 +-
 src/campaign.rs                                   |  359 +++++++++-
 src/campaign_map.rs                               |   99 ++-
 src/config.rs                                     |   15 +
 src/deck_builder.rs                               |   23 +-
 src/economy.rs                                    |  128 +++-
 src/layout.rs                                     |  339 +++++++++
 src/lib.rs                                        |    1 +
 src/overlay.rs                                    |   18 +-
 src/profile.rs                                    |  457 ++++++++++--
 src/screen.rs                                     |    2 +
 src/shop.rs                                       |   28 +-
 src/venue.rs                                      |  632 ++++++++++++++++
 src/wager.rs                                      |   37 +-
 tests/balance.rs                                  |   47 +-
 45 files changed, 9740 insertions(+), 254 deletions(-)
```

**None of the seven forbidden paths appears**, and the explicit query is empty:

```
$ git diff main...HEAD --stat=120 -- src/game.rs src/card.rs src/player.rs src/save.rs src/opponent.rs Cargo.toml Cargo.lock
(exit 0)
```

### The versions still read 1 and 1

```
$ grep -n "VERSION" src/save.rs src/profile.rs
src/save.rs:30:const SAVE_VERSION: u32 = 1;
src/save.rs:91:        version: SAVE_VERSION,
src/save.rs:171:    (saved.version == SAVE_VERSION).then(|| from_saved(saved))
src/save.rs:389:        val["version"] = serde_json::json!(SAVE_VERSION + 1);
src/profile.rs:35:const PROFILE_VERSION: u32 = 1;
src/profile.rs:99:    PROFILE_VERSION
src/profile.rs:143:            version: PROFILE_VERSION,
src/profile.rs:268:        (profile.version == PROFILE_VERSION)
src/profile.rs:654:            version: PROFILE_VERSION,
src/profile.rs:786:        assert_eq!(PROFILE_VERSION, 1, "the seed purse is no on-disk shape change");
src/profile.rs:814:        assert_eq!(PROFILE_VERSION, 1, "the seen-marks are no on-disk shape change");
src/profile.rs:1265:        assert_eq!(PROFILE_VERSION, 1, "no version bump for the additive stats field");
src/profile.rs:1400:        assert_eq!(PROFILE_VERSION, 1, "the counters and the record are no shape change");
src/profile.rs:1687:        assert_eq!(PROFILE_VERSION, 1, "no save-format change in this spec");
src/profile.rs:1718:        val["version"] = serde_json::json!(PROFILE_VERSION + 1);
src/profile.rs:1729:        val["version"] = serde_json::json!(PROFILE_VERSION + 1);
```

`SAVE_VERSION = 1`, `PROFILE_VERSION = 1`.

### The two new `#[serde(default)]` fields, among the old

```
$ grep -rn "serde(default" src/campaign.rs
src/campaign.rs:175:    #[serde(default)]
src/campaign.rs:179:    #[serde(default)]
src/campaign.rs:239:    #[serde(default)]
src/campaign.rs:241:    #[serde(default)]
src/campaign.rs:248:    #[serde(default)]
src/campaign.rs:253:    #[serde(default)]

$ git show main:src/campaign.rs | grep -n "serde(default"
152:    #[serde(default)]
162:    #[serde(default)]
164:    #[serde(default)]
169:    #[serde(default)]
```

The field under each, on the branch: 175 `pub stake: u32` (spec 021), **179
`pub settled: bool` (new)**, 239 `beaten`, 241 `in_progress`, **248
`series: Option<Series>` (new)**, 253 `run_stats`. Four on `main`, six on the
branch — the four old ones unchanged, the two new ones added. (The lines moved
down 23 since the first run: T014's two art fields on `Planet`, above.)

### `cheapest_floor` survives only inside `src/economy.rs`

```
$ grep -rn "cheapest_floor" src/ tests/ docs/
src/economy.rs:132:fn cheapest_floor(run: &CampaignRun) -> u32 {
src/economy.rs:151:        None => cheapest_floor(run),
src/economy.rs:311:    fn cheapest_floor_is_the_min_over_launchable_nodes() {
src/economy.rs:341:            assert_eq!(cheapest_floor(run), expected(run), "{label}");
src/economy.rs:343:            assert_eq!(cheapest_floor(run), 10, "{label}");
src/economy.rs:361:            assert_eq!(reserve_floor(run), cheapest_floor(run), "{label}: no lock, no change");
src/economy.rs:369:        assert_eq!(cheapest_floor(&locked), 10, "sanity: Cinder is still the cheapest node");
```

Every hit is in `src/economy.rs` (the private function, its one caller
`reserve_floor`, and its tests).

### The single-assignment invariant

```
$ grep -nE "self\.screen = Screen::(CampaignMap|Venue)" src/app.rs
782:            self.screen = Screen::Venue {
786:            self.screen = Screen::CampaignMap {

$ grep -n "fn open_campaign_home" src/app.rs
780:    fn open_campaign_home(&mut self) {
```

Exactly two lines, both inside `fn open_campaign_home(&mut self)` (declared at
`src/app.rs:780`, closing at 790).

### `Readme.md` no longer claims a launch opens a wager prompt

The readme is tracked as `Readme.md` (on darwin `README.md` resolves to the same
file; the tracked name is used here):

```
$ grep -rn "wager prompt" Readme.md
Readme.md:85:costs only the stakes already lost. Each match opens a **wager prompt** — pick
```

In context the sentence follows "played at the planet's **venue**, where you
start each match": it says each **match** opens a wager prompt, which is true,
and no longer says launching from the map does.

### The art (R9): sixteen files, sixteen loads, LF-only, credited

```
$ git ls-files assets/planets | wc -l
      16

$ grep -c 'include_str!("../assets/planets/' src/campaign.rs
16

$ cat .gitattributes
assets/planets/*.txt -text

$ grep -n "ART TOOL" assets/CREDITS.md
(exit 1)

$ LC_ALL=C grep -c $'\x1b' assets/planets/*.txt | grep -v ':0$'
(exit 1 — all sixteen files count 0 escape bytes)

$ LC_ALL=C grep -c $'\r' assets/planets/*.txt | grep -v ':0$'
(exit 1 — all sixteen files count 0 carriage returns)

$ ls -a assets/planets | grep "^\."
.
..

$ grep -nwE "48|92|20|50|94|22" src/venue.rs
(exit 1)
```

Sixteen tracked art files, sixteen `include_str!` loads; the `-text` rule is
in place; the credit names the tool (no marker left); no art file carries an
escape byte (and AC 21's test enforces the 23-character palette, which admits
none); no dotfile sits in the folder the checklist test lists; and `venue.rs`
restates no canvas or box size — they are derived from `VenueLayout`.

### No colour path, no balance value, no starter-deck line (AC 19, AC 20 after Phase 6)

```
$ git diff main...HEAD -- src tests | grep -nE "^\+.*(Color|SetForegroundColor|SetBackgroundColor|style::)"
(exit 1)

$ git diff main...HEAD --stat -- src/opponent.rs
(exit 0)

$ git diff main...HEAD -- src/economy.rs | grep -E "^-" | grep -v "^---"
-    /// (spec 025: the Outfitter's group headings use the map's words).
-/// since the start planet is always unlocked and always has a rematch.
-pub fn cheapest_floor(run: &CampaignRun) -> u32 {

$ git diff main...HEAD -- src/ | grep -nE "^[-+].*(STARTER|starter_deck|fn starter)"
(exit 1)
```

Every removed line of `economy.rs` is a doc line or the `pub` on
`cheapest_floor` (re-added without it); no constant is removed or changed;
`src/opponent.rs` is untouched; no starter-deck line changed; no added line in
`src/` or `tests/` names a colour API. Phase 6 added no crate (`Cargo.toml` and
`Cargo.lock` are in the empty forbidden-path query above) and touched no engine
or save file.

### Warning count equals `main`'s

Method: `main` was checked out into a throwaway worktree (`git worktree add
--detach`) — the branch was never switched — and both trees were built with
`cargo build --all-targets` into their **own empty** `CARGO_TARGET_DIR`, so
neither replayed a cached diagnostic and neither disturbed the other's
`target/`.

```
$ git worktree add --detach <scratch>/main-wt main
Preparing worktree (detached HEAD a5a854c)
HEAD is now at a5a854c Roadmap: per-planet venue art as its own spec
$ (cd <scratch>/main-wt && CARGO_TARGET_DIR=<scratch>/main-target cargo build --all-targets) > main-build.txt 2>&1
main exit 0
$ CARGO_TARGET_DIR=<scratch>/branch-target cargo build --all-targets > branch-build.txt 2>&1
branch exit 0
$ grep -c warning main-build.txt
0
$ grep -c warning branch-build.txt
0
$ git worktree remove <scratch>/main-wt; git worktree list
/Users/erikh/Projects/Rust/kaazap 7e5fe11 [029-tournament-rounds]
```

Both finished successfully (``Finished `dev` profile … in 15.84s`` on `main`,
``… in 16.68s`` on the branch) and neither emitted a line containing "warning".
**0 on `main`, 0 on the branch — equal.** The worktree and both target
directories were removed afterwards; `git worktree list` shows only the main
checkout, and the branch is still `029-tournament-rounds`.

---

## 6. `spec.md`'s 21 acceptance criteria, checked off with evidence

Criteria 1–20 were checked at the first close-out; the refresh re-checked the
evidence each cites against Phase 6 and changed only criterion 16's test name
(renamed by T012). Criterion 21 is new with R9.

- [x] **1. Series length.** `wins_needed_is_two_except_for_the_final_opponent`,
  `a_series_resolves_only_at_the_required_wins`, `the_final_opponent_needs_three`
  (`src/campaign.rs`, `src/profile.rs`) and
  `a_series_beats_the_opponent_only_at_the_deciding_win`. Phase 1 walkthrough:
  one win left `0/8 cleared` with Greeb un-beaten at 1–0; a 2–0 series flipped
  the map to `1/8 cleared` and unlocked Scree and Ashfall.
- [x] **2. Starting a series.** Phase 2 walkthrough: Enter on Cinder opened the
  venue at 0–0 with the balance still 50 and `in_progress: None` — nothing
  staked. Phase 3 walkthrough: the map's detail row reads
  `Cinder · Outer Rim   Best of 3` before a launch;
  `the_detail_row_names_the_series_length_only_before_a_clear`.
- [x] **3. The venue.** Phase 2 walkthrough (four rows, four actions, `b` and
  `c` each returning to the venue); first amendment re-walkthrough (the action
  row's **Card Shop** — taken from the row, not just `b` — opens a screen headed
  **Card Shop**, Esc returns to the venue); second amendment walkthrough (the
  `Credits: ◈ …` row, moving after a match settles).
  `the_venue_keys_move_confirm_and_shortcut`, `the_series_line_names_the_score_and_the_length`,
  and `the_venue_rows_breathe_only_around_the_action_row` (the credit row
  non-blank). Caveat recorded: the credit row's text is not asserted (§2a).
- [x] **4. Every match starts at the venue through today's wager prompt.**
  Phase 2 walkthrough: Play → wager over the venue → Esc → venue, balance
  untouched; re-confirmed at the amendment re-walkthrough. The prompt's floor
  and escrow are today's code path; its reserve is `reserve_floor`
  (`reserve_floor_follows_the_lock`).
- [x] **5. Between matches.** Phase 2 walkthrough: 0–1 and 1–1 each returned to
  the venue with the score moved; Phase 3 walkthrough: the non-deciding
  game-over frame shows the updated `Series 0 – 1`.
  `the_deciding_match_and_the_series_agree` (the `Continues` arm).
- [x] **6. Deciding match.** Phase 2 walkthrough: the second win landed on the
  map. Phase 3 walkthrough: the deciding loss's banner reads
  `Series lost · Lost 10 credits`. The winning banner was not seen on screen
  (six auto-played attempts lost) and is pinned by
  `the_banner_names_the_series_beside_the_settlement`.
- [x] **7. Winning a series.** Phase 1 and Phase 2 walkthroughs (`1/8 cleared`,
  `beaten: {cinder: [greeb]}`, Scree and Ashfall unlocked);
  `a_series_beats_the_opponent_only_at_the_deciding_win`,
  `resolving_a_match_twice_pays_beats_and_counts_once`,
  `resolve_match_reports_the_completion_edge_and_skips_quick_play`.
- [x] **8. Losing a series.** Phase 1 walkthrough: a lost series cleared the
  score, left Greeb un-beaten and the planet uncleared, and took only the two
  stakes; Phase 2 walkthrough: a deciding loss landed on the map.
  `a_lost_series_takes_nothing_beyond_the_stakes`.
- [x] **9. The lock.** Phase 2 walkthrough: Start Campaign → Continue went
  straight to the venue twice, the map never drawn. The single-assignment grep
  (§5) returns exactly the two lines in `open_campaign_home`; the Phase 2 review
  widened it three ways and walked every door, confirming `launch_from_map` is
  unreachable while a series runs.
- [x] **10. Broke while locked.** `reserve_floor_follows_the_lock` and
  `broke_and_affordable_follow_the_locked_floor` (20 credits against Rix's 50
  is broke; against Cinder's 10 it is not — corrected at the sweep, which found
  this line crediting a "locked half" of `is_broke_reads_…` that does not
  exist); both seams are the one `enter_campaign` check. Phase 2
  walkthrough: the Card Shop read `spendable ◈ 40` (50 minus Greeb's ante of
  10). The run-over modal and reset are untouched code. The sweep's B1 closed a
  third door the floor rises at — the map's series launch now refuses an
  uncovered opponent (driven: 15 credits, Enter on Ashfall → "Can't cover the
  20-credit ante", no series written) — and its N1 made the wager warning
  predict the post-loss floor (driven: locked on Rix at 0–1 with 80 credits,
  no warning at a 50 stake; at 0–0, "Lose this and the run is over.").
- [x] **11. In-match score.** Phase 3 walkthrough: `Series 0 – 0` and
  `Series 0 – 1` on the status band beside the turn prompt at **89** and
  **139**; a rematch shows no line. `the_board_shows_a_score_only_for_a_series_match`,
  `the_series_score_fits_beside_the_longest_turn_prompt`,
  `the_compact_board_carries_the_series_score_clear_of_the_prompt`. Two recorded
  edges, neither judged a failure by the reviews: the deciding match's game-over
  frame (plan §Open questions 1, asked and unanswered — present on every frame
  the player can act in, per sign-off), and a pre-029 match left in flight,
  which shows no score while played (sweep S18; the Phase 3 review ruled it not
  an AC 11 failure).
- [x] **12. Rematches.** Phase 2 walkthrough: Enter on cleared Cinder opened the
  wager directly, no venue; Phase 3 walkthrough: a rematch shows no score and
  its banner is the plain `Lost 10 credits`.
  `enter_on_a_cleared_planet_launches_a_rematch`.
- [x] **13. Save and resume.** Phase 2 walkthrough: quit at the venue and
  return → same score; a match killed mid-play resumed at the same score with
  the same hand, and its finish decided the series and routed to the map.
  `a_campaign_run_round_trips_its_series` (a pre-029 run loads with no series
  and Greeb still beaten), `a_pre_029_node_is_unsettled_and_unstaked`,
  `a_match_in_flight_with_no_series_starts_one`.
- [x] **14. Reset paths.** `every_reset_clears_the_lock` — New Campaign and
  Reset Everything asserted; the run-over reset is the same call as Reset
  Everything, stated in the test (coverage recorded in §2a).
- [x] **15. Records and statistics unchanged.** No stats or records file is in
  the diff (§5); `resolve_match_moves_the_run_credit_counters_and_nothing_else_does`
  and `resolve_match_reports_the_completion_edge_and_skips_quick_play` pass; the
  `Some(Won(0))` → `None` change moves no counter (plan §Design tension 3).
- [x] **16. Both layouts.** `the_venue_bands_stack_around_the_art` (renamed
  from `…_and_the_art_takes_the_rest` by T012, since under R9 the art no longer
  takes the rest), `the_art_region_dominates_at_both_widths` (art 1100 cells
  at 89, 2068 at 139, portrait 330 — largest at both, strictly larger at 139),
  `the_venue_text_fits_the_minimum_terminal` (right *and* left edges; since
  T012 at every size from the minimum, not only the two fit sizes). After R9
  the region holds the planet's art rather than the placeholder — see 21. The
  second amendment review counted the rendered borders (50×22 and 94×22 against
  a 22×15 portrait); walkthroughs at both widths after each amendment and after
  R7 (three clear columns between art and portrait at each). Caveat recorded:
  "largest" is asserted against the portrait only (§2a).
- [x] **17. Density.** `the_venue_rows_breathe_only_around_the_action_row` and
  `the_action_row_keeps_its_width_as_the_cursor_moves`; the rendered frames
  blank only around the action row at both widths; the modals the venue can
  raise pad evenly through `OverlayLayout` (Phase 2 review).
- [x] **18. What the player is told.** Phase 4 walkthrough: the primer reads
  "Each opponent is Best of 3: win 2 matches of 3. / The last is Best of 5: win
  3 of 5. A series, / once started, is played out."; How to Play (after R8)
  reads "Each opponent is Best of 3 matches, the last / Best of 5. A started
  series is played out." Both fit 89×31. `onboarding_texts_are_the_spec_text_and_fit`,
  `help_texts_fit_the_minimum_terminal_unclamped`,
  `help_texts_name_the_new_keys_and_nothing_old`.
- [x] **19. Balance measured, not changed.** T010: the simulator re-run at
  N = 10 000 (release), still `targets 8/8, coupling 1/1, bounds 5/5`, no lever
  moved; `docs/balance.md` *Series rates* records Bo3 for nine opponents and Bo5
  for `sovereign` across the five decks; `series_rate_matches_the_closed_form`
  passes. §5: `src/opponent.rs` untouched, no constant in `src/economy.rs`
  changed; the starter deck (`STARTER_SIDE_DECK`, `STARTER_SPARES` in
  `src/profile.rs`) has no changed line in the diff
  (`git diff main...HEAD -- src/ | grep -nE "^[-+].*(STARTER|starter_deck|fn starter)"`
  is empty).
- [x] **20. No forbidden change.** §5: `game.rs`, `card.rs`, `player.rs`,
  `save.rs` absent from the diff; `SAVE_VERSION` and `PROFILE_VERSION` both 1;
  `Cargo.toml` and `Cargo.lock` untouched (no new crate); no colour call added.
- [x] **21. The planet's art at the venue** (ruling R9). **Validation, items
  1–6**: T013 validated the delivery by shell against the brief's checklist,
  first time — exactly the sixteen expected names; 20 lines each with one
  trailing newline; every line exactly 48 or 92 characters (counted as
  characters, not bytes); only the 23 whitelisted codepoints, valid UTF-8, LF
  only (the brief's own one-command check, every file `lf-ok glyphs-ok`); eight
  distinct narrow grids and eight distinct wide (tier log, T013). The person's
  redraw of Cinder and Scree (`bc2598c`) was validated the same way (items 2–5
  by shell on the four files) and by the test below. **As a test**, with the
  canvas derived from the layout: `every_planets_art_passes_the_briefs_checklist`
  reads the expected sizes from `VenueLayout::new(c).art` over
  `Config::fit_sizes()` (and `grep -nwE "48|92|20|50|94|22" src/venue.rs` is
  empty, §5), with T014's two mutation checks failing it as they should (an
  extra space on width, a CRLF file on the `\r` assertion). **In place of the
  placeholder, in a box sized exactly to the drawing, at every size from
  89×31, nothing clipped, no blank space inside the frame**:
  `every_planets_art_fills_its_box_at_every_size` (all 660 sizes),
  `the_venue_draws_the_planets_art_inside_its_box` (every planet, both fit
  sizes, the drawn frame's interior equals the drawing),
  `a_planet_without_art_shows_its_name` (the fallback), and T012's
  `the_art_box_is_the_drawing_plus_its_border_at_every_size` — all five green
  in §5 (`5 passed; 0 failed`), and in all three full runs. **Seen**: the Phase 6
  walkthrough (tier log, *Phase 6 walkthrough*) rendered all eight planets'
  venues at 89×31 and 139×31 — "every one shows its own drawing filling its
  box, the portrait beside it, nothing clipped, no planet name in the box" —
  plus Cinder at 120×31 (the narrow box, spare columns either side of the
  group) and 139×40 (the whole venue centred vertically), and the sixteen
  fit-size frames were sent to the person for item 7. The Phase 6 review
  confirmed AC 21 end to end ("all 660 sizes fill exactly, and the drawn-frame
  test catches a fallback to one planet"). **The go/no-go, checklist item 7**:
  at the Phase 6 pause the person had Cinder's and Scree's art redrawn "with
  right angles only", committed it (`bc2598c`), and then **said to continue** —
  recorded in the tier log (*Phase 6 pause — the person's answers*) as "read
  as the item-7 go on the art". That is the product owner's look at both fit
  sizes, given by continuing after acting on what they saw rather than in so
  many words.
