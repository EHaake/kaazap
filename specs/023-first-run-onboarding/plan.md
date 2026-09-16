# Plan: First-run onboarding & controls refinement — spec 023

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-13)
**Implements**: `spec.md` in this directory

## Context

Two once-per-profile texts — a **primer** over the galaxy map and a
**first-match popup** over the board — each dismissed with one key, each
recorded as a serde-defaulted boolean on the profile that survives resets;
one line on opponent select; a campaign section and a new controls line in
How to Play; and the **controls refinement** the popup describes: **1–4
select** a hand card, **Enter / P play** it, **Space draws** on the player's
turn, and the ± sign prompt is gone (rulings F, G, H).

It is a **UI + input spec with two profile booleans**. The engine's rules
(card effects, scoring, resolution), the AI, the economy, the wager prompt,
settlement, `save.rs`, and both on-disk formats' versions are untouched. No
new crate. `game.rs` changes only in its **key map** (`game_action_from_key`)
and gains one timer helper; `tests/balance.rs` is not edited.

## What the code already gives us

- **One modal at a time, routed before any screen.** `Modal` (`app.rs:257`)
  is `Option<Modal>` on `App`; `handle_key` (`app.rs:797`) walks an
  `else if matches!(self.modal, …)` chain and only reaches the screen arms
  when no modal is open. `Modal::RunOver` is the unit-like pattern: a pure
  `run_over_acknowledged(key)` (`app.rs:326`), a `handle_run_over_input`
  (`app.rs:554`), a draw arm (`app.rs:1457`). Both new pieces copy it.
- **The text-overlay box is a free function.** `draw_text_overlay(config,
  &[String], frame)` (`overlay.rs:68`) measures the lines, builds an
  `OverlayLayout`, clears, boxes, centers line 0 as the title and draws the
  rest left-aligned, `Emphasis::Normal` throughout (monochrome). `Overlay`
  wraps it with an `include_str!` per `OverlayKind` (`overlay.rs:31-46`).
  `OverlayLayout::new` insets `inner` by `V_PAD / 2 = 2` rows top and bottom
  (the border row plus exactly one blank row above and below the content)
  (`layout.rs:679-721`, `lib.rs:39-43`) — the even box padding the design
  brief asks for, by construction.
