//! Opponent banter: which line an opponent says, and when. Pure logic + static
//! content, no dependency on rendering (the panel drawer lives in `portrait.rs`).
//!
//! `App` diffs successive [`BanterSnapshot`]s through [`banter_event`] to decide
//! which line class fires — mirroring `audio.rs`'s snapshot/diff pattern (the
//! engine itself says nothing). Lines are picked from a per-opponent [`BanterSet`]
//! (or [`GENERIC`] for the default / an unknown id) via [`pick`], which never
//! repeats the currently-shown line back-to-back. Spec 017.
//!
//! A chosen line is spoken word by word, in place, through a [`Speech`] (spec 030).
//!
//! Inside a campaign series (spec 030), a match start or a venue arrival draws
//! from the pool for where the series stands ([`SeriesState`], via
//! [`start_lines`]), and the match that decides the series draws series won /
//! series lost ([`lines_in_series`]).

use std::time::Duration;

use crate::WORD_STEP_MS;
use crate::campaign::{Series, wins_needed};
use crate::game::{GamePhase, GameState, RoundOutcome};
use crate::player::Player;

/// A banter-worthy event, from the **opponent's** point of view (RoundWin = the
/// opponent won the round; MatchWin = the opponent won the match). `MatchStart`
/// is set at seeding, never produced by [`banter_event`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BanterEvent {
    MatchStart,
    RoundWin,
    RoundLoss,
    RoundTie,
    OpponentBust,
    PlayerBust,
    MatchWin,
    MatchLoss,
}

/// A minimal snapshot of the banter-relevant facts. `App` diffs successive
/// snapshots to decide which line fires. (No `PartialEq` — the diff runs through
/// [`banter_event`], and `RoundOutcome` isn't `Eq`.)
#[derive(Debug, Clone, Copy)]
pub struct BanterSnapshot {
    o_bust: bool,
    p_bust: bool,
    outcome: Option<RoundOutcome>,
    game_over: bool,
    opp_won_game: bool,
    /// The player has acted this round — drawn a dealer card, played a side
    /// card, or stood. `false` at every round's pristine start and at match
    /// start; drives phase-based banter clearing (spec 017 §8).
    player_engaged: bool,
}

impl BanterSnapshot {
    pub fn of(gs: &GameState) -> Self {
        let (game_over, opp_won_game) = match gs.game_phase {
            GamePhase::GameOver { winner } => (true, matches!(winner, Player::Opponent)),
            _ => (false, false),
        };
        Self {
            o_bust: gs.opponent.bust,
            p_bust: gs.player.bust,
            outcome: gs.round_outcome,
            game_over,
            opp_won_game,
            player_engaged: !gs.player.dealer_row.is_empty()
                || !gs.player.played_row.is_empty()
                || gs.player.stood,
        }
    }
}

/// Whether the player's play has *resumed* across `prev` → `curr`: the
/// false→true transition of [`BanterSnapshot::player_engaged`], i.e. the
/// player's first action of a round. `App` clears the current banter line on
/// this transition (spec 017 §8), so a reaction shown through the "next round"
/// pause is gone once the next round's play begins. Pure — unit-tested.
pub fn play_resumed(prev: &BanterSnapshot, curr: &BanterSnapshot) -> bool {
    curr.player_engaged && !prev.player_engaged
}

/// Whether a match has just *restarted* across `prev` → `curr`: the true→false
/// transition of [`BanterSnapshot::game_over`]. A rematch (`new_game` in place,
/// after game over) is the only in-game `game_over` true→false transition, so
/// this is a clean diff signal that a fresh match has begun without leaving
/// `Screen::InGame` — `App` uses it to seed a match-start greeting (spec 017 §8
/// rematch note). Pure — unit-tested.
pub fn match_restarted(prev: &BanterSnapshot, curr: &BanterSnapshot) -> bool {
    prev.game_over && !curr.game_over
}

/// The banter event for the transition `prev` → `curr`, or `None` if nothing
/// new fires. Ordered precedence (spec 017 §5): **match end** (the closing line)
/// beats a **bust** (the vivid reaction) beats a **round outcome** (the plain
/// resolution) — a bust and the outcome it causes surface in one tick, and the
/// final blow surfaces the bust and game-over together. Pure — unit-tested with
/// no game loop.
pub fn banter_event(prev: &BanterSnapshot, curr: &BanterSnapshot) -> Option<BanterEvent> {
    if curr.game_over && !prev.game_over {
        return Some(if curr.opp_won_game {
            BanterEvent::MatchWin
        } else {
            BanterEvent::MatchLoss
        });
    }
    if curr.o_bust && !prev.o_bust {
        return Some(BanterEvent::OpponentBust);
    }
    if curr.p_bust && !prev.p_bust {
        return Some(BanterEvent::PlayerBust);
    }
    if curr.outcome.is_some() && prev.outcome.is_none() {
        return Some(match curr.outcome {
            Some(RoundOutcome::OpponentWon) => BanterEvent::RoundWin,
            Some(RoundOutcome::PlayerWon) => BanterEvent::RoundLoss,
            _ => BanterEvent::RoundTie,
        });
    }
    None
}

/// A voice: one line pool per event class, all `&'static`. The five *repeatable*
/// classes (`round_win`/`round_loss`/`round_tie`/`opponent_bust`/`player_bust` —
/// events that can fire in consecutive rounds) carry ≥3 lines each so [`pick`]
/// always has an alternative and can honor the no-back-to-back-repeat rule; the
/// three once-per-match classes (`match_start`/`match_win`/`match_loss`) carry ≥2.
pub struct BanterSet {
    pub match_start: &'static [&'static str],
    pub round_win: &'static [&'static str],
    pub round_loss: &'static [&'static str],
    pub round_tie: &'static [&'static str],
    pub opponent_bust: &'static [&'static str],
    pub player_bust: &'static [&'static str],
    pub match_win: &'static [&'static str],
    pub match_loss: &'static [&'static str],
    /// Spec 030: a match (or the venue) mid-series. ≥3 each, so the venue line
    /// and the match start after it always have an alternative.
    pub leading: &'static [&'static str],
    pub trailing: &'static [&'static str],
    pub decider: &'static [&'static str],
    /// Level at 1–1 in a best of 5 — only the final opponent reaches it, so
    /// only its voice carries lines (≥3); every other voice's is empty.
    pub all_square: &'static [&'static str],
    /// The match that ends a series: the opponent took it / lost it. ≥2 each.
    pub series_won: &'static [&'static str],
    pub series_lost: &'static [&'static str],
}

