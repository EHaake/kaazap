use crate::{
    OPPONENT_THINKING_TIME_MS,
    card::{Card, DEFAULT_SIDE_DECK, FlipKind, PlayedCard, deal_hand},
    opponent::{AiStrategy, DEFAULT_OPPONENT, OpponentProfile},
    player::{Player, PlayerState},
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    Hit,
    Stand,
    NextRound,
    NextGame,
    PlayHand { index: usize },
    ChooseSign { positive: bool },
    CancelSignChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpponentAction {
    Hit,
    Stand,
    // The AI resolves sign choices itself, so the committed value rides
    // in the action — the sign-choice prompt is player UI only
    PlayHand { index: usize, value: i8 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RoundOutcome {
    PlayerWon,
    OpponentWon,
    Tied,
}

#[derive(Debug, Clone)]
pub enum GamePhase {
    PlayerTurn,
    // A ± or tiebreaker card was selected and waits in hand for the
    // player's +/- answer
    AwaitingSignChoice { hand_index: usize },
    OpponentThinking { until: Instant },
    OpponentTurn,
    RoundEnd,
    AwaitingNextRound,
    GameOver { winner: Player },
}

#[derive(Debug)]
pub struct GameState {
    pub player: PlayerState,
    pub opponent: PlayerState,
    pub game_phase: GamePhase,
    pub round_outcome: Option<RoundOutcome>,
    /// The opponent being faced — drives its name, side deck, and the AI's
    /// stand threshold. Set once at match start and carried across rounds.
    pub opponent_profile: OpponentProfile,
    /// The player's side deck for this match — the deck they built in their
    /// profile (`profile.rs`), snapshotted at match start so hands (this
    /// match's and its rematches) deal from it, mirroring `opponent_profile`.
    pub player_deck: Vec<Card>,
}

/// How far below its stand threshold an opponent may *misplay* into an early
/// stand. A "chicken out" is only believable near a sane total — standing on 15
/// when it would normally push to 17 — so a would-be timid stand this many points
/// or more below the threshold degrades to a hit instead. This is the competence
/// floor that keeps a misplay from ever being a suicidal low stand (the classic
/// "stand on 0"); see `misplay` and `specs/013-bounded-misplays/`.
const MISPLAY_TIMID_MARGIN: i32 = 2;

impl GameState {
    /// A match against the neutral default opponent, with the default side
    /// deck (the pre-roster, pre-deck-builder behavior). Used by `Default` and
    /// the tests; the real flow uses [`with_opponent`] with the profile deck.
    pub fn new() -> Self {
        Self::with_opponent(DEFAULT_OPPONENT, DEFAULT_SIDE_DECK.to_vec())
    }

    /// Start a match against a specific opponent, dealing the player's hand
    /// from `player_deck` (the deck built in the profile). The opponent profile
    /// seeds its displayed name and its own side deck; both the profile and the
    /// player deck are stored so the AI and any post-`GameOver` rematch keep
    /// using them.
    pub fn with_opponent(opponent_profile: OpponentProfile, player_deck: Vec<Card>) -> Self {
        let player_hand = deal_hand(&mut rand::rng(), &player_deck);
        Self {
            player: PlayerState {
                name: "Your Name".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                hand: player_hand,
                bust: false,
                stood: false,
                rounds_won: 0,
            },
            opponent: PlayerState {
                name: opponent_profile.name.to_string(),
                dealer_row: vec![],
                played_row: vec![],
                hand: deal_hand(&mut rand::rng(), opponent_profile.side_deck),
                bust: false,
                stood: false,
                rounds_won: 0,
            },
            game_phase: GamePhase::PlayerTurn,
            round_outcome: None,
            opponent_profile,
            player_deck,
        }
    }

    /// Take the keys from the game loop and hand them it to action_from_key
    ///
    pub fn handle_game_input(&mut self, key: char) -> Option<GameAction> {
        self.game_action_from_key(key)
    }

    /// Convert a key pressed into an Action
    ///
    pub fn game_action_from_key(&self, key: char) -> Option<GameAction> {
        // While a sign choice is pending, the only meaningful keys are
        // the choice itself and cancel — everything else is ignored.
        // h/l (higher/lower) are the home-row pair the prompt shows;
        // +/- and 1/2 work as synonyms.
        if matches!(self.game_phase, GamePhase::AwaitingSignChoice { .. }) {
            return match key {
                'h' | '+' | '1' => Some(GameAction::ChooseSign { positive: true }),
                'l' | '-' | '2' => Some(GameAction::ChooseSign { positive: false }),
                'c' => Some(GameAction::CancelSignChoice),
                _ => None,
            };
        }

        match key {
            '1' | '2' | '3' | '4' => Some(GameAction::PlayHand {
                index: key.to_digit(10)? as usize - 1,
            }),
            'd' => Some(GameAction::Hit),
            // Space is the "proceed" key at the between-round pauses only:
            // advance the round at round-end, start a new game at game over.
            // On the player's turn, Space plays the highlighted card via the
            // cursor model in app.rs and never reaches here; drawing has its
            // own dedicated key (D).
            ' ' => match self.game_phase {
                GamePhase::AwaitingNextRound => Some(GameAction::NextRound),
                GamePhase::GameOver { .. } => Some(GameAction::NextGame),
                _ => None,
            },
            's' => Some(GameAction::Stand),
            'n' => Some(GameAction::NextRound),
            'g' => Some(GameAction::NextGame),
            _ => None,
        }
    }

    /// Centralize action validation
    ///
    pub fn apply_game_action(&mut self, action: GameAction) {
        match action {
            GameAction::Hit => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) && !self.player.stood {
                    if self.player.score() <= 20 {
                        self.player_hit();
                    } else {
                        // Drawing while over 20 is meaningless, so the
                        // draw key accepts the bust instead — same as
                        // standing (human-requested, T008b)
                        self.player_stand();
                    }
                    self.resolve_after_action();
                }
            }
            GameAction::Stand => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) {
                    self.player_stand();
                    self.resolve_after_action();
                }
            }
            GameAction::PlayHand { index } => {
                // The stood guard matters: after every opponent action
                // there's one input-before-tick frame where the phase
                // reads PlayerTurn even though the player has stood
                if matches!(self.game_phase, GamePhase::PlayerTurn) && !self.player.stood {
                    self.play_card(index);
                    self.resolve_after_action();
                }
            }
            GameAction::ChooseSign { positive } => {
                if let GamePhase::AwaitingSignChoice { hand_index } = self.game_phase {
                    self.commit_sign_choice(hand_index, positive);
                    self.resolve_after_action();
                }
            }
            GameAction::CancelSignChoice => {
                if matches!(self.game_phase, GamePhase::AwaitingSignChoice { .. }) {
                    self.game_phase = GamePhase::PlayerTurn;
                }
            }
            GameAction::NextRound => {
                if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
                    self.next_round();
                    self.resolve_after_action();
                }
            }
            GameAction::NextGame => {
                if matches!(self.game_phase, GamePhase::GameOver { .. }) {
                    self.new_game();
                }
            }
        }
    }

    /// Borrow the given side's state mutably
    ///
    fn side_state_mut(&mut self, side: Player) -> &mut PlayerState {
        match side {
            Player::Player => &mut self.player,
            Player::Opponent => &mut self.opponent,
        }
    }

    /// The one path a hand card takes onto the table: remove it from the
    /// hand and push it to played_row with its committed signed value
    ///
    fn commit_play(&mut self, side: Player, index: usize, value: i8) {
        let state = self.side_state_mut(side);
        let Some(Some(card)) = state.hand.get(index).copied() else {
            return;
        };

        state.hand[index] = None;
        state.played_row.push(PlayedCard { card, value });
    }

    /// After each state mutation action, check scores to see if status or
    /// GamePhase updates need to be applied
    ///
    fn resolve_after_action(&mut self) {
        // Don't resolve if awaiting next turn
        if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
            return;
        }

        // Note: playing a side card deliberately keeps the turn with the
        // player (matches real Pazaak — play a card, then hit or stand)

        let player_score = self.player.score();
        let opponent_score = self.opponent.score();

        // A side that has filled the table can hold no more cards, so it
        // stands on its current total. The over-20 checks below then bust
        // it if that total is over — exactly as a manual stand would
        // (001's over-20-at-stand rule). The recovery window survives
        // while a slot remains: an 11-card side over 20 keeps its turn and
        // can play its 12th as a recovery, which fills the table and
        // auto-stands it at the recovered total.
        if self.player.table_full() {
            self.player.stood = true;
        }
        if self.opponent.table_full() {
            self.opponent.stood = true;
        }

        // Busting resolves when a side can no longer respond: standing
        // (or already being stood, e.g. pushed over by a flip) with a
        // total over 20. Merely going over is survivable while cards
        // can still be played — that window is what makes minus and ±
        // cards worth holding (T008a ruling).
        if self.player.stood && player_score > 20 {
            self.player.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            return;
        }

        if self.opponent.stood && opponent_score > 20 {
            self.opponent.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            return;
        }

        // If player is at 20, stand
        if player_score == 20 {
            self.player.stood = true
        }

        // If opponent at 20, stand
        if opponent_score == 20 {
            self.opponent.stood = true;
        }

        // Check if both players have stood
        if self.player.stood && self.opponent.stood {
            self.game_phase = GamePhase::RoundEnd;
        }
    }

    /// Perform end of round tabulations and score updates,
    /// transitioning into AwaitingNextRound phase.
    ///
    fn finalize_round(&mut self) {
        let player_score = self.player.score();
        let opponent_score = self.opponent.score();

        // Resolution-time ruling (T012a, from skeptical review): any
        // side over 20 when the round ends is bust, stood or not — a
        // live over-20 total only survives while its owner can still
        // act, and the round ending means they no longer can. Without
        // this, a player sitting at 25 could win by flipping a standing
        // opponent over 20.
        if player_score > 20 {
            self.player.bust = true;
        }
        if opponent_score > 20 {
            self.opponent.bust = true;
        }

        // Check busts and scores to decide the round outcome; both
        // sides bust cancels to a tie (explicit ruling)
        let outcome = if self.player.bust && self.opponent.bust {
            RoundOutcome::Tied
        } else if self.player.bust {
            RoundOutcome::OpponentWon
        } else if self.opponent.bust || player_score > opponent_score {
            RoundOutcome::PlayerWon
        } else if opponent_score > player_score {
            RoundOutcome::OpponentWon
        } else {
            // Equal totals: a lone tiebreaker in play wins the round for
            // its owner; two of them cancel and the tie stands
            match (
                self.player.has_tiebreaker_in_play(),
                self.opponent.has_tiebreaker_in_play(),
            ) {
                (true, false) => RoundOutcome::PlayerWon,
                (false, true) => RoundOutcome::OpponentWon,
                _ => RoundOutcome::Tied,
            }
        };

        // Apply reward outcome (increment rounds won or not if tied)
        self.round_outcome = Some(outcome);
        self.apply_reward(outcome);

        // Check for game win else we move into AwaitingNextRound
        if self.player.rounds_won == 3 {
            self.game_phase = GamePhase::GameOver {
                winner: Player::Player,
            }
        } else if self.opponent.rounds_won == 3 {
            self.game_phase = GamePhase::GameOver {
                winner: Player::Opponent,
            }
        } else {
            self.game_phase = GamePhase::AwaitingNextRound;
        }
    }

    /// Apply round reward to the player who won, or nothing if tied
    ///
    fn apply_reward(&mut self, outcome: RoundOutcome) {
        match outcome {
            RoundOutcome::OpponentWon => {
                self.opponent.rounds_won += 1;
            }
            RoundOutcome::PlayerWon => {
                self.player.rounds_won += 1;
            }
            RoundOutcome::Tied => {}
        }
    }

    /// Check the GamePhase each tick of the gameloop and take appropriate actions
    ///
    pub fn update(&mut self) {
        match self.game_phase {
            GamePhase::PlayerTurn => {
                // If player is done for the round, immediately switch back to Opponent
                if !self.player_can_act() {
                    self.game_phase = GamePhase::OpponentThinking {
                        until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
                    };
                }
            }
            GamePhase::OpponentThinking { until } => {
                if Instant::now() >= until {
                    self.game_phase = GamePhase::OpponentTurn;
                }
            }
            GamePhase::OpponentTurn => {
                self.play_opponent_turn();
            }
            GamePhase::RoundEnd => {
                self.finalize_round();
            }
            _ => {}
        }
    }

    /// Check if opponent can still play this round
    ///
    fn opponent_can_act(&self) -> bool {
        !self.opponent.stood && !self.opponent.bust
    }

    /// Check if player can still play this round
    ///
    fn player_can_act(&self) -> bool {
        !self.player.stood && !self.player.bust
    }

    /// Deal a card to the player
    ///
    fn player_hit(&mut self) {
        self.player.dealer_row.push(draw_dealer_card());

        // A draw that goes over 20 keeps the turn: the player may still
        // recover with a minus card before standing into the bust
        if self.player.score() <= 20 && self.opponent_can_act() {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
        }
    }

    /// Opponent's play logic:
    /// Return an OpponentAction based on opponent's hand and state
    ///
    fn decide_opponent_move(&self) -> OpponentAction {
        // A full table can hold no more cards: stand rather than choose an
        // impossible hit or play. The resolve_after_action auto-stand is
        // the real enforcement; this keeps the decision itself honest.
        if self.opponent.table_full() {
            return OpponentAction::Stand;
        }

        let score = self.opponent.score();

        // Over 20: play the best recovery card that fits back under, or
        // stand and accept the bust (T008a)
        if score > 20 {
            if let Some((index, value)) = self.best_recovery_play() {
                return OpponentAction::PlayHand { index, value };
            }
            return OpponentAction::Stand;
        }

        // Board-aware (spec 010): once the player has stood, their total is
        // final — play to beat it. While they're still live, play to our own
        // (strategy-flavored) threshold, as before.
        if self.player.stood {
            self.decide_vs_stood_player(score)
        } else {
            self.decide_vs_live_player(score)
        }
    }

    /// Decide against a **stood** player, whose final total `p` is known: beat
    /// it. Stand once already ahead (or winning a tie); else play a card that
    /// lands a winning total (a guaranteed, bust-free win); else hit and chase.
    fn decide_vs_stood_player(&self, score: i32) -> OpponentAction {
        let p = self.player.score();

        // Player busted → any non-bust total wins, and we're already ≤ 20 here.
        if p > 20 {
            return OpponentAction::Stand;
        }
        // Already beating a stood player without busting → lock in the win.
        if score > p {
            return OpponentAction::Stand;
        }
        // A tie we actually win (only if we alone hold a tiebreaker) → stand.
        if score == p && self.opponent_wins_tie() {
            return OpponentAction::Stand;
        }
        // Behind (or a tie we'd lose): play a card that lands an outright
        // winning total if we have one.
        if let Some((index, value)) = self.best_winning_play_vs(p) {
            return OpponentAction::PlayHand { index, value };
        }
        // A Calculating opponent will also steal the round by landing exactly on
        // the player's total with a tiebreaker it alone holds — a guaranteed
        // win the other archetypes don't reach for.
        if self.opponent_profile.strategy == AiStrategy::Calculating
            && let Some((index, value)) = self.winning_tie_play_vs(p)
        {
            return OpponentAction::PlayHand { index, value };
        }
        // Otherwise hit to chase — standing behind is a sure loss.
        OpponentAction::Hit
    }

    /// A play landing exactly on the stood player's total `target` that leaves
    /// the opponent *alone* holding a tiebreaker in play — a tie the opponent
    /// wins per `finalize_round`. Impossible if the player holds a tiebreaker in
    /// play (their tiebreaker cancels the steal). Only reached when no outright
    /// winning play exists.
    fn winning_tie_play_vs(&self, target: i32) -> Option<(usize, i8)> {
        if self.player.has_tiebreaker_in_play() {
            return None;
        }
        let score = self.opponent.score();
        let already_has_tiebreaker = self.opponent.has_tiebreaker_in_play();
        for (index, slot) in self.opponent.hand.iter().enumerate() {
            let Some(card) = slot else { continue };
            // After this play the opponent holds a tiebreaker iff it already did
            // or the card it plays is the tiebreaker itself.
            if !already_has_tiebreaker && !matches!(card, Card::Tiebreaker) {
                continue;
            }
            for value in card.playable_values() {
                if score + value as i32 == target {
                    return Some((index, value));
                }
            }
        }
        None
    }

    /// Decide while the player is still **live** (no final target yet): play to
    /// this opponent's own threshold, exactly as before spec 010 — the
    /// Aggressive/Cautious archetypes only shift the effective threshold.
    fn decide_vs_live_player(&self, score: i32) -> OpponentAction {
        // Play any card that lands exactly on 20. A target outside card range
        // isn't reachable, so don't bother looking.
        if let Ok(target) = i8::try_from(20 - score)
            && let Some(index) = self.first_hand_index(|card| card.can_play_as(target))
        {
            return OpponentAction::PlayHand { index, value: target };
        }

        // At or above the (strategy-adjusted) stand threshold, stand.
        if score >= self.effective_threshold() {
            return OpponentAction::Stand;
        }

        OpponentAction::Hit
    }

    /// The opponent's stand threshold adjusted by its strategy: Aggressive
    /// pushes one higher, Cautious stands one earlier, others use it as-is.
    fn effective_threshold(&self) -> i32 {
        let base = self.opponent_profile.stand_threshold as i32;
        match self.opponent_profile.strategy {
            AiStrategy::Aggressive => base + 1,
            AiStrategy::Cautious => base - 1,
            AiStrategy::Basic | AiStrategy::Calculating => base,
        }
    }

    /// A hand play landing a **winning** total against a stood player at
    /// `target` — score + value in `(target, 20]`. Among candidates, Aggressive
    /// pushes the highest safe total, everyone else takes the minimal safe one;
    /// all such plays win outright, so the pick is flavor that makes the
    /// archetypes read differently.
    fn best_winning_play_vs(&self, target: i32) -> Option<(usize, i8)> {
        let score = self.opponent.score();
        let mut best: Option<(usize, i8, i32)> = None; // (index, value, total)
        for (index, slot) in self.opponent.hand.iter().enumerate() {
            let Some(card) = slot else { continue };
            for value in card.playable_values() {
                let total = score + value as i32;
                if total > target && total <= 20 {
                    let better = match best {
                        None => true,
                        Some((_, _, bt)) => match self.opponent_profile.strategy {
                            AiStrategy::Aggressive => total > bt,
                            _ => total < bt,
                        },
                    };
                    if better {
                        best = Some((index, value, total));
                    }
                }
            }
        }
        best.map(|(i, v, _)| (i, v))
    }

    /// Whether the opponent wins a tie — it alone holds a tiebreaker in play.
    fn opponent_wins_tie(&self) -> bool {
        self.opponent.has_tiebreaker_in_play() && !self.player.has_tiebreaker_in_play()
    }

    /// The seam where the AI gets a human touch: usually the deterministic best
    /// move, but with probability `profile.misplay` a legal-but-suboptimal
    /// deviation — and only while the position is still *open* (see
    /// `position_is_open` below), so a slip is never a suicidal concession once
    /// the outcome is decided. `roll` is drawn once per turn by the caller, so the
    /// policy stays a pure function of `(state, roll)` and is fully testable — and
    /// with `misplay == 0.0` (the default opponent) it is exactly
    /// `decide_opponent_move`.
    fn opponent_action(&self, roll: f32) -> OpponentAction {
        let best = self.decide_opponent_move();
        if roll < self.opponent_profile.misplay && self.position_is_open() {
            self.misplay(best)
        } else {
            best
        }
    }

    /// Whether a misplay is even plausible here: the outcome must still be *open*.
    /// A misplay is only a believable human error while the player is live (no
    /// fixed total to concede, no lead to throw) and the opponent is at or under
    /// 20 (not fumbling a bust-saving recovery card). Once the position is
    /// resolved — the player has stood, or the opponent has busted — every
    /// deviation is pure self-harm, so the AI plays its deterministic best.
    fn position_is_open(&self) -> bool {
        !self.player.stood && self.opponent.score() <= 20
    }

    /// A recognizable, always-legal human error standing in for `best`, reached
    /// only in an *open* position (`opponent_action` gates on `position_is_open`):
    /// over-greedy (`Stand → Hit`, but only when it can still draw), chickening
    /// out (`Hit → Stand`, but only a *plausible* early stand — see below), or
    /// fumbling a good card (`PlayHand → Hit` — legal because a `PlayHand` best
    /// implies the table isn't full, since a full table stands first).
    fn misplay(&self, best: OpponentAction) -> OpponentAction {
        match best {
            OpponentAction::Stand if !self.opponent.table_full() => OpponentAction::Hit,
            OpponentAction::Stand => OpponentAction::Stand,
            // Chicken out — but only within `MISPLAY_TIMID_MARGIN` of the
            // threshold (stand on ~15, not on 0). `best == Hit` already implies
            // `score < effective_threshold`; deeper below it, hitting is so
            // clearly right that no one would stand, so the slip just hits. This
            // is the floor that stops the suicidal low stand.
            OpponentAction::Hit
                if self.opponent.score() >= self.effective_threshold() - MISPLAY_TIMID_MARGIN =>
            {
                OpponentAction::Stand
            }
            OpponentAction::Hit => OpponentAction::Hit,
            OpponentAction::PlayHand { .. } => OpponentAction::Hit,
        }
    }

    /// The play that best recovers the opponent's over-20 total: the
    /// (index, value) leaving the highest score that fits within 20
    ///
    fn best_recovery_play(&self) -> Option<(usize, i8)> {
        let score = self.opponent.score();
        let mut best: Option<(usize, i8)> = None;

        for (index, slot) in self.opponent.hand.iter().enumerate() {
            let Some(card) = slot else { continue };
            for value in card.playable_values() {
                if score + value as i32 <= 20 && best.is_none_or(|(_, b)| value > b) {
                    best = Some((index, value));
                }
            }
        }

        best
    }

    /// Helper to finds first occurrence of card in hand that matches predicate
    ///
    fn first_hand_index<P>(&self, mut pred: P) -> Option<usize>
    where
        P: FnMut(&Card) -> bool,
    {
        self.opponent
            .hand
            .iter()
            .enumerate()
            .find_map(|(i, slot)| slot.as_ref().filter(|card| pred(card)).map(|_| i))
    }

    /// Play the opponent's turn (deal, play card, stand)
    ///
    fn play_opponent_turn(&mut self) {
        // Draw the misplay roll once per turn; the default opponent (misplay 0)
        // always gets its deterministic best move, so update()-driven tests stay
        // deterministic.
        match self.opponent_action(rand::random_range(0.0f32..1.0)) {
            OpponentAction::Hit => {
                self.opponent_hit();
            }
            OpponentAction::Stand => {
                self.opponent_stand();
            }
            OpponentAction::PlayHand { index, value } => {
                self.commit_play(Player::Opponent, index, value);
            }
        }

        // An over-20 opponent that hasn't stood keeps the turn to try a
        // recovery play — the mirror of the player's over-20 window
        if !self.opponent.stood && self.opponent.score() > 20 {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
        } else {
            self.game_phase = GamePhase::PlayerTurn;
        }

        self.resolve_after_action();
    }

    /// Opponent hits (gets dealer card)
    ///
    fn opponent_hit(&mut self) {
        self.opponent.dealer_row.push(draw_dealer_card());
    }

    /// Set gamestate to opponent's turn if we are on the player's turn
    ///
    pub fn player_stand(&mut self) {
        // Only allow if GamePhase is player's turn
        if let GamePhase::PlayerTurn = self.game_phase {
            self.player.stood = true;

            if self.opponent_can_act() {
                self.game_phase = GamePhase::OpponentThinking {
                    until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
                };
            }
        }
    }

    /// Opponent Stands
    ///
    fn opponent_stand(&mut self) {
        self.opponent.stood = true;
    }

    /// Resolve how the player's selected card commits: fixed-value cards
    /// commit immediately at face value; sign-choice kinds wait in hand
    /// for the player's +/- answer
    ///
    fn play_card(&mut self, index: usize) {
        let Some(Some(card)) = self.player.hand.get(index).copied() else {
            return;
        };

        if card.sign_choice_magnitude().is_some() {
            self.game_phase = GamePhase::AwaitingSignChoice { hand_index: index };
            return;
        }

        // Flips land at value 0 and immediately rework the whole table
        if let Card::Flip(kind) = card {
            self.commit_play(Player::Player, index, 0);
            self.apply_flip(kind);
            return;
        }

        let Some(value) = fixed_face_value(card) else {
            return;
        };

        self.commit_play(Player::Player, index, value);
    }

    /// Invert the sign of every table card the flip kind matches — both
    /// sides, dealer-drawn and hand-played alike. Bust/stand fallout is
    /// resolve_after_action's job, which runs after every action.
    ///
    fn apply_flip(&mut self, kind: FlipKind) {
        for row in [
            &mut self.player.dealer_row,
            &mut self.player.played_row,
            &mut self.opponent.dealer_row,
            &mut self.opponent.played_row,
        ] {
            for pc in row.iter_mut() {
                if kind.flips_value(pc.value) {
                    pc.value = -pc.value;
                }
            }
        }
    }

    /// Answer the pending sign choice: commit the card at its magnitude
    /// with the chosen sign, and hand the turn state back to PlayerTurn
    ///
    fn commit_sign_choice(&mut self, index: usize, positive: bool) {
        // Whatever happens below, the prompt is over
        self.game_phase = GamePhase::PlayerTurn;

        let Some(Some(card)) = self.player.hand.get(index).copied() else {
            return;
        };
        let Some(magnitude) = card.sign_choice_magnitude() else {
            return;
        };

        let value = if positive { magnitude } else { -magnitude };
        self.commit_play(Player::Player, index, value);
    }

    /// Setup for next round.
    /// Clear the player and opponent's dealer and played rows, and reset flags.
    fn setup_next_round(&mut self) {
        // Clear dealer row for both players
        self.player.dealer_row = vec![];
        self.opponent.dealer_row = vec![];

        // Clear played row for both players
        self.player.played_row = vec![];
        self.opponent.played_row = vec![];

        // Reset stood and busted flags
        self.player.bust = false;
        self.player.stood = false;
        self.opponent.bust = false;
        self.opponent.stood = false;

        // Reset round outcome
        self.round_outcome = None;

        // Set GamePhase to player turn
        self.game_phase = GamePhase::PlayerTurn;
    }

    /// If in proper game phase, setup next round
    ///
    fn next_round(&mut self) {
        if let GamePhase::AwaitingNextRound = self.game_phase {
            self.setup_next_round();
        }
    }

    /// Reset all game stats and deal each side a fresh hand
    ///
    fn new_game(&mut self) {
        if let GamePhase::GameOver { winner: _ } = self.game_phase {
            self.player.rounds_won = 0;
            self.opponent.rounds_won = 0;

            // New game, new hands — drawn independently, each from its own
            // side deck: the player from the deck this match was built with,
            // the opponent from the profile it was started with.
            let mut rng = rand::rng();
            self.player.hand = deal_hand(&mut rng, &self.player_deck);
            self.opponent.hand = deal_hand(&mut rng, self.opponent_profile.side_deck);

            self.setup_next_round();
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a random dealer card, 0-10 inclusive (intentional variant)
///
fn draw_dealer_card() -> PlayedCard {
    let n: u8 = rand::random_range(0..=10);
    PlayedCard {
        card: Card::Dealer(n),
        value: n as i8,
    }
}

/// The committed value of a card that needs no play-time choice,
/// or None for kinds that do (or that have a board effect instead)
///
fn fixed_face_value(card: Card) -> Option<i8> {
    match card {
        Card::Plus(n) => Some(n as i8),
        Card::Minus(n) => Some(-(n as i8)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HAND_SIZE, MAX_TABLE_CARDS};

    fn pc(card: Card, value: i8) -> PlayedCard {
        PlayedCard { card, value }
    }

    /// A run of `n` dealer cards each worth `v` (0-10), as table cards.
    fn dealer_run(n: usize, v: i8) -> Vec<PlayedCard> {
        (0..n).map(|_| pc(Card::Dealer(v as u8), v)).collect()
    }

    #[test]
    fn flip_inverts_matching_values_on_both_sides_including_dealer_rows() {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(2), 2), pc(Card::Dealer(7), 7)];
        gs.player.played_row = vec![pc(Card::Minus(4), -4)];
        gs.opponent.dealer_row = vec![pc(Card::Dealer(4), 4)];
        gs.opponent.played_row = vec![pc(Card::Plus(2), 2), pc(Card::Minus(3), -3)];
        gs.player.hand[0] = Some(Card::Flip(FlipKind::TwoFour));

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.player.dealer_row[0].value, -2);
        assert_eq!(gs.player.dealer_row[1].value, 7); // non-matching untouched
        assert_eq!(gs.player.played_row[0].value, 4); // -4 -> +4
        assert_eq!(gs.opponent.dealer_row[0].value, -4);
        assert_eq!(gs.opponent.played_row[0].value, -2);
        assert_eq!(gs.opponent.played_row[1].value, -3); // 3 is not 2&4's pair
    }

    #[test]
    fn flip_three_six_inverts_threes_and_sixes() {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(3), 3), pc(Card::Dealer(6), 6)];
        gs.opponent.dealer_row = vec![pc(Card::Dealer(2), 2)];
        gs.player.hand[1] = Some(Card::Flip(FlipKind::ThreeSix));

        gs.apply_game_action(GameAction::PlayHand { index: 1 });

        assert_eq!(gs.player.dealer_row[0].value, -3);
        assert_eq!(gs.player.dealer_row[1].value, -6);
        assert_eq!(gs.opponent.dealer_row[0].value, 2);
        // Totals reflect the inversion immediately
        assert_eq!(gs.player.score(), -9);
    }

    #[test]
    fn flip_card_itself_contributes_zero_and_lands_on_table() {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(5), 5)];
        gs.player.hand[0] = Some(Card::Flip(FlipKind::ThreeSix));

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert!(gs.player.hand[0].is_none());
        let flip = gs.player.played_row[0];
        assert_eq!(flip.card, Card::Flip(FlipKind::ThreeSix));
        assert_eq!(flip.value, 0);
        assert_eq!(gs.player.score(), 5);
    }

    #[test]
    fn flip_ignores_zero_valued_cards() {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(0), 0)];
        gs.player.played_row = vec![pc(Card::Flip(FlipKind::TwoFour), 0)];
        gs.player.hand[0] = Some(Card::Flip(FlipKind::TwoFour));

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.player.dealer_row[0].value, 0);
        assert_eq!(gs.player.played_row[0].value, 0);
    }

    #[test]
    fn flip_busts_a_standing_player_pushed_over_twenty() {
        let mut gs = GameState::new();
        // Opponent stood at 18: 10 + 10 - 2
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(10), 10)];
        gs.opponent.played_row = vec![pc(Card::Minus(2), -2)];
        gs.opponent.stood = true;
        gs.player.dealer_row = vec![pc(Card::Dealer(7), 7)];
        gs.player.hand[0] = Some(Card::Flip(FlipKind::TwoFour));

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.opponent.score(), 22);
        assert!(gs.opponent.bust);
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));

        // And the round resolves against the busted side
        gs.update();
        assert!(matches!(gs.round_outcome, Some(RoundOutcome::PlayerWon)));
    }

    #[test]
    fn flip_that_lowers_a_standing_player_keeps_them_standing_not_bust() {
        let mut gs = GameState::new();
        // Opponent stood at 18: 10 + 4 + 4
        gs.opponent.dealer_row = vec![
            pc(Card::Dealer(10), 10),
            pc(Card::Dealer(4), 4),
            pc(Card::Dealer(4), 4),
        ];
        gs.opponent.stood = true;
        gs.player.dealer_row = vec![pc(Card::Dealer(7), 7)];
        gs.player.hand[0] = Some(Card::Flip(FlipKind::TwoFour));

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.opponent.score(), 2); // 10 - 4 - 4
        assert!(!gs.opponent.bust);
        assert!(gs.opponent.stood);
        // Round continues, turn still with the player
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    /// Build a finished round where both sides stand on equal totals.
    /// Each side's dealer row is filled to `dealer_total`, and any extra
    /// played cards are appended to reach the same final score.
    fn tied_round(player_played: Vec<PlayedCard>, opponent_played: Vec<PlayedCard>) -> GameState {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(8), 8)];
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(8), 8)];
        gs.player.played_row = player_played;
        gs.opponent.played_row = opponent_played;
        gs.player.stood = true;
        gs.opponent.stood = true;
        gs.game_phase = GamePhase::RoundEnd;
        gs
    }

    #[test]
    fn tiebreaker_one_side_in_play_wins_the_tie_for_that_side() {
        // Both land on 19: player 18 + tiebreaker(+1), opponent 18 + 1
        let mut gs = tied_round(
            vec![pc(Card::Tiebreaker, 1)],
            vec![pc(Card::Plus(1), 1)],
        );
        assert_eq!(gs.player.score(), gs.opponent.score());

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::PlayerWon)));
        assert_eq!(gs.player.rounds_won, 1);
        assert_eq!(gs.opponent.rounds_won, 0);
    }

    #[test]
    fn tiebreaker_wins_the_tie_for_the_opponent_too() {
        let mut gs = tied_round(
            vec![pc(Card::Plus(1), 1)],
            vec![pc(Card::Tiebreaker, 1)],
        );
        assert_eq!(gs.player.score(), gs.opponent.score());

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
        assert_eq!(gs.opponent.rounds_won, 1);
        assert_eq!(gs.player.rounds_won, 0);
    }

    #[test]
    fn tiebreaker_played_as_minus_one_still_wins_the_tie() {
        // Identity decides the tie, not the sign it was played at:
        // player 18 - 1 = 17, opponent 18 - 1 = 17
        let mut gs = tied_round(
            vec![pc(Card::Tiebreaker, -1)],
            vec![pc(Card::Minus(1), -1)],
        );
        assert_eq!(gs.player.score(), gs.opponent.score());

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::PlayerWon)));
    }

    #[test]
    fn tiebreaker_on_both_sides_cancels_and_the_tie_stands() {
        let mut gs = tied_round(
            vec![pc(Card::Tiebreaker, 1)],
            vec![pc(Card::Tiebreaker, 1)],
        );
        assert_eq!(gs.player.score(), gs.opponent.score());

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::Tied)));
        assert_eq!(gs.player.rounds_won, 0);
        assert_eq!(gs.opponent.rounds_won, 0);
    }

    #[test]
    fn tie_with_no_tiebreaker_stands() {
        let mut gs = tied_round(vec![], vec![]);
        assert_eq!(gs.player.score(), gs.opponent.score());

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::Tied)));
        assert_eq!(gs.player.rounds_won, 0);
        assert_eq!(gs.opponent.rounds_won, 0);
    }

    #[test]
    fn tiebreaker_does_not_rescue_a_losing_round() {
        // The tiebreaker only breaks ties — it never overturns a loss
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(10), 10)];
        gs.player.played_row = vec![pc(Card::Tiebreaker, 1)]; // 11
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(9), 9)]; // 19
        gs.player.stood = true;
        gs.opponent.stood = true;
        gs.game_phase = GamePhase::RoundEnd;

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
    }

    #[test]
    fn tiebreaker_does_not_rescue_a_bust() {
        // Busting loses outright, tiebreaker in play or not
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(10), 10)];
        gs.player.played_row = vec![pc(Card::Tiebreaker, 1)]; // 21
        gs.player.bust = true;
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(9), 9)]; // 19
        gs.opponent.stood = true;
        gs.game_phase = GamePhase::RoundEnd;

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
    }

    #[test]
    fn bust_over_twenty_at_round_end_cannot_win_the_round() {
        // Skeptical-review finding: a live player at 25 flips a
        // standing opponent over 20, ending the round. The player must
        // not win it while over 20 themselves — both over, both bust,
        // tie (explicit ruling).
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![
            pc(Card::Dealer(10), 10),
            pc(Card::Dealer(10), 10),
            pc(Card::Dealer(5), 5),
        ]; // 25, live, never stood
        gs.player.hand = vec![Some(Card::Flip(FlipKind::TwoFour)), None, None, None];
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(10), 10)];
        gs.opponent.played_row = vec![pc(Card::Minus(2), -2)]; // 18
        gs.opponent.stood = true;

        gs.apply_game_action(GameAction::PlayHand { index: 0 });
        assert!(gs.opponent.bust); // flipped to 22 while standing
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));

        gs.update();

        assert!(gs.player.bust, "a side over 20 at resolution is bust");
        assert!(matches!(gs.round_outcome, Some(RoundOutcome::Tied)));
        assert_eq!(gs.player.rounds_won, 0);
        assert_eq!(gs.opponent.rounds_won, 0);
    }

    #[test]
    fn bust_stood_player_cannot_play_a_card() {
        // Skeptical-review finding: after every opponent action there's
        // one input-before-tick frame where the phase reads PlayerTurn
        // with the player already stood — a card play must be refused
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(8), 8)];
        gs.player.hand = vec![Some(Card::Plus(4)), None, None, None];
        gs.player.stood = true;
        gs.game_phase = GamePhase::PlayerTurn; // the stale-frame state

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.player.hand[0], Some(Card::Plus(4)));
        assert!(gs.player.played_row.is_empty());
        assert_eq!(gs.player.score(), 18);
    }

    #[test]
    fn full_match_terminates_within_bounded_updates() {
        // Headless full match through the production loop. The player hits
        // toward 17 and then stands, so the opponent faces a *live* player
        // (exercising its threshold climb and the over-20 recovery loop, the
        // riskiest part of T008a) and then a *stood* one (the board-aware
        // close-out). The match must reach GameOver within a bounded number of
        // steps — verified by instrumentation, not observation.
        let mut gs = GameState::new();
        let mut finished = false;

        for _ in 0..1000 {
            match gs.game_phase {
                GamePhase::PlayerTurn => {
                    if gs.player.score() < 17 {
                        gs.apply_game_action(GameAction::Hit);
                    } else {
                        gs.apply_game_action(GameAction::Stand);
                    }
                }
                GamePhase::AwaitingSignChoice { .. } => {
                    unreachable!("a hitting/standing player never opens the sign prompt")
                }
                // Skip the wall-clock thinking delay
                GamePhase::OpponentThinking { .. } => gs.game_phase = GamePhase::OpponentTurn,
                GamePhase::OpponentTurn | GamePhase::RoundEnd => gs.update(),
                GamePhase::AwaitingNextRound => gs.apply_game_action(GameAction::NextRound),
                GamePhase::GameOver { .. } => {
                    finished = true;
                    break;
                }
            }
        }

        assert!(finished, "match did not reach GameOver in 1000 steps");
        assert!(gs.player.rounds_won == 3 || gs.opponent.rounds_won == 3);
    }

    #[test]
    fn full_match_terminates_with_a_maximally_misplaying_opponent() {
        // S3: drive the misplay seam end-to-end. An opponent at misplay 1.0
        // deviates on every *open-position* decision (spec 013 plays a resolved
        // position — player stood, or over 20 — straight), so a greedy Stand→Hit
        // can push it over 20, re-entering OpponentThinking to recover. The turn
        // must still terminate (each hit adds a table card, bounded by
        // MAX_TABLE_CARDS, then table_full forces Stand) with no illegal 13th card
        // and no hang.
        let mut gs = GameState::with_opponent(
            OpponentProfile { misplay: 1.0, ..DEFAULT_OPPONENT },
            DEFAULT_SIDE_DECK.to_vec(),
        );
        let mut finished = false;

        for _ in 0..2000 {
            match gs.game_phase {
                GamePhase::PlayerTurn => {
                    if gs.player.score() < 17 {
                        gs.apply_game_action(GameAction::Hit);
                    } else {
                        gs.apply_game_action(GameAction::Stand);
                    }
                }
                GamePhase::AwaitingSignChoice { .. } => {
                    unreachable!("a hitting/standing player never opens the sign prompt")
                }
                GamePhase::OpponentThinking { .. } => gs.game_phase = GamePhase::OpponentTurn,
                GamePhase::OpponentTurn | GamePhase::RoundEnd => gs.update(),
                GamePhase::AwaitingNextRound => gs.apply_game_action(GameAction::NextRound),
                GamePhase::GameOver { .. } => {
                    finished = true;
                    break;
                }
            }
            // No side ever exceeds the table cap, even mid-turn.
            assert!(gs.opponent.table_card_count() <= MAX_TABLE_CARDS);
            assert!(gs.player.table_card_count() <= MAX_TABLE_CARDS);
        }

        assert!(finished, "misplaying match did not reach GameOver in 2000 steps");
    }

    /// An opponent sitting on `dealer_total` (split into valid 0-10
    /// dealer cards) with an exact hand — hands are dealt randomly, so
    /// tests must set their own.
    fn opponent_at(dealer_total: u8, hand: Vec<Card>) -> GameState {
        let mut gs = GameState::new();
        gs.opponent.dealer_row = vec![];

        let mut remaining = dealer_total;
        while remaining > 0 {
            let n = remaining.min(10);
            gs.opponent.dealer_row.push(pc(Card::Dealer(n), n as i8));
            remaining -= n;
        }

        gs.opponent.hand = hand.into_iter().map(Some).collect();
        gs
    }

    #[test]
    fn ai_plays_a_plus_card_that_lands_on_twenty() {
        let gs = opponent_at(15, vec![Card::Plus(5)]);

        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 5 }
        );
    }

    #[test]
    fn ai_plays_plus_minus_as_the_sign_that_reaches_twenty() {
        let gs = opponent_at(18, vec![Card::PlusMinus(2)]);

        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 2 }
        );
    }

    #[test]
    fn ai_plays_the_tiebreaker_to_reach_twenty() {
        let gs = opponent_at(19, vec![Card::Tiebreaker]);

        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 1 }
        );
    }

    #[test]
    fn ai_never_plays_a_flip_card() {
        // Both flips in hand, nothing else: the AI must not play one,
        // even sitting where a card play would otherwise be considered
        let gs = opponent_at(
            18,
            vec![Card::Flip(FlipKind::TwoFour), Card::Flip(FlipKind::ThreeSix)],
        );

        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
    }

    #[test]
    fn ai_skips_cards_that_miss_twenty_and_falls_through_to_stand() {
        let gs = opponent_at(18, vec![Card::Plus(4), Card::PlusMinus(3)]);

        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
    }

    #[test]
    fn ai_hits_below_the_stand_threshold_with_no_playable_card() {
        let gs = opponent_at(10, vec![Card::Plus(4)]);

        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit);
    }

    #[test]
    fn ai_stand_threshold_comes_from_the_profile() {
        // A hand that can't reach 20 from the given score, so only the
        // opponent's stand threshold decides hit vs. stand.

        // Score 16 (target 4, unreachable with +1): the default profile (17)
        // hits, a cautious opponent (15) stands on the same board.
        let mut gs = opponent_at(16, vec![Card::Plus(1)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit); // default = 17
        gs.opponent_profile = OpponentProfile { stand_threshold: 15, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);

        // Score 17: the default stands, an aggressive opponent (19) hits.
        let mut gs = opponent_at(17, vec![Card::Plus(1)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand); // default = 17
        gs.opponent_profile = OpponentProfile { stand_threshold: 19, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit);
    }

    // --- Board-aware decisions (spec 010): the player has stood, so the
    // opponent knows the final target and plays to beat it. ---

    /// Seed both boards: the player at `player_total` (optionally stood) and the
    /// opponent at `opp_total` holding `opp_hand`. Pure `pub`-field assignment,
    /// like `opponent_at` plus the player side.
    fn board_at(player_total: u8, player_stood: bool, opp_total: u8, opp_hand: Vec<Card>) -> GameState {
        let mut gs = opponent_at(opp_total, opp_hand);
        gs.player.dealer_row = vec![];
        let mut remaining = player_total;
        while remaining > 0 {
            let n = remaining.min(10);
            gs.player.dealer_row.push(pc(Card::Dealer(n), n as i8));
            remaining -= n;
        }
        gs.player.stood = player_stood;
        gs
    }

    #[test]
    fn ai_stands_once_it_is_already_beating_a_stood_player() {
        // The headline fix. Opponent at 14 is ahead of a stood player at 12 but
        // below its own threshold (17), so the old solitaire AI would keep
        // hitting (and risk busting a won round). Now it stands.
        let gs = board_at(12, true, 14, vec![Card::Plus(5)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
    }

    #[test]
    fn ai_behind_a_stood_player_plays_a_winning_card() {
        // At 15 vs a stood 18: +4 lands 19, a bust-free win — take it.
        let gs = board_at(18, true, 15, vec![Card::Plus(4)]);
        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 4 }
        );
    }

    #[test]
    fn ai_behind_a_stood_player_with_no_winning_card_hits_to_chase() {
        // At 15 vs a stood 18 with only +1 (→16, still losing): standing is a
        // sure loss, so hit and chase.
        let gs = board_at(18, true, 15, vec![Card::Plus(1)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit);
    }

    #[test]
    fn ai_stands_on_a_tie_it_wins_with_a_lone_tiebreaker() {
        // Both at 18, but the opponent alone holds a tiebreaker in play → the
        // tie is already its round → stand rather than risk pushing off it.
        let mut gs = board_at(18, true, 17, vec![Card::Plus(2)]);
        gs.opponent.played_row = vec![pc(Card::Tiebreaker, 1)]; // 17 + 1 = 18
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
    }

    #[test]
    fn ai_on_a_losing_tie_tries_to_pull_ahead() {
        // Both at 18, neither holds a tiebreaker → the opponent loses the tie,
        // so it plays +2 to reach 20 and actually win.
        let gs = board_at(18, true, 18, vec![Card::Plus(2)]);
        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 2 }
        );
    }

    #[test]
    fn ai_stands_when_the_player_has_already_busted() {
        // Player stood at 23 (busted); the opponent wins with any ≤20 total, so
        // even sitting at 5 it stands instead of chasing its threshold.
        let gs = board_at(23, true, 5, vec![Card::Plus(5)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
    }

    #[test]
    fn ai_archetypes_pick_different_winning_totals() {
        // At 15 vs a stood 16, both +2 (→17) and +5 (→20) win. Calculating/Basic
        // take the minimal safe total (17); Aggressive pushes the highest (20).
        let mut gs = board_at(16, true, 15, vec![Card::Plus(2), Card::Plus(5)]);
        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 2 },
            "Basic takes the minimal safe win"
        );
        gs.opponent_profile = OpponentProfile { strategy: AiStrategy::Aggressive, ..DEFAULT_OPPONENT };
        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 1, value: 5 },
            "Aggressive pushes the highest safe win"
        );
    }

    #[test]
    fn ai_calculating_steals_a_tie_with_the_tiebreaker() {
        // At 17 vs a stood 18, holding only the tiebreaker: no card lands an
        // outright win (18 isn't > 18), but playing the tiebreaker as +1 lands
        // exactly on 18 — a tie the opponent alone wins. Calculating takes it.
        let mut gs = board_at(18, true, 17, vec![Card::Tiebreaker]);
        gs.opponent_profile = OpponentProfile { strategy: AiStrategy::Calculating, ..DEFAULT_OPPONENT };
        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 1 }
        );

        // The other archetypes don't reach for the steal — they just hit.
        for strategy in [AiStrategy::Basic, AiStrategy::Aggressive, AiStrategy::Cautious] {
            gs.opponent_profile = OpponentProfile { strategy, ..DEFAULT_OPPONENT };
            assert_eq!(
                gs.decide_opponent_move(),
                OpponentAction::Hit,
                "{strategy:?} should not steal the tie"
            );
        }

        // The steal is off if the player *also* holds a tiebreaker in play (the
        // two cancel, so the tie wins for nobody): Calculating falls back to
        // hitting. Player 18 = 17 dealer + a tiebreaker in play.
        let mut gs2 = board_at(17, true, 17, vec![Card::Tiebreaker]);
        gs2.player.played_row = vec![pc(Card::Tiebreaker, 1)];
        gs2.opponent_profile = OpponentProfile { strategy: AiStrategy::Calculating, ..DEFAULT_OPPONENT };
        assert_eq!(gs2.player.score(), 18);
        assert_eq!(gs2.decide_opponent_move(), OpponentAction::Hit);
    }

    #[test]
    fn ai_cautious_stands_one_sooner_and_aggressive_one_later_while_live() {
        // Player still live (not stood): the threshold-shift archetypes change
        // where the opponent stands. At 16 with no way to 20, Basic (17) hits
        // but Cautious (16) stands; at 17 Basic stands but Aggressive (18) hits.
        let mut gs = opponent_at(16, vec![Card::Plus(1)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit); // Basic = 17
        gs.opponent_profile = OpponentProfile { strategy: AiStrategy::Cautious, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);

        let mut gs = opponent_at(17, vec![Card::Plus(1)]);
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand); // Basic = 17
        gs.opponent_profile = OpponentProfile { strategy: AiStrategy::Aggressive, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit);
    }

    // --- The misplay seam (spec 010): a thin, testable layer of randomness
    // over the deterministic core. ---

    // NOTE (spec 013): the two tests below were re-authored. They previously
    // locked in the *unbounded* misplay contract — `misplay(Hit) == Stand` at any
    // score, and a misplay against a stood player — which was the "stand on 0" /
    // "concede from behind" bug. They now assert the bounded, position-gated
    // contract. See `specs/013-bounded-misplays/` and `DECISIONS.md`.

    #[test]
    fn opponent_action_misplays_below_the_rate_and_plays_best_at_or_above() {
        // While the player is live and the opponent sits in its timid band (16 =
        // one below the default threshold 17), the best move is Hit and a misplay
        // chickens out into an allowed early Stand. A rate of 0.5 reads the roll
        // boundary; an empty hand keeps the best move a plain Hit.
        let mut gs = opponent_at(16, vec![]);
        gs.opponent_profile = OpponentProfile { misplay: 0.5, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit); // the best move

        assert_eq!(gs.opponent_action(0.1), OpponentAction::Stand); // roll < rate → misplay
        assert_eq!(gs.opponent_action(0.9), OpponentAction::Hit); // roll > rate → best
        assert_eq!(gs.opponent_action(0.5), OpponentAction::Hit); // roll == rate → best (not <)
    }

    #[test]
    fn misplay_deviates_each_best_move_legally() {
        // In the timid band: Stand → Hit (when it can still draw); Hit → Stand;
        // PlayHand → Hit. (16 = default threshold 17 − 1, inside the band.)
        let in_band = opponent_at(16, vec![Card::Plus(4)]);
        assert_eq!(in_band.misplay(OpponentAction::Stand), OpponentAction::Hit);
        assert_eq!(in_band.misplay(OpponentAction::Hit), OpponentAction::Stand);
        assert_eq!(
            in_band.misplay(OpponentAction::PlayHand { index: 0, value: 4 }),
            OpponentAction::Hit
        );

        // Deep below the threshold a "chicken out" would be suicidal, so a Hit
        // slip just hits — never a low stand (the competence floor).
        let deep = opponent_at(10, vec![Card::Plus(4)]);
        assert_eq!(deep.misplay(OpponentAction::Hit), OpponentAction::Hit);

        // A full table can't draw, so an over-greedy Stand stays Stand (legal).
        let mut full = GameState::new();
        full.opponent.dealer_row = dealer_run(MAX_TABLE_CARDS, 1);
        assert!(full.opponent.table_full());
        assert_eq!(full.misplay(OpponentAction::Stand), OpponentAction::Stand);
    }

    #[test]
    fn the_default_opponent_never_misplays() {
        // misplay 0.0: no roll in [0.0, 1.0) is < 0.0, so even the smallest
        // possible roll yields the deterministic best move.
        let gs = opponent_at(10, vec![Card::Plus(4)]); // default profile → best is Hit
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit);
        assert_eq!(gs.opponent_action(0.0), OpponentAction::Hit);
    }

    // --- Bounded misplays (spec 013): a slip is a believable error, never a
    // suicidal one. ---

    #[test]
    fn a_misplay_never_stands_on_a_low_total() {
        // The reported bug, pinned: a rookie standing on 0. With the player live
        // and the opponent at 0, best is Hit; even a certain misplay (rate 1.0)
        // must stay a Hit, never degrade into a suicidal Stand.
        let mut gs = opponent_at(0, vec![]);
        gs.opponent_profile = OpponentProfile { misplay: 1.0, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit);
        assert_eq!(gs.opponent_action(0.0), OpponentAction::Hit);
        assert_eq!(gs.opponent_action(0.999), OpponentAction::Hit);
    }

    #[test]
    fn no_roster_opponent_misplays_into_a_suicidal_stand_while_live() {
        // The user's success criterion as a property over the whole roster: with
        // the player live and the table not full, no opponent — however high its
        // misplay rate — ever stands below `effective_threshold − MISPLAY_TIMID_MARGIN`.
        // A roll of 0.0 forces the misplay branch for every profile with a rate
        // > 0; the two masters (rate 0.0) never misplay, so they only ever hit
        // below their threshold. This is `t`-relative, so it moves with the const
        // rather than pinning its value (see `the_timid_stand_is_bounded_to_the_band`).
        use crate::opponent::OPPONENTS;
        for profile in OPPONENTS {
            let mut probe = GameState::new();
            probe.opponent_profile = profile;
            let eff = probe.effective_threshold();
            let floor = eff - MISPLAY_TIMID_MARGIN;

            for score in 0..eff {
                let mut gs = opponent_at(score as u8, vec![]); // live player; empty hand → best Hit
                gs.opponent_profile = profile;
                assert_eq!(
                    gs.decide_opponent_move(),
                    OpponentAction::Hit,
                    "{} at {score}: best below threshold should be Hit",
                    profile.id
                );

                // A misplay-stand is allowed only inside the band; below it (and
                // for the never-misplaying masters) the action stays a Hit.
                let expected = if profile.misplay > 0.0 && score >= floor {
                    OpponentAction::Stand
                } else {
                    OpponentAction::Hit
                };
                assert_eq!(
                    gs.opponent_action(0.0),
                    expected,
                    "{} at score {score} (floor {floor}, rate {})",
                    profile.id,
                    profile.misplay
                );
            }
        }
    }

    #[test]
    fn the_timid_stand_is_bounded_to_the_band() {
        // Pins the chosen MISPLAY_TIMID_MARGIN = 2: default threshold 17 → the
        // lowest allowed misplay-stand is 15. At 14 and below a slip must hit; at
        // 15–16 it may chicken out. (Hardcoded so a change to the const trips here.)
        let profile = OpponentProfile { misplay: 1.0, ..DEFAULT_OPPONENT };
        for (score, expected) in [
            (14u8, OpponentAction::Hit),
            (15, OpponentAction::Stand),
            (16, OpponentAction::Stand),
        ] {
            let mut gs = opponent_at(score, vec![]);
            gs.opponent_profile = profile;
            assert_eq!(gs.decide_opponent_move(), OpponentAction::Hit, "best at {score}");
            assert_eq!(gs.opponent_action(0.0), expected, "misplay at {score}");
        }
    }

    #[test]
    fn a_resolved_position_is_played_straight_never_misplayed() {
        // Once the outcome is decided, every deviation is pure self-harm, so even a
        // certain misplay (1.0) plays the deterministic best for each move shape.
        let rate = OpponentProfile { misplay: 1.0, ..DEFAULT_OPPONENT };

        // Ahead of a stood player → Stand, not a greedy Hit that could bust a win.
        let mut ahead = board_at(12, true, 14, vec![Card::Plus(5)]);
        ahead.opponent_profile = rate;
        assert_eq!(ahead.decide_opponent_move(), OpponentAction::Stand);
        assert_eq!(ahead.opponent_action(0.0), OpponentAction::Stand);

        // Behind a stood player with no winning card → Hit to chase, not a
        // conceding Stand (the "standing when far behind" bug).
        let mut behind = board_at(18, true, 15, vec![Card::Plus(1)]);
        behind.opponent_profile = rate;
        assert_eq!(behind.decide_opponent_move(), OpponentAction::Hit);
        assert_eq!(behind.opponent_action(0.0), OpponentAction::Hit);

        // Holding a winning card vs a stood player → play it, not fumble to Hit.
        let mut winning = board_at(18, true, 15, vec![Card::Plus(4)]);
        winning.opponent_profile = rate;
        assert!(matches!(winning.decide_opponent_move(), OpponentAction::PlayHand { .. }));
        assert!(matches!(winning.opponent_action(0.0), OpponentAction::PlayHand { .. }));

        // Over 20 holding a recovery card → play the save, not fumble into a
        // certain bust (the position is closed by the over-20 gate, player live).
        let mut over = opponent_at(22, vec![Card::Minus(4)]); // 22 − 4 = 18
        over.opponent_profile = rate;
        assert!(matches!(over.decide_opponent_move(), OpponentAction::PlayHand { .. }));
        assert!(matches!(over.opponent_action(0.0), OpponentAction::PlayHand { .. }));
    }

    #[test]
    fn a_greedy_over_hit_still_fires_while_the_player_is_live() {
        // The kept flavor: at/above its threshold vs a live player best is Stand,
        // and a misplay still over-reaches into a Hit — the classic beginner bust,
        // the visible weakness that keeps rookies beatable.
        let mut gs = opponent_at(18, vec![]); // 18 ≥ default threshold 17, live player
        gs.opponent_profile = OpponentProfile { misplay: 1.0, ..DEFAULT_OPPONENT };
        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
        assert_eq!(gs.opponent_action(0.0), OpponentAction::Hit);
    }

    #[test]
    fn a_live_card_fumble_still_fires_through_the_seam() {
        // The other kept flavor, exercised through the gated seam (not just a
        // direct misplay call): vs a live player the best move is a card landing
        // on 20, and a misplay fumbles it into a Hit. Confirms the open-position
        // gate lets the fumble through while the player is live.
        let mut gs = opponent_at(15, vec![Card::Plus(5)]); // +5 lands 20 → best is PlayHand
        gs.opponent_profile = OpponentProfile { misplay: 1.0, ..DEFAULT_OPPONENT };
        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 0, value: 5 }
        );
        assert_eq!(gs.opponent_action(0.0), OpponentAction::Hit);
    }

    #[test]
    fn with_opponent_seeds_the_profile_and_name_while_new_stays_default() {
        use crate::{STAND_THRESHOLD, opponent::OPPONENTS};

        let gs = GameState::with_opponent(OPPONENTS[0], DEFAULT_SIDE_DECK.to_vec());
        assert_eq!(gs.opponent_profile.id, OPPONENTS[0].id);
        assert_eq!(gs.opponent.name, OPPONENTS[0].name);

        // new() is unchanged: the neutral default opponent.
        let d = GameState::new();
        assert_eq!(d.opponent_profile.id, DEFAULT_OPPONENT.id);
        assert_eq!(d.opponent.name, "Opponent");
        assert_eq!(d.opponent_profile.stand_threshold, STAND_THRESHOLD);
    }

    #[test]
    fn opponents_deal_from_their_own_deck_at_start_and_rematch() {
        // A deck sharing no card with DEFAULT_SIDE_DECK: a hand drawn from it
        // proves the opponent deals its *profile* deck, not the default pool —
        // both at match start (with_opponent) and on the post-GameOver rematch
        // (new_game). Guards against a regression that points either deal back
        // at DEFAULT_SIDE_DECK, which the roster's overlap would otherwise hide.
        static DECK: [Card; 4] = [Card::Plus(7), Card::Plus(8), Card::Minus(7), Card::Minus(8)];
        let profile = OpponentProfile {
            id: "test",
            name: "Test",
            difficulty: "",
            blurb: "",
            stand_threshold: 17,
            side_deck: &DECK,
            strategy: AiStrategy::Basic,
            misplay: 0.0,
        };

        let assert_opponent_hand_from_deck = |gs: &GameState, when: &str| {
            for slot in gs.opponent.hand.iter().copied() {
                let card = slot.expect("every dealt slot is filled");
                assert!(
                    DECK.contains(&card),
                    "{when}: opponent dealt {card:?}, not from its own deck"
                );
            }
        };

        let mut gs = GameState::with_opponent(profile, DEFAULT_SIDE_DECK.to_vec());
        assert_opponent_hand_from_deck(&gs, "start");
        // The player still deals from the default deck, not the opponent's.
        for slot in gs.player.hand.iter().copied() {
            let card = slot.expect("every dealt slot is filled");
            assert!(DEFAULT_SIDE_DECK.contains(&card));
        }

        // Rematch after GameOver re-deals the opponent from the same deck.
        gs.game_phase = GamePhase::GameOver { winner: Player::Player };
        gs.apply_game_action(GameAction::NextGame);
        assert_opponent_hand_from_deck(&gs, "rematch");
    }

    #[test]
    fn player_deals_from_the_supplied_deck_at_start_and_rematch() {
        // The mirror of the opponent test, for the seam this spec adds: the
        // player's hand comes from the deck handed to with_opponent (the built
        // profile deck), not DEFAULT_SIDE_DECK — at start and on rematch.
        let deck = vec![Card::Plus(7), Card::Plus(8), Card::Minus(7), Card::Minus(8)];
        let assert_player_hand_from_deck = |gs: &GameState, when: &str| {
            for slot in gs.player.hand.iter().copied() {
                let card = slot.expect("every dealt slot is filled");
                assert!(
                    deck.contains(&card),
                    "{when}: player dealt {card:?}, not from its deck"
                );
            }
        };

        let mut gs = GameState::with_opponent(DEFAULT_OPPONENT, deck.clone());
        assert_player_hand_from_deck(&gs, "start");

        gs.game_phase = GamePhase::GameOver { winner: Player::Player };
        gs.apply_game_action(GameAction::NextGame);
        assert_player_hand_from_deck(&gs, "rematch");

        // new() still deals the player from the default pool, unchanged.
        let d = GameState::new();
        for slot in d.player.hand.iter().copied() {
            let card = slot.expect("every dealt slot is filled");
            assert!(DEFAULT_SIDE_DECK.contains(&card));
        }
    }

    #[test]
    fn ai_play_reaches_exactly_twenty_end_to_end() {
        // The value the AI chooses is the value that lands on the
        // table, driven through the production turn path
        let mut gs = opponent_at(18, vec![Card::PlusMinus(2)]);
        gs.game_phase = GamePhase::OpponentTurn;

        gs.update();

        assert_eq!(gs.opponent.score(), 20);
        assert_eq!(gs.opponent.played_row[0].value, 2);
    }

    /// A player sitting over 20 (23 = 10 + 10 + 3), still live, with an
    /// exact hand — the state the T008a recovery window exists for
    fn player_over_at_23(hand: Vec<Option<Card>>) -> GameState {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![
            pc(Card::Dealer(10), 10),
            pc(Card::Dealer(10), 10),
            pc(Card::Dealer(3), 3),
        ];
        gs.player.hand = hand;
        gs
    }

    #[test]
    fn bust_going_over_twenty_no_longer_busts_immediately() {
        let mut gs = player_over_at_23(vec![None, None, None, None]);

        gs.resolve_after_action();

        assert!(!gs.player.bust);
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn bust_minus_card_recovers_an_over_twenty_total() {
        // The whole point of T008a: 23, play the -4, live at 19
        let mut gs = player_over_at_23(vec![Some(Card::Minus(4)), None, None, None]);

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.player.score(), 19);
        assert!(!gs.player.bust);
        assert!(gs.player.hand[0].is_none());
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn bust_standing_while_over_twenty_confirms_the_bust() {
        let mut gs = player_over_at_23(vec![None, None, None, None]);

        gs.apply_game_action(GameAction::Stand);

        assert!(gs.player.bust);
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));

        gs.update();
        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
        assert_eq!(gs.opponent.rounds_won, 1);
    }

    #[test]
    fn bust_hit_while_over_twenty_stands_into_the_bust() {
        // The draw key draws nothing while over — it accepts the bust,
        // exactly like standing (T008b ruling)
        let mut gs = player_over_at_23(vec![None, None, None, None]);
        let drawn_before = gs.player.dealer_row.len();

        gs.apply_game_action(GameAction::Hit);

        assert_eq!(gs.player.dealer_row.len(), drawn_before);
        assert!(gs.player.bust);
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));

        gs.update();
        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
    }

    #[test]
    fn bust_flip_pushing_a_live_opponent_over_leaves_them_alive() {
        let mut gs = GameState::new();
        // Opponent live at 16: 10 + 10 - 4; the 2&4 flips it to 24
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(10), 10)];
        gs.opponent.played_row = vec![pc(Card::Minus(4), -4)];
        gs.player.dealer_row = vec![pc(Card::Dealer(7), 7)];
        gs.player.hand = vec![Some(Card::Flip(FlipKind::TwoFour)), None, None, None];

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.opponent.score(), 24);
        assert!(!gs.opponent.bust);
        // (A standing opponent in the same spot still busts on the spot —
        // covered by flip_busts_a_standing_player_pushed_over_twenty)
    }

    #[test]
    fn ai_recovers_from_over_twenty_with_the_best_fitting_card() {
        // At 23: -2 leaves 21 (no), -4 leaves 19 (ok), ±3 as -3 leaves
        // 20 (best), +2 leaves 25 (no)
        let gs = opponent_at(23, vec![
            Card::Minus(2),
            Card::Minus(4),
            Card::PlusMinus(3),
            Card::Plus(2),
        ]);

        assert_eq!(
            gs.decide_opponent_move(),
            OpponentAction::PlayHand { index: 2, value: -3 }
        );
    }

    #[test]
    fn ai_with_no_recovery_stands_into_the_bust() {
        // At 23 nothing fits back under 20 (tiebreaker's -1 leaves 22)
        let mut gs = opponent_at(23, vec![Card::Plus(2), Card::Tiebreaker]);

        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);

        // And through the production path the stand confirms the bust
        gs.game_phase = GamePhase::OpponentTurn;
        gs.update();
        assert!(gs.opponent.bust);
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));
    }

    #[test]
    fn ai_over_twenty_turn_plays_the_recovery_and_hands_back_the_turn() {
        let mut gs = opponent_at(23, vec![Card::Minus(4)]);
        gs.game_phase = GamePhase::OpponentTurn;

        gs.update();

        assert_eq!(gs.opponent.score(), 19);
        assert!(!gs.opponent.bust);
        assert_eq!(gs.opponent.played_row[0].value, -4);
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    fn filled_slots(hand: &[Option<Card>]) -> usize {
        hand.iter().filter(|slot| slot.is_some()).count()
    }

    #[test]
    fn deal_new_game_starts_both_sides_with_full_hands() {
        let gs = GameState::new();

        assert_eq!(filled_slots(&gs.player.hand), HAND_SIZE);
        assert_eq!(filled_slots(&gs.opponent.hand), HAND_SIZE);
    }

    #[test]
    fn deal_gives_the_two_sides_independent_hands() {
        // A shared draw copied to both sides would make these always
        // equal; independent draws must sometimes differ
        let differed = (0..50).any(|_| {
            let gs = GameState::new();
            gs.player.hand != gs.opponent.hand
        });

        assert!(
            differed,
            "player and opponent hands were identical in 50 consecutive deals"
        );
    }

    #[test]
    fn deal_new_game_redeals_spent_hands() {
        let mut gs = GameState::new();
        gs.player.hand[0] = None;
        gs.player.hand[2] = None;
        gs.opponent.hand[1] = None;
        gs.player.rounds_won = 3;
        gs.game_phase = GamePhase::GameOver {
            winner: Player::Player,
        };

        gs.apply_game_action(GameAction::NextGame);

        assert_eq!(filled_slots(&gs.player.hand), HAND_SIZE);
        assert_eq!(filled_slots(&gs.opponent.hand), HAND_SIZE);
        assert_eq!(gs.player.rounds_won, 0);
        assert_eq!(gs.opponent.rounds_won, 0);
    }

    #[test]
    fn deal_next_round_leaves_spent_hands_untouched() {
        // Regression: no mid-match redraw — a played card stays gone
        // until the next game deals fresh hands
        let mut gs = GameState::new();
        gs.player.hand[0] = None;
        gs.player.hand[2] = None;
        gs.opponent.hand[3] = None;
        let player_before = gs.player.hand.clone();
        let opponent_before = gs.opponent.hand.clone();
        gs.game_phase = GamePhase::AwaitingNextRound;

        gs.apply_game_action(GameAction::NextRound);

        assert_eq!(gs.player.hand, player_before);
        assert_eq!(gs.opponent.hand, opponent_before);
    }

    #[test]
    fn dealer_draw_stays_within_bounds_and_hits_both_ends() {
        let mut seen_zero = false;
        let mut seen_ten = false;

        for _ in 0..1000 {
            let pc = draw_dealer_card();
            let Card::Dealer(n) = pc.card else {
                panic!("dealer draw produced a non-dealer card: {:?}", pc.card);
            };

            assert!(n <= 10, "dealer draw out of bounds: {n}");
            assert_eq!(pc.value, n as i8);

            seen_zero |= n == 0;
            seen_ten |= n == 10;
        }

        // Catches the range shrinking (e.g. 0..10 instead of 0..=10);
        // odds of a false failure are (10/11)^1000 — negligible
        assert!(seen_zero, "0 never drawn in 1000 draws");
        assert!(seen_ten, "10 never drawn in 1000 draws");
    }

    #[test]
    fn commit_play_empties_slot_and_records_card_with_value() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::Plus(5));
        gs.commit_play(Player::Player, 0, 5);

        assert!(gs.player.hand[0].is_none());
        assert_eq!(gs.player.played_row.len(), 1);
        let pc = gs.player.played_row[0];
        assert_eq!(pc.card, Card::Plus(5));
        assert_eq!(pc.value, 5);
        assert_eq!(gs.player.score(), 5);
    }

    #[test]
    fn commit_play_negative_value_lands_negative() {
        let mut gs = GameState::new();
        // A negative committed value must land as passed — this is the
        // path ± signs ride on
        gs.opponent.hand[1] = Some(Card::Plus(6));
        gs.commit_play(Player::Opponent, 1, -6);

        assert!(gs.opponent.hand[1].is_none());
        let pc = gs.opponent.played_row[0];
        assert_eq!(pc.card, Card::Plus(6));
        assert_eq!(pc.value, -6);
        assert_eq!(gs.opponent.score(), -6);
    }

    #[test]
    fn commit_play_on_empty_or_bad_slot_is_a_noop() {
        let mut gs = GameState::new();
        gs.commit_play(Player::Player, 2, 6);
        gs.commit_play(Player::Player, 2, 6); // slot already empty
        gs.commit_play(Player::Player, 99, 1); // out of bounds

        assert_eq!(gs.player.played_row.len(), 1);
    }

    #[test]
    fn commit_via_opponent_turn_carries_the_decided_value() {
        // Through the production path: OpponentTurn -> update() ->
        // decide -> commit. At 18 with a +2, the AI plays it at 2.
        let mut gs = opponent_at(18, vec![Card::Plus(2)]);
        gs.game_phase = GamePhase::OpponentTurn;

        gs.update();

        let pc = gs.opponent.played_row[0];
        assert_eq!(pc.card, Card::Plus(2));
        assert_eq!(pc.value, 2);
        assert_eq!(gs.opponent.score(), 20);
    }

    #[test]
    fn sign_play_enters_choice_phase_and_keeps_card_in_hand() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::PlusMinus(3));

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert!(matches!(
            gs.game_phase,
            GamePhase::AwaitingSignChoice { hand_index: 0 }
        ));
        assert_eq!(gs.player.hand[0], Some(Card::PlusMinus(3)));
        assert!(gs.player.played_row.is_empty());
    }

    #[test]
    fn sign_choose_positive_commits_positive() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::PlusMinus(3));
        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        gs.apply_game_action(GameAction::ChooseSign { positive: true });

        assert!(gs.player.hand[0].is_none());
        let pc = gs.player.played_row[0];
        assert_eq!(pc.card, Card::PlusMinus(3));
        assert_eq!(pc.value, 3);
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn sign_choose_negative_commits_negative() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::PlusMinus(3));
        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        gs.apply_game_action(GameAction::ChooseSign { positive: false });

        assert!(gs.player.hand[0].is_none());
        let pc = gs.player.played_row[0];
        assert_eq!(pc.card, Card::PlusMinus(3));
        assert_eq!(pc.value, -3);
    }

    #[test]
    fn sign_tiebreaker_plays_as_plus_or_minus_one() {
        let mut gs = GameState::new();
        gs.player.hand[2] = Some(Card::Tiebreaker);

        gs.apply_game_action(GameAction::PlayHand { index: 2 });
        assert!(matches!(
            gs.game_phase,
            GamePhase::AwaitingSignChoice { hand_index: 2 }
        ));

        gs.apply_game_action(GameAction::ChooseSign { positive: false });
        let pc = gs.player.played_row[0];
        assert_eq!(pc.card, Card::Tiebreaker);
        assert_eq!(pc.value, -1);
    }

    #[test]
    fn sign_cancel_restores_turn_with_card_unspent() {
        let mut gs = GameState::new();
        gs.player.hand[1] = Some(Card::PlusMinus(6));
        gs.apply_game_action(GameAction::PlayHand { index: 1 });

        gs.apply_game_action(GameAction::CancelSignChoice);

        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
        assert_eq!(gs.player.hand[1], Some(Card::PlusMinus(6)));
        assert!(gs.player.played_row.is_empty());
    }

    #[test]
    fn space_advances_at_pauses_and_never_draws() {
        let mut gs = GameState::new(); // starts at PlayerTurn
        // On the player's turn Space is not a draw: it plays the highlighted
        // card, which is the cursor model's job (app.rs) — so the engine's
        // key map returns None here. Draw keeps its own dedicated key, D.
        assert_eq!(gs.game_action_from_key(' '), None);
        assert_eq!(gs.game_action_from_key('d'), Some(GameAction::Hit)); // D still draws

        gs.game_phase = GamePhase::AwaitingNextRound;
        assert_eq!(gs.game_action_from_key(' '), Some(GameAction::NextRound));
        assert_eq!(gs.game_action_from_key('n'), Some(GameAction::NextRound)); // n unchanged

        gs.game_phase = GamePhase::GameOver { winner: Player::Player };
        assert_eq!(gs.game_action_from_key(' '), Some(GameAction::NextGame));
        assert_eq!(gs.game_action_from_key('g'), Some(GameAction::NextGame)); // g unchanged
    }

    #[test]
    fn sign_phase_maps_only_choice_and_cancel_keys() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::PlusMinus(3));
        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(
            gs.game_action_from_key('+'),
            Some(GameAction::ChooseSign { positive: true })
        );
        assert_eq!(
            gs.game_action_from_key('1'),
            Some(GameAction::ChooseSign { positive: true })
        );
        assert_eq!(
            gs.game_action_from_key('-'),
            Some(GameAction::ChooseSign { positive: false })
        );
        assert_eq!(
            gs.game_action_from_key('2'),
            Some(GameAction::ChooseSign { positive: false })
        );
        // Home-row pair: h = higher (+), l = lower (-)
        assert_eq!(
            gs.game_action_from_key('h'),
            Some(GameAction::ChooseSign { positive: true })
        );
        assert_eq!(
            gs.game_action_from_key('l'),
            Some(GameAction::ChooseSign { positive: false })
        );
        assert_eq!(
            gs.game_action_from_key('c'),
            Some(GameAction::CancelSignChoice)
        );
        // Hit/Stand/other play keys are ignored while the prompt is up
        assert_eq!(gs.game_action_from_key('d'), None);
        assert_eq!(gs.game_action_from_key('s'), None);
        assert_eq!(gs.game_action_from_key('3'), None);
    }

    #[test]
    fn sign_normal_phase_key_mapping_unchanged() {
        let gs = GameState::new();

        assert_eq!(
            gs.game_action_from_key('1'),
            Some(GameAction::PlayHand { index: 0 })
        );
        assert_eq!(gs.game_action_from_key('d'), Some(GameAction::Hit));
        assert_eq!(gs.game_action_from_key('s'), Some(GameAction::Stand));
        assert_eq!(gs.game_action_from_key('+'), None);
        assert_eq!(gs.game_action_from_key('c'), None);
        assert_eq!(gs.game_action_from_key('h'), None);
        assert_eq!(gs.game_action_from_key('l'), None);
    }

    #[test]
    fn sign_update_leaves_choice_phase_alone() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::PlusMinus(3));
        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        gs.update();

        assert!(matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
    }

    #[test]
    fn sign_choose_outside_choice_phase_is_noop() {
        let mut gs = GameState::new();

        gs.apply_game_action(GameAction::ChooseSign { positive: true });

        assert!(gs.player.played_row.is_empty());
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn fixed_face_value_covers_only_choice_free_kinds() {
        assert_eq!(fixed_face_value(Card::Plus(4)), Some(4));
        assert_eq!(fixed_face_value(Card::Minus(3)), Some(-3));
        assert_eq!(fixed_face_value(Card::PlusMinus(2)), None);
        assert_eq!(fixed_face_value(Card::Tiebreaker), None);
        assert_eq!(
            fixed_face_value(Card::Flip(crate::card::FlipKind::TwoFour)),
            None
        );
    }

    #[test]
    fn cap_full_table_at_or_under_twenty_auto_stands() {
        let mut gs = GameState::new();
        gs.player.dealer_row = dealer_run(MAX_TABLE_CARDS, 1); // 12 cards, score 12
        assert!(gs.player.table_full());

        gs.resolve_after_action();

        assert!(gs.player.stood);
        assert!(!gs.player.bust);
        // opponent still live, so the turn simply passes — not a round end
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn cap_full_opponent_table_auto_stands() {
        // The auto-stand insertion covers both sides at the shared choke
        let mut gs = GameState::new();
        gs.opponent.dealer_row = dealer_run(MAX_TABLE_CARDS, 1); // 12, score 12

        gs.resolve_after_action();

        assert!(gs.opponent.stood);
        assert!(!gs.opponent.bust);
    }

    #[test]
    fn cap_twelfth_card_over_twenty_busts() {
        let mut gs = GameState::new();
        gs.player.dealer_row = dealer_run(MAX_TABLE_CARDS, 2); // 12 cards, score 24

        gs.resolve_after_action();

        assert!(gs.player.stood);
        assert!(gs.player.bust);
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));

        gs.update();
        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
    }

    #[test]
    fn cap_recovery_as_the_twelfth_card_holds() {
        // 11 dealer cards at 23 (over 20), still live — the recovery
        // window is open because a slot remains. The -4 lands as the
        // 12th card, fills the table, and auto-stands the recovered 19.
        let mut gs = GameState::new();
        let mut row = dealer_run(10, 2); // 20
        row.push(pc(Card::Dealer(3), 3)); // 23, 11 cards
        gs.player.dealer_row = row;
        gs.player.hand = vec![Some(Card::Minus(4)), None, None, None];
        assert!(!gs.player.table_full());

        gs.apply_game_action(GameAction::PlayHand { index: 0 });

        assert_eq!(gs.player.table_card_count(), MAX_TABLE_CARDS);
        assert_eq!(gs.player.score(), 19);
        assert!(gs.player.stood); // auto-stood on fill
        assert!(!gs.player.bust); // 19 holds, not a bust
    }

    #[test]
    fn cap_full_table_refuses_further_cards() {
        // The auto-stand is what blocks a 13th card: once stood, the Hit
        // guard refuses. (Remove the auto-stand line and this goes red —
        // stood stays false and a 13th dealer card lands.)
        let mut gs = GameState::new();
        gs.player.dealer_row = dealer_run(MAX_TABLE_CARDS, 1); // full, score 12
        gs.resolve_after_action();
        assert!(gs.player.stood);

        let before = gs.player.table_card_count();
        gs.apply_game_action(GameAction::Hit);

        assert_eq!(gs.player.table_card_count(), before); // no 13th card
    }

    #[test]
    fn cap_both_tables_full_without_bust_resolves_by_totals() {
        // Both sides fill at ≤ 20: both auto-stand, and the round resolves
        // by the existing totals rule — a full table is just a stand.
        let mut gs = GameState::new();
        gs.player.dealer_row = dealer_run(MAX_TABLE_CARDS, 1); // 12 cards, 12
        let mut opp = dealer_run(10, 1);
        opp.extend(dealer_run(2, 0));
        gs.opponent.dealer_row = opp; // 12 cards, 10

        gs.resolve_after_action();
        assert!(matches!(gs.game_phase, GamePhase::RoundEnd));

        gs.update();
        assert!(matches!(gs.round_outcome, Some(RoundOutcome::PlayerWon)));
    }

    #[test]
    fn ai_full_table_stands_over_a_winning_card() {
        // 12 cards at 14, and a +6 in hand that would land exactly on 20.
        // Without the full-table guard the AI plays the winner; with it,
        // the impossible 13th play is refused and it stands.
        let mut gs = GameState::new();
        let mut row = dealer_run(10, 1);
        row.extend(dealer_run(2, 2));
        gs.opponent.dealer_row = row; // 12 cards, score 14
        gs.opponent.hand = vec![Some(Card::Plus(6)), None, None, None];
        assert!(gs.opponent.table_full());

        assert_eq!(gs.decide_opponent_move(), OpponentAction::Stand);
    }

    #[test]
    fn ai_full_table_stands_through_the_turn_without_drawing() {
        // End to end: a full-table opponent below the stand threshold with
        // nothing playable would Hit and draw a 13th card without the
        // guard; instead it stands and the table stays at 12.
        let mut gs = GameState::new();
        gs.opponent.dealer_row = dealer_run(MAX_TABLE_CARDS, 1); // 12, score 12
        gs.opponent.hand = vec![None, None, None, None];
        gs.game_phase = GamePhase::OpponentTurn;
        let before = gs.opponent.table_card_count();

        gs.update();

        assert!(gs.opponent.stood);
        assert_eq!(gs.opponent.table_card_count(), before); // no 13th card
    }

    #[test]
    fn cap_opponent_hitting_its_twelfth_card_auto_stands_and_busts_if_over() {
        // The subtle ordering in play_opponent_turn: the over-20-keeps-turn
        // check runs BEFORE resolve_after_action. 11 cards at 16 with an
        // empty hand → the AI hits its 12th (below threshold, nothing to
        // play). Whatever it draws, the table is now full, so it auto-stands
        // — and if the draw pushed it over 20, resolve overrides the kept
        // turn into a bust. Deterministic in structure: always full + stood,
        // and bust iff over 20.
        let mut gs = GameState::new();
        let mut row = dealer_run(8, 2); // 16
        row.extend(dealer_run(3, 0));
        gs.opponent.dealer_row = row; // 11 cards, score 16
        gs.opponent.hand = vec![None, None, None, None];
        gs.game_phase = GamePhase::OpponentTurn;

        gs.update(); // AI hits its 12th card

        assert_eq!(gs.opponent.table_card_count(), MAX_TABLE_CARDS, "the hit filled the table");
        assert!(gs.opponent.stood, "a full table auto-stands");
        if gs.opponent.score() > 20 {
            assert!(gs.opponent.bust, "full and over 20 must bust");
            assert!(matches!(gs.game_phase, GamePhase::RoundEnd));
        } else {
            assert!(!gs.opponent.bust, "full and <= 20 holds");
        }
    }
}
