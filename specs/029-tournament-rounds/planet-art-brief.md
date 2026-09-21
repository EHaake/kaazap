# Per-planet venue art brief (spec 029)

**Purpose.** A precise, self-contained specification for authoring the **eight
planets' venue art** — the large picture the tournament venue screen is built
around — so a more capable art tool (**Claude Design** or **Fable 5.1**) can
produce it. Hand this whole file over as the prompt/spec. Claude Code (this
project) keeps the layout, the geometry and the render path; it will **validate
and integrate** whatever comes back. There is no baseline art to beat: the venue
ships today with an empty bordered box carrying the planet's name as a
placeholder, and this art replaces it.

Written the way spec 016's `specs/016-opponent-portraits/portrait-art-brief.md`
was, and it is worth reading that file alongside this one — the format,
the palette and the validation discipline are the same, and the two constraints
that differ (a much larger canvas, and **every line exactly the canvas width**)
are called out below.

Authoring this art is a **non-goal of spec 029**. The brief rides spec 029's
branch so the drawing can be handed out and folded back in under its own spec.

---

## The hard format constraint (read this first — it is not negotiable)

kaazap is a terminal game with a **custom monochrome character-grid renderer**.
There is **no image support and no color** — the art is drawn as literal text
characters, one glyph per grid cell, all at a single uniform weight. So the
deliverable for each planet is a plain-text file, **not** a PNG, not HTML, not
high-res ASCII.

Each art file must be:

- **Exactly 20 lines tall**, and **every line exactly the canvas width** — 48
  columns for a narrow file, 92 for a wide one (count *columns*, i.e. displayed
  cells, not bytes — the glyphs below are multi-byte in UTF-8 but occupy one
  cell). Short lines are **not** acceptable here; see *Every line exactly the
  canvas width*, below, which is the one constraint the portrait brief did not
  have.
- **One glyph per cell.** No zero-width, no combining marks, no wide/full-width
  characters, no tabs.
- **Monochrome.** No ANSI color, no escape codes — just the characters. Depth
  and shading come **only from glyph density** (see palette), because the
  renderer draws the whole picture at one uniform emphasis. A darker region = a
  denser glyph (`█`/`▓`); a lighter region = a sparser glyph (`░`) or a space.
- **UTF-8, LF newlines, one trailing newline.** Twenty lines and one final LF —
  no 21st line, blank or otherwise.
- Filenames: **two files per planet**, `<id>-narrow.txt` (48 × 20) and
  `<id>-wide.txt` (92 × 20), for all eight planet ids in the table below —
  **sixteen files**, delivered into `assets/planets/`.

**Deliver files, not pasted grids.** Trailing spaces are load-bearing in this
format (a row whose right half is empty sky still ends in spaces out to the
canvas width), and chat transcripts and markdown fences routinely strip them.
Sixteen actual `.txt` files survive; sixteen pasted code blocks may not.

### Allowed glyph palette (use only these + space)

```
full / shade density:   █  ▓  ▒  ░           (dark → light)
half blocks:            ▀ (upper)  ▄ (lower)  ▌ (left)  ▐ (right)
quadrant blocks:        ▖ ▗ ▘ ▝   ▙ ▟ ▛ ▜   ▚ ▞
space:                  (blank cell / background)
```

