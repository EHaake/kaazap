# Design Brief: Kaazap

Visual and interaction direction for Kaazap's TUI. Functional
requirements live in the spec that consumes this
(`specs/002-ui-overhaul/spec.md` first); this document is the direction
those screens get designed and built against. There is no separate
visual-design tool in the loop for a terminal app — the "design process"
is the implementation itself, so this brief is the reference the
implementer checks rendering work against.

## Identity in one line

A stark, monochrome card table: the game's character comes from
precision and restraint, not decoration.

## Audience and their existing visual vocabulary

Two audiences overlap here, and both already own a visual language:

- **KOTOR players** know the Pazaak *table*: a two-sided layout, your
  cards against theirs, a dealer row building downward and a small hand
  of side-deck cards you guard. Kaazap borrows that spatial structure —
  the two-sided board IS the recognizable shape — while deliberately
  not borrowing KOTOR's card color-coding (see Palette).
- **Terminal dwellers** carry the heritage of monochrome phosphor
  displays and box-drawing interfaces (the Norton Commander / classic
  curses school): interfaces that earned clarity through alignment,
  weight, and brightness because color didn't exist. That constraint-
  as-character is the aesthetic Kaazap runs toward on purpose.

## What to explicitly avoid

Named and ruled out, because they are the current defaults an
AI-assisted TUI reaches for:

- **Color theming of any kind** — no 256-color palettes, no
  catppuccin/dracula-style theme, no "just a splash" that grows. Ruled
  out by explicit decision (see Palette for the one narrow future door).
- **The modern rounded-TUI look** (lazygit/btop school): rounded
  corners, padded panels, dashboard density. Kaazap is a table, not a
  dashboard.
- **Nerd-font icons and emoji.** Glyphs beyond box-drawing and plain
  text must justify themselves individually; iconography is not the
  vocabulary here.
- **Double-line-everywhere DOS pastiche.** Retro is the heritage, not a
  costume — double/heavy weight is reserved as emphasis (see below),
  never the default texture.
- **Unicode block-art flourishes** (sparklines, shade-block gradients,
  big-type banners) outside the existing title art.

## Palette

Strictly monochrome: the terminal's default foreground on default
background, and nothing else. This is a deliberate identity decision
(human-ruled during spec 002 scoping): starkness and simplicity ARE the
character of the game.

Emphasis therefore has exactly three axes, used as semantic tokens:

| Token | Rendering | Meaning |
|---|---|---|
| `emphasis.selected` | heavy border weight | the cursor is here |
| `emphasis.strong` | bold | the thing to look at (active turn, totals) |
| `emphasis.muted` | dim | spent, inactive, opponent's private info |
| `emphasis.alert` | inverse video | interrupts: OVER 20, BUSTED, round/game outcome |

Rules: one axis at a time per element wherever possible; alert
(inverse) is rationed to genuinely interruptive states so it never
becomes wallpaper. Attributes are emphasis, not decoration — a screen
with nothing special happening renders almost entirely in plain weight.

**The accent-color door:** a future ruling MAY admit a single accent
hue for a small number of elements. It is not part of this brief, no
work anticipates it beyond the styled-cell architecture existing, and
"the game becomes colorful" is explicitly ruled out.

## Typography

The medium fixes the typeface (user's monospace terminal font), so
typography here means glyph discipline:

- **Borders**: sharp single-line box drawing (`┌ ─ ┐ │ └ ┘`) as the
  default weight everywhere; heavy (`┏ ━ ┓ ┃ ┗ ┛`) exclusively for
  `emphasis.selected`. Two weights, no mixing of styles within a
  screen, ASCII `+-|` retired.
- **Labels**: Title Case for zone labels and menu items; UPPERCASE
  reserved for alerts ("OVER 20!", "BUSTED!!").
- **Key hints**: the established parenthetical convention —
  `(n: next round)`, `(c cancels)` — everywhere a key is being taught.

## Signature element

**The card frame.** It is recurring (dealer rows, played rows, hands —
later the shop, pack opening, and deck views), functional (its face
text carries kind and value: `+4`, `±3`, `2&4`, `±1T`; its border
weight carries selection; its brightness carries spent/hidden state),
and distinctive — a single monochrome box-drawn card reading `±3` is
recognizably this game and nothing else. Every screen that shows a
card shows it in this exact frame; nothing else on screen uses a
card-shaped bordered box, so the shape stays unambiguous.

## Motion

Motion is emphasis, not decoration — the animated corollary of the
palette rules. The baseline (spec 002): **exactly one thing moves on a
still screen** — the current selection breathes, a gentle two-phase
pulse between emphasis states at one shared cadence, on every screen
that has a selection. Stillness everywhere else is what makes the one
moving thing legible.

A future, deliberate animation pass (roadmapped) may add sparse
eye-guiding moments during play — a dealt card arriving, a flip
resolving, a total changing. Its rule is set now: build on the pulse's
vocabulary (emphasis transitions over time), never particle effects,
sweeps, or continuous ambient motion. If everything moves, nothing is
emphasized.

## Skeuomorphism boundary

Cards are the only physical metaphor, rendered as bordered rectangles
with face text — nothing more. No table texture, no card-back art
beyond the `?` face, no simulated depth or stacking offsets. The board
is a layout, not a picture of a table.

## Screens to design

Mapping to spec flows (details live in the spec, not here):

- **Game board** — the two-sided table: labeled zones (dealer row,
  played row, hand), score/status headers, one status line owning
  turn/prompt/alert text.
- **Start menu** — existing title art (kept), menu list driven by the
  same cursor-selection vocabulary as the board.
- **Overlays** — controls help, How-to-Play rules, and the
  terminal-too-small recovery state; one shared overlay frame.
- **Future screens** (designed-for, not built): shop, pack opening,
  opponent select — all list-or-grid selections that must inherit the
  cursor vocabulary and card frame without new invention.

## Voice

Terse and imperative. Labels name things ("Hand", "Played"); prompts
state the choice and its keys ("+3 (h) or -3 (l)? (c cancels)");
alerts state the fact ("OVER 20!"). No apologies, no filler, no
exclamation marks outside alerts. A confirmation echoes the action's
own vocabulary rather than inventing synonyms.

## Open for implementation to refine

- Exact zone label wording and placement.
- How far `emphasis.muted` (dim) extends (spent hand slots? the
  non-active side entirely?) — start narrow, widen only if play-tests
  read poorly.
- Heavy vs. double-line for `emphasis.selected` if a common terminal
  font renders heavy box-drawing badly (double `╔ ═ ╗` is the approved
  fallback; decide once during implementation, verified in real
  terminals).
- Status-line composition when two messages compete (e.g. prompt while
  over 20).
