#!/usr/bin/env python3
"""Generate the game's sound effects as short retro (square/triangle-wave)
WAV clips. These are *synthesized here*, not sourced, so they carry no
license encumbrance. Re-run to regenerate assets/sfx/*.wav.

    python3 scripts/gen_sfx.py

One file per Sfx variant in src/audio.rs. Tune the numbers below by ear.
"""
import math
import os
import random
import struct
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
        "game_win": arpeggio([C5, E5, G5, C6, E6], 0.09, 0.5),
        "game_loss": arpeggio([G4, E4, C4, A3], 0.13, 0.5),
        "menu_move": tone(660, 0.04, 0.3, triangle),
        "menu_select": tone(880, 0.07, 0.4),
    }
    for name, samples in sounds.items():
        write_wav(name, samples)


if __name__ == "__main__":
    main()
