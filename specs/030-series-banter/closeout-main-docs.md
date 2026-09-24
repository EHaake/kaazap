# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 030)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files. Per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T009, ready to paste on `main` after the merge
(the same close-out shape specs 020–029 used). The spec branch applies none of
it.

Spec 030 amends no rule that `CLAUDE.md` states (see §3). Nothing to apply
there.

Line numbers below are **`main`'s** at the time of drafting (2026-09-23,
`main` at `b86b95d`, *Roadmap: the opponent's line on the compact board*; the
branch at `f358789`, merge base `bd50036`). Each edit quotes its anchor text
verbatim, and that text is what to match on. Every anchor below was checked
with `grep -cF` against `git show main:<file>` at `b86b95d`, and each count is
1 (`ROADMAP.md` 1037 lines, `DECISIONS.md` 2388).

Applying: never chain a file edit, a branch switch and a commit in one shell
command (spec 025's miss). Switch to `main`, edit, check the anchors again,
then commit.

---

## 1. `ROADMAP.md`: five edits; the other hits judged and left

`grep -n -i "banter" ROADMAP.md` on `main` returns 17 hits. Judged one by one:

- **215, 219, 220, 224, 228, 245** are the Shipped entries for specs 016 and
  017 (portraits, *Opponent banter*). **Left**: they record what those specs
  shipped, and nothing in them is false now.
- **429–430** and **933–936** are spec 026's compact layout, Shipped and
  backlog: "no panel, no portrait, no banter". **Left**: still true, and
  ruling 7A kept it true.
- **489** (spec 027: "the same snapshot pattern banter and audio use").
  **Left**: still true.
- **612–614**, the *What's left for v1* paragraph. **Edited (1b)**: spec 030
  is one of its two items, so music is the only one left.
- **714**, the **F · Tournament rounds** shipped bullet ("per-planet music and
  series-aware banter (below) are its next concrete pieces"). **Left**: it
  records spec 029's view when it shipped. The banter bullet it points at is
  marked shipped (1c), so the pointer still lands somewhere true.
- **721, 724, 728**, the *Immersion & personality* intro and the shipped
  **Opponent banter** bullet. **Left.**
- **748–757**, the **Series-aware banter** bullet, which this spec ships.
  **Edited (1c).**
- **759–765**, the **opponent's line on the compact board** bullet. The
  person added it on `main` during this spec's planning, and it already
  carries the burble ("let it be spoken there like everywhere else").
  **Edited (1d)**: one sentence naming ruling 7A. No second bullet is needed.

A wider grep for `burble`, `word-by-word`, `skip key`, `taunt` and `run-over`
found only the compact-board bullet (762) and spec 024's run-over items
(334, 365, 854–882). Those record other specs and are left as written.

### 1a. New entry at the end of the `## Shipped` list

Insert right after the **Tournament rounds (spec 029)** entry, which ends with
this line (currently `ROADMAP.md` line 607, followed by a blank line and
`## Backlog`):

```markdown
  `Readme.md`'s campaign paragraph names the series, the venue and the lock.
```

Insert after it:

```markdown
- **Series-aware banter, spoken word by word** (spec 030) — the opponent's
  lines now follow the series, and every line is spoken. A match started
  mid-series opens on a line for where the series stands, read from the
  opponent's side: **Leading**, **Trailing**, **Decider** (both one win away),
  and **All square** (level at 1–1 in The Sovereign's best of five). At 0–0
  the opener is today's greeting. The match that decides a series ends on a
  **series won** or **series lost** line. Other match ends keep today's lines.
  Quick Play and cleared-planet rematches pick exactly as before. The opponent
  also **speaks at the venue** on arrival: when a series starts from the map,
  after a match that didn't decide it, and when the campaign is entered or the
  game launched mid-series. A trip to the Card Shop or the collection and back
  is not an arrival, so the line shown is still there, whole and silent. An
  arrival under the run-over notice says nothing. The venue's presence panel
  grew two rows to hold the line, on the same row the board uses. The venue
  line and the match start draw from one pool, so the match never opens on the
  line the venue just said. The work added **146 lines**: 13 for each of the
  other nine roster voices and the fallback, and 16 for The Sovereign. The
  person read them all at the Phase 2 pause and kept them as written. Every
  line is **spoken**: its words appear one at a time, in place, about
  **0.2 s** apart.
  Each word plays one **burble**, a soft, voice-like murmur synthesized by
  `scripts/gen_sfx.py` like the other sounds and about as loud as a card being
  played. A line answering a round, bust or match event, and the venue line,
  waits a **400 ms beat** before its first word, so the event's sound (or the
  menu click) lands first. Settings gained a **Voices** slider between Sound FX
  and Animations. The burble follows it and nothing else does, `m` mutes it,
  and it defaults to 50%, set by the person's ear. With Animations off a line
  appears whole with one burble. The compact board (89–138 columns) draws no
  line, so nothing is spoken there, and a resumed match stays blank. No key
  waits for a line. The deciding match's game-over frame now keeps the **final
  series score** (e.g. `Series 2 – 1`). The map afterwards is unchanged. No
  engine, AI, save-format, economy, balance-data or dependency change:
  `game.rs`, `card.rs`, `player.rs`, `campaign.rs`, `profile.rs`, `save.rs`,
  `economy.rs`, `opponent.rs`, `Cargo.toml` and `Cargo.lock` are untouched, and
  `PROFILE_VERSION` / `SAVE_VERSION` stay 1. The settings file gained one
  serde-defaulted field (`voices_volume`), so an older file loads with Voices
  at the default and every other value kept. The person amended the spec
  twice, after listening at Phase 1 (9A–11A) and after walking the venue at
  Phase 3 (12A, 13A). They approved the burble by ear and attested every
  paused phase. Driver walkthroughs against a scratch `KAAZAP_DATA_DIR`
  covered the reveal, the beat, Settings, the series lines and the venue at
  89×31 and 139×31. `design/brief.md`'s *Motion* section calls a spoken line a
  one-shot transition. `Readme.md` names the voice volume and the word-by-word
  lines. `assets/CREDITS.md` covers the burble.
```

### 1b. Rewrite the *What's left for v1* paragraph's first sentences (currently lines 611–615)

Replace

```markdown
complete. Two things remain before it counts as done: **series-aware
banter** (below, taken as spec 030 with a word-by-word speaking animation)
and **music** — *Original cantina-vibe music* and/or *Per-planet music*
(below). Everything else here is optional polish or post-v1. The
```

with

```markdown
complete. One thing remains before it counts as done: **music** —
*Original cantina-vibe music* and/or *Per-planet music* (below).
**Series-aware banter**, the other item named here on 2026-09-23, shipped as
spec 030. Everything else here is optional polish or post-v1. The
```

### 1c. Mark the **Series-aware banter** backlog bullet shipped (currently lines 748–757)

Replace the whole bullet, from

```markdown
- **Series-aware banter** (deferred by spec 029, 2026-09-20). Since spec 029 a
```

through its last line

```markdown
  017, the real cost is the writing — ten voices and the fallback.
```

with:

```markdown
- **Series-aware banter** — ✅ **Shipped (spec 030** — see Shipped above and
  `DECISIONS.md`). Deferred by spec 029 on 2026-09-20. The shape this entry
  guessed held: the series state is read from what the board already shows
  (`match_series`, pulled out of `board_series_line`), and match start gained
  Leading, Trailing, Decider and (for The Sovereign) All square pools. Match
  end also gained a series won / series lost pair. As predicted, the real
  cost was the writing: 146 lines across the eleven voices. The spec also
  made every line **spoken**, word by word with a burble, gave the burble a
  **Voices** volume, and had the opponent **speak at the venue**. Deferred
  from it, each below: per-opponent burble voices, word-by-word text
  elsewhere, a skip key, and a taunt when the run ends. The compact board's
  silence is the bullet just after this one.
```

### 1d. Amend the **opponent's line on the compact board** bullet (currently lines 759–765)

After its last line

```markdown
  line fits on the 89-column board).
```

append, continuing the same bullet:

```markdown
  **Spec 030 shipped it silent, by ruling 7A**: "if the banter isn't
  visible, it makes no sense to include the speech burble." One predicate
  decides both (`App::line_visible`, which reads `BoardView::is_wide()`), so
  once the line is drawn on the compact board, making `line_visible` true
  there brings the burble back with no audio change.
```

### 1e. Four new bullets: spec 030's deferred follow-ups, in *Immersion & personality*

Insert right after the bullet amended in 1d, so after the text 1d appends,
still before the blank line and `### Stakes, loss condition & difficulty
balance`:

```markdown
- **Per-opponent burble voices** (deferred by spec 030, 2026-09-23). Every
  opponent speaks with one burble today, pitched per word by the five-entry
  `audio::BURBLE_PITCHES`. A voice per opponent (Greeb higher and quicker, The
  Sovereign low and slow) could be a pitch multiplier per voice fed into
  `burble_cue`, or its own `BURBLE`-style parameter block in
  `scripts/gen_sfx.py` and its own WAV. The first costs one table; the second
  costs one generated file per voice. It must still pass
  `the_burble_is_as_loud_as_the_other_sounds` and the `BURBLE_GAP_MS` bound
  (`a_burble_ends_before_the_next_can_start`), because a slower voice is a
  longer clip.
- **Word-by-word text elsewhere** (deferred by spec 030). Only opponent lines
  are spoken. Notices, popups, How to Play, the primer and the map print
  whole. `banter::revealed` and `Speech` are pure and would serve another
  caller, but each surface would need its own ruling on whether it should
  wait, and on its sound.
- **A key to skip the reveal** (deferred by spec 030). Lines are at most five
  words and finish within a second, and no key waits for them, so the spec
  kept every key's effect unchanged. If longer lines ever arrive, a skip is
  `Speech::settle` on a key, but that key must also keep its current meaning.
- **The opponent taunts the end of the run** (ruling 13A's preferred option,
  deferred as needing design, 2026-09-23). A venue arrival that leaves the
  player broke says **no** line today. The run-over notice covers the
  presence panel at 89 columns and clips its first character at 139, and the
  run resets behind it. The person would prefer a taunt ("the run is over"),
  but not if it needs more design, and it does: every voice needs a new pool,
  and the line needs a place to be seen, either on the notice itself or with
  the notice moved off the panel. `arrive_at_campaign`'s `is_broke()` early
  return is where it would start.
```

**Leave alone** every other roadmap bullet, including **Original cantina-vibe
music** (976) and **Per-planet music** (1002–1024). Both are still accurate,
and neither mentions this spec.

---

## 2. `DECISIONS.md`: one edit

`grep -n -i "banter\|burble\|voices" DECISIONS.md` on `main` returns spec
017's section (465–495), spec 016's panel note (419), spec 026's Q1 B (1318,
1320, 1383, 1399), spec 027's notes (1428, 1465, 1517) and the spec 017
observer notes (528–544). **All left as written.** Line 474 names the field
`banter` ("`banter` / `banter_last` / `prev_banter`"), which spec 030 renamed
`speech`. It is the record of spec 017, and the new section says the field
was renamed, which is where a reader following the name will land next.

### 2a. Append a new section at the end of the file

Append at the **end of the file**. `DECISIONS.md`'s last two lines are
currently the tail of the "## Tournament rounds (spec 029)" section:

```
`.gitattributes` line and a `CREDITS.md` section — no crate, and no engine or
save file.
```

(Checked with `grep -cxF`: each line occurs once, and together they end the
file, which ends with a newline.) Append after them:

```markdown

## Series-aware banter, spoken word by word (spec 030)

Since spec 029 a campaign opponent is played as a series, but its lines
didn't know it: every match opened on a greeting, and a match that took the
series ended like any other. Spec 030 gives each voice six series pools and
has the opponent speak at the venue. It also makes every line **spoken**,
word by word, with a synthesized burble per word, and gives the burble a
**Voices** volume. And it holds the final series score on the deciding
match's game-over frame. It adds two types (`banter::Speech`,
`banter::SeriesState`), one sound (`assets/sfx/burble.wav`), one settings
field (`voices_volume`), two constants (`WORD_STEP_MS` 200, `EVENT_BEAT_MS`
400), and 146 lines of dialogue. No engine, AI, save-format, economy,
balance-data or dependency change. The person ruled 1B–6A in the spec
conversation (2026-09-23) and 7A and 8A at planning. They amended the spec at
the Phase 1 pause (9A–11A), after listening, and at the Phase 3 pause (12A,
13A), after walking the venue.

### The rulings (the person, 2026-09-23)

- **1B — a match started mid-series has lines for Leading, Trailing and
  Decider**, plus All square for the best of 5 at 1–1, rather than folding
  the decider into "all square". Decider means both sides are one win from
  the series (1–1 in a best of 3, 2–2 in a best of 5), so only The Sovereign
  reaches All square, and only its voice carries those lines. Every other
  voice's pool is empty, and no test pins that emptiness (unreachable, and
  harmless).
- **2A — the match that decides a series ends on series won / series lost
  lines.** Other match ends keep today's lines.
- **3B — the opponent speaks one line at the venue on arrival**, reacting to
  the series score.
- **4A — about 0.2 s per word, the first word at once** (`WORD_STEP_MS` =
  200, bounds pinned at 150–250). Every line has at most five words, so every
  line finishes within a second of its first word. That is pinned for every
  line of every pool.
- **5B — a soft burble per word.** The person's words were "soft and not
  louder than the music … sort of a 'soft burble' if possible." 9A later
  dropped the "not louder than the music" half.
- **6A — the deciding match's game-over frame holds the final series
  score**, which "should not follow the player to the galaxy screen." This
  closes spec 029's *Asked, and not answered* item (plan §Open questions 1
  there) in the direction that section predicted: a one-shot `App` field.
- **7A (asked at planning) — the compact board draws no line, so no burble
  plays there.** "If the banter isn't visible, it makes no sense to include
  the speech burble." The line on the compact board is a roadmap item.
- **8A (asked at planning) — a resumed match stays blank**, as spec 017
  shipped it, and no line is spoken on resume.

### Defaults set in the spec conversation, not separately ruled

Words appear in place. A new line interrupts the old one. Animations Off
shows the line whole, with one burble. A resumed match's line is whole and
silent (superseded by 8A: a resumed match shows no line). Quick Play keeps
today's lines but is spoken. The venue and match-start lines share a pool, with
no repeat across the two. The Card Shop / collection round trip is not an
arrival.

### The Phase 1 amendment (the person, 2026-09-23, after the first listen)

- **9A — the burble is as loud as the other sound effects**, and "never
  louder than the music" is dropped. The first build met that rule to the
  letter: its peak (0.08) sat under a ceiling of the music's RMS × default
  music volume / default SFX volume (0.1025). The person found it "really,
  really quiet" and had to turn the music off to hear it. Measured by peak,
  a soft-enveloped voice with tremolo is far quieter than a square-wave blip
  with the same peak, so the rule had been measuring the wrong thing.
- **10A — a Voices volume in Settings**, which the burble follows instead of
  Sound FX. `m` mutes it like every other sound.
- **11A — a line answering an event waits a beat.** A line said on a round,
  bust, tie or match/series event is chosen when it is today, but its first
  word waits `EVENT_BEAT_MS` (400 ms), so the event's own sound plays first.
  Popups, sounds, phases and keys keep their timing. The value is the top of
  the ruled 0.3–0.4 s. The opponent's bust sound at `OPPONENT_PITCH` (0.92)
  runs 380 ms, which rules out 350.
- **The Voices default is 50%** (T002c, at the re-listen). The person's
  words: "I have the volume set to 50% where it sounds well mixed with the
  rest of the sounds, so maybe we should make that the default." The plan had
  made it equal to Sound FX's 80%, so that the default settings compared like
  with like. The ear overruled the arithmetic. At 50% the loudness test still
  passes unchanged, but only just (below).
- **The approval (AC 9).** At the re-listen: "Murmur is good now and is
  correctly reflecting the settings." That was the only tweak asked for
  (T002c). No `BURBLE` number moved after T002b.

### The Phase 3 rulings (the person, 2026-09-23, after walking the venue)

- **12A — the venue line waits the same beat.** "Add a slight delay to the
  line so that the sounds don't overlap." The acknowledgement's menu click
  and the venue line's first burble had landed on the same tick. The venue
  arrival now passes `EVENT_BEAT_MS` too, reusing the one beat constant, so
  only the match-start and rematch greetings start at once. `spec.md`'s AC 8
  was amended to say so. The Phase 3 re-review caught that the amendment had
  first been missing, and the orchestrator transcribed it.
- **13A — an arrival under the run-over notice says no line.** The notice
  covers the presence panel at 89 columns and clips its first character at
  139, yet the line was spoken with its burbles and the run then reset.
  `arrive_at_campaign` now returns early when `profile.is_broke()` (the check
  `campaign_entry_modal` uses for the notice) and sets `speech = None`. The
  `None` is required, because otherwise the match's closing line, possibly
  still in its beat, keeps burbling at the venue. The person would prefer
  the opponent to taunt that the run is over, but not if it needs more
  design, and it does. It is on `ROADMAP.md`.

### The burble as shipped

`scripts/gen_sfx.py`'s `BURBLE` block, as it ships:

    "length_s": 0.12, "pitch_hz": 150, "glide": -0.12,
    "vibrato_hz": 18, "vibrato_depth": 0.04,
    "tremolo_hz": 24, "tremolo_depth": 0.35,
    "formants": ((500, 750, 90), (1100, 1400, 140)),
    "harmonics": 18, "attack_s": 0.015, "release_s": 0.06,
    "peak": 0.79

The synthesis is additive: `harmonics` harmonics of a gliding, vibrato'd
150 Hz fundamental, each weighted `1/k` times the resonance of two gliding
formants (a "wo→a" vowel), with a raised-cosine envelope and a tremolo. It
uses no `random`, so `bust`'s seeded noise is undisturbed. Per-word pitch is
`audio::BURBLE_PITCHES` = `[1.0, 0.94, 1.05, 0.97, 1.02]`, and `burble_cue`
wraps past its end. `python3 scripts/gen_sfx.py burble` regenerates that one
file and leaves the other thirteen byte-identical.

**`peak` went 0.08 → 0.79** (T002b), chosen so the burble's whole-clip RMS
(0.1668) lands at the mean RMS of the four board move sounds (0.1660). Those
are `CardDraw` 0.1629, `CardPlay` 0.2137, `Flip` 0.1882 and `Stand` 0.0992.
At default volumes the band is 0.0794–0.1710 and the music floor is 0.0820
(music RMS 0.1641 over its first 60 s × 0.5). At Voices 50% the burble reads
**0.0834**: inside the band, and 0.0014 over the music floor. That margin is
thin. A later default below about 49% fails the floor, and so does a quieter
burble. Either way the change is a product question (plan §Open questions 3),
not a looser test. A peak of 0.79 is near full scale at Voices 100%, which is
for the ear to judge. The test decodes a 60 s MP3 window and takes about 4 s
in a debug build.

### The design calls (plan, signed off 2026-09-23; amended the same day)

- **One `Speech` replaces `banter`.** The shown line and how much of it has
  been said live in one `Option<Speech>`, the renamed spec 017 field
  (`App::banter` → `App::speech`; `banter_last` unchanged). Replacing the line
  replaces its clock, and clearing it drops the clock, so interruption (AC 10)
  is a property of the data, not something each call site remembers.
- **Reveal by blanking, not slicing.** `revealed(line, n)` returns the line
  at its full length with each unsaid word's characters turned to spaces. Any
  centring drawer then puts each said word at its finished column (AC 8) with
  no new geometry. Passing an offset and a prefix would have duplicated
  `draw_text_in`'s centring in a second place.
- **Burbles come only from advancing the one `Speech`, at most one per
  step, and never two within `BURBLE_GAP_MS` (150 ms).** Nothing is queued in
  the audio thread, so a replaced or cleared line cannot burble again. Every
  burble goes through `App::burble`, the only caller of `burble_cue` (one
  call), which plays only if `audio::burble_clear` holds. Two burbles at once
  play louder than one, and after 9A that pushed the burble past the other
  effects and garbled it. Two bounds are pinned: the burble at its slowest
  pitch ends within the gap, and the gap is at most three loop ticks. **The
  gap drops a burble in exactly two cases**, both to keep the sound soft:
  (1) a new line whose first word arrives within 150 ms of the old line's
  last burble shows that word silently, which since the beat (11A, 12A) can
  happen only to a greeting; and (2) a stalled frame that crosses two word
  boundaries shows both words and owes one burble. A third caller arrived
  with 10A: the Voices row's preview. The gap keeps a held ←/→ from stacking
  previews. `self.burble(` has exactly three callers (`say`,
  `advance_speech`, the Settings arm), and the Verify gates counted them in
  every task after T002a.
- **The Animations setting is read at say-time.** `Speech::new(line,
  animated)` decides how many burbles the line is owed, which is settled when
  it is chosen. The setting can only change from the Settings overlay over
  the start menu, where no line is on screen, so reading it at say-time is
  never stale.
- **Arrival is the caller's word; returns settle in `tick`.**
  `open_campaign_home` keeps its single job and stays the only place either
  campaign screen is assigned (spec 029's grep still returns exactly two
  lines). The new `arrive_at_campaign` calls it and then says the venue's
  line. Its callers are `enter_campaign` (every menu entry and the game-over
  acknowledgement) and `launch_from_map`. The Card Shop's and the deck
  builder's Back arms keep calling `open_campaign_home`, so a return says
  nothing new. On any screen other than the board or the venue, `tick`
  **settles** the current `Speech` (every word shown, silently), the way it
  resets `BoardMotion`, so coming back draws the whole line. Two alternatives
  were rejected: a flag on `VenueState` (rebuilt on every return) and an
  `arrival: bool` on `open_campaign_home` (spec 015's bug was a caller
  forgetting what a return is).
- **The series state lives in `banter.rs` and is read through
  `match_series`.** `SeriesState` is a from-the-opponent's-side reading used
  only to pick lines, so `campaign.rs` stays untouched. At match start the
  series comes from `match_series(in_progress, series)`, which is
  `board_series_line`'s node-matches-series rule pulled out into its own
  function. Quick Play, a rematch and a match left in flight with no series
  therefore all read as "no series", exactly as the board does. At the venue
  it comes from the locked series, because the pointer is already cleared by
  then.
- **`banter_last` is fed to the match start only inside a series**
  (`let last = state.and(self.banter_last)`; sign-off B1). The first draft
  fed it always, which would have changed Quick Play's pick, against AC 4.
  Now a series match start avoids the venue's line (AC 6), and every
  no-series match passes `None` exactly as before.
- **`final_series` follows the motion idiom rather than `victory_due`'s
  take-on-entry.** It is set in `tick`'s resolution block from the match's
  series cloned *before* `resolve_match` (settlement clears the live series
  one tick before the game-over frame and the closing line need it). It is
  cleared in `tick`'s `_` screen arm beside the `BoardMotion` reset, so it is
  gone on every exit from the match, not just the one that exists today. The
  map never reads it. **`decided_series` repeats the tally rule under a
  test** (`decided_series_agrees_with_the_series_rule`, which drives
  `record_series_match` through both series lengths) rather than changing
  `SeriesOutcome` to carry the score. That change would have touched
  `campaign.rs`, `profile.rs` and every `MapBanner` match.
- **Loudness as a number.** As signed off it was a ceiling: peak ≤ music RMS
  × default music / default SFX. 9A dropped the sentence it tested, and the
  first listen showed that peak was the wrong measure anyway. It was
  **replaced, not deleted**. `the_burble_is_as_loud_as_the_other_sounds` took
  `the_burble_is_softer_than_the_music`'s place in the same diff. It checks a
  **band** (the burble's whole-clip RMS × default Voices lies between the
  quietest and loudest of `CardDraw`, `CardPlay`, `Flip`, `Stand` × default
  Sound FX) and a **floor** (at least the music's first-60-s RMS × default
  Music), plus a peak under 1.0. Whole-clip RMS is the simplest measure the
  repo can take without a crate. It does not weight frequencies the way the
  ear does, which is why the person's approval is the other half of AC 9.
- **The event beat lives inside `Speech`, not at the `App`.** `Speech` gains
  `wait` and `after(wait)`. While waiting it shows nothing and owes nothing,
  and the step that ends the wait owes the first burble. A pending line held
  beside `speech` was rejected: it would be a second field that
  interruption, clearing and settling must each remember, which is the split
  the first design call refused. With the wait inside, every existing rule
  covers the beat as it stands. **The value is 400 ms.** It outlasts every
  round, tie and bust sound (pinned by
  `the_event_beat_outlasts_the_round_sounds`: `RoundWin` 210 ms, `RoundLoss`
  270, `RoundTie` 210, `Bust` at 0.92 pitch 380). It does **not** outlast
  the match-end jingles (`GameWin` 450 ms, `GameLoss` 520), so a match or
  series line's first word lands on their last note. Waiting out the whole
  jingle, about 0.55 s, would be outside the ruled range, and the person
  heard it at the re-listen and asked for no change.
- **Voices: one field and one routing function.** `Settings.voices_volume`
  is `#[serde(default)]`, so an older file loads with Voices at its default
  and keeps every other value. Without the serde default,
  `from_json_or_default` would have reset the whole file. One pure
  `audio::sfx_level(muted, sfx, settings)` gives each effect's volume or
  `None`: the burble on Voices, every other effect on Sound FX through the
  unchanged `should_play_sfx`. `AudioState::play` calls it. The default was
  set equal to Sound FX's (80%) so that the defaults compared like with like,
  and **T002c moved it to 50%** by ear (above). The Voices row **previews one
  burble** at the new level instead of the `MenuMove` tick, because a tick at
  the Sound FX volume tells the player nothing about Voices.
- **`voices_volume` is declared last in the struct** (T002a's decision
  review). serde's derived `Deserialize` also accepts a struct as a
  positional JSON array. `settings_malformed_or_empty_json_falls_back_to_default`
  relies on `[1,2,3]` failing when its third element lands on the
  `animations` bool, and declaring the volume third, as the task line said,
  let that array parse. Three options were weighed: declare it last (the test
  is untouched, and the only visible effect is key order in `settings.json`);
  edit the test (a test changed to pass); or add a custom deserializer (code
  nobody asked for). The review chose declaring it last. **Any later
  `Settings` field should also be declared last.** The field's doc says why.
- **The venue panel grew two rows** (`VENUE_PANEL_H` 15 → 17: border, name,
  portrait, a gap row, the line row, border). The venue line sits on the
  panel's interior row 14, exactly where the board draws it, through one
  shared drawer, `portrait::draw_banter_line`. The rejected alternative was a
  line floating under the 15-row panel, which reads as a caption *under* the
  opponent rather than the opponent speaking. **The class of layout
  assertions that moved**, sanctioned as a class in T007: six pinned venue
  portrait `Rect`s in `layout.rs`, each `y1` + 2; one area comment (330 →
  374); and two `VenueState::draw` calls gaining their `line` argument. That
  is a stated deviation from AC 11's parenthetical "existing tests
  untouched", which is about phases, popups and timings, none of which these
  assertions pin.
- **Tests never build an `App`.** `App::new` reads the real settings and
  profile, and `Profile::save()` has no `cfg(test)` guard. Every decision
  sits behind a pure function and is tested there. The `App` wiring (`say`,
  `advance_speech`, `arrive_at_campaign`, the settle-on-leave) is checked by
  the driven walkthroughs and the counted-call greps.

### Coverage, stated honestly

- **An opponent bust's line never shows now.** It is replaced one tick later
  by the round-outcome line (`game.rs` ~282–285 against ~399–400). Before the
  beat it showed for one tick, about 50 ms; with the beat it never shows.
  Nobody could see the difference. It is recorded so it isn't taken for a
  bug.
- **The beat lands at 400 ms plus up to one loop tick** (≈ 0.40–0.45 s), at
  the top of AC 8's range. Round-end timing was driven. Bust and match-end
  timing were not, since they take the same `update_banter` branch. The
  person heard the match end at the re-listen.
- **The silent cases rest on construction and the counted-call greps**
  (Animations Off with one burble, 89 columns silent, the resumed match, a
  replaced or cleared line, a return from the Card Shop). A driver cannot
  hear. The person's ear was the check.
- **A line still revealing can burble under something that covers it**, for
  under a second: under the `?` help overlay on the board, and under the
  wager modal at 89 columns, which covers the venue's line row.
- **`the_burble_follows_voices_and_nothing_else_does` lists the sounds by
  hand**, so a later `Sfx` variant is not caught automatically.
- **AC 4 rests on the board's own rule.** `match_series` returns a series
  only when the in-flight node is the one the locked series is played
  against. Three callers rely on that during Quick Play: `series_state_now`,
  `tick`'s `before` and the draw arm.
- **`banter_last` isn't persisted**, so the first venue line after a relaunch
  can repeat the one said before quitting (1 in 3). That is no regression:
  spec 017's no-repeat rule was always within a session.
- **Whether the menu-select sound ends inside 400 ms**, so that the venue's
  first burble truly follows it (12A), is for the ear. It is not measured.
  `the_event_beat_outlasts_the_round_sounds` could be extended to it.
- **`the_venue_rows_breathe_only_around_the_action_row` still draws with no
  line.** The geometry holds either way. Drawing a line there would be
  optional hardening.

### Process notes

- **A driver session touched the real data directory once.** In the Phase 1
  walkthrough, an Animations-Off run hit a zsh `nomatch` error that skipped
  the chained `export KAAZAP_DATA_DIR=…`. The run re-saved the person's
  profile, wrote a Quick Play `savegame.json` and set Animations Off in their
  `settings.json`. It was reported to the person and nothing further was
  touched. The lesson: set the data directory on the command itself
  (`KAAZAP_DATA_DIR=… cmd`), not in an earlier link of a chain that can fail.
- **Verify gates that could not be read as written**: T003's
  `grep -nE '^\s*banter:'` also matches the `banter::{` import, which matched
  before the task too. The amendment sign-off's B1 was a bare
  `EVENT_BEAT_MS` grep that would also match the import and `say`'s doc
  (fixed to `from_millis(EVENT_BEAT_MS)`). And T001 found that `tail -n 25`
  of `cargo test -q` shows only the last test binaries' summaries, not the
  lib's 519, so reports add `cargo test -q 2>&1 | grep "test result"`.
- **What worked**: sanctioned changes stated as a **class** (T004's widened
  "every line" iterators; T007's venue panel assertions) landed first time,
  as they did in spec 029. So did call sites enumerated by grep before
  editing (T003, T007).

### What didn't change

`game.rs`, `card.rs`, `player.rs`, `campaign.rs`, `profile.rs`, `save.rs`,
`economy.rs`, `opponent.rs`, `board.rs`, `Cargo.toml` and `Cargo.lock` are
untouched. `PROFILE_VERSION` and `SAVE_VERSION` stay 1. The one new
persisted field is in the settings file, not a save or the profile, and is
`#[serde(default)]`. No existing banter line or pool changed. The thirteen
existing sounds are byte-identical. No new crate. The build has no warnings,
the same count as `main`. Monochrome by construction: the spoken line is the
existing banter line drawn through the existing drawer, and no colour path
was added.
```

---

## 3. Not drafted here (deliberately)

- **`design/brief.md` is on the branch.** Its *Motion* section gained the
  spec 030 amendment (T003): a spoken line is a one-shot transition that ends
  on its own "in under a second from its first word — which, for a line
  answering a round or match event, comes a short beat after the event's own
  sound" (T003a's clause). It rides into `main` with the merge.
- **`Readme.md` is on the branch.** Its status block quote reads "music/SFX/
  voice volume" (T002a) and "the board's card and score transitions and the
  opponent's word-by-word lines" (T003), both with their `> ` prefix.
- **`assets/CREDITS.md` is on the branch.** Its *Sound effects* sentence
  reads "square/triangle-wave blips and one additive, voice-like burble (the
  opponent's murmur as a line is spoken, spec 030), all generated from
  scratch" (T002).
- **`assets/how_to_play_text.txt` needs no change.**
  `grep -n -i "banter\|sound\|animation\|line" assets/how_to_play_text.txt`
  returns nothing (exit 1). How to Play describes neither banter, sound,
  animation nor settings rows, so nothing in it became inaccurate.
- **No `CLAUDE.md` amendment.** The spec adds no `Screen` and no overlay.
  Voices is a row on the existing Settings overlay, and the venue line draws
  on the existing venue `Screen`. `GamePhase` is untouched. The
  draw-never-mutates boundary holds: `draw` only reads `Speech::text`. The
  speech changes in `tick` (`advance_speech`, `update_banter`), `say`, and
  the two clears (`arrive_at_campaign`'s broke arrival, the Continue arm). The verification command is unchanged,
  and no crate was added.

---

## 4. The close-out notes, triaged

Every bullet of `tasks.md`'s *Notes for the close-out*, plus the tier-log
items that bear on it. Each is **resolved** (nothing owed), **DECISIONS**
(recorded in §2a), **ROADMAP** (§1) or **sweep** (§4b).

### 4a. The notes

| Note | Subject | Disposition | Where / why |
|---|---|---|---|
| Phase 1 review, 2nd look 1 | `line_visible` true on the venue before T007; which path reaches the venue after a game over | resolved | T007 drew the venue line. T008 confirmed that the acknowledgement reaches `arrive_at_campaign` through `enter_campaign(false)`. Phase 3 walkthrough item 4 showed a new line |
| Phase 1 review, 2nd look 2 | `BURBLE_GAP_MS` doc gave pre-sign-off reasoning | resolved | Corrected by the orchestrator (numbers unchanged) |
| Phase 1 review, 2nd look 3 | Words burble under the `?` overlay | DECISIONS | *Coverage* |
| Phase 1 review, 2nd look 4 | Silent cases rest on construction | DECISIONS | *Coverage*; the person's ear was the check (AC 9 approved) |
| Amendment sign-off, 2nd look 4 | Brief's "under a second" gains the beat clause | resolved | Landed (T003a); §3 and §5 quote it |
| Amendment sign-off, 2nd look 6 | Opponent bust's line never shows now | DECISIONS | *Coverage* |
| Amendment review notes | Beat 0.40–0.45 s; bust and match-end timing not driven; the Voices routing test lists sounds by hand; peak 0.79; later fields declared last | DECISIONS | *Coverage*, *The burble as shipped*, and the field-order design call |
| Phase 2 review (a) | AC 4 rests on `match_series`'s rule during Quick Play | sweep | S2: confirm `in_progress` can never point at the locked series in Quick Play. Also in *Coverage* |
| Phase 2 review (b) | No test pins non-final voices' `all_square` empty | DECISIONS | 1B bullet |
| Phase 2 review (c) | Lines flagged for the person's edit pass | resolved | The person: "Banter lines are fine. Continue". All kept as written |
| Phase 3 review (a) | Run-over notice covers the venue line | resolved | Ruling 13A, T008a; the taunt is on ROADMAP (§1e) |
| Phase 3 review (b) | Wager modal covers the line row at 89 | DECISIONS | *Coverage* |
| Phase 3 review (c) | "No burble on return" rests on the Back arms | DECISIONS | *Coverage* (silent cases) and the arrival design call; attested by the person at the Phase 3 pause ("as expected after testing") |
| Phase 3 review (d) | `banter_last` not persisted | DECISIONS | *Coverage* |
| Phase 3 review (e) | Acknowledgement MenuSelect and the venue's first burble on the same tick | resolved | Ruling 12A, T008a: the venue line waits the beat |
| Phase 3 review (f) | Breathing test draws with `None` | DECISIONS | *Coverage* (optional hardening) |
| Phase 3 re-review | Plan sentences superseded by 12A read as current | resolved | Marked **Superseded** in `plan.md` at this close-out: the §Amendment 11A row, §Design tension 3's interruption bullet, §Design tension 13's `say` paragraph, §Design 5's `say` doc quote, and §Design 5 Phase 3's `say(line, Duration::ZERO)` and "still says its line" (13A). Also marked, for T002c: the §Amendment 10A row's "serde default 0.8", §Design tension 9's band estimate, §Design 11's doc quote, §Tests' `…default_sfx_volume()` equality, and §Verification item 9's "Voices 80%" |
| Phase 3 re-review | `EVENT_BEAT_MS`'s doc names only event lines | resolved | `src/lib.rs` doc gains the 12A clause at this close-out (doc only) |
| Phase 3 re-review | Is the menu-select sound under 400 ms? | DECISIONS | *Coverage* (for the ear; an optional test extension) |
| Phase 3 re-review | ROADMAP gains the run-over taunt | ROADMAP | §1e |
| Tier log, T003 | `^\s*banter:` grep false-matches the import | DECISIONS | *Process notes* |
| Tier log, Phase 1 walkthrough | A run touched the real data dir | DECISIONS | *Process notes* |
| This close-out | The brief's beat clause doesn't name the venue line (12A) | sweep | S1 |

### 4b. For the pre-merge sweep

- **S1.** `design/brief.md`'s spec 030 sentence names the beat only for "a
  line answering a round or match event". Since 12A the venue line waits it
  too. It is not false but incomplete. Rule whether to add "or arriving at the
  venue" on the branch; AC 16 asks only that the brief "says a spoken line is
  a one-shot transition", which it does.
- **S2.** Confirm that `in_progress` can never point at the locked series
  during Quick Play (Phase 2 review (a)). AC 4 rests on it through
  `series_state_now`, `tick`'s `before` and the `InGame` draw arm.

---

## 5. Mechanical checks (T009, run on the branch at `f358789`, 2026-09-23)

`main` was at `b86b95d` for every comparison. No branch was switched. `main`
was built in a throwaway `git worktree` with its own `CARGO_TARGET_DIR`,
removed afterwards. The uncommitted changes when the checks ran were this
close-out's own: this file, the `plan.md` supersession marks, the `spec.md`
ticks, and the `src/lib.rs` doc clause.

### The verification command, three consecutive runs

`cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`,
three times in a row. The first build compiled the `lib.rs` doc change
(``Finished `dev` profile … in 4.78s``); runs 2 and 3 were no-ops
(``… in 0.06s``, ``… in 0.05s``). Each run's tail is identical: four
`1 passed` binaries and one `0 passed`, all `ok`. `tail -n 25` reaches only
the last test binaries, so every binary's summary is below, from
`cargo test -q 2>&1 | grep "test result"`:

```
test result: ok. 519 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.89s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(… six more `1 passed` lines and one `0 passed`, all ok)
```

### The branch's whole diff against `main` (three-dot)

```
$ git diff main...HEAD --stat
 Readme.md                        |    5 +-
 assets/CREDITS.md                |    7 +-
 assets/sfx/burble.wav            |  Bin 0 -> 10628 bytes
 design/brief.md                  |    7 +-
 scripts/gen_sfx.py               |   71 +-
 specs/030-series-banter/plan.md  | 1325 ++++++++++++++++++++++++++++++++++++++
 specs/030-series-banter/spec.md  |  288 +++++++++
 specs/030-series-banter/tasks.md |  771 ++++++++++++++++++++++
 src/app.rs                       |  295 +++++++--
 src/audio.rs                     |  187 +++++-
 src/banter.rs                    |  605 ++++++++++++++++-
 src/layout.rs                    |   26 +-
 src/lib.rs                       |   10 +
 src/portrait.rs                  |   40 +-
 src/settings.rs                  |  147 ++++-
 src/venue.rs                     |   63 +-
 tests/whole_file_write.rs        |    3 +-
 17 files changed, 3753 insertions(+), 97 deletions(-)
$ git diff main...HEAD --stat -- src/game.rs src/card.rs src/player.rs src/campaign.rs src/profile.rs src/save.rs src/economy.rs src/opponent.rs Cargo.toml Cargo.lock
(empty)
$ git diff main...HEAD --stat -- tests/
 tests/whole_file_write.rs | 3 ++-
 1 file changed, 2 insertions(+), 1 deletion(-)
```

### Versions, sounds, and the counted calls

```
$ grep -n "VERSION" src/save.rs src/profile.rs   (the two constants)
src/profile.rs:35:const PROFILE_VERSION: u32 = 1;
src/save.rs:30:const SAVE_VERSION: u32 = 1;
$ git ls-files assets/sfx | wc -l
      14
$ grep -c 'include_bytes!("../assets/sfx/' src/audio.rs
14
$ git diff main...HEAD --stat -- assets/sfx
 assets/sfx/burble.wav | Bin 0 -> 10628 bytes
$ grep -nE 'self\.screen = Screen::(CampaignMap|Venue)' src/app.rs   (open_campaign_home is 815–)
817:            self.screen = Screen::Venue {
821:            self.screen = Screen::CampaignMap {
$ grep -n 'burble_cue(' src/app.rs   (in burble, 1256–)
1258:            self.audio.play_cue(burble_cue(word));
$ grep -n 'self\.burble(' src/app.rs   (say 1206–, advance_speech 1234–, handle_settings_input 1710–)
1212:            self.burble(0);
1249:            self.burble(word);
1739:                    self.burble(0);
$ grep -n 'from_millis(EVENT_BEAT_MS)' src/app.rs   (arrive_at_campaign 836–, update_banter 1267–)
855:        self.say(line, Duration::from_millis(EVENT_BEAT_MS));
1288:                self.say(line, Duration::from_millis(EVENT_BEAT_MS));
```

### Warning count equals `main`'s

```
$ git worktree add --detach <scratch>/main-wt main
HEAD is now at b86b95d Roadmap: the opponent's line on the compact board
$ (cd <scratch>/main-wt && CARGO_TARGET_DIR=<scratch>/main-target cargo build --all-targets) > main-build.txt 2>&1
main exit 0
$ CARGO_TARGET_DIR=<scratch>/branch-target cargo build --all-targets > branch-build.txt 2>&1
branch exit 0
$ grep -c warning main-build.txt
0
$ grep -c warning branch-build.txt
0
$ git worktree remove <scratch>/main-wt; git worktree list
/Users/erikh/Projects/Rust/kaazap f358789 [030-series-banter]
```

**0 on `main`, 0 on the branch: equal.** Both target directories were
removed, and the branch is still `030-series-banter`.

### No colour path

`git diff main...HEAD -- src tests | grep -nE '^\+.*(Color|SetForegroundColor|SetBackgroundColor|\\x1b)'`
returns nothing (exit 1).

### The burble's figures and the beat's lengths

```
$ cargo test -q --lib the_burble_is_as_loud_as_the_other_sounds -- --nocapture 2>&1 | grep -iE 'peak|rms|band|floor|test result'
CardDraw rms 0.1629
CardPlay rms 0.2137
Flip rms 0.1882
Stand rms 0.0992
move sounds mean rms 0.1660
burble rms 0.1668, peak 0.7899; at default voices rms 0.0834
band (rms at default sfx) 0.0794..=0.1710
floor: music rms 0.1641 over its first 60 s, at default music rms 0.0820
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 518 filtered out; finished in 3.94s
$ cargo test -q --lib the_event_beat_outlasts_the_round_sounds -- --nocapture
RoundWin at pitch 1: 210 ms, beat 400 ms
RoundLoss at pitch 1: 270 ms, beat 400 ms
RoundTie at pitch 1: 210 ms, beat 400 ms
Bust at pitch 0.92: 380 ms, beat 400 ms
GameWin: 450 ms, beat 400 ms
GameLoss: 520 ms, beat 400 ms
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 518 filtered out; finished in 0.01s
```

### Spec 030's tests, by name

`cargo test --lib -- <the names below>` → `test result: ok. 38 passed; 0
failed; 0 ignored; 0 measured; 481 filtered out`. Each passed:
`the_word_step_is_about_a_fifth_of_a_second`,
`every_line_finishes_inside_a_second`,
`a_line_is_revealed_in_place_word_by_word`,
`a_spoken_line_shows_a_word_per_step_from_the_first_frame`,
`animations_off_speaks_the_whole_line_and_owes_no_more_burbles`,
`a_settled_line_is_whole_and_silent`,
`a_stalled_step_shows_every_due_word_and_owes_one_burble`,
`a_revealed_line_draws_each_word_where_the_finished_line_has_it`,
`the_burble_is_as_loud_as_the_other_sounds`,
`a_burble_ends_before_the_next_can_start`, `burbles_are_spaced_by_the_gap`,
`each_word_has_its_own_burble_pitch`,
`the_voices_volume_defaults_loads_and_persists`,
`adjust_steps_and_clamps_the_voices_volume`,
`the_voices_row_sits_between_sound_fx_and_animations`,
`settings_rows_move_over_four_rows_and_clamp`,
`the_burble_follows_voices_and_nothing_else_does`,
`the_event_beat_is_about_a_third_of_a_second`,
`a_line_after_a_beat_shows_nothing_until_the_beat_ends`,
`after_zero_is_the_line_at_once`,
`animations_off_after_a_beat_is_whole_with_one_burble`,
`a_line_settled_in_its_beat_is_whole_and_silent`,
`a_stalled_step_across_the_beat_shows_every_due_word_once`,
`the_event_beat_outlasts_the_round_sounds`,
`the_series_state_for_every_score`,
`every_reachable_state_has_lines_in_every_voice`,
`every_voice_is_complete_for_series_play`,
`the_start_pool_follows_the_series_state`,
`a_deciding_match_end_draws_the_series_result`,
`outside_a_decided_series_every_event_draws_todays_pool`,
`decided_series_agrees_with_the_series_rule`,
`the_venue_line_sits_inside_the_portrait_panel`, the widened
`every_line_of_every_set_fits_the_panel`,
`every_class_of_every_set_has_distinct_lines` and
`no_line_is_shared_across_any_two_sets`, and the existing
`settings_missing_or_legacy_fields_use_defaults`,
`settings_malformed_or_empty_json_falls_back_to_default` and
`audio_gating_respects_mute_and_volume`. Separately,
`the_board_shows_a_score_only_for_a_series_match` (the existing
`board_series_line` test that pins `match_series`) → `1 passed`.

---

## 6. `spec.md`'s 17 acceptance criteria, checked off with evidence

Criteria 1–8 and 10–17 are ticked in `spec.md` by this close-out. **Criterion
9 is the orchestrator's to tick**, with the person's approval quoted.

- [x] **1. Series states.** `the_series_state_for_every_score` checks literal
  tables, not the formula: all four best-of-3 cells and all nine best-of-5
  cells, with `(1,1)` All square and `(2,2)` Decider in the best of 5.
- [x] **2. Match-start lines follow the series.**
  `the_start_pool_follows_the_series_state` (None and Opening give today's
  `match_start`, each other state its own pool);
  `every_reachable_state_has_lines_in_every_voice`. Phase 2 walkthrough:
  against Greeb the Opening greeting, then Trailing, Leading and Decider
  openers; The Sovereign's All square at 1–1 and Decider at 2–2.
- [x] **3. Series result lines.** `a_deciding_match_end_draws_the_series_result`
  and `outside_a_decided_series_every_event_draws_todays_pool`. Phase 2
  walkthrough: series won (opponent) and series lost (player) end lines on
  deciding matches.
- [x] **4. No series, no change.**
  `outside_a_decided_series_every_event_draws_todays_pool` (every voice,
  every event), `the_start_pool_follows_the_series_state` (None is
  `match_start`), and `start_match` passes `pick` `None` outside a series
  (`state.and(self.banter_last)`, sign-off B1). The Phase 2 walkthrough
  re-drove a cleared-planet rematch and Quick Play after T006. Sweep S2
  confirms the rule this rests on.
- [x] **5. Venue lines.** Phase 3 walkthrough, at 89×31 and 139×31: a line
  word by word on launch from the map; the same line whole after Card Shop →
  Back and collection → Back; a new line after a non-deciding match, on
  menu re-entry and after a relaunch. T008a re-drive: the venue line starts
  after the beat at both widths, and the panel is blank under the run-over
  notice at both. The person attested the silent return ("as expected after
  testing"). `the_venue_line_sits_inside_the_portrait_panel`.
- [x] **6. No back-to-back repeat.** Phase 3 walkthrough: the match opened on
  a different line from the venue's. `arrive_at_campaign` and the series
  match start both pass `banter_last` to `pick`, whose no-repeat rule is spec
  017's (tested there); both draw from `start_lines` for the same state.
- [x] **7. Every voice is complete.** `every_voice_is_complete_for_series_play`
  (≥ 3 Leading, Trailing and Decider, ≥ 2 series won and lost, ≥ 3 All square
  where `wins_needed` is 3), `every_line_of_every_set_fits_the_panel`,
  `every_class_of_every_set_has_distinct_lines` and
  `no_line_is_shared_across_any_two_sets`, all widened to the new pools.
  There are 146 lines, and the person read them all at the Phase 2 pause.
- [x] **8. Word by word, in place.** `a_line_is_revealed_in_place_word_by_word`
  (every line of every pool), `a_spoken_line_shows_a_word_per_step_from_the_first_frame`,
  `a_revealed_line_draws_each_word_where_the_finished_line_has_it`,
  `a_line_after_a_beat_shows_nothing_until_the_beat_ends`,
  `the_event_beat_is_about_a_third_of_a_second`. Phase 1 walkthrough: the
  greeting and round lines grow a word per ~0.2 s in place. Amended
  walkthrough: the round line row is empty ~0.3 s after the round resolves,
  then word by word, and the greeting has no beat. T008a re-drive: the venue
  line starts after the beat.
- [ ] **9. The burble.** *The orchestrator's to tick.* The approval to quote,
  from the Phase 1 re-listen: "Murmur is good now and is correctly
  reflecting the settings." Supporting evidence:
  `the_burble_is_as_loud_as_the_other_sounds`,
  `the_burble_follows_voices_and_nothing_else_does`,
  `each_word_has_its_own_burble_pitch`, and the burble is generated by
  `scripts/gen_sfx.py` (`assets/CREDITS.md`).
- [x] **10. Interruption and clearing.** By construction: one
  `Option<Speech>`, replaced by `say` and set to `None` on a clear, and
  burbles come only from advancing it. `a_settled_line_is_whole_and_silent`,
  `a_line_settled_in_its_beat_is_whole_and_silent`,
  `burbles_are_spaced_by_the_gap`, `a_burble_ends_before_the_next_can_start`,
  and the counted calls (§5: one `burble_cue(`, three `self.burble(`).
- [x] **11. Nothing waits.** No input path reads `speech`: in `app.rs`,
  `self.speech` is written in `arrive_at_campaign`, `say`, `update_banter`'s
  clear and the Continue arm, and read only in `advance_speech` (from
  `tick`) and `draw` (checked at the Phase 1 review and the amendment
  review). No existing assertion about phases, popups or timings changed. The
  only existing-test changes are the sanctioned classes (T002a's superseded
  settings tests, T004's widened iterators, T007's venue panel geometry,
  T002c's one default). Amended walkthrough: the popup still draws at
  ~0.8 s.
- [x] **12. Animations Off.**
  `animations_off_speaks_the_whole_line_and_owes_no_more_burbles`,
  `animations_off_after_a_beat_is_whole_with_one_burble`. Phase 1 walkthrough
  item 2 (whole); amended walkthrough item 7 (whole, after the beat).
- [x] **13. Resumed match and compact board.** Phase 1 walkthrough: a resumed
  match is blank until the next event; at 89 columns there is no line on the
  board. The Continue arm sets `speech = None`, and `line_visible` gates
  every burble on `BoardView::is_wide()`. Silence was the person's to hear
  (AC 9's walkthrough).
- [x] **14. Final score held.** `decided_series_agrees_with_the_series_rule`
  and the unchanged `the_board_shows_a_score_only_for_a_series_match`. Phase
  2 walkthrough: the game-over frames held 0–2, 1–2 and 2–0, and the map
  banner was unchanged with no score. `final_series` is read only by
  `update_banter` and the `InGame` draw arm (§2a).
- [x] **15. Nothing else changes.** §5: no engine, campaign, profile, save,
  economy or opponent file in the diff; both versions 1; `Cargo.toml` and
  `Cargo.lock` untouched; no colour path; the only `tests/` change is T002a's
  one literal. The settings field is serde-defaulted
  (`the_voices_volume_defaults_loads_and_persists`,
  `settings_missing_or_legacy_fields_use_defaults`).
- [x] **16. Docs.** §3: `design/brief.md`'s *Motion* amendment (a spoken line
  is a one-shot transition, with the beat clause), `Readme.md`'s "voice
  volume" and "word-by-word lines", and `assets/CREDITS.md`'s burble sentence
  all landed. How to Play's grep is empty, so nothing in it became
  inaccurate. (Sweep S1 on the brief's venue beat.)
- [x] **17. Voices volume.** `the_voices_row_sits_between_sound_fx_and_animations`,
  `adjust_steps_and_clamps_the_voices_volume`,
  `settings_rows_move_over_four_rows_and_clamp`,
  `the_voices_volume_defaults_loads_and_persists` (a file without the key
  loads with the default and keeps every other value) and
  `the_burble_follows_voices_and_nothing_else_does`. `tests/whole_file_write.rs`
  round-trips a non-default Voices on disk. Amended walkthrough items 8–9:
  the overlay order at 89 and 139, persistence across a relaunch, and an old
  file loading with the default (80% then; 50% since T002c).
