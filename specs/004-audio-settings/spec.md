# Spec: Audio & Settings

**Status**: Approved
**Depends on**: 001-core-engine, 002-ui-overhaul, 003-board-slot-cap (all merged)
**Design reference**: `design/brief.md` (the restraint ethos extends to sound)

## Summary

Give Kaazap a voice: looping background music and a set of retro sound
effects for the key moments of play — all gated by user preference and,
for the first time, **persisted between launches**. This spec introduces
the project's first **Settings** panel (an overlay over the start menu,
reached from a new start-menu item) with independent **Music** and **SFX**
controls, plus a global mute keypress. Settings save to a JSON config file
via `serde`/`directories`, making this spec the first slice of the
persistence layer the campaign will later build on.

Two ground rules shape it. **Licensing:** the actual Star Wars / KOTOR
Pazaak music is copyrighted, so we bundle a **license-clean chiptune**
track (CC-BY, credited) that *evokes* the spacey-cantina vibe instead — no
copyrighted audio ships. **Restraint:** audio follows the brief's
aesthetic the way the visuals do — purposeful and sparse, mixed low,
never a barrage.

## Goals

1. **Audio playback layer.** A small module that loads and plays SFX and
   loops background music, honoring the current mute / per-channel state.
   Uses the sanctioned libraries (`rusty_audio` for SFX; `rodio`, its
   sanctioned escape hatch, for looping music — the exact split is a plan
   decision, see Non-goals / Resolved).
2. **Background music.** One bundled CC0 / royalty-free chiptune track
   (spacey-cantina feel, license-verified) that loops while the app runs,
   gated by the Music setting. Candidates sourced and presented for
   approval during the design step.
3. **Sound effects.** Retro SFX for the moments that matter — card draw,
   card play, stand, bust, round win / loss, game win / loss, menu move /
   select, and a flip effect — CC0 or generated, gated by the SFX
   setting. Tasteful and sparse; **no per-cursor-move blip.**
4. **Settings panel.** An overlay over the start menu (like How to Play),
   reached via a new **Settings** item, with independent Music and SFX
   volume sliders navigated with the established cursor vocabulary (arrows
   to select a row and adjust its level, the `▸` marker and pulse) and an
   Escape back to the menu with the selection preserved. Monochrome,
   consistent with the menu — no new interaction invention. *(Refined
   during implementation: sliders in from on/off toggles, and overlay in
   from a full `Screen` — see Resolved decisions.)*
5. **Global mute keypress.** `m` from any screen instantly silences all
   audio and restores it — a session master mute layered over the
   persisted per-channel volumes.
6. **Persisted preferences.** A `Settings` struct saved as JSON in the
   platform config directory (`serde_json` + `directories`): loaded on
   startup (defaults if the file is missing or corrupt), saved on every
   change. The first, deliberately small slice of the persistence layer.
7. **Housekeeping.** Document the Linux ALSA build dependency in the
   README (per the constitution, now that sound work has started).
8. **Test coverage** for the logic per `CLAUDE.md`: settings volume
   behavior, mute gating (what plays in each state), and settings
   load/save round-trip including the missing/corrupt-file fallback.
   Audio output itself is a side effect — verified by running, not mocked.

## Non-goals (explicitly deferred)

- ~~**Volume levels / sliders.** On/off toggles only for now.~~
  *Un-deferred (human-requested, T009): per-channel volume sliders replace
  the on/off toggles — a slider at 0 is that channel off. See "Refined
  during implementation" below.*
- **Actual Star Wars / KOTOR music.** Copyrighted; explicitly not shipped
  (see Summary). No fan "8-bit cover" of those either — still derivative.
- **Runtime music synthesis / procedural audio.** We bundle a track file,
  not generate tones live.
- **Full save/resume** (game state, campaign, currency, unlocked cards).
  This spec persists **only settings**; the broader save system is its
  own roadmap item. The persistence code here is written to be extended
  by it, not to pre-build it.
- **Dynamic / reactive music** (layers that respond to game state),
  spatial audio, per-sound theming.
- **Sound for every micro-interaction** — cursor movement across the hand
  and sign toggling stay silent to avoid a barrage.

