# Tasks: Audio & Settings

**Status**: Draft (in review)
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task is completable and
testable on its own; resume at the first unchecked task.

<!-- WARNING, worth leaving in: once implementation starts, this file
gets written by more than one party. Never edit it from a stale copy;
prefer small targeted edits over regenerating it wholesale. -->

Per the constitution: every task ends with `cargo build` and `cargo test`,
both green, actual output reported. *Verify:* lines are the task-specific
checks on top of that baseline. One commit per task, referencing the task
ID; push after each commit — open the draft PR when T001 lands.

**Review cadence**: Phases 1–3 (deps + amendment, persistence, audio
core) are foundational — new subsystems and a new save layer — so stop
for review after **every task**. Phases 4–6 (settings screen, wiring,
assets/acceptance) are more mechanical — stop after each **phase**.

**Scope guard**: `game.rs`/`player.rs`/`card.rs` are **not** modified —
the engine makes no sound. Audio is triggered at the `App` layer by
observing state (`audio_cues`). A task that needs to edit the engine has
gone off-plan and stops for review.

**Audio can't be pty-tested** — the run-kaazap driver can't hear. Sound,
looping, and the mute key are verified by ear in a real terminal and
reported; all the *logic* (cues, gating, persistence) is unit-tested.

---

## Phase 1 — Amendment & dependencies (`CLAUDE.md`, `Cargo.toml`)

Review: after the task.

- [x] **T001 — Constitution amendment + dependency swap**
  *Done. CLAUDE.md dependency note amended (rusty_audio → rodio; dropped
  the obsolete rusty_time line) and committed straight to main, then
  merged into the branch. Cargo.toml (via cargo add/remove): removed
  rusty_audio; added rodio 0.22.2, serde 1 (derive), serde_json 1,
  directories 6. Deps compile (rodio via coreaudio on macOS), 134 tests
  pass, 0 warnings. Note: rodio 0.22 is recent — its exact OutputStream/
  Sink API is pinned at T004.*
  Two parts. (a) Amend `CLAUDE.md`'s dependency note: `rusty_audio` is
  dropped in favor of `rodio` (looping music + pause/volume need it
  directly; one backend beats two contending output streams) — commit
  **straight to `main`** in its own commit, per the constitution's rule.
  (b) On the branch, `Cargo.toml`: remove `rusty_audio`, add `rodio`
  (current stable), `serde` (derive feature), `serde_json`, `directories`.
  Push the branch and open the draft PR.
  *Verify: `cargo build` resolves the new deps with no errors; `grep
  rusty_audio Cargo.toml src/` is empty; `cargo test` still green;
  `git log main -1` shows the amendment commit on main.*

## Phase 2 — Persistence (`settings.rs`)

Review: after every task.

- [ ] **T002 — `Settings` struct + JSON load/save**
  New `settings.rs`: `Settings { music: bool, sfx: bool }` with
  `serde` derive and `Default` (both true). `Settings::load()` reads
  `directories::ProjectDirs::from("", "", "kaazap")` → `config_dir()/
  settings.json`, parses via `serde_json`, and returns `Settings::default()`
  on **any** error (no dir, missing file, malformed JSON) — never panics.
  `save(&self)` creates the config dir if needed and writes the JSON; a
  write failure is swallowed, not fatal. Not yet wired to `App`.
  Tests (`settings_`): serde round-trip (`Settings` → string → equal
  `Settings`); `load`-equivalent parse helper returns defaults for
  malformed and empty input; default is music+sfx on.
  *Verify: `cargo test settings_` green; no panic path in load/save.*

## Phase 3 — Audio core (`audio.rs`, `assets/sfx/`)

Review: after every task.

- [ ] **T003 — SFX assets + `Sfx` enum**
  Add a small committed generator (`scripts/gen_sfx.py`, stdlib `wave` —
  short square/triangle blips with envelopes, license-free) that writes
  one WAV per effect into `assets/sfx/`. Add the `Sfx` enum in `audio.rs`
  (CardDraw, CardPlay, Flip, Stand, Bust, RoundWin, RoundLoss, GameWin,
  GameLoss, MenuMove, MenuSelect) and a `Sfx::bytes()` mapping each to its
  embedded clip via `include_bytes!`. (If generated blips read poorly by
  ear in T008, a CC0 SFX pack is the fallback — same enum, swapped files.)
  *Verify: `cargo build` (the `include_bytes!` paths all resolve);
  `ls assets/sfx/` shows a WAV per `Sfx` variant; the generator script
  is committed and re-runnable.*

- [ ] **T004 — `Audio` player: rodio backend, music loop, gating, fallback**
  `Audio` owns a rodio `OutputStream` + handle, a looping music `Sink`,
  the embedded SFX bytes, the current `Settings`, and a `muted` flag.
  `play(Sfx)` decodes the clip and plays it on a fresh detached sink,
  no-op unless `!muted && settings.sfx`. Music loads from a runtime path
  (`assets/music/…`, silent if absent — decouples the audio core from the
  yet-unsourced track) and loops via `repeat_infinite`; `apply_music()`
  reconciles the sink (play when `!muted && settings.music`, else pause).
  `set_settings` / `toggle_mute` update state then `apply_music`. If
  `OutputStream::try_default()` fails, `Audio::new` returns a **silent
  stub** whose methods no-op. Pin the exact rodio API to the resolved
  version.
  Tests (`audio_`): a pure `should_play_sfx(muted, settings)` /
  `should_music_sound(muted, settings)` helper across the full mute ×
  channel matrix. (No `OutputStream` is constructed in tests — device-free.)
  *Verify: `cargo test audio_` green; `cargo build` clean; real-terminal
  smoke deferred to T007/T008 (nothing calls `Audio` yet).*

