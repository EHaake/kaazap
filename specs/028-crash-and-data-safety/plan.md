# Plan: Crash & data safety — spec 028

> **Status**: Final — signed off 2026-09-19 (`skeptical-reviewer`, one review
> and one re-review; B1 and B2 fixed, B3 ruled by the person as Q6, S1–S8
> folded in)
**Implements**: `spec.md` in this directory

## Context

Three failures, found by audit and confirmed in the code: the terminal is
restored in exactly one place (`main.rs:124-126`, reachable only by `break
'gameloop` on `q`); all three writers replace a file in place with `fs::write`;
and `Profile::load` collapses missing, unreadable, malformed and wrong-version
into `unwrap_or_default()`, after which the first `save()` writes the starter
over the file it could not read.

This is a **repair spec, not a refactor**. It adds one small module
(`src/crash.rs`), one shared function (`paths::write_whole`), one modal variant
and one enum + one struct on `Profile`. It adds no screen, no phase, no crate,
and changes no engine, AI, economy or balance code. The version gates
(`PROFILE_VERSION` / `SAVE_VERSION`, both 1) and every `#[serde(default)]` stay
exactly as they are — the gate's *comparison* is untouched; it only reports
which of the two failures it was.

## What the code already gives us

- **One root for all three files.** `paths::data_dir()` / `config_dir()`
  (chore 2026-09-19), resolved once per process through a `OnceLock`.
  `Profile::path`, `Settings::config_path` and `save::save_path` all go through
  it, so a scratch root is one `set_root` call — but only from a process that
  has resolved nothing yet, which is why the disk-level tests below are
  integration tests with **one `#[test]` per binary**
  (`tests/paths_override.rs` is the shipped example of that discipline).
- **Filesystem-free parsing cores.** `Profile::from_json`, `save::from_json`
  and `Settings::from_json_or_default` already isolate parsing from disk; they
  stay the seam for everything that can be unit-tested.
- **A dismiss-only notice already exists, twice.** `Modal::RunOver` and
  `Modal::Victory` build a `Vec<String>` from a pure free function
  (`run_over_notice_lines`, `victory_notice_lines`) and hand it to
  `App::draw_notice`, which sizes an `OverlayLayout`, clears, draws the box,
  and draws row 0 `Strong`, the last row `Muted`, the rest `Normal`. Dismissal
  is the shared `notice_dismissed(key)` (Enter / Space / Esc). A new notice is
  a `Modal` variant, a lines function and two three-line arms — no new drawing
  code, no new screen. `App::resize` needs no arm: only `Modal::Help` caches a
  sized `Overlay`; everything else rebuilds from `self.config` each draw.
- **Even padding is `OverlayLayout`'s job.** `content_height + V_PAD` with
  `inner` inset `V_PAD / 2` top and bottom gives the one empty row above and
  below, and the existing
  `onboarding_texts_breathe_only_around_the_dismiss_line` /
  `both_notices_read_right_breathe_and_fit_the_minimum_terminal` tests are the
  pattern for pinning the *acted-on element stands apart* rule on content.
- **`Profile::load` has exactly one caller in `src/`** (`App::new:623`), and so
  does `save::exists` (`App::new:624`). Changing `load`'s signature costs one
  line there — **plus every integration test that calls it**, including the one
  this spec adds at T004 (`tests/whole_file_write.rs`), which is why T005's
  footprint names that file too. `Profile::save` has **twelve** callers, so the
  suspension must live behind `save` itself, not at the call sites.
- **An env-var seam is already the project's idiom** for aiming a run at
  something unusual: `KAAZAP_DATA_DIR` (chore 2026-09-19, documented in
  `Readme.md`). The crash seam below follows it.
- **The render thread is the only other writer to stdout**, fed by a
  `sync_channel(1)` whose sender lives in `main`. When the sender drops, the
  thread's `recv` fails and the thread ends.

## Design tensions resolved

### 1. A panic hook that *captures*, and a guard that restores and prints

Both mechanisms are needed, but not for the jobs they're usually given.

- A **guard** (`TerminalGuard` in `main.rs`) is the only way to reach the
  restore on the non-panic paths: `q`, and an `Err` from
  `event::poll`/`event::read` leaving `main` through `?`. `Drop` runs in all
  three cases and while a panic unwinds, so the restore has exactly one home
  and the three teardown lines leave the end of `main`.
- A **panic hook** is the only way to see the panic's message at all: the
  default hook prints it *before* unwinding reaches any guard — i.e. while the
  alternate screen is still up — and that output is thrown away when we leave
  it. So the hook **prints nothing**; it records the first panic's message and
  location into a `OnceLock` in `crash.rs`, and the guard's `Drop` prints the
  report **after** the restore, on the terminal the player came from.

Consequences, each deliberate:

- **First panic wins.** The render thread's panic is the original cause;
  `render_handle.join().unwrap()` then panics on the main thread at shutdown,
  its hook call is a no-op against an already-set `OnceLock`, and the guard
  prints the render thread's report. `.unwrap()` stays exactly as it is, and
  the process exits 101 — spec AC 3 with no new code.
- **The guard is declared first in `main`, so it drops last.** Locals drop in
  reverse declaration order, so `render_tx` (and with it the render thread)
  goes before the terminal is restored. This ordering is load-bearing and
  cannot be unit-tested; it carries a comment saying why, and is checked at
  review and in the walkthrough.
