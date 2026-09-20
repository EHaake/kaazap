# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 028)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T009, ready to paste on `main` after the merge
(the same close-out shape specs 020–027 used). Nothing here is applied by the
spec branch.

Spec 028 amends no rule `CLAUDE.md` states — see §3. Nothing to apply there.

Line numbers below are **`main`'s** at the time of drafting (2026-09-20,
`main` at `cd015c5`, *Merge pull request #35 from EHaake/chore/data-dir-seam*).
Each edit quotes its anchor text verbatim, which is what to match on; every
anchor below was re-checked with `grep -c` against `git show main:<file>` at
`cd015c5` and is unique there (`ROADMAP.md` 827 lines, `DECISIONS.md` 1654).

Applying: never chain a file edit, a branch switch and a commit in one shell
command (spec 025's miss) — switch to `main`, edit, verify the anchors again,
then commit.

**One wording rule this close-out is held to.** The terminal guarantee is
described as **the endings `spec.md` enumerates** — quitting with `q`, an error
reading a terminal event, and a panic in key handling, in the per-frame update,
in drawing, or on the render thread — and never as "however kaazap ends". A
`SIGTERM` or `kill -9` still leaves the terminal unrestored (the driver's
kill-to-leave-a-save trick is exactly that case). Signals are outside this
spec's scope by design (plan §Design tension 1, §Open questions 8), and the
close-out is the place that claim would otherwise get widened.

---

## 1. `ROADMAP.md` — two edits, the rest judged and left

`grep -n -iE "crash|data safety|whole-file|corrupt|atomic" ROADMAP.md` on
`main` returns **no hits** (exit 1). There is **no backlog entry for this spec
to close**: crash and data safety was found by an audit, not carried on the
roadmap. So 1a is a new *Shipped* entry and there is no "mark the backlog
bullet shipped" edit to make.

A wider read was done and judged: `grep -n -iE "panic|restore|archive the last
run|profile\.json|savegame"` returns the `savegame.json` / `profile.json`
mentions inside earlier Shipped entries (45, 82, 266, 272, 334, 443, 535, 776 —
all about what those files *hold*, none about how they are written or read, all
left), and the **Archive the last run at reset** backlog bullet (713–719),
which stays open and untouched: it is a named non-goal of this spec, and
setting an *unreadable* profile aside is a different thing from archiving a
*good* one at reset. The only edit outside 1a is the path-injection seam's
follow-up, 1b.

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Animation pass (spec 027)** entry, which ends
with these two lines (currently `ROADMAP.md` lines 508–509, just before the
blank line and `## Backlog`):

```markdown
  frames at 89×31 and 139×31, and the Off state at 89×31. `Readme.md`'s settings mention names the
  row.
```

Insert after them:

```markdown
- **Crash & data safety** (spec 028) — three ways kaazap failed badly, found
  by an audit of the existing code and all three fixed. **The terminal now
  comes back at every ending `spec.md` enumerates**: quitting with `q`, an
  error reading a terminal event, and a panic in key handling, in the
  per-frame update, in drawing, or on the render thread. A `TerminalGuard`
  declared first in `main` (so it drops last) owns the restore — its `Drop`
  runs while a panic unwinds — and a panic hook installed before raw mode
  **records** the first panic instead of printing it, so the **crash report**
  (one kaazap line, then the panic's own message and location — Q5 a) lands on
  the terminal the player came from rather than on the alternate screen being
  torn down. Signals are deliberately outside the spec: `SIGTERM` and
  `kill -9` still leave the terminal unrestored. **Every file is written whole
  or not at all**: one `paths::write_whole` (bytes to a fixed temp file beside
  the target, then `fs::rename` over it) now serves the profile, the settings
  and the match save, so an interrupted write leaves the previous file
  byte-for-byte unchanged and leaves at most one `*.tmp` of debris, which is
  never `*.json` and so is never loaded and never accumulates. **A profile
  that can't be read is no longer mistaken for a first launch**: missing — or
  no data directory at all — stays silent with a starter profile, but
  unreadable, malformed and wrong-version each keep the file aside as
  `profile-YYYYMMDD-HHMMSS.json` beside where it was (Q1 a) and raise a **data
  notice** modal over the start menu (Q2 a) saying which failure it was
  ("couldn't be read", or "saved by a different version of kaazap" — Q3 b) and
  where the file went. If the move itself fails, the notice says so and a
  process-wide flag **suspends every profile save for the rest of that
  launch**, so nothing overwrites the file that couldn't be read. A match save
  that can't be read gets its own line in the same notice and is **removed**
  (Q4 a): **Continue** is absent, as before, and the notice does not repeat on
  the next launch. One notice per launch carries both failures. Enter, Space
  and Esc dismiss it to the untouched start menu; `q` and `m` still quit and
  mute as they do under every other modal (Q6, ruled by the person at sign-off
  and written into `spec.md`); nothing else acts. `KAAZAP_CRASH_AT`
  (`key`/`tick`/`draw`/`render`/`input`) ships in the binary as the only way
  to demonstrate a crash, following the `KAAZAP_DATA_DIR` idiom and
  deliberately **not** documented in `Readme.md`. No engine, AI, economy,
  wager, balance-data or dependency change, and no new screen or mode:
  `card.rs`, `game.rs`, `player.rs`, `opponent.rs`, `economy.rs`,
  `campaign.rs`, `wager.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock`
  are untouched, `PROFILE_VERSION` / `SAVE_VERSION` stay 1, and the version
  gates and every `#[serde(default)]` are unchanged. Driver walkthroughs
  against a scratch `KAAZAP_DATA_DIR` attested 33 crash-and-quit runs at 89×31
  after Phase 1 (the first pass found `=tick` and `=draw` garbling the report
  6/6, which is why the guard now joins the render thread before restoring)
  and all six data scenarios at 89×31 after Phase 4. `Readme.md`'s **Saved
  data** paragraph names the set-aside file.
```

### 1b. Amend the path-injection seam bullet's "Still open" follow-up (currently lines 732–744)

The bullet stays shipped; only its **Still open** sentence changes, because
this spec made that follow-up matter more. Replace:

```markdown
  nothing migrated). **Still open, the follow-up the seam exists for:** seven
  `app.rs` unit tests construct an `App` and so still read the real profile,
  settings and save — pointing them at a scratch root changes those tests'
  behaviour and was left outside the chore's footprint.
