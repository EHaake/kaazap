# Plan: Collection & Side-Deck Customization

Technical design for `spec.md`. Subsystem B of the campaign epic. The master
plan (`~/.claude/plans/iterative-meandering-blum.md`) mirrors this.

## Shape of the change

Two new modules — a persistent `Profile` (collection + built deck) and a
deck-builder `Screen` — plus a one-field addition to `GameState` that lets a
match deal the player's hand from a supplied deck instead of the global
`DEFAULT_SIDE_DECK`. The engine's decision logic and phase machine are
untouched; only the *source of the player's deck* moves from a const to
profile-backed state, exactly as spec 007 moved the opponent's deck from a
const to `OpponentProfile`.

## Data model — new `src/profile.rs`

Modeled on `src/settings.rs` (self-owned serde struct; `load() -> Self` with
defaults; best-effort `save(&self)`) **plus** `src/save.rs`'s version
discipline. `Card` already derives `Serialize`/`Deserialize` (`card.rs:27`),
so collection and deck persist as plain `Vec<Card>` — no projection type.

```rust
const PROFILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "default_version")]    version: u32,
    #[serde(default = "starter_collection")] collection: Vec<Card>, // bag of owned copies
    #[serde(default = "starter_deck")]       deck: Vec<Card>,       // the built side deck
}
```

