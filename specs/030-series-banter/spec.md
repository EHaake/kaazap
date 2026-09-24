# Spec: Series-aware banter, spoken word by word — spec 030

**Status**: Approved (2026-09-23). Rulings 1B, 2A, 3B, 4A, 5B, 6A (the
person, 2026-09-23) — see *Resolved decisions*. Approved with one condition:
the person approves the burble by ear once it is built, and may ask for it to
be tweaked (see AC 9). **Amended 2026-09-23 at the Phase 1 pause** (rulings
9A, 10A, 11A): the burble is as loud as the other sound effects rather than
softer than the music, has its own Voices volume in Settings, and a line said
on a round or match event waits a short beat so the event's own sound lands
first.
**Depends on**: spec 017 (opponent banter), spec 004 (audio and the
generated sound effects), spec 027 (the Animations setting and the Motion
amendment), spec 029 (series, the venue, the in-match series line).

## Summary

Since spec 029 a campaign opponent is played as a series — best of 3, best
of 5 for The Sovereign — but the opponent's lines don't know it. Every match
of a series opens with the same kind of greeting, so a series repeats "Here
goes nothing!" two to five times, and the line at the end of a match can't
tell a match that just took the series from one that didn't.

This spec makes the opponent's lines **follow the series**: a match started
mid-series gets a line for where the series stands (the opponent leading,
trailing, level, or one match from deciding it all), the match that decides
a series ends on a **series won** or **series lost** line, and the opponent
now also **speaks at the venue** when the player arrives there.

And every line is now **spoken**: its words appear one at a time, in place,
as if the opponent were talking, each word with a **soft burble** — soft in
character, as audible as the game's other sounds, with its own **Voices**
volume in Settings.

The deciding match's game-over frame also now keeps the **final series
score** on screen, where today it shows none.

No engine, AI, save-format, economy or balance change. Monochrome by
construction.

## Goals

1. **A series reads as one contest, not two to five unrelated matches.**
   The opponent notices the score: gloats when ahead, needles when behind,
   and meets the decider as the decider.
2. **The end of a series lands.** The line after the deciding match says the
   series is won or lost, not merely the match.
3. **The opponent feels present, not captioned.** Lines are spoken, word by
   word, with a voice-like sound — without ever slowing the player down.
4. **The final score is seen.** The frame that ends a series shows how it
   ended.

## Non-goals (explicitly deferred)

- **New banter moments** beyond those named here. Rounds, busts and ties keep
  their lines and fire exactly when they do today.
- **Per-opponent voices for the burble** (a different pitch or sound per
  opponent). One burble for everyone in this spec.
- **Word-by-word text anywhere else** — notices, popups, How to Play, the
  primer, the map. Only opponent lines are spoken.
- **Skipping the reveal with a key.** Lines are two to four words and finish
  in under a second; keys keep doing exactly what they do today.
- **Music** — its own backlog items (*Original cantina-vibe music*,
  *Per-planet music*).
- **Carrying the final series score to the map.** The map's existing result
  banner is unchanged; the held score belongs to the game-over frame only
  (ruling 6A).

## Entities

- **Line** — one short thing the opponent says, at most 20 characters (the
  panel's interior width, unchanged), in that opponent's voice.
- **Series state** — where the series stands from the opponent's side, read
  at the moment a line is chosen:
  - **Opening** — the first match (0–0).
  - **Leading** — the opponent has won more matches than the player.
  - **Trailing** — the opponent has won fewer.
  - **All square** — level, but neither side is one win from the series.
    Only reachable in a best of 5 (at 1–1), so only The Sovereign plays it.
  - **Decider** — both sides are one win from the series (1–1 in a best of
    3, 2–2 in a best of 5).
- **Series result** — the deciding match's outcome: **series won** (the
  opponent took it) or **series lost**.
- **Spoken line** — a line whose words appear one at a time.
- **Burble** — the soft, voice-like sound played as each word appears.

## Key behavior

### Which line is said

- **A match starting in a series** (ruling 1B) chooses from the lines for
  its series state:
  - Opening → the opponent's greetings, exactly the lines used today.
  - Leading, Trailing, Decider → new lines for each.
  - All square → new lines, written for The Sovereign.
- **A match ending in a series** (ruling 2A):
  - If the match decides the series → a **series won** or **series lost**
    line.
  - Otherwise → today's match-win / match-loss lines, unchanged.
- **Quick Play and rematches on a cleared planet** have no series, so they
  keep today's lines for every event, unchanged. They are spoken like every
  other line.
- **At the venue** (ruling 3B), the opponent says one line when the player
  **arrives**: on starting a series from the map, on returning from a match
  of the series that didn't decide it, and on entering the campaign while a
  series is in progress (from the menu, or when the game is launched). The
  line comes from the same series-state lines as a match start. A trip to the
  Card Shop or the collection and back is not an arrival: the line already
  shown is still there, drawn whole, with no burble. An arrival that ends the
  run — a loss that leaves the player broke, shown under the run-over notice —
  says **no** line and leaves the panel blank (ruling 13A).
