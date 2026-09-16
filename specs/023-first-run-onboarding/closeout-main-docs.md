# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 023)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T009, ready to paste on `main` after the merge
(the same close-out shape specs 020, 021 and 022 used). Nothing here is applied
by the spec branch.

No `CLAUDE.md` amendment is needed for the feature itself: spec 023 changed no
rule the constitution states. (The branch does carry one unrelated `CLAUDE.md`
commit, `d0eb6e8` — the experiment-2 model-policy note — which landed on the
branch rather than on `main`; see §3.)

---

## 1. `ROADMAP.md` — three edits

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Difficulty & economy balance pass (spec 022)**
entry (which ends "…`docs/opponents.md` and `docs/economy.md` re-synced.",
currently `ROADMAP.md` line ~297, just before `## Backlog`):

```markdown
- **First-run onboarding & controls refinement** (spec 023) — the two ends of
  onboarding, plus the control pass the second one depends on. **A first-campaign
  primer** (`assets/primer_text.txt`) is raised over the galaxy map the first
  time a profile enters it *from the menu* — Start Campaign, Continue, the
  discard-and-enter confirm, or a confirmed New Campaign — and says only what a
  new player must know: every match is staked, a loss forfeits it, going broke
  resets the run, the Outfitter (`b`) sells the cards that get you past the Mid
  Rim. **A first-match popup** (`assets/first_match_text.txt`) is raised over the
  dealt board the moment a profile's first match *starts* — Quick Play or
  campaign, whichever comes first, never on a resume — and names the rules and
  the keys, holding the match (including an opponent-first deal) until it is
  dismissed. Both are `Modal` variants drawn through the existing overlay seam,
  shown **once per profile**, dismissed with Enter, Space or Esc, and blocking
  every key underneath. The marks are two serde-defaulted `bool`s in
  `profile.json` (`primer_seen` / `first_match_seen`) that **survive a run-over
  reset and New Campaign** like the lifetime stats — `PROFILE_VERSION` stays 1
  and `save.rs` is untouched, so existing profiles load unset and see each piece
  once. **Controls refinement** (supersedes spec 006's goal 2): **1–4 now
  *select* a hand card** like ←/→ instead of playing it, **Enter or P plays**
  the selected card at the sign shown on it, and **Space draws** on the player's
  turn (accepting the bust while over 20, like D) while keeping every other
  "proceed" role it had. The separate "+ or −?" prompt 1–4 used to open on a ±
  card is **retired** — ↑/↓ on the card is the only answer — though the engine's
  `AwaitingSignChoice` phase and its actions stay as an unobservable transient
  so `save.rs`, `tests/balance.rs` and every `sign_*` engine test are unchanged.
  How to Play gained a short **campaign section** so the primer's content is
  findable afterward, the in-game `?` overlay and the board's turn hint were
  re-synced, and the opponent select screen finally says **"Quick Play deals the
  standard deck."** (spec 022's deferred line). No rules, AI, economy, wager or
  settlement change.
```

### 1b. The spec 006 entry's Space description is superseded

In the **Control & input polish (spec 006)** entry under `## Shipped`
(currently `ROADMAP.md` lines 56–59), replace:

```markdown
  changes that make **Space** the single "confirm / proceed" key: it plays
  the highlighted hand card on your turn (like Enter), advances at the
  round-end pause (like `n`), and starts a new game at game over (like `g`).
  Drawing moved to its own dedicated key, `D` (Space no longer draws).
```

with:

```markdown
  changes that made **Space** the single "confirm / proceed" key: it played
  the highlighted hand card on your turn (like Enter), advances at the
  round-end pause (like `n`), and starts a new game at game over (like `g`).
  Drawing moved to its own dedicated key, `D`. **The in-play half is superseded
  by spec 023**: on the player's turn Space now *draws* (D still does too),
  Enter or P plays the selected card, and 1–4 select rather than play. The
  round-end and game-over roles are unchanged.
```

Note: the quoted block stops at "(Space no longer draws)." — the rest of that
bullet (the Emacs nav keys, `resolve_key`, the `?` overlays, the
`game_action_from_key` note) is **not** part of the replacement and stays as
written.

### 1c. Replace the **First-run onboarding** backlog bullet

Under "### Onboarding, endgame & release readiness (suggested 2026-09-13, after
spec 022)", replace the whole bullet (currently `ROADMAP.md` lines ~448–466,
beginning "- **First-run onboarding (human-prioritized).** Two pieces, both
shown once per profile and dismissable") with:

```markdown
- **First-run onboarding (human-prioritized).** — ✅ **Shipped (spec 023** — see
  Shipped above). Both pieces landed as specified: the first-campaign primer on
  a fresh profile's first menu entry to the galaxy map, and the first-match
  popup the moment its first match starts, each shown once per profile, each
  dismissed with one key, each holding everything underneath while it is up. The
  marks are serde-defaulted profile flags with no version bump, and they survive
  a reset — a player who has read the rules is not re-taught on the way back in.
  How to Play kept its role as the full reference and gained the campaign
  section; the cheap extra shipped too ("Quick Play deals the standard deck." on
  the opponent select screen). The spec also carried the **controls refinement**
  the popup's key list depends on (1–4 select, Enter/P play, Space draws, the ±
  prompt retired), which the backlog had not anticipated.
```

---

## 2. `DECISIONS.md` — two edits

### 2a. The spec 001 draw-key bullet's parenthetical is superseded

In the **"The draw key accepts the bust while over 20"** bullet (currently
`DECISIONS.md` lines 40–41), replace:

```markdown
  (T008b). (Spec 006 made Space "play the selected card" rather than
  draw, so `d`/`s` are the bust-accepting keys now — Space isn't.)
```

with:

```markdown
  (T008b). (Spec 006 made Space "play the selected card" rather than
  draw, so for a while `d`/`s` were the bust-accepting keys and Space wasn't;
  **spec 023 gave Space back to drawing**, so Space, `d` and `s` all accept the
  bust now — the in-game controls overlay says "Over 20: Space, D or S accepts the bust." and the board's alert reads "OVER 20!  (Space/D/S: bust)")
```

### 2b. Append a new section at the end of the file

Append after the "## Chore: only the acted-on row gets air; modals pad evenly
(2026-09-13)" section's last bullet ("…`V_PAD` now means the total vertical
padding; `H_PAD` is still per side."):

```markdown
## First-run onboarding & controls refinement (spec 023)

A new player is now told the two things the loop never told them — what a stake
costs and what the keys do — each once per profile, in the fewest possible
words. Ruled with the human on 2026-09-13, on the recommendations as proposed.

- **A — The seen marks survive resets.** A run-over and New Campaign wipe the
  run but keep the marks, as they keep lifetime stats; a player who has read the
  rules is not re-taught on the way back in.
- **B — The first match of either mode** shows the popup: the mechanics and
  controls are identical in Quick Play and campaign, so whichever comes first
  gets it.
- **C — Existing profiles see both pieces once.** The profile format could have
  treated a missing mark as seen; the human chose to show them, so an existing
  profile can try them without a wipe, at the cost of one key each.
- **D — The Quick Play line lives on the opponent select screen**, where Quick
  Play is chosen, not in How to Play.
- **E — How to Play gains a short campaign section**, so it is genuinely the
  full reference once the primer is gone.
- **Timing (the human's words).** The primer shows the first time through the
  campaign, on first entering the galaxy screen; the gameplay popup once the
  first game actually starts.
- **F — The controls refinement rides in this spec**, not a separate one or a
  chore: the popup's key list depends on it, and retiring the ± prompt touches
  the engine's phase list, which the chore lane excludes. **Spec 006's goal 2 is
  superseded** — the most common in-play input is drawing and moving on, so that
  is what the spacebar does.
- **G — P is the secondary play key.** Free on the player's turn, mnemonic, and
  away from the draw/stand hand (D, S, Space). Enter stays primary.
- **H — The ± prompt is retired outright** rather than kept for the direct keys:
  with 1–4 selecting, every play goes through the cursor, whose ↑/↓ sign is
  already the answer; a second way to answer the same question would be the
  indirection the constitution tells us to cut.

Design tensions resolved during planning:

- **The engine keeps the sign-choice pass-through; only the player-facing prompt
  goes.** `GameAction::{PlayHand, ChooseSign, CancelSignChoice}`,
  `GamePhase::AwaitingSignChoice`, `play_card`, `commit_sign_choice` and every
  `sign_*` engine test are unchanged; what was removed is the
  `AwaitingSignChoice` branch of `game_action_from_key` (h/l/+/−/1/2/c), the
  `'1'..'4' → PlayHand` arm, and the board's sign prompt. Reasons: the spec
  requires the engine's card tests unchanged and `save.rs` untouched, and
  `SavedPhase::AwaitingSignChoice` exists on disk; `tests/balance.rs` and the
  two headless loops are written against it; and after this spec the only
  producer of `PlayHand` in the binary is `cursor_confirm`, which answers the
  phase inside the same key event, so the phase is unobservable — no key maps
  into it, `update()` leaves it alone, `status_message` returns `None` for it.
  The cost is one transient phase the player can never see; the alternative (a
  signed `PlayHand`) would rewrite `cursor_confirm`, nine tests, the simulator
  and the save format for no visible gain.
- **Where the primer is raised: the three menu-entry sites, not the game-over
  path.** `enter_campaign_map` gained a `from_menu: bool`, and
  `map_entry_modal(broke, primer_due)` is the pure precedence seam — run-over
  first, then the primer, else nothing. The menu-entry callers pass `true`:
  `enter_campaign_continue` (both the no-progress path and `CampaignEntry`'s
  Continue), the `PendingStart::Campaign` confirm arm, and `start_new_campaign`
  — which switched from `open_campaign_map()` to `enter_campaign_map(true)`,
  and is never broke at that moment because the reset just left the seed purse.
  New Campaign is **menu-only** (no `MapOutcome` variant; `ConfirmNewCampaign`
  is raised only from `CampaignEntry`), so no origin flag beyond `from_menu` is
  needed. The game-over acknowledgement passes `false` — the spec says the
  primer is not raised from a match's game-over path, which is reachable with
  the primer unseen only by a pre-023 profile resuming a saved campaign match.
  The shop and deck-builder Backs keep plain `open_campaign_map` and never raise
  it: they return to a map already seen.
- **A pre-023 mid-match save can be sitting in `AwaitingSignChoice`.** Pressing
  `1` on a ± card used to save the game in that phase (`save_game` fires on every
  `game_changed`), and after this spec no key maps to `ChooseSign`, so such a
  save would resume soft-locked. Continue therefore applies
  `GameAction::CancelSignChoice` immediately after `save::load()` — a no-op in
  every other phase, and in that one it returns the card to hand at `PlayerTurn`
  by the engine's existing cancel semantics
  (`sign_cancel_restores_turn_with_card_unspent`). `save.rs` is untouched;
  `CancelSignChoice` stays for exactly this.

**Supersedes spec 006's goal 2** ("Space plays the selected card", `ROADMAP.md`
under *Control & input polish*): Space draws on the player's turn again. Its
round-end, game-over, menu, prompt and notice roles are unchanged, and the
parenthetical on the spec 001 draw-key bullet above was corrected to match.

No `game.rs` rules change (card effects, scoring, resolution), no AI, economy,
wager-prompt or settlement change, and no `save.rs` change; `game.rs` moved only
at `game_action_from_key`, a new `restart_opponent_pause` (so the opponent's
thinking pause runs from the popup's dismissal rather than through it), and its
own tests. `PROFILE_VERSION` and `SAVE_VERSION` both stay 1. Monochrome by
construction.
```

