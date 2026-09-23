#!/usr/bin/env python3
"""Generate the game's sound effects as short retro (square/triangle-wave)
WAV clips, plus one additive, voice-like burble. These are *synthesized
here*, not sourced, so they carry no license encumbrance. Re-run to
regenerate assets/sfx/*.wav:

    python3 scripts/gen_sfx.py           # every sound
    python3 scripts/gen_sfx.py burble    # only the named sounds

One file per Sfx variant in src/audio.rs. Tune the numbers below by ear.
"""
import math
import os
import random
import struct
import sys
import wave

SAMPLE_RATE = 44100
OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "assets", "sfx")


def square(freq, t):
    return 1.0 if math.sin(2 * math.pi * freq * t) >= 0 else -1.0


def triangle(freq, t):
    frac = (freq * t) % 1.0
    return 4.0 * abs(frac - 0.5) - 1.0


def tone(freq, duration, volume=0.5, wave_fn=square):
    """A single tone with a short attack and exponential decay (a blip)."""
    n = int(SAMPLE_RATE * duration)
    attack = max(1, int(0.005 * SAMPLE_RATE))  # 5ms, avoids a click
    out = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = i / attack if i < attack else math.exp(-3.0 * (i - attack) / n)
        out.append(volume * env * wave_fn(freq, t))
    return out


def sweep(f0, f1, duration, volume=0.5):
    """A square-wave frequency glide from f0 to f1."""
    n = int(SAMPLE_RATE * duration)
    attack = max(1, int(0.005 * SAMPLE_RATE))
    out = []
    phase = 0.0
    for i in range(n):
        frac = i / n
        freq = f0 + (f1 - f0) * frac
        phase += 2 * math.pi * freq / SAMPLE_RATE
        env = i / attack if i < attack else math.exp(-2.5 * frac)
        out.append(volume * env * (1.0 if math.sin(phase) >= 0 else -1.0))
    return out


def noise(duration, volume=0.3):
    n = int(SAMPLE_RATE * duration)
    return [volume * math.exp(-4.0 * i / n) * random.uniform(-1, 1) for i in range(n)]


def arpeggio(freqs, note_dur, volume=0.5, wave_fn=square):
    out = []
    for f in freqs:
        out.extend(tone(f, note_dur, volume, wave_fn))
    return out


def mix(a, b):
    n = max(len(a), len(b))
    a = a + [0.0] * (n - len(a))
    b = b + [0.0] * (n - len(b))
    return [x + y for x, y in zip(a, b)]


# The opponent's burble (spec 030): one soft, voice-like murmur per spoken word.
# Every number that shapes it is here, so a tweak by ear is an edit to this
# block and `python3 scripts/gen_sfx.py burble`. `peak` is bounded by the audio
# test `the_burble_is_softer_than_the_music`; `length_s` bounded by BURBLE_GAP_MS.
BURBLE = {
    "length_s": 0.12,          # within BURBLE_GAP_MS even at the slowest pitch
    "pitch_hz": 150,           # the voice's fundamental
    "glide": -0.12,            # fractional pitch fall over the burble
    "vibrato_hz": 18, "vibrato_depth": 0.04,   # the murmur's wobble in pitch …
    "tremolo_hz": 24, "tremolo_depth": 0.35,   # … and in level
    "formants": ((500, 750, 90), (1100, 1400, 140)),  # (start Hz, end Hz, width Hz)
    "harmonics": 18,           # fewer = rounder, more = buzzier
    "attack_s": 0.015, "release_s": 0.06,
    "peak": 0.08,              # loudest sample, full scale 1.0 — set from the test's measurement
}


