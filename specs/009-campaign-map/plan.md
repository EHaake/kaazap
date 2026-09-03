# Plan: Campaign Map

Technical design for `spec.md`. Subsystem D of the campaign epic (navigation +
progression-structure layer; economy stubbed). The master plan
(`~/.claude/plans/iterative-meandering-blum.md`) mirrors this.

## Shape of the change

A new `const` map-graph data model (`campaign.rs`), a persisted `CampaignRun`
embedded in the profile, and a new full-screen `Screen` (`campaign_map.rs`).
The start menu gains Start Campaign + Quick Play. Matches launch through the
existing `start_match`; a profile-side "in-progress node" pointer marks a match
as a campaign match so the app can, at the existing `GameOver` seam, record the
win and route back to the map. The engine (`game.rs`) stays campaign-agnostic.

## Data model — new `src/campaign.rs`

`const` map graph, mirroring `opponent.rs`'s roster:

```rust
#[derive(Debug, Clone, Copy)]
pub struct Planet {
    pub id: &'static str,
    pub name: &'static str,
    pub region: &'static str,               // "Outer Rim" / "Mid Rim" / "Core"
    pub blurb: &'static str,                 // info-panel flavor
    pub fx: f32, pub fy: f32,                // normalized bird's-eye position (0..=1); fx rim→core
    pub opponents: &'static [&'static str],  // opponent ids (opponent_by_id), in order
    pub requires: &'static [&'static str],   // planet ids all cleared → this unlocks
}
pub const PLANETS: [Planet; 4] = [ … ];
pub const START_PLANET: &str = "cinder";
pub fn planet_by_id(id: &str) -> Option<Planet>;
```

**First map:** Cinder (Outer, `requires []`, `["greeb"]`) → Ashfall (Mid,
`["cinder"]`, `["vessa"]`) & Drift (Mid, `["cinder"]`, `["toran"]`) → The
Spindle (Core, `["ashfall","drift"]`, `["rix","magistrate"]`).

**Run state** — `CampaignRun` (in `campaign.rs`, embedded in `Profile`):
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CampaignRun {
    beaten: BTreeMap<String, Vec<String>>,  // planet_id -> beaten opponent_ids (set)
    current: Option<String>,                // last-highlighted planet (cursor restore)
    in_progress: Option<NodeRef>,           // the campaign match in flight (the spine signal)
}
pub struct NodeRef { pub planet: String, pub opponent: String } // owned, serde
```
Derived (single source of truth = `beaten` + `PLANETS`, no stored redundancy):
`planet_cleared(p)` (all `p.opponents` beaten), `planet_unlocked(run, p)`
(`p.requires` empty or all cleared), `next_opponent(run, p)` (first unbeaten),
`run_complete(run)` (all cleared). `mark_beaten(planet, opp)` is idempotent
(set insert). These pure methods are the tested core.

## Persistence — `src/profile.rs`

- Add `#[serde(default)] campaign: CampaignRun` to `Profile` (`profile.rs:39`);
  a `campaign()` / `campaign_mut()` accessor. **No `PROFILE_VERSION` bump**
  (additive, serde-defaulted — a pre-009 profile loads with an empty run).
- Update `Default for Profile` and the `profile_with` test helper (compiler
  forces both).
- Mutations pair with `self.profile.save()` at the app call site (no
  autosave), like the deck-builder pair (`app.rs:524`).

## The map screen — new `src/campaign_map.rs`

Mirrors `opponent_select.rs` (state + owned-outcome enum + `draw(...,pulse)` +
one app arm), full-screen with a twinkle clock.

- `Screen::CampaignMap { state: CampaignMapState }` (`src/screen.rs`).
- `CampaignMapState { cursor: usize, stars: Starfield }`; cursor indexes the
  **unlocked** planets.
  - `handle_input(key, profile) -> Option<MapOutcome>`,
    `enum MapOutcome { Moved, Launch { planet, opponent }, Back }`. Arrows/`wasd`
    move over unlocked planets; Enter/Space on an unlocked, un-cleared planet →
    `Launch { planet.id, next_opponent }`; a cleared planet → no-op; Esc/`x` →
    `Back`. Emacs nav free via `resolve_key`.
  - `tick(&mut self, dt)` advances the starfield; called from `App::tick`
    beside `game_state.update()` (`app.rs:695`).
  - `draw(&self, frame, config, profile, pulse)` — header band; node scatter +
    routes; twinkling starfield behind; bottom info panel (name · region;
    opponents beaten/current/locked; blurb). Node emphasis: cleared bright,
    cursor `BorderWeight::Heavy` + `pulse`, open `Normal`, locked `Muted`.

