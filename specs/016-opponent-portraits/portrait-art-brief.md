# Opponent-portrait art brief (spec 016)

**Purpose.** A precise, self-contained specification for authoring the 11 opponent
portraits (10 roster + 1 generic fallback) so a more capable art tool — **Claude
Design** or **Fable 5.1** — can produce them. Hand this whole file over as the
prompt/spec. Claude Code (this project) keeps the render path, the `OpponentProfile.portrait`
field, and the panel geometry already built in T001; it will **validate and integrate**
whatever comes back. The two faces authored in T001 (below) are the *baseline to beat* —
"not terrible," per the product owner — not a floor to merely match.

---

## The hard format constraint (read this first — it is not negotiable)

kaazap is a terminal game with a **custom monochrome character-grid renderer**. There is
**no image support and no color** — every portrait is drawn as literal text characters,
one glyph per grid cell, all at a single uniform weight. So the deliverable for each
opponent is a plain-text file, **not** a PNG, not HTML, not high-res ASCII.

Each portrait file must be:

- **Exactly 12 lines tall**, each line **at most 18 columns wide** (count *columns*, i.e.
  displayed cells, not bytes — the glyphs below are multi-byte in UTF-8 but occupy one cell).
- **One glyph per cell.** No zero-width, no combining marks, no wide/full-width characters,
  no tabs. Short lines (< 18) are fine and draw left-aligned, but center the face so it
  sits symmetric within the 18-column field.
- **Monochrome.** No ANSI color, no escape codes — just the characters. Depth and shading
  come **only from glyph density** (see palette), because the renderer draws the whole
  portrait at one uniform emphasis. A darker region = a denser glyph (`█`/`▓`); a lighter
  region = a sparser glyph (`░`) or a space.
- **UTF-8, LF newlines, one trailing newline.** No trailing spaces needed.
- Filenames: one file per opponent, `<id>.txt` (ids listed in the table below), delivered
  for all **11** (`generic` + the 10 roster ids).

### Allowed glyph palette (use only these + space)

```
full / shade density:   █  ▓  ▒  ░           (dark → light)
half blocks:            ▀ (upper)  ▄ (lower)  ▌ (left)  ▐ (right)
quadrant blocks:        ▖ ▗ ▘ ▝   ▙ ▟ ▛ ▜   ▚ ▞
space:                  (blank cell / background)
```

