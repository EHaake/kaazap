# Spec: Audio & Settings

**Status**: Approved
**Depends on**: 001-core-engine, 002-ui-overhaul, 003-board-slot-cap (all merged)
**Design reference**: `design/brief.md` (the restraint ethos extends to sound)

## Summary

Give Kaazap a voice: looping background music and a set of retro sound
effects for the key moments of play — all gated by user preference and,
for the first time, **persisted between launches**. This spec introduces
the project's first **Settings** screen (a new `Screen::Settings`,
reached from the start menu) with independent **Music** and **SFX**
toggles, plus a global mute keypress. Settings save to a JSON config file
via `serde`/`directories`, making this spec the first slice of the
persistence layer the campaign will later build on.

Two ground rules shape it. **Licensing:** the actual Star Wars / KOTOR
Pazaak music is copyrighted, so we bundle a **CC0 / royalty-free
chiptune** track that *evokes* the spacey-cantina vibe instead — no
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
4. **Settings screen.** A new `Screen::Settings`, reached via a new
   **Settings** item on the start menu, with independent Music and SFX
   toggles navigated with the established cursor vocabulary (arrows +
   Enter/Space, the `▸` marker and pulse) and an Escape back to the menu.
   Monochrome, consistent with the menu — no new interaction invention.
5. **Global mute keypress.** `m` from any screen instantly silences all
   audio and restores it — a session master mute layered over the
   persisted per-channel toggles.
6. **Persisted preferences.** A `Settings` struct saved as JSON in the
   platform config directory (`serde_json` + `directories`): loaded on
   startup (defaults if the file is missing or corrupt), saved on every
   change. The first, deliberately small slice of the persistence layer.
7. **Housekeeping.** Document the Linux ALSA build dependency in the
   README (per the constitution, now that sound work has started).
8. **Test coverage** for the logic per `CLAUDE.md`: settings toggle
   behavior, mute gating (what plays in each state), and settings
   load/save round-trip including the missing/corrupt-file fallback.
   Audio output itself is a side effect — verified by running, not mocked.

## Non-goals (explicitly deferred)

- **Volume levels / sliders.** On/off toggles only for now; a volume
  control is a future settings addition.
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

- **Settings** — the persisted preferences: `music: bool`, `sfx: bool`.
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

Menu → **Settings** → arrow to **Music** or **SFX** → Enter/Space toggles
it On/Off. The change takes effect immediately — music starts or stops,
SFX begin or stop sounding — and is written to the config file. Escape
returns to the menu.

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
- **Toggle state is legible at a glance** (e.g. `Music   ◄ On ►` / `Off`),
  the current value unmistakable.
- **All shipped audio is license-clean and credited.** The music track is
  CC0 / royalty-free (verified); SFX are CC0 or generated. The track's
  source and license are recorded (`DECISIONS.md` and/or an assets
  credits file).
- **Muting is instant and total; unmuting restores exactly** the saved
  per-channel state — no surprise where a channel comes back wrong.

## Acceptance criteria

- [ ] Background music loops while the app runs and starts/stops with the
      Music toggle; verified by running.
- [ ] SFX play for the defined moments and are silenced by the SFX toggle;
      verified by running.
- [ ] `m` from any screen instantly silences all audio and restores it to
      the saved per-channel state; verified by running.
- [ ] Menu → Settings opens the settings screen; Music and SFX toggle
      independently with the cursor vocabulary; Escape returns to the
      menu; verified by running.
- [ ] Settings persist: changing a toggle writes the config file, a
      relaunch loads it, and a missing/corrupt file falls back to defaults
      without crashing. Unit-tested (round-trip + fallback).
- [ ] The bundled music track is CC0 / royalty-free with its license
      recorded; no copyrighted (Star Wars / KOTOR) audio ships.
- [ ] README documents the Linux ALSA build dependency.
- [ ] `cargo test` green with the new coverage; `cargo build` introduces
      no new warnings.

## Resolved decisions

- **CC0 / royalty-free chiptune, not actual Star Wars / KOTOR music**
  (human-ruled, licensing) — the real tracks (and fan 8-bit covers of
  them) are copyrighted and can't ship in a project meant to be shared.
  A fitting CC0 track is sourced, license-verified, and presented as a
  couple of candidates for approval.
- **Settings persist now**, to a JSON config file via `serde_json` +
  `directories` (human-ruled) — the first slice of the persistence layer,
  extended later by the save/resume spec rather than rebuilt.
- **On/off toggles only** for the first settings; volume is deferred.
- **Global mute (`m`) is a session master** over the persisted per-channel
  toggles — one quick key, plus the granular settings.
- **Settings is a new `Screen::Settings`** (per the architecture rule for
  new top-level modes), reached from a new start-menu item.
- **SFX set chosen by the implementer** (human-delegated): card draw, card
  play, stand, bust, round win/loss, game win/loss, menu move/select,
  flip; cursor movement and sign toggling stay silent.
- **Audio library split resolved in the plan.** `rusty_audio` for SFX and
  `rodio` (its sanctioned escape hatch) for looping music is the expected
  shape; if the plan finds an all-`rodio` backend cleaner, that contradicts
  the constitution's "keep rusty_audio" note and will be raised as an
  explicit constitution amendment first, per the constitution's own rule.