- **The venue line waits a beat too** (ruling 12A): its first word comes the
  same short beat after the arrival as an event line's, so the burble does
  not land on the menu click that brought the player there.
- The venue's line and the match-start line that follows it come from the
  same set, so the **no back-to-back repeat** rule (spec 017) holds across
  the two: the match never opens with the line the venue just said.
- Every voice — the ten roster opponents and the neutral fallback — has lines
  for every state it can reach, so no line is ever blank and never another
  opponent's voice (spec 017's rule). Each new state has **at least three**
  lines per voice (the venue and the match start can draw on it back to back),
  and series won / series lost **at least two**.
- The lines are written in each opponent's existing voice by Claude Code;
  the person edits whatever doesn't land (spec 017's practice).

### How a line is spoken

- **Word by word** (ruling 4A). The first word appears on the frame the line
  is chosen; each following word about **0.2 seconds** after the one before.
  A four-word line is complete in about 0.6 seconds.
- **A short beat on events** (ruling 11A). A line said in answer to a game
  event — a round won, lost or tied, a bust, a match or series ending — is
  chosen when it is today, but its first word appears about **0.3–0.4
  seconds** later, once the event's own sound has played, so the burble does
  not compete with it. Until then the panel shows no line. The match-start
  greeting starts at once; the venue line waits the same beat (ruling 12A). Nothing else moves: popups, sounds, phases and keys keep today's
  timing.
- **In place.** Each word appears in the position it will hold in the
  finished line; the line never shifts sideways or re-centres as it grows.
- **A soft burble per word** (ruling 5B). Each word, as it appears, plays one
  short, soft, voice-like burble — an alien murmur, not a beep or a
  typewriter click. It is **as loud as the game's other sound effects**
  (ruling 9A): soft in character, not in level, and clearly audible over the
  music at the default settings. (The first build held it below the music's
  average level, and it was nearly inaudible.) It is generated from scratch
  like the existing sound effects, so it carries no third-party licence.
- **A Voices volume** (ruling 10A). Settings gains a third slider, **Voices**,
  beside Music and Sound FX, working the same way. The burble follows Voices,
  not Sound FX; the `m` mute silences it like every other sound. With Voices
  at zero or muted it is silent. A settings file written before this spec
  loads with Voices at its default.
- **Interruption.** A new line replaces one still being spoken, at once, and
  starts from its first word; the old line's remaining words and burbles
  never play. A line cleared while being spoken (play resuming, as spec 017
  clears it) stops there, and no further burbles play.
- **Nothing waits for it.** No key is delayed or ignored while a line is
  being spoken, and the game, the phases, the popups and every other sound
  happen exactly when they do today.