- **Starter data** (`impl Default` uses these):
  - `starter_deck()` = the 10 cards of `DEFAULT_SIDE_DECK` (deck valid at 10
    from first launch; equals today's behavior).
  - `starter_collection()` = those 10 + a few spare adjusters (proposed
    `+1, -1, ±2`, single copies). Tunable balance data; the deck is always a
    sub-multiset of the collection.
- **Persistence**:
  - `load() -> Self` — read file → `from_json` (version-checked) →
    `.unwrap_or_default()`. Missing / corrupt / wrong-version → starter.
  - `save(&self)` — `create_dir_all` + `to_string_pretty` + `write`, all
    best-effort (`let _ =`), like `settings.rs`.
  - `from_json(&str) -> Option<Self>` — parse `.ok()?` then
    `(p.version == PROFILE_VERSION).then_some(p)` (discard on mismatch).
  - `path()` — `ProjectDirs::from("", "", "kaazap").map(|d|
    d.data_dir().join("profile.json"))` (same triple as saves/settings; sits
    beside the `saves/` subfolder).
- **Mutation through methods** (validation centralized, per the architecture
  rule; callers save):
  - `try_add_to_deck(&mut self, card) -> bool` — adds one copy iff
    `deck.len() < 10` **and** the deck doesn't already hold every owned copy of
    `card`; returns whether it added.
  - `remove_from_deck(&mut self, card) -> bool` — removes one copy iff present.
  - `deck_is_valid(&self) -> bool` — `deck.len() == 10` and deck ⊆ collection.
  - `deck(&self) -> &[Card]` accessor for the match hand-off.
  - `collection_by_type(&self) -> Vec<CardEntry>` — grid source of truth: each
    distinct owned card in canonical order with `owned` and `in_deck` counts.
    Iterates `card::ALL_SIDE_CARDS`, filters to owned, counts via
    `.iter().filter(..).count()` (no `Hash` needed — collection is tiny).

## Engine changes

- **`src/card.rs`** — add `pub const ALL_SIDE_CARDS: [Card; 15]`, the canonical
  card universe in stable display order (`+1 +2 +3 +4 / -1 -2 -3 -4 / ±1 ±2 ±3
  ±6 / 2&4 3&6 / ±1T`). Single source of truth for the grid order; available to
  spec C. `deal_hand` unchanged (already takes `&[Card]`); `Card` derives
  unchanged.
- **`src/game.rs`** — `GameState` gains `pub player_deck: Vec<Card>` (mirrors
  `opponent_profile`).
  - `with_opponent(profile, player_deck: Vec<Card>)` — store the deck, deal the
    player hand from `&self.player_deck` at the `game.rs:77` seam.
  - `new()` → `with_opponent(DEFAULT_OPPONENT, DEFAULT_SIDE_DECK.to_vec())` —
    preserves current behavior; every existing test that calls `new()` keeps
    the default deck.
  - `new_game()` (post-`GameOver` rematch, `game.rs:647`) re-deals the player
    from `&self.player_deck` instead of `&DEFAULT_SIDE_DECK`.

## Deck-builder screen — new `src/deck_builder.rs`

A full mode navigated *to* = a `Screen`. Mirrors `opponent_select.rs` plumbing
(state + outcome enum + `draw(frame, config, …, pulse)` + one app.rs arm) over
a **2D grid** rather than a vertical list.

- **`src/screen.rs`** — add `DeckBuilder { state: DeckBuilderState }`.
- **`DeckBuilderState { cursor: usize }`** — index into `collection_by_type()`
  (≤ 15 cells, always one screen; no paging). Reads `&Profile`; owns only the
  cursor.
  - `handle_input(&mut self, key, profile: &Profile) -> Option<BuildOutcome>`,
    `enum BuildOutcome { Moved, Add(Card), Remove(Card), Back }`. Arrows /
    `wasd` move the 2D cursor (±1 horizontal, ±cols vertical, `rem_euclid`
    wrap); Enter/Space → `Add(card under cursor)`; Backspace/`-` → `Remove`;
    Esc/`x` → `Back`. Returns the *intent*; the router mutates the `Profile`
    (so validation stays in one place) and plays SFX.
  - `draw(&self, frame, config, profile: &Profile, pulse)` — title "Side Deck";
    a **"Deck: N/10"** readout (`Emphasis::Alert` when N≠10); the grid via
    `card_slot` (`layout.rs:157`) + `CardView` (`card.rs:131`) per cell with an
    `M/N` badge in the gap row beneath each card; a hint line
    ("↑/↓/←/→ move · Enter add · Backspace remove · Esc done"). Reused visual
    language: cursored cell = `BorderWeight::Heavy` + `pulse`; cells with ≥1 in
    deck = `Emphasis::Strong`; others `Normal`.
- **`src/layout.rs`** — a small centered-grid helper beside `card_slot`: from
  `Config` + cell count + cols, return the centered grid origin `Rect` (reuses
  `MenuLayout`'s centering math). Minimal.
- **`src/menu.rs`** — add `MenuItem::SideDeck`, list it in `items.extend([...])`
  (after `StartGame`), and add its `Display` label ("Side Deck").
- **`src/app.rs`**:
  - New field `profile: Profile`, loaded in `App::new` (`Profile::load()`,
    beside `Settings::load()`).
  - `apply_menu_event`: `MenuItem::SideDeck => self.open_deck_builder()`; add
    `open_deck_builder()` beside `open_opponent_select`.
  - The three forced `Screen` arms — input router, `?`-help (→ `None`, inert),
    draw (→ `state.draw(frame, &self.config, &self.profile, pulse)`).
    Input-router arm (template = `OpponentSelect`): on `Add(c)`/`Remove(c)` call
    `self.profile.try_add_to_deck(c)` / `remove_from_deck(c)`, play `MenuSelect`
    on success else `MenuBack`, then `self.profile.save()`; `Moved` →
    `MenuMove`; `Back` → `MenuBack` + `start_menu()`. Return the owned
    `BuildOutcome` before touching `self` (NLL, per `app.rs:462`).
  - `start_match(opponent)` → `GameState::with_opponent(opponent,
    self.profile.deck().to_vec())`.
  - **Exactly-10 guard**: `open_opponent_select` (and the `ConfirmNewGame`-Yes
    path) first checks `self.profile.deck_is_valid()`; if not,
    `open_deck_builder()` instead. The `Alert`-styled "Deck: N/10" readout is
    self-explanatory — no separate notice state. Only reachable by deliberately
    under-filling and leaving; resume is always valid (deck snapshotted).

## Save / resume — `src/save.rs`

`player_deck` becomes snapshotted match state so a resume deals from the deck
the match began with:

- `SavedGame` gains `#[serde(default)] player_deck: Vec<Card>`. **No
  `SAVE_VERSION` bump** (additive).
- `to_saved`: `player_deck: game.player_deck.clone()`.
- `from_saved`: `player_deck = if saved.player_deck.is_empty() {
  DEFAULT_SIDE_DECK.to_vec() } else { saved.player_deck }` — old saves (missing
  → empty) fall back to the default, mirroring the `opponent_id → DEFAULT_
  OPPONENT` tolerance (`save.rs:114`). New saves always carry a real 10-card
  deck (guarded at start).

## Testing

Per `CLAUDE.md`, new/changed logic ships with tests:

- **Profile** (`profile.rs`): default profile deck is valid (10, ⊆ collection);
  JSON round-trip preserves collection + deck; wrong-`version` discarded →
  default; missing/garbage → default; `try_add_to_deck` rejects at 10 and with
  no spare copy, succeeds otherwise; `remove_from_deck` removes exactly one;
  `collection_by_type` counts/order correct.
- **Engine seam** (`game.rs`): `with_opponent(profile, custom_deck)` deals the
  player hand from `custom_deck` (⊆ custom, disjoint from `DEFAULT_SIDE_DECK`);
  `new()` still deals default; `GameOver → new_game` re-deals player from
  `self.player_deck`.
- **Builder** (`deck_builder.rs`): 2D cursor moves/wrap; `handle_input` returns
  `Add`/`Remove`/`Back` given a `Profile`.
- **Save** (`save.rs`): round-trip preserves `player_deck`; a field-less save
  resolves to `DEFAULT_SIDE_DECK`.

## Files

New: `src/profile.rs`, `src/deck_builder.rs`.
Modified: `src/lib.rs` (mods), `src/card.rs` (`ALL_SIDE_CARDS`), `src/game.rs`
(`player_deck` + seam), `src/screen.rs`, `src/menu.rs`, `src/app.rs`,
`src/save.rs`, `src/layout.rs` (grid helper).
No change expected: `src/board.rs`, `src/opponent.rs`, `src/opponent_select.rs`,
`src/settings.rs`.
