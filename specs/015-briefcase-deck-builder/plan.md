# Plan: Two-panel "briefcase" deck-builder — spec 015

**Status**: Draft — pending review (Plan Mode + skeptical-reviewer)
**Implements**: `spec.md` in this directory

## Context

The side-deck builder (`src/deck_builder.rs`) is a single centered grid of every owned card
type, each `CardView` carrying an `in-deck/owned` badge and leaning on **three** border
weights (Heavy = cursor, Double = in-deck, Single = owned-not-in-deck) to encode where each
card is. Spec 015 reshapes it into two side-by-side panels — **Collection** (left) | **Deck**
(right) — where building a deck means moving a card *copy* across from one panel to the other,
and makes the builder reachable from the campaign map for between-match retooling. Deck
semantics are unchanged (10 copies drawn from owned cards; the hand is a random sample via
`deal_hand`, so order is cosmetic).

This is a screen + layout reshape plus one small new entry point. No engine, save-format, or
profile-model change: the same `Profile` methods do the work, validated in one place.

## What the code already gives us

- **Profile model (unchanged).** `collection_by_type() -> Vec<CardEntry{card, owned, in_deck}>`
  (canonical order, unowned omitted), `deck() -> &[Card]`, **`try_add_to_deck(card) -> bool`**
  (adds iff a spare copy exists AND deck < 10), `remove_from_deck(card) -> bool`,
  `deck_is_valid()`. The screen only expresses *intent*; the app applies it and saves
  (`app.rs:675-700`). Split-by-location math is just `available = owned - in_deck`.
- **Screen shape.** `Screen::DeckBuilder { state: DeckBuilderState }` (`screen.rs:11`); the
  profile-aware variant of the `opponent_select.rs` pattern — `handle_input(key, &profile)
  -> Option<BuildOutcome>`, `draw(frame, &config, &profile, pulse)`, one app arm. `?`-help
  already returns `None` for this screen (`app.rs:573-578`) — no overlay wiring.
- **Rendering primitives.** `CardView` (`card.rs:154`) is a fixed **9×5** (`CARD_WIDTH=9`,
  `CARD_HEIGHT=5`) all-or-nothing frame with `.weight`/`.emphasis`. In `frame.rs`:
  `draw_box(rect, weight, emphasis)` draws a perimeter (the only bordered-box primitive),
  `draw_text` / `draw_text_in(rect, row, align, ..)` / `draw_text_centered` place text,
  `clear_rect` blanks. `Emphasis`: Normal/Strong(bold)/Muted(dim)/Alert(reverse).
- **The board is the side-by-side precedent (`board.rs` / `layout.rs`).** `BoardLayout` splits
  the screen left/right around a `divider_x` into two `SideLayout`s and draws borderless zones
  with plain one-row-above labels; `draw_popup` is the in-repo "bordered box + centered lines"
  template. `GridLayout` (`layout.rs:222`) is today's single-grid geometry (cell pitch
  `CELL_W=12`, `CELL_H=7`; fields `center_x/title_y/readout_y/hint_y`). **No existing
  two-bordered-panel helper — this plan adds one.**
- **The shop's map round-trip is the entry-point pattern.** Map `b` → `MapOutcome::OpenShop`
  → `open_shop()`; `ShopOutcome::Back` → `open_campaign_map()` (`app.rs:722-748`). It works
  because the shop has exactly one caller, so `Back` can hardcode its destination.
- **No origin tracking today (the real work).** All three deck-builder entries funnel through
  a param-less `open_deck_builder()`, and `BuildOutcome::Back` hardcodes `start_menu()`
  (`app.rs:695-698`). This already misbehaves: the campaign-launch "incomplete deck" divert
  (`app.rs:445`) drops the player on the **menu**, not the map. Adding a map entry forces an
  origin, which also fixes that latent bug.

## The layout decision (the crux)

> **Superseded at the first visual review — see "Revision: the fixed-album redesign"
> below.** The first cut (a scrolling collection) shipped as T001–T002; the review
> replaced it with a fixed, content-sized album (no scrolling). This section is kept
> as the record of *why* scrolling was first chosen.