```

with:

```markdown
  nothing migrated). **Still open, the follow-up the seam exists for:** seven
  `app.rs` unit tests construct an `App` and so still read the real profile,
  settings and save — pointing them at a scratch root changes those tests'
  behaviour and was left outside the chore's footprint. **Spec 028 raised the
  stakes on it.** `App::new` no longer merely *reads* the real data folder, it
  repairs it: `Profile::load` **moves** an unreadable profile aside to a dated
  name, and `save::check_at_launch` **deletes** an unreadable match save. On
  healthy files nothing happens; on a machine whose data folder is damaged, a
  `cargo test` run now *changes* it — doing what the next launch would have
  done, so nothing recoverable is lost, but doing it from a test run rather
  than from the game. Three of those seven tests also call `draw` without
  overwriting `app.modal`, and since spec 028 `App::new` can set it, so on such
  a machine the launch notice would draw over the board and fail their row
  assertions (before spec 028 the modal was unconditionally `None` at
  construction). Related trap for anyone testing by hand: **never run
  `cargo test` from a shell with `KAAZAP_DATA_DIR` exported at a fixture
  directory** — it repairs the scenario you were about to test.
```

**Leave alone** every other roadmap bullet, including **Archive the last run
at reset** (713–719, a spec 028 non-goal, still open), the **Release
readiness** bullet, and the `profile.json` / `savegame.json` mentions inside
earlier Shipped entries.

---

## 2. `DECISIONS.md` — one edit

`grep -n -iE "crash|panic|whole-file|atomic|corrupt" DECISIONS.md` on `main`
returns two hits, neither a ruling this spec touches: **line 17**, the
project-intent paragraph wanting "no crashes/panics in normal play" (which
this spec serves rather than amends), and **line 876**, spec 022's balance
harness printing rather than panicking. Both stay as written.

### 2a. Append a new section at the end of the file

Append at the **end of the file** — `DECISIONS.md`'s last three lines are
currently the tail of the "## Chore: a path seam for the profile, match save
and settings (2026-09-19)" section:

```
- **Why a chore and not a spec.** One new module and three one-line call-site
  changes, with no engine, AI, save-format, balance-data or dependency change
  and no new screen or mode.
```

(The last line is unique in the file — `grep -c` reads 1 — and it is also the
end of the file, which is the anchor; the file ends with a newline.) Append
after it:

```markdown

## Crash & data safety (spec 028)

An audit found three ways kaazap failed badly, all confirmed in the code. The
terminal was restored in exactly one place, reachable only by `break
'gameloop` on `q`, so any panic or input error dropped the player into a shell
with no cursor and no echo. All three writers replaced a file in place with
`fs::write`, so an interrupted write destroyed the progress it was meant to
preserve. And `Profile::load` collapsed missing, unreadable, malformed and
wrong-version into `unwrap_or_default()`, after which the first `save()` wrote
a starter profile over the campaign it could not read — silently. This is a
repair spec: one new module (`src/crash.rs`), one shared function
(`paths::write_whole`), one modal variant, one enum and one struct on
`Profile`. No new screen, no new phase, no new crate, no engine, AI, economy
or balance change. Ruled by the person on 2026-09-19, with Q6 added at plan
sign-off the same day.

- **Q1 a — an unreadable profile is moved aside under a dated name, and the
  notice says where it went.** The alternatives were to leave it and refuse to
  save (nothing persists, and the player has to know that), or to leave it and
  let the next save overwrite it after a warning. Setting it aside is the only
  one where a recoverable campaign is actually recoverable, and it is the same
  instinct as the *archive the last run at reset* backlog item — which stays
  separate and still open.
- **Q2 a — the notice is a modal on the start menu, dismissed with a key.** A
  persistent line, and a banner that clears on the first navigation, were both
  offered. Losing a run deserves a stop, not a line the player scrolls past.
- **Q3 b — the notice says why.** "Couldn't be read" and "saved by a different
  version of kaazap" are separate messages, because the version case is the
  only one where the player can act, by finding the build that wrote it. The
  two internal cases the player *can't* act on differently — an I/O or
  permission failure and a malformed document — are deliberately one message
  (`ProfileProblem::Unreadable`).
- **Q4 a — a bad match save gets a notice too, but no file is kept.** Before
  this spec **Continue** simply disappeared with no explanation. The accepted
  consequence: the unreadable save is removed so the notice doesn't repeat
  every launch, so that one match is gone for good. A match is not a run.
- **Q5 a — the crash report is one kaazap line plus the panic's message and
  location**, rather than raw panic output alone, so the player knows it was
  kaazap and a bug report has something in it.
- **Q6 a — `q` and `m` still act while the notice is up** (the person,
  2026-09-19, at plan sign-off). The sign-off raised this as blocking rather
  than deciding it: the plan kept both keys live against the spec's original
  "no other key does anything while it is up". Both are handled before the
  modal chain — `q` in the game loop itself, before `App` sees the key; `m`
  ahead of modal routing since spec 004 — so suppressing them would have made
  this the one modal in the game you cannot quit or mute under, and would have
  meant the game loop asking `App` which modal is open. **`spec.md` was
  amended** in two places and carries a dated Q6 section naming them as the
  standing exception, so the acceptance criterion and the design now agree.
  Enter, Space and Esc dismiss; nothing else acts.

Design calls made during planning:

- **A panic hook that records, and a guard that restores and prints** (§1).
  Two mechanisms, neither doing its usual job. The **guard** (`TerminalGuard`
  in `main.rs`) is the only way to reach the restore on the non-panic paths —
  `q`, and an `Err` from `event::poll`/`event::read` leaving `main` through
  `?` — and its `Drop` also runs while a panic unwinds, so the restore has
  exactly one home and the three teardown lines left the end of `main`. The
  **hook** prints nothing: the default hook prints *before* unwinding reaches
  any guard, i.e. while the alternate screen is still up, and that output is
  thrown away when we leave it. So the hook records the first panic's message
  and location into a `OnceLock` in `crash.rs`, and the guard's `Drop` prints
  the report **after** the restore. Consequences, each deliberate: **first
  panic wins**, so a render-thread panic (the cause) is what gets reported and
  the `join().unwrap()` that surfaces it on the main thread (the symptom) is a
  no-op against an already-set `OnceLock`, and the process still exits 101;
  the hook is installed **inside `TerminalGuard::enter`, before raw mode**, so
  no window exists where the default hook prints onto the alternate screen
  with nothing recorded to reprint; `restore_terminal` swallows every error
  (`let _ = …`) rather than unwrapping, because a panic inside a `Drop` during
  unwinding aborts the process, and because restoring twice must change
  nothing. The whole design depends on `Drop` running: `Cargo.toml` sets no
  `panic = "abort"` in any profile (checked at planning and re-checked at
  T002), so a panic unwinds in release as well as debug. If that ever changes,
  the crash path has to move into the hook.
  - **The guard joins the render thread before restoring**, and that was not
    the first design. The plan shipped with one flag check at the top of the
    render loop (`if crash::report().is_some() { break; }`) and named a
    deterministic fallback in case it wasn't enough. The Phase 1 walkthrough
    found `KAAZAP_CRASH_AT=tick` and `=draw` garbled the crash report **6 runs
    out of 6** — the forced first render before the loop never reaches the
    check at all — so the fallback was taken (T002a): the guard carries
    `render: Option<JoinHandle<()>>` and joins it at the top of its `Drop`.
    The re-walkthrough was clean in all 33 runs.
  - **The join is only bounded because `render_tx` is declared *after* the
    guard.** Locals drop in reverse declaration order, so the channel sender
    goes first, the render thread's `recv` fails, and the thread ends — then
    the join returns. Hoisting the channel above the guard would deadlock
    every ending. Two comments in `main.rs` say so; it cannot be unit-tested,
    so it is checked at review and in the walkthrough.
  - **`main` keeps its own `join().unwrap()` on the `q` path**, taking the
    handle back out of the guard. That `unwrap`, not the guard's ignored
    `Err`, is what makes a panicked render thread exit 101.
  - **Scope: the endings `spec.md` enumerates.** `q`, an input error, and a
    panic on either thread. A `SIGTERM` or `kill -9` still leaves the terminal
    unrestored; a signal handler is not in this spec. That is a boundary, not
    a gap — but nothing should describe the guarantee as "however kaazap
    ends".
  - Rejected: `catch_unwind` around the loop (the spec's own non-goal — no
    recovery — and more code for the same teardown); printing from the hook
    (the report lands on the alternate screen); a guard owning the channel as
    well as the handle (the sender has to drop *before* the join, which
    reverse drop order already gives for free).
- **`KAAZAP_CRASH_AT` ships in the binary, and is deliberately undocumented**
  (§2). Four acceptance criteria are about what a crash looks like, and a
  shipped binary cannot otherwise panic on purpose. `main` reads the variable
  **once** before the loop; `key`, `tick`, `draw` and `render` panic at those
  points, and `input` returns `Err` from `main` at the same place
  `event::poll`'s `?` would — the no-panic path of AC 4. Nothing in the game
  sets it, and no key, file or menu reaches it. It matches the existing
  `KAAZAP_DATA_DIR` idiom, and it is **not** in `Readme.md`: it is a review
  and walkthrough seam, not a player feature.
- **One `write_whole` in `paths.rs`, a fixed temp name, and no `fsync`** (§3).
  `pub(crate) fn write_whole(path: &Path, contents: &str) -> bool` writes to
  `path.with_extension("tmp")` and `fs::rename`s it over the target. Stated no
  more strongly than it is true: the bytes are complete before anything
  replaces the file the game reads, so an interrupted write leaves the
  previous file byte-for-byte unchanged and never leaves a half-written file
  *under the real name*. On POSIX the replacement is atomic; on Windows
  `std::fs::rename` prefers `FileRenameInfoEx` and falls back to `MoveFileEx`,
  which Microsoft does not guarantee atomic in every case — safe either way
  for this purpose, with one Windows-only consequence: if another process
  holds the target open the rename fails and the save is a silent no-op, which
  is exactly the best-effort contract all three callers already had. The temp
  name is **fixed, not unique**, so repeated interrupted writes leave one
  piece of debris rather than a growing pile, and it is never `*.json`, so no
  loader looks at it. It lives in `paths.rs` — "where kaazap's files live"
  becomes "…and how they are written" — rather than in a module whose whole
  content would be one function, and it takes an explicit `&Path`, so its own
  unit tests need no data root and can't race the `OnceLock`.
  - **No `fsync`, and here is the reason, so a later power-cut question finds
    it.** `File::create` + `write_all` + `sync_all` would additionally survive
    a *machine* crash, but Rust's `sync_all` is `F_FULLFSYNC` on macOS — a
    full device flush — and the match save is written on **every state
    change**, i.e. effectively every key press. The acceptance criteria are
    about a write that fails partway, which rename alone covers completely;
    the power cut appears only in the spec's narrative. A durable version
    would also have to fsync the *directory*. This is one line to change if
    the person ever wants to pay for it; `write_whole`'s doc comment names
    `fsync` and cites plan §Design tension 3 so the trade is findable from the
    code.
  - **`write_whole` is a convention, not an enforced rule.** All three writers
    go through it, and nothing in the test suite would catch a fourth writer
    calling `fs::write` directly — or one built from `File::create` +
    `write_all`, or `serde_json::to_writer`. The greps in T009 are a one-shot
    check at merge, not a regression guard. **The next writer of a kaazap file
    goes through `paths::write_whole`.**
  - **Two kaazap processes writing at once is the one case the fixed temp name
    does not cover.** Both would write the same `<stem>.tmp` and an
    interleaving could splice them before either renames. This was bought
    deliberately: unique temp names would trade a rare two-instance corruption
    for certain debris accumulation across crashes. A known limit, not an
    oversight.
  - **Write-then-rename changes two things `fs::write` did not.** The file
    gets fresh permission bits at the default umask rather than keeping the
    ones it had, and a symlink at the target is replaced rather than written
    through. Neither matters for a game's saves; both are now inherited by
    every future writer.