- **The render thread stops when a crash is being reported**: one line at the
  top of its loop, `if crash::report().is_some() { break; }`, so a frame queued
  at the moment of the panic can't be drawn over the report. The hook sets the
  report before unwinding begins, so the flag is up before the sender drops.
  **This narrows the window, it does not close it**, and the gap is structural
  rather than rare: the **forced first render before the loop** never reaches
  that check at all, and `tick`, `draw` and `input` all fire on the first
  game-loop iteration, microseconds after `thread::spawn` returns. So a frame
  is drawn while the main thread is running `LeaveAlternateScreen` and
  `eprintln!`, and nothing joins the render thread on the panic path. The
  visible consequence is not a hidden cursor — `render.rs` emits no per-frame
  `Hide`, so AC 2's cursor half holds either way — but the render thread's
  `MoveTo` sequences landing on the real terminal, garbling the crash report.
  **Fallback if it isn't**: give the guard the `JoinHandle` and have its `Drop`
  `join` it (ignoring an `Err` — the hook already recorded it) before
  restoring. That is deterministic and costs one field.
  **The Phase 1 walkthrough took the fallback (T002a, 2026-09-19)**: six runs
  each of `KAAZAP_CRASH_AT=tick` and `=draw` were garbled every time, so
  `TerminalGuard` now carries `render: Option<JoinHandle<()>>`, set just after
  the spawn, and joins it at the top of its `Drop`. The sender stays in `main`:
  `render_tx` is declared after the guard, so reverse drop order disconnects
  the thread first and the join is bounded. `main`'s own
  `join().unwrap()` on the `q` path stays as it is — it takes the handle back
  out of the guard — because that `unwrap`, not the guard's ignored `Err`, is
  what makes a panicked render thread exit 101 (AC 3).
- **Restoring twice is harmless** because `crash::restore_terminal()` ignores
  every error (`let _ = …`) rather than unwrapping — which it must anyway: a
  panic inside `Drop` during unwinding aborts the process.
- The hook is installed **inside** `TerminalGuard::enter`, immediately after
  `let guard = Self;` and **before** `enable_raw_mode`. (An earlier draft put
  it after the three terminal commands; that leaves a window — between
  `EnterAlternateScreen` and the install — where the default hook prints onto
  the alternate screen and the output is discarded with nothing recorded to
  reprint. Installing first is strictly safer and no more code: the guard
  already exists, so anything recorded gets printed after the restore.)
- **`Drop` runs**: `Cargo.toml` sets no `panic = "abort"` in any profile
  (checked), so a panic unwinds in release as well as debug and the guard's
  `Drop` is reached. If that ever changes, this whole design stops working and
  the crash path has to move into the hook.
- **Scope: the endings `spec.md` enumerates.** `q`, an input error, and a panic
  on either thread. A `SIGTERM` or `kill -9` still leaves the terminal
  unrestored — a signal handler is not in this spec, and the driver's
  kill-to-leave-a-save trick will show it. That is not a gap against the spec,
  but the close-out must not claim "however kaazap ends" more broadly than the
  spec's own list (T009).

Rejected: `catch_unwind` around the loop (the spec's non-goal — no recovery —
and more code for the same teardown); printing from the hook and restoring in
the guard (the report lands on the alternate screen); a guard that also owns
the channel as well as the handle (the sender has to drop *before* the join
for it to terminate, which reverse drop order already gives for free).

### 2. `KAAZAP_CRASH_AT` — the only way to demonstrate a crash

The shipped binary cannot panic on purpose, and four acceptance criteria are
about what a crash looks like. `main` reads `KAAZAP_CRASH_AT` **once** before
the loop; `key`, `tick`, `draw` and `render` panic at those points, and `input`
returns `Err` from `main` at the same place `event::poll`'s `?` would — the
no-panic path of AC 4. Nothing in the game sets the variable, no key or file
reaches it, and it matches the existing `KAAZAP_DATA_DIR` idiom. It is
deliberately **not** documented in `Readme.md`: it is a review and walkthrough
seam, not a player feature. Recorded in the close-out's DECISIONS entry.

### 3. The whole-file write is one function in `paths.rs`, used three times

```rust
pub(crate) fn write_whole(path: &Path, contents: &str) -> bool
```

It writes to a fixed sibling temp path (`path.with_extension("tmp")`), then
`fs::rename`s it over the target. What that buys, stated no more strongly than
it is true: the bytes are complete before anything replaces the file the game
reads, so an interrupted write leaves the previous file byte-for-byte unchanged
and never leaves a half-written file *under the real name*. On POSIX the
replacement is atomic; on Windows `std::fs::rename` prefers `FileRenameInfoEx`
(POSIX semantics) and falls back to `MoveFileEx`, which Microsoft does not
guarantee to be atomic in every case — safe either way for this purpose. One
Windows-only consequence: if the target file is open in another process the
rename fails, so the save is a silent no-op, which is exactly the best-effort
contract all three callers already have. The temp name is **fixed, not
unique**: the next write reuses it, so
repeated interrupted writes leave exactly one piece of debris rather than a
growing pile, and it is never named `*.json`, so no loader looks at it. On any
failure the temp file is removed (best-effort) and `false` comes back.

It lives in `paths.rs` — "where kaazap's files live" becomes "…and how they are
written" — rather than in a new module whose whole content would be one
function. It is `pub(crate)`: all three callers are in-crate, and taking an
explicit `&Path` means its own unit tests need no data root at all, so they
live in `src/paths.rs` and can't race the `OnceLock`.

**No `fsync`, stated deliberately.** `File::create` + `write_all` + `sync_all`
would additionally survive a power cut, but Rust's `sync_all` is
`F_FULLFSYNC` on macOS — a full device flush — and the match save is written on
*every* state change, i.e. every key press. The acceptance criteria are about a
write that fails partway (rename alone covers that completely); the power cut
appears only in the spec's narrative. A durable version would also have to
fsync the directory. One line to change if the person wants it (§Open
questions 1).

### 4. The suspension is a process flag, because "this launch" is the process

When the unreadable profile cannot be moved aside, nothing may overwrite it for
the rest of the launch. With twelve `save()` call sites, the guard belongs in
`save` itself:

```rust
static SAVES_SUSPENDED: AtomicBool = AtomicBool::new(false); // profile.rs
```

set by `load` when the move fails, read as the first line of `save`. Rejected:
a `#[serde(skip)]` field on `Profile` — `reset_to_starter` does `*self =
Profile::default()`, which would silently clear it and let Reset Everything
write over the very file the flag exists to protect; a flag on `App` — twelve
call sites, and every future one, would have to remember it. A process-scoped
static matches "the rest of that launch" exactly, and is the same shape as
`paths::ROOT`.

### 5. What `load` carries back: two variants and one optional name