At the guaranteed **89×31** minimum, two half-width panels get ~44 cols each → **4 card
columns** (9-wide cards at board pitch 10: `4*10-1=39` within the ~41-col interior). After a title, the two
panel labels (one shared row), the `Deck: N/10` readout and the controls hint, bordered panels
leave room for **~3 card rows** (with a count caption under each card). That's **12 visible
frames per panel**.

- The **Deck** panel holds ≤ 10 distinct types → always fits, no scroll.
- The **Collection** panel can reach all **15** types (a spare of everything) → **exceeds 12**.

**Resolution: the collection grid scrolls vertically** (a `collection_scroll` row offset kept
so the cursor stays visible); at 89×31 it shows ~3 rows and scrolls only for a near-complete
collection, and larger terminals show more rows and rarely scroll. Chosen over a borderless
zero-slack 4×4 max-pack (which would fit 15 exactly only by dropping panel borders and the
count caption — fragile at the minimum and a bigger change to the card's look). Card frames and
the count caption stay; the panels keep borders (the "briefcase" feel).

## Design

### 1. `src/layout.rs` — a `BriefcaseLayout` type (foundational)

New geometry mirroring `BoardLayout`'s split, sized from `Config`:
- `title_y`, and a `readout_y` / `hint_y` on shared rows.
- Two panel `Rect`s (`collection`, `deck`) side by side within `num_cols` (bordered, a center
  gap), each with a `label` anchor.
- `visible_rows` and `cols` (=4) derived from panel height/width and the card pitch, plus
  `card_origin(panel, visible_index) -> (x, y)` for a card at a visible grid slot.
- Deterministic and pure (takes `Config`), so it unit-tests without a terminal.

### 2. `src/deck_builder.rs` — reshape the screen

- **State:** replace the single `cursor` with `{ active: Panel, collection_cursor: usize,
  deck_cursor: usize, collection_scroll: usize }` where `enum Panel { Collection, Deck }`
  (per-panel cursors so switching back remembers position). Add `origin: BuilderOrigin` (§3)
  via `DeckBuilderState::new(origin)`.
- **Rows from the profile, split by location:** collection rows = `collection_by_type()`
  filtered to `owned - in_deck > 0`, shown with count `available`; deck rows =
  `collection_by_type()` filtered to `in_deck > 0`, shown with count `in_deck`. Both already
  in canonical order.
- **Input:** `Tab`/`BackTab` switch `active`; arrows + `wasd` move the active panel's cursor
  over its grid (wrapping, reusing today's ragged-grid movement math); `Enter`/`Space` →
  **move across**: `Add(card)` when `active==Collection`, `Remove(card)` when `active==Deck`
  (existing `BuildOutcome` variants — the app already maps them to
  `try_add_to_deck`/`remove_from_deck`; no change to those app arms); `Esc`/`x` → `Back`. Keep
  the cursor in range and update `collection_scroll` to keep the cursor visible. Retire
  `Backspace`/`-` (direction is now the panel). Ignore an add/remove when the active side is
  empty.
- **Draw:** two bordered, labeled panels via `BriefcaseLayout`; each visible card a `CardView`
  with **Heavy border = cursor (pulse), Single otherwise** — the **Double weight is gone** (a
  card's panel conveys in-deck now); a `×N` count caption under each card; the `Deck: N/10`
  readout (Alert while `!deck_is_valid()`, as today) owned by the deck side; the controls hint
  (`Tab switch · Enter move · Esc done`, final wording during implementation). Empty-panel cue
  when a side has no rows.

### 3. Origin tracking + return routing (foundational — touches the shared `Back` arm)

- Add `enum BuilderOrigin { Menu, Map }` (`deck_builder.rs`, `Copy`), stored on
  `DeckBuilderState`, exposed via `origin()`.
- `open_deck_builder(&mut self, origin: BuilderOrigin)` (`app.rs:385`) threads it into
  `DeckBuilderState::new(origin)`.
- `BuildOutcome::Back` arm (`app.rs:695`) branches on `state.origin()`:
  `Menu => self.screen = self.start_menu()`, `Map => self.open_campaign_map()`.
- Update the call sites: menu `SideDeck` (`app.rs:957`) → `Menu`; opponent-select divert
  (`app.rs:374`) → `Menu`; **campaign-launch divert (`app.rs:445`) → `Map`** (fixes the latent
  bug — call it out in the task report); new map entry (§4) → `Map`.

### 4. `src/campaign_map.rs` + app arm — the map entry point

- `MapOutcome::OpenDeckBuilder` (`campaign_map.rs:32`); `KeyCode::Char('c') =>
  Some(MapOutcome::OpenDeckBuilder)` (`campaign_map.rs:135` area — **`c`, since `d` is taken by
  wasd movement**); extend the hint at `campaign_map.rs:268` to
  `"↑/↓ move · Enter play · b shop · c deck · Esc menu"`.
- App CampaignMap arm (`app.rs:722` area): `Some(MapOutcome::OpenDeckBuilder) => { play
  MenuSelect; self.open_deck_builder(BuilderOrigin::Map); }`.

## Revision — the fixed-album redesign (from the first visual review)

The product owner reviewed the T001–T004 build and changed the presentation: the
panels were oversized/empty, and every card type should have a fixed slot with a
**placeholder** for ones absent from that panel. This supersedes the scrolling
resolution above and *simplifies* the screen (scrolling is removed). §3 (origin) and
§4 (map) are unaffected.

**New model — a fixed card album.** Both panels show **all** `ALL_SIDE_CARDS` types
(15) in fixed canonical-order slots. Per panel, a type present (Collection:
`available = owned − in_deck > 0`; Deck: `in_deck > 0`) renders as a solid `CardView`
with a `×count` caption; a type absent renders as a **placeholder** — a faint
(`Emphasis::Muted`) **dashed**-border `CardView` showing the card's dimmed face, no
count.

**T006 — `BriefcaseLayout` becomes a fixed content-sized grid (`src/layout.rs`).**
- Drop `visible_rows`/scrolling. The grid is a fixed **4 cols × 4 rows** (16 slots, 15
  used) per panel; `card_origin(panel, index)` for `index in 0..15`.
- Cell pitch **`CELL_H = CARD_HEIGHT + 1 = 6`** (card 5 + a caption row, no inter-row
  gap) so 4 rows fit: `4 × 6 = 24` card rows + chrome (title, readout, panel
  border+label, hint ≈ 5–7) ≤ **31**. `CELL_W` unchanged (board pitch 10 → 4 cols in
  ~39, within the ~41 interior).
- Panel `Rect`s **hug** the 4×4 grid (≈41 wide, ≈26 tall) and center within the
  terminal (no half-screen sprawl). Revise the fit test to the fixed grid: `cols == 4`,
  a fixed `rows == 4`, both panels within frame and non-overlapping, every slot
  `0..15` contained and clear of the hint, at 89×31.

**T007 — `BorderWeight::Dashed` + album redraw + remove scroll (`src/frame.rs`,
`src/deck_builder.rs`).**
- `src/frame.rs`: add `BorderWeight::Dashed` with dashed box-drawing glyphs (corners
  `┌┐└┘`, horizontals `╌`, verticals `╎`) — a fourth weight alongside
  Single/Heavy/Double. (`CardView` already carries a `weight`, so a placeholder is a
  `CardView { weight: Dashed, emphasis: Muted, text: face }` drawn without a count.)
- `src/deck_builder.rs`: draw iterates `ALL_SIDE_CARDS` (15), computing each panel's
  count; filled slot → solid card + `×count` (Heavy+pulse if cursored, else Single);
  absent → dashed-faint placeholder + dimmed face. **Remove** `collection_scroll`,
  `MIN_VISIBLE_ROWS`, `scroll_to_reveal`, and their tests + the guard test (no scroll
  now). Cursor is per-panel over the fixed 15-slot grid (ragged last-row skip for the
  empty 16th slot); Enter on a placeholder is a no-op. The empty-panel focus
  special-case goes away (panels always render 15 slots).
- Input, `BuildOutcome`, `origin`, and the `handle_input`/`draw` signatures are
  otherwise unchanged, so §3/§4 and the app arms still line up.
- New draw tests: given a profile where a type is present in one panel and absent in
  the other, that slot is filled (with the right count) on one side and a placeholder
  on the other; a type owned 0 is a placeholder in both.

**T008 — placeholders are not navigable (`src/deck_builder.rs`, second visual review).**
The selection cursor only lands on **present** slots (Collection `available>0` / Deck
`in_deck>0`); arrow/`wasd` movement skips placeholders, landing on the next present slot
in that direction (wrapping; stays put if the panel has a single present card). Initial
focus and `Tab` target a panel that has present cards — a panel with none isn't
focusable, and an action that empties the active panel moves focus to the other (if it
has present cards). `draw` highlights a present card only via a display-cursor fallback
(the cursored slot if present, else the first present slot), so no placeholder is ever
cursored — even the transient frame after an add empties a slot. Enter still can't act on
a placeholder (the cursor never rests on one; the defensive no-op stays). Re-introduces a
minimal `first_present(panel)` focus seam that the album's always-15-slots draw dropped.

## Files

- `src/layout.rs` — `BriefcaseLayout`: first a scrolling grid (T001), then the fixed
  content-sized album grid (T006) + revised fit test.
- `src/frame.rs` — `BorderWeight::Dashed` for placeholders (T007).
- `src/deck_builder.rs` — the two-panel screen: reshape (T002), origin (T003), then the
  album redraw + scroll removal (T007); `Panel` + `BuilderOrigin`; tests.
- `src/app.rs` — `open_deck_builder(origin)`, `Back` routing, four call sites, the map arm.
- `src/campaign_map.rs` — `MapOutcome::OpenDeckBuilder`, `c` key, hint.
- **No change:** `profile.rs`, `card.rs`/`CardView` (placeholders reuse `CardView` with
  the new weight), `game.rs`, `screen.rs`, save format.

## Tests

- **Layout (foundational):** `briefcase_fits_the_minimum_terminal` at `Config(89,31)` —
  both panels within `num_cols` and non-overlapping, ordered chrome on-frame, cards
  contained and clearing the hint. **Revised by T006** to the fixed album grid: assert
  `cols == 4`, `rows == 4`, and every slot `0..15` contained (was `visible_rows >= 3`).
- **Screen input:** panel switch changes `active`; `Enter` in Collection yields `Add(cursored)`,
  in Deck yields `Remove(cursored)`; cursor movement + wrap; scroll clamps and keeps the cursor
  visible; `Esc`/`x` → `Back`; unknown keys ignored (follow the existing `deck_builder.rs` test
  style).
- **Origin routing:** a `DeckBuilderState::new(Map)` reports `Map`; the `Back` branch selects
  the map vs the menu (test at whatever seam is reachable without a terminal — the `origin()`
  accessor plus the app-arm branch).
- **Map entry:** `c` yields `MapOutcome::OpenDeckBuilder`; the hint string contains `deck`.

## Verification

- `cargo build` (no new warnings) + `cargo test` (reported verbatim).
- **Driver** (back up + checksum-restore the real profile first, per the standing data-safety
  practice; stage a profile with duplicates so both panels are populated): menu → Side Deck
  shows Collection | Deck; Enter moves a card across and the counts/readout update; from the
  campaign map, `c` opens the builder and Esc returns **to the map**; snapshots at 89×31 and
  ~120 wide.

## Non-goals (from spec)

No buying/unowned cards in the builder (shop's job); no deck reordering (order is cosmetic);
no NG+/persistence change; not restyling the shop or the map (only the `c` launch key is
added); deck size and add/remove rules unchanged.
