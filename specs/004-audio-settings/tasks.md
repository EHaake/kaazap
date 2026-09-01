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

- [x] **T002 — `Settings` struct + JSON load/save**
  *Done. `settings.rs`: `Settings { music, sfx }` (serde derive, both
  default on) with `#[serde(default)]` per field so partial/older files
  still load (forward-compat for the save format). `load()` reads the
  `directories` config dir's settings.json and defaults on any error;
  `save()` is best-effort (creates the dir, swallows write failures);
  neither panics. A filesystem-free `from_json_or_default` core makes the
  fallback testable without touching the real config dir. Four
  `settings_` tests (default, round-trip, malformed→default, missing
  fields→on). Not yet wired to App (T006). 138 tests pass (+4), 0 warnings.*
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

- [x] **T003 — SFX assets + `Sfx` enum**
  *Done. `scripts/gen_sfx.py` (stdlib `wave`, deterministic) synthesizes
  11 short square/triangle-wave WAVs into `assets/sfx/` (~200 KB total,
  license-free). `Sfx` enum + `Sfx::bytes()` embeds each via
  `include_bytes!`. Tuning by ear deferred to T008.*
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

- [x] **T004 — `Audio` player: rodio backend, music loop, gating, fallback**
  *Done. Pinned to the rodio 0.22 API (`DeviceSinkBuilder::open_default_sink`
  → `MixerDeviceSink`; `mixer().add(source)` for fire-and-forget SFX; a
  `Player` via `connect_new`/`append`/`play`/`pause` for the loop;
  `Decoder::new` / `new_looped`). `Audio` holds `Option<Backend>` — a
  silent stub when `open_default_sink` fails (headless/no device), so the
  game always runs. Music loads from the runtime path `assets/music/
  theme.ogg` (silent if absent, until T008). `play`/`apply_music` gate on
  pure `should_play_sfx`/`should_music_sound(muted, settings)`, unit-tested
  across the mute × channel matrix. No `OutputStream` constructed in tests
  — device-free.*
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

- [x] **T005 — `audio_cues`: the pure state-diff → SFX mapping**
  *Done. `AudioSnapshot::of(&GameState)` captures both sides' dealer/played
  counts, bust/stood, last-played-is-flip, plus round outcome and game-over
  (with winner). `audio_cues(prev, curr) -> Vec<Sfx>` implements the
  priority rules — draw; play/flip; bust suppresses that side's stand;
  game-over replaces the round cue; tie is silent; both sides covered.
  Seven `cues_` tests cover the truth table (incl. no-repeat once a flag is
  set). Pure, no audio, `game.rs`/`player.rs`/`card.rs` diff vs main empty.
  146 tests pass (+8 across T004/T005), 0 warnings.*
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

- [x] **T006 — Settings screen, menu entry, and `App` audio/settings ownership**
  *Done. `Screen::Settings`; `settings.rs` gains `SettingRow`,
  `SettingsAction`, and `SettingsState` (input ↑/↓+w/s move, Enter/Space
  toggle, Esc back) with a centered monochrome draw (title, `▸`+pulse on
  the selected row, On/Off values, a controls hint). `App` loads `Settings`
  and constructs `Audio` in `new`, routes the screen, and on a toggle
  updates settings, calls `audio.set_settings`, and saves. `menu.rs` gains
  `MenuItem::Settings` + real wrapping N-item navigation (retiring the
  2-item toggle). `?` is a no-op on the settings screen. Verified in-app:
  Menu → Settings opens the centered screen, Music/SFX toggle
  independently, the change persists across a fresh relaunch
  (`settings.json` = `{"music": false, "sfx": true}`), Esc returns to the
  menu. Audio still inaudible (SFX cues are T007; music needs the T008
  track). Engine untouched. 147 tests pass (+1 menu nav), 0 warnings.*
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

