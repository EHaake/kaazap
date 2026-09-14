# Spec: First-run onboarding & controls refinement — spec 023

**Status**: Approved (2026-09-13)
**Depends on**: spec 021 (wager & loss, run-over), spec 012 (economy, shop),
spec 009 (campaign map), spec 006 (control & input polish — partly superseded
here), spec 022 (Quick Play deals the standard deck)

## Summary

The campaign loop is complete but tells a new player nothing: a fresh profile
lands on the galaxy map with no idea that every match is staked, that a loss
forfeits the stake, that going broke ends the run, or that the shop is how you
get past the Mid Rim — and the first match opens with no idea what the keys
do. This spec adds the two ends of onboarding, both shown **once per profile**
and dismissed with one key, both **in the fewest possible words**:

1. **A first-campaign primer** over the galaxy map, the first time a profile
   enters it.
2. **A first-match popup** over the board, the moment a profile's first match
   actually starts.

The existing How to Play overlay stays the full reference and gains a short
campaign section so the primer's content can be found again. The opponent
select screen gets the one line spec 022 deferred: Quick Play deals the
standard deck.

Because the first-match popup lists the keys, this spec also carries a
**controls refinement pass** the person asked for at the same time, so the
popup describes the controls the game actually has:

- **1–4 select a hand card** (like ←/→) instead of playing it outright; a
  second key plays it. The separate "+ or −?" prompt that 1–4 used to open on
  a ± card is gone — the sign shown on the card (flipped with ↑/↓) is the
  sign played, as it already is for the cursor.
- **Space draws** on the player's turn. Playing the selected card is
  **Enter**, with **P** as its secondary key. Space keeps every other role it
  already has: confirm in menus and prompts, next round, new game,
  acknowledge a notice.

This supersedes spec 006's "Space plays the selected card" (its goal 2): the
most common in-play input is drawing from the dealer deck and moving on, so
that is what the spacebar does.

## Goals

1. **A first-campaign primer.** The first time a profile's galaxy map opens, a
   short overlay says what to pay attention to and nothing else: matches are
   staked and a loss forfeits the stake; going broke ends the run and resets
   you; the Outfitter sells the cards that get you past the Mid Rim; the ante
   rises as you go deeper. One key dismisses it; it never shows again for that
   profile.
2. **A first-match popup.** When a profile's first match starts, a popup
   explains the mechanics and the basic controls: draw toward 20 without going
   over, stand to lock your total, play a side card from your hand of four to
   adjust it, first to three rounds wins; the keys for draw, stand, pick a
   card, play it, and flip a ± sign. One key dismisses it; the match does not
   advance underneath it; it never shows again for that profile.
3. **Once per profile, surviving resets.** Both "seen" marks persist in the
   profile without a format version bump, and survive a run-over reset and
   New Campaign the way lifetime stats do. An existing profile from before this
   spec sees each piece once.
4. **Quick Play says it deals the standard deck.** One line on the opponent
   select screen.
5. **How to Play covers the campaign layer** in a few lines, so the primer's
   content remains findable after it is dismissed, and its controls line
   matches the new keys.
6. **Controls refinement.** 1–4 select; Enter or P plays; Space draws on the
   player's turn (and, like D, accepts a bust when over 20); the ± sign prompt
   after a direct key no longer exists; every on-screen hint and the controls
   overlay describe the new keys.
7. **No regression.** No change to scoring, round or match resolution, the
   AI, the economy, the wager prompt, settlement, or any save format; an
   existing profile and an existing mid-match save load as before.

## Non-goals (explicitly deferred)

- **An interactive tutorial** (a guided first match, step-by-step prompts).
  Two static texts and one key each.
- **Reopening the primer or the popup**, or a "show hints again" setting. How
  to Play is the reference; the once-only pieces are not menu items.
- **Any other onboarding surface**: no first-launch welcome on the start menu,
  no shop or deck-builder primer, no hint on the wager prompt beyond what it
  already shows.