/// The line pool for `ev` within `set`.
pub fn lines_for(set: &'static BanterSet, ev: BanterEvent) -> &'static [&'static str] {
    match ev {
        BanterEvent::MatchStart => set.match_start,
        BanterEvent::RoundWin => set.round_win,
        BanterEvent::RoundLoss => set.round_loss,
        BanterEvent::RoundTie => set.round_tie,
        BanterEvent::OpponentBust => set.opponent_bust,
        BanterEvent::PlayerBust => set.player_bust,
        BanterEvent::MatchWin => set.match_win,
        BanterEvent::MatchLoss => set.match_loss,
    }
}

/// Where a series stands, from the opponent's side (spec 030).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesState {
    Opening,
    Leading,
    Trailing,
    AllSquare,
    Decider,
}

/// Opening at 0–0; Leading / Trailing when the opponent has more / fewer wins;
/// Decider when both are one win from `wins_needed`; AllSquare when level
/// otherwise (only a best of 5 at 1–1).
pub fn series_state(series: &Series) -> SeriesState {
    let (player, opponent) = (series.player_wins, series.opponent_wins);
    if player == 0 && opponent == 0 {
        SeriesState::Opening
    } else if opponent > player {
        SeriesState::Leading
    } else if opponent < player {
        SeriesState::Trailing
    } else if opponent + 1 == wins_needed(&series.opponent) {
        SeriesState::Decider
    } else {
        SeriesState::AllSquare
    }
}

