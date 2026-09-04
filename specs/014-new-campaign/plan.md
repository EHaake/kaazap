# Plan: New Campaign option (spec 014)

## What the code already gives us

- **Menu → campaign entry** is `activate_menu_item(MenuItem::StartCampaign)`
  (`app.rs:808`): today `has_save ? ConfirmNewGame{Campaign} : open_campaign_map()`.
  That `ConfirmNewGame` discards a stray **mid-match save** on entry — separate from
  campaign progress.
- **Modals** are a one-at-a-time `enum Modal` (`app.rs:250`); input dispatched in
  `handle_input` (`app.rs:476-487`), drawn in the `match &self.modal` (`app.rs:929`).
  `ConfirmNewGame { on_yes, pending }` is the reference: a 2-way ←/→ toggle
  (`handle_confirm_input`, `app.rs:745`) drawn by `draw_confirm_new_game`
  (`app.rs:944`, safe option default).
- **Starter profile** is `Profile::default()` (`profile.rs:79`) — `starter_collection`
  + `starter_deck` + empty `CampaignRun` + `credits: 0`. Settings are a separate
  file, so a profile reset leaves them untouched.
- **`CampaignRun`** (`campaign.rs:155`) — private `beaten` map; no progress accessor.
- `open_campaign_map()` (`app.rs:364`) reads the live profile, so a reset shows a
  fresh map immediately.

## Design

### `src/campaign.rs` — detect progress
`pub fn has_progress(&self) -> bool { !self.beaten.is_empty() }`. Cleared opponents
= real progress; a started-but-unwon first match has empty `beaten`, so the choice
isn't offered there (Continue and New would be identical).

### `src/profile.rs` — reset
`pub fn reset_to_starter(&mut self) { *self = Profile::default(); }` — the full
fresh start (starter collection/deck, empty campaign, 0 credits; version unchanged).
Settings persist (separate file). Unit-tested.

### `src/app.rs` — the flow
Two new `Modal` variants, both 2-way toggles like `ConfirmNewGame`:
- `CampaignEntry { on_new: bool }` — Continue (default) vs New Campaign.
- `ConfirmNewCampaign { on_yes: bool }` — destructive confirm, default No.

Extract today's StartCampaign body into `enter_campaign_continue()`
(`has_save ? ConfirmNewGame{Campaign} : open_campaign_map()`), then:
```
MenuItem::StartCampaign => {
    if self.profile.campaign().has_progress() {
        self.modal = Some(Modal::CampaignEntry { on_new: false });
    } else {
        self.enter_campaign_continue();
    }
}
```
Input (extend the modal dispatch at `app.rs:476` + a handler like `handle_confirm_input`):
- **CampaignEntry**: ←/→ (a/d) toggle; Enter → `on_new` ? open `ConfirmNewCampaign{on_yes:false}` : `enter_campaign_continue()`; Esc → close.
- **ConfirmNewCampaign**: ←/→ toggle; Enter+Yes → `reset_to_starter()` + `profile.save()` + `crate::save::clear()` + `has_save=false` + `last_reward=None` + `open_campaign_map()`; Enter+No / Esc → close to menu (mirrors `ConfirmNewGame`).

Drawing: add two arms in the `match &self.modal` (`app.rs:929`). Factor
`draw_confirm_new_game`'s body into `draw_two_choice(title, hint, left_label,
right_label, left_selected, pulse)` (widths from the actual label strings, so
"New Campaign" fits) and have all three modals use it — DRYs rendering without
changing `ConfirmNewGame`'s look.

Confirm copy (explicit, destructive): title ≈ "Start a new campaign? This erases
your progress, credits, and collection.", choices No / Yes, default No.

## Files

- `src/campaign.rs` — `has_progress()` + test.
- `src/profile.rs` — `reset_to_starter()` + test.
- `src/app.rs` — two `Modal` variants, `enter_campaign_continue()`, the
  StartCampaign branch, input handling, the reset action, `draw_two_choice` + two
  draw arms; input-handler tests.
- **No change:** engine, board, campaign map/shop screens, save *format*.

## Tests

- `reset_to_starter`: a dirtied profile (progress + credits + a grown/edited
  collection) → starter collection/deck, empty campaign, 0 credits.
- `has_progress`: false on default, true after `mark_beaten`.
- App input flow: from `CampaignEntry`, Enter on New opens `ConfirmNewCampaign`;
  from `ConfirmNewCampaign`, Yes resets (progress gone, credits 0), No leaves it
  intact. Follow the existing `app.rs` input-test style; test the reachable
  toggle/branch logic (avoid disk-touching `App` construction where possible).

## Alternatives rejected

- A separate top-level "New Campaign" menu item — the human asked for a choice off
  Campaign, and it keeps the menu list stable.
- NG+ (keep cards/credits) — deferred to the roguelike mode (spec E).

## Verification

`cargo build` (no new warnings) + `cargo test` (verbatim). Driver (back up real
`profile.json`/`saves/`, restore + checksum after): cleared progress → Campaign
shows Continue/New; New → confirm; Yes → fresh map (0/8, only Cinder unlocked);
no-progress profile skips the modal. Snapshot the modals at 89×31.
