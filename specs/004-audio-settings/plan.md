# Plan: Audio & Settings

**Status**: Approved
**Implements**: spec.md in this directory

Guiding constraints from `CLAUDE.md`: simplest design that satisfies the
spec; **game logic stays decoupled from rendering *and* audio** — the
engine (`game.rs`) neither draws nor makes sound, so audio cues are
emitted at the `App` layer by *observing* state changes, never by adding
calls inside `game.rs`; new top-level modes become new `Screen` variants;
persistence uses `serde`/`serde_json` + `directories`. One constitution
amendment is a prerequisite (below).

## Constitution amendment (prerequisite, its own commit to `main`)

`CLAUDE.md` currently says "Keep `rusty_audio` for sound … rodio if needs
ever outgrow it." Looping background music with pause/stop and volume is
exactly that outgrowing: `rusty_audio` is fire-and-forget with no loop or
pause, and running it *alongside* rodio means two output streams
contending for the device. A single **`rodio`** backend (one
`OutputStream`; a looping music `Sink`; short-lived sinks for SFX) is
genuinely cleaner. Human pre-approved. So before implementation, amend
that `CLAUDE.md` dependency note (rusty_audio → rodio, drop rusty_audio)
in its own commit straight to `main`, per the constitution's own rule.

## Dependencies

- **Add** `rodio` (current stable), `serde` (derive), `serde_json`,
  `directories`. **Remove** `rusty_audio`.
- `rodio` was already a transitive dep of `rusty_audio`; the others are
  the constitution's named persistence stack. No unsanctioned crates.
- Linux/CI needs ALSA dev libs to build rodio (`libasound2-dev`) —
  documented in the README (Goal 7).

## Core types

### Settings (`settings.rs`, persisted)

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub music: bool,
    pub sfx: bool,
}
impl Default for Settings { /* music: true, sfx: true */ }
```

Load/save helpers (persistence section). This struct is the seed of the
save format — the campaign spec extends it, it doesn't rebuild it.

### Sfx (`audio.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    CardDraw, CardPlay, Flip, Stand, Bust,
    RoundWin, RoundLoss, GameWin, GameLoss,
    MenuMove, MenuSelect,
}
```

Each maps to one bundled clip. (Round tie plays nothing — a non-event.)

### Audio (`audio.rs`)

```rust
pub struct Audio {
    _stream: OutputStream,          // kept alive; dropping it kills sound
    handle: OutputStreamHandle,
    music: Sink,                    // the looping music sink
    sfx: HashMap<Sfx, &'static [u8]>, // embedded clip bytes, decoded per play
    settings: Settings,
    muted: bool,                    // session master mute (the `m` key)
}
```

- `play(Sfx)` — no-op unless `!muted && settings.sfx`; else decode the
  clip bytes (`Decoder::new(Cursor::new(bytes))`) and play on a fresh
  detached sink so SFX can overlap and never block.
- `apply_music()` — the single place that reconciles the music sink with
  state: play/resume the looping track when `!muted && settings.music`,
  pause it otherwise. Called after any settings or mute change.
- `set_settings(Settings)` / `toggle_mute()` — update state, then
  `apply_music()`.
- Music loops via `Decoder…repeat_infinite()` appended to `music` once at
  construction; muting/disabling just pauses the sink (cheap, resumes in
  place).
- **Graceful no-audio fallback:** if `OutputStream::try_default()` fails
  (no device, headless CI), `Audio::new` returns a silent stub whose
  methods are no-ops — the game still runs. (Kept as an internal `Option`
  or an enum; the app never has to care.)

### Screen + SettingsState

```rust
// screen.rs
Settings { settings_state: SettingsState },
// settings.rs
pub struct SettingsState { selected: SettingRow } // Music | Sfx
```

## Audio cues — observing the engine, not touching it

The engine stays audio-free. `App` derives SFX from what *changed* in
game state, via a pure, unit-testable function over a small snapshot:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
struct AudioSnapshot {
    p_dealer: usize, p_played: usize, p_bust: bool, p_stood: bool, p_last_flip: bool,
    o_dealer: usize, o_played: usize, o_bust: bool, o_stood: bool, o_last_flip: bool,
    outcome: Option<RoundOutcome>, game_over: bool,
}
fn audio_cues(prev: AudioSnapshot, curr: AudioSnapshot) -> Vec<Sfx>;
```

Rules (cover both sides, so you hear the opponent play too), with
priority so one change makes one sensible sound:

- a side's `dealer` count rose → `CardDraw`
- a side's `played` count rose → `Flip` if that new card is a flip, else
  `CardPlay`
- a side's `bust` newly true → `Bust` (suppresses that side's Stand this
  transition)
- else a side's `stood` newly true → `Stand`
- `game_over` newly true → `GameWin`/`GameLoss` (by winner) — and this
  *replaces* the round cue
- else `outcome` newly `Some` → `RoundWin`/`RoundLoss` (tie → nothing)

`App` keeps `prev: AudioSnapshot`, recomputes `curr` after every
state-changing step (a player action in `handle_key`, and each `tick`'s
`update`), plays `audio_cues(prev, curr)`, then stores `curr`. Menu and
settings SFX (`MenuMove`, `MenuSelect`) fire directly where `App` already
processes those actions — they need no snapshot.

Because `audio_cues` is pure, its whole truth table is unit-tested with
no audio device.

## Settings screen (`settings.rs`)

Mirrors `menu.rs`: `SettingsState` owns the selected row; input maps
arrows ↑/↓ (and `w`/`s`) to move, Enter/Space to toggle the selected row,
Esc to return to the menu. Drawing reuses the shared vocabulary — a
centered block (same vertical-centering as the menu/board), the `▸`
marker + pulse on the selected row, monochrome. Each row reads
`Music    ◄ On ►` / `◄ Off ►` (or similar) so the state is unmistakable.
Toggling a row updates `App.settings`, calls `audio.set_settings(..)`
(music starts/stops immediately), and saves to disk.

## Start menu changes (`menu.rs`)