- **A configurable / rebindable keymap**, vim-style keys, or mouse input
  (spec 006's non-goals stand).
- **Other control changes** than the two named: the emacs synonyms, Esc/X,
  Q, N, G, S, D, the play log key, and the arrow keys are unchanged.
- **The endgame award, the run summary, the run archive** — their own roadmap
  items.
- **Wording tuning by playtest.** The texts below ship as written unless the
  person changes them in this spec; later wording changes are chores.

## Entities

- **Primer** — the first-campaign overlay: a static text shown once over the
  galaxy map. Content in an asset text file like the existing overlays.
- **First-match popup** — the mechanics-and-controls text shown once over the
  board at a match's start. Static text, same treatment.
- **Seen marks** — two booleans in the profile: primer seen, first-match popup
  seen. Additive, default false, no version bump. Set when the piece is
  *dismissed* (not when it is shown), saved immediately.
- **Hand cursor** — the existing selection over the four hand slots, with its
  pending ± sign. 1–4 now move it; Enter and P commit it; Space no longer
  touches it.

## Key behavior

### The primer

- **When.** The galaxy map screen opens (from Start Campaign's no-progress
  path, Continue, or a confirmed discard-and-enter) and the profile's primer
  mark is unset. Not on Back from the shop or the deck-builder — those return
  to a map the player has already seen — and not from a match's game-over
  acknowledgement.
- **Precedence.** If the map opens broke (an existing profile from before this
  spec could), the run-over notice shows instead and the primer waits for the
  next time the map opens. One modal at a time.
- **While up.** The map is visible underneath. No node can be launched, no
  wager prompt, shop, deck-builder or New Campaign panel opens until it is
  dismissed.
- **Dismissal.** Enter, Space or Esc; every other key is ignored, matching the
  run-over notice. Dismissing sets and saves the mark. Quitting the app with the
  primer up leaves the mark unset, so it shows again next time.
- **Text**, verbatim (the title row centered like the other overlays):

```
=====  Your first campaign  =====

Every match is played for a stake you choose.
Win and it comes back doubled. Lose and it's gone.
Go broke and the run ends: starter deck, seed purse.

The Outfitter (b) sells the cards that get you
past the Mid Rim. The ante rises as you go deeper.

             Enter to continue
```

### The first-match popup

- **When.** A match *starts* — Quick Play or campaign, whichever the profile
  plays first — and the profile's first-match mark is unset. The board is
  drawn underneath with the opening hands dealt. A match resumed from a save
  is not a start and never shows it.
- **While up.** The match does not advance: no player key reaches the game,
  and if the opponent is to act first it does not act until the popup is
  dismissed. (Once dismissed, the opponent's usual thinking pause runs from
  then.)
- **Dismissal.** Enter, Space or Esc; every other key ignored. Dismissing sets
  and saves the mark. Quitting with the popup up leaves the mark unset.
- **Text**, verbatim, describing the keys *as refined below*:

```
=====  How a match works  =====

Draw toward 20 without going over.
Stand to lock in your total.
Play a side card from your hand of four to adjust it.
First to 3 rounds wins the match.

 Space  draw          S  stand
 1-4 or ←/→  pick a card    Enter  play it
 ↑/↓  flip a ± card's sign

             Enter to begin
```

### Seen marks and persistence

- Two additive, serde-defaulted booleans on the profile, default false, saved
  with the profile. `PROFILE_VERSION` stays 1; the mid-match save format is
  untouched.
- A profile from before this spec loads with both unset and sees each piece
  once (ruling C). A fresh profile is the same.
- The run-over reset and New Campaign preserve both marks, alongside lifetime
  stats (ruling A). Nothing else in the reset changes.

### Quick Play line and How to Play

- **Opponent select** gains one line above its controls hint:
  `Quick Play deals the standard deck.` It fits the minimum terminal with the
  full roster, blurb and hint.
- **How to Play** gains a campaign section at the end and its controls line
  changes to the new keys. Proposed text (the person may edit):

```
Campaign: every match is staked. Win and it
comes back doubled; lose and it's gone. Go
broke and the run resets. The Outfitter sells
cards; the ante rises as you go deeper.

1-4 or ←/→ pick a card, Enter plays it,
Space draws, S stands. ? closes.
```

### Controls refinement

On the player's turn:

| Key | Before | After |
|---|---|---|
| 1 2 3 4 | Play that hand card (a ± card then asked + or −) | Select that hand card (empty slot: nothing) |
| ← → | Select a hand card | unchanged |
| ↑ ↓ | Flip the selected ± card's sign | unchanged |
| Enter | Play the selected card | unchanged |
| Space | Play the selected card | Draw (over 20: accept the bust, like D) |
| P | — | Play the selected card |
| D | Draw | unchanged |
| S | Stand | unchanged |
| h / l / + / − / c | Answer or cancel the ± prompt | gone (the prompt no longer exists) |

- A ± card plays with the sign shown on it; ↑/↓ flips it before playing. There
  is no separate sign prompt anywhere, so no player-facing state waits for a
  + or − answer.
- Everywhere else Space is unchanged: menus and modals confirm, round end
  advances, game over starts a new game or acknowledges, the run-over notice
  and the two new pieces dismiss.
- The board's turn hint, the in-game controls overlay (`?`) and How to Play
  name the new keys; nothing on screen still says Space plays a card or that
  1–4 play one.
- Whether the engine keeps an internal pass-through for sign-choice cards is a
  plan decision; the acceptance criteria only require that no player-facing
  prompt exists and that the engine's rules and tests for card effects are
  unchanged.

## Design requirements

- **Overlay convention.** Both pieces are modals over their screen — one modal
  at a time, neutral default, the same bordered box the other text overlays
  use, monochrome.
- **Density and breathing room** (design brief; `CLAUDE.md`). The dismiss line
  is the acted-on element: it gets an empty row above and below it; every
  other row stays compact. The box pads its content evenly — one empty row
  above and below — and never a slab of empty rows under the text. The box's
  own bottom padding row may serve as the row below the dismiss line; do not
  add a second one.
- **Fewest possible words.** The texts above are the ceiling, not a floor to
  pad out.
- **Fits the minimum terminal** (139×31) with the map or board drawn
  underneath.
- **Sound.** Dismissal plays the menu-select sound, like acknowledging the
  other notices.

## Acceptance criteria

- [ ] A fresh profile's first entry to the galaxy map shows the primer with
      the text above; Enter, Space or Esc dismisses it; no node, wager, shop,
      deck-builder or New Campaign panel opens while it is up; it does not show
      on any later map open, including after a run-over reset and after New
      Campaign.
- [ ] A fresh profile's first match — Quick Play or campaign — shows the
      popup with the text above over the dealt board; the match does not
      advance while it is up, including when the opponent acts first; Enter,
      Space or Esc dismisses it; it does not show for any later match, a
      resumed match, or after a reset.
- [ ] Both marks round-trip through `profile.json`; a profile document without
      them loads with both unset; `PROFILE_VERSION` is 1 and the match save
      format is untouched; quitting with either piece up leaves its mark unset.
- [ ] If the map opens broke, the run-over notice shows and the primer does
      not; the primer shows on the next map open.
- [ ] The opponent select screen shows "Quick Play deals the standard deck."
      above its hint, on-frame at the minimum terminal with the full roster.
- [ ] How to Play contains the campaign section and the new controls line;
      the in-game controls overlay and the board's turn hint name Space as
      draw, Enter/P as play, and 1–4 as select; no on-screen text says Space
      plays a card or that 1–4 play one.
- [ ] On the player's turn: 1–4 move the selection to that slot (an empty slot
      changes nothing) and play nothing; Enter or P plays the selected card
      with the sign shown on it, for fixed, ±, flip and tiebreaker cards
      alike; Space draws, and over 20 accepts the bust like D; no key opens a
      + or − prompt. Space at round end, game over, menus, modals and notices
      behaves as before.