```rust
pub enum ProfileProblem { Unreadable, WrongVersion }
pub struct ProfileFailure { pub problem: ProfileProblem, pub set_aside: Option<String> }
pub fn load() -> (Self, Option<ProfileFailure>)
```

`Unreadable` covers both "couldn't be read" (I/O, permissions, non-UTF-8) and
"couldn't be parsed": the player can do nothing different about either, and
spec Q3 b separates only the version case. `set_aside: Some(name)` is the name
the old file now has; `None` means the move failed — which is also exactly the
condition under which saves are suspended, so there is one source of truth and
no third field. The match save needs no enum at all: spec Q4 a gives it a
single line with no reason, so `save::check_at_launch() -> bool`.

The version gate keeps its comparison and its discard. `from_json` becomes a
`#[cfg(test)]` wrapper over a new `classify(text) -> Result<Self,
ProfileProblem>`, so every existing profile test stands unedited and the gate's
behaviour is visibly unchanged.

### 6. A dated name, hand-rolled, in UTC

`profile-YYYYMMDD-HHMMSS.json`, beside the file it replaces. The spec requires
a dated name the player can find, and the toolbox has no date crate, so
`profile.rs` gets ~20 pure lines: seconds since the epoch split into a day
count and a time of day, and the standard civil-from-days conversion. **UTC**,
because local time needs the platform's zone database, which needs a crate.
Rejected: `profile-<unix-seconds>.json` (no date math, but the player can't
read it), and the file's own mtime (same formatting problem).

If the name is taken — two bad launches in the same second, or a hand-made
copy — `-2` … `-9` follow; past that the move is not attempted, which lands in
the suspended branch, the safe end. `set_aside_names(stamp) -> Vec<String>` is
pure and tested; the caller takes the first whose path doesn't exist, because
`fs::rename` would otherwise *replace* an existing set-aside file.

### 7. The notice is a `Modal` carrying its lines

`Modal::DataNotice(Vec<String>)`. The lines are built once in `App::new` by a
pure `data_notice_lines(profile: Option<&ProfileFailure>, save_unreadable:
bool) -> Option<Vec<String>>` — `None` when nothing went wrong, which is also
the "raise no modal" answer, so there is no separate "is anything wrong"
predicate. The other notices rebuild their lines each draw because their
content is live run state; this content is fixed at launch and its inputs are
gone by the second frame, so it is carried, and `App` gains no field.

The notice is transient by construction: it lives only in `self.modal`, is
never saved, and dismissing it sets `self.modal = None` over the untouched
start menu — "the selection where it starts" is `App::new`'s default
`MenuState`, which the notice never touched.

### 8. Two keys still act under the notice, as under every other modal

`q` (handled in the game loop, before `App` sees it) and `m` (global mute,
handled before modal routing since spec 004) act under `Modal::Victory`,
`Modal::RunOver` and the onboarding pieces today. The data notice does not
change that. A literal reading of the acceptance criterion "no other key does
anything while it is up" would require an exception for this modal alone;
this plan keeps the existing behaviour and **flags it** (§Open questions 2)
rather than making the notice the one modal you can't quit under.

**Ruled by the person, 2026-09-19, at sign-off (Q6): `q` and `m` stay live.**
`spec.md` is amended in two places and carries a dated Q6 amendment section
naming them as the standing exception, so the criterion and this design now
agree. The plan is unchanged by the ruling; nothing else depended on it.

And the set is a claim, not a reading: the Phase 4 review must **enumerate**
which keys actually reach an open modal, from the top of `handle_key` down, and
not take "`q` and `m`" on trust. The one that would matter most is `?`: it is
handled today inside the `else` branch that runs only when **no** modal is open
(`app.rs:1169-1188`), so it cannot reach the notice — but if that ever moved
ahead of the modal chain, opening and closing help would set `self.modal =
None` and the notice would be gone for good, with nothing to bring it back.

### 9. What `cargo test` can now do to a real profile

Seven `app.rs` unit tests construct an `App`, and `App::new` calls
`Profile::load` — which, from this spec on, *moves a file* when the real
profile can't be read — and (from T007) `save::check_at_launch`, which
**deletes** the real match save when it can't be read. On healthy files nothing
happens. On an already broken profile, a test run renames it to the dated name
instead of the game doing it at the next launch; the file is kept either way,
which is the whole point of the move. On an already broken match save, a test
run removes it — the same removal the next launch would do, and nothing
recoverable is lost, since a save that can't be read is a save that can't be
resumed. Both are worth knowing before someone runs `cargo test` on a machine
with a damaged profile folder.
Fixing this properly is the open `ROADMAP.md` follow-up (point those
seven tests at a scratch root), explicitly outside this spec's footprint. This
plan therefore **adds no new unit test that constructs an `App`** — every new
disk-level check is an integration test with its own root, and every new pure
check is a free function.

## Design

### 1. `src/crash.rs` (new) + `src/lib.rs`