- **The save suspension is a process-scoped flag, not a field** (§4). When an
  unreadable profile cannot be moved aside, nothing may overwrite it for the
  rest of the launch — and "the rest of that launch" *is* the process, so
  `static SAVES_SUSPENDED: AtomicBool` in `profile.rs`, set by `load` when the
  move fails and read as the first line of `save`. With twelve `save()` call
  sites the guard has to live behind `save` itself. Rejected: a
  `#[serde(skip)]` field on `Profile`, because `reset_to_starter` does `*self
  = Profile::default()`, which would silently clear the flag and let **Reset
  Everything** write over the very file the flag exists to protect; and a flag
  on `App`, because twelve call sites and every future one would have to
  remember it. The static is the same shape as `paths::ROOT`.
- **A dated name, hand-rolled, in UTC** (§6). `profile-YYYYMMDD-HHMMSS.json`,
  beside the file it replaces. The spec wants a name the player can find, and
  the toolbox has no date crate and is not getting one, so `profile.rs` gained
  about twenty pure lines: seconds since the epoch split into a day count and
  a time of day, plus the standard civil-from-days conversion (verified
  longhand against all three test vectors at review, including a pre-epoch
  one). **UTC**, because local time needs the platform's zone database, which
  needs a crate — so a player well east or west of UTC may see a name a day
  off their local clock. If the name is taken — two bad launches in the same
  second, or a hand-made copy — `-2` … `-9` follow; past that the move is not
  attempted, which lands in the suspended branch, the safe end. The caller
  takes the first name whose path doesn't exist, because `fs::rename` would
  otherwise *replace* an existing set-aside file. The integration test
  produced `profile-20260920-023255{,-2,-3}.json` with all three collisions in
  one second, so the numbered branch is what actually ran. Rejected:
  `profile-<unix-seconds>.json` (no date math, but unreadable to a player) and
  the file's own mtime (the same formatting problem).
- **Two keys still act under the notice, and the `?` ordering is now a
  comment** (§8). See Q6 above for the ruling. The plan's own claim — that the
  set is just `q` and `m` — was treated as a claim, not a reading: the Phase 4
  review **enumerated** the keys from the top of `handle_key` down and
  confirmed it (`q` in `main.rs` before `App`; `m` ahead of the modal ladder;
  Enter/Space/Esc in the arm; everything else a no-op, including `?`, `L`,
  `Q`, `M`, the arrows, the digits and Ctrl+P/N/B/F, which arrive as arrows).
  One ordering is load-bearing and invisible: **`?` is handled inside the
  `else` branch that runs only when no modal is open**, so it cannot reach the
  notice — but if it ever moved ahead of the modal chain, opening and closing
  help would set `self.modal = None` and the notice would be gone for good,
  with nothing to bring it back. That constraint was carried out of the review
  and into the code as a comment at the `?` site (T008a), so it outlives the
  review that found it. For the record, `m` under the notice **mutes
  silently** — it is not a no-op, it simply has nothing to draw.
- **The notice is a `Modal` carrying its lines** (§7). `Modal::DataNotice(
  Vec<String>)`, built once in `App::new` by a pure `data_notice_lines(profile,
  save_unreadable) -> Option<Vec<String>>` whose `None` *is* the "raise no
  modal" answer, so there is no separate predicate. The other notices rebuild
  their lines each draw because their content is live run state; this content
  is fixed at launch and its inputs are gone by the second frame, so it is
  carried and `App` gains no field. Dismissal reuses the existing
  `notice_dismissed` and `draw_notice`, so there is no new drawing code and no
  `resize` arm. One line of the plan's drafted wording was **replaced at T008**
  on a Phase 3 review finding: "nothing from this session is kept" was
  literally false, because only `Profile::save` no-ops under suspension while
  `save::save` and `Settings::save` still write. It now reads "kaazap won't
  save over it, so the campaign you play this session won't be kept."

Two things a future reader should know that have no other home:

- **`payload_as_str` pins the crate to Rust ≥ 1.91.** The panic hook uses
  `PanicHookInfo::payload_as_str`, stable from 1.91, and nothing in the tree
  says so — AC 17 forbids adding `rust-version` to `Cargo.toml`. The toolchain
  in use is 1.91.1. The two-arm `&str` / `String` downcast is the equivalent
  with no version floor if that ever bites.
- **`eprintln!` inside `TerminalGuard::drop` can panic on a closed stderr**
  (`kaazap 2>&1 | head -1`), and a panic inside a `Drop` during unwinding
  aborts the process. `restore_terminal` is unwrap-free by design; this is the
  one unguarded panic site left on that path. No acceptance criterion covers
  it, so it is recorded here rather than fixed under this spec.

