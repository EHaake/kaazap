# Spec: Crash & data safety — spec 028

**Status**: Approved (2026-09-19). Rulings Q1 a, Q2 a, Q3 b, Q4 a, Q5 a — all
taken as recommended by the person, 2026-09-19 (see *Resolved decisions*).
**Depends on**: spec 004 (the settings file), spec 005 (the match save and
the menu's **Continue**), spec 012/021/024 (the campaign profile: credits,
collection, run), the data-directory chore of 2026-09-19 (`paths.rs`, the
one root all three files resolve through)

## Summary

Three ways kaazap currently fails badly, all found in an audit of the
existing code and all confirmed in it.

**The terminal is left broken by anything but a clean quit.** kaazap turns
on raw mode, switches to the alternate screen and hides the cursor at
startup, and undoes all three in exactly one place: after the game loop
breaks on `q`. A failed terminal read, a panic while handling a key,
updating or drawing, or a panic on the render thread all leave that
restore unrun, and the player is dropped back to a shell with no cursor,
no echo and no line editing — a terminal they have to close or `reset`.

**A crash in the middle of a write truncates the file being written.** The
profile, the settings and the match save are each written by replacing the
file's contents in place. Interrupted — by a crash, a full disk, a power
cut — the file is left half written, and a half-written file is not
loadable. The write that was meant to preserve progress is what destroys
it.

**A profile that can't be read looks exactly like a first launch.** A
missing file, an unreadable one, a malformed one and one from an
incompatible version all collapse into "here is a brand-new starter
profile", with no signal anywhere. The player sees a fresh campaign, an
empty collection and the seed purse, and the first thing the game does
after that is save — writing that starter profile over the file it could
not read. The campaign is then genuinely gone, and nothing ever told the
player anything happened.

This spec fixes the three: the terminal is restored however kaazap ends,
every file is written whole or not at all, and a profile that can't be
read is set aside and reported rather than silently replaced.

No engine, AI, economy, balance-data or dependency change. No new screen
or mode. The version gate and the `serde` field defaults, which are
correct, are untouched.

## Goals

1. **The terminal always comes back.** However kaazap ends — clean quit,
   error, panic on either thread — the player's terminal is left as it was
   found: cursor visible, raw mode off, the alternate screen exited and
   the scrollback that was there before launch intact.
2. **A crash leaves something to say.** When kaazap ends by crashing, the
   player is told so in one plain line, and the underlying detail is
   printed with it, so a bug report has something in it and the player
   doesn't think the game merely vanished.
3. **A file is never half written.** After any interruption, each of the
   three files kaazap writes is either exactly as it was before the write
   or exactly as the write intended. There is no third state.
4. **Losing a campaign is never silent, and never automatic.** If the
   profile can't be read, the file is kept, the player is told at launch,
   and no save is allowed to overwrite what couldn't be read.
5. **A real first launch stays silent.** No file and no data directory is
   the ordinary case for a new player and says nothing.

## Non-goals (explicitly deferred)

- **Crash recovery.** kaazap does not try to keep running after a panic,
  does not catch a panic to return to the menu, and does not try to write
  anything from inside the crash path. It restores the terminal, reports,
  and exits. (An in-progress match survives a crash already, because the
  match save is written on every state change — nothing new is needed and
  nothing is added.)
- **Any change to the version gate or the field defaults.** A document
  whose version doesn't match is still discarded rather than
  mis-read; every `#[serde(default)]` field still fills in. `PROFILE_VERSION`
  and `SAVE_VERSION` stay 1.
- **Migrating an old profile.** A profile from an incompatible version is
  set aside and reported, not upgraded.
- **A notice for settings.** Volume preferences falling back to defaults
  stays silent, as today. Settings gets the whole-file write and nothing
  else.
- **Backups of good files**, rotation, or an archive on reset. The
  *archive the last run at reset* backlog item stays separate and still
  open.
- **Any in-app recovery.** A set-aside profile is recovered by hand,
  outside the game, by someone who knows what JSON is. There is no
  "restore my profile" screen, no repair, no partial salvage.
- **Any restructuring.** No engine, AI, economy, balance-data, save-format
  or dependency change; no new screen or mode; no change to the campaign,
  the shop, the deck builder or the board. The audit that found these
  three problems found under 1% excess structure elsewhere, and this spec
  does not touch it.

## Entities

- **Restored terminal** — the state the player's terminal is in after
  kaazap ends: cursor shown, raw mode off, alternate screen exited, prior
  scrollback intact. Reaching it is not conditional on how kaazap ended,
  and reaching it twice is harmless.
- **Crash report** — what is printed on the restored terminal when kaazap
  ends by crashing: one kaazap line naming what happened, followed by the
  underlying panic message and its location. Nothing is printed when
  kaazap ends normally.
