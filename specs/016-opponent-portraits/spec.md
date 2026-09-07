# Spec 016 — Opponent portraits

## Summary

Give each opponent a face. Today an opponent is a name, a difficulty, and an
AI — the player never *sees* who they're up against. This spec adds a
low-resolution, monochrome portrait for every opponent, shown during the match
and when previewing/selecting one, so opponents read as characters — and so the
banter planned next has a face to come from.

## Why

Personality and immersion. The engine is complete and impersonal; the opponents
are the game's cast, and a face is the cheapest, strongest way to make them feel
like people. Portraits are also the anchor the coming banter attaches to, which
is why they land first.

## User-facing behavior

### In-match

- Every match (campaign and Quick Play) shows the opponent's portrait, **always
  visible**, in an **opponent-presence panel** beside the board.
- The board keeps its current layout and size; the in-match view grows to include
  the panel, and the **minimum terminal size increases** to fit it. Below the new
  minimum the game errors clearly with the required size, exactly as it does today
  — it never renders broken.
- The panel shows the portrait and the opponent's name, and is the reserved home
  for later additions (a banter line, round pips) — reserved here, not built.
- The player has no portrait; the mirrored space opposite the panel is left for a
  future player-status panel (arrives with the stakes spec), so the layout is
  intentionally asymmetric for now.

### Previewing an opponent

- The opponent's portrait appears as a preview in **Quick Play opponent-select**
  and when a **campaign-map node is focused**.

### Appearance

- Portraits are **low-resolution, monochrome, stylized alien faces** — clearly
  readable as faces, with eyes and expression — consistent with the game's
  monochrome identity.
- Designs are **original**, in the spirit of a cantina's mix of species; they
  reproduce no trademarked species (the same stance as the game's original planet
  names and no-copyrighted-music rule).
- A **generic fallback portrait** stands in for any opponent lacking bespoke art
  (e.g. the Quick Play default opponent), so the panel is never blank.

## Acceptance criteria

- [x] Each of the 10 roster opponents shows a **distinct** portrait in-match.
  *(Evidence: `roster_portraits_are_pairwise_distinct_and_default_is_generic`;
  driver — cursoring the roster shows 10 distinct faces, none blank.)*
- [x] The opponent portrait is visible throughout every campaign and Quick Play
  match. *(Evidence: T006 draws it every frame from `state.opponent_profile`;
  driver — the panel is beside the board for the whole match, Quick Play and campaign.)*
- [x] The portrait also appears when previewing an opponent (opponent-select;
  focused campaign-map node). *(Evidence: T005 opponent-select preview + T007 campaign
  rail; driver-confirmed on both.)*
- [x] Portraits are monochrome and legible as faces — eyes and expression
  discernible — verified by looking at the running game, not only by tests.
  *(Evidence: person sign-off on the Fable-authored faces + the panels in the running
  game; monochrome is structural — `frame.rs` has no color path.)*
- [x] The fallback opponent shows the generic portrait; the panel is never empty.
  *(Evidence: `DEFAULT_OPPONENT.portrait == generic` test; T007's
  next→last→`unwrap_or(DEFAULT_OPPONENT)` chain has no blank path — reviewer-confirmed.)*
- [x] The minimum terminal size is updated and documented (README), and every
  existing screen (board, start menu, opponent-select, campaign map, deck-builder,
  shop) still fits at the new minimum; below it, the game errors clearly with the
  required dimensions. *(Evidence: `min_size() == (139,31)` test; `Readme.md` documents
  it; fit tests run at 139×31 — board+panel, campaign map, briefcase, roster — and every
  other screen is a centered block that already fit 89, so it fits wider; `from_terminal`/
  `draw_too_small` error below the minimum, unchanged, reading `min_size()`.)*
- [x] Build and tests green; **no color introduced** — monochrome preserved.
  *(Evidence: 267 tests green, no new warnings; the render model has no color API —
  portraits draw only `draw_text`/`draw_box` with an `Emphasis`.)*

## Non-goals

- **Animation** of any kind (blink, poses, angles) — a later spec.
- **Player-side portrait / player-status panel** — later, with the stakes spec.
- **Banter text** — the next personality spec; this one only reserves the space.
- **Color** — permanently out, per the design brief.
- A runtime portrait editor or art-authoring UI.

## Resolved decisions (from the design conversation)

- **Always-visible in-match portrait, growing the minimum terminal** — chosen
  over a responsive panel that hides on narrow terminals, because portraits are
  central to the game's feel. [human-ruled]
- **Low-res monochrome "pixel-art" alien faces, generated in-repo** (Claude-
  authored), validated by an **art-format spike** — one portrait built and looked
  at before authoring the rest. If in-repo generation can't reach "recognizable as
  a face with eyes and expression," revisit the approach (e.g. human-supplied
  source art) rather than ship mush. [human-ruled]
- **Static only; animation deferred** to a later spec. [human-ruled]
- **Monochrome, original non-trademarked alien designs.** [human-ruled]