- [ ] **T005 — `audio_cues`: the pure state-diff → SFX mapping**
  `AudioSnapshot` (per-side dealer/played counts, bust/stood, last-played-
  is-flip; plus `outcome` and `game_over`) with a `from(&GameState)`
  constructor, and `audio_cues(prev, curr) -> Vec<Sfx>` implementing the
  plan's priority rules (draw; play/flip; bust suppresses that side's
  stand; game-over replaces the round cue; tie is silent; both sides
  covered).
  Tests (`cues_`): the full truth table — dealer draw, card play, flip
  play, stand, bust-suppresses-stand, opponent-side draw/play, round
  win/loss, tie (empty), game-over-replaces-round; and no-change → empty.
  *Verify: `cargo test cues_` green; `audio_cues` is pure (no `Audio`,
  no I/O); `git diff main -- src/game.rs src/player.rs src/card.rs` empty.*

## Phase 4 — Settings screen & menu (`settings.rs`, `screen.rs`, `menu.rs`, `app.rs`)

Review: after the phase.

- [ ] **T006 — Settings screen, menu entry, and `App` audio/settings ownership**
  `Screen::Settings { settings_state }`; `settings.rs` gains
  `SettingsState { selected: SettingRow }` (Music | Sfx), its input
  (↑/↓ + w/s move, Enter/Space toggle the row, Esc → menu), and its draw
  (centered block like the menu, `▸` marker + pulse, each row reading
  its On/Off state legibly). `App` gains `settings: Settings` (loaded in
  `new`) and `audio: Audio` (constructed in `new`), routes the Settings
  screen in `handle_key`/`draw`, and on a toggle updates `settings`, calls
  `audio.set_settings`, and `settings.save()`. `menu.rs`: `MenuItem::
  Settings`, real N-item up/down navigation (retiring the 2-item toggle,
  the file's own TODO), and `Activate { Settings }` → open the screen.
  Tests (`menu_`): N-item navigation (move/wrap or clamp across the three
  items) with `Settings` present.
  *Verify: `cargo test` green; in a real terminal — Menu → Settings opens
  a centered screen, Music/SFX toggle independently and audibly (music
  starts/stops), the toggle persists across relaunch, Esc returns to the
  menu; `git diff main -- src/game.rs src/player.rs src/card.rs` empty.*

## Phase 5 — Cues & mute wiring (`app.rs`)

Review: after the phase.

- [ ] **T007 — Global mute + in-game audio cues**
  `App` intercepts `m` **before** per-screen routing (global mute on every
  screen → `audio.toggle_mute`). `App` keeps `prev_audio: AudioSnapshot`
  and, after every state-changing step (a player action in `handle_key`,
  and each `tick`'s `update`), plays `audio_cues(prev, curr)` then stores
  `curr`. Menu/settings navigation and activation play `MenuMove`/
  `MenuSelect` where `App` already handles those actions.
  *Verify: `cargo test` green; in a real terminal — draws/plays/stand/
  bust/flip and round+game outcomes each sound (including the opponent's
  moves), `m` instantly silences all audio and restores it, menu/settings
  navigation clicks; `git diff main -- src/game.rs …` still empty.*

## Phase 6 — Assets & acceptance

Review: after the phase.

- [ ] **T008 — Music track, credits, README, acceptance sweep + review**
  Source a **CC0 / royalty-free chiptune** track (spacey-cantina feel),
  **verify its license**, and present a couple of candidates for the
  human to pick; drop the chosen one into `assets/music/` and confirm it
  loops cleanly by ear. Re-tune the generated SFX by ear (or swap in a CC0
  pack) if any read poorly. Add `assets/CREDITS.md` + a `DECISIONS.md`
  note recording every track/pack source + license. Add the Linux ALSA
  build note to the README. Walk every `spec.md` acceptance box with
  evidence (build/test output; real-terminal audio checks; persistence
  round-trip). Run the `skeptical-reviewer` over the branch; sub-letter
  any real findings. Mark the PR ready only on the human's word.
  *Verify: all `spec.md` boxes checked with evidence; the shipped track is
  CC0/royalty-free with its license recorded; `grep` finds no copyrighted-
  audio filenames; build/test output reported verbatim; reviewer findings
  resolved or ruled.*

---

## Handoff note

Read `CLAUDE.md` (post-amendment), `design/brief.md`, then this spec's
`spec.md`, `plan.md`, and this file. Implement in order from T001. **Stop
for review after every task in Phases 1–3; after each full phase for
Phases 4–6.** One commit per task referencing the task ID; push after each
commit — the draft PR (opened at T001) tracks the diff. Sub-letter
(`T00Xa`) any genuinely new scope and flag it. The engine scope guard
(no `game.rs`/`player.rs`/`card.rs` edits) applies to every task.
