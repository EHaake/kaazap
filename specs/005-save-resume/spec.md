# Spec: Save & Resume (mid-match)

## Summary

Right now a match evaporates the moment you quit — there's no way to step
away from an in-progress game and return to it. This spec adds **mid-match
save & resume**: an in-progress match persists to disk automatically, and a
**Continue** entry on the start menu drops you back exactly where you left
off.

It's the first real payload for the save layer spec 004 seeded (which
persisted settings), and it establishes two things the campaign will reuse:
an on-disk save file carrying a schema version, and serde-serializable
engine state.

Scope is deliberately just the in-progress match. Campaign progress,
currency, opponent ladders, and unlocked cards don't exist yet, so there is
nothing of that kind to persist; those ride the specs that build them (see
Non-goals). Keeping the format grounded in state that actually exists — a
single match — avoids designing around an undesigned campaign.

## Goals

1. **Auto-save the in-progress match.** No manual save — the match persists
   on its own as you play and when you quit, so you never lose more than (at
   most) the turn you're on.
2. **Continue from the menu.** A **Continue** item appears on the start menu
   only when a resumable match exists, and it resumes that match.
3. **Faithful resume.** Continuing restores the full match position — the
   round number, each side's round wins, the cards on both tables, both
   hands, whose turn it is, and the running totals — with no re-dealing or
   lost progress.
4. **New Game respects a save.** Choosing **Start Game** while a saved match
   exists asks to confirm before discarding it, so a returning player can't
   wipe their game by reflex.
5. **The save clears when the match ends.** When a match is won (or
   abandoned via a confirmed New Game), the save is removed — there's no
   longer an in-progress match, so **Continue** disappears.
6. **Graceful failure.** A missing, unreadable, corrupt, or
   version-incompatible save is ignored — no **Continue** shown — and never
   crashes the game.
7. **Versioned, forward-compatible format.** The save carries a schema
   version so later specs (campaign) can extend it, migrate it, or cleanly
   discard an incompatible file rather than mis-reading it.
8. **Test coverage** for the logic per `CLAUDE.md`: a save→load round-trip
   that preserves match state, the corrupt/incompatible-file fallback, and
   that finishing a match clears the save. Disk I/O itself is a side effect
   — verified by running, not mocked.

## Non-goals (explicitly deferred)

- **Campaign / progression persistence.** Currency, unlocked cards,
  opponent-ladder position, campaign progress — none exist yet. Persisting
  them belongs to the specs that introduce them; this spec saves only an
  in-progress match. (The format is versioned so they extend it later.)
- **Multiple save slots.** One in-progress match, one autosave. Slots are a
  campaign-era concern at most.
- **A manual save/load menu.** Saving is automatic and invisible; no "Save
  Game" / "Load Game" UI, no named saves.
- **Undo, rewind, or save-scumming affordances.** The save is a single
  resumable position, not a history you can step through or exploit to
  re-roll a draw.
- **Cross-device or cloud sync.** A local file on this machine only.
- **Migrating pre-005 saves.** There is no prior save format to be
  compatible with.

## Entities

- **Saved match** — the persisted snapshot of an in-progress match: enough
  to restore the exact position (both players' state, the tables, the hands,
  round wins, whose turn, the phase), plus a schema version. Written to a
  single file in the platform's per-user data location — the same
  `directories`-based persistence layer spec 004 established for settings.
- **Save presence** — whether a resumable match currently exists on disk.
  This is what the start menu reads to decide whether to offer **Continue**,
  checked at launch and after finishing or discarding a match.

## Key user flows

### Quit mid-match and come back

You're partway through a match and press quit. Next launch, the start menu
shows **Continue** at the top; choosing it drops you back into the match
exactly where you left off. (If the terminal is simply closed rather than
quit cleanly, the autosave still holds your position to at least the start
of your current turn.)

### Start a new game when a save exists