Attested by driver walkthroughs at 89×31 with `KAAZAP_DATA_DIR` pointed at a
scratch directory throughout — the real profile, save and settings were never
in play. **Phase 1** (T002, then T002a after the fix): 33 runs — 3 quits plus
6 each of `=tick`, `=draw`, `=input`, `=key` and `=render` — all clean, the
cursor visible every time, exit codes 0 for the quit, 101 for each panic and 1
for the input error, with the `input` run printing `Error: kaazap couldn't
read the terminal` and **no panic text**. **Phase 4** (T008), ten runs: (a) a
clean launch showed no notice and created nothing; (b) a malformed profile
raised the notice naming the dated file, the kept bytes were identical, and
playing a **whole match** afterwards left the kept file md5-identical with
exactly one set-aside file and a fresh `profile.json`; (c) a `"version": 2`
profile gave the wrong-version wording; (d) a malformed match save gave its
line, no **Continue**, the file removed, and a **silent next launch**; (e)
both damaged at once gave **one** notice, box 73 wide, nothing clipped, and
the move-failed case gave the widest box — 87 columns with one column of
margin each side, exactly the review's arithmetic — with the unreadable file
still there unchanged; (f) `?`, `L`, `m`, an arrow, `3` and `x` left the
screen byte-identical with the menu selection unmoved, Enter, Space and Esc
each dismissed to the menu with the selection where it starts, and `q` quit
with exit 0 and a clean terminal.

No engine, AI, economy, wager, balance-data or dependency change: `card.rs`,
`game.rs`, `player.rs`, `opponent.rs`, `economy.rs`, `campaign.rs`,
`wager.rs`, `tests/balance.rs`, `Cargo.toml` and `Cargo.lock` are untouched;
`PROFILE_VERSION` and `SAVE_VERSION` both stay 1; the version comparisons and
every `#[serde(default)]` are unchanged — `Profile::from_json` became a
`#[cfg(test)]` wrapper over a new `classify(text) -> Result<Self,
ProfileProblem>` precisely so every existing profile test stands unedited and
the gate's behaviour is visibly the same; no new crate; the build has no
warnings, the same count as `main`. Monochrome by construction: the notice
reuses `draw_notice`'s existing emphasis levels and adds none.
```

---

## 3. Not drafted here (deliberately)

- **`Readme.md` is on the branch** — two sentences in the **Saved data**
  paragraph naming the set-aside file and the removed match save (T008). It
  rides into `main` with the merge and needs no close-out edit.
  `KAAZAP_CRASH_AT` is **not** in it, by design (plan §2, §Open questions 6).
- **`design/brief.md` is untouched** — the notice uses the existing modal
  vocabulary and adds no visual rule. The constitution's *acted-on element
  stands apart* rule was applied as written (an empty row above the dismiss
  line, the box's own even padding below) and is pinned by
  `the_data_notice_reads_right_breathes_and_fits_the_minimum_terminal`.
- **`docs/economy.md`, `docs/balance.md` and `docs/opponents.md` are
  untouched** — no number moved and no screen they describe changed.
- **No `CLAUDE.md` amendment.** The rendering pattern (`Frame`, the render
  thread) is unchanged — the guard joins that thread, it does not replace it;
  the `Screen`-versus-overlay line is unchanged, because the notice is a
  `Modal` over the start menu, which is what that line already covers;
  the draw-never-mutates boundary holds (the notice's lines are built in
  `App::new` and only read by `draw`); `GamePhase` is untouched; the
  verification command is unchanged; no new crate.
- **The *Archive the last run at reset* backlog bullet stays open**
  (`ROADMAP.md` 713–719). It is a named non-goal of this spec, and archiving a
  *good* profile at reset is a different feature from keeping an *unreadable*
  one at launch.
- **Reviewer notes left for the sweep**, all non-blocking, stay in
  `specs/028-crash-and-data-safety/tasks.md`'s tier log: the fixed scratch
  directory names in the new integration tests, a scratch directory leaked on
  a red run, no entry-set check after the failed saves in
  `whole_file_write.rs`, the three profile-loader assertions that sit behind
  the byte comparisons and can only be shown live by mutating a loader rather
  than a writer, and the plan's drifted line citations (§Design tension 8
  cites `app.rs:1169-1188`, now `1221-1235`). Process evidence, not project
  decisions.
- **Three `app.rs` test comments say "nothing here writes to disk"** and stay
  as written: they are true of what those tests *press*, and the thing that
  now touches disk is `App::new` itself, which the roadmap follow-up in §1b
  covers. Fixing them belongs with that follow-up, not here.

---

## 4. Mechanical checks (T009, run on the branch at `07f8500`, 2026-09-20)

`main` was at `cd015c5` for every comparison below. Nothing was applied to
`main` and no branch was switched: `main`'s content was read with
`git show main:<path>`, and `main` was built in a throwaway `git worktree`
with its own `CARGO_TARGET_DIR`.

### `cargo test -q`, three consecutive runs

`tail -n 25` now reaches only the last four of the **eleven** test binaries
(spec 028 adds four integration binaries, each deliberately one `#[test]` per
process because the data root resolves once). The tails, verbatim:

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
`running` / `test result` line — **455 + 6 + 7 = 468 passing, 0 failing, 1
ignored** (the long balance simulation, ignored since spec 022), identical
across the three runs:

```
$ for i in 1 2 3; do echo "=== run $i ==="; cargo test -q 2>&1 | grep -E "^(running|test result|error)"; done
=== run 1 ===
running 455 tests
test result: ok. 455 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 7 tests
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.01s
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
running 455 tests
test result: ok. 455 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 7 tests
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.00s
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
running 455 tests
test result: ok. 455 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 7 tests
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.01s
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

All three runs were made with `KAAZAP_DATA_DIR` **unset** (confirmed before
the loop), for the reason §1b now records in the roadmap.

### The branch's whole diff against `main` (three-dot, since `main` may move)

```
$ git diff main...HEAD --stat
 Readme.md                                |   6 +-
 specs/028-crash-and-data-safety/plan.md  | 974 +++++++++++++++++++++++++++++++
 specs/028-crash-and-data-safety/spec.md  | 310 ++++++++++
 specs/028-crash-and-data-safety/tasks.md | 747 ++++++++++++++++++++++++
 src/app.rs                               | 168 +++++-
 src/crash.rs                             | 122 ++++
 src/lib.rs                               |   1 +
 src/main.rs                              | 113 +++-
 src/paths.rs                             | 126 +++-
 src/profile.rs                           | 211 ++++++-
 src/save.rs                              |  20 +-
 src/settings.rs                          |   2 +-
 tests/match_save_recovery.rs             |  58 ++
 tests/profile_recovery.rs                | 195 +++++++
 tests/profile_save_suspended.rs          |  82 +++
 tests/whole_file_write.rs                | 137 +++++
 16 files changed, 3234 insertions(+), 38 deletions(-)
