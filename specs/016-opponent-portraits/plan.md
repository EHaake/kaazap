# Plan: Opponent portraits — spec 016

**Status**: Signed off (skeptical-reviewer) — pending person approval
**Implements**: `spec.md` in this directory

## Context

Every opponent already carries identity data — `OpponentProfile { id, name, difficulty,
blurb, … }` in `opponent.rs` — but the player never sees a face. This spec gives each of the
10 roster opponents (plus a generic fallback for the pre-roster `DEFAULT_OPPONENT`) a
low-resolution, monochrome, stylized alien-face portrait, shown in an always-visible
**opponent-presence panel** beside the board during a match and as a **preview** in
opponent-select and on a focused campaign-map node.

This is presentation work over a settled data model. No engine, save-format, AI, or campaign
change. The work is: a portrait render path (a new `portrait.rs`), one authored art asset per
opponent under `assets/portraits/`, a `portrait` field on `OpponentProfile`, an in-match panel
that grows `BoardLayout` and the enforced minimum terminal, and a shared presence-panel drawer
reused by all three surfaces.

Two things make this more than "draw some text": the in-match panel **grows the minimum
terminal size** (a real, spec-mandated change to `config.rs` + the fit tests), and the portrait
vocabulary **deviates from `design/brief.md`** (which today rules out block-art and pictorial
elements) — see "The two cruxes" below. Both are surfaced, not silent.

## What the code already gives us

- **The monochrome render model (hard constraint).** `frame.rs` is `Cell { ch, emphasis }`
  with `Emphasis` = Normal/Strong/Muted/Alert and **no color path** — `render.rs` only ever
  emits `SetAttribute`, never a color. Portraits render as characters + those four attributes,
  full stop. `draw_char`/`draw_text`/`draw_box`/`draw_ghost_slot`/`clear_rect` are all
  clip-safe. This is the spec's monochrome guarantee, structural rather than policed.
- **Opponent identity is already on `GameState`.** `GameState.opponent_profile: OpponentProfile`
  (`game.rs:58`) is `Copy` and all-`&'static`, so `board.rs` can read the opponent's name and
  (new) portrait directly in `BoardView::draw` — no new plumbing through `app.rs`. Adding a
  `portrait: &'static str` field is like the existing `blurb` field: data, not logic.
- **Assets are compile-time embedded.** Title/overlay text use `include_str!`
  (`menu.rs:45`, `overlay.rs:34-42`); SFX/music use `include_bytes!` (`audio.rs`). Portraits
  follow the same pattern: authored `.txt` art files embedded via `include_str!`, so the binary
  stays self-contained and a missing/renamed file is a compile error, not a runtime blank.
- **The board is a fixed block centered in the terminal.** `BoardLayout::new` (`layout.rs:77`)
  centers `BOARD_WIDTH` (89) horizontally: `left = (num_cols - BOARD_WIDTH)/2`. `Config::min_size`
  (`config.rs:16`) returns `(BOARD_WIDTH, BOARD_BLOCK_HEIGHT)` = **(89, 31)**, and both the
  startup bail (`from_terminal`) and the runtime too-small recovery (`app.rs:173`,
  `draw_too_small`) read `min_size()` — so growing the minimum is a **one-place change** that
  propagates everywhere. The board is drawn against `BoardLayout`, the single source of truth.