From the menu, **Continue** resumes the match; **Start Game** begins a fresh
one — but because that would discard your saved match, it first asks to
confirm. Confirm and the old save is replaced by the new game; decline and
you're back at the menu with your save intact.

### Finishing a match

When a match reaches its end (best-of-3 resolved), it's over, so its save is
cleared. Back at the menu, **Continue** is gone — there's nothing in
progress to resume — until you start and step away from another match.

### First launch, or a corrupt save

With no save file (a fresh install) the menu simply omits **Continue**. If a
save file exists but can't be read — corrupt, truncated, or written by an
incompatible version — it's treated as no save: **Continue** is hidden and
the game starts normally, never crashing.

## Design requirements

- **Saving is invisible.** No "Saving…" spinner, no confirmation toasts, no
  save prompts interrupting play. The only visible surfaces of the whole
  feature are the **Continue** menu item and the discard confirmation.
- **Continue speaks the established menu vocabulary** — the same monochrome
  list, the `▸` marker and pulse, arrow/Enter navigation — it's just another
  menu item, present only when relevant.
- **The discard confirmation reuses the overlay pattern** (How to Play /
  Settings) rather than inventing a new dialog style — a small yes/no over
  the menu.
- **Resume is exact, not approximate.** The restored match is
  indistinguishable from the one you left: no reshuffled hands, no reset
  totals, no replayed rounds. Any unavoidable rounding to a stable point is
  at most to the start of the current turn, never further back.
- **A bad save never breaks the game.** Every failure mode (absent,
  unreadable, malformed, wrong version) degrades to "no Continue," exactly
  like the settings loader degrades to defaults.

## Acceptance criteria

- [ ] Quitting mid-match and relaunching shows **Continue**, which restores
      the exact match position (round, round-wins, both tables, both hands,
      whose turn, totals).
- [ ] With no in-progress match (fresh install, or just after finishing
      one), the menu does not show **Continue**.
- [ ] **Start Game** while a save exists prompts to confirm; confirming
      replaces the save with a new match, declining leaves the save intact.
- [ ] Finishing a match (best-of-3 resolved) clears the save — **Continue**
      is gone next time at the menu.
- [ ] A missing / corrupt / incompatible save file is ignored (no
      **Continue**, no crash); the game starts normally.
- [ ] The save file carries a schema version, and a file whose version
      doesn't match is discarded rather than mis-parsed.
- [ ] Unit tests cover: a save→load round-trip preserving match state, the
      corrupt/incompatible-file fallback, and save-cleared-on-completion.
- [ ] `cargo test` green with the new coverage; `cargo build` introduces no
      new warnings; the game/render/audio boundaries stay intact.

## Resolved decisions

- **Scope is the in-progress match only** (human-ruled). Campaign
  persistence waits for the campaign — there's no campaign state to save
  yet, and designing a save format around an undesigned campaign is exactly
  the speculative generality the constitution warns against. The format is
  versioned so the campaign specs extend it rather than replace it.
- **Resume via a `Continue` menu item** (human-ruled), shown only when a
  save exists, over auto-resuming into the match on launch — it keeps the
  player in control (they can choose New Game instead) and reads naturally
  in the existing menu.
- **Auto-save, no manual save** (proposed — confirm in review). A short TUI
  match has no need for named saves or a save menu; the natural model is a
  single invisible autosave of the current match. The exact save points
  (turn/round boundaries, on quit) are a plan concern; the user-facing
  guarantee is "quit anytime and Continue puts you back."
- **One save file, in the platform data location** (proposed), using the
  same `directories`-based persistence spec 004 established for settings — a
  savegame is per-user data, sitting alongside or near `settings.json`. The
  exact path/filename is a plan detail.
- **Serialization via `serde`** (per the constitution's persistence choice),
  which means the engine's state types gain serde derives. This is an
  engine-touching change — the first since spec 003 — so it ships with
  tests and must not otherwise alter game logic or the render/audio
  boundaries.