- **Whole-file write** — the way the profile, the settings and the match
  save are written from this spec on: the new contents are complete on
  disk before they become the file the game reads. An interrupted write
  leaves the previous file untouched and leaves no debris that a later
  load could mistake for the real file.
- **Set-aside profile** — a profile file that couldn't be read, kept under
  a dated name beside where it was, so the player (or someone helping
  them) can look at it. Created before any fresh profile is written.
- **Data notice** — the modal raised over the start menu at launch when a
  file couldn't be read: what happened, and where the old file went. One
  notice per launch, covering everything that went wrong at that launch.

## Key behavior

### When kaazap ends

- **Every ending restores the terminal**: quitting with `q`, an error
  reading terminal input, a panic while handling a key / updating / drawing,
  and a panic on the render thread surfacing when the game loop waits for
  it. The restore happens once; a second attempt changes nothing.
- **A normal quit prints nothing** and exits successfully, exactly as
  today.
- **A crash prints the crash report** on the restored terminal — the
  kaazap line first, then the panic message and location — and exits
  unsuccessfully, so a shell or a wrapper can tell.
- **The crash report is readable**: it appears on the terminal the player
  came from, not on the alternate screen that is being torn down, and is
  not swallowed by the redraw.
- **A crash never loses an in-progress match beyond the last state
  change** — the behaviour the match save already has. **Continue** works
  on the next launch.

### How files are written

The profile, the settings and the match save are all written whole. For
each of them:

- An interrupted write leaves **the previous file exactly as it was**.
- A completed write leaves **the new contents in full**.
- Whatever an interrupted write leaves behind is never loaded as if it
  were the real file, and does not accumulate across launches.
- Nothing else about when these files are written changes: the same
  moments, the same contents, the same silent best-effort failure when
  there is nowhere to write.

### When the profile can't be read

Four cases, and they stop being one case:

1. **No file, or no data directory.** A first launch. Silent. A starter
   profile, exactly as today.
2. **The file is there but can't be read** (permissions, an I/O error).
3. **The file is there but can't be parsed** (malformed, truncated).
4. **The file parses but its version doesn't match.**

In cases 2, 3 and 4:

- The game **keeps the file**: before anything else, it is moved aside
  under a dated name beside where it was.
- The player gets a **data notice** at launch, over the start menu. It
  says the profile couldn't be read, **why** — "couldn't be read" for
  cases 2 and 3, "saved by a different version of kaazap" for case 4 —
  and where the file was moved to, by name.
- Play continues on a **starter profile**. Everything works; the campaign,
  the collection and the purse are the new-player ones.
- **If the file cannot be moved aside**, the game says so in the notice
  and **does not save the profile at all for the rest of that launch** —
  nothing the player does overwrites a file that couldn't be read. Play
  still continues on a starter profile; nothing from that session
  persists, and the notice says that too.

### When the match save can't be read

The same four cases, with a smaller consequence and no file kept.

- Missing, or no data directory: silent, as today. No **Continue**.
- Unreadable, malformed, or the wrong version: the **data notice** carries
  a line saying the in-progress match couldn't be read, and the unreadable
  save file is removed so the notice doesn't repeat on every later launch.
  **Continue** is absent, as it is today.

### The data notice

- Raised at launch, over the start menu, before the player does anything.
- Covers everything that went wrong at that launch — profile, match save,
  or both — in one modal.
- Enter, Space or Esc dismiss it. `q` still quits and `m` still mutes,
  as they do under every other modal in the game; nothing else acts while
  it is up. (Amended 2026-09-19 at sign-off — Q6.)
- Transient: it is shown once, for the launch it belongs to. It is not
  saved, not re-shown after dismissal, and dismissing it is not recorded
  anywhere.
- Dismissing it leaves the player on the start menu with the selection
  where it starts.

## Design requirements

- The notice is a modal in the game's existing visual language — a
  centred, bordered box, monochrome, padded evenly by one empty row above
  and below its content.
- The one line the player acts on (the dismiss line) gets an empty row
  above and below it; the rest of the text stays compact. This is the
  constitution's *acted-on element stands apart* rule, and it is checked
  at review, not after the person plays it.
- The notice fits the minimum terminal (89×31) with nothing clipped, at
  its longest — both failures reported at once, with a long file name.
- The wording is plain. No error codes, no Rust types, no paths the player
  can't act on. It names the file it set aside the way the player would
  find it.

## Acceptance criteria

- [ ] Quitting kaazap with `q` leaves the terminal exactly as it is today:
      cursor back, echo back, prior scrollback intact, nothing printed, a
      successful exit.
