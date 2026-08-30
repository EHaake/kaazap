# Decisions

Business, product, and process context that doesn't belong in `plan.md`
(technical design) — the reasoning behind naming, scope, and process
calls that a future session (or future you) would otherwise have to
reconstruct from chat history that's already gone.

## Naming

"Kaazap" is "Pazaak" spelled backwards.

## Purpose and bar for quality

This is a personal project, but the intent is for it to eventually be
presentable enough to share (portfolio piece, itch.io release). That
raises the bar somewhat versus a pure hobby throwaway: worth caring about
things like a real README, no crashes/panics in normal play, and
documenting platform-specific build requirements (e.g. ALSA dev libs on
Linux once sound lands) — without turning this into a project that needs
enterprise-grade process for a solo effort.

## Rule variants (intentional deviations from real Pazaak)

- **Main deck draws 0–10, not 1–10.** Real Pazaak's main deck only goes
  1–10; Kaazap's includes 0. This was set intentionally during earlier
  development — the original reasoning wasn't written down and isn't
  currently recalled, but it's deliberate, not a bug. Kept as-is; treat
  it as one of Kaazap's subtle rule changes rather than something to
  "fix" without a real design conversation first.
- **No per-turn side-card limit.** Real Pazaak allows exactly one
  side-deck card per turn; Kaazap allows any number in a turn,
  including chaining several recovery cards while over 20. Deliberate
  simplicity (no per-turn flag to track) — confirmed as a variant, not
  canon fidelity, during spec 001's review.
- **The draw key accepts the bust while over 20.** Going over 20
  doesn't bust by itself (that part is canon — you may play a card to
  recover before your turn ends); Kaazap's variant is that while over,
  `d`/space draws nothing and instead stands into the bust, same as
  `s`, so every key keeps a meaning in that state. Human-ruled during
  spec 001 (T008b).
- **Flip/bust edge rulings** (canon sources are thin here, spec 001):
  a *standing* player pushed over 20 by a flip card busts immediately;
  a *live* player pushed over gets their recovery window; any side
  still over 20 when the round ends for another reason is bust at
  resolution; and if both sides end up bust, the round is a tie.

## Why a campaign/progression system exists at all

In KOTOR, Pazaak's minus and special side-deck cards are gated by the
game's larger economy — you buy better cards from vendors using credits
earned across the whole RPG. Kaazap has no surrounding RPG to supply that
context. Rather than just handing the player every card type for free
from the start, Kaazap introduces its own lightweight progression:
currency and/or card packs earned from match wins, unlocking better cards
over a campaign of increasingly difficult opponents. This is what gives
the "distinct opponents," "full save/resume," and "card variety" pieces
of the project a shared reason to exist together, rather than being three
unrelated features.

## Explicitly deferred out of v1

- **Full side-deck customization** (building/collecting your own 10-card
  deck between matches, KOTOR-vendor style). v1 ships a simple default
  deck instead. Revisit once the campaign loop itself is proven out.
- **Mid-run side-deck redraw.** Real Pazaak doesn't redraw your hand
  within a given match, and v1 of Kaazap won't either. Deferred rather
  than rejected: once the full game exists, rule enhancements that fit
  the TUI format get considered as a deliberate post-v1 pass (see
  `ROADMAP.md`), and a redraw mechanic is a candidate there.

## Testing

No tests exist in the codebase as of the project's pickup (last commit
Jan 2026). Going forward, game logic gets unit test coverage as it's
written or touched — not retroactively applied to every existing line on
day one, but treated as real discipline from here on, not aspirational.