**Layout** — new `CampaignMapLayout` in `src/layout.rs`, computed **per-draw**
from `*config` (like `MenuLayout`, so `resize` needs no map code). Departs from
centered-block: spans `0..num_cols × 0..num_rows`; node at
`x = pad + fx*usable_w`, `y = header_h + fy*usable_h`; reserves a header band
(top) and info-panel band (bottom). Reuse `Rect` + `OverlayLayout` clamp
discipline (`layout.rs:382`).

**Drawing** — no line primitive: compose routes/stars from single-char
`draw_text` calls (`frame.rs:146`, clip-safe). Routes: `─ │` + box-drawing
corners, diagonal `╱ ╲` where needed, from each planet to its `requires`
predecessors. Nodes: `● ◉ ○ ◌` glyphs (matching the approved mockups); stars:
`· ✦`. Glyphs justified as map ornament per `design/brief.md`.

## Motion — the twinkle (`src/campaign_map.rs`, `src/lib.rs`)

The selection pulse is one 2-state signal and must not be reused (per the
amendment). `Starfield` holds fixed star cells (position + a per-star phase,
seeded once) and an accumulated `Duration`; `tick(dt)` advances it; `draw`
derives each star's emphasis (`Muted` ↔ `Normal`, mostly dim) from
`time + phase`, so stars breathe slowly on independent phases — background-only.
New const `STARFIELD_TWINKLE_MS` near `SELECTION_PULSE_MS` (`lib.rs:53`),
slower than the pulse.

## App wiring — `src/app.rs`, `src/menu.rs`

- **Menu:** rename `MenuItem::StartGame` → `QuickPlay` (arm unchanged: the
  has-save→ConfirmNewGame / else `open_opponent_select` logic *is* Quick Play)
  and add `MenuItem::StartCampaign`; list both in `MenuState::new` (`menu.rs:49`),
  add `Display` labels, update the menu count/sequence tests. `StartCampaign` →
  `open_campaign_map()` (mirrors `open_opponent_select`).
- **Three forced `Screen` arms** (template `OpponentSelect`): `?`-help
  (`app.rs:428` → `None`), input router (`app.rs:497` shape, owned outcome;
  `Launch` guards `deck_is_valid()` then `start_match(profile, Some(node))`,
  `Back` → `start_menu`), draw (`app.rs:722`).
- **`start_match` gains campaign context:**
  `start_match(&mut self, opponent, campaign: Option<NodeRef>)` sets
  `self.profile.campaign_mut().in_progress = campaign` + `profile.save()`
  (`Some` for a campaign launch; `None` for Quick Play, clearing any stale
  pointer). OpponentSelect passes `None`; the map passes `Some`.

## The win → progress spine

`in_progress` (profile, persisted → survives Continue) is the single "campaign
match" signal:
- **Record: `App::tick` GameOver seam** (`app.rs:692`). On the transition into
  `GamePhase::GameOver { winner }`, if `in_progress` is `Some(node)` and
  `winner == Player::Player` (`player.rs:8`): `mark_beaten(node)` +
  `profile.save()`. Idempotent; robust to a quit at the popup. (`save_game()`
  still clears the match file on GameOver — fine; progress is in the profile.)
- **Return: InGame input at GameOver, campaign match.** When `in_progress` is
  `Some` and phase is `GameOver`, the acknowledge key clears `in_progress` +
  `profile.save()` + routes to `Screen::CampaignMap`, **suppressing the
  `NextGame` rematch** (`game.rs:648`, quick-play only). Quick-play GameOver
  unchanged.
- `Continue` (`app.rs:641`) unchanged — resumes the singleton save; the
  persisted `in_progress` makes a resumed campaign match still route to the map.

## Testing (per `CLAUDE.md`)

- **`campaign.rs`:** `PLANETS` integrity (unique ids; opponents resolve via
  `opponent_by_id`; `requires` are real planets; `fx/fy` in `0..=1`;
  `START_PLANET.requires` empty). `CampaignRun`: `mark_beaten`/`planet_cleared`,
  `planet_unlocked` (start + both-required rejoin), `next_opponent`,
  `run_complete`; the fork (Ashfall & Drift unlock after Cinder; Spindle only
  after both).
- **`profile.rs`:** campaign round-trip; a pre-009 profile → default empty run.
- **`campaign_map.rs`:** cursor moves only over unlocked planets; `handle_input`
  → `Launch`/`Back`/no-op-on-cleared.
- **`layout.rs`:** `CampaignMapLayout` in-bounds at 89×31 (mirrors the
  `GridLayout` fit test).

## Files

New: `src/campaign.rs`, `src/campaign_map.rs`.
Modified: `src/lib.rs` (mods + `STARFIELD_TWINKLE_MS`), `src/profile.rs`,
`src/screen.rs`, `src/menu.rs`, `src/app.rs`, `src/layout.rs`.
No change expected: `src/game.rs` (engine campaign-agnostic), `src/save.rs`,
`src/frame.rs`, `src/opponent.rs`, `src/board.rs`.