## Entities

- **Settings** — the persisted preferences: `music_volume: f32`,
  `sfx_volume: f32` (0.0–1.0; 0.0 is off).
  Serializable; the seed of the save format.
- **Audio player** — owns the loaded SFX clips and the music stream;
  exposes "play this SFX" and music start/stop/pause; applies the
  mute / per-channel gating so callers just say what happened.
- **Sfx** — the enumerated set of effects (card draw, card play, stand,
  bust, round win/loss, game win/loss, menu move/select, flip).
- **Settings screen state** — which row (Music / SFX) is selected, drawn
  with the shared cursor vocabulary.
- **Mute** — a session master flag that overrides both channels; `m`
  toggles it.

## Key user flows

### Changing a sound setting

Menu → **Settings** → `↑`/`↓` to **Music** or **SFX** → `←`/`→` adjusts
that channel's volume by 10%. The change takes effect immediately — the
music sink and SFX loudness follow the level, a channel at 0% is off — and
is written to the config file. Escape returns to the menu.

### Quick mute

Pressing `m` anywhere (menu, settings, in-game) instantly silences
everything — music pauses, SFX stop. Pressing `m` again restores audio to
exactly the saved per-channel state.

### First launch vs later launches

On first launch there's no config file, so audio defaults to on and a
config file is written. Later launches load the saved preferences. A
corrupt or unreadable file falls back to defaults without crashing.

### Playing with sound

During a match, SFX punctuate the beats — a draw, a card played, a stand,
a bust, the round and game outcomes — while the chiptune loops quietly
underneath, all subject to the toggles and the mute key.

## Design requirements

- **Restraint (the audio analog of the brief's "motion is emphasis").**
  Sounds are short, purposeful, and sparse, mixed low; music is
  unobtrusive background. Sound marks meaning, it doesn't fill silence.
- **The Settings screen speaks the established vocabulary** — monochrome,
  cursor selection (arrows + Enter/Space), the `▸` marker and the shared
  pulse — reading like the start menu, not a new dialect.
- **Volume level is legible at a glance** (a filled bar plus a percentage,
  e.g. `Music [█████░░░░░] 50%`), the current level unmistakable.
- **All shipped audio is license-clean and credited.** The music track is
  CC0 or CC-BY (license-verified, credited when CC-BY); SFX are generated.
  The track's source and license are recorded (`assets/CREDITS.md` and
  `DECISIONS.md`).
- **Muting is instant and total; unmuting restores exactly** the saved
  per-channel state — no surprise where a channel comes back wrong.

## Acceptance criteria

- [x] Background music loops while the app runs and follows the Music
      volume level (0% pauses it). *(Chipper Doodle loops via rodio
      `new_looped`; `set_settings` → the sink's `set_volume` / pause.
      Audible confirmation is the human's.)*
- [x] SFX play for the defined moments and scale with the SFX volume level
      (0% silences them). *(T007: a full round fires
      draw/play/stand/bust/outcome cues, amplified by `sfx_volume`.)*
- [x] Music and SFX each carry an independent volume slider on the Settings
      screen; `←`/`→` adjust the selected row by 10%, shown as a bar +
      percentage. *(T009: verified in-app — Music 50%→70%, SFX adjusts
      independently; `volume_bar` unit-tested.)*
- [x] `m` from any screen instantly silences all audio and restores it to
      the saved per-channel state. *(T007: global `m` → `toggle_mute`;
      verified in-app.)*
- [x] Menu → Settings opens the settings panel as an overlay over the menu;
      the cursor vocabulary selects a row and adjusts its level; Escape
      closes it back to the menu with the selection preserved, and closing
      (Settings or How to Play) plays the `MenuBack` cue.
      *(T006/T009/T010: verified in-app — box over the menu, slider adjust,
      selection on `▸ Settings` after close.)*
- [x] Settings persist: changing a level writes the config file, a
      relaunch loads it, and a missing/corrupt file falls back to defaults
      without crashing. Unit-tested. *(T002/T009 five `settings_` tests over
      the `{music_volume, sfx_volume}` schema; legacy bool files fall back
      to defaults.)*
- [x] The bundled music track is license-clean (CC-BY, credited — human
      OK'd attribution) with its license recorded; no copyrighted (Star
      Wars / KOTOR) audio ships. *(Chipper Doodle CC-BY 4.0 in
      `assets/CREDITS.md` + `DECISIONS.md`; SFX generated; no copyrighted
      filenames.)*
- [x] README documents the Linux ALSA build dependency. *(Building
      section, refreshed for rodio.)*
- [x] `cargo test` green (148) with the new coverage; `cargo build`
      introduces no new warnings (0).

## Resolved decisions

- **Licensed royalty-free music, not actual Star Wars / KOTOR music**
  (human-ruled, licensing) — the real tracks (and fan 8-bit covers of
  them) are copyrighted and can't ship in a project meant to be shared;
  attribution is not a license. Candidates were sourced and license-
  verified; the human, fine with attribution, chose **CC-BY** Kevin
  MacLeod's "Chipper Doodle" as a placeholder loop (credited in
  `assets/CREDITS.md`). An original cantina-vibe track is roadmapped.