- [ ] Unit tests cover: the seen marks' defaults, round trip and survival of
      the reset; the primer and popup showing exactly once and blocking input
      underneath; 1–4 selecting; Enter/P playing each card kind at the shown
      sign; Space drawing on the player's turn and advancing at the pauses;
      the existing card-effect and resolution tests unchanged.
- [ ] No change to `game.rs` rules (card effects, scoring, resolution), the
      AI, the economy, the wager prompt, settlement, or `save.rs`; `cargo
      build` has no new warnings; `cargo test` is green.
- [ ] Attested in play by the person: a fresh profile sees the primer once on
      the map and the popup once at its first match; the new keys feel right;
      an existing profile sees each piece once and keeps everything else.

## Resolved decisions

Ruled with the person on 2026-09-13, on the recommendations as proposed:

- **A — The seen marks survive resets.** A run-over and New Campaign wipe the
  run but keep the marks, as they keep lifetime stats; a player who has read
  the rules is not re-taught on the way back in.
- **B — The first match of either mode** shows the popup: the mechanics and
  controls are identical in Quick Play and campaign, so whichever comes first
  gets it.
- **C — Existing profiles see both pieces once.** The profile format could
  treat a missing mark as seen; the person chose to show them, so an existing
  profile can try them without a wipe, at the cost of one key each.
- **D — The Quick Play line lives on the opponent select screen**, where Quick
  Play is chosen, not in How to Play.
- **E — How to Play gains a short campaign section**, so it is genuinely the
  full reference once the primer is gone.
- **Timing (person's words).** The primer shows the first time through the
  campaign, on first entering the galaxy screen; the gameplay popup once the
  first game actually starts.

Ruled the same day, on the recommendations as proposed:

- **F — The controls refinement rides in this spec**, not a separate one or
  a chore: the popup's key list depends on it, and retiring the ± prompt
  touches the engine's phase list, which the chore lane excludes. Spec 006's
  goal 2 is superseded; `DECISIONS.md` records it at the merge.
- **G — P is the secondary play key.** Free on the player's turn, mnemonic,
  and away from the draw/stand hand (D, S, Space). Enter stays primary.
- **H — The ± prompt is retired outright** rather than kept for the direct
  keys: with 1–4 selecting, every play goes through the cursor, whose ↑/↓
  sign is already the answer; a second way to answer the same question would
  be the indirection the constitution tells us to cut.