```rust
//! How kaazap ends (spec 028): the terminal restore, and the crash report.
//!
//! The panic hook here *records* and prints nothing — at the moment a panic
//! fires, the alternate screen is still up and anything printed on it is
//! discarded when we leave it. `main`'s terminal guard restores the terminal
//! first and prints the recorded report after, on the terminal the player
//! came from.

use std::{io, panic::{self, PanicHookInfo}, sync::OnceLock};
use crossterm::{ExecutableCommand, cursor::Show, terminal::{self, LeaveAlternateScreen}};

/// The first panic of this run. First wins: a panic on the render thread is
/// the cause, and the `join().unwrap()` that surfaces it on the main thread
/// at shutdown is only the symptom.
static REPORT: OnceLock<CrashReport> = OnceLock::new();

/// A recorded panic: what it said, and where it happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReport { pub message: String, pub location: Option<String> }

/// Record every panic, print nothing. Installed by the terminal guard as its
/// first act, before raw mode — the guard exists by then, so anything recorded
/// is printed after the restore, and no window is left where a panic prints
/// onto the alternate screen and is thrown away with it.
pub fn install_hook() {
    panic::set_hook(Box::new(|info: &PanicHookInfo| {
        // `payload_as_str` is stable from 1.91 and the crate pins no
        // rust-version; the two-arm `&str` / `String` downcast is the
        // equivalent with no version floor (T001).
        let _ = REPORT.set(CrashReport {
            message: info.payload_as_str().unwrap_or("(no message)").to_string(),
            location: info.location().map(|l| l.to_string()),
        });
    }));
}

/// The recorded panic, if this run crashed.
pub fn report() -> Option<&'static CrashReport> { REPORT.get() }

/// Put the terminal back the way it was found: cursor shown, alternate screen
/// left, raw mode off. Every error is swallowed — this runs from a `Drop`
/// during unwinding, where a panic would abort the process, and running it
/// twice must change nothing.
pub fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = stdout.execute(Show);
    let _ = stdout.execute(LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

/// What a crash prints on the restored terminal: one kaazap line, then the
/// panic's own message and location (spec Q5 a). Pure, so the wording is
/// tested without crashing anything.
pub fn crash_lines(report: &CrashReport) -> Vec<String> {
    let mut lines = vec![
        "kaazap crashed — this is a bug in the game, not something you did.".to_string(),
        format!("  {}", report.message),
    ];
    if let Some(at) = &report.location { lines.push(format!("  at {at}")); }
    lines
}
```

`lib.rs` gains `pub mod crash;` beside `pub mod paths;`.

### 2. `src/main.rs`

```rust
fn main() -> anyhow::Result<()> {
    let mut config = Config::from_terminal()?;

    // Declared first, so it is dropped *last*: locals drop in reverse order,
    // so the render thread's sender (and with it the render thread) is gone
    // before the terminal is restored and any crash report printed on it.
    let _terminal = TerminalGuard::enter()?;

    // The deliberate-crash seam (spec 028); `None` on every ordinary run.
    let crash_at = std::env::var("KAAZAP_CRASH_AT").ok();

    let mut app = App::new(config.clone());
    …
    let render_crash_at = crash_at.clone();
    let render_handle = thread::spawn(move || {
        …
        while let Ok(mut curr_frame) = render_rx.recv() {
            // A crash is being reported on the real terminal — stop drawing.
            if crash::report().is_some() { break; }
            crash_if_requested(&render_crash_at, "render");
            …
        }
    });

    'gameloop: loop {
        if crash_at.as_deref() == Some("input") {
            // The `?` path below, forced: an input error leaves main through
            // a return, restoring the terminal on the way (spec 028 AC 4).
            anyhow::bail!("kaazap couldn't read the terminal (KAAZAP_CRASH_AT=input)");
        }
        // `.context` so the line the player sees names kaazap, like the crash
        // line does — an input error is not a panic and records no report.
        if event::poll(…).context("kaazap couldn't read the terminal")? {
            match event::read().context("kaazap couldn't read the terminal")? {
                Event::Key(key_event) => {
                    …
                    crash_if_requested(&crash_at, "key");
                    app.handle_key(code)
                }
                … // the Resize arm and the rest, unchanged
            }
        }
        …
        crash_if_requested(&crash_at, "tick");
        app.tick(dt);
        crash_if_requested(&crash_at, "draw");
        app.draw(&mut curr_frame);
        …
    }

    drop(render_tx);
    render_handle.join().unwrap(); // a panicked render thread surfaces here
    Ok(())
}

/// Raw mode, the alternate screen and a hidden cursor — undone on every ending
/// spec 028 names: a clean quit, an error returned from `main`, and a panic on
/// either thread. `Drop` covers all three, so the restore has exactly one home
/// and the crash report is printed after it, on the real terminal. (Not a
/// signal: `kill` still leaves the terminal as it was.)
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        let guard = Self;      // from here on, an early `?` restores too
        crash::install_hook(); // …and from here on, a panic is recorded and
                               // printed by the Drop above, after the restore
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        crash::restore_terminal();
        if let Some(report) = crash::report() {
            for line in crash::crash_lines(report) { eprintln!("{line}"); }
        }
    }
}

/// Panic on purpose at `point`, if `KAAZAP_CRASH_AT` named it (spec 028).
/// Nothing in the game sets that variable and no key reaches this.
fn crash_if_requested(crash_at: &Option<String>, point: &str) {
    if crash_at.as_deref() == Some(point) {
        panic!("deliberate crash at {point} (KAAZAP_CRASH_AT)");
    }
}
```

The three old teardown lines (`Show`, `LeaveAlternateScreen`,
`disable_raw_mode`) and the `let mut stdout` they used are **removed** from the
end of `main`; `drop(render_tx)` and `render_handle.join().unwrap()` stay
exactly as they are.

**What each ending prints**, so the walkthrough knows what it is looking at:

| Ending | On the restored terminal | Exit |
|---|---|---|
| `q` | nothing | 0 |
| input error (AC 4) | `Error: kaazap couldn't read the terminal: <io detail>`, printed by `Termination` after `main` returns and after the guard has restored — **no panic text**, because no panic was recorded | 1 |
| panic on either thread (AC 2, 3) | the crash line, the panic's message, its location | 101 |

The input line is `anyhow`'s Debug rendering of the context chain, not our
`crash_lines`. AC 4 asks for the same *restore* and an unsuccessful exit, not
for the kaazap crash line; the `.context` is what keeps it from reading as a
bare `Os { code: 5, … }`.

### 3. `src/paths.rs`

Module doc gains a second paragraph: the root, *and* the one way the three
files are written. Then:

```rust
/// Write `contents` to `path` whole (spec 028): the bytes go to a temp file
/// beside it, which is then renamed over it, so the contents are complete
/// before they become the file the game reads and an interrupted write leaves
/// the previous file exactly as it was. (The replacement is atomic on POSIX;
/// on Windows it is a rename that may not be, but still never leaves a
/// half-written file under the real name — and fails outright, as a silent
/// no-op save, if another process holds the target open.) The temp name is
/// fixed, so repeated interrupted writes leave one piece of debris rather than
/// a pile, and it is never `*.json`, so no loader reads it. Best-effort like
/// the three callers it serves: any failure removes the temp file and returns
/// `false`, leaving the previous file alone. Not durable against a power cut —
/// see plan §3.
pub(crate) fn write_whole(path: &Path, contents: &str) -> bool {
    let tmp = path.with_extension("tmp");
    if fs::write(&tmp, contents).is_err() || fs::rename(&tmp, path).is_err() {
        let _ = fs::remove_file(&tmp);
        return false;
    }
    true
}
```