- `MenuItem` gains `Settings` (order: Start Game, How To Play, Settings).
- The 2-item `toggle_selected` becomes real up/down navigation over
  `MenuItem::iter()` (the file's own TODO) — clamp or wrap across N items.
  `MenuLayout` already takes the item count (spec 003), so centering just
  works with three.
- `MenuEvent::Activate { Settings }` routes in `App` to
  `Screen::Settings { settings_state: SettingsState::default() }`.

## Persistence (`settings.rs`)

- Save location via `directories::ProjectDirs::from("", "", "kaazap")` →
  its `config_dir()`, file `settings.json`.
- `Settings::load()` → read + `serde_json::from_str`; on *any* error
  (no dir, missing file, parse failure) return `Settings::default()` —
  never panic, never block startup. `settings.save()` → create the config
  dir if needed, serialize, write; a write failure is logged/ignored, not
  fatal (audio prefs aren't worth crashing over).
- Loaded once in `App::new`; saved on every toggle change.

## Architecture / flow changes

- **`app.rs`**: `App` gains `settings: Settings` and `audio: Audio`
  (loaded/constructed in `new`) and `prev_audio: AudioSnapshot`. New
  `Screen::Settings` arm in `handle_key` (nav/toggle/Esc) and `draw`.
  `handle_key` also intercepts `m` **before** per-screen routing (global
  mute), on every screen. After each state-changing action (player action;
  each `tick` `update`), emit `audio_cues`. Menu/settings actions play
  their `MenuMove`/`MenuSelect` directly.
- **`screen.rs`**: add the `Settings` variant.
- **`menu.rs`**: `Settings` item + N-item navigation (above).
- **`settings.rs`** (new): `Settings`, load/save, `SettingsState`, the
  settings-screen input + draw.
- **`audio.rs`** (new): `Audio`, `Sfx`, `audio_cues`, snapshot.
- **`main.rs`**: unchanged in shape — the game loop already owns `App`;
  audio lives inside it. (*Revised in T007a:* the plan first kept the rodio
  backend on the game-loop thread, but the device open blocked input on a
  cold start. The backend now lives on its **own** thread — `Audio` is a
  `Send` handle over an `mpsc::Sender<AudioCommand>` — so the `!Send`
  backend stays put on that one thread and never stalls the loop.)
- **Untouched: `game.rs`, `player.rs`, `card.rs`** — the engine makes no
  sound. `board.rs`/`frame.rs`/`layout.rs` unchanged (settings screen is
  its own small view; if it reuses a centered-block helper, that's a
  read-only borrow of existing layout).

### Startup audio latency (investigated, T009 follow-up)

The one blocking cost before sound starts is `open_default_sink()` — the OS
handing the process an output stream. Instrumented: `connect_new` and
`load_music` (the streamed 7.5MB MP3) are ~0ms/6ms; a **warm** open is
~0.5s. A **cold** open can take up to ~10s, and the dominant cause is
environmental, not ours: a **Bluetooth default output** (e.g. AirPods) has
to wake and connect its audio link on the first open — confirmed, the
delay is present on AirPods and gone on wired/built-in output. (CoreAudio
warming after idle is a smaller secondary factor.) There is no code fix —
nothing can play before the OS opens the stream — so the mitigation is the
T007a off-thread move: the wait never blocks input, only delays first
sound, cold-only. A normally-launched shipped build warms like any app.

## Assets

- `assets/music/…` — one bundled CC0/royalty-free chiptune track (sourced
  + license-verified in the design step, presented as candidates).
  Embedded via `include_bytes!` for a self-contained binary (chiptune is
  small; verify total binary size stays reasonable).
- `assets/sfx/*.wav` — one short clip per `Sfx`, CC0 or generated
  (e.g. via an sfxr-style tool; generated sounds carry no license
  encumbrance). Embedded via `include_bytes!`.
- `assets/CREDITS.md` (new) + a `DECISIONS.md` note — record every track's
  and pack's source and license.

## Known limitations

- **Audio is a side effect — verified by running, not asserted.** The
  run-kaazap pty driver can't hear sound; playback, looping, and the mute
  key are eye/ear-checked in a real terminal. The *logic* (cue mapping,
  gating, settings round-trip) is unit-tested.
- **rodio API churn across versions.** The exact `OutputStream` / `Sink`
  / `Decoder` calls are pinned to the chosen rodio version at
  implementation; the shapes above are the intent.
- **Headless/CI has no audio device** — the silent-stub fallback keeps
  `cargo test`/`cargo build` working without one (and ALSA dev libs are
  still needed to *compile* rodio on Linux).
- **`include_bytes!` music** trades a larger binary for a self-contained
  one; if a track is big, fall back to loading from an `assets/` path at
  runtime (one-line change).

## Testing strategy

Unit tests target the pure logic (constitution):

- `audio_cues`: the full truth table — a dealer draw, a card play, a flip
  play, a stand, a bust (suppressing stand), both sides, a round
  win/loss, a tie (silence), a game over replacing the round cue.
- Gating: `Audio::play` respects `muted`/`settings.sfx`; `apply_music`
  chooses play vs pause across the mute × music matrix (testable on a
  pure "should music sound?" helper without a device).
- Settings persistence: `serde_json` round-trip (`Settings` → string →
  `Settings`), and `load()` returns defaults for missing and malformed
  input (feed it a bad string / nonexistent path).
- Menu navigation over N items (wrap/clamp) with the new `Settings` item.
- Playback, looping, the mute key, and the settings screen's feel:
  verified by running in a real terminal, reported.

## Suggested phasing (detailed in tasks.md)

1. **Amendment + deps** — CLAUDE.md rusty_audio→rodio (own commit to
   main); Cargo.toml swaps. Foundational.
2. **Persistence** — `Settings` + load/save + tests. Review every task.
3. **Audio core** — `Audio`, `Sfx`, embedded clips, music loop, gating,
   silent fallback; `audio_cues` + its tests. Review every task.
4. **Settings screen + menu** — `Screen::Settings`, `settings.rs` view,
   the `Settings` menu item + N-item nav, global `m`.
5. **Wire cues + music into `App`** — snapshot diff on every state change;
   music start/stop on settings/mute.
6. **Assets + acceptance** — source & license-verify the music (candidates
   for approval) and SFX; README ALSA note; CREDITS; acceptance sweep +
   skeptical review.

Review cadence: phases 1–3 (deps, persistence, audio core) are
foundational — every task; 4–6 more mechanical — per phase.

## Resolved decisions

- **All-`rodio` backend, drop `rusty_audio`** — one output stream, real
  loop/pause/volume; gated behind the CLAUDE.md amendment (human-approved).
- **Engine stays audio-free; cues observed at `App`** via the pure
  `audio_cues` snapshot diff — covers player *and* opponent, unit-tested,
  `game.rs` untouched.
- **Global mute is a session flag**; the per-channel Music/SFX toggles are
  the persisted prefs; `apply_music` reconciles the sink from both.
- **Settings persist to `directories` config dir as JSON**, defaulting on
  any read error — the first, deliberately small slice of the save layer.
- **Silent-stub fallback** when no audio device — the game never fails to
  run for lack of sound.
- **Assets embedded** via `include_bytes!` (self-contained binary),
  CC0/generated only, sources + licenses recorded.