- **Animations Off** (spec 027's setting) → the line appears whole on the
  frame it is chosen (after the event beat, where one applies), with **one**
  burble (the sound follows the Voices setting, not the Animations setting).
- **A resumed saved match** shows no line, as today (spec 017 resumes with
  the line blank), so nothing is spoken and no burble plays; the first frame
  of a resumed match is drawn settled (spec 027). Ruling 8A.
- **The compact board** (89–138 columns, spec 026) draws no opponent line, so
  nothing is spoken there and no burble plays: "if the banter isn't visible,
  it makes no sense to include the speech burble." Ruling 7A.

### The deciding match's final score

- On the game-over frame of a match that decides a series — won or lost —
  the in-match series line shows the **final score** (for example 2–1),
  where today it shows nothing (ruling 6A).
- It stays until the player leaves the match. The map that follows is
  unchanged: its result banner, and no series score (ruling 6A).
- Frames of matches that don't decide the series are unchanged.

## Acceptance criteria

1. [ ] **Series states.** For every series score in a best of 3 and a best
   of 5, the state is Opening at 0–0, Leading when the opponent is ahead,
   Trailing when behind, Decider when both are one win from the series, and
   All square otherwise when level (best of 5 at 1–1 only).
2. [ ] **Match-start lines follow the series.** A campaign match started at
   each state shows a line from that state's lines for that opponent; at
   Opening, today's greeting lines.
3. [ ] **Series result lines.** A match that ends a series shows a series won
   line when the opponent took the series and a series lost line when the
   player did; a match that doesn't end it shows today's match-win /
   match-loss lines.
4. [ ] **No series, no change.** Quick Play and a cleared-planet rematch
   choose their lines exactly as they do today, for every event.
5. [ ] **Venue lines.** Arriving at the venue — from the map, after a
   non-deciding match, and on entering the campaign or launching the game
   with a series in progress — shows a line from the current state's lines,
   spoken, its first word a short beat after the arrival (ruling 12A);
   returning from the Card Shop or the collection shows the same line, whole,
   with no burble; an arrival under the run-over notice shows and says no
   line (ruling 13A).
6. [ ] **No back-to-back repeat.** The match-start line is never the line the
   venue just showed, and spec 017's no-repeat rule holds for every event.
7. [ ] **Every voice is complete.** Each of the ten roster voices and the
   fallback has at least three lines for Leading, Trailing and Decider and at
   least two for series won and series lost; The Sovereign has at least three
   All square lines; every line fits the panel's 20-character width.
8. [ ] **Word by word, in place.** A spoken line shows its first word on the
   frame it is chosen — or, for a line said on a round or match event, about
   0.3–0.4 seconds later (ruling 11A) — and one more word about every 0.2
   seconds; at every step each shown word is at the column it holds in the
   finished line.
9. [ ] **The burble.** One burble plays as each word appears; it is soft in
   character, voice-like, and about as loud as the other sound effects
   (ruling 9A); it follows the new Voices volume (ruling 10A) and is silent
   with Voices at zero or muted; it is generated in-repo.
   **The person approves the sound by ear** at a walkthrough once it is
   playable in the game, and may ask for it to be tweaked; this criterion is
   not met until they have.
10. [ ] **Interruption and clearing.** A new line replaces one being spoken
    and starts from its first word; a line cleared mid-reveal stops, and in
    both cases no further burble from the old line plays.
11. [ ] **Nothing waits.** Every key has the same effect on the same frame
    while a line is being spoken or waiting out its event beat; phases,
    popups, timings and other sounds are unchanged (existing tests
    untouched).
12. [ ] **Animations Off.** A line appears whole on the frame it is chosen
    (after the event beat, where one applies), with one burble.
13. [ ] **Resumed match and compact board.** A resumed saved match shows no
    line and plays no burble (ruling 8A); a match on the compact board shows
    no line and plays no burble (ruling 7A).
14. [ ] **Final score held.** The game-over frame of a deciding match, won or
    lost, shows the final series score; non-deciding matches' frames are
    unchanged; the map afterwards shows no series score and its banner is
    unchanged.
15. [ ] **Nothing else changes.** No engine, AI, save-format, economy or
    balance change; no new crate; monochrome. (The settings file gaining a
    Voices field, ruling 10A, is not a save-format change: saves and the
    profile are untouched.)
16. [ ] **Docs.** The README and How to Play still describe the game
    accurately; the design brief's Motion section says a spoken line is a
    one-shot transition; `assets/CREDITS.md` covers the new sound.
17. [ ] **Voices volume.** Settings shows a Voices slider beside Music and
    Sound FX that changes and persists like them; the burble follows it and
    no other sound does; a settings file without it loads with the default
    (ruling 10A).

## Resolved decisions (the person, 2026-09-23)

- **1B** — a match started mid-series has lines for Leading, Trailing and
  Decider, plus All square for the best of 5 at 1–1, rather than folding
  the decider into "all square".
- **2A** — the match that decides a series ends on series won / series lost
  lines; other match ends keep today's lines.
- **3B** — the opponent speaks one line at the venue on arrival, reacting to
  the series score.
- **4A** — about 0.2 seconds per word, the first word at once.
- **5B** — a soft burble per word: "soft and not louder than the music …
  sort of a 'soft burble' if possible."
- **6A** — the deciding match's game-over frame holds the final series
  score, which "should not follow the player to the galaxy screen."
- **7A** (asked at planning) — on the compact board, where no line is
  drawn, no burble plays. The person may later want the line on the compact
  board without the portrait; that is a backlog item, not this spec.
- **8A** (asked at planning) — a resumed match stays blank, as today; no
  line is spoken on resume.
- **9A** (Phase 1 pause) — the burble is as loud as the other sound effects;
  "never louder than the music" is dropped. The person found it "really,
  really quiet" and had to turn the music off to hear it.
- **10A** (Phase 1 pause) — a separate Voices volume slider in Settings,
  which the burble follows instead of Sound FX.
- **11A** (Phase 1 pause) — a line said on a round or match event waits a
  short beat (about 0.3–0.4 s) so it doesn't compete with the event's sound
  and popup; popups and keys keep today's timing.
- **12A** (Phase 3 pause) — the venue line waits the same short beat as an
  event line, so its first burble doesn't overlap the menu click of the
  arrival: "add a slight delay to the line so that the sounds don't overlap."
- **13A** (Phase 3 pause) — an arrival under the run-over notice says no line.
  The person would prefer the opponent to taunt that the run is over, but not
  if it needs further design; it does (the notice covers the panel at 89
  columns, and every voice would need new lines), so it is a roadmap
  follow-up.
- **Approval (2026-09-23)** — approved, with the burble subject to the
  person's approval by ear once implemented, and possible tweaks.

Defaults set in the spec conversation, not separately ruled: words appear in
place; a new line interrupts; Animations Off shows the line whole; a resumed
match's line is whole and silent; Quick Play keeps today's lines but is
spoken; the venue and match-start lines share a set, with no repeat across
the two; with Animations Off one burble plays; the Card Shop / collection
round trip is not an arrival.