### 4. `src/profile.rs`

```rust
/// Why a profile couldn't be read (spec 028). Two cases, because only one of
/// them is something the player can act on: a file saved by a different build
/// can be read by finding that build. "Couldn't be read" covers an I/O error,
/// a permissions failure and a malformed document alike — the player does the
/// same thing about all three (spec Q3 b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileProblem { Unreadable, WrongVersion }

/// A profile that couldn't be read: what was wrong, and the name the old file
/// was kept under — `None` if it couldn't be moved, in which case nothing may
/// overwrite it for the rest of the launch (see [`SAVES_SUSPENDED`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFailure { pub problem: ProfileProblem, pub set_aside: Option<String> }

/// Set when the profile on disk couldn't be read *and* couldn't be moved
/// aside. Process-scoped, because "the rest of this launch" is exactly the
/// process — and because `save` has a dozen callers, none of which should have
/// to remember this.
static SAVES_SUSPENDED: AtomicBool = AtomicBool::new(false);

impl Profile {
    pub fn load() -> (Self, Option<ProfileFailure>) {
        let Some(path) = Self::path() else { return (Self::default(), None) };
        let problem = match fs::read_to_string(&path) {
            // No file and no data directory are a first launch: silent.
            Err(e) if e.kind() == io::ErrorKind::NotFound => return (Self::default(), None),
            Err(_) => ProfileProblem::Unreadable,
            Ok(text) => match Self::classify(&text) {
                Ok(profile) => return (profile, None),
                Err(problem) => problem,
            },
        };
        // The file is there and unusable: keep it, and never write over it.
        let set_aside = set_aside(&path);
        if set_aside.is_none() { SAVES_SUSPENDED.store(true, Ordering::Relaxed); }
        (Self::default(), Some(ProfileFailure { problem, set_aside }))
    }

    pub fn save(&self) {
        // The profile on disk couldn't be read and couldn't be moved aside:
        // nothing this session does may overwrite it (spec 028).
        if SAVES_SUSPENDED.load(Ordering::Relaxed) { return; }
        let Some(path) = Self::path() else { return };
        if let Some(dir) = path.parent() { let _ = fs::create_dir_all(dir); }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            crate::paths::write_whole(&path, &json);
        }
    }

    /// Parse profile JSON: a document that doesn't parse is `Unreadable`, one
    /// whose version doesn't match is `WrongVersion` — discarded rather than
    /// mis-read, exactly as before; the gate only says which failure it was.
    fn classify(text: &str) -> Result<Self, ProfileProblem> {
        let profile: Profile = serde_json::from_str(text).map_err(|_| ProfileProblem::Unreadable)?;
        (profile.version == PROFILE_VERSION).then_some(profile).ok_or(ProfileProblem::WrongVersion)
    }

    /// The filesystem-free core as the tests have always used it.
    #[cfg(test)]
    fn from_json(text: &str) -> Option<Self> { Self::classify(text).ok() }
}

/// Move an unreadable profile out of the way, returning the name it now has —
/// `None` if there is nowhere to put it or the move failed, which suspends
/// saving. Checked before renaming, because `fs::rename` would replace an
/// existing set-aside file rather than fail.
fn set_aside(path: &Path) -> Option<String> {
    let dir = path.parent()?;
    let name = set_aside_names(&utc_stamp(SystemTime::now()))
        .into_iter()
        .find(|n| !dir.join(n).exists())?;
    fs::rename(path, dir.join(&name)).ok()?;
    Some(name)
}

/// The names an unreadable profile is kept under, in order: the dated name,
/// then `-2` … `-9` if it is taken. Past that, nothing — the move fails and
/// saving is suspended, which is the safe end.
fn set_aside_names(stamp: &str) -> Vec<String> { … }   // profile-{stamp}[-n].json

/// `YYYYMMDD-HHMMSS` in UTC. Hand-rolled: the toolbox has no date crate, and
/// local time would need the platform's zone database, which does.
fn utc_stamp(t: SystemTime) -> String { … }
fn civil_from_days(days: i64) -> (i64, u32, u32) { … }  // the standard conversion
```

`profile.rs`'s two test helpers (`profile_with`, `profile_with_credits`) are
untouched: no field is added to `Profile`.

### 5. `src/save.rs`

```rust
/// Check the match save at launch (spec 028). `false` when it is missing or
/// loadable — the ordinary cases, silent as today. When the file is there but
/// unreadable, malformed or the wrong version, **remove it** and return
/// `true`: that match is gone for good, so the notice reports it once instead
/// of on every later launch (spec Q4 a). Nothing is kept — a match is not a
/// run.
pub fn check_at_launch() -> bool {
    let Some(path) = save_path() else { return false };
    let unreadable = match fs::read_to_string(&path) {
        Err(e) => e.kind() != io::ErrorKind::NotFound,
        Ok(text) => from_json(&text).is_none(),
    };
    if unreadable { clear(); }
    unreadable
}
```

`save()`'s `fs::write` becomes `crate::paths::write_whole(&path, &json)`;
nothing else in the file changes. (If `clear()` itself fails, the notice
repeats next launch — best-effort, like every other write here.)

### 6. `src/settings.rs`

One line: `fs::write(path, json)` → `crate::paths::write_whole(&path, &json)`.
No notice, no new behaviour — spec's explicit non-goal.

### 7. `src/app.rs`

```rust
/// The data notice (spec 028): what a launch found unreadable, and where the
/// old file went. Carries its lines because they are fixed at launch and their
/// inputs are gone by the second frame. Enter, Space or Esc dismiss it back to
/// the untouched start menu. Transient: never saved, never re-shown.
Modal::DataNotice(Vec<String>),
```