Nothing outside this set. (Box-drawing glyphs `─│┌┐└┘…` are reserved for the panel *border*,
which the game draws around the portrait — don't use them inside the face.) A few plain ASCII
marks for tiny features (an eye glint, a nostril) are acceptable **only** if strictly
single-width and used sparingly, but the block/shade set above should carry the whole face.

> Width caveat: the shade/half/full blocks (`█▀▄▓▒░`) are East-Asian *Ambiguous* width. The
> game already ships its border and title art in this same class and assumes narrow, so this
> palette is consistent — just don't reach for any glyph outside the list, some of which are
> unconditionally double-width and would shear the grid.

### The canvas (18 × 12)

```
    ┌──────────────────┐   columns 0..17  (18 wide)
    │..................│   row 0   ← top of head
    │..................│   row 1
    │..................│   row 2
    │..................│   row 3
    │..................│   row 4   ← eyes usually land here
    │..................│   row 5
    │..................│   row 6
    │..................│   row 7
    │..................│   row 8   ← mouth
    │..................│   row 9
    │..................│   row 10
    │..................│   row 11  ← chin / shoulders
    └──────────────────┘
```

Compose a **head-and-shoulders bust** that fills most of the frame — a face floating in a
small void reads as weak at this resolution. Half/quarter blocks buy you effective
half-cell "sub-pixels," so 18×12 has more resolution than it looks.

> Size is the one open lever: 18×12 is fixed here because the in-match panel width and the
> game's minimum terminal size (139×31) are derived from it. If a bigger canvas is wanted for
> more detail, that is a deliberate product decision (it widens the minimum terminal) — raise
> it with the product owner before changing the dimensions, and tell Claude Code so it moves
> the derived constants together.

---

## The aesthetic

- **Low-resolution, monochrome, stylized alien faces** — clearly readable as faces, with
  **eyes and an expression discernible**. That legibility (eyes + expression) is the pass/fail
  bar the product owner judges by looking at the running game.
- **Original designs, no trademarked species** — a cantina's mix of aliens, in the spirit of
  the game's original planet names and no-copyrighted-music stance. No Star Wars species,
  no recognizable franchise creatures.
- **A consistent set.** The 11 should feel drawn by one hand — same framing (bust), same
  lighting logic (denser = shadow), same "monochrome pixel-art" register — while each face is
  visually **distinct** (the 10 roster faces must be pairwise different; the generic is its own
  anonymous face).
- Expression is the point: each opponent's personality (below) should show in the face —
  brow, eyes, mouth, silhouette. This is *why* portraits ship first: the banter planned next
  attaches to these faces.

---

## Per-opponent art direction

`id` → the filename (`<id>.txt`). Difficulty rises down the roster (Greeb easiest → Sovereign
hardest); let the faces harden accordingly — the early opponents looser and more expressive,
the late ones colder and more imposing.

| id | Name | Difficulty | Blurb (canon) | Expression / vibe to author |
|---|---|---|---|---|
| `generic` | *Opponent* | — (fallback) | *(none — stands in for any opponent lacking bespoke art)* | Neutral, anonymous, impassive — a "default" cantina alien. Readable but characterless on purpose; it's the fallback so the panel is never blank. |
| `greeb` | Greeb | Rookie | "Green and eager — folds early, and slips." | Young, eager, a little nervous. Big hopeful eyes, raised brows, maybe an overbite/toothy grin. Reads as harmless. |
| `dax` | Dax Runo | Greenhorn | "A cocky kid who bets big and busts bigger." | Cocky, brash youth. Smirk, chin up, one brow cocked — overconfident, not yet skilled. |
| `vessa` | Vessa Korr | Scrapper | "A scrapper who pushes hard — chases the win, risks the bust." | Tough, weathered fighter. Scowl, maybe a scar or notched feature, aggressive set to the jaw. |
| `nima` | Nima Sarn | Broker | "Counts every credit — folds the moment she's ahead." | Shrewd, calculating (reads female). Narrowed, appraising eyes, tight neutral mouth — a card-counter's poker face. |
| `toran` | Old Toran | Veteran | "Balanced and patient. Knows the game." | Old, calm, weathered. Heavy-lidded steady gaze, lines/wrinkles, unhurried — quietly formidable. |
| `brakka` | Brakka | Bruiser | "Swings for twenty and dares you to match it." | Big, brutish. Heavy brow, broad jaw, tusks or an underbite, small mean eyes — a wall of a creature. |
| `rix` | Rix Vandal | Ace | "An ace who counts every point — takes the exact play to win." | Sharp, cool, precise. Sleek features, confident half-smile, alert level eyes — a professional. |
| `kesh` | Kesh Varn | Duelist | "A hair-trigger duelist who plays every edge hard." | Intense, edgy, coiled. Angular face, narrowed hard eyes, tension in the mouth — dangerous and quick. |
| `magistrate` | The Magistrate | Master | "Relentless. Pushes to the edge and seldom slips." | Severe, cold authority. Imposing silhouette (crest, headdress, or high collar), unblinking, humorless — a face that judges you. |
| `sovereign` | The Sovereign | Kingpin | "The house's untouchable best — never a wasted card, never a slip." | Regal and untouchable — the final boss. The most ornate/menacing: crown, horns, or a masklike face; still, symmetrical, absolute. Should read as clearly the top of the ladder. |

---

## Reference: the T001 baseline faces (the two to beat)

These render correctly in-game today and are the baseline. Match their format exactly;
improve their legibility and character.

`generic.txt` (broad-skulled grey; forehead sheen; big almond eyes):

```
     ▄▄▄▄▄▄▄▄
   ▄▓▓▓▓▓▓▓▓▓▓▄
  ▟▓▓▓▓▓▓▓▓▓▓▓▓▙
  █▓▓░░░░░░░░▓▓█
  █▓▓▓▓▓▓▓▓▓▓▓▓█
  █▓██▓▓▓▓▓▓██▓█
   ▜▓██▓▓▓▓██▓▜
    ▜▓▓▓▓▓▓▓▓▙
     ▜▓▓▄▄▓▓▟
      ▜▓▓▓▓▟
       ▀▓▓▀
        ▀▀
```

`greeb.txt` (eager big-eared rookie; raised brows; toothy grin):

```
   ▚▖      ▗▞
    ▝▙▄▄▄▄▄▄▟▘
     ▟▓▓▓▓▓▓▓▙
     █▓▓▓▓▓▓▓█
     █▓▄▓▓▓▄▓█
     █▓██▓██▓█
     ▜▓██▓██▓▛
      ▜▓▓▄▓▓▛
      █▓▓▓▓▓█
      █▄███▄█
      ▜▓▀▀▀▓▛
       ▀▀▀▀▀
```

---

## Validation checklist (Claude Code enforces this before art ships)

For every file, Claude Code will confirm and, if needed, bounce back:

1. Exactly **12 lines**; every line **≤ 18 columns** (displayed cells).
2. Only glyphs from the **allowed palette** (+ space); no color/escape codes; no
   wide/zero-width/combining characters.
3. The 10 roster portraits are **pairwise distinct** strings; `generic` is its own face.
4. Rendered in the actual game (opponent-select preview, in-match panel, campaign-map rail):
   the face is legible with **eyes and expression discernible** — the product owner's
   go/no-go look.

Deliver the 11 files (or paste the 11 grids), and Claude Code drops them into
`assets/portraits/`, runs the dimension/distinctness tests, and renders them for the
product owner's sign-off.
