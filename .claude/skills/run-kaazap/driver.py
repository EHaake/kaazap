#!/usr/bin/env python3
"""Pty driver for the kaazap TUI.

kaazap renders by cursor-positioning every changed cell individually
(column-major), so its raw output stream never contains readable words.
This driver spawns `cargo run` in a pty, applies MoveTo/clear sequences
to a character grid, sends scripted keys, and prints labeled snapshots.

Usage: python3 driver.py [step ...]
  wait:TEXT[:SECONDS]  pump output until TEXT appears (default 15s)
  key:KEYS             send KEYS to the app (\\r = Enter)
  pump:SECONDS         absorb output for SECONDS
  snap:LABEL           print the current screen grid, labeled

No args: wait for the menu and snapshot it. Exits nonzero if a wait
times out (a *_TIMEOUT snapshot is printed first).
"""
import codecs
import fcntl
import os
import pty
import re
import select
import struct
import subprocess
import sys
import termios
import time
from pathlib import Path

ROWS, COLS = 48, 180
REPO_ROOT = Path(__file__).resolve().parents[3]

CSI_RE = re.compile(r"\x1b\[([0-9;?]*)([A-Za-z])")


class Term:
    def __init__(self):
        self.grid = [[" "] * COLS for _ in range(ROWS)]
        self.cx = self.cy = 0
        self.pending = ""  # partial escape sequence tail
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")

    def feed(self, data: bytes):
        s = self.pending + self.decoder.decode(data)
        self.pending = ""
        i = 0
        while i < len(s):
            ch = s[i]
            if ch == "\x1b":
                m = CSI_RE.match(s, i)
                if m:
                    self._csi(m.group(1), m.group(2))
                    i = m.end()
                    continue
                if i >= len(s) - 16:  # partial sequence at chunk end
                    self.pending = s[i:]
                    return
                i += 1  # unrecognized escape, skip
                continue
            if ch == "\r":
                self.cx = 0
            elif ch == "\n":
                self.cy = min(self.cy + 1, ROWS - 1)
                self.cx = 0
            elif ch == "\b":
                self.cx = max(self.cx - 1, 0)
            elif ch >= " ":
                if self.cy < ROWS and self.cx < COLS:
                    self.grid[self.cy][self.cx] = ch
                self.cx = min(self.cx + 1, COLS - 1)
            i += 1

    def _csi(self, params: str, final: str):
        if final in "Hf":
            parts = [p for p in params.split(";") if p and not p.startswith("?")]
            row = int(parts[0]) if len(parts) > 0 else 1
            col = int(parts[1]) if len(parts) > 1 else 1
            self.cy, self.cx = min(row - 1, ROWS - 1), min(col - 1, COLS - 1)
        elif final == "J":
            self.grid = [[" "] * COLS for _ in range(ROWS)]
        # everything else (colors, cursor visibility, alt screen) ignored

    def text(self) -> str:
        return "\n".join("".join(r).rstrip() for r in self.grid)

    def snapshot(self, label: str):
        print(f"===== SNAP {label} =====")
        for n, row in enumerate(self.grid):
            line = "".join(row).rstrip()
            if line:
                print(f"{n:2}|{line}")
        print(f"===== END {label} =====", flush=True)


def main(argv):
    steps = argv or ["wait:Start Game", "snap:MENU"]

    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
    env = dict(os.environ, TERM="xterm-256color")

    def child_setup():
        os.setsid()
        fcntl.ioctl(0, termios.TIOCSCTTY, 0)

    proc = subprocess.Popen(
        ["cargo", "run"],
        stdin=slave, stdout=slave, stderr=slave,
        cwd=REPO_ROOT, env=env, preexec_fn=child_setup,
    )
    os.close(slave)
    term = Term()

    def pump(seconds: float):
        deadline = time.time() + seconds
        while time.time() < deadline:
            r, _, _ = select.select([master], [], [], 0.1)
            if master in r:
                try:
                    data = os.read(master, 65536)
                except OSError:
                    return False
                if not data:
                    return False
                term.feed(data)
        return True

    def wait_for(needle: str, timeout: float) -> bool:
        deadline = time.time() + timeout
        while time.time() < deadline:
            pump(0.2)
            if needle in term.text():
                return True
        return False

    failed = False
    try:
        for step in steps:
            op, _, arg = step.partition(":")
            if op == "wait":
                text, _, tmo = arg.rpartition(":")
                if text and tmo.replace(".", "").isdigit():
                    ok = wait_for(text, float(tmo))
                    label = text
                else:
                    ok = wait_for(arg, 15)
                    label = arg
                if not ok:
                    term.snapshot(f"WAIT_{label!r}_TIMEOUT")
                    failed = True
                    break
            elif op == "key":
                keys = arg.replace("\\r", "\r").replace("\\n", "\n").replace("\\t", "\t")
                os.write(master, keys.encode())
            elif op == "pump":
                pump(float(arg))
            elif op == "snap":
                term.snapshot(arg or "SNAP")
            else:
                print(f"unknown step: {step}", file=sys.stderr)
                failed = True
                break
        return 1 if failed else 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()
        os.close(master)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
