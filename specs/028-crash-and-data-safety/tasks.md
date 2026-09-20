# Tasks: Crash & data safety — spec 028

> **Status**: Final — signed off 2026-09-19 (`skeptical-reviewer`, one review
> and one re-review; B1 and B2 fixed, B3 ruled by the person as Q6, S1–S8
> folded in)
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task should be completable (and
testable) on its own. If a session ends mid-list, resume by finding the first
unchecked task — don't re-verify everything above it unless something looks off.

<!-- WARNING: once implementation starts, this file gets written by more than one
party — whoever's steering adds scope and reshuffles tasks; the implementing
session (the orchestrator — never the sdd-implementer subagent) checks boxes and
adds findings. Never edit this file from a stale copy. Prefer small, targeted
edits over regenerating it wholesale — a full replacement silently discards
whatever the other party added since your copy was taken. -->

Per the constitution: every implementation task ends with an actual build and,
where tests exist for what changed, an actual test run — reported, not
summarized. Under the model policy, the orchestrator verifies after an
implementer returns, and only the orchestrator commits. This spec runs on the
**economy profile** (the project's since 2026-09-19): every role resolves to
`opus` / `claude-opus-5`, no dispatch carries a top-tier override, and the
close-out is dispatched to `sdd-implementer`.

**Foundational phases: Phase 2 and Phase 3.** Phase 2 is the whole-file write
every writer then goes through; Phase 3 is the load contract the notice in
Phase 4 reads. Phase 1 is independent of both and comes first because it is the
most visible and the person can attest it on its own.

**Two tasks carry `review: per-task`** — T003 (`paths::write_whole` and the
three writers it lands with, which every future writer inherits) and T005 (the
one task in this spec that can move a player's file, and whose disk-level tests
don't land until T006). Every other task is covered by its phase review.

**No new unit test may construct an `App`** (plan §Design tension 9):
`App::new` reads the real data directory, and from T005 on it can *move* a file
there. Disk-level checks are integration tests with their own scratch root;
everything else goes through a pure function.

**Never run `cargo fmt`** (the repo is not rustfmt-clean; it would rewrite ~28
files).

**Settled — Q6, the person, 2026-09-19** (plan §Design tension 8, §Open
questions 2): `q` and `m` act under every modal in the game today, which a
literal reading of the acceptance criterion "no other key does anything while
it is up" would have forbidden for this one. The person ruled that both stay
live, and `spec.md` is amended to name them as the standing exception. T008's
input arm is as drafted; no task line needs editing.

---

## Phase 1 — The terminal always comes back (walkthrough: quit with `q` and see a normal shell; then force a crash and see a usable terminal with the crash line, the panic message and its location on it)

<!-- T001 is the pure module; T002 is main.rs wiring it. After T002 every
ending restores the terminal and a crash says so. -->

- [x] **T001** — `src/crash.rs` (new) + `src/lib.rs`: the recorded crash report
  and the terminal restore. Per plan §Design 1: the module doc; `static REPORT:
  OnceLock<CrashReport>`; `pub struct CrashReport { message: String, location:
  Option<String> }` deriving `Debug, Clone, PartialEq, Eq`; `pub fn
  install_hook()` setting a hook that **records and prints nothing** (first
  panic wins — `let _ = REPORT.set(…)`), reading the payload and
  `info.location()`. **`PanicHookInfo::payload_as_str` is stable only from Rust
  1.91** and `Cargo.toml` declares no `rust-version` (AC 17 forbids adding
  one): if the toolchain in use doesn't have it, use the two-arm downcast
  (`info.payload().downcast_ref::<&str>()`, then
  `downcast_ref::<String>()`, else `"(no message)"`) — no version floor either
  way, and say in the report which one you used. Then `pub fn report() ->
  Option<&'static CrashReport>`; `pub fn
  restore_terminal()` doing `Show`, `LeaveAlternateScreen`,
  `disable_raw_mode`, **every error swallowed with `let _ =`, never
  unwrapped** (it runs from a `Drop` during unwinding, where a panic aborts the
  process); `pub fn crash_lines(&CrashReport) -> Vec<String>` with the plan's
  exact wording. In `lib.rs`, `pub mod crash;` beside `pub mod paths;`.
  Tests, in `crash.rs`'s tests module (plan §Tests):
  `crash_lines_name_kaazap_then_the_message_and_location` (both fields → three
  lines, first names kaazap, second the message, third starts `  at `;
  `location: None` → two lines, no `at` line) and
  `restoring_the_terminal_twice_is_harmless` (call `restore_terminal()` twice;
  it returns normally both times). **Do not call `install_hook` from any
  test** — it would swallow the panic output of every failing test in the
  binary. (Copies: `src/paths.rs` for the small-module-with-a-`OnceLock` shape
  and its doc voice; `src/render.rs` for the crossterm command calls.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green,
  reported verbatim, with the two named tests passing; `git diff --stat` shows
  only `src/crash.rs` and `src/lib.rs`; `git diff -- src/lib.rs` is the one
  `pub mod` line; the implementer's report quotes `install_hook` and
  `restore_terminal` verbatim.*

- [x] **T002** — `src/main.rs`: the terminal guard and the crash seam. Per plan
  §Design 2: add `struct TerminalGuard` with `fn enter() -> anyhow::Result<Self>`
  (construct the value **first**, then `crash::install_hook()` **before**
  `enable_raw_mode` — installing it after the screen switch would leave a
  window in which the default hook prints onto the alternate screen and that
  output is discarded with nothing recorded to reprint — then
  `enable_raw_mode`, `EnterAlternateScreen`, `Hide`) and `impl Drop`
  (`crash::restore_terminal()`,
  then `eprintln!` each of `crash::crash_lines(report)` if `crash::report()` is
  `Some`); in `main`, replace the three setup lines and the `let mut stdout`
  with `let _terminal = TerminalGuard::enter()?;` **declared immediately after
  `Config::from_terminal()?` and before every other local**, carrying the
  plan's comment about reverse drop order — this ordering is what keeps the
  render thread from drawing over the report and is the one thing in this task
  no test can catch; delete the three teardown lines at the end of `main`
  (`Show`, `LeaveAlternateScreen`, `disable_raw_mode`) and leave
  `drop(render_tx)` and `render_handle.join().unwrap()` exactly as they are
  (a panicked render thread surfaces there and the guard prints *its* report,
  because the first panic wins); add `if crash::report().is_some() { break; }`
  as the first line of the render thread's `while let Ok(…) = recv()` body with
  the plan's comment; add `fn crash_if_requested(crash_at: &Option<String>,
  point: &str)` and the `let crash_at = std::env::var("KAAZAP_CRASH_AT").ok();`
  read **once** before the loop (cloned into the render closure), called at
  `key` (before `app.handle_key`), `tick`, `draw` and `render`, plus the
  `input` arm that `anyhow::bail!`s at the top of the loop body with the plan's
  comment. Add `.context("kaazap couldn't read the terminal")` to the
  `event::poll(…)?` and `event::read()?` calls (importing `anyhow::Context`),
  so the AC 4 path prints a line that names kaazap instead of a bare
  `Os { code: … }` — plan §Design 2's table is what each ending must print. No
  other change to the loop, the resize arm, or the frame plumbing.
  No tests (terminal side-effect and process-exit behaviour — the constitution
  says verify it by running it; the walkthrough below is the verification).
  Do not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/main.rs`;
  `grep -n "disable_raw_mode\|LeaveAlternateScreen\|Show" src/main.rs` is
  **empty**, imports included — the restore lives in `crash.rs` now — while
  `enable_raw_mode`, `EnterAlternateScreen` and `Hide` remain, used only inside
  `TerminalGuard::enter`; the implementer's report quotes `TerminalGuard`,
  its `Drop`, and the first ten lines of `main` verbatim.
  **PAUSE for the person** (after the Phase 1 review): the orchestrator drives
  the Phase 1 walkthrough in plan §Verification with the `run-kaazap` skill —
  `KAAZAP_DATA_DIR` pointed at a scratch directory, confirmed in the report —
  and reports in plain language: a `q` quit leaving a normal shell with the
  scrollback intact, nothing printed and a zero exit; each of
  `KAAZAP_CRASH_AT=key|tick|draw|render|input` leaving a usable terminal with
  the crash line and the panic's message and location readable on it (and the
  `input` run showing `Error: kaazap couldn't read the terminal: …` with no
  panic text), each with a non-zero exit. **For `=draw` and `=tick` the report
  must say explicitly whether the cursor was visible afterwards and whether
  anything was garbled above the crash line** — that is the render-thread race
  in plan §Design tension 1, and a hidden cursor is exactly what AC 2 tests. If
  either shows, log **T002a** and turn on the fallback the plan names (the
  guard owns the sender and the `JoinHandle` and joins in its `Drop`); it is a
  sub-lettered task, not a redesign.*

- [x] **T002a** — `src/main.rs`: the guard joins the render thread. Logged
  2026-09-19 from the Phase 1 walkthrough, which the task line required: with
  `KAAZAP_CRASH_AT=tick` and `=draw`, **all six runs of each** put render-thread
  `MoveTo` sequences on the real terminal after `LeaveAlternateScreen`, and in
  five of the twelve the crash text itself was visibly broken by stray escape
  fragments and blank padding. The cursor was visible in every run (AC 2's
  cursor half holds — `render.rs` emits no `Hide`), so the failure is the
  garbling, not the cursor. `=key`, `=render`, `=input` and a normal `q` quit
  were clean in all six runs each.
  The Phase 1 review found *why* the in-loop `crash::report()` check doesn't
  cover it, and it is not the interleaving the plan modelled: the **forced
  first render before the loop** (`main.rs:51`, `render(&mut stdout, …, true)`)
  can never reach that check, and `tick`, `draw` and `input` all fire on the
  first game-loop iteration, microseconds after `thread::spawn` returns. The
  window is structural, not rare.
  Turn on the fallback plan §Design tension 1 pre-authorizes: give
  `TerminalGuard` an `Option<JoinHandle<()>>`, set after the spawn, and have its
  `Drop` `take()` and `join()` it (ignoring an `Err` — the hook already recorded
  it) **before** `crash::restore_terminal()`. The sender does not move into the
  guard: `render_tx` is declared after `_terminal`, so reverse drop order
  already drops it first, and the join is then bounded.
  **Keep `main`'s own `join().unwrap()` on the `q` path** — that is what turns
  a panicked render thread into an unsuccessful exit (AC 3). Main takes the
  handle out of the guard and joins/unwraps it exactly as today; the guard's
  `Drop` joins only what main left behind. Collapsing the two would make AC 3
  exit 0.
  Also correct the two sentences in `plan.md` §Design tension 1 the review
  found false — "a frame already past that check" (it is the pre-loop forced
  render) and the "if `render.rs` emits a per-frame cursor `Hide`" conditional
  (it does not) — and the over-claiming parenthetical in `main.rs`'s drop-order
  comment, "(and with it the render thread)": dropping a `SyncSender`
  disconnects the thread, it does not join it — which is precisely what this
  task changes.
  No tests (same terminal side-effect and process-exit behaviour as T002). Do
  not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/main.rs` and
  `specs/028-crash-and-data-safety/plan.md`; the orchestrator re-runs the
  walkthrough harness, six runs of each crash point, and the report says
  whether `=tick` and `=draw` are clean.*

## Phase 2 — Every file is written whole (foundational; walkthrough: none — the same three files are written at the same moments with the same contents, so nothing the person can see changes; the criteria are pinned by tests)

<!-- T003 is the function AND its three call sites — they cannot be split: a
pub(crate) fn whose only callers are #[cfg(test)] is dead_code in the plain lib
target that `cargo build --all-targets` also builds, and every task here has a
"no new warnings" bar. T004 is the integration test that pins it on disk. -->

- [x] **T003** — `src/paths.rs` + `src/profile.rs` + `src/settings.rs` +
  `src/save.rs`: the whole-file write, and the three writers that use it.
  `review: per-task`. Per plan §Design 3: `pub(crate) fn write_whole(path:
  &Path, contents: &str) -> bool` — write to `path.with_extension("tmp")`, then
  `fs::rename` it over `path`; on either failure remove the temp file
  (best-effort) and return `false`; with the plan's doc comment **as written
  there**, including the fixed-temp-name reason, the POSIX/Windows wording (do
  not claim atomicity on Windows) and the **no-fsync** note. Extend the module
  doc with a second paragraph: this module now owns both where the files live
  and how they are written. **No `sync_all`** (plan §Design tension 3 and
  §Open questions 1) and no unique temp names.
  Then, per plan §Design 4–6, in each of `Profile::save`, `Settings::save` and
  `save::save` the single `fs::write(path, json)` becomes
  `crate::paths::write_whole(&path, &json)` — **three one-line changes, in this
  same task**, so the new function has a real caller the moment it lands.
  Nothing else in those three functions changes: same moments, same contents,
  same `create_dir_all`, same swallowed failure. Do not remove any `fs` import
  unless the compiler says it is unused (all three files still use `fs`
  elsewhere — check, don't assume).
  Tests, in `paths.rs`'s tests module (plan §Tests), each using its own
  uniquely-named directory under `std::env::temp_dir()` and removing it at the
  end — **none of them may call `data_dir`, `config_dir` or `set_root`**, which
  would resolve the process root:
  `write_whole_replaces_the_file_and_leaves_no_debris`,
  `a_failed_write_leaves_the_previous_file_untouched` (create the temp path as
  a **directory** so the write fails portably; assert `false` and the original
  bytes), `repeated_failed_writes_do_not_accumulate` (three failed writes; the
  directory's entry set is unchanged). Do not run `cargo fmt`. (Copies:
  `src/paths.rs`'s own doc voice and test module.)
  *Verify: `cargo build --all-targets` no new warnings — **in particular no
  `dead_code` on `write_whole`**; `cargo test -q` green verbatim with the three
  new tests passing and every existing `settings.rs`, `profile.rs` and `save.rs`
  test unedited; `git diff --stat` shows only those four files, with one changed
  line each in the three writers; `grep -rn "fs::write" src/` matches **only**
  `src/paths.rs` (the one inside `write_whole`); the implementer's report quotes
  `write_whole` and the three changed call sites verbatim. The orchestrator
  re-runs the verification command itself before committing (per-task review),
  then dispatches a `skeptical-reviewer` on T003's diff alone before T004
  starts.*

- [x] **T003a** — `src/paths.rs`: pin the cleanup the tests currently miss.
  Logged 2026-09-19 from T003's per-task review (non-blocking finding 1).
  Both failure tests force the failure by making the **temp path** a
  directory, so `fs::write(&tmp, …)` fails on the first `is_err()` and no temp
  file is ever created. The `let _ = fs::remove_file(&tmp);` line and the whole
  `fs::rename` failure branch are therefore never executed: delete the cleanup
  and all three tests still pass, which makes the doc comment's "any failure
  removes the temp file" an untested claim.
  Add one test, `a_failed_rename_cleans_up_its_temp_file`: make the **target**
  a directory instead, so `fs::write` succeeds, `fs::rename` fails portably
  (`EISDIR` on POSIX, access-denied on Windows), and assert that `write_whole`
  returns `false` **and** that no `*.tmp` is left behind — an assertion that
  fails if the cleanup line is removed. Keep the three existing tests as they
  are; this is a fourth, not a replacement.
  While in the file: change the doc comment's closing sentence to name `fsync`
  explicitly, so a later power-cut question finds it by grep, and disambiguate
  `plan §3` (the document has both a *Design tension 3* and a *Design 3*).
  Do not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `cargo test --lib paths` naming all four tests; `git diff --stat`
  shows only `src/paths.rs`; the implementer's report shows the new test
  **failing** when the cleanup line is commented out, then passing with it
  restored — that is the point of the task.*

- [x] **T004** — `tests/whole_file_write.rs` (new): the three writers pinned on
  disk. **One `#[test]` only** (the data root resolves once per process):
  `paths::set_root` at a fresh scratch directory under `std::env::temp_dir()`;
  save settings, a profile and a match save; assert each file holds its
  contents and no `*.tmp` is left; then plant a `*.tmp` **directory** beside
  each of the three and save again — each save fails silently, each `*.json` is
  byte-for-byte unchanged, and `Settings::load`, `Profile::load` and
  `save::load` all return the previous contents; remove the scratch directory
  at the end. Write it against **today's** `Profile::load() -> Self`; **T005
  changes that signature and updates this file** — that is in T005's footprint,
  not a surprise at its build step. No `src/` change in this task. Do not run
  `cargo fmt`. (Copies: `tests/paths_override.rs` for the one-test-per-binary
  discipline and its header comment saying why; `src/save.rs`'s test module for
  building a `GameState` to save.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new integration test passing; `git diff --stat` shows only
  `tests/whole_file_write.rs`; the implementer's report confirms the scratch
  directory it used and that it removed it.*

- [x] **T004a** — `tests/whole_file_write.rs`: pin "debris is never loaded as
  the real file", and show the other five assertions bite. Logged 2026-09-19
  from the Phase 2 review (non-blocking N1 and N2).
  **N2** — AC 7's second clause, *nothing left behind is loaded as the real
  file*, is currently argued rather than tested: no test in the spec ever
  leaves a genuine `*.tmp` **file** beside a `*.json` and then calls a loader.
  Every failure fixture plants a `.tmp` *directory*, and T003a's temp file is
  removed by the cleanup it exists to test. Add a third phase to the existing
  test: write a real `settings.tmp` holding garbage, then assert
  `Settings::load()` still returns the previous settings **and** that the
  `.tmp` file is still there and byte-for-byte unchanged (the loader must
  neither read it nor consume it).
  **N1** — the mutation check the T004 report offered stops at the first
  `assert_eq!`, so it demonstrated one of six failure-phase assertions is live
  and said nothing about the other five. Re-run it properly: revert **one**
  writer at a time to a direct `fs::write(path, json)`, confirm that writer's
  own byte comparison fires, restore it, and repeat for the other two. Report
  all three failures verbatim. This is evidence, not a code change — the file
  must be byte-identical to its committed state afterwards apart from the N2
  addition.
  Still **one `#[test]`** in the binary. Do not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `cargo test --test whole_file_write` naming the one test;
  `git diff --stat` shows only `tests/whole_file_write.rs`; the report carries
  the three one-writer-at-a-time failures.*

## Phase 3 — A profile that can't be read is kept, not replaced (foundational; walkthrough: none — the file is set aside silently until the notice lands in Phase 4, so there is nothing on screen for the person to judge yet)

<!-- T005 is the profile's load contract and the suspension; T006 pins its
disk-level criteria; T007 does the same, smaller, for the match save. App::new
binds both results with a placeholder until T008 raises the notice. -->

- [x] **T005** — `src/profile.rs` + `src/app.rs` (one call site) +
  `tests/whole_file_write.rs` (**two** call sites — the task line said one;
  corrected 2026-09-19 at the per-task review, and both had to change for
  `cargo build --all-targets` to be green): classification, the set-aside,
  and the suspension. `review: per-task`. Per
  plan §Design 4: `pub enum ProfileProblem { Unreadable, WrongVersion }` and
  `pub struct ProfileFailure { pub problem: ProfileProblem, pub set_aside:
  Option<String> }` with the plan's docs; `static SAVES_SUSPENDED:
  AtomicBool`; `load()` returning `(Self, Option<ProfileFailure>)` exactly as
  the plan's listing (a `NotFound` read error and an unresolvable path are the
  silent first-launch cases; everything else sets the file aside and reports);
  `save()` gaining the suspension guard as its first line and calling
  `write_whole` (already done in T004 — keep it); `classify(text) -> Result<Self,
  ProfileProblem>` carrying the **unchanged** version comparison and discard,
  with `from_json` kept as a `#[cfg(test)] fn` delegating to it so every
  existing test stands unedited; free fns `set_aside(path) -> Option<String>`
  (first unused candidate name, then `fs::rename`; `None` on any failure),
  `set_aside_names(stamp) -> Vec<String>` (`profile-{stamp}.json`, then `-2` …
  `-9`), `utc_stamp(SystemTime) -> String` (`YYYYMMDD-HHMMSS`, UTC) and
  `civil_from_days`. In `app.rs`, `App::new`'s `let profile =
  Profile::load();` becomes `let (profile, _profile_failure) =
  Profile::load();` with a `// T008 raises the data notice from this` comment —
  **no other `app.rs` change**. In `tests/whole_file_write.rs` (T004), the
  `Profile::load()` call becomes `let (profile, _) = Profile::load();` —
  **the signature change breaks that test binary, and `cargo build
  --all-targets` builds it**, so it is part of this task, not a later
  discovery; change nothing else in that file. Do **not** add a field to
  `Profile` (the plan
  rejects it: `reset_to_starter` would clear it) and do **not** touch
  `PROFILE_VERSION`, any `#[serde(default)]`, or the two test helpers. Tests,
  in `profile.rs`'s tests module (plan §Tests):
  `the_set_aside_stamp_is_the_utc_date_and_time` (epoch → `19700101-000000`;
  epoch + 1_600_000_000 s → `20200913-122640`; epoch + 1_583_020_799 s →
  `20200229-235959`) and `set_aside_names_start_dated_then_number`. **No disk
  test here** — those are T006. Do not run `cargo fmt`. (Copies: `src/save.rs`'s
  `from_json` for the version-gate shape; `src/paths.rs` for the static-plus-doc
  voice.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the two new tests passing and **every existing `profile.rs`
  test unedited** (`a_wrong_version_document_is_discarded` and the serde-default
  tests especially); `git diff --stat` shows exactly `src/profile.rs`,
  `src/app.rs` and `tests/whole_file_write.rs`; `git diff -- src/app.rs` is the
  one changed line plus its comment; `git diff --
  tests/whole_file_write.rs` is the one destructuring line;
  `git diff -- src/profile.rs` leaves `PROFILE_VERSION`, every
  `#[serde(default)]` and the struct's fields untouched; the implementer's
  report quotes `load`, `classify` and `set_aside` verbatim. The orchestrator
  re-runs the verification command itself before committing (per-task review),
  then dispatches a `skeptical-reviewer` on T005's diff alone before T006
  starts.*

- [x] **T005a** — `src/profile.rs` (tests only) + `tests/whole_file_write.rs`
  (two lines): the three assertions T005's review found missing. Logged
  2026-09-19 from T005's per-task review (second-look notes 1, 3, 4). All
  filesystem-free or one-line; none of them waits on T006.
  **(1) The `Unreadable`/`WrongVersion` split is the one new decision in T005
  and nothing pins it.** Every existing test reaches `classify` through the
  `#[cfg(test)] from_json` wrapper, which collapses `Result` to `Option`, so
  `a_wrong_version_document_is_discarded` passes byte-for-byte whichever
  variant the version gate returns — while the spec makes the distinction
  player-visible ("couldn't be read" vs "saved by a different version of
  kaazap"). Add `classify_tells_a_bad_document_from_a_bad_version` to
  `profile.rs`'s tests: malformed JSON → `Err(ProfileProblem::Unreadable)`, a
  valid document with `"version": 2` → `Err(ProfileProblem::WrongVersion)`, a
  good document → `Ok`.
  **(2) `utc_stamp`'s pre-epoch claim has nothing behind it.** Its doc says a
  time before the epoch reads as the epoch itself; the three vectors test the
  epoch *value*, not the `Err` branch of `duration_since`. Extend
  `the_set_aside_stamp_is_the_utc_date_and_time` with
  `UNIX_EPOCH - Duration::from_secs(1)` → `19700101-000000`.
  **(3) `tests/whole_file_write.rs` discards the failure at both call sites**,
  so `assert_eq!(reloaded.credits(), credits)` does not prove `load` took the
  success path — a starter profile whose credits happened to match would pass
  while leaving a set-aside file in the scratch root. Bind it and
  `assert!(failure.is_none())` at both.
  Change nothing outside those tests. Do not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `cargo test --lib profile` and `cargo test --test whole_file_write`
  naming the tests; `git diff --stat` shows only `src/profile.rs` and
  `tests/whole_file_write.rs`, with `src/profile.rs` changed **inside its
  `#[cfg(test)]` module only**.*

- [x] **T006** — `tests/profile_recovery.rs` + `tests/profile_save_suspended.rs`
  (both new): the profile's disk-level criteria. Two binaries because the
  data root resolves once per process **and** the suspension flag is sticky
  once set. Per plan §Tests:
  `tests/profile_recovery.rs`, one `#[test]`, against a scratch root under
  `std::env::temp_dir()` — a root that doesn't exist yet → `(starter, None)`
  and nothing created; the directory created but empty → `(starter, None)`; a
  malformed `profile.json` → `Unreadable` + `set_aside: Some(name)` matching
  `profile-<8 digits>-<6 digits>[-n].json`, the set-aside file holding the
  original bytes, `profile.json` gone; `profile.save()` → a new parseable
  `profile.json`, the set-aside file still byte-identical, **exactly one**
  set-aside file in the directory; a `"version": 2` document → `WrongVersion` +
  a *different* set-aside name, both set-aside files intact; a `#[cfg(unix)]`
  step with a `chmod 0o000` profile → `Unreadable`; remove the directory at the
  end. **Two traps in that step**: `set_aside` has already *renamed* the file
  by the time you clean up, so a chmod-back must target the **set-aside path**,
  not `profile.json` — or skip it entirely, since removing a file needs write
  permission on the directory, not on the file; and a `0o000` file is still
  readable **as root**, where the step would silently pass as "readable" and
  prove nothing. Assert the failure rather than tolerating it — under root the
  test then fails loudly, which is the honest outcome — and say so in the
  file's header comment.
  `tests/profile_save_suspended.rs`, one `#[test]`, `#[cfg(unix)]` (say why in
  the header: a read-only directory doesn't block creation on Windows, so there
  is no portable way to make `fs::rename` fail) — a malformed `profile.json` in
  a directory `chmod`'d `0o555` → `set_aside: None`; `profile.save()` writes
  nothing; the file's bytes and the directory's entries are unchanged at the
  end; chmod back and remove. Do not run `cargo fmt`. (Copies:
  `tests/paths_override.rs` for the header comment and the one-test rule;
  `tests/whole_file_write.rs` from T004 for the scratch-root helper — lift it,
  don't share it: a test binary can't depend on another test binary.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with both new tests passing; `git diff --stat` shows only the two new
  files; the implementer's report names the scratch directories used, confirms
  both were removed, and quotes the set-aside file names the run actually
  produced.*

- [x] **T007** — `src/save.rs` + `src/app.rs` (one call site) +
  `tests/match_save_recovery.rs` (new): the match save reports once and is
  removed. Per plan §Design 5: `pub fn check_at_launch() -> bool` exactly as
  the plan's listing — missing (`NotFound`) or loadable is `false` and silent;
  anything else calls `clear()` and returns `true`. No change to `load`,
  `exists`, `from_json`, `SAVE_VERSION` or the `#[serde(default)]` fields. In
  `app.rs`, `App::new` gains `let save_unreadable = crate::save::check_at_launch();`
  **before** the existing `let has_save = crate::save::exists();` (~~so
  `has_save` sees the removal~~ — **the stated reason is false**, corrected
  2026-09-19 at the Phase 3 review: `exists()` is `load().is_some()`, i.e.
  validity rather than file presence, so `has_save` is false for an unreadable
  save whether or not `clear()` ran, and even if `clear()` itself fails. The
  ordering is right and harmless; the rationale is not load-bearing and nobody
  should reorder on the strength of it), bound as `let _save_unreadable = …` with a `//
  T008 raises the data notice from this` comment — no other `app.rs` change.
  New integration test `tests/match_save_recovery.rs`, one `#[test]`, scratch
  root: no file → `false`, nothing created; a malformed `saves/savegame.json`
  → `true`, the file gone, `save::exists()` false; a second call → `false` (the
  notice cannot repeat); a `"version": 2` document → `true` and removed; a
  valid save written through `save::save` → `false` and still there. Do not run
  `cargo fmt`. (Copies: `src/save.rs`'s own `exists`/`clear` for the shape and
  `src/save.rs`'s test module for building a `GameState`;
  `tests/paths_override.rs` for the one-test discipline.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new test passing and every existing `save.rs` test
  unedited; `git diff --stat` shows only `src/save.rs`, `src/app.rs` and the new
  test; `grep -n "SAVE_VERSION" src/save.rs` still reads 1; the implementer's
  report quotes `check_at_launch` verbatim.*

- [x] **T007a** — `tests/profile_save_suspended.rs`: chmod back before the
  assertions. Logged 2026-09-19 from the Phase 3 review (non-blocking 4).
  The root is `chmod 0o555` before `Profile::load()`, and the chmod back to
  `0o755` happens *after* four assertions. If any of them fires, the read-only
  directory survives the process — and the next run's opening
  `let _ = fs::remove_dir_all(&root)` cannot unlink entries inside a read-only
  directory and swallows the error, `create_dir_all` succeeds because the
  directory exists, and `fs::write(&profile_path, malformed).expect(...)` then
  panics pointing at the wrong thing. One failure becomes a permanently red
  test that needs a human to `chmod` a directory under `/tmp`.
  Move the chmod back to `0o755` to immediately after `Profile::load()`
  returns, before any assertion. The test's meaning is unchanged — the
  suspension flag is already set by then, and the later saves still have to be
  blocked by the flag rather than by permissions, which is the whole point of
  the existing chmod-back. Say in a comment why it is there rather than at the
  end.
  Do not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `cargo test --test profile_save_suspended`; `git diff --stat` shows
  only that file; the report confirms the test still fails if the suspension
  guard in `Profile::save` is commented out (run it, show it).*

## Phase 4 — The data notice (walkthrough: damage the profile, the match save, and both at once in a scratch data directory, and see one notice at the start menu naming what happened and where the old file went)

- [x] **T008** — `src/app.rs` + `Readme.md`: the notice. Per plan §Design 7:
  `Modal::DataNotice(Vec<String>)` with the plan's doc, placed beside
  `Modal::Victory`; the free fn `data_notice_lines(profile:
  Option<&ProfileFailure>, save_unreadable: bool) -> Option<Vec<String>>`
  beside `victory_notice_lines`, with the plan's exact wording and its
  breathing-room comment; in `App::new`, drop the two `_` placeholders from
  T005/T007 and set `modal:
  data_notice_lines(profile_failure.as_ref(), save_unreadable).map(Modal::DataNotice)`;
  a draw arm `Some(Modal::DataNotice(lines)) => self.draw_notice(frame, lines)`
  beside the `Victory` arm; an input arm beside the `Victory` one, `matches!`
  + `notice_dismissed(key)` → `self.modal = None; self.audio.play(Sfx::MenuSelect)`.
  Import `ProfileFailure` and `ProfileProblem`. **No `resize` arm** (the lines
  are carried; `draw_notice` re-sizes from `self.config` each draw), **no new
  `App` field**, and no change to `has_save`, the menu or any `save()` call
  site. In `Readme.md`, the two sentences in plan §Design 8, appended to the
  **Saved data** paragraph before the `KAAZAP_DATA_DIR` example; do **not**
  document `KAAZAP_CRASH_AT` anywhere in the README. Test (plan §Tests), in
  `app.rs`'s tests module:
  `the_data_notice_reads_right_breathes_and_fits_the_minimum_terminal` —
  **pure, calling `data_notice_lines` directly; it must not construct an
  `App`** — covering the `None` case, the four wording cases, both failures in
  one notice, and, for the longest content at both `Config::fit_sizes()`, the
  breathing-room and unclamped-`OverlayLayout` assertions. Do not run `cargo
  fmt`. (Copies: `app.rs`'s own `victory_notice_lines` and
  `both_notices_read_right_breathe_and_fit_the_minimum_terminal` — the fit and
  breathing assertions lift almost verbatim; the `Modal::Victory` input arm for
  the three-line dismissal.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new test passing; `git diff --stat` shows only `src/app.rs`
  and `Readme.md`; `grep -rn "KAAZAP_CRASH_AT" Readme.md` is empty; the
  implementer's report quotes `data_notice_lines` and the input arm verbatim.
  **The Phase 4 review must enumerate**, reading `handle_key` from the top
  down, exactly which keys reach an open modal and what each does — not assert
  that the set is `q` and `m` (plan §Design tension 8). `?` is the one that
  would hurt: it lives inside the branch that runs only when no modal is open
  (`app.rs:1169-1188`), so it cannot reach the notice today, but if it ever
  moved ahead of the modal chain, opening and closing help would set
  `self.modal = None` and lose the notice permanently, with nothing to bring it
  back.
  **PAUSE for the person** (after the Phase 4 review): the orchestrator drives
  the Phase 4 walkthrough in plan §Verification with the `run-kaazap` skill —
  `KAAZAP_DATA_DIR` at a scratch directory, confirmed in the report, the real
  data folder never in play — and reports in plain language what the six
  scenarios (a)–(f) showed: a clean first launch saying nothing; a damaged
  profile named in the notice with the dated file it was kept under, still on
  disk afterwards and not overwritten by playing on; a wrong-version profile
  reading differently; a damaged match save reported once with no Continue and
  silence next launch; both damaged at once in one notice that fits an
  89-column terminal; Enter, Space and Esc dismissing it, and the keys the
  review enumerated — at minimum `?`, `L`, `m`, an arrow, a digit and a letter
  — doing nothing that loses the notice or acts on the menu underneath.*

- [x] **T008a** — `src/app.rs`: give the `?` ordering constraint a durable
  form. Logged 2026-09-19 from the Phase 4 review (non-blocking B).
  The review enumerated the keys and confirmed the set is exactly `q`
  (`main.rs`, before `App`), `m` (ahead of the modal ladder), and
  Enter/Space/Esc in the notice's own arm — nothing else acts. But it also
  found that **the entire protection for `?` is positional**: it is handled in
  the terminal `else` block that runs only when `self.modal.is_none()`, and
  `m` sits ahead of the ladder precisely because someone wanted it global, so
  `?` is the obvious next candidate for the same treatment.
  If `?` ever moved ahead of the ladder, opening help would overwrite
  `Modal::DataNotice(Vec<String>)` and closing it would set `self.modal =
  None`. **The loss is permanent and unrecoverable**: `data_notice_lines` is
  called from exactly one site, its inputs are consumed in `App::new` and
  deliberately not stored (the no-new-field decision), and by the time the
  notice is up the on-disk conditions are already repaired — the profile
  renamed, the match save deleted — so re-running the load would report
  nothing wrong. The player would never learn their campaign had been reset.
  The mitigation the plan chose (§Design tension 8) was "the Phase 4 review
  enumerates", and that expires the moment the review is read. Add a one-line
  comment at the `?` site saying it must stay inside the no-modal branch
  because `Modal::DataNotice` cannot be rebuilt. **A comment, not a test**: a
  test would have to construct an `App`, which is the one thing plan §Design
  tension 9 forbids, and `App::new` now reads *and repairs* the real data
  directory.
  Also append one sentence to plan.md §Design tension 8 recording that the
  ordering constraint was carried past the review into the code.
  Do not run `cargo fmt`.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/app.rs` and
  `specs/028-crash-and-data-safety/plan.md`; the report quotes the comment and
  the five lines around it.*

## Final phase — Spec close-out

- [ ] **T009** — Close-out. Draft
  `specs/028-crash-and-data-safety/closeout-main-docs.md` in spec 027's shape:
  **ROADMAP** — mark crash & data safety shipped as spec 028 (`grep -n -i
  "crash\|data safety\|whole-file\|corrupt" ROADMAP.md`, read and judge), and
  amend the path-injection seam's **still open** follow-up (line ~741) to note
  that `App::new` can now *move* an unreadable profile **and *delete* an
  unreadable match save**, so a `cargo test` run on a machine with a damaged
  data folder changes it — pointing the seven `app.rs` tests at a scratch root
  matters more than it did; **DECISIONS** —
  the spec's rulings (Q1 a set aside under a dated name; Q2 a a modal on the
  start menu; Q3 b the notice says why; Q4 a a bad match save gets a line and
  no kept file; Q5 a one kaazap line plus the panic's own message), and the
  plan's design calls: §1 (a recording panic hook plus a restoring guard, and
  the drop-order rule in `main`), §2 (`KAAZAP_CRASH_AT`, a seam that ships in
  the binary and is deliberately undocumented for players), §3 (one
  `write_whole` in `paths.rs`, a fixed temp name, **and no fsync** — with the
  reason, so a later power-cut question finds it), §4 (a process-scoped
  suspension flag rather than a field), §6 (a hand-rolled UTC stamp, no date
  crate), §8 (`q` and `m` under the notice — **record what the person actually
  ruled**, and the `spec.md` amendment that went with it). Describe the
  terminal guarantee as **the endings `spec.md` enumerates** — `q`, an input
  error, a panic on either thread — never as "however kaazap ends": a
  `SIGTERM` or `kill -9` still leaves the terminal unrestored, and the driver's
  kill-to-leave-a-save trick shows it. Signals are outside this spec, and the
  close-out is where that would otherwise be overclaimed. All of it applies on
  `main` after the merge, never on the branch. Run `cargo test -q` three consecutive
  times and paste the tails. Mechanical checks (three-dot, since `main` may
  move): `git diff main...HEAD --stat` lists none of `src/card.rs`,
  `src/game.rs`, `src/player.rs`, `src/opponent.rs`, `src/economy.rs`,
  `src/campaign.rs`, `src/wager.rs`, `tests/balance.rs`, `Cargo.toml`,
  `Cargo.lock`; `grep -n "VERSION" src/save.rs src/profile.rs` still reads 1
  and 1; `grep -rn "serde(default" src/profile.rs src/save.rs src/settings.rs`
  matches `main`'s output exactly; `grep -rn "fs::write" src/` matches only
  `src/paths.rs`;
  `cargo build --all-targets` warning count equals `main`'s. Check off
  `spec.md`'s 17 acceptance criteria with evidence (the T002 and T008
  walkthrough reports are the evidence for the terminal, crash and notice
  criteria). Request the pre-merge sweep; apply `closeout-main-docs.md` on
  `main` after the merge. Never chain a file edit, a branch switch and a commit
  in one shell command (spec 025's miss).
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/028-crash-and-data-safety/{spec,plan,tasks}.md`,
then implement from the first unchecked task. Involvement level is **product
owner**: the person owns `spec.md`, attests by using the app at phase pauses,
and receives spec-conformance summaries — not architecture reviews.

**Model policy — the economy profile** (this project's since 2026-09-19, and
this is the first spec under it): every row of the role table resolves to
`opus` / `claude-opus-5`, **no dispatch carries a per-call model override**,
the session runs at `claude-opus-5` medium from `.claude/settings.json`, and
the close-out (T009) goes to `sdd-implementer` like every other task. Nothing
runs on Fable; do not move a role back to it unless the person says so.

Dispatch each task to `sdd-implementer` on a shell-assembled task bundle (task
line, plan section, acceptance criteria, files, the pattern file to copy).
Verify from the implementer's verbatim output, except **T003 and T005
(`review: per-task`)**: the orchestrator re-runs the verification command
itself and dispatches a `skeptical-reviewer` on that task's diff alone before
the next task starts.

**Foundational phases: Phase 2 and Phase 3.** One `skeptical-reviewer` pass at
the end of each phase — after T002, T004, T007 and T008 — on a shell-assembled
bundle (the phase diff, the task lines, plan §Design and §Tests, the acceptance
criteria), one review plus at most one re-review. The Phase 4 review also
checks the notice against the constitution's *acted-on element stands apart*
rule and the modal's even padding.

**Pause cadence**: pause for the person after **Phase 1** and **Phase 4**, once
each phase's review and its walkthrough are done. Phases 2 and 3 are marked
`walkthrough: none` — nothing they change is observable in the app — so they
run through without a pause; their criteria are pinned by tests.

**Every driver session runs with `KAAZAP_DATA_DIR` pointed at a scratch
directory**, and the report says which. This spec's walkthroughs deliberately
damage profile and save files; the real data folder must never be in play, and
no backup-and-restore dance substitutes for the environment variable here.

Repo-wide docs (`ROADMAP.md`, `DECISIONS.md`) change only via
`closeout-main-docs.md` on `main` after the merge; `Readme.md` rides in on the
branch (T008). Never run `cargo fmt`. A task that isn't routine goes to a
decision review (`skeptical-reviewer`, top tier — which on this profile is the
same model, no override); a product question `spec.md` doesn't settle goes to
the person. A phase pause ends with what to check and how to say continue — no
continuation prompt unless the person asks or says they're stopping.

---

## Notes for the close-out (T009), gathered during implementation

These came out of per-task and phase reviews as non-blocking observations. They
are not code changes; they belong in `closeout-main-docs.md`'s DECISIONS entry
or in the roadmap follow-up, so the next person to touch this code finds them.

- **`write_whole` is a convention, not an enforced rule.** The three writers go
  through it; nothing in the test suite would catch a fourth writer calling
  `fs::write` directly. The `grep -rn "fs::write" src/` check is a one-shot at
  merge, not a regression guard, and it would not catch a writer built from
  `File::create` + `write_all` or `serde_json::to_writer` (Phase 2 review N6) —
  T009's footprint grep should widen to match those too. Say so, so the next
  writer knows the rule.
- **Two kaazap processes writing at once is the one case the fixed temp name
  doesn't cover.** Both would write the same `<stem>.tmp`, and an interleaving
  can splice them before either renames. The plan bought this deliberately —
  unique temp names would trade a rare two-instance corruption for certain
  debris accumulation across crashes — but it is a known limit, not an
  oversight, and should be named as one.
- **Write-then-rename changes two things `fs::write` did not**: the file gets
  fresh permission bits at the default umask rather than keeping the ones it
  had, and a symlink at the target is replaced rather than written through.
  Neither matters for a game's saves; both are inherited by every future
  writer.
- **`App::new` now repairs the real data directory, and five `app.rs` tests
  still carry comments saying it doesn't.** It renames a damaged profile and
  deletes an unreadable match save, and since T008 it also sets `self.modal`
  from the result — so three tests that call `App::new` then `draw` without
  overwriting `app.modal` would get a notice box over the board and fail their
  row assertions, on a machine whose data folder happens to be damaged. Before
  T008 the modal was unconditionally `None` at construction. This is the
  roadmap follow-up T009 already amends; the stale comments are worth naming
  with it. Related trap for anyone testing by hand: **never run `cargo test`
  from a shell with `KAAZAP_DATA_DIR` exported at a fixture directory** — it
  repairs the scenario you were about to test.
- **`payload_as_str` pins the crate to Rust >= 1.91** with nothing in the tree
  saying so (AC 17 forbids adding `rust-version`). Worth a line.
- **The guard's `Drop` join is only bounded because `render_tx` is declared
  after `_terminal`.** Hoisting the channel above the guard would deadlock
  every ending. Both comments in `main.rs` say so; the close-out should too.
- **`eprintln!` inside `TerminalGuard::drop` can panic on a closed stderr**
  (`kaazap 2>&1 | head -1`), and a panic inside a `Drop` during unwinding
  aborts. `restore_terminal` is unwrap-free by design; this is the one
  unguarded panic site left on that path. Not covered by any acceptance
  criterion — record it rather than fix it under this spec.
- **The `.context("kaazap couldn't read the terminal")` on `event::poll`/
  `event::read` is never executed** by any test or by the walkthrough: the
  `KAAZAP_CRASH_AT=input` seam bails before `poll`. The close-out must not
  claim the colon-detail form was observed.

## Tier log (this spec, under the model policy)

<!-- One row per implementer run and per reviewer invocation; a summary row per
phase the orchestrator fills at that phase's review. Record token usage from
each subagent return and any escape-hatch miss (a task the orchestrator had to
redo, and why). The constitution asks for per-run token logging for the first
spec under a policy — this is it for the economy profile. -->

| Task / invocation | Tier (dispatched → ran) | Tokens | Dispatches | First try | Blocking findings | Outcome / miss reason |
|---|---|---|---|---|---|---|
| **Economy profile, first spec under it** — adopted 2026-09-19 (recorded in `CLAUDE.md`'s History, no spec in flight at the time). Every role resolves to `opus` / `claude-opus-5`; no per-call override anywhere, including the planner, the sign-off and the close-out; session at `claude-opus-5` medium. Compare against spec 027's log (planning + sign-offs ≈ 651K at fable, implementer 8 dispatches ≈ 504K, reviewer ≈ 486K) and spec 025's (the whole spec on Opus at the person's choice). | — | — | — | — | — | header |
| Planning: draft (sdd-planner) | opus → opus | 225K | 1 | — | — | drafted; 9 tasks in 5 phases, Phases 2 and 3 foundational, T003 and T005 `review: per-task`; no product question; 6 design choices flagged for sign-off |
| plan + tasks sign-off (skeptical-reviewer) | opus → opus | 126K (82K in / 9K out over both passes) | 2 (review + re-review) | — | 3 (B1: T004's integration test breaks T005's build and its stated footprint; B2: `write_whole` lands with no caller and is `dead_code` against its own no-warnings bar; B3: `q`/`m` under the notice contradicts an acceptance criterion — **with the person**) + S1–S8 | B1 and B2 applied by the planner (T003 now carries the three call sites, T004 is the test alone, T005 owns the test's one-line update); B3 ruled by the person as Q6 (both keys stay live) and `spec.md` amended; S1–S8 folded in. Re-review: every finding fixed, nothing new blocking, **signed off**. Its one non-blocking sweep item — three passages still describing B3 as open — was applied by the orchestrator in the same commit as this row. |
| Planning: sign-off notes (sdd-planner, same context) | opus → opus | 39K harness-measured (264K cumulative for the planner across both dispatches, less the 225K first pass; the planner's own estimate for the revision was ~25K) | 1 | yes | — | B1, B2 and S1–S8 applied. Premise confirmed by the reviewer and now recorded in plan tension 1: `Cargo.toml` sets no `panic = "abort"`, so `Drop` runs in release and the Phase 1 design holds. Both files still Draft |
| T001 (sdd-implementer) | opus → opus | 38K | 1 | yes | — | `src/crash.rs` + `pub mod crash;`; both named tests pass; toolchain is rustc 1.91.1 so `payload_as_str` is in (no `rust-version` added) |
| T002 (sdd-implementer) | opus → opus | 38K | 1 | yes | — | `TerminalGuard` + `crash_if_requested` in `main.rs`; teardown greps empty; no `panic = "abort"` in `Cargo.toml` re-confirmed. Note for the walkthrough: under `=key`, `q` still quits cleanly (the `q` arm precedes the seam) — press another key to crash |
| Phase 1 review (skeptical-reviewer) | opus → opus | 61K | 1 | — | 0 blocking | Signed off. Non-blocking: the pre-loop forced render is outside the in-loop crash check and is the real window (plan tension 1's model corrected); the drop-order comment over-claims; `eprintln!` in `Drop` can panic on a closed stderr; `payload_as_str` now pins Rust >= 1.91 with nothing saying so. Predicted the `=tick`/`=draw` garbling the walkthrough then confirmed |
| Phase 1 walkthrough (orchestrator) | — | — | 5 points x 6 runs + quit | — | — | `q` exit 0 silent; `=key`, `=render` exit 101 clean; `=input` exit 1, `Error: kaazap couldn't read the terminal`, no panic text; **`=tick` and `=draw` garbled 6/6 each** -> T002a logged. Cursor visible in every run. Scratch `KAAZAP_DATA_DIR` throughout |
| T002a (sdd-implementer) | opus → opus | 40K | 1 | yes | — | Guard owns `Option<JoinHandle>`, joins before restoring; `main` keeps its own `join().unwrap()` for AC 3's exit 101; plan tension 1's two false sentences and its Rejected entry corrected. **Re-walkthrough: 33 runs (3 quit + 6 each of tick/draw/input/key/render) all CLEAN, cursor visible, exit codes 0/101/101/1/101 as the plan's table** |
| T003 (sdd-implementer) | opus → opus | 48K | 1 | yes | — | `paths::write_whole` + the three one-line call sites; 3 tests; zero warnings, no `dead_code`; `fs::write` now only in `paths.rs`. Orchestrator re-ran verification (per-task): green |
| T003 per-task review (skeptical-reviewer) | opus → opus | 44K | 1 | 0 blocking | 0 blocking | Signed off. Walked every failure mode; guarantee holds. Non-blocking: **the accumulation test is vacuous** (the temp-path-as-directory trick fails before `fs::rename`, so the cleanup line is never executed) -> **T003a** logged; plus 5 close-out notes, now in *Notes for the close-out* above |
| T003a (sdd-implementer) | opus → opus | 32K | 1 | yes | — | Fourth test reaches `fs::rename`'s failure branch (target-as-directory). Demonstrated failing with the cleanup line commented out (`the temp file was left behind: ["data.json", "data.tmp"]`) and passing with it restored. Doc comment now names `fsync` and cites §Design tension 3 |
| T004 (sdd-implementer) | opus → opus | 54K | 1 | yes | — | `tests/whole_file_write.rs`, one test, scratch root under `temp_dir()`, removed after. Mutation-checked: with the blocking directories removed it fails on the settings byte comparison, so the assertions bite. Two `Profile::load()` call sites for T005 to update |
| Phase 2 review (skeptical-reviewer) | opus → opus | 53K | 1 | — | 0 blocking | Signed off, clean. T003a confirmed to close its finding. Non-blocking N1 (the mutation check stops at the first `assert_eq!`, so it evidenced 1 of 6 assertions) and N2 (AC 7's *debris is never loaded* clause is argued, not pinned) -> **T004a**. N3–N6 (no entry-set check after the failed saves; fixed scratch names; leaked dir on a red run; the `fs::write` grep wouldn't catch `File::create`+`write_all`) recorded, N6 into the close-out notes |
| T004a (sdd-implementer) | opus → opus | 48K | 1 | yes | — | N2 closed for settings: a real `settings.tmp` holding garbage is neither read nor consumed by `Settings::load`. N1 closed properly: reverting each writer in turn fires **that writer's own** comparison, at lines 108/109/110. `src/` byte-identical after (SHA-1s checked). Note: the three loader assertions sit behind the byte comparisons and can only be shown live by mutating a loader, not a writer |
| T005 (sdd-implementer) | opus → opus | 89K | 1 | yes | — | `ProfileProblem`/`ProfileFailure`, `load -> (Self, Option<ProfileFailure>)`, `SAVES_SUSPENDED`, `set_aside`/`set_aside_names`/`utc_stamp`/`civil_from_days`; `from_json` kept as a `#[cfg(test)]` wrapper so all 36 `profile.rs` tests stand unedited. Orchestrator re-ran verification (per-task): green |
| T005 per-task review (skeptical-reviewer) | opus → opus | 50K | 1 | 0 blocking | 0 blocking | Signed off. Walked every `load` path: no input or filesystem state loses the file; first launch still exactly silent (incl. Windows `ERROR_PATH_NOT_FOUND`); suspension set on exactly the right condition and the guard genuinely first; `classify` decision-for-decision the old gate; Hinnant's civil-from-days verified longhand on all three vectors. Second-look 1/3/4 -> **T005a**; 2 already handled (T006's suspension test is its own binary); 6 = the task line's call-site count, corrected above |
| T005a (sdd-implementer) | opus → opus | 52K | 1 | yes | — | `classify_tells_a_bad_document_from_a_bad_version` pins the variant split the `from_json` wrapper hid; pre-epoch stamp vector added; both `whole_file_write.rs` call sites now assert `failure.is_none()`. `src/profile.rs` diff starts at line 1389, `#[cfg(test)]` at 615 — test module only |
| T006 (sdd-implementer) | opus → opus | 62K | 1 | yes | — | `tests/profile_recovery.rs` + `tests/profile_save_suspended.rs`, one test each, separate binaries (sticky flag). Both traps handled: chmod-back targets the set-aside path; the `0o000` step asserts the failure so it fails loudly under root. Real names produced: `profile-20260920-023255{,-2,-3}.json` — all three collided in one second, so the `-2`/`-3` branch is what ran. Deviation (an improvement): chmods the directory back before the last saves, so a suspended save is distinguishable from a permission-denied one |
| T007 (sdd-implementer) | opus → opus | 61K | 1 | yes | — | `save::check_at_launch` + the `app.rs` line before `has_save`; `tests/match_save_recovery.rs`. Deviation (an improvement): the wrong-version step edits a *real* save's version rather than using a bare `{"version": 2}` stub, which would fail serde's missing-field check and pass even with the version gate deleted. Note for T008: `app.rs` now has two adjacent identical `// T008 raises the data notice from this` comments — collapse them |
| Phase 3 review (skeptical-reviewer) | opus → opus | 77K | 1 | — | 0 blocking | Signed off, clean. T005a confirmed to close all three parent notes. Non-blocking: (3) **the plan's notice line "nothing from this session is kept" is literally false** — `save::save` and `Settings::save` still write under suspension; AC 11 only scopes to *profile* saves, so it is a wording problem, not a divergence -> constraint carried into T008's bundle; (4) `profile_save_suspended` poisons its scratch dir on a mid-test failure -> **T007a**; (2) T007's stated placement rationale is false, corrected above; (6) §Design tension 9 is now live and belongs in the person's phase report; (1) the orchestrator's diff base was one commit early (my error, harmless) |
| T007a (sdd-implementer) | opus → opus | 36K | 1 | yes | — | chmod-back moved ahead of the assertions; duplicate `save()` collapsed. Mutation-checked: commenting out the suspension guard fails it at line 74 (`a save wrote over the file that couldn't be read`). **The property was observed, not argued** — the passing re-run happened on the scratch root the failing run left behind, and needed no human `chmod` |
| T008 (sdd-implementer) | opus → opus | 71K | 1 | yes | — | `Modal::DataNotice` + `data_notice_lines` + draw/input arms + `Readme.md`; pure test at both `fit_sizes()`. **One line of the plan's wording replaced** on the orchestrator's constraint (Phase 3 review note 3): *"nothing from this session is kept"* was false — only `Profile::save` no-ops under suspension — now *"the campaign you play this session won't be kept"*. Widest line is 79 chars -> an 87-wide box at 89 columns |
| Phase 4 review (skeptical-reviewer) | opus → opus | 80K | 1 | — | 0 blocking | Signed off, clean. Enumerated the keys top-down: `q` (main.rs, before `App`), `m` (ahead of the ladder), Enter/Space/Esc in the arm — everything else a no-op, incl. `?`, `L`, `Q`, `M`, arrows, digits, and Ctrl+P/N/B/F (which arrive as arrows). Also verified shrink-below-minimum-then-grow preserves the notice, so the no-`resize`-arm call is right. Fit worked longhand: worst line 79 chars -> 87-wide box at 89 cols, 9 content lines -> 13 rows at 31 — **two characters of headroom**. Non-blocking B (the `?` guard is a review artifact with no durable form) -> **T008a**; A (the 2-char width budget), C (the title over-claims on the wrong-version path; plan-verbatim), D (`App::new` repairs the real data dir — close-out note), E (walkthrough wording: `m` mutes silently, it does not "do nothing") |
| T008a (sdd-implementer) | opus → opus | 41K | 1 | yes | — | Five comment lines at the `?` site + one sentence in plan tension 8; additions only, no logic touched. Finding for the sweep: plan tension 8 cites `app.rs:1169-1188`, now `1221-1235` — plan line citations have drifted as tasks landed |
| Phase 4 walkthrough (orchestrator) | — | — | 10 driver runs | — | — | All six scenarios pass, `KAAZAP_DATA_DIR` at a scratch dir throughout, real data folder never in play. (a) clean launch: no notice, nothing created. (b) malformed profile: notice names the dated file; kept bytes identical; playing a **whole match** afterwards left the kept file md5-identical with exactly one set-aside and a fresh `profile.json`. (c) `"version": 2`: the wrong-version wording. (d) malformed save: its line, no Continue, file removed, **next launch silent**. (e) both at 89x31: one notice, box 73 wide, nothing clipped. Move-failed case at 89x31: box **87 wide, 1 column margin each side** — exactly the review's arithmetic — file still there unchanged. (f) `?`, `L`, `m`, arrow, `3`, `x` left the screen byte-identical and the menu selection unmoved; Enter, Space and Esc each dismissed to the menu with the selection where it starts; `q` quit with exit 0 and a clean terminal. `m` **toggles mute silently** — it is not a no-op, it just has nothing to draw |