```

**None of the ten forbidden paths appears in that list**, and the explicit
query is empty:

```
$ git diff main...HEAD --stat -- src/card.rs src/game.rs src/player.rs src/opponent.rs src/economy.rs src/campaign.rs src/wager.rs tests/balance.rs Cargo.toml Cargo.lock
(empty, exit 0)
```

### The versions still read 1 and 1

```
$ grep -n "VERSION" src/save.rs src/profile.rs
src/profile.rs:35:const PROFILE_VERSION: u32 = 1;
src/profile.rs:96:    PROFILE_VERSION
src/profile.rs:140:            version: PROFILE_VERSION,
src/profile.rs:265:        (profile.version == PROFILE_VERSION)
src/profile.rs:623:            version: PROFILE_VERSION,
src/profile.rs:745:        assert_eq!(PROFILE_VERSION, 1, "the seed purse is no on-disk shape change");
src/profile.rs:773:        assert_eq!(PROFILE_VERSION, 1, "the seen-marks are no on-disk shape change");
src/profile.rs:937:        assert_eq!(PROFILE_VERSION, 1, "no version bump for the additive stats field");
src/profile.rs:1072:        assert_eq!(PROFILE_VERSION, 1, "the counters and the record are no shape change");
src/profile.rs:1354:        assert_eq!(PROFILE_VERSION, 1, "no save-format change in this spec");
src/profile.rs:1385:        val["version"] = serde_json::json!(PROFILE_VERSION + 1);
src/profile.rs:1396:        val["version"] = serde_json::json!(PROFILE_VERSION + 1);
src/save.rs:30:const SAVE_VERSION: u32 = 1;
src/save.rs:91:        version: SAVE_VERSION,
src/save.rs:171:    (saved.version == SAVE_VERSION).then(|| from_saved(saved))
src/save.rs:389:        val["version"] = serde_json::json!(SAVE_VERSION + 1);
```

`PROFILE_VERSION` and `SAVE_VERSION` are both `1`. The **comparisons** are
unchanged too: `src/save.rs:171` is byte-identical to `main`'s, and
`src/profile.rs:265` is `main`'s `(profile.version == PROFILE_VERSION)
.then_some(profile)` with `.ok_or(ProfileProblem::WrongVersion)` appended —
the same predicate, the same discard, now reporting *which* failure it was:

```
$ git show main:src/profile.rs | sed -n '204,206p'
    fn from_json(text: &str) -> Option<Self> {
        let profile: Profile = serde_json::from_str(text).ok()?;
        (profile.version == PROFILE_VERSION).then_some(profile)

$ sed -n '262,268p' src/profile.rs
    fn classify(text: &str) -> Result<Self, ProfileProblem> {
        let profile: Profile =
            serde_json::from_str(text).map_err(|_| ProfileProblem::Unreadable)?;
        (profile.version == PROFILE_VERSION)
            .then_some(profile)
            .ok_or(ProfileProblem::WrongVersion)
    }
```

The two extra `PROFILE_VERSION` hits relative to `main` (1354 and 1396) are
both inside `#[cfg(test)]`: the second is T005a's
`classify_tells_a_bad_document_from_a_bad_version`, which pins the variant
split the `from_json` wrapper would otherwise hide.

### Every `#[serde(default)]` matches `main` exactly

`grep -r` walks the three files in a different order in the two trees, so the
comparison is made on the matched text:

```
$ grep -rn "serde(default" src/profile.rs src/save.rs src/settings.rs
src/save.rs:45:    #[serde(default = "default_opponent_id")]
src/save.rs:53:    #[serde(default)]
src/save.rs:307:        // mimic a pre-roster save. serde(default) fills it, resolving to the
src/save.rs:336:        // A pre-deck-builder save lacks the field; serde(default) leaves it
src/profile.rs:59:/// field carries a `#[serde(default)]` so a partial or older file still loads
src/profile.rs:64:    #[serde(default = "default_version")]
src/profile.rs:66:    #[serde(default = "starter_collection")]
src/profile.rs:68:    #[serde(default = "starter_deck")]
src/profile.rs:72:    #[serde(default)]
src/profile.rs:79:    #[serde(default)]
src/profile.rs:83:    #[serde(default)]
src/profile.rs:89:    #[serde(default)]
src/profile.rs:91:    #[serde(default)]
src/settings.rs:14:/// fields carry `#[serde(default)]`: an older or partial settings file
src/settings.rs:19:    #[serde(default = "default_music_volume")]
src/settings.rs:21:    #[serde(default = "default_sfx_volume")]
src/settings.rs:25:    #[serde(default = "default_animations")]

$ for f in profile.rs save.rs settings.rs; do git show main:src/$f > /tmp/main-$f; done
$ diff <(grep -rn "serde(default" src/profile.rs src/save.rs src/settings.rs | sed 's/:[0-9]*:/:/' | sort) \
       <(for f in profile.rs save.rs settings.rs; do grep -n "serde(default" /tmp/main-$f | sed "s|^[0-9]*:|src/$f:|"; done | sort)
(empty — identical: the same 17 lines, the same text, in the same files)

$ grep -c "serde(default" on each file — branch: profile.rs 9, save.rs 4, settings.rs 4
                                          main: profile.rs 9, save.rs 4, settings.rs 4
```

Only the line numbers moved, and only in `profile.rs` and `save.rs`, where
code was added above them.

### `fs::write` is now confined to `paths.rs`

```
$ grep -rn "fs::write" src/
src/paths.rs:81:    if fs::write(&tmp, contents).is_err() || fs::rename(&tmp, path).is_err() {
src/paths.rs:142:        fs::write(&path, "the previous file").expect("previous file");
src/paths.rs:161:        fs::write(&path, "the previous file").expect("previous file");
```

Three matches, not one. **Line 81 is the only writer** — it is the body of
`write_whole` itself, writing the temp file. **Lines 142 and 161 are test
fixtures** inside `paths.rs`'s own `#[cfg(test)] mod tests`: they lay down a
"previous file" for `a_failed_write_leaves_the_previous_file_untouched` and
`repeated_failed_writes_do_not_accumulate` to prove is left byte-for-byte
alone. Neither writes a kaazap data file. No profile, settings or match-save
writer calls `fs::write` any more.

### The widened footprint grep the Phase 2 review asked for (N6)

The `fs::write` grep alone would not catch a writer built any other way, so:

```
$ grep -rnE "File::create|to_writer|write_all" src/
(empty, exit 1)
```

Nothing in `src/` creates a file handle or streams into one. Together with the
grep above, **`paths::write_whole` is the only path by which `src/` puts bytes
into a file.** Both greps are a one-shot check at merge, not a regression
guard — which is why the DECISIONS entry states the convention in words for
the next writer.

### Warning count equals `main`'s

Method, so it can be repeated: `main` was checked out into a throwaway
worktree (`git worktree add`) — the branch was never switched — and both trees
were built with `cargo build --all-targets` into their **own empty**
`CARGO_TARGET_DIR`, so neither replayed a cached diagnostic and neither
disturbed the other's `target/`.

```
$ git worktree add <scratch>/main-wt main
$ (cd <scratch>/main-wt && CARGO_TARGET_DIR=<scratch>/main-target cargo build --all-targets) > main-build.txt 2>&1
$ CARGO_TARGET_DIR=<scratch>/branch-target cargo build --all-targets > branch-build.txt 2>&1

$ grep -c warning main-build.txt
0
$ grep -c warning branch-build.txt
0
```

Both finished successfully (`Finished "dev" profile … in 17.74s` on `main`,
`… in 14.53s` on the branch) and neither emitted a single line containing the
word "warning". **0 on `main`, 0 on the branch — equal.** The worktree is
temporary and is removed after the check; nothing about it reaches `main`.

---

## 5. `spec.md`'s 17 acceptance criteria, checked off with evidence

- [x] **`q` leaves the terminal exactly as today** — Phase 1 walkthrough
  (T002, then T002a): three quit runs, the shell prompt returned, typed
  characters echoed, the cursor was visible, the pre-launch scrollback was
  intact, nothing was printed, `echo $?` was **0**. Re-confirmed in the Phase 4
  walkthrough (f). Nothing is printed because the guard prints only when
  `crash::report()` is `Some`.
- [x] **A panic in key handling, the per-frame update, or drawing leaves a
  usable terminal, then the kaazap line, the panic's message and location, and
  an unsuccessful exit** — Phase 1 re-walkthrough (T002a): six runs each of
  `KAAZAP_CRASH_AT=key`, `=tick` and `=draw`, **all clean**, cursor visible,
  report readable, exit **101**. The first walkthrough (T002) found `=tick` and
  `=draw` garbled 6/6 by the render thread; that is what T002a's
  join-before-restore fixed, and the fix was verified by the 33-run
  re-walkthrough, not argued.
- [x] **A render-thread panic, surfacing at the shutdown join, does the same**
  — Phase 1 re-walkthrough: six `=render` runs, press `q` to reach the join,
  clean report, exit **101**. The first-panic-wins `OnceLock` makes the
  reported panic the render thread's own, not the `join().unwrap()` symptom.
- [x] **An input error does the same, with no panic text** — Phase 1
  re-walkthrough: six `=input` runs, exit **1**, `Error: kaazap couldn't read
  the terminal`, **no panic text** (no panic fires, so nothing is recorded and
  the guard prints nothing). One honest limit: the `.context("kaazap couldn't
  read the terminal")` on the real `event::poll`/`event::read` is **never
  executed** by any test or by the walkthrough — the seam bails before `poll` —
  so the colon-detail form (`… : <io error>`) has been reasoned about but not
  observed.
- [x] **Restoring twice is harmless** — `crash::restore_terminal` swallows
  every error rather than unwrapping, and
  `src/crash.rs:116 restoring_the_terminal_twice_is_harmless` calls it twice
  and asserts nothing errors and nothing is emitted. Also exercised in every
  crash run, where the guard's restore follows the hook's recording.
- [x] **All three writers: a partway failure leaves the previous file
  byte-for-byte unchanged; a completed write lands in full** — pinned against a
  scratch root, never the real one:
  `tests/whole_file_write.rs::every_writer_lands_whole_and_a_failed_save_leaves_the_previous_file`
  (its own process, root under `temp_dir()`, removed after), plus
  `src/paths.rs` unit tests `write_whole_replaces_the_file_and_leaves_no_debris`
  (125) and `a_failed_write_leaves_the_previous_file_untouched` (139). **The
  assertions were shown to bite** (T004a): reverting each of the three writers
  to `fs::write` in turn fires *that writer's own* byte comparison, at lines
  108, 109 and 110 respectively.
- [x] **Debris is never loaded and never accumulates** —
  `repeated_failed_writes_do_not_accumulate` (`src/paths.rs:158`) and
  `a_failed_rename_cleans_up_its_temp_file` (175, added at T003a because the
  original accumulation test was vacuous — the temp-path-as-directory trick
  failed before `fs::rename`, so the cleanup line never ran; T003a demonstrated
  the new test failing with that line commented out —
  `the temp file was left behind: ["data.json", "data.tmp"]` — and passing with
  it restored). The *never loaded* half is pinned rather than argued at T004a:
  a real `settings.tmp` holding garbage is neither read nor consumed by
  `Settings::load`. Structurally, the temp name is fixed and is never `*.json`,
  so no loader looks at it and one write's debris is the next write's scratch.
- [x] **No profile file and no data directory is silent, with a starter
  profile** — `Profile::load` returns `(Self::default(), None)` on both
  `Self::path() == None` and `ErrorKind::NotFound`, before anything is moved or
  reported; `data_notice_lines(None, false)` is `None`, so no modal is raised.
  Attested by Phase 4 walkthrough (a): everything deleted, launch, no notice, a
  normal start menu, a fresh campaign, nothing created.
- [x] **Unreadable / malformed / wrong-version each set the file aside, keep
  its bytes, show the reason and the name, and continue on a starter profile** —
  `tests/profile_recovery.rs::a_profile_that_cannot_be_read_is_kept_reported_and_played_past`
  covers all three (the permissions case Unix-only, per plan §Open questions 5)
  and produced real names `profile-20260920-023255`, `-2` and `-3` — all three
  collided inside one second, so the numbered branch is what actually ran.
  Walkthrough (b) and (c): the notice named the dated file, the kept bytes were
  identical, and the `"version": 2` profile gave the wrong-version wording.
- [x] **Playing on afterwards writes neither over the set-aside file nor a
  second copy** — same integration test, and Phase 4 walkthrough (b), where a
  **whole match** was played after the notice: the kept file stayed
  md5-identical, exactly one set-aside file existed, and a fresh `profile.json`
  appeared beside it.
- [x] **When the move itself fails: the notice says so, the file is still there
  unchanged at the end of the session, and no profile save wrote anything** —
  `tests/profile_save_suspended.rs::a_profile_that_cannot_be_moved_aside_stops_every_save_for_the_launch`,
  its own process because `SAVES_SUSPENDED` is sticky. **Mutation-checked**
  (T007a): commenting out the suspension guard fails it at line 74 —
  `a save wrote over the file that couldn't be read` — and the property was
  *observed*, not argued, because the passing re-run happened on the scratch
  root the failing run left behind and needed no human `chmod`. T007a also
  moved the chmod-back ahead of the assertions so a suspended save is
  distinguishable from a permission-denied one. Walkthrough: the move-failed
  notice at 89×31, box 87 wide with one column of margin each side, file still
  present and unchanged.
- [x] **An unreadable/malformed/wrong-version match save gets its line, has no
  Continue, and does not repeat next launch** —
  `tests/match_save_recovery.rs::an_unreadable_match_save_is_reported_once_and_removed`;
  its wrong-version step edits a **real** save's version rather than using a
  `{"version": 2}` stub, which would have failed serde's missing-field check
  and passed even with the version gate deleted (T007). Walkthrough (d):
  the line, no **Continue**, the file gone, and a **silent next launch**.
- [x] **Both unreadable → one notice carrying both** — `data_notice_lines`
  appends both sections to a single `Vec` and returns one `Modal::DataNotice`;
  `src/app.rs:2703
  the_data_notice_reads_right_breathes_and_fits_the_minimum_terminal` exercises
  the both-at-once content. Walkthrough (e): **one** notice, box 73 wide,
  nothing clipped.
- [x] **Fits 89×31 at its longest, breathes around the dismiss line, pads
  evenly; Enter/Space/Esc dismiss; nothing else acts but `q` and `m` (Q6)** —
  the app test above runs at both `fit_sizes()` and pins the content, the
  breathing room and the fit; the Phase 4 review worked the fit longhand
  (worst line **79 chars → an 87-wide box at 89 columns**, 9 content lines →
  13 rows at 31 — **two characters of headroom**, recorded as a known tight
  budget) and **enumerated** the keys that reach an open modal from the top of
  `handle_key` down rather than taking `q`/`m` on trust: everything else is a
  no-op, including `?`, `L`, `Q`, `M`, the arrows, the digits and
  Ctrl+P/N/B/F. Walkthrough (f): `?`, `L`, `m`, an arrow, `3` and `x` left the
  screen byte-identical with the menu selection unmoved; Enter, Space and Esc
  each dismissed to the menu with the selection where it starts; `q` quit with
  exit 0. Note for accuracy: **`m` mutes silently** — it is not a no-op, it
  simply has nothing to draw.
- [x] **A settings file that can't be read is still silent and still falls back
  to defaults** — `src/settings.rs:277
  settings_malformed_or_empty_json_falls_back_to_default` (unchanged by this
  spec), and structurally: `data_notice_lines` takes only a profile failure and
  a match-save flag, so no settings outcome can raise the notice. `Settings`
  changed by exactly one line, its `save` routed through `write_whole`.
- [x] **`cargo build --all-targets` and `cargo test` green, and the driver
  walkthrough shows a normal quit, a crash, a corrupted profile, a corrupted
  save, and both at once** — §4 above (three green runs, 468 passing / 0
  failing / 1 ignored; both builds clean with 0 warnings), and the Phase 4
  walkthrough's ten runs covering scenarios (a)–(f), with a scratch
  `KAAZAP_DATA_DIR` throughout and the real data folder never in play.
- [x] **No change to the ten named files; both versions stay 1; the version
  checks and every `#[serde(default)]` unchanged** — §4 above: the three-dot
  `--stat` lists none of them and the explicit query is empty; `PROFILE_VERSION`
  and `SAVE_VERSION` both read 1; `save.rs`'s gate is byte-identical and
  `profile.rs`'s is the same predicate with the failure reason appended; the 17
  `serde(default)` matches are identical to `main`'s, line for line, after
  sorting away `grep -r`'s file order.