- **Settings persist now**, to a JSON config file via `serde_json` +
  `directories` (human-ruled) — the first slice of the persistence layer,
  extended later by the save/resume spec rather than rebuilt.
- **Per-channel volume sliders, not on/off toggles** (human-requested,
  refined during implementation — see below). Each channel (Music, SFX)
  carries a 0.0–1.0 volume; a slider dragged to 0 is that channel off, so
  the sliders subsume the originally-planned toggles rather than sitting
  beside them.
- **Global mute (`m`) is a session master** over the persisted per-channel
  volumes — one quick key, plus the granular sliders.
- ~~**Settings is a new `Screen::Settings`** (per the architecture rule for
  new top-level modes), reached from a new start-menu item.~~ *Superseded
  (T010): Settings is an **overlay** over the start menu, like How to Play,
  not a `Screen`. Reached from the same start-menu item. See "Refined
  during implementation" — the constitution was amended to match.*
- **SFX set chosen by the implementer** (human-delegated): card draw, card
  play, stand, bust, round win/loss, game win/loss, menu move/select,
  flip; cursor movement and sign toggling stay silent.
- **Audio library split resolved in the plan.** `rusty_audio` for SFX and
  `rodio` (its sanctioned escape hatch) for looping music is the expected
  shape; if the plan finds an all-`rodio` backend cleaner, that contradicts
  the constitution's "keep rusty_audio" note and will be raised as an
  explicit constitution amendment first, per the constitution's own rule.

### Refined during implementation

- **Volume sliders replaced the on/off toggles** (human-requested after the
  audio system was working, T009). Each channel gets a 0.0–1.0 volume shown
  as a 10-segment bar plus a percentage; `←`/`→` (or `a`/`d`) adjust the
  selected row by 10%, clamped to 0–100%. A channel at 0% is off, so this
  strictly subsumes the toggle: `Settings` now stores `music_volume` /
  `sfx_volume` floats instead of two bools, the SFX path amplifies by
  `sfx_volume` and the music sink's `set_volume` tracks `music_volume`, and
  the global `m` mute still overrides both. Legacy `{music, sfx}` bool
  config files are read as unknown fields and fall back to the default
  volumes rather than erroring.
- **Settings became an overlay, not a `Screen`** (human-requested UI pass,
  T010). Originally a full-screen `Screen::Settings`; now an overlay panel
  drawn over the start menu (a bordered box, sized like How to Play), so
  the two menu panels are consistent and the menu selection is preserved
  when you close Settings (it was resetting to "Start Game"). `App` holds
  the transient `Option<SettingsState>` and routes input to it while open;
  Esc closes back to the menu. This contradicted the constitution's
  "settings → `Screen` variant" line, so the constitution was amended first
  (its own commit on `main`) to classify menu sub-panels (How to Play,
  Settings) as overlays and reserve `Screen` variants for full modes
  (campaign, shop). Also added a distinct `MenuBack` SFX played when
  closing either menu panel (there was an enter sound but no exit sound).
