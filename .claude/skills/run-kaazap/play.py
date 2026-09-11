#!/usr/bin/env python3
"""Driver wrapper: same steps as driver.py plus `play[:STAND]` which plays a
whole match (hit until score >= STAND, default 17; else stand) until the
game-over banner, and `text:NEEDLE` which prints whether NEEDLE is on screen."""
import sys, os, re, time, select, subprocess, pty, fcntl, termios, struct
sys.path.insert(0, "/Users/erikh/Projects/Rust/kaazap/.claude/skills/run-kaazap")
from driver import Term, ROWS, COLS, REPO_ROOT

def main(steps):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
    env = dict(os.environ, TERM="xterm-256color")
    def child_setup():
        os.setsid(); fcntl.ioctl(0, termios.TIOCSCTTY, 0)
    proc = subprocess.Popen(["cargo", "run"], stdin=slave, stdout=slave, stderr=slave,
                            cwd=REPO_ROOT, env=env, preexec_fn=child_setup)
    os.close(slave); term = Term()
    def pump(seconds):
        deadline = time.time() + seconds
        while time.time() < deadline:
            r, _, _ = select.select([master], [], [], 0.1)
            if master in r:
                try: data = os.read(master, 65536)
                except OSError: return False
                if not data: return False
                term.feed(data)
        return True
    def wait_for(needle, timeout):
        deadline = time.time() + timeout
        while time.time() < deadline:
            pump(0.2)
            if needle in term.text(): return True
        return False
    def send(k): os.write(master, k.encode())
    def play(stand):
        deadline = time.time() + 240
        while time.time() < deadline:
            pump(0.4)
            t = term.text()
            if "YOU WIN THE GAME" in t or "YOU LOST THE GAME" in t:
                return True
            if "won this round" in t or "won the round" in t or "You Tied" in t:
                send("n"); pump(1.0); continue
            m = re.search(r"Player:.*?Score:\s*(\d+)", t)
            if m:
                send("d" if int(m.group(1)) < stand else "s"); pump(1.5)
        return False
    failed = False
    try:
        for step in steps:
            op, _, arg = step.partition(":")
            if op == "wait":
                text, _, tmo = arg.rpartition(":")
                if text and tmo.replace(".", "").isdigit(): ok = wait_for(text, float(tmo)); label = text
                else: ok = wait_for(arg, 15); label = arg
                if not ok: term.snapshot(f"WAIT_{label!r}_TIMEOUT"); failed = True; break
            elif op == "key":
                send(arg.replace("\\r", "\r").replace("\\n", "\n").replace("\\e", "\x1b"))
            elif op == "pump": pump(float(arg))
            elif op == "snap": term.snapshot(arg or "SNAP")
            elif op == "play":
                ok = play(int(arg) if arg else 17)
                if not ok: term.snapshot("PLAY_TIMEOUT"); failed = True; break
                print("PLAY_RESULT:", "WIN" if "YOU WIN THE GAME" in term.text() else "LOSS", flush=True)
            elif op == "text":
                print(f"TEXT {arg!r}: {'present' if arg in term.text() else 'ABSENT'}", flush=True)
            elif op == "rows":
                # print only rows containing any of the |-separated needles
                needles = arg.split("|")
                for n, row in enumerate(term.grid):
                    line = "".join(row).rstrip()
                    if any(x in line for x in needles): print(f"{n:2}|{re.sub('  +', ' ', line)}")
                sys.stdout.flush()
            elif op == "resize":
                w, h = (int(v) for v in arg.lower().split("x"))
                fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack("HHHH", h, w, 0, 0))
            else: print("unknown step", step); failed = True; break
        return 1 if failed else 0
    finally:
        proc.terminate()
        try: proc.wait(timeout=3)
        except subprocess.TimeoutExpired: proc.kill()
        os.close(master)

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
