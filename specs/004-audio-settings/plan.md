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
// T009 evolved this from on/off bools to per-channel volumes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)] // no Eq: f32
pub struct Settings {
    pub music_volume: f32, // 0.0–1.0; 0.0 = that channel off
    pub sfx_volume: f32,
}
impl Default for Settings { /* music_volume: 0.5, sfx_volume: 0.8 */ }
```

Load/save helpers (persistence section). This struct is the seed of the
save format — the campaign spec extends it, it doesn't rebuild it. Legacy
`{music, sfx}` bool files parse as unknown fields and fall back to the
default volumes.

### Sfx (`audio.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    CardDraw, CardPlay, Flip, Stand, Bust,
    RoundWin, RoundLoss, RoundTie, GameWin, GameLoss,
    MenuMove, MenuSelect,
}
```

Each maps to one embedded WAV clip. (T008 gave a round tie its own distinct
`RoundTie` sound rather than the original silence — otherwise a double-stand
to a tie just played the stand sound twice.)

### Audio (`audio.rs`)

*(T007a reshaped this. The rodio backend is `!Send` and its device-open
blocks for seconds on a cold start, so it can't live on the game-loop
thread. It now lives on a dedicated **audio thread**; `Audio` is just a
handle that sends it commands.)*

```rust
// Held by the game-loop thread — a non-blocking sender.
pub struct Audio { tx: Option<mpsc::Sender<AudioCommand>> }
enum AudioCommand { PlaySfx { sfx: Sfx, pitch: f32 }, SetSettings(Settings), ToggleMute }

// Owned by the audio thread; never crosses threads (rodio 0.22 is !Send).
struct AudioState { backend: Option<Backend>, settings: Settings, muted: bool }
struct Backend { sink: MixerDeviceSink, music: Player, has_music: bool }
```

- `Audio::new` spawns the audio thread and returns instantly; `play` /
  `set_settings` / `toggle_mute` just `tx.send(..)` a command (non-blocking).
- `play(Sfx, pitch)` (on the audio thread) — no-op unless `!muted &&
  settings.sfx_volume > 0.0`; else decode the embedded clip
  (`Decoder::new(Cursor::new(bytes))`), `.speed(pitch).amplify(sfx_volume)`,
  and add it straight to the device mixer so SFX overlap and never block.
- `apply_music()` — the single place that reconciles the music `Player`
  with state: `set_volume(music_volume)` + play when `!muted &&
  music_volume > 0.0`, pause otherwise. Called after every settings/mute
  change (`SetSettings` applies music volume live).
- Music loops via `Decoder::new_looped(Cursor::new(MUSIC_BYTES))` appended
  once at construction; muting/zeroing just pauses (cheap, resumes in place).
  The track is **embedded via `include_bytes!`** (finding-1 fix), so the
  binary is self-contained and plays from any working directory.
- **Graceful no-audio fallback:** if `DeviceSinkBuilder::open_default_sink()`
  fails (no device, headless CI), `backend` stays `None` and every method
  is a no-op — the game still runs, and `cargo test` opens no device.

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
#[derive(Debug, Clone, Copy)] // no Eq: RoundOutcome isn't Eq — the diff runs through audio_cues
struct AudioSnapshot {
    p_dealer: usize, p_played: usize, p_bust: bool, p_stood: bool, p_last_flip: bool,
    o_dealer: usize, o_played: usize, o_bust: bool, o_stood: bool, o_last_flip: bool,
    outcome: Option<RoundOutcome>, game_over: bool,
}
// A Cue is an Sfx plus a playback-speed factor, so the opponent's actions
// sound a touch lower (OPPONENT_PITCH = 0.92) than the player's.
pub struct Cue { sfx: Sfx, pitch: f32 }
fn audio_cues(prev: AudioSnapshot, curr: AudioSnapshot) -> Vec<Cue>;
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

## Settings panel (`settings.rs`)

*(Revised T009/T010 — see the Refinements at the end. Originally a
toggle-based full `Screen`; now a volume-slider **overlay**.)*

`SettingsState` owns the selected row; input maps arrows ↑/↓ (and `w`/`s`)
to move between rows, ←/→ (and `a`/`d`) to adjust the selected channel's
volume, Esc to close back to the menu. Drawing reuses the shared
vocabulary — the `▸` marker + pulse on the selected row, monochrome —
inside a bordered overlay box (`OverlayLayout`, sized like How to Play, via
`draw_overlay`), drawn over the still-visible start menu. Each row reads
`Music    [█████░░░░░]  50%` so the level is unmistakable. Adjusting a row
updates `App.settings`, calls `audio.set_settings(..)` (music volume
changes immediately), and saves to disk.

## Start menu changes (`menu.rs`)

- `MenuItem` gains `Settings` (order: Start Game, How To Play, Settings).
- The 2-item `toggle_selected` becomes real up/down navigation over
  `MenuItem::iter()` (the file's own TODO) — clamp or wrap across N items.
  `MenuLayout` already takes the item count (spec 003), so centering just
  works with three.
- `MenuEvent::Activate { Settings }` routes in `App` to open the settings
  overlay — `self.settings_panel = Some(SettingsState::default())` — rather
  than switching screens, so the menu stays put underneath (T010).

## Persistence (`settings.rs`)

- Save location via `directories::ProjectDirs::from("", "", "kaazap")` →
  its `config_dir()`, file `settings.json`.
- `Settings::load()` → read + `serde_json::from_str`; on *any* error
  (no dir, missing file, parse failure) return `Settings::default()` —
  never panic, never block startup. `settings.save()` → create the config
  dir if needed, serialize, write; a write failure is logged/ignored, not
  fatal (audio prefs aren't worth crashing over).
- Loaded once in `App::new`; saved on every volume change.

## Architecture / flow changes

- **`app.rs`**: `App` gains `settings: Settings` and `audio: Audio`
  (loaded/constructed in `new`) and `prev_audio: AudioSnapshot`. *(T010:*
  also `settings_panel: Option<SettingsState>` — the open settings overlay;
  `handle_key` checks it first as a modal over the menu, `handle_settings_input`
  serves nav/volume/Esc, and `draw` paints it over the menu via
  `draw_overlay`.) `handle_key` also intercepts `m` **before** per-screen
  routing (global mute), on every screen, and closing any overlay/panel
  plays `MenuBack`. After each state-changing action (player action; each
  `tick` `update`), emit `audio_cues`. Menu/settings actions play their
  `MenuMove`/`MenuSelect` directly.
- **`screen.rs`**: ~~add the `Settings` variant.~~ *(T010: no Settings
  variant — it's an overlay in `App`, not a `Screen`.)*
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
