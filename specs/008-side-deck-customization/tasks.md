# Tasks: Collection & Side-Deck Customization

Second spec of the campaign epic (subsystem B). Each task builds (`cargo
build`) and tests (`cargo test`) green before it's done, with actual output
reported. One commit per task referencing its ID; the draft PR (opened at
T001) tracks the diff. Persistence/engine work is reviewed **per task**; the
UI is reviewed **per phase**. Do not weaken a test to make it pass.

Branch `008-side-deck-customization` created with the spec/plan/tasks. Plan:
`~/.claude/plans/iterative-meandering-blum.md` (mirrors this spec's docs).

---

- [ ] **T001 — Player profile: collection, deck & persistence**
  New `src/profile.rs`: `Profile` (`version`, `collection: Vec<Card>`,
  `deck: Vec<Card>`; serde with `#[serde(default)]` fields + `PROFILE_VERSION`
  discard-on-mismatch), `starter_collection`/`starter_deck` (deck = today's
  `DEFAULT_SIDE_DECK`; collection = that + a few spares), `load()/save()/path()`
  modeled on `settings.rs` + `save.rs`, and the mutation/query methods
  `try_add_to_deck`, `remove_from_deck`, `deck_is_valid`, `deck`,
  `collection_by_type`. New `pub const ALL_SIDE_CARDS: [Card; 15]` in
  `src/card.rs`; `pub mod profile;` in `src/lib.rs`. Open the draft PR.
  *Verify: `cargo build` no new warnings; `cargo test` green — default deck
  valid (10, ⊆ collection); JSON round-trip; wrong-version discarded → default;
  missing/garbage → default; `try_add_to_deck` rejects at 10 and with no spare,
  succeeds otherwise; `remove_from_deck` removes one; `collection_by_type`
  counts/order.*

- [ ] **T002 — Engine seam: matches deal from the built deck**
  `src/game.rs`: `GameState` gains `player_deck: Vec<Card>`; `with_opponent`
  takes `player_deck: Vec<Card>` and deals the player hand from it; `new()` →
  `with_opponent(DEFAULT_OPPONENT, DEFAULT_SIDE_DECK.to_vec())`; `new_game`
  re-deals the player from `self.player_deck`. `src/app.rs`: `start_match`
  passes `self.profile.deck().to_vec()` (add the `profile` field + `Profile::
  load()` in `App::new`).
  *Verify: `cargo test` green — `with_opponent` deals player hand from a custom
  deck (⊆ custom, disjoint from default); `new()` unchanged default; `GameOver
  → new_game` re-deals from `player_deck`; every existing game test still
  passing.*

- [ ] **T003 — Deck-builder screen + Side Deck menu flow**
  New `src/deck_builder.rs` (`DeckBuilderState` over `collection_by_type()`,
  2D cursor with `wasd`/emacs, `handle_input → Add/Remove/Back`, `draw` — grid
  of `CardView`s with `M/N` badges, "Deck: N/10" readout, hint line); small
  centered-grid helper in `src/layout.rs`; `Screen::DeckBuilder` in
  `src/screen.rs`; `pub mod deck_builder;` in `src/lib.rs`. `src/menu.rs`:
  `MenuItem::SideDeck` + label. `src/app.rs`: `open_deck_builder`, the
  input/`?`/draw arms (mutate the profile through its methods + `save()`, menu
  SFX), and the exactly-10 guard routing an incomplete deck to the builder.
  *Verify: `cargo test` green with builder navigation tests; driver — menu
  shows Side Deck, grid renders with badges + "Deck: 10/10", add/remove update
  live, `Alert` readout when <10, Esc returns to menu.*

- [ ] **T004 — Save/resume carries the built deck**
  `SavedGame` gains `player_deck: Vec<Card>` (`#[serde(default)]`); `to_saved`
  writes `game.player_deck`; `from_saved` uses it, falling back to
  `DEFAULT_SIDE_DECK` when empty. No `SAVE_VERSION` bump.
  *Verify: `cargo test` green — round-trip preserves `player_deck`; a
  field-less (pre-spec) JSON resolves to `DEFAULT_SIDE_DECK`; driver — build a
  distinctive deck, start a match, quit, Continue resumes dealing from it.*

- [ ] **T005 — Verification & close-out**
  Full driver sweep of every acceptance box (build a deck, play from it,
  incomplete-deck guard, resume, no-regression with starter). Run the
  `skeptical-reviewer`. Update `Readme.md` (deck-building is new user-facing
  behavior), `ROADMAP.md` (Campaign epic — B shipped), and `DECISIONS.md`
  (record the three rulings: bag-of-copies, modest starter, exactly-10). On the
  human's word: mark the PR ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim; reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The
player's deck moves from the `DEFAULT_SIDE_DECK` const to a `player_deck`
carried on `GameState` (mirroring spec 007's `opponent_profile`) and sourced
from a new persistent `Profile` (`profile.json`), modeled on `settings.rs` +
`save.rs`'s versioning. The deck-builder is a new `Screen` mirroring
`opponent_select.rs` but over a 2D grid, reusing `CardView` + `card_slot`.
Matches snapshot their deck into the save, so editing your deck never rewrites
an in-progress match. Collection is a bag of copies (spec C grows it); starter
deck == the old default, so an untouched profile plays identically to before.