def burble(p=BURBLE):
    """Additive voice: harmonics of a gliding, vibrato'd fundamental, each
    weighted 1/k times the resonance of two gliding formants (a "wo→a"
    vowel), with a raised-cosine attack/release and a tremolo, normalised so
    the loudest sample is `peak`. Uses no `random`, so `bust`'s seeded noise
    is undisturbed."""
    n = int(SAMPLE_RATE * p["length_s"])
    attack = max(1, int(p["attack_s"] * SAMPLE_RATE))
    release = max(1, int(p["release_s"] * SAMPLE_RATE))
    out = []
    phase = 0.0
    for i in range(n):
        t = i / SAMPLE_RATE
        frac = i / n
        vibrato = 1.0 + p["vibrato_depth"] * math.sin(2 * math.pi * p["vibrato_hz"] * t)
        f0 = p["pitch_hz"] * (1.0 + p["glide"] * frac) * vibrato
        phase += 2 * math.pi * f0 / SAMPLE_RATE
        centres = [(lo + (hi - lo) * frac, width) for lo, hi, width in p["formants"]]
        s = 0.0
        for k in range(1, p["harmonics"] + 1):
            fk = k * f0
            resonance = sum(1.0 / (1.0 + ((fk - c) / w) ** 2) for c, w in centres)
            s += resonance / k * math.sin(k * phase)
        env = 1.0
        if i < attack:
            env = 0.5 * (1.0 - math.cos(math.pi * i / attack))
        elif i >= n - release:
            env = 0.5 * (1.0 - math.cos(math.pi * (n - i) / release))
        tremolo = 1.0 - p["tremolo_depth"] * 0.5 * (1.0 - math.cos(2 * math.pi * p["tremolo_hz"] * t))
        out.append(s * env * tremolo)
    loudest = max(abs(x) for x in out)
    return [p["peak"] * x / loudest for x in out]


def write_wav(name, samples):
    path = os.path.join(OUT_DIR, name + ".wav")
    with wave.open(path, "w") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SAMPLE_RATE)
        frames = bytearray()
        for s in samples:
            s = max(-1.0, min(1.0, s))
            frames += struct.pack("<h", int(s * 32767))
        w.writeframes(bytes(frames))
    print(f"wrote {name}.wav ({len(samples) / SAMPLE_RATE:.2f}s)")


# A small scale to draw melodic cues from (approx. equal-temperament Hz).
C4, E4, G4, A3 = 262, 330, 392, 220
C5, E5, G5, C6, E6 = 523, 659, 784, 1047, 1319


def main():
    random.seed(0)  # deterministic output across runs
    os.makedirs(OUT_DIR, exist_ok=True)
    sounds = {
        "card_draw": sweep(400, 900, 0.08, 0.4),
        "card_play": tone(300, 0.10, 0.5),
        "flip": sweep(720, 360, 0.12, 0.45),
        "stand": tone(500, 0.09, 0.4, triangle),
        "bust": mix(sweep(300, 90, 0.35, 0.5), noise(0.35, 0.15)),
        "round_win": arpeggio([C5, E5, G5], 0.07, 0.45),
        "round_loss": arpeggio([G4, E4, C4], 0.09, 0.45),
        # A tie: up-down-back-to-start, so it resolves nowhere ("even").
        "round_tie": arpeggio([E5, C5, E5], 0.07, 0.42),
        "game_win": arpeggio([C5, E5, G5, C6, E6], 0.09, 0.5),
        "game_loss": arpeggio([G4, E4, C4, A3], 0.13, 0.5),
        "menu_move": tone(660, 0.04, 0.3, triangle),
        "menu_select": tone(880, 0.07, 0.4),
        # Closing a panel: a descending two-note, the "back" counterpart to
        # select's single higher blip.
        "menu_back": arpeggio([G5, C5], 0.05, 0.35),
        # The opponent's per-word murmur (spec 030); see BURBLE above.
        "burble": burble(),
    }
    # Named sounds only (e.g. `burble`), or all of them with no arguments.
    names = sys.argv[1:] or list(sounds)
    unknown = [name for name in names if name not in sounds]
    if unknown:
        sys.exit(f"unknown sound(s): {', '.join(unknown)}; known: {', '.join(sounds)}")
    for name in names:
        write_wav(name, sounds[name])


if __name__ == "__main__":
    main()