- **`CardView` is the precedent for a fixed-size rendered element** (`card.rs:154`): a fixed
  9×5 box, `.weight`/`.emphasis`, all-or-nothing draw. Portraits mirror this shape — a fixed
  `PORTRAIT_WIDTH × PORTRAIT_HEIGHT` art block with its own drawer — living in a parallel
  `portrait.rs` module (rendering stays decoupled from `opponent.rs`'s data).
- **The two preview surfaces already own everything they need.** `opponent_select.rs` draws
  from `OPPONENTS[selected]` (has `.portrait`); `campaign_map.rs` has `profile`/`run` and
  resolves opponents via `opponent_by_id`. Neither needs an `app.rs` change. `opponent_select`
  centers a ~27-col list via `MenuLayout` inside an 89-col frame — there is already room beside
  it for a preview panel, so **the previews do not force the minimum wider; only the in-match
  panel does.**
- **`CampaignMapLayout` (`layout.rs:181`) is full-screen and reflows.** Nodes are placed at
  normalized `(fx, fy)` within `field`; shrinking `field.x1` reflows them with no position
  edits. Two guard tests (`campaign_map_layout_fits_the_minimum_terminal`,
  `the_campaign_map_is_legible_at_the_minimum_terminal`) pin node/label placement at the
  minimum and must move to the new minimum when the field changes.

## The two cruxes

### Crux 1 — monochrome faces vs. `design/brief.md` (a declared deviation)

`design/brief.md` is the standing visual contract. Today it **explicitly rules out** the exact
things a portrait needs:

- *"Unicode block-art flourishes (sparklines, shade-block gradients, big-type banners) outside
  the existing title art."*
- Skeuomorphism boundary: *"Cards are the only physical metaphor… The board is a layout, not a
  picture of a table,"* and *"no card-back art beyond the `?` face."*
- *"Glyphs beyond box-drawing and plain text must justify themselves individually."*

Spec 016 is approved and **human-ruled** to add monochrome pixel-art alien portraits, so the
*product* question is settled — this is not an open fork. But per the constitution ("if a spec
needs to [contradict the standing contract], the constitution gets amended first, explicitly"),
the brief must gain a **bounded-exception amendment**, mirroring the existing Motion amendment
that admitted the campaign-map starfield. The exception is narrow and stated now so sign-off of
this plan *is* the approval to proceed: **opponent portraits are the one pictorial element —
monochrome, static, opponent-only, drawn in a single fixed portrait frame, no color, no
animation.** The card frame remains the only *card-shaped* box; the portrait frame is a distinct,
clearly-non-card element. Per the sign-off (finding 1), the brief amendment lands in its own
commit at **T001b** — right after the T001 spike confirms the vocabulary, before the rest of
the portraits and panels — so the standing contract sanctions portraits before the bulk of the
contradicting code lands; the `DECISIONS.md` entry rides the T008 close-out. Declaring the
exception here at sign-off is what blesses it.

**Glyph vocabulary (a technical choice, mine to make, gated by the spike).** Portraits use a
small, deliberate set — quadrant/half/full block glyphs (`▁▂▄▆█▌▐▖▗▘▝▙▟` …) and at most the
shade ramp (`░▒▓`) for interior modeling, plus plain characters for features (e.g. eyes) —
rendered at uniform `Emphasis::Normal` (depth comes from glyph choice, not attribute layers, so
parsing stays trivial: one char per cell). This is exactly what the **art-format spike** (T001)
validates before the rest are authored; the brief amendment blesses the confirmed vocabulary.
If the spike can't reach "recognizable as a face with eyes and expression," we stop and revisit
the approach (e.g. human-supplied source art) per the spec's human-ruled escape hatch — that is
a phase pause to the person, not a silent downgrade.

### Crux 2 — always-visible in-match panel grows the minimum terminal

The board is 89 wide and already equals today's minimum, so an always-visible panel beside it
has to grow the minimum. The spec settles the shape: *"the mirrored space opposite the panel is
left for a future player-status panel… so the layout is intentionally asymmetric for now."*

**Resolution: keep the board centered and reserve symmetric margin.** `BoardLayout` already
centers `BOARD_WIDTH`; the opponent-presence panel is drawn in the **right margin**, anchored
`PANEL_GAP` right of the centered board's right edge, and the equal **left margin is left empty**
— the reserved home for the future player-status panel. The board's own layout and centering are
untouched (spec: "the board keeps its current layout and size"); we only *add* a panel Rect and
its drawing, and *raise the minimum*. New minimum width:

```
IN_MATCH_MIN_WIDTH = BOARD_WIDTH + 2 * (PANEL_GAP + PANEL_W)   // 89 + 2*(3+22) = 139
```

so the **new minimum terminal is 139 × 31** (height unchanged: the panel is sized to fit inside
`BOARD_BLOCK_HEIGHT`). Chosen over centering a `[board | panel]` block (which would shove the
board left-of-center and contradict "mirrored space… is left for a future panel"). The wider
minimum is exactly what the spec's resolved decision accepts ("growing the minimum terminal…
over a responsive panel that hides on narrow terminals"). Below it, the existing
`from_terminal`/`draw_too_small` machinery already errors with the required dimensions,
unchanged — it just reads the larger `min_size()`.

## Design

Concrete constants (initial values; the spike may tune the portrait size, and because every
downstream size is *derived* from it, the panel width and the minimum move together):

```
PORTRAIT_WIDTH  = 18     PORTRAIT_HEIGHT = 12          // the art block (portrait.rs)
PANEL_PAD_X     = 1      PANEL_GAP       = 3
PANEL_W         = PORTRAIT_WIDTH + 2*PANEL_PAD_X + 2   // = 22  (interior pad + border)
PANEL_H_INMATCH = 2 + 1 + PORTRAIT_HEIGHT + 1 + 2      // = 18  (border, name, portrait, gap, reserved)
```

### 1. `src/portrait.rs` — the render path (foundational)

New module (added to `lib.rs`), the portrait counterpart to `card.rs`'s `CardView`, pure
rendering + constants, no dependency on `opponent.rs`:

- `pub const PORTRAIT_WIDTH/PORTRAIT_HEIGHT`.
- `pub fn draw_portrait(frame, x, y, art: &str, emphasis)` — draws each line of `art`
  left-to-right at `(x, y+row)` via `draw_text`, clip-safe (never panics off-frame, mirroring
  `CardView::draw`'s bounds guard). Art is a plain multi-line string; one glyph per cell.
- `pub fn draw_presence_panel(frame, panel: Rect, name: &str, art: &str)` (added in §4, the
  **shared component**) — a `BorderWeight::Single` box, the opponent `name` (`Emphasis::Strong`,
  centered on the top interior row), `draw_portrait` centered below it, and any remaining panel
  height left blank (the reserved banter-line / round-pips space — spec: "reserved here, not
  built"). Callers size the `panel` Rect: in-match passes a `PANEL_H_INMATCH`-tall rect; the
  previews pass a snug rect (no reserved rows). One drawer, three callers.

### 2. `assets/portraits/*.txt` + `OpponentProfile.portrait` (the cast)

- One authored art file per roster opponent (`greeb.txt`, `dax.txt`, …, `sovereign.txt`) plus
  `generic.txt`, each **original** (no trademarked species), monochrome, `PORTRAIT_HEIGHT` lines
  of ≤ `PORTRAIT_WIDTH` cells. Claude-authored — the `gen_sfx.py` "synthesized in-repo, no
  license encumbrance" spirit, but **authored assets, not a generation script**: procedurally
  synthesizing a *recognizable face* isn't feasible, and the spec offers "a script and/or
  authored assets," so we take the authored branch (Simplicity — no script that can't do the
  job). CREDITS/DECISIONS record the originality.
- Add `pub portrait: &'static str` to `OpponentProfile` (`opponent.rs`), populated per entry
  with `include_str!("../assets/portraits/<id>.txt")`; `DEFAULT_OPPONENT.portrait =
  include_str!(".../generic.txt")`. The struct stays `Copy`/`Debug`. Only the 11 struct
  literals need the field — `game.rs`'s test constructions all use `..DEFAULT_OPPONENT`.
- **Fallback is automatic:** an unknown/absent id resolves to `DEFAULT_OPPONENT`
  (`opponent_by_id → None → caller falls back`), whose portrait is the generic — so the Quick
  Play default opponent and any unmapped opponent show the generic and the panel is never blank.

### 3. `src/layout.rs` + `src/config.rs` — geometry + the grown minimum (foundational)

- `BoardLayout` gains `pub opponent_panel: Rect`, computed in `new` where `left` is in scope:
  `panel_x0 = left + BOARD_WIDTH + PANEL_GAP`, width `PANEL_W`, top-aligned with the board block
  (`top`), height `PANEL_H_INMATCH`. Board centering and every existing board Rect are unchanged.
- `pub const IN_MATCH_MIN_WIDTH` (layout.rs) `= BOARD_WIDTH + 2*(PANEL_GAP + PANEL_W)` = 139.
- `Config::min_size` returns `(IN_MATCH_MIN_WIDTH, BOARD_BLOCK_HEIGHT)`; update its doc comment
  (no longer "~89"). `fits`/`from_terminal`/`draw_too_small` need no change — they read
  `min_size()`.
- `CampaignMapLayout` gains `pub portrait_panel: Rect` on the right of the field band (width
  `PANEL_W`, ~`2 + 1 + PORTRAIT_HEIGHT` tall, near the field top), and `field.x1` shrinks to
  `portrait_panel.x0 - 2`. The reduced field spans more columns than the old 89-wide field, but
  (sign-off finding 2) that does **not** by itself guarantee rail clearance: the old field had
  no rail, so a far-right cursored label can come within ~1 cell of `portrait_panel` — T004's
  "clear of `portrait_panel`" assertion is the real check, and if it fails, widen the field/rail
  gap or `FIELD_MARGIN_X`.

### 4. Wire the presence panel onto every surface (shared component + three callers)

- **In-match** (`board.rs`): `BoardView::draw` calls
  `draw_presence_panel(layout.opponent_panel, state.opponent_profile.name,
  state.opponent_profile.portrait)` after drawing the board. It lands in the empty right margin,
  clear of the board's cards. The board's existing "Opponent" header label and centered outcome
  popup are unaffected.
- **Opponent-select** (`opponent_select.rs`): a pure `preview_rect(config) -> Rect` places the
  panel to the right of the centered list (`x0` a fixed offset right of `MenuLayout::center_x`,
  clearing the widest roster row), vertically aligned with the list; `draw` calls
  `draw_presence_panel(preview_rect, o.name, o.portrait)` for the cursored opponent.
- **Campaign-map** (`campaign_map.rs`): `draw` resolves the focused planet's shown opponent —
  `run.next_opponent(planet)`, else the planet's **last** opponent (a cleared planet still has a
  resident face), else `DEFAULT_OPPONENT` — and calls
  `draw_presence_panel(layout.portrait_panel, name, portrait)`.

## Files

- `src/portrait.rs` — **new**: `PORTRAIT_WIDTH/HEIGHT`, `draw_portrait`, `draw_presence_panel`;
  clip-safety tests. Added to `src/lib.rs`.
- `assets/portraits/*.txt` — **new**: `generic.txt` + one per roster opponent (11 files).
- `src/opponent.rs` — add `portrait: &'static str` field + `include_str!` per profile;
  dimension / distinctness / fallback tests.
- `src/layout.rs` — `BoardLayout.opponent_panel`, `IN_MATCH_MIN_WIDTH`,
  `CampaignMapLayout.portrait_panel` (+ shrunk `field`); board+panel fit test; updated
  campaign-map fit/legibility tests.
- `src/config.rs` — `min_size` → `IN_MATCH_MIN_WIDTH`; doc comment; updated min-size test.
- `src/board.rs` — draw the in-match presence panel.
- `src/opponent_select.rs` — `preview_rect` + draw the preview; preview fit test.
- `src/campaign_map.rs` — resolve the focused opponent + draw the preview.
- `design/brief.md`, `DECISIONS.md`, `ROADMAP.md`, `Readme.md`, `assets/CREDITS.md` — close-out
  docs (brief amendment, decisions, minimum-size, originality).
- **No change:** `game.rs`, `card.rs`, `screen.rs`, `app.rs` (the board view and both preview
  screens already receive everything they need), `render.rs`, save format.

## Tests

Each claim about behavior names the task that owns its check. Two claims are **not**
unit-testable and are marked for driver/human verification.

- **Portrait dimensions** (T001 for the spike pair; T002 for all): every portrait
  (`OPPONENTS[*].portrait` + `DEFAULT_OPPONENT.portrait`) has `lines().count() == PORTRAIT_HEIGHT`
  and every line `chars().count() <= PORTRAIT_WIDTH`. (`<=`, not `==`, so trailing-whitespace
  stripping by editors/pre-commit can't break the panel — short rows draw left-aligned into a
  reserved-width panel.) Guards "portraits fit the panel."
- **Distinctness** (T002): the 10 roster `.portrait` strings are pairwise distinct. Guards "each
  of the 10 shows a **distinct** portrait" (distinct *strings* is the machine-checkable proxy;
  "reads as a face" is the human part below).
- **Fallback** (T002): `DEFAULT_OPPONENT.portrait == generic` and is non-empty; the existing
  `opponent_by_id_resolves_known_and_rejects_unknown` already covers unknown-id → default.
  Guards "fallback opponent shows the generic; panel never blank."
- **Board + panel fit the grown minimum** (T003): at `(IN_MATCH_MIN_WIDTH, 31)`,
  `opponent_panel` is on-frame, sits right of the opponent half (`opponent_panel.x0 >
  opponent.hand.x1`), does not overlap the board, and its bottom `<= num_rows-1`. Guards the
  in-match panel is "always visible… beside the board" at the minimum.
- **Minimum is updated and enforced** (T003): the renamed `config_min_size` test asserts
  `min_size() == (139, 31)` and `== (IN_MATCH_MIN_WIDTH, BOARD_BLOCK_HEIGHT)`. The
  below-minimum error path (`from_terminal`/`draw_too_small`) is unchanged mechanism, driven by
  this value.
- **Campaign map still legible at the new minimum** (T004): the two map guard tests, moved to
  `(IN_MATCH_MIN_WIDTH, 31)`, assert every node + label stays within the **reduced** field and
  clear of `portrait_panel`. Guards "every existing screen still fits."
- **Opponent-select preview fits** (T005): `preview_rect(min)` is on-frame and `x0 >` the
  widest roster row's right edge, so the preview never overlaps the list at the minimum.
- **Legible as faces, eyes + expression discernible** — *not unit-testable.* Verified by the
  T001 spike gate (the go/no-go look) and the close-out driver, per the spec ("verified by
  looking at the running game, not only by tests"). **Marked needing verification.**
- **No color introduced / monochrome preserved** — structural: `frame.rs` has no color API and
  `draw_portrait` only ever calls `draw_text` with an `Emphasis`. There is no positive unit test
  for the absence of color; verified by the render model + the close-out driver look. **Marked
  needing verification.**

## Verification

- `cargo build` (no new warnings) + `cargo test` (reported verbatim, per the constitution).
- **Driver** (back up + checksum-restore the real profile first, per the standing data-safety
  practice): open opponent-select and cursor the roster — 10 distinct faces, none blank; start a
  Quick Play match (default opponent → generic portrait) and a campaign match — the portrait +
  name panel is visible beside the board throughout; on the campaign map, focus each node — the
  next opponent's face shows in the rail. Snapshots at the new minimum (139×31) and wider
  (~180). Confirm faces read (eyes + expression) and nothing renders in color.

## Non-goals (from spec)

No animation (blink/poses); no player-side portrait or player-status panel (the mirrored space
is reserved, empty); no banter text (space reserved, not built); no color, ever; no runtime
portrait editor or art-authoring UI. Engine, AI, save format, and campaign logic are untouched.