```rust
/// The data notice's content, title first and dismiss line last, blank rows
/// included — `None` when the launch found nothing wrong, which is also the
/// "raise no modal" answer. Pure, so the wording, the breathing room and the
/// fit are testable without a terminal. One notice covers both files.
fn data_notice_lines(profile: Option<&ProfileFailure>, save_unreadable: bool) -> Option<Vec<String>> {
    if profile.is_none() && !save_unreadable { return None; }
    let mut lines = vec!["Some of your saved data couldn't be read.".to_string()];
    if let Some(f) = profile {
        lines.push(String::new());
        lines.push(match f.problem {
            ProfileProblem::Unreadable => "Your campaign profile couldn't be read.",
            ProfileProblem::WrongVersion =>
                "Your campaign profile was saved by a different version of kaazap.",
        }.to_string());
        match &f.set_aside {
            Some(name) => {
                lines.push(format!("The old file is kept as {name}."));
                lines.push("You're playing on a new starter profile.".to_string());
            }
            None => {
                lines.push("The old file couldn't be moved out of the way.".to_string());
                lines.push("kaazap won't save over it, so nothing from this session is kept.".to_string());
            }
        }
    }
    if save_unreadable {
        lines.push(String::new());
        lines.push("Your in-progress match couldn't be read, so it wasn't kept.".to_string());
    }
    // The dismiss line is the acted-on element: an empty row above it, and the
    // box's own padding below (constitution, *acted-on element stands apart*).
    lines.push(String::new());
    lines.push("Enter  continue".to_string());
    Some(lines)
}
```

Blank rows can only ever appear *between* sections and above the dismiss line,
so two consecutive blanks are unreachable by construction.

`App::new`:

```rust
let settings = Settings::load();
let (profile, profile_failure) = Profile::load();
// A match save that can't be read is reported and removed, so `has_save` below
// sees the truth and the notice doesn't repeat next launch.
let save_unreadable = crate::save::check_at_launch();
let has_save = crate::save::exists();
…
modal: data_notice_lines(profile_failure.as_ref(), save_unreadable).map(Modal::DataNotice),
```

Draw (beside the `Victory` arm): `Some(Modal::DataNotice(lines)) =>
self.draw_notice(frame, lines)`.

Input (beside the `Victory` arm):

```rust
} else if matches!(self.modal, Some(Modal::DataNotice(_))) {
    // Raised at launch over the start menu: Enter/Space/Esc dismiss it,
    // nothing else acts, and nothing underneath is touched (spec 028).
    if notice_dismissed(key) {
        self.modal = None;
        self.audio.play(Sfx::MenuSelect);
    }
}
```

No `resize` arm, no new `App` field, no change to `has_save`, the menu, or any
`save()` call site.

### 8. `Readme.md`

Two sentences appended to the **Saved data** paragraph (~line 120), before the
`KAAZAP_DATA_DIR` example:

> If `profile.json` can't be read — damaged, or written by a different version
> of kaazap — kaazap keeps it under a dated name beside it (for example
> `profile-20260919-143005.json`), says so at the start menu, and plays on a
> fresh starter profile. An in-progress match that can't be read is reported
> the same way and removed.

## Files

- `src/crash.rs` — new: the hook, the recorded report, `restore_terminal`,
  `crash_lines`; tests. `src/lib.rs` — one `pub mod` line.
- `src/main.rs` — `TerminalGuard`, the hook install, the declaration order, the
  render loop's stop check, `crash_if_requested`; the three teardown lines and
  the `stdout` local removed.
- `src/paths.rs` — `write_whole`, the module doc's second paragraph; tests.
  The three one-line call-site changes land **with it** (T003), not after it:
  a `pub(crate)` function whose only non-test caller arrives a task later is
  `dead_code` in the plain lib target, which `cargo build --all-targets`
  builds and every task's "no new warnings" bar fails on.
- `src/profile.rs` — `ProfileProblem`, `ProfileFailure`, `SAVES_SUSPENDED`,
  `load`, `save`, `classify` (+ `from_json` as a `#[cfg(test)]` wrapper),
  `set_aside`, `set_aside_names`, `utc_stamp`, `civil_from_days`; tests.
- `src/save.rs` — `check_at_launch`, `save` through `write_whole`.
- `src/settings.rs` — `save` through `write_whole` (one line).
- `src/app.rs` — `Modal::DataNotice`, `data_notice_lines`, `App::new`'s three
  lines, the draw arm, the input arm; one test.
- `tests/whole_file_write.rs`, `tests/profile_recovery.rs`,
  `tests/profile_save_suspended.rs`, `tests/match_save_recovery.rs` — new, one
  `#[test]` each (the root resolves once per process).
  `tests/whole_file_write.rs` is written at T004 against today's
  `Profile::load() -> Self` and **updated at T005** when the signature changes
  (`let (profile, _) = Profile::load();`) — it is part of T005's footprint, not
  a surprise at its build step.
- `Readme.md` — two sentences.
- `specs/028-crash-and-data-safety/closeout-main-docs.md` (T009).
- **No change**: `src/card.rs`, `src/game.rs`, `src/player.rs`,
  `src/opponent.rs`, `src/economy.rs`, `src/campaign.rs`, `src/wager.rs`,
  `src/board.rs`, `src/motion.rs`, `src/render.rs`, `src/frame.rs`,
  `src/layout.rs`, `src/overlay.rs`, `tests/balance.rs`, `Cargo.toml`,
  `Cargo.lock`, the assets.

## Tests

Each claim names the task that owns its check. Every disk-level test sets its
own scratch root under `std::env::temp_dir()` and removes it at the end; none
touches the real data directory, and no new test constructs an `App`.

- **The crash report names kaazap, then the panic** (AC 2) — T001, `crash.rs`:
  `crash_lines_name_kaazap_then_the_message_and_location`: a `CrashReport` with
  both fields → three lines, the first naming kaazap, the second the message,
  the third `at …`; with `location: None` → two lines and no `at` line.