/// The pool a match start or a venue arrival draws from (spec 030): the
/// greetings with no series or at Opening — exactly today's — else the
/// state's own lines.
pub fn start_lines(set: &'static BanterSet, state: Option<SeriesState>) -> &'static [&'static str] {
    match state {
        None | Some(SeriesState::Opening) => set.match_start,
        Some(SeriesState::Leading) => set.leading,
        Some(SeriesState::Trailing) => set.trailing,
        Some(SeriesState::AllSquare) => set.all_square,
        Some(SeriesState::Decider) => set.decider,
    }
}

/// The pool for `ev` (spec 030): a match end that decided a series draws
/// series won (the opponent's `MatchWin`) or series lost (`MatchLoss`);
/// every other case is `lines_for`, unchanged.
pub fn lines_in_series(set: &'static BanterSet, ev: BanterEvent, decided: bool) -> &'static [&'static str] {
    match (ev, decided) {
        (BanterEvent::MatchWin, true) => set.series_won,
        (BanterEvent::MatchLoss, true) => set.series_lost,
        _ => lines_for(set, ev),
    }
}

/// A random line from `lines`, never equal to `last` when `lines.len() > 1`
/// (the no-back-to-back-repeat rule, spec 017 §4). A single-line pool returns
/// that sole line. Pure — `rng` is injected so the choice is unit-testable.
pub fn pick(lines: &[&'static str], last: Option<&str>, rng: &mut impl rand::Rng) -> &'static str {
    if lines.len() <= 1 {
        return lines[0];
    }
    loop {
        let candidate = lines[rng.random_range(0..lines.len())];
        if Some(candidate) != last {
            return candidate;
        }
    }
}

/// The number of words in `line`: its maximal runs of non-space characters.
pub fn word_count(line: &str) -> usize {
    line.split(' ').filter(|w| !w.is_empty()).count()
}

/// `line` with only its first `words` words showing — each later word's
/// characters blanked to spaces — so the text always has the finished line's
/// length and every shown word sits where it will in the finished line,
/// whatever centres it (spec 030, AC 8). `words >= word_count` is the line.
pub fn revealed(line: &str, words: usize) -> String {
    let mut seen = 0;
    let mut prev_space = true;
    line.chars()
        .map(|c| {
            if c == ' ' {
                prev_space = true;
                return ' ';
            }
            if prev_space {
                seen += 1;
                prev_space = false;
            }
            if seen <= words { c } else { ' ' }
        })
        .collect()
}

/// Words showing `elapsed` after the line was chosen: the first at once, one
/// more each WORD_STEP_MS, never more than the line has.
pub fn words_due(line: &str, elapsed: Duration) -> usize {
    let steps = (elapsed.as_millis() / WORD_STEP_MS as u128) as usize;
    steps.saturating_add(1).min(word_count(line))
}

/// A line being spoken (spec 030): the line, the time since its beat ended
/// (since it was chosen, when there is no beat), and how many of its words are
/// showing. Drawing state only — never saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speech {
    line: &'static str,
    elapsed: Duration,
    shown: usize,
    wait: Duration,
}

impl Speech {
    /// Just chosen: its first word showing, or — `animated` false, the
    /// Animations setting Off — every word at once. The caller plays the one
    /// burble the first appearance owes.
    pub fn new(line: &'static str, animated: bool) -> Self {
        let shown = if animated { words_due(line, Duration::ZERO) } else { word_count(line) };
        Self { line, elapsed: Duration::ZERO, shown, wait: Duration::ZERO }
    }

    /// The same line, but nothing shows and nothing is owed until `wait` has
    /// passed (ruling 11A: a line answering an event waits out the event's
    /// sound). `after(Duration::ZERO)` is `self`.
    pub fn after(self, wait: Duration) -> Self {
        Self { wait, ..self }
    }

    /// Advance by `dt`. True when a word appeared on this step, meaning the caller
    /// owes one burble. A step that crosses more than one word boundary shows
    /// them all and still returns true once (plan §Design tension 3).
    /// `shown` never decreases, so a settled speech stays settled.
    /// A line waiting out its beat shows nothing and owes nothing until the step
    /// that ends the beat, which owes one burble.
    pub fn advance(&mut self, dt: Duration) -> bool {
        if !self.wait.is_zero() {
            if dt < self.wait {
                self.wait -= dt;
                return false;
            }
            self.elapsed = self.elapsed.saturating_add(dt - self.wait);
            self.wait = Duration::ZERO;
            self.shown = self.shown.max(words_due(self.line, self.elapsed));
            return true;
        }
        self.elapsed = self.elapsed.saturating_add(dt);
        let due = words_due(self.line, self.elapsed);
        if due > self.shown {
            self.shown = due;
            true
        } else {
            false
        }
    }

    /// Show the rest at once, silently — the line's screen was left.
    pub fn settle(&mut self) {
        self.wait = Duration::ZERO;
        self.shown = word_count(self.line);
    }

    pub fn words_shown(&self) -> usize {
        if self.wait.is_zero() { self.shown } else { 0 }
    }

    /// What the panel draws: `revealed(line, words_shown())`.
    pub fn text(&self) -> String {
        revealed(self.line, self.words_shown())
    }
}

/// The neutral, characterless voice — used for the default opponent and as the
/// fallback for any unknown id. Never blank, never a roster voice.
const GENERIC: BanterSet = BanterSet {
    match_start: &["Let's play.", "Ready to begin.", "Shall we?"],
    round_win: &["That one's mine.", "A point to me.", "My round.", "One for me."],
    round_loss: &["Your round.", "Take it.", "You win that.", "Point to you."],
    round_tie: &["A draw.", "Dead even.", "All square.", "No winner."],
    opponent_bust: &["Too far.", "Over I go.", "I went over.", "Past twenty."],
    player_bust: &["You went over.", "Busted.", "You busted.", "Past the line."],
    match_win: &["I win.", "The game's mine."],
    match_loss: &["Well played.", "You take it."],
    leading: &["I'm ahead.", "One up on you.", "I lead the series."],
    trailing: &["You're ahead.", "I'm behind.", "You lead the series."],
    decider: &["One match left.", "This one decides it.", "Winner takes all."],
    all_square: &[],
    series_won: &["The series is mine.", "I take the series."],
    series_lost: &["The series is yours.", "You take the series."],
};

/// Greeb (Rookie) — green and eager: jittery, over-excited, rattled and
/// flustered when he loses, and yelps when he busts. Surprised by his own wins.
const GREEB: BanterSet = BanterSet {
    match_start: &["Here goes nothing!", "Okay, okay, ready!", "Deep breath, Greeb."],
    round_win: &["I got one?! Whoa!", "Did that just work?", "Yes! Ha, it worked!", "I-I actually did it!"],
    round_loss: &["Aw, shucks.", "You're really good!", "Dang it, so close!", "G-good one, ah!"],
    round_tie: &["A tie? Phew!", "Even? Okay, okay!", "W-we match! Wow.", "Both stuck! Yikes."],
    opponent_bust: &["Yikes, too much!", "No no no, over!", "Aah, I blew it!", "Too far, too far!"],
    player_bust: &["Oh! You popped!", "Phew, not me!", "Eep, you went over!", "Y-you busted! Wow."],
    match_win: &["I actually won?!", "Me? I won! Wow!"],
    match_loss: &["Aw, well played.", "Gosh, good game."],
    leading: &["I'm ahead?! Really?", "Wait, I'm winning?", "Ahead?! No jinxing!"],
    trailing: &["Uh-oh, uh-oh.", "I-I can catch up!", "Still time! Right?"],
    decider: &["Last one?! Gulp.", "All or nothing, eek!", "Hands... shaking!"],
    all_square: &[],
    series_won: &["I won it all?! Me?!", "The whole series?!"],
    series_lost: &["Aw, that's the set.", "You won it all, wow."],
};

/// Dax Runo (Greenhorn) — a cocky kid: brash trash-talk, struts on a win, makes
/// excuses when it goes wrong, and busts big.
const DAX: BanterSet = BanterSet {
    match_start: &["Watch and learn.", "This'll be quick.", "Sit tight, rookie."],
    round_win: &["Too easy.", "Boom. Called it.", "Get used to it.", "All day, kid."],
    round_loss: &["Lucky. So lucky.", "Bah, whatever.", "That's a fluke!", "Won't happen twice."],
    round_tie: &["Tch, a tie.", "Even? Boring.", "Push. Yawn.", "Nobody wins? Lame."],
    opponent_bust: &["Ah, come on!", "Blew it big time.", "Rigged, I swear!", "Ugh, one bad move!"],
    player_bust: &["Ha! Nice one, kid.", "Told you. Splat.", "Amateur hour.", "Whoops. Busted."],
    match_win: &["Not even close.", "Told ya so, kid."],
    match_loss: &["Rematch. Right now.", "Best of nine?!"],
    leading: &["Scoreboard, kid.", "Already up. Yawn.", "Just getting warm."],
    trailing: &["Warming up, is all.", "Just a slow start.", "Charity, that's all."],
    decider: &["Winner takes it. Me.", "Last one. Easy.", "Now I get serious."],
    all_square: &[],
    series_won: &["Series? Mine. Duh.", "Called the series."],
    series_lost: &["Stacked deck, man!", "Off day. Whatever."],
};

/// Vessa Korr (Scrapper) — street-hard and defiant: takes the hit and swings
/// back, never done even when she's down.
const VESSA: BanterSet = BanterSet {
    match_start: &["Come on then.", "Let's scrap.", "Bring it, then."],
    round_win: &["That's how.", "Chalk it up.", "Hit and done.", "One in the bank."],
    round_loss: &["Fine. I'm not done.", "You'll pay for that.", "Cheap shot.", "Damn, you're quick."],
    round_tie: &["Nobody blinks.", "Locked up tight.", "Toe to toe.", "Neither backs off."],
    opponent_bust: &["Pushed too hard.", "Ah, hell.", "Overswung it.", "My own damn fault."],
    player_bust: &["Ha, you cracked.", "Down you go.", "Glass jaw.", "That's a knockout."],
    match_win: &["Still standing.", "You're done, kid."],
    match_loss: &["Next time's mine.", "This ain't over."],
    leading: &["Up and swinging.", "Stay down this time.", "On the ropes, kid."],
    trailing: &["Down ain't out.", "Knocked, not out.", "Try that again."],
    decider: &["One more scrap.", "Winner walks away.", "All in, right now."],
    all_square: &[],
    series_won: &["Last one standing.", "Whole fight's mine."],
    series_lost: &["Beat me fair. Once.", "I'll be back for it."],
};

/// Nima Sarn (Broker) — cool and mercantile: everything is a transaction, wins
/// are profit, losses a minor cost. Never ruffled.
const NIMA: BanterSet = BanterSet {
    match_start: &["Let's talk terms.", "Credits on the line.", "Open the books."],
    round_win: &["Profit.", "Into the ledger.", "Pure margin.", "Paid in full."],
    round_loss: &["A minor cost.", "I'll recoup it.", "Write it off.", "A slim loss."],
    round_tie: &["We break even.", "No margin lost.", "Balanced books.", "A wash."],
    opponent_bust: &["Overspent.", "A bad investment.", "Overleveraged.", "Bought too high."],
    player_bust: &["That'll cost you.", "Poor accounting.", "I own you now.", "Debt collected."],
    match_win: &["I always collect.", "A tidy return."],
    match_loss: &["I've paid worse.", "A rare deficit."],
    leading: &["Ahead on the books.", "Interest accrues.", "Up on the ledger."],
    trailing: &["A temporary debt.", "Short-term loss.", "Borrowed luck."],
    decider: &["Final settlement.", "All accounts due.", "Time to settle up."],
    all_square: &[],
    series_won: &["Account closed.", "Contract fulfilled."],
    series_lost: &["Cost of business.", "I'll bill you later."],
};

/// Old Toran (Veteran) — dry, calm, wry: he's seen it all, understated, with a
/// faint teaching tone.
const TORAN: BanterSet = BanterSet {
    match_start: &["Sit. Let's play.", "Been at this awhile.", "Let's see it, then."],
    round_win: &["Patience wins.", "As it goes.", "Slow and sure.", "Old habits, boy."],
    round_loss: &["Nicely done.", "You've got an eye.", "Sharp, that.", "Not bad. Not bad."],
    round_tie: &["Even hands.", "Happens.", "So it goes.", "A fair split."],
    opponent_bust: &["Ah, greedy of me.", "Should've held.", "Too eager, old man.", "My mistake."],
    player_bust: &["Over you go.", "Reached too far.", "One too many, hm?", "Patience, lad."],
    match_win: &["Age and cunning.", "Years still tell."],
    match_loss: &["Well earned, that.", "You've learned well."],
    leading: &["Ahead, for now.", "Steady does it, lad.", "Mind the score, boy."],
    trailing: &["Learning fast, eh?", "Hm. You're ahead.", "Long game, lad."],
    decider: &["Last hand. Breathe.", "Now we find out.", "One more. Steady."],
    all_square: &[],
    series_won: &["The old way wins.", "Still got it, then."],
    series_lost: &["Taught you too well.", "Your series. Fairly."],
};

/// Brakka (Bruiser) — big, booming brute: blunt bravado, dares and taunts, and
/// laughs off his own busts.
const BRAKKA: BanterSet = BanterSet {
    match_start: &["Try to keep up!", "Sit down, small fry!", "Fists up, runt!"],
    round_win: &["Crushed it!", "Ha! Feel that?!", "Boom! Down ya go!", "Squashed ya flat!"],
    round_loss: &["Pah, a scratch!", "Enjoy it, runt.", "Barely felt that.", "Grr, cheap hit!"],
    round_tie: &["A standoff!", "Nobody flinched!", "Head to head!", "Two rocks clash!"],
    opponent_bust: &["Bah! Too greedy!", "Ha! Blew right past!", "Whoops, big swing!", "Overdid it! Haha!"],
    player_bust: &["HA! Splat!", "Too big for ya!", "Smashed to bits!", "Flattened ya!"],
    match_win: &["Smashed ya to bits!", "Ha! Timber!"],
    match_loss: &["Bah! You got lucky!", "Grr! Again! Again!"],
    leading: &["Ha! Out front!", "Stay down, runt!", "Ya feelin' small?!"],
    trailing: &["Just a warmup!", "Now I'm angry!", "Bah! Lucky punches!"],
    decider: &["Last smash! Ready?!", "Winner smashes all!", "One more brawl!"],
    all_square: &[],
    series_won: &["Smashed the lot!", "BRAKKA WINS ALL!"],
    series_lost: &["Bah! Ya beat Brakka!", "Grr... fair fight."],
};

/// Rix Vandal (Ace) — precise and clinical: talks in odds and math, arrogant,
/// with open disdain for sloppy play.
const RIX: BanterSet = BanterSet {
    match_start: &["The odds favor me.", "Precision wins.", "Run the numbers."],
    round_win: &["Calculated.", "As predicted.", "Within tolerance.", "Exactly to plan."],
    round_loss: &["A rounding error.", "Statistically rare.", "An outlier.", "Noise. Nothing more."],
    round_tie: &["A null result.", "Precisely even.", "Zero net.", "A dead heat."],
    opponent_bust: &["Miscalculated.", "An error. Rare.", "Off by a margin.", "A slight overshoot."],
    player_bust: &["Predictable.", "Sloppy. Pitiful.", "Amateur variance.", "You were the error."],
    match_win: &["The math held.", "Odds confirmed."],
    match_loss: &["A variance. Once.", "Improbable. Yet."],
    leading: &["Trend confirmed.", "The curve favors me.", "Ahead, as modeled."],
    trailing: &["Sample too small.", "Regression looms.", "Temporary deviation."],
    decider: &["Final data point.", "Fifty-fifty. Fine.", "The last variable."],
    all_square: &[],
    series_won: &["Series: as computed.", "Proof complete."],
    series_lost: &["Model... revised.", "An anomaly. Noted."],
};

/// Kesh Varn (Duelist) — sharp and dangerous: a duelist's menace and honor,
/// clipped threats, blade imagery.
const KESH: BanterSet = BanterSet {
    match_start: &["Blades out.", "Guard yourself.", "En garde."],
    round_win: &["First blood.", "A clean cut.", "Through the guard.", "To the heart."],
    round_loss: &["A fair touch.", "Well struck.", "A worthy parry.", "Point to your blade."],
    round_tie: &["Blade to blade.", "We cross even.", "Locked guards.", "Steel on steel."],
    opponent_bust: &["My edge slipped.", "Cut too deep.", "Overextended.", "A wild swing."],
    player_bust: &["You overreached.", "Your guard broke.", "One breath left.", "Disarmed."],
    match_win: &["The edge was mine.", "First to the kill."],
    match_loss: &["A worthy blade.", "Sharper than most."],
    leading: &["You bleed already.", "Your guard weakens.", "One more cut."],
    trailing: &["A scratch. Nothing.", "Now I draw steel.", "You've marked me."],
    decider: &["The final bout.", "Sudden death.", "To the last blade."],
    all_square: &[],
    series_won: &["The duel is mine.", "Sheathe your blade."],
    series_lost: &["You won the duel.", "I yield. Honorably."],
};

/// The Magistrate (Master) — imperious cold authority: pronounces rather than
/// talks, in the language of law, judgment, and sentence.
const MAGISTRATE: BanterSet = BanterSet {
    match_start: &["Court is in session.", "State your case.", "Order. We begin."],
    round_win: &["So ruled.", "The verdict stands.", "Case closed.", "By my authority."],
    round_loss: &["Noted for appeal.", "A minor objection.", "Struck from record.", "The court concedes."],
    round_tie: &["Case adjourned.", "No ruling yet.", "A hung jury.", "We recess."],
    opponent_bust: &["I overstepped.", "A misjudgment.", "Beyond my writ.", "An overreach."],
    player_bust: &["Condemned.", "Sentence: bust.", "Contempt of court.", "The line was law."],
    match_win: &["The law prevails.", "Justice is served."],
    match_loss: &["An odd verdict.", "Appeal granted."],
    leading: &["The evidence mounts.", "Precedent is mine.", "Your case weakens."],
    trailing: &["Objection noted.", "The trial continues.", "Pending review."],
    decider: &["Final arguments.", "The jury decides.", "Closing statements."],
    all_square: &[],
    series_won: &["Guilty as charged.", "Sentence is passed."],
    series_lost: &["Case dismissed.", "Acquitted. Go."],
};

/// The Sovereign (Kingpin) — regal and glacial: minimal words, utterly
/// untouchable, the house always wins; even the rare loss is dismissed.
const SOVEREIGN: BanterSet = BanterSet {
    match_start: &["Begin.", "The house awaits.", "You may sit."],
    round_win: &["Naturally.", "As it must be.", "Inevitable.", "The house rises."],
    round_loss: &["Amusing.", "A trifle.", "Fleeting.", "Savor it."],
    round_tie: &["Inconsequential.", "It matters not.", "Meaningless.", "A pause, merely."],
    opponent_bust: &["A rare indulgence.", "How careless.", "A slip. Once.", "Beneath my custom."],
    player_bust: &["You were warned.", "Beneath me.", "A mercy, granted.", "The house owns you."],
    match_win: &["The house wins.", "It was never yours."],
    match_loss: &["Enjoy it. Briefly.", "A rounding, no more."],
    leading: &["As expected.", "The end nears.", "Already decided."],
    trailing: &["Tolerable.", "A loan, merely.", "I permit this."],
    decider: &["The final hand.", "Let us conclude.", "At last."],
    all_square: &["Level. For now.", "Balance. Brief.", "Even. Irrelevant."],
    series_won: &["The house takes all.", "Kneel."],
    series_lost: &["You may leave.", "Take it. Go."],
};

/// The voice for opponent `id`: the roster set, or [`GENERIC`] for `"default"`
/// and any unknown id.
pub fn banter_for(id: &str) -> &'static BanterSet {
    match id {
        "greeb" => &GREEB,
        "dax" => &DAX,
        "vessa" => &VESSA,
        "nima" => &NIMA,
        "toran" => &TORAN,
        "brakka" => &BRAKKA,
        "rix" => &RIX,
        "kesh" => &KESH,
        "magistrate" => &MAGISTRATE,
        "sovereign" => &SOVEREIGN,
        _ => &GENERIC,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(
        o_bust: bool,
        p_bust: bool,
        outcome: Option<RoundOutcome>,
        game_over: bool,
        opp_won_game: bool,
    ) -> BanterSnapshot {
        BanterSnapshot {
            o_bust,
            p_bust,
            outcome,
            game_over,
            opp_won_game,
            player_engaged: false,
        }
    }

    const EMPTY: BanterSnapshot = BanterSnapshot {
        o_bust: false,
        p_bust: false,
        outcome: None,
        game_over: false,
        opp_won_game: false,
        player_engaged: false,
    };

    #[test]
    fn game_over_beats_co_firing_bust_and_outcome() {
        // The final blow: opponent-loses-match-by-busting surfaces game_over,
        // the bust, and the (player-won) outcome in one tick. Match end wins.
        let curr = snap(true, false, Some(RoundOutcome::PlayerWon), true, false);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::MatchLoss));

        // Opponent wins the match on the final round.
        let curr = snap(false, true, Some(RoundOutcome::OpponentWon), true, true);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::MatchWin));
    }

    #[test]
    fn bust_beats_the_outcome_it_causes() {
        // Opponent busts, losing the (non-final) round: reads the bust, not RoundLoss.
        let curr = snap(true, false, Some(RoundOutcome::PlayerWon), false, false);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::OpponentBust));

        // Player busts, losing the round to the opponent: reads the bust, not RoundWin.
        let curr = snap(false, true, Some(RoundOutcome::OpponentWon), false, false);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::PlayerBust));
    }

    #[test]
    fn outcome_maps_to_the_right_round_event() {
        assert_eq!(
            banter_event(&EMPTY, &snap(false, false, Some(RoundOutcome::OpponentWon), false, false)),
            Some(BanterEvent::RoundWin)
        );
        assert_eq!(
            banter_event(&EMPTY, &snap(false, false, Some(RoundOutcome::PlayerWon), false, false)),
            Some(BanterEvent::RoundLoss)
        );
        assert_eq!(
            banter_event(&EMPTY, &snap(false, false, Some(RoundOutcome::Tied), false, false)),
            Some(BanterEvent::RoundTie)
        );
    }

    #[test]
    fn stale_or_no_change_yields_none() {
        // No change at all.
        assert_eq!(banter_event(&EMPTY, &EMPTY), None);
        // Outcome already set in prev (round already resolved) — not newly set.
        let resolved = snap(false, false, Some(RoundOutcome::OpponentWon), false, false);
        assert_eq!(banter_event(&resolved, &resolved), None);
        // Bust already true in prev — not new.
        let busted = snap(true, false, None, false, false);
        assert_eq!(banter_event(&busted, &busted), None);
        // Game-over already true in prev — not new.
        let over = snap(false, false, None, true, true);
        assert_eq!(banter_event(&over, &over), None);
    }

    #[test]
    fn play_resumed_only_on_false_to_true_engaged_transition() {
        let idle = BanterSnapshot { player_engaged: false, ..EMPTY };
        let engaged = BanterSnapshot { player_engaged: true, ..EMPTY };
        // false -> true: the player's first action of a round.
        assert!(play_resumed(&idle, &engaged));
        // No transition in either steady state, and no clear on disengage.
        assert!(!play_resumed(&idle, &idle));
        assert!(!play_resumed(&engaged, &engaged));
        assert!(!play_resumed(&engaged, &idle));
    }

    #[test]
    fn match_restarted_only_on_game_over_true_to_false_transition() {
        let over = BanterSnapshot { game_over: true, ..EMPTY };
        let live = BanterSnapshot { game_over: false, ..EMPTY };
        // true -> false: a rematch begins in place.
        assert!(match_restarted(&over, &live));
        // No restart in either steady state, and none when a match just ended.
        assert!(!match_restarted(&live, &live));
        assert!(!match_restarted(&over, &over));
        assert!(!match_restarted(&live, &over));
    }

    #[test]
    fn of_player_engaged_reflects_the_players_first_action() {
        use crate::card::{Card, PlayedCard};
        use crate::game::GameState;

        // Pristine round start: no dealer card drawn, nothing played, not stood.
        let gs = GameState::new();
        assert!(!BanterSnapshot::of(&gs).player_engaged);

        // After a hit (a dealer card on the table).
        let mut hit = GameState::new();
        hit.player.dealer_row.push(PlayedCard { card: Card::Dealer(5), value: 5 });
        assert!(BanterSnapshot::of(&hit).player_engaged);

        // After playing a side card.
        let mut played = GameState::new();
        played.player.played_row.push(PlayedCard { card: Card::Plus(3), value: 3 });
        assert!(BanterSnapshot::of(&played).player_engaged);

        // After standing.
        let mut stood = GameState::new();
        stood.player.stood = true;
        assert!(BanterSnapshot::of(&stood).player_engaged);
    }

    #[test]
    fn pick_never_repeats_last_for_multi_line_pool() {
        let pool = GENERIC.round_win;
        assert!(pool.len() >= 2);
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let got = pick(pool, Some(pool[0]), &mut rng);
            assert_ne!(got, pool[0]);
        }
    }

    #[test]
    fn pick_returns_sole_line_for_single_line_pool() {
        // Every real pool now carries >= 2 lines (T005e), so exercise `pick`'s
        // single-line branch with a local one-line pool.
        let pool: &[&'static str] = &["Only line."];
        assert_eq!(pool.len(), 1);
        let mut rng = rand::rng();
        // Even asked to avoid the only line, it returns it.
        assert_eq!(pick(pool, Some(pool[0]), &mut rng), pool[0]);
    }

    #[test]
    fn generic_classes_non_empty_and_repeatables_have_floor() {
        let g = &GENERIC;
        for class in [
            g.match_start,
            g.round_win,
            g.round_loss,
            g.round_tie,
            g.opponent_bust,
            g.player_bust,
            g.match_win,
            g.match_loss,
        ] {
            assert!(class.len() >= 2, "GENERIC class must carry >= 2 lines");
        }
        for class in [g.round_win, g.round_loss, g.round_tie, g.opponent_bust, g.player_bust] {
            assert!(class.len() >= 3, "repeatable class must carry >= 3 lines");
        }
    }

    #[test]
    fn every_generic_line_fits_the_panel() {
        use crate::portrait::BANTER_MAX_WIDTH;
        let g = &GENERIC;
        for class in [
            g.match_start,
            g.round_win,
            g.round_loss,
            g.round_tie,
            g.opponent_bust,
            g.player_bust,
            g.match_win,
            g.match_loss,
        ] {
            for line in class {
                assert!(
                    line.chars().count() <= BANTER_MAX_WIDTH,
                    "line {line:?} exceeds BANTER_MAX_WIDTH"
                );
            }
        }
    }

    #[test]
    fn banter_for_default_and_unknown_resolve_to_generic() {
        // `default` and any unknown id both resolve to GENERIC (compared by
        // content — BanterSet isn't PartialEq, and a `const`'s address isn't
        // stable across reference sites). match_start is a sufficient proxy.
        assert_eq!(banter_for("default").match_start, GENERIC.match_start);
        assert_eq!(banter_for("nonexistent-id").match_start, GENERIC.match_start);
        // The placeholder roster id resolves to a different voice.
        assert_ne!(banter_for("greeb").match_start, GENERIC.match_start);
    }

    // ---- T002: the ten roster voices --------------------------------------

    /// The ten roster ids, in tasks.md order.
    const ROSTER_IDS: [&str; 10] = [
        "greeb", "dax", "vessa", "nima", "toran", "brakka", "rix", "kesh",
        "magistrate", "sovereign",
    ];

    /// All eight classes of a set, in a fixed order.
    fn classes(set: &'static BanterSet) -> [&'static [&'static str]; 8] {
        [
            set.match_start,
            set.round_win,
            set.round_loss,
            set.round_tie,
            set.opponent_bust,
            set.player_bust,
            set.match_win,
            set.match_loss,
        ]
    }

    /// Spec 030's six series pools, in a fixed order.
    fn series_classes(set: &'static BanterSet) -> [&'static [&'static str]; 6] {
        [
            set.leading,
            set.trailing,
            set.decider,
            set.all_square,
            set.series_won,
            set.series_lost,
        ]
    }

    /// Every pool of a set: the eight classes followed by the six series pools.
    fn all_pools(set: &'static BanterSet) -> [&'static [&'static str]; 14] {
        let [a, b, c, d, e, f, g, h] = classes(set);
        let [i, j, k, l, m, n] = series_classes(set);
        [a, b, c, d, e, f, g, h, i, j, k, l, m, n]
    }

    /// The five *repeatable* classes (must carry >= 2 distinct lines).
    fn repeatable(set: &'static BanterSet) -> [&'static [&'static str]; 5] {
        [set.round_win, set.round_loss, set.round_tie, set.opponent_bust, set.player_bust]
    }

    /// GENERIC plus the ten roster sets — every set the game can voice.
    fn all_sets() -> Vec<(&'static str, &'static BanterSet)> {
        let mut v = vec![("default", &GENERIC as &'static BanterSet)];
        for id in ROSTER_IDS {
            v.push((id, banter_for(id)));
        }
        v
    }

    /// A concatenation proxy for whole-set equality (BanterSet isn't PartialEq).
    fn set_fingerprint(set: &'static BanterSet) -> String {
        classes(set).iter().flat_map(|c| c.iter().copied()).collect::<Vec<_>>().join("|")
    }

    #[test]
    fn every_line_of_every_set_fits_the_panel() {
        use crate::portrait::BANTER_MAX_WIDTH;
        for (id, set) in all_sets() {
            for class in all_pools(set) {
                for line in class {
                    assert!(
                        line.chars().count() <= BANTER_MAX_WIDTH,
                        "{id}: line {line:?} exceeds BANTER_MAX_WIDTH"
                    );
                }
            }
        }
    }

    #[test]
    fn every_set_non_empty_with_repeatable_floor() {
        for (id, set) in all_sets() {
            for class in classes(set) {
                assert!(class.len() >= 2, "{id}: every class must carry >= 2 lines");
            }
            for class in repeatable(set) {
                assert!(class.len() >= 3, "{id}: repeatable class must carry >= 3 lines");
            }
        }
    }

    #[test]
    fn every_class_of_every_set_has_distinct_lines() {
        // Carried from the T001 review: `pick` loops forever on an all-equal
        // pool, so no class may hold a duplicate variant.
        for (id, set) in all_sets() {
            for class in all_pools(set) {
                for i in 0..class.len() {
                    for j in (i + 1)..class.len() {
                        assert_ne!(
                            class[i], class[j],
                            "{id}: duplicate line {:?} within a class",
                            class[i]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn roster_sets_are_pairwise_distinct_and_differ_from_generic() {
        let generic = set_fingerprint(&GENERIC);
        let roster: Vec<(&str, String)> =
            ROSTER_IDS.iter().map(|&id| (id, set_fingerprint(banter_for(id)))).collect();
        for (id, fp) in &roster {
            assert_ne!(*fp, generic, "{id} matches GENERIC");
        }
        for i in 0..roster.len() {
            for j in (i + 1)..roster.len() {
                assert_ne!(
                    roster[i].1, roster[j].1,
                    "{} and {} share a voice",
                    roster[i].0, roster[j].0
                );
            }
        }
    }

    #[test]
    fn no_line_is_shared_across_any_two_sets() {
        // Stronger than fingerprint distinctness: no individual line may recur
        // in another set (including GENERIC).
        let sets = all_sets();
        for a in 0..sets.len() {
            for b in (a + 1)..sets.len() {
                let (id_a, set_a) = sets[a];
                let (id_b, set_b) = sets[b];
                for class_a in all_pools(set_a) {
                    for line in class_a {
                        for class_b in all_pools(set_b) {
                            assert!(
                                !class_b.contains(line),
                                "{id_a} and {id_b} share the line {line:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn banter_for_maps_each_roster_id_to_a_non_generic_set() {
        let generic = set_fingerprint(&GENERIC);
        for id in ROSTER_IDS {
            assert_ne!(
                set_fingerprint(banter_for(id)),
                generic,
                "{id} resolves to GENERIC"
            );
        }
        assert_eq!(set_fingerprint(banter_for("default")), generic);
        assert_eq!(set_fingerprint(banter_for("nonexistent-id")), generic);
    }

    // ---- Spec 030 T001: the reveal ----------------------------------------

    fn step() -> Duration {
        Duration::from_millis(WORD_STEP_MS)
    }

    #[test]
    fn the_word_step_is_about_a_fifth_of_a_second() {
        assert!(150 <= WORD_STEP_MS);
        assert!(WORD_STEP_MS <= 250);
    }

    #[test]
    fn every_line_finishes_inside_a_second() {
        for (id, set) in all_sets() {
            for class in all_pools(set) {
                for line in class {
                    let words = word_count(line) as u64;
                    assert!(
                        words.saturating_sub(1) * WORD_STEP_MS < 1000,
                        "{id}: line {line:?} takes a second or more to say"
                    );
                }
            }
        }
    }

    #[test]
    fn a_line_is_revealed_in_place_word_by_word() {
        for (id, set) in all_sets() {
            for class in all_pools(set) {
                for &line in class {
                    let total = word_count(line);
                    let chars: Vec<char> = line.chars().collect();
                    for n in 0..=total {
                        let shown = revealed(line, n);
                        let got: Vec<char> = shown.chars().collect();
                        assert_eq!(got.len(), chars.len(), "{id}: {line:?} at {n} changed length");
                        for (i, &c) in got.iter().enumerate() {
                            assert!(c == chars[i] || c == ' ', "{id}: {line:?} at {n}: stray {c:?}");
                        }
                        assert_eq!(word_count(&shown), n, "{id}: {line:?} at {n}: word count");
                        // The first n words' characters sit where the line has them.
                        let mut word = 0;
                        let mut prev_space = true;
                        for (i, &c) in chars.iter().enumerate() {
                            if c != ' ' && prev_space {
                                word += 1;
                            }
                            prev_space = c == ' ';
                            if c != ' ' && word <= n {
                                assert_eq!(got[i], c, "{id}: {line:?} at {n}: col {i} moved");
                            }
                        }
                    }
                    assert_eq!(revealed(line, total), line, "{id}: {line:?} not whole at the end");
                }
            }
        }
    }

    #[test]
    fn a_spoken_line_shows_a_word_per_step_from_the_first_frame() {
        let line = "Here goes nothing!";
        let mut s = Speech::new(line, true);
        assert_eq!(s.words_shown(), 1);
        assert_eq!(s.text(), "Here              ");

        assert!(!s.advance(step() - Duration::from_millis(1)));
        assert_eq!(s.words_shown(), 1);
        assert!(s.advance(Duration::from_millis(1)));
        assert_eq!(s.words_shown(), 2);
        assert_eq!(s.text(), "Here goes         ");

        assert!(s.advance(step()));
        assert_eq!(s.words_shown(), 3);
        assert_eq!(s.text(), line);

        for _ in 0..20 {
            assert!(!s.advance(step()));
        }
        assert_eq!(s.text(), line);
    }

    #[test]
    fn animations_off_speaks_the_whole_line_and_owes_no_more_burbles() {
        let line = "Here goes nothing!";
        let mut s = Speech::new(line, false);
        assert_eq!(s.words_shown(), word_count(line));
        assert_eq!(s.text(), line);
        for _ in 0..20 {
            assert!(!s.advance(step()));
        }
        assert_eq!(s.text(), line);
    }

    #[test]
    fn a_settled_line_is_whole_and_silent() {
        let line = "Here goes nothing!";
        let mut s = Speech::new(line, true);
        assert!(s.advance(step()));
        assert_eq!(s.words_shown(), 2);
        s.settle();
        assert_eq!(s.words_shown(), word_count(line));
        assert_eq!(s.text(), line);
        for _ in 0..20 {
            assert!(!s.advance(step()));
        }
        assert_eq!(s.text(), line);
    }

    #[test]
    fn a_stalled_step_shows_every_due_word_and_owes_one_burble() {
        let line = "Court is in session.";
        assert_eq!(word_count(line), 4);
        let mut s = Speech::new(line, true);
        assert_eq!(s.words_shown(), 1);
        assert!(s.advance(step() * 3));
        assert_eq!(s.words_shown(), 4);
        assert_eq!(s.text(), line);
        assert!(!s.advance(Duration::ZERO));
        assert!(!s.advance(step()));
    }

    // ---- Spec 030 T003a: the event beat (ruling 11A) ----------------------

    fn beat() -> Duration {
        Duration::from_millis(crate::EVENT_BEAT_MS)
    }

    #[test]
    fn the_event_beat_is_about_a_third_of_a_second() {
        assert!(300 <= crate::EVENT_BEAT_MS);
        assert!(crate::EVENT_BEAT_MS <= 400);
    }

    #[test]
    fn a_line_after_a_beat_shows_nothing_until_the_beat_ends() {
        let line = "Here goes nothing!";
        let mut s = Speech::new(line, true).after(beat());
        assert_eq!(s.words_shown(), 0);
        assert_eq!(s.text(), " ".repeat(line.chars().count()));

        assert!(!s.advance(beat() - Duration::from_millis(1)));
        assert_eq!(s.words_shown(), 0);
        assert_eq!(s.text(), " ".repeat(line.chars().count()));

        assert!(s.advance(Duration::from_millis(1)));
        assert_eq!(s.words_shown(), 1);
        assert_eq!(s.text(), "Here              ");

        assert!(s.advance(step()));
        assert_eq!(s.words_shown(), 2);
        assert_eq!(s.text(), "Here goes         ");
    }

    #[test]
    fn after_zero_is_the_line_at_once() {
        let line = "Here goes nothing!";
        for animated in [true, false] {
            assert_eq!(
                Speech::new(line, animated).after(Duration::ZERO),
                Speech::new(line, animated),
                "animated {animated}"
            );
        }
    }

    #[test]
    fn animations_off_after_a_beat_is_whole_with_one_burble() {
        let line = "Here goes nothing!";
        let mut s = Speech::new(line, false).after(beat());
        assert_eq!(s.words_shown(), 0);
        assert_eq!(s.text(), " ".repeat(line.chars().count()));
        assert!(!s.advance(beat() - Duration::from_millis(1)));
        assert_eq!(s.words_shown(), 0);

        let mut s = Speech::new(line, false).after(beat());
        assert!(s.advance(beat()));
        assert_eq!(s.words_shown(), word_count(line));
        assert_eq!(s.text(), line);
        for _ in 0..20 {
            assert!(!s.advance(step()));
        }
        assert_eq!(s.text(), line);
    }

    #[test]
    fn a_line_settled_in_its_beat_is_whole_and_silent() {
        let line = "Here goes nothing!";
        let mut s = Speech::new(line, true).after(beat());
        assert!(!s.advance(beat() / 2));
        assert_eq!(s.words_shown(), 0);
        s.settle();
        assert_eq!(s.words_shown(), word_count(line));
        assert_eq!(s.text(), line);
        for _ in 0..20 {
            assert!(!s.advance(step()));
        }
        assert_eq!(s.text(), line);
    }

    #[test]
    fn a_stalled_step_across_the_beat_shows_every_due_word_once() {
        let line = "Court is in session.";
        assert_eq!(word_count(line), 4);
        let mut s = Speech::new(line, true).after(beat());
        assert_eq!(s.words_shown(), 0);
        assert!(s.advance(beat() + step() * 2));
        assert_eq!(s.words_shown(), 3);
        assert_eq!(s.text(), "Court is in         ");
        assert!(!s.advance(Duration::ZERO));
        assert_eq!(s.words_shown(), 3);
    }

    // ---- Spec 030 T004: the series pools ----------------------------------

    fn series(opponent: &str, player_wins: u32, opponent_wins: u32) -> Series {
        Series {
            planet: "tatooine".to_string(),
            opponent: opponent.to_string(),
            player_wins,
            opponent_wins,
        }
    }

    const EVENTS: [BanterEvent; 8] = [
        BanterEvent::MatchStart,
        BanterEvent::RoundWin,
        BanterEvent::RoundLoss,
        BanterEvent::RoundTie,
        BanterEvent::OpponentBust,
        BanterEvent::PlayerBust,
        BanterEvent::MatchWin,
        BanterEvent::MatchLoss,
    ];

    #[test]
    fn the_series_state_for_every_score() {
        use SeriesState::*;
        // (player wins, opponent wins) -> state, from the opponent's side.
        let best_of_3 = [
            ((0, 0), Opening),
            ((1, 0), Trailing),
            ((0, 1), Leading),
            ((1, 1), Decider),
        ];
        for ((p, o), want) in best_of_3 {
            assert_eq!(series_state(&series("greeb", p, o)), want, "best of 3 at {p}-{o}");
        }
        let best_of_5 = [
            ((0, 0), Opening),
            ((1, 0), Trailing),
            ((2, 0), Trailing),
            ((0, 1), Leading),
            ((1, 1), AllSquare),
            ((2, 1), Trailing),
            ((0, 2), Leading),
            ((1, 2), Leading),
            ((2, 2), Decider),
        ];
        for ((p, o), want) in best_of_5 {
            assert_eq!(series_state(&series("sovereign", p, o)), want, "best of 5 at {p}-{o}");
        }
    }

    #[test]
    fn every_reachable_state_has_lines_in_every_voice() {
        for (id, set) in all_sets() {
            let need = wins_needed(id);
            for p in 0..need {
                for o in 0..need {
                    let state = series_state(&series(id, p, o));
                    assert!(
                        !start_lines(set, Some(state)).is_empty(),
                        "{id}: no lines at {p}-{o} ({state:?})"
                    );
                }
            }
            assert!(!start_lines(set, None).is_empty(), "{id}: no lines with no series");
        }
    }

    #[test]
    fn every_voice_is_complete_for_series_play() {
        for (id, set) in all_sets() {
            for pool in [set.leading, set.trailing, set.decider] {
                assert!(pool.len() >= 3, "{id}: a mid-series pool must carry >= 3 lines");
            }
            for pool in [set.series_won, set.series_lost] {
                assert!(pool.len() >= 2, "{id}: a series result pool must carry >= 2 lines");
            }
            if wins_needed(id) == 3 {
                assert!(set.all_square.len() >= 3, "{id}: all square must carry >= 3 lines");
            }
        }
    }

    #[test]
    fn the_start_pool_follows_the_series_state() {
        for (id, set) in all_sets() {
            assert_eq!(start_lines(set, None), set.match_start, "{id}: no series");
            assert_eq!(start_lines(set, Some(SeriesState::Opening)), set.match_start, "{id}: Opening");
            assert_eq!(start_lines(set, Some(SeriesState::Leading)), set.leading, "{id}: Leading");
            assert_eq!(start_lines(set, Some(SeriesState::Trailing)), set.trailing, "{id}: Trailing");
            assert_eq!(start_lines(set, Some(SeriesState::AllSquare)), set.all_square, "{id}: AllSquare");
            assert_eq!(start_lines(set, Some(SeriesState::Decider)), set.decider, "{id}: Decider");
        }
    }

    #[test]
    fn a_deciding_match_end_draws_the_series_result() {
        for (id, set) in all_sets() {
            assert_eq!(lines_in_series(set, BanterEvent::MatchWin, true), set.series_won, "{id}");
            assert_eq!(lines_in_series(set, BanterEvent::MatchLoss, true), set.series_lost, "{id}");
        }
    }

    #[test]
    fn outside_a_decided_series_every_event_draws_todays_pool() {
        for (id, set) in all_sets() {
            for ev in EVENTS {
                assert_eq!(lines_in_series(set, ev, false), lines_for(set, ev), "{id}: {ev:?}");
                if !matches!(ev, BanterEvent::MatchWin | BanterEvent::MatchLoss) {
                    assert_eq!(
                        lines_in_series(set, ev, true),
                        lines_for(set, ev),
                        "{id}: {ev:?} in a decided series"
                    );
                }
            }
        }
    }
}