---

### 2c. Two more lines for the spec 023 section (append after the tension bullets)

```markdown
- **Invariant, recorded at the sweep:** `enter_campaign_map` and `start_match`
  now *assign* the modal (`map_entry_modal(...)` / `Modal::FirstMatch`), so
  every caller clears its own modal *before* entering the map or starting a
  match — the wager Commit, the discard-and-enter confirm, the campaign-entry
  panel and the New Campaign confirm all do. A future caller that raises a
  modal first would lose it silently; the invariant is verified by reading and
  by the driver, not by a test (plan tension §4: App tests never touch disk).
- **Process note:** the experiment-2 amendment to `CLAUDE.md` (commit
  `d0eb6e8`) landed on the 023 branch by the person's one-commit ruling of
  2026-09-14 and rode into `main` at the merge; never force-pushing outranked
  the commit-straight-to-`main` convention for repo-wide files.
```

## 3. Not drafted here (deliberately)

- **`README.md` and `docs/` name no in-match keys** — grepped; nothing there
  claims Space plays a card or that 1–4 do. No change needed.
- **The `CLAUDE.md` commit on this branch** (`d0eb6e8`, "Constitution: run
  experiment 2 from the skill's exp-2-implementer-fable-medium branch") is a
  repo-wide file that the git conventions say commits straight to `main`. It is
  already on the branch and will ride in at the merge; flagged here rather than
  rewritten, since never force-pushing and keeping history is the stronger rule.
  Nothing about it is spec-023 content.
- **`.claude/skills/run-kaazap/SKILL.md` moved on the branch** although
  `plan.md` §Files does not list it: its key-reference table asserted "1–4
  play a hand card" and "Space no longer draws", both false after this spec,
  so it was corrected under T009 as a carried review note. A documentation
  correction the shipped change made necessary, not new scope.
- **The in-game `?` overlay and How to Play texts are assets on the branch**
  (`assets/game_overlay_text.txt`, `assets/how_to_play_text.txt`), so they merge
  with the spec and need no close-out edit.