- [x] **T006a — Fix startup blank: defer the audio device open past the first paint**
  *(human-reported: blank screen at launch, "noticeably longer than
  before".)* T006 opened the rodio device inside `App::new`, which runs
  *before* the render thread and first paint in `main.rs` — so startup sat
  on a blank alternate screen until device init finished (longer on some
  machines). Split `Audio::new` (instant, no device) from `Audio::open`
  (opens the device + reconciles music); `main.rs` now draws and sends the
  first menu frame, then calls `app.open_audio()`. The menu paints
  instantly; the device opens right after. Bonus: `cargo test` no longer
  opens any device (only `open` does, and tests only call `new`) — test
  time dropped 0.75s → 0.01s. Verified in-terminal: menu appears
  immediately, game plays. 147 tests pass, 0 warnings.*

## Phase 5 — Cues & mute wiring (`app.rs`)

Review: after the phase.

- [x] **T007 — Global mute + in-game audio cues**
  *Done. `App` intercepts `m` before per-screen routing → `audio.toggle_mute`
  (works on every screen and under an overlay). `App` keeps
  `prev_audio: Option<AudioSnapshot>` and calls `emit_audio_cues` after
  every input (`handle_key`) and every `tick` update — diffing the snapshot
  and playing `audio_cues`, so both the player's and the opponent's moves
  (and round/game resolutions from `update`) sound. `prev_audio` resets to
  None on a fresh game so the empty starting board is silent. Menu and
  settings navigation play `MenuMove`/`MenuSelect` (the settings toggle
  sound comes after `set_settings`, so muting SFX silences its own
  confirm). Disjoint-field borrows kept it inside the existing match arms.
  Smoke-tested: a full round plays to the outcome popup with SFX firing in
  the pty, stable. By-ear verification (do they sound right, does `m`
  silence) is the human's. Engine untouched. 147 tests pass, 0 warnings.*
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

- [x] **T007a — Move audio onto its own thread (fix the first-run freeze)**
  *(human-reported: first run after a rebuild froze the menu ~10s with
  inputs queuing, then firing all at once; later runs fast.)* T006a's
  `open_audio` still ran on the main thread, so the device open — which
  cold-starts for seconds the first time a fresh binary touches the OS
  audio stack — froze the input loop after the menu painted. Fixed
  properly: `Audio` is now a thin handle over an `mpsc::Sender<AudioCommand>`;
  a dedicated **audio thread** owns the rodio backend (which is `!Send`, so
  it must stay on one thread), opens the device there, and serves
  PlaySfx/SetSettings/ToggleMute commands. `Audio::new` just spawns the
  thread and returns instantly; every call is a non-blocking send. Removed
  the `open`/`attempted` split and the `main.rs` paint-first-frame
  workaround. Verified in-app: navigating the menu the instant it appears
  works (no freeze), a game plays, `m` mutes. 147 tests pass, 0 warnings;
  tests still open no device (they construct the handle and drop it).*

## Phase 6 — Assets & acceptance

Review: after the phase.

- [ ] **T008 — Music track, credits, README, acceptance sweep + review**
  *In progress. **Music:** human chose (from sourced, license-verified
  candidates) Kevin MacLeod's CC-BY "Chipper Doodle" — bundled as
  `assets/music/theme.mp3` (MUSIC_PATH updated; MP3 is a default rodio
  decoder, verified it decodes), with `assets/CREDITS.md` + a
  `DECISIONS.md` note recording the attribution. Actual Star Wars/KOTOR
  music (and 8-bit covers) declined as copyrighted — attribution isn't a
  license. **SFX** tuned per human feedback: a distinct `RoundTie` sound
  (was silent) and the opponent's actions pitched down (`OPPONENT_PITCH`)
  so player/opponent read apart by ear; kept the bust. **Docs:** README
  ALSA note refreshed for rodio + credits pointer; ROADMAP (main) gains an
  "original cantina-vibe music" item (human-requested). **Acceptance
  sweep:** every spec.md box checked with evidence — 147 tests, 0 warnings,
  no copyrighted audio filenames, engine untouched, end-to-end (menu →
  settings → game → mute) verified in-app. Remaining: the skeptical-
  reviewer pass (offered) and marking the PR ready, on the human's word.*
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