- **Every campaign entry from the menu funnels through two call sites.**
  `enter_campaign_continue` (`app.rs:514`, the no-progress path and
  `CampaignEntry`'s Continue) and `handle_confirm_input`'s
  `PendingStart::Campaign` arm (`app.rs:1198`, the discard-and-enter) both
  call `enter_campaign_map` (`app.rs:540`), which opens the map and raises
  `RunOver` if broke. The **game-over acknowledgement** (`app.rs:938`) calls
  the same fn; Back from the shop / builder and `start_new_campaign` call
  `open_campaign_map` directly. **New Campaign is menu-only**: `MapOutcome`
  has no such variant, `Modal::ConfirmNewCampaign` is raised only from the
  menu's `CampaignEntry` choice, and `start_new_campaign` (`app.rs:580`) has
  one caller — that confirm's Commit — so its doc's "the map's own New
  Campaign panel" is stale wording, not a path.
- **One match-start seam.** `start_match` (`app.rs:661`) is the only place a
  match begins — Quick Play (`SelectOutcome::Picked`) and the wager Commit
  both land there — and the Continue path (`app.rs:1286`) is the only place a
  match resumes. "A start, not a resume" is therefore a hook in one fn.
- **A match always opens on the player's turn.** `with_opponent` sets
  `GamePhase::PlayerTurn` (`game.rs:107`) and so does `setup_next_round`.
  `OpponentThinking { until }` is only ever entered from a player action or
  the opponent's own recovery (`game.rs:381, 421, 691, 715`), and only
  `GameState::update()` — called from `App::tick` (`app.rs:1361`) — advances
  it. Holding the match is therefore "don't call `update()`".
- **The sign-choice pass-through.** `PlayHand { index }` on a `±N` /
  tiebreaker moves to `AwaitingSignChoice { hand_index }`; `ChooseSign`
  commits; `CancelSignChoice` returns to `PlayerTurn` with the card in hand
  (`game.rs:181-200, 732-791`). `cursor_confirm` (`app.rs:201`) issues both
  actions in one key event, so the phase is never observed between events
  on the cursor path. The **direct-key path** — `'1'..'4'` → `PlayHand` and
  the `AwaitingSignChoice` branch mapping `h/l/+/-/1/2/c`
  (`game.rs:127-139`) — plus the board's `"+3 (h) or -3 (l)? (c cancels)"`
  prompt (`board.rs:313-323`) is the only way a player reaches it today.
- **Nine engine tests and the simulator drive the pass-through.** `game.rs`
  `sign_*` tests (`2039-2206`), the two headless loops (`1178, 1220`), and
  `tests/balance.rs:190-197` all do `PlayHand` then `ChooseSign` through
  `AwaitingSignChoice`. `save.rs` round-trips the phase (`SavedPhase::AwaitingSignChoice`,
  `save.rs:70, 82, 107`).
- **Space today.** The InGame arm catches `Enter | ' '` on the player's turn
  for `cursor_confirm` (`app.rs:975`); off it, `' '` reaches
  `game_action_from_key`, which maps it to `NextRound` / `NextGame` at the
  pauses and `None` otherwise (`game.rs:146-150`). The campaign game-over
  acknowledgement (Enter/Space/g/x/Esc, `app.rs:924`) runs **before** the
  InGame arm and stays as is.
- **Profile fields are additive by discipline.** Every `Profile` field is
  `#[serde(default)]`-ed (`profile.rs:42-68`); `reset_to_starter`
  (`profile.rs:154`) is `take(stats); *self = default(); restore` — the
  exact shape the two marks join. `PROFILE_VERSION = 1`.
- **Opponent select's footer.** Blurb at `y + 2`, hint at `y + 4`, with a
  footer reserve of 6 passed to `MenuLayout` (`opponent_select.rs:105-127`);
  `the_full_roster_and_footer_fit_the_minimum_terminal` pins `hint_y < 31`.
- **The status band is 81 columns wide** at any terminal (`BOARD_WIDTH = 89`
  less `2·H_PAD + 1`, `layout.rs:120-125`) — the ceiling for the turn hint.
- **Tests never touch disk.** No test in the crate calls `Profile::save` or
  `save::save`; `App::handle_key` saves the game on any `game_changed` key
  and the profile on every mark/settle. App-level tests must therefore stay
  on paths that write nothing (design tension §4).

## Design tensions resolved

### 1. The engine keeps the sign-choice pass-through; only the player-facing prompt goes

**Kept**: `GameAction::{PlayHand, ChooseSign, CancelSignChoice}`,
`GamePhase::AwaitingSignChoice`, `play_card`, `commit_sign_choice`, and every
`sign_*` engine test, unchanged. **Removed**: the `AwaitingSignChoice` branch
of `game_action_from_key` (h/l/+/−/1/2/c), the `'1'..'4' → PlayHand` arm, and
the board's sign prompt. Reasons: (a) the spec requires the engine's card
tests unchanged and `save.rs` untouched — `SavedPhase::AwaitingSignChoice`
exists on disk, so removing the phase would force a `save.rs` edit; (b)
`tests/balance.rs` and the two headless loops are written against it; (c)
after this spec the only producer of `PlayHand` in the binary is
`cursor_confirm`, which answers the phase in the same key event, so it is
unobservable — no key maps to anything in it, `update()` leaves it alone, and
`status_message` returns `None` for it. The cost is one transient phase the
player can never see; the alternative (a signed `PlayHand`) rewrites
`cursor_confirm`, nine tests, the simulator, and the save format for no
visible gain.

### 2. Holding the match under the popup: skip `update()`, and re-arm the pause on dismissal

`App::tick` skips `game_state.update()` while `Modal::FirstMatch` is open; the
modal chain already swallows every key. That holds any phase, including an
`OpponentThinking { until }` whose deadline passes under the popup. Today a
match opens in `PlayerTurn` (facts above), so the opponent-acts-first case
cannot arise at a match start; to make the spec's "the usual thinking pause
runs from then" true by construction rather than by that accident, dismissal
calls a new `GameState::restart_opponent_pause()` — re-arms `until` from now
if the phase is `OpponentThinking`, no-op otherwise. Five lines, in
`game.rs` because the constitution routes state mutation through the engine,
not a `pub` field write from `app.rs`.

### 3. Where the primer is raised: the three menu-entry sites, not the game-over path

`enter_campaign_map` gains a `from_menu: bool` and sets `self.modal =
map_entry_modal(broke, primer_due)` where `primer_due = from_menu &&
!profile.primer_seen()`. The three menu-entry callers pass `true`:
`enter_campaign_continue` (the no-progress path and `CampaignEntry`'s
Continue), the `PendingStart::Campaign` confirm arm, and `start_new_campaign`
— which switches from `open_campaign_map()` to `enter_campaign_map(true)`
(the reset just left the seed purse, so it is never broke and resolves to the
primer when unseen, else `None`). The game-over acknowledgement passes
`false` (spec: not from a match's game-over acknowledgement — reachable with
the primer unseen only by a pre-023 profile resuming a saved campaign match).
`map_entry_modal` is the pure precedence seam (run-over first, then primer,
else `None`). Only the shop / builder Backs keep `open_campaign_map` and
never raise it — they return to a map already seen. Because New Campaign is
reachable only from the menu (facts above), no origin flag beyond `from_menu`
is needed: a pre-023 profile with progress whose first act is New Campaign
sees the primer on that map, once.

### 4. Testability without an `App` that writes to disk

App-level tests read the real profile (`App::new` → `Profile::load`, as the
existing resize test does) but must not save. So: (a) every *decision* is a
pure fn in the `confirm_choice` spirit — `turn_key`, `onboarding_dismissed`,
`map_entry_modal`, `HandCursor::select` — each unit-tested; (b) the two
App-level tests (the popup holds the match; the primer swallows map keys)
press only **non-dismiss** keys and tick with no phase change, so
`save_game` / `profile.save` are never reached; (c) the dismissal handler
itself (mark + save + close + sfx + re-arm) is five lines read at review,
with the re-arm tested in `game.rs` and the mark round-trip in `profile.rs`.

### 5. A pre-023 mid-match save can be sitting in `AwaitingSignChoice`

A `1` on a ± card used to save the game in that phase (`save_game` on every
`game_changed`). After this spec no key maps to `ChooseSign`, so such a save
would resume soft-locked. Continue therefore applies
`GameAction::CancelSignChoice` right after `save::load()` — a no-op in every
other phase, and in that one it returns the card to hand at `PlayerTurn`
(the engine's existing cancel semantics, `sign_cancel_restores_turn_with_card_unspent`).
`save.rs` is untouched; `CancelSignChoice` stays for exactly this.

### 6. The pieces are unit-like modals drawn through `draw_text_overlay`, with their text in `overlay.rs`

Not `Modal::Help(Overlay)`: Help dismisses on `?` too, plays `MenuBack`, and
caches a `Config` that `resize` must rebuild. `Modal::Primer` /
`Modal::FirstMatch` carry no data; `draw` calls `draw_text_overlay(self.config,
&overlay_text(kind), frame)` (the `PlayLog` pattern — rebuilt every frame, no
resize arm). `overlay_text(kind) -> Vec<String>` is the existing
`read_text_from_file` body made a free fn with two new `OverlayKind` variants
and two new asset files, so every static text stays in one place.
Emphasis is `Normal` throughout like How to Play (the "same bordered box the
other text overlays use"); the dismiss line's breathing room comes from the
asset text (a blank line above it) plus the box's own bottom padding row.

### 7. The board's turn hint names all five keys in one row

Three strings, each ≤ 64 chars against an 81-column band, pinned by a test:

- empty hand: `Space draw · S stand`
- fixed / flip card: `1-4/←/→ pick · Enter/P play <label> · Space draw · S stand`
- ± / tiebreaker: `1-4/←/→ pick · ↑/↓ flip · Enter/P play <±N> · Space draw · S stand`

The over-20 alert becomes `OVER 20!  (Space/D/S: bust)`. How to Play's "or
press d/s to bust" stays (still true; not in the spec's change list).

## Design

### 1. `src/profile.rs` — the seen marks

```rust
/// Onboarding seen-marks (spec 023): set when the primer / first-match popup
/// is *dismissed*, never when shown. Additive and serde-defaulted false — no
/// version bump — and, like `stats`, they survive `reset_to_starter`.
#[serde(default)] primer_seen: bool,
#[serde(default)] first_match_seen: bool,

pub fn primer_seen(&self) -> bool
pub fn mark_primer_seen(&mut self)        // callers pair with `save`, as with deck edits
pub fn first_match_seen(&self) -> bool
pub fn mark_first_match_seen(&mut self)
```

`reset_to_starter` takes the two bools alongside `stats` and restores them
after the `Profile::default()` swap. `Default` leaves both false.

### 2. `src/game.rs` — key map and the pause helper

`game_action_from_key`: delete the `AwaitingSignChoice` branch and the
`'1' | '2' | '3' | '4'` arm; Space becomes

```rust
' ' => match self.game_phase {
    GamePhase::PlayerTurn => Some(GameAction::Hit),        // spec 023: Space draws; over 20 the Hit arm already accepts the bust like D
    GamePhase::AwaitingNextRound => Some(GameAction::NextRound),
    GamePhase::GameOver { .. } => Some(GameAction::NextGame),
    _ => None,
},
```

`d`, `s`, `n`, `g` unchanged on the player's turn; the fn opens with an early `return None` for `AwaitingSignChoice`, so no key — `d` and `s` included — maps to anything in that phase (tension §1; reconciled with §Tests bullet 3 at T002). New:

```rust
/// Re-arm the opponent's thinking pause from now, if one is running (spec 023:
/// the first-match popup holds the match; on dismissal the usual pause runs
/// from then). No-op in every other phase.
pub fn restart_opponent_pause(&mut self)
```

Everything else in `game.rs` — `apply_game_action`, `play_card`, the AI, the
sign tests, the headless loops — is untouched.

### 3. `src/app.rs` — the turn-key table, cursor select, the two modals

```rust
/// What a key does in a match — the decision table behind the InGame arm,
/// pulled out so the spec-023 bindings are unit-testable without an `App`
/// (the `confirm_choice` pattern). On the player's turn: 1–4 select, ←/→ move,
/// ↑/↓ flip, Enter / P play; every other char (Space, d, s, n, g…) goes to the
/// engine's key map, which validates by phase. Esc / x always leave.
#[derive(Debug, PartialEq, Eq)]
enum TurnKey { Menu, MoveLeft, MoveRight, FlipSign, Select(usize), Play, Engine(char), Ignore }
fn turn_key(key: KeyCode, player_turn: bool) -> TurnKey

impl HandCursor {
    /// Select `index` if that slot is occupied (1–4, spec 023) — then does
    /// exactly what `move_left` / `move_right` do after landing: sets the index
    /// and resets the pending sign to positive (selecting the already-selected
    /// slot resets it too, as an arrow that lands on the same slot does). An
    /// empty or out-of-range slot changes nothing, sign included.
    pub fn select(&mut self, index: usize, hand: &[Option<Card>])
}

/// Enter / Space / Esc dismiss the primer and the first-match popup; every
/// other key is ignored (the `run_over_acknowledged` pattern).
fn onboarding_dismissed(key: KeyCode) -> bool

/// What a freshly opened campaign map raises (spec 021 + 023): the run-over
/// notice if broke, else the primer if it is due, else nothing. One modal.
fn map_entry_modal(broke: bool, primer_due: bool) -> Option<Modal>
```

`Modal` gains two unit-like variants, `Primer` and `FirstMatch`, doc-commented
like `RunOver`. Wiring:

- **InGame arm** (`app.rs:959-988`) becomes `match turn_key(key, player_turn)`:
  `Menu` → `start_menu()`; `MoveLeft/MoveRight/FlipSign/Select(i)` → the cursor
  (no save); `Play` → `cursor_confirm` + `game_changed`; `Engine(c)` → the
  existing `handle_game_input` block; `Ignore` → nothing. The `?`, `L`, and
  campaign game-over acknowledgement blocks above it are unchanged.
- **`handle_key` modal chain**: one new branch,
  `Some(Modal::Primer | Modal::FirstMatch)` → `handle_onboarding_input(key)`:
  if `onboarding_dismissed(key)` — mark the matching profile flag (for
  `FirstMatch` also `game_state.restart_opponent_pause()` when in a match),
  `profile.save()`, `modal = None`, `Sfx::MenuSelect`; otherwise nothing.
- **`enter_campaign_map(from_menu: bool)`**: `open_campaign_map()` then
  `self.modal = map_entry_modal(self.profile.is_broke(), from_menu &&
  !self.profile.primer_seen())`. Four call sites: `enter_campaign_continue`
  → `true`, the `PendingStart::Campaign` arm → `true`, `start_new_campaign`
  → `enter_campaign_map(true)` (replacing its `open_campaign_map()`; its doc
  drops "the map's own New Campaign panel"), the game-over acknowledgement
  → `false`.
- **`start_match`**: after the screen is set (before `save_game()`), `if
  !self.profile.first_match_seen() { self.modal = Some(Modal::FirstMatch) }`.
  The save still happens — quitting under the popup leaves a resumable match
  and an unset mark; Continue never raises the popup. A Quick Play `G` /
  Space at game over restarts inside the engine (`GameAction::NextGame`,
  `new_game`), never through `start_match`, so it never shows the popup
  either — the next `start_match` does. Consistent with "a resume is not a
  start".
- **Continue** (`activate_menu_item`): after `save::load()` returns `game`,
  `game.apply_game_action(GameAction::CancelSignChoice)` (tension §5), then
  the existing cursor normalize.
- **`tick`**: `let held = matches!(self.modal, Some(Modal::FirstMatch));` and
  the `update()` block runs only when `!held`. Audio/banter/log observers
  still run (nothing changed → nothing fires).
- **`draw`**: `Some(Modal::Primer) => draw_text_overlay(self.config,
  &overlay_text(OverlayKind::Primer), frame)`, likewise `FirstMatch`.

### 4. `src/overlay.rs` + `assets/` — the texts

`OverlayKind` gains `Primer` and `FirstMatch`; `read_text_from_file` becomes
`pub fn overlay_text(kind: OverlayKind) -> Vec<String>` (five `include_str!`
arms; `Overlay::draw` calls it). New files, the spec's text **verbatim**:
`assets/primer_text.txt` (10 lines, title `=====  Your first campaign  =====`,
last line `             Enter to continue`, one blank line above it) and
`assets/first_match_text.txt` (12 lines, title `=====  How a match works  =====`,
last line `             Enter to begin`). Boxes: 10 + 4 = 14 rows × 52 + 8 = 60
cols and 16 × 61 — inside 139×31 unclamped.

`assets/game_overlay_text.txt` — the five control rows become:

```
  1 2 3 4 / ← →  Select a hand card
  ↑ / ↓          Flip a ± card's sign
  Enter / P      Play the selected card
  Space / D      Draw a card (ends turn)
  S              Stand (ends turn)
```

and the last line `  Over 20: Space, D or S accepts the bust.`; `N / Space`,
`G / Space`, Esc/X, Q, `?`, the Ctrl line stay.

`assets/how_to_play_text.txt` — line 18 (`Play with 1-4 or arrows +
Enter/Space. ? closes.`) is replaced by the campaign section and the new
controls line, both from the spec's proposed text, in that order at the end
of the file. The file separates paragraphs with one blank line, so the new
tail is: lines 1–17 as they are (ending on the blank line 17), the four
`Campaign:` lines, one blank line, the two controls lines ending `? closes.`
— 17 + 4 + 1 + 2 = 24 lines, a 24 + `V_PAD` = 28-row box, inside 31.

### 5. `src/board.rs` — hints

`status_message`: delete the `AwaitingSignChoice` branch (the phase falls to
`_ => None`). `play_prompt_line` and `over_twenty_alert`: the strings in
tension §7. Doc comments drop "number-key path" / "h/l".

### 6. `src/opponent_select.rs` — the Quick Play line

`const QUICK_PLAY_NOTE: &str = "Quick Play deals the standard deck.";` drawn
`Muted` at `y + 4`; the hint moves to `y + 5`; the `MenuLayout` footer reserve
becomes 7. Compact — neither row is acted on.

### 7. Close-out text (repo-wide files, applied on `main` after the merge)

`specs/023-first-run-onboarding/closeout-main-docs.md` (021/022's shape):
**ROADMAP** — the onboarding item shipped; lines 56–59 (spec 006's "Space
plays the selected card … Space no longer draws") superseded by spec 023.
**DECISIONS** — rulings A–H, tension §1 (pass-through kept), §3 (the primer
is raised at every menu entry to the map — Start Campaign, Continue, the
discard-and-enter confirm, and a confirmed New Campaign, which is menu-only;
never from the game-over acknowledgement or a shop / builder Back), §5
(Continue cancels a mid-prompt save); lines 40–41's "Space isn't [a bust
key]" superseded. `README.md` and `docs/` name no
in-match keys (grepped) — no change.

## Files

- `src/profile.rs` — two fields, four accessors, `reset_to_starter`; tests.
- `src/game.rs` — `game_action_from_key` (Space → Hit on `PlayerTurn`; drop
  1–4 and the sign-phase branch); `restart_opponent_pause`; key-map tests
  rewritten, sign-action tests untouched.
- `src/app.rs` — `TurnKey` / `turn_key`, `HandCursor::select`,
  `onboarding_dismissed`, `map_entry_modal`, `Modal::{Primer, FirstMatch}`,
  `handle_onboarding_input`, `enter_campaign_map(from_menu)`, `start_match`
  hook, Continue cancel, `tick` hold, two draw arms; tests.
- `src/overlay.rs` — `OverlayKind::{Primer, FirstMatch}`, `overlay_text`;
  tests (fit, density, wording).
- `src/board.rs` — `status_message`, `play_prompt_line`, `over_twenty_alert`;
  tests.
- `src/opponent_select.rs` — `QUICK_PLAY_NOTE`, draw, reserve; tests.
- `assets/primer_text.txt`, `assets/first_match_text.txt` (new);
  `assets/game_overlay_text.txt`, `assets/how_to_play_text.txt`.
- `specs/023-first-run-onboarding/closeout-main-docs.md` (T009).
- **No change**: `save.rs`, `player.rs`, `card.rs`, `economy.rs`,
  `wager.rs`, `campaign.rs`, `campaign_map.rs`, `shop.rs`, `deck_builder.rs`,
  `menu.rs`, `layout.rs`, `frame.rs`, `render.rs`, `audio.rs`, `banter.rs`,
  `play_log.rs`, `stats.rs`, `records.rs`, `opponent.rs`, `tests/balance.rs`,
  `docs/*`, `README.md`, `Cargo.toml`, `Cargo.lock`.

## Tests

Each claim names the task that owns its check. Driver items are marked.

- **The marks default unset, round-trip, load unset from an older document,
  and survive the reset** (T001): `Profile::default()` and
  `{"version":1,"collection":[],"deck":[]}` → both false; mark both → JSON →
  `from_json` → both true; `reset_to_starter` on a dirtied profile with both
  marked keeps both; `PROFILE_VERSION == 1`.
- **Space draws on the player's turn and still advances at the pauses**
  (T002): `game_action_from_key(' ')` → `Hit` in `PlayerTurn`, `NextRound` in
  `AwaitingNextRound`, `NextGame` in `GameOver`, `None` in `OpponentThinking`
  and `AwaitingSignChoice`; over 20, the action from `' '` and from `'d'`
  leave the same stood/bust state.
- **No key plays or answers a sign** (T002): on `PlayerTurn` `'1'..'4'`,
  `'h'`, `'l'`, `'+'`, `'-'`, `'c'` → `None`; in `AwaitingSignChoice` every
  key → `None`; `CancelSignChoice` outside the phase is a no-op.
- **The pause re-arms only when running** (T002): `OpponentThinking { until:
  now }` → after the call `until > now`; `PlayerTurn` → unchanged.
- **The turn-key table** (T003): on the player's turn `'1'..'4'` →
  `Select(0..3)`, `Enter` and `'p'` → `Play`, `' '` / `'d'` / `'s'` →
  `Engine`, arrows → move/flip; off it `Enter` → `Ignore`, `' '` → `Engine`;
  `Esc` / `'x'` → `Menu` either way.
- **Select lands only on an occupied slot and mirrors a move** (T003): on
  `[±3, −4, ∅, ∅]` flip the sign, select slot 2 → index 1 and the sign reset
  to positive, exactly the state `move_right` leaves from index 0; flip again
  at index 0 and select slot 1 (the current slot) → sign reset, like an
  arrow landing on the same slot; select slot 3 (empty) and slot 9 → index
  and sign both unchanged.
- **Enter/P play each kind at the shown sign** — existing `cursor_confirm_*`
  tests (fixed, ±, tiebreaker, flip) stand unchanged; the key mapping is the
  table test above.
- **The hints name the keys and fit** (T004): exact strings for empty /
  fixed / ± / flip; each ≤ `BoardLayout::new(min).status.width()`;
  `status_message` in `AwaitingSignChoice` is `None`; the alert reads
  `OVER 20!  (Space/D/S: bust)`.
- **The help texts say the new keys and nothing old** (T005): game overlay
  has rows `Space / D`+`Draw`, `Enter / P`+`Play`, `1 2 3 4`+`Select`, no
  `Enter / Space`, no `1 2 3 4` row containing `Play`; How to Play contains
  `Campaign:`, `Space draws`, `Enter plays it`, no `Enter/Space`; both fit
  139×31 unclamped.
- **The onboarding texts are verbatim, fit, and breathe only around the
  dismiss line** (T006): 10 / 12 lines; titles as spec'd; last line is the
  `Enter to …` line; the line above it is blank; no other two consecutive
  blank lines; `OverlayLayout` at 139×31 is unclamped for both.
- **Dismiss keys and precedence** (T007): `onboarding_dismissed` true for
  Enter/Space/Esc only; `map_entry_modal(true, true)` → `RunOver`, `(false,
  true)` → `Primer`, `(false, false)` → `None`, `(true, false)` → `RunOver`.
- **The popup holds the match and swallows play keys** (T007, App-level, no
  disk write): `OpponentThinking { until: now }` under `FirstMatch`, `tick`
  → still `OpponentThinking`; then `PlayerTurn` and keys `d`, `1`, `s`, `p`
  → hand, rows, phase, modal all unchanged.
- **The primer swallows map keys** (T007, App-level): `CampaignMap` under
  `Primer`, keys `b`, `c`, `Up`, `x` → still the map, modal still up.
- **Once per profile / never on resume / not after a reset / the run-over
  precedence in play** — *driver* (Phase 2 pause, profile backed up +
  checksum-restored): fresh profile → primer once → popup once at the first
  match; a second match, a run-over reset and a New Campaign show neither;
  with the mark still unset, quit with the popup up → Continue resumes with
  no popup → the next new match shows it; a broke pre-023 profile gets the
  notice, then the primer next time; a pre-023-shaped profile with progress
  whose first act is New Campaign sees the primer on that map.
- **Dismissal wiring** (T007 phase review, read): the handler marks, saves,
  closes, plays `MenuSelect`, re-arms; Continue applies `CancelSignChoice`.
- **The Quick Play line fits** (T008): note at `y + 4`, hint at `y + 5`,
  both `< 31` at the minimum; `QUICK_PLAY_NOTE == "Quick Play deals the
  standard deck."`.
- **No engine-rule / AI / economy / save change** (T009 sweep): `git diff
  main --stat` lists no `save.rs`, `player.rs`, `card.rs`, `economy.rs`,
  `wager.rs`, `tests/balance.rs`; the `game.rs` diff touches only
  `game_action_from_key`, the new helper, and `#[cfg(test)]`;
  `PROFILE_VERSION == 1`, `SAVE_VERSION == 1`; warning count equals `main`'s;
  `grep -rn "Space" assets src/board.rs` shows no "play" pairing.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim per the constitution.
- **Driver / person attestation** (back up + checksum-restore the real
  profile/saves first): Phase 1 pause — a match with the new keys (1–4
  select, Enter/P play, Space draws and busts over 20, no sign prompt; `?`
  and the turn hint read right); Phase 2 pause — a fresh profile sees the
  primer once and the popup once, the popup holds the board, an existing
  profile sees each once and keeps its cards/credits/records; snapshots at
  139×31.

## Non-goals (from spec)

No interactive tutorial; no way to reopen either piece; no other onboarding
surface; no rebindable keys; no other control change (emacs chords, Esc/X,
Q, N, G, S, D, L, arrows unchanged); no endgame award / run summary /
archive; no wording tuning here (later wording changes are chores).

## Open questions

None product-level that blocks drafting. Two edges are settled here as
design, flagged for the sign-off and the person:
1. **A pre-023 save mid-prompt resumes with the card back in hand** (tension
   §5) rather than committed at `+`.
2. **How to Play keeps "press d/s to bust"** (still true; Space also busts
   now). Adding Space there is a wording edit the person may make in T005.