- **Restoring the terminal twice is harmless** (AC 5) — T001:
  `restoring_the_terminal_twice_is_harmless`: `restore_terminal()` twice under
  a non-terminal stdout returns normally both times (it swallows every error
  rather than unwrapping — the property that keeps `Drop`-during-unwind from
  aborting).
- **Every ending restores the terminal; a crash prints the report on it; a
  clean quit prints nothing** (AC 1, 2, 3, 4) — T002, **driver walkthrough**
  (§Verification). Not unit-testable: it is terminal side-effect code and
  process-exit behaviour, which the constitution says to verify by running.
  The load-bearing declaration order in `main` is checked at review.
- **A completed write leaves the new contents; a failed one leaves the previous
  file byte-for-byte; debris is one file and is never loaded** (AC 6, 7) —
  T003, `paths.rs`, against its own temp directory (no data root involved):
  `write_whole_replaces_the_file_and_leaves_no_debris` (write A, write B, read
  back B, the directory holds exactly one entry);
  `a_failed_write_leaves_the_previous_file_untouched` (make the temp path a
  **directory** — portable, deterministic — so the write fails: `false` comes
  back and the original bytes are unchanged);
  `repeated_failed_writes_do_not_accumulate` (three failed writes, the
  directory's entries unchanged).
- **All three writers go through it, and a following launch loads the previous
  file rather than the debris** (AC 6, 7) — T004,
  `tests/whole_file_write.rs` (scratch root): `Settings::save`,
  `Profile::save` and `save::save` each land their contents and leave no
  `.tmp`; with a `*.tmp` directory planted beside each, each `save` fails
  silently, the previous `*.json` is unchanged, and `Settings::load`,
  `Profile::load` and `save::load` all return the previous contents.
- **A first launch is silent** (AC 8) — T006, `tests/profile_recovery.rs`: a
  root that doesn't exist → `(starter, None)` and no file or directory created;
  the same root created but empty → `(starter, None)`.
- **A malformed and a wrong-version profile are each set aside, reported, and
  played past on a starter profile; the set-aside file holds the original
  bytes; saving afterwards writes a new profile and neither overwrites the
  set-aside file nor makes a second one** (AC 9, 10) — T006, same file, in
  sequence in the one test: malformed → `Unreadable` + `set_aside: Some(name)`
  matching `profile-\d{8}-\d{6}(-\d)?\.json`, the set-aside file's bytes equal
  what was written, `profile.json` gone; `profile.save()` → `profile.json`
  exists and parses, the set-aside file still byte-identical, exactly one
  set-aside file in the directory; then a wrong-version document (`"version":
  2`) → `WrongVersion` + a *different* set-aside name, both set-aside files
  intact. (`#[cfg(unix)]` step in the same test: a `chmod 0o000` profile →
  `Unreadable`, covering the I/O case the other two can't reach portably; the
  file is chmod'd back before the directory is removed.)
- **When the move fails, the notice says so, the file is untouched, and no
  profile save writes anything for the rest of the launch** (AC 11) — T006,
  `tests/profile_save_suspended.rs`, `#[cfg(unix)]` and in its own binary
  because the suspension is sticky for the process: a malformed
  `profile.json` in a directory chmod'd `0o555` → `set_aside: None`;
  `profile.save()` writes nothing; the file's bytes are unchanged at the end;
  the directory's entries are unchanged (no `.tmp`, no set-aside). The
  directory is chmod'd back and removed. Platform note: a read-only directory
  does not block creation on Windows, so this criterion is pinned on
  Unix only.
- **An unreadable match save is reported once, removed, and leaves no
  Continue** (AC 12) — T007, `tests/match_save_recovery.rs` (scratch root): no
  file → `check_at_launch()` false, nothing created; a malformed
  `saves/savegame.json` → true, the file gone, `save::exists()` false; a second
  `check_at_launch()` → false (the notice cannot repeat); a wrong-version
  document → true and removed; a valid save (written through `save::save`) →
  false and still there.
- **One notice carries both failures, reads right, breathes, and fits 89×31 at
  its longest** (AC 13, 14) — T008, `app.rs`:
  `the_data_notice_reads_right_breathes_and_fits_the_minimum_terminal`:
  `data_notice_lines(None, false)` is `None`; the `Unreadable` + set-aside case
  names "couldn't be read" and the file name; the `WrongVersion` case names
  "different version" and not "couldn't be read"; the failed-move case names no
  file and says nothing is kept; save-only carries the match line and no
  profile line; both together carry both, in one notice. For the longest
  content (`WrongVersion` + failed move + the match line, and the set-aside
  variant with a `-9` name), at both `Config::fit_sizes()`: the last line is
  non-blank, the line above it is blank, no two consecutive blanks, and
  `OverlayLayout::new` gives `outer.height() == lines.len() + V_PAD`,
  `outer.width() == width + 2 * H_PAD` (i.e. unclamped) and a box inside the
  frame — the assertions
  `both_notices_read_right_breathe_and_fit_the_minimum_terminal` already makes.
- **Enter, Space and Esc dismiss it and nothing else acts** (AC 14) — T008 via
  the shared `notice_dismissed`, already pinned by
  `notice_dismissed_on_enter_space_or_esc_only`, plus the review of the input
  arm against `Modal::Victory`'s and the Phase 4 walkthrough. Not unit-tested
  further: the routing needs an `App`, and an `App` in a unit test reads the
  real data directory (§Design tension 9). See §Open questions 3.
- **A settings file that can't be read is still silent** (AC 15) — T003:
  `settings.rs`'s existing `settings_malformed_or_empty_json_falls_back_to_default`
  and `settings_missing_or_legacy_fields_use_defaults` pass unedited, and
  `Settings::load`'s body is untouched (the T003 diff, which changes exactly
  one line in that file).
- **The version gates and the field defaults are unchanged** (AC 17) — T005:
  every existing `profile.rs` test (`a_wrong_version_document_is_discarded`,
  `missing_or_garbage_json_is_rejected_but_an_empty_object_is_the_starter`, the
  serde-default tests) passes **unedited** through the `#[cfg(test)] from_json`
  wrapper; `save.rs`'s `corrupt_or_incompatible_json_loads_as_none` likewise.
  T009 greps both constants.
- **The dated name and the collision rule** (AC 9) — T005, `profile.rs`:
  `the_set_aside_stamp_is_the_utc_date_and_time`: `utc_stamp(UNIX_EPOCH)` is
  `"19700101-000000"`, `UNIX_EPOCH + 1_600_000_000 s` is `"20200913-122640"`,
  and `UNIX_EPOCH + 1_583_020_799 s` is `"20200229-235959"` (the leap-day
  branch); `set_aside_names_start_dated_then_number`: the first name is
  `profile-<stamp>.json`, the rest are `-2` … `-9`, all distinct, all ending
  `.json`.
- **No forbidden file changed, no new crate, no version bump** (AC 17) — T009:
  `git diff main...HEAD --stat` lists none of the forbidden paths;
  `git diff main...HEAD -- Cargo.toml Cargo.lock` is empty.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim.
- **Driver walkthroughs** (the orchestrator, `run-kaazap` skill, **with
  `KAAZAP_DATA_DIR` pointed at a scratch directory** — every scenario below
  deliberately damages files, so the real profile, save and settings are never
  in play; confirm the scratch root in the report).
  - **After the Phase 1 review (T002)**, at 89×31: launch and quit with `q` —
    the shell prompt returns, typed characters echo, the cursor is visible, the
    scrollback from before the launch is still there, nothing was printed, and
    `echo $?` is 0. Then, one at a time, `KAAZAP_CRASH_AT=key` (press a key),
    `=tick`, `=draw`, `=render` (press `q` to reach the join), and `=input`:
    each time the terminal is usable, the crash line and the panic's message
    and location are readable on the terminal (and the `input` run shows
    `Error: kaazap couldn't read the terminal: …` with **no panic text**), and
    `echo $?` is non-zero. **For `=draw` and `=tick` the report must state
    explicitly** that the cursor was visible afterwards and that nothing was
    garbled above the crash line — that is the race in tension 1, and a hidden
    cursor or a smear of frame output is the symptom. If either shows, the
    fallback named there (the guard owns the sender and the `JoinHandle` and
    joins in `Drop`) is a sub-lettered task, not a redesign. Report what the
    terminal looked like in plain language.
  - **After the Phase 4 review (T008)**, at 89×31, against the scratch root:
    (a) delete everything → launch → no notice, a normal start menu, a fresh
    campaign; (b) `echo 'not json' > profile.json` → launch → the notice names
    the profile and the dated file it was kept under, Esc dismisses it to the
    start menu with the usual selection, the dated file is still on disk with
    the same bytes, playing on and returning to the menu writes a new
    `profile.json` and no second dated file appears; (c) a `"version": 2`
    profile → the notice says a different version of kaazap wrote it; (d) a
    malformed `saves/savegame.json` with a good profile → the notice carries
    only the match line, **Continue** is absent, the file is gone, and the next
    launch is silent; (e) both damaged at once → **one** notice carrying both,
    fitting the 89-column terminal with nothing clipped and an empty row above
    and below the dismiss line; (f) Enter and Space also dismiss, and the keys
    the Phase 4 review enumerated as reaching an open modal — at minimum `?`,
    `L`, `m`, an arrow, a digit and a letter — do nothing that loses the notice
    or acts on the menu underneath. Report in plain language.

## Non-goals (from spec)

No crash recovery, no writing from inside the crash path, no change to the
version gates or the field defaults, no migration of an old profile, no notice
for settings, no backups of good files or rotation, no in-app recovery, and no
restructuring of anything the audit found sound.

## Open questions

Settled here as design and flagged for sign-off:

1. **No `fsync` in `write_whole`** (tension 3) — rename alone covers every
   acceptance criterion (a write that fails partway); a power cut needs
   `sync_all`, which is a full device flush on macOS on **every key press**,
   since the match save is written on every state change. One line to add if
   the person would rather pay it.
2. **`q` and `m` still act under the notice** (tension 8), as they do under
   every other modal in the game. A literal reading of the "no other key does
   anything while it is up" criterion would make this notice the one modal you
   cannot quit or mute under. Raised as a blocking finding at sign-off and
   **ruled by the person, 2026-09-19 (Q6): both stay live.** `spec.md` carries
   the amendment naming them as the standing exception, so this is settled and
   T008's input arm is as drafted. The Phase 4 review still enumerates the keys
   that actually reach an open modal rather than taking this set on trust.
3. **The notice's dismissal is not unit-tested** (tension 9) — the routing arm
   needs an `App`, and an `App` in a unit test reads the real data directory,
   which the bundle says not to make worse. It is the same three lines as
   `Modal::Victory`'s, checked at review, and attested in the Phase 4
   walkthrough.
4. **The set-aside stamp is UTC** (tension 6) — a player east or west of UTC
   may see a name a day off their local clock. Local time needs a zone
   database, which needs a crate.
5. **The failed-move criterion is pinned on Unix only** (§Tests) — a read-only
   directory doesn't block file creation on Windows, so there is no portable
   way to force `fs::rename` to fail. The code path is platform-independent;
   only its test is not.
6. **`KAAZAP_CRASH_AT` ships in the binary** (tension 2) — four acceptance
   criteria are about what a crash looks like, and there is no other way to
   produce one. Not documented in `Readme.md`, not reachable by any key, file
   or menu.
7. **The render thread can still, in principle, draw over the crash report**
   (tension 1) — narrowed to a microsecond window by one flag check, not
   closed. The deterministic fix is named and held back on Simplicity grounds;
   the Phase 1 walkthrough has to report on it explicitly for `=draw` and
   `=tick`, and turning the fallback on is a sub-lettered task.
8. **Signals are out of scope** (tension 1) — `SIGTERM` and `kill -9` still
   leave the terminal unrestored. `spec.md` enumerates the endings it means and
   signals are not among them, so this is a boundary rather than a gap; the
   close-out must describe the guarantee as the spec's list, not as "however
   kaazap ends".