Nothing outside this set. (Box-drawing glyphs `─│┌┐└┘…` are reserved for the
**border the game draws around the art** — see *The canvas* — so they must not
appear inside the picture: a stray `│` inside the grid reads as a seam in the
game's own frame.) Four plain ASCII marks are also permitted, for tiny features
only (a light in a window, a glint on glass) — `.` `'` `*` `+`, and no others
— used sparingly; the block/shade set above should carry the whole image.
Validation checklist item 4 lists all **23** permitted codepoints, and that list
is the whole of what a delivered file may contain.

> Width caveat: the shade/half/full blocks (`█▀▄▓▒░`) are East-Asian *Ambiguous*
> width. The game already ships its border, its title art and its eleven
> opponent portraits in this same class and assumes narrow, so this palette is
> consistent — just don't reach for any glyph outside the list, some of which
> are unconditionally double-width and would shear the grid.

### Every line exactly the canvas width (the new constraint — illustrated)

The portraits were drawn into an unbordered block and were allowed to be short:
a 14-column line in an 18-column field simply drew left-aligned. This art is
drawn **inside a bordered box**, so a short line leaves a ragged hole between
the end of the picture and the game's own right-hand border — not a
left-aligned image. Every row must therefore be padded with spaces out to the
full canvas width.

Illustrated on a 48-column narrow row, with `·` standing in for the space
character (in the delivered file these are ordinary U+0020 spaces, not `·`):

```
        0         1         2         3         4       
        012345678901234567890123456789012345678901234567
row  9: ····▄▄▄▓▓▓▓▓▓▄▄▄····░░░░························   ← 48 cells: 4 lead + 20 art + 24 trailing spaces
row 10: ····█▓▓▓░░░░▓▓▓█····░░░░························   ← 48 cells
row 11: ········································████····   ← 48 cells: 40 blank, 4 of art, 4 blank
```

Each of those three rows is exactly 48 characters long, counting every leading
and trailing space; the two ruler lines above them are 48 columns wide too, so
the grid lines up under them. A row that is "empty sky" is 48 spaces — not an empty
line. The same holds at 92 columns, with 92 in place of 48.

You can check a finished file with:

```
python3 -c 'import sys
for i, l in enumerate(open(sys.argv[1], encoding="utf-8")): print(i + 1, len(l.rstrip("\n")))' cinder-narrow.txt
```

That must print the numbers 1 to 20, each followed by 48 (92 for a wide file),
and nothing after 20. Use this rather than `awk`/`wc -c`, which count bytes —
every glyph in the palette is multi-byte in UTF-8, so a byte count will look
wrong even when the file is right.

---

## The canvas (48 × 20 and 92 × 20) — and it is the art region's *interior*

The venue reserves one large rectangle for this art and draws a **single-weight
box around it**; the picture goes **inside** that box. So the drawable canvas is
the reserved rectangle **minus its one-cell border on every side**.

The rectangle is pinned, at both of the game's two fit sizes, by the test
`the_venue_bands_stack_and_the_art_takes_the_rest` in `src/layout.rs`:

```rust
let (art, portrait) = if cols < WIDE_LAYOUT_MIN_WIDTH {
    (Rect::new(7, 56, 5, 26), Rect::new(64, 85, 5, 19))
} else {
    (Rect::new(10, 103, 5, 26), Rect::new(114, 135, 5, 19))
};
```

`Rect::new(x0, x1, y0, y1)` is inclusive on all four edges, so the derivation
is:

| Terminal | Art `Rect` | Rect size | Border | **Canvas (the deliverable)** |
|---|---|---|---|---|
| 89 × 31 (minimum) | `x0=7, x1=56, y0=5, y1=26` | (56 − 7 + 1) × (26 − 5 + 1) = **50 × 22** | −1 cell each side | (50 − 2) × (22 − 2) = **48 × 20** |
| 139 × 31 (wide) | `x0=10, x1=103, y0=5, y1=26` | (103 − 10 + 1) × (26 − 5 + 1) = **94 × 22** | −1 cell each side | (94 − 2) × (22 − 2) = **92 × 20** |

Both canvases are **20 rows**; they differ only in width, 48 against 92.

```
       ┌────────────────────────────────────────────────┐   ← the border the GAME draws
       │................................................│   row 0    (48 cells; 92 at the wide size)
       │................................................│   row 1
       │................................................│   row 2
       │................................................│   rows 3..16  ← the body of the composition
       │................................................│   row 17
       │................................................│   row 18
       │................................................│   row 19
       └────────────────────────────────────────────────┘
```

> **If the venue's geometry ever moves again, re-derive these numbers from
> `VenueLayout` — do not copy them from this brief.** Nothing in the build reads
> this document, so nothing will catch it drifting out of date. The source of
> truth is `VenueLayout::new` in `src/layout.rs` and the Rects pinned in
> `the_venue_bands_stack_and_the_art_takes_the_rest`; the canvas is always that
> Rect's width and height each minus 2.

Keeping the border rather than handing the art the whole rectangle is
deliberate: the border is already drawn, it reads as a viewport onto the planet,
and it means no asset can collide with the box glyphs.

---

## Two assets per planet — sixteen files — and why

The two regions are the same 20 rows but **48 and 92 columns**: the wide one is
nearly twice the narrow one. One grid cannot serve both.

- The wide grid **centre-cropped** at 89 columns throws away 48 % of the
  composition — and it throws it away at the *minimum* terminal size, the one
  the product owner's R3 ruling said they least want to be the weak one.
- The narrow grid **centred** in the wide region leaves 22 blank columns on each
  side, which undoes exactly the dominance rulings R3 and R4 are about.

So: `assets/planets/<id>-narrow.txt` at 48 × 20 and
`assets/planets/<id>-wide.txt` at 92 × 20, for each of the eight planet ids —
sixteen files. And in as many words: **the narrow grid is its own composition of
the same subject, not a crop of the wide one.** Same venue, same mood, same
hand; re-staged for a frame half as wide. Expect the wide grid to show more of
the room and more of the view out the window, and the narrow one to sit closer
in on the same subject.

This spends a cost `spec.md`'s R3 ruling already named and the product owner
accepted — "each planet's art must work at two quite different sizes" — rather
than hiding it behind a crop rule.

### Rejected alternatives (recorded so a later spec can revisit them knowingly)

- **One asset per planet:** a single 92-wide grid composed so that its central
  48 columns stand alone, cropped at the narrow size. Halves the drawing work;
  rejected here because composing for a crop constrains both framings and the
  narrow one pays for it. If eight drawings are wanted instead of sixteen, this
  is the shape to ask for — deliberately, not by accident.
- **Making the art region the same width at both sizes** (the obvious way to
  need only one asset per planet) is **foreclosed by acceptance criterion 16**,
  which requires the art region to be *strictly larger* at 139 columns than at
  89. That is what makes the two-asset cost unavoidable rather than chosen, and
  it is the first thing a reader of this brief will ask.

---

## The aesthetic

- **The subject is the venue on that planet** — the tournament hall the player
  is standing in, and what is out its window. **Not** a map, **not** a planet
  seen from space, **not** a logo or a title card. The product owner's reason
  for reserving a region this large was that "it feels like you are actually in
  the venue, on the planet," and that is the pass/fail feeling. Interior first,
  view second: the room's architecture, the table, the light; then whatever the
  window or the opening shows of the world outside.
- **No figures at the table.** The opponent has their own portrait panel beside
  the art, drawn as a separate element; a face inside the art would compete with
  it. Distant silhouettes in a crowd are fine as texture — a discernible
  character is not.
- **Low-resolution monochrome, stylized, architectural.** Legible at a glance as
  a place: a horizon or floor line, a clear light source, foreground mass
  against a lighter background. Half and quarter blocks buy effective half-cell
  "sub-pixels", so 48 × 20 has more resolution than it looks, and 92 × 20 has
  room for real depth.
- **Fill the frame.** A small vignette floating in a 92 × 20 void reads as a
  rendering fault, which is precisely what the placeholder it replaces looks
  like. The composition should reach all four edges.
- **Original designs, no trademarked material.** No Star Wars locations,
  vehicles, architecture or iconography; no recognizable franchise ships,
  buildings or insignia; no logos or lettering of any kind. kaazap is an
  original game in the spirit of its own invented planet names, with the same
  no-copyrighted-music stance, and it is intended to be shared publicly
  (portfolio, itch.io) — so this constraint is harder here than it was for the
  portraits, because "a tournament hall on a desert world" invites the
  reference. Invent the place.
- **A consistent set.** The sixteen grids should feel drawn by one hand — same
  lighting logic (denser = shadow), same "monochrome pixel-art" register, same
  sense of scale — while each planet is visually **distinct**. The eight narrow
  grids must be pairwise different, and so must the eight wide ones.
- **The set has an arc.** The campaign runs Outer Rim → Mid Rim → Core, and the
  art should harden along it the way spec 016's faces harden down the difficulty
  ladder: the Outer Rim venues improvised, cramped, dirty and warm; the Mid Rim
  ones built on purpose, orderly, cooler; the Core ones formal, symmetrical,
  vast and cold. By Zenith the room should feel like it is judging the player.

---

## Per-planet art direction

`id` → the filenames (`<id>-narrow.txt`, `<id>-wide.txt`). `name`, `region` and
`blurb` are canon, from `PLANETS` in `src/campaign.rs`; the direction column is
drafted from each planet's own blurb and region.

**The crowd clause, restated here because these directions get read on their
own.** Where a direction asks for a crowd — Scree's "loose crowd pressed close
around a small table", The Spindle's "tiered seating looking down on a single
table" — draw it. *The aesthetic*'s "No figures at the table" bars a
**discernible character**, the kind that would compete with the opponent's
portrait panel beside the art; it does not bar the distant silhouettes those two
rooms get their pressure from. Bodies as texture and mass, yes; a face, a
readable figure, or anyone seated at the table, no.

| id | Name | Region | Blurb (canon) | Venue / view to author |
|---|---|---|---|---|
| `cinder` | Cinder | Outer Rim | "A slag-heap world where every hand is a warm-up." | The roughest venue of the eight: a lean-to hall welded from smelter plate, low and crooked, one bare lamp over a scarred table. Out the opening, terraced slag heaps still glowing dull at the base. Warm, dirty, improvised — the first room, and it looks like it. |
| `scree` | Scree | Outer Rim | "A rubble moon where the young come to make a name." | A hall dug into a rubble shelf — raw rock ceiling, crates for seating, a loose crowd pressed close around a small table. Out the gap, a boulder field under hard moonlight and the thin arc of a debris ring. Cramped, noisy, young. |
| `ashfall` | Ashfall | Outer Rim | "Dust, debt, and a scrapper who plays like she has both." | A scrap-built hall: salvaged panels, hanging chain lamps, ash drifted in grey wedges against the panes. Out the window, steady falling ash over a flat grey field and the hulk of something stripped for parts. Dim and close, the last of the Outer Rim. |
| `karrus` | Karrus | Mid Rim | "A way-station of brokers who never bet past their means." | The first venue built on purpose: a way-station concourse, straight lines, a ruled board of ledger rows on the back wall, tidy rank of tables. Out the tall window, docked freighters nose-in at a gantry. Orderly, lit evenly, unsentimental — money, not grit. |
| `drift` | Drift | Mid Rim | "A quiet station where an old hand waits out the years." | Near-empty and still: a wide station lounge with most of it in shadow, one table lit, chairs stacked along the wall. Out a long curved window, a slow turning starfield. The quietest composition in the set — restraint is the point, but the frame still fills. |
| `the-anvil` | The Anvil | Mid Rim | "A furnace-world where the hard cases hammer it out." | A gallery hung over a working foundry floor: heavy vaulting, a rail at the edge, two tables, the light coming from *below* and glaring up. Out past the rail, furnace mouths and rolling heat haze. The heaviest, hottest, densest grid — the Mid Rim's hard end. |
| `the-spindle` | The Spindle | Core | "The core-world tower where the table's sharpest hold court." | High in a tower: a narrow gallery with tiered seating looking down on a single table, tall slot windows floor to ceiling. Out them, spires stepping away below and traffic lanes ruled across the sky. Vertical, formal, cold, watched from above. |
| `zenith` | Zenith | Core | "The summit table, where the house's best has never lost." | The summit chamber, and the most imposing grid in the set: perfectly symmetrical, one table centred under a vaulted span, the whole composition converging on it. Out a full wall of glass, a planet-wide skyline far beneath cloud. Vast, still, absolute — this is the top of the ladder and it should read as such at a glance. |

---

## How it loads — and that it is **not** spec 029's work

For context only, so the format lands where it is going. The portraits'
mechanism is the model: authored text under `assets/`, `include_str!`-embedded
into a `&'static str` field on the profile struct
(`OpponentProfile.portrait`), drawn line-by-line by a clip-safe drawer. Planet
art would hang off a field on `Planet` in `src/campaign.rs`, and the venue would
pick the **widest asset that fits the region** and centre it.

**That rule is exact at the two fit sizes and unsettled at every width in
between — the later art spec decides it, not this brief.** The art interior is
`span_w * 7/8 - 2` of a span that varies continuously with the terminal width
(`VenueLayout::new`), so it equals 48 or 92 only at exactly 89 and 139 columns:

| Terminal | `span_w` | `art_w` | Interior | Widest asset that fits | Blank columns each side |
|---|---|---|---|---|---|
| 89 (minimum fit) | 58 | 50 | 48 | narrow, 48 | 0 |
| 120 | 89 | 77 | 75 | narrow, 48 | 13 and 14 |
| 138 | 107 | 93 | 91 | narrow, 48 | 21 and 22 |
| 139 (wide fit) | 108 | 94 | 92 | wide, 92 | 0 |
| 160 | 129 | 112 | 110 | wide, 92 | 9 and 9 |

At 138 columns "widest that fits, centred" leaves 21 and 22 blank columns — the
same emptiness this brief rejects the single-asset option for ("leaves 22 blank
columns on each side, which undoes exactly the dominance rulings R3 and R4 are
about") — and above 139 the gap opens again and widens with the terminal. So
centring is not a settled answer away from the two fit sizes. **The later art
spec chooses** among:

- **letterbox** — centre and leave the blank columns, accepting up to ~22 a side
  as the price of one rule with no width threshold in it;
- **stretch** — resample the grid to the interior width, which on a character
  grid means duplicating whole columns, visibly, in a hand-authored image;
- **tile or extend** — repeat or continue edge material outward, which the
  compositions would have to be authored for;
- **a third size**, or one size per band of widths, which multiplies the drawing
  work this brief already prices at sixteen files.

Nothing here picks one, and spec 029 builds none of them. **What it means for the
drawing now: nothing changes.** Author exactly 48 × 20 and 92 × 20 — those two
grids are what the two fit sizes need under all four options, and two of the four
would arrive later as an additional ask rather than a change to these sixteen
files.

**None of that is built in spec 029, and wiring it up is the deferred art spec's
job.** Spec 029 reserves the region and draws a placeholder in it; this brief
specifies the *artifact*, not the plumbing. Nothing in it is a work order
against spec 029's branch.

---

## Validation checklist (Claude Code enforces this before art ships)

For every delivery, Claude Code will confirm and, if needed, bounce back:

1. **Sixteen files present and correctly named** — `<id>-narrow.txt` and
   `<id>-wide.txt` for each of `cinder`, `scree`, `ashfall`, `karrus`, `drift`,
   `the-anvil`, `the-spindle`, `zenith`, in `assets/planets/`.
2. **Exactly 20 lines** in every file, with a single trailing newline and no
   21st line.
3. **Every line exactly the canvas width in displayed columns** — 48 in every
   `-narrow.txt`, 92 in every `-wide.txt` — trailing spaces included.
4. **Only whitelisted codepoints — these 23 and nothing else.** No color or
   escape codes, no wide, zero-width or combining characters, no tabs, no
   box-drawing glyphs, no letters or digits:
   - space — `U+0020`;
   - shade and full blocks — `U+2591` ░, `U+2592` ▒, `U+2593` ▓, `U+2588` █;
   - half blocks — `U+2580` ▀, `U+2584` ▄, `U+258C` ▌, `U+2590` ▐;
   - quadrant blocks — `U+2596` ▖, `U+2597` ▗, `U+2598` ▘, `U+259D` ▝,
     `U+2599` ▙, `U+259F` ▟, `U+259B` ▛, `U+259C` ▜, `U+259A` ▚,
     `U+259E` ▞;
   - the four permitted ASCII marks — `U+002E` `.`, `U+0027` `'`, `U+002A` `*`,
     `U+002B` `+`.

   That closed list is the whole check, so one command decides it. Whether the
   ASCII marks were used *sparingly* is a judgement, not a mechanical test: it
   belongs to item 7's look, not to this item.
5. **Valid UTF-8, and LF line endings — not CRLF.** Every file decodes as UTF-8,
   and no file contains a carriage return.
6. The eight `-narrow` grids are **pairwise distinct** strings, and the eight
   `-wide` grids likewise.
7. Rendered in the actual running venue at **89 × 31 and again at 139 × 31**:
   the art reads as a place, fills its frame, is not clipped, and does not
   compete with the opponent's portrait panel beside it — the product owner's
   go/no-go look.

Items 4 and 5 are one command — it raises on invalid UTF-8, reports any carriage
return, and prints every codepoint outside the whitelist:

```
python3 -c 'import sys
ok = set(" .\x27*+") | {chr(c) for c in [0x2588, 0x2593, 0x2592, 0x2591,
    0x2580, 0x2584, 0x258C, 0x2590, 0x2596, 0x2597, 0x2598, 0x259D,
    0x2599, 0x259F, 0x259B, 0x259C, 0x259A, 0x259E]}
for p in sys.argv[1:]:
    b = open(p, "rb").read()
    t = b.decode("utf-8")
    bad = sorted({"U+%04X" % ord(c) for c in t.replace("\n", "") if c not in ok})
    print(p, "CRLF!" if b"\r" in b else "lf-ok", "bad: " + ",".join(bad) if bad else "glyphs-ok")' assets/planets/*.txt
```

Deliver the sixteen files, and Claude Code drops them into `assets/planets/`,
runs the dimension, encoding, palette and distinctness checks, and renders them
at both fit sizes for the product owner's sign-off.