- [ ] A panic in key handling, in the per-frame update, or in drawing
      leaves a usable terminal — typed characters echo, the cursor is
      visible, the shell responds — followed by the kaazap crash line and
      the panic's own message and location, and an unsuccessful exit.
- [ ] A panic on the render thread, surfacing when the game loop waits for
      it at shutdown, does the same.
- [ ] An error reading a terminal event does the same, with no panic text
      where there was no panic.
- [ ] Restoring the terminal twice is harmless: no error, no double
      output.
- [ ] For each of the profile, the settings and the match save: a write
      that fails partway leaves the previous file byte-for-byte unchanged,
      and a write that completes leaves the new contents in full — pinned
      by tests against a scratch data directory, not the real one.
- [ ] After an interrupted write, a following launch loads the previous
      file, not the debris; nothing left behind is loaded as the real file
      and nothing accumulates across repeated interrupted writes.
- [ ] A launch with no profile file and no data directory is silent, shows
      no notice, and gives a starter profile — today's behaviour, pinned so
      it stays.
- [ ] A launch with an unreadable profile, a malformed profile, and a
      wrong-version profile each: set the file aside under a dated name,
      leave the original contents intact in the set-aside file, show the
      notice naming the reason and the set-aside file, and continue on a
      starter profile.
- [ ] After any of those three, playing on and triggering a profile save
      does not write over the set-aside file and does not produce a second
      set-aside copy.
- [ ] When the set-aside move itself fails, the notice says so, the
      unreadable file is still there and unchanged at the end of the
      session, and no profile save wrote anything for the rest of that
      launch.
- [ ] A launch with an unreadable, malformed or wrong-version match save
      shows the notice line for it, has no **Continue**, and does not show
      that line again on the next launch.
- [ ] A launch where both the profile and the match save are unreadable
      shows one notice carrying both.
- [ ] The notice draws inside an 89×31 terminal with its longest content,
      nothing clipped, with an empty row above and below the dismiss line
      and even padding inside the box; Enter, Space and Esc each dismiss it
      to the start menu, and no other key does anything while it is up —
      apart from `q` and `m`, which quit and mute as they do under every
      other modal (Q6).
- [ ] A settings file that can't be read is still silent, and still falls
      back to defaults.
- [ ] `cargo build --all-targets` and `cargo test` are green, and the
      driver walkthrough (against a scratch data directory) shows: a normal
      quit, a crash, a corrupted profile, a corrupted save, and both at
      once.
- [ ] No change to `card.rs`, `game.rs`, `player.rs`, `opponent.rs`,
      `economy.rs`, `campaign.rs`, `wager.rs`, `tests/balance.rs`,
      `Cargo.toml` or `Cargo.lock`; `PROFILE_VERSION` and `SAVE_VERSION`
      stay 1; the version checks and every `#[serde(default)]` are
      unchanged.

## Resolved decisions (the person, 2026-09-19)

- **Q1 a — the unreadable profile is moved aside under a dated name, and
  the notice says where it went.** The alternatives were to leave it and
  refuse to save (nothing persists, and the player has to know that), or
  to leave it and let the next save overwrite it with only a warning
  beforehand. Setting it aside is the only one where a recoverable
  campaign is actually recoverable, and it is the same instinct as the
  *archive the last run at reset* backlog item.
- **Q2 a — the notice is a modal on the start menu, dismissed with a
  key.** A persistent line or a banner that clears on the first navigation
  were both offered. Losing a run deserves a stop, not a line the player
  scrolls past.
- **Q3 b — the notice says why.** "Couldn't be read" and "saved by a
  different version of kaazap" are separate messages, because the version
  case is the only one where the player can act — by finding the build
  that wrote it.
- **Q4 a — a bad match save gets a notice too, but no file is kept.**
  Today **Continue** simply disappears with no explanation. The
  consequence of not keeping it, accepted here: the unreadable save is
  removed so the notice doesn't repeat every launch, so that one match is
  gone for good. A match is not a run.
- **Q5 a — the crash report is one kaazap line plus the panic message and
  location**, rather than raw panic output alone, so the player knows it
  was kaazap and a bug report has something in it.

## Amendment (the person, 2026-09-19, at plan sign-off)

- **Q6 a — `q` and `m` still act while the notice is up.** The sign-off
  found the plan keeping both live against this spec's original "no other
  key does anything while it is up", and raised it rather than deciding it.
  Both keys are handled before the modal chain — `q` in the game loop
  itself — so suppressing them would make this the one modal in the game
  you cannot quit or mute under, and would mean the game loop has to start
  asking `App` which modal is open. The two lines above are amended to say
  so; Enter, Space and Esc still dismiss, and nothing else acts.
