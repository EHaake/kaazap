use crate::{
    OPPONENT_THINKING_TIME_MS,
    card::LogicCard,
    player::{Player, PlayerState},
};
use std::time::{Duration, Instant};

// Game score consts
const BUST_PENALTY: i32 = 10_000;
const WIN_BONUS: i32 = 10_000;
const LOSE_PENALTY: i32 = 5_000;
const TIE_PENALTY: i32 = 1_000;

const SETUP_TARGET: i32 = 16;
const POSITION_WEIGHT: i32 = 200; // how strongly we prefer being near SETUP_TARGET
const PRESERVE_WEIGHT: i32 = 50; // how strongly we avoid spending valuable cards

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    Hit,
    Stand,
    NextRound,
    NextGame,
    PlayHand { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpponentAction {
    Hit,
    Stand,
    PlayHand { index: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum RoundOutcome {
    PlayerWon,
    OpponentWon,
    Tied,
}

#[derive(Debug, Clone)]
pub enum GamePhase {
    PlayerTurn,
    OpponentThinking { until: Instant },
    OpponentTurn,
    RoundEnd,
    AwaitingNextRound,
    GameOver { winner: Player },
}

type PlayableCard = (usize, i32); // (index/hand position, card value)

#[derive(Debug, Clone)]
pub struct GameContext {
    pub opponent_score: i32,
    pub player_score: i32,
    pub player_stood: bool,
    pub opponent_hand: Vec<PlayableCard>,
}

impl GameContext {
    pub fn new(
        opponent_score: i32,
        player_score: i32,
        player_stood: bool,
        opponent_hand: Vec<PlayableCard>,
    ) -> Self {
        Self {
            opponent_score,
            player_score,
            player_stood,
            opponent_hand,
        }
    }
}

type ScoredOpponentMove = (i32, OpponentAction);

#[derive(Debug, Clone)]
pub struct MoveOutcome {
    pub new_opponent_score: i32,
    pub opponent_bust: bool,
    pub card_preservation_value: i32, // card preservation cost
}

impl MoveOutcome {
    fn new(context: &GameContext, action: OpponentAction) -> Self {
        match action {
            OpponentAction::Hit => {
                Self {
                    // TODO: modify to use distribution/non-determinism
                    new_opponent_score: context.opponent_score + 5, // expected outcome
                    opponent_bust: context.opponent_score > 20,
                    card_preservation_value: 0,
                }
            }
            OpponentAction::Stand => Self {
                new_opponent_score: context.opponent_score,
                opponent_bust: false,
                card_preservation_value: 0,
            },
            OpponentAction::PlayHand { index } => {
                // Get the played card that matches the index of 
                let played_card_value = context
                    .opponent_hand
                    .iter()
                    .find(|&&num| num.0 == index)
                    .unwrap()
                    .1;
                let new_opponent_score = context.opponent_score + played_card_value;
                Self {
                    new_opponent_score,
                    opponent_bust: new_opponent_score > 20,
                    card_preservation_value:played_card_value.abs(),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    pub player: PlayerState,
    pub opponent: PlayerState,
    pub game_phase: GamePhase,
    pub round_outcome: Option<RoundOutcome>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: PlayerState {
                name: "Your Name".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                hand: vec![
                    Some(LogicCard { value: 5 }),
                    Some(LogicCard { value: 3 }),
                    Some(LogicCard { value: 6 }),
                    Some(LogicCard { value: 2 }),
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
                played_card: false,
            },
            opponent: PlayerState {
                name: "Opponent".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                hand: vec![
                    Some(LogicCard { value: 2 }),
                    Some(LogicCard { value: 6 }),
                    Some(LogicCard { value: 1 }),
                    Some(LogicCard { value: 4 }),
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
                played_card: false,
            },
            game_phase: GamePhase::PlayerTurn,
            round_outcome: None,
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
        match key {
            '1' | '2' | '3' | '4' => Some(GameAction::PlayHand {
                index: key.to_digit(10)? as usize - 1,
            }),
            'd' | ' ' => Some(GameAction::Hit),
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
                    self.player_hit();
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
                if matches!(self.game_phase, GamePhase::PlayerTurn) {
                    self.play_card(index);
                    self.resolve_after_action();
                }
            }
            GameAction::NextRound => {
                if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
                    self.next_round();
                    self.resolve_after_action();
                }
            }
            GameAction::NextGame => {
                if matches!(self.game_phase, GamePhase::GameOver { winner: _ }) {
                    self.new_game();
                }
            }
        }
    }

    /// Take an OpponentAction and perform the action by calling appropriate fn's
    ///
    pub fn apply_opponent_action(&mut self, action: OpponentAction) {
        match action {
            OpponentAction::Hit => {
                self.opponent_hit();
                self.resolve_after_action();
            }
            OpponentAction::Stand => {
                self.opponent_stand();
                self.resolve_after_action();
            }
            OpponentAction::PlayHand { index } => {
                self.opponent_play_card(index);
                self.resolve_after_action();
            }
        }
    }

    /// After each state mutation action, check scores to see if status or
    /// GamePhase updates need to be applied
    ///
    fn resolve_after_action(&mut self) {
        // Don't resolve if awaiting next turn
        if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
            return;
        }

        // If player has played a card, move to Opponent's turn and reset flag
        if self.player.played_card {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
            self.player.played_card = false;
        }

        let player_score = self.player.score();
        let opponent_score = self.opponent.score();

        // Check for bust
        //
        // If player busts, round ends
        if player_score > 20 {
            self.player.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            return;
        }

        // If opponent busts, round ends
        if opponent_score > 20 {
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

        // Check scores and decide round outcome
        let outcome = if self.player.bust {
            RoundOutcome::OpponentWon
        } else if self.opponent.bust || player_score > opponent_score {
            RoundOutcome::PlayerWon
        } else if opponent_score > player_score {
            RoundOutcome::OpponentWon
        } else {
            RoundOutcome::Tied
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
        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.player.dealer_row.push(LogicCard {
            value: new_dealer_card_val,
        });

        // Set gamephase to opponent's turn
        if self.opponent_can_act() {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
        }
    }

    /// Moves fall into 3 buckets:
    /// 1. Hit/Get dealer card
    /// 2. Stand
    /// 3. Play hand card
    fn generate_candidate_moves(&self, context: &GameContext) -> Vec<OpponentAction> {
        let mut candidate_moves: Vec<OpponentAction> =
            vec![OpponentAction::Hit, OpponentAction::Stand];

        for card in context.opponent_hand.iter() {
            let (index, _value) = card;
            candidate_moves.push(OpponentAction::PlayHand { index: *index });
        }
        candidate_moves
    }

    fn score_outcome(&self, context: &GameContext, outcome: &MoveOutcome) -> i32 {
        let mut score: i32 = 0;

        // Always subtract bust penalty and card preservation penalty
        if outcome.opponent_bust {
            score -= BUST_PENALTY;
        }

        score -= outcome.card_preservation_value * PRESERVE_WEIGHT;

        if context.player_stood {
            if outcome.new_opponent_score > context.player_score && outcome.new_opponent_score <= 20
            {
                score += WIN_BONUS;
            } else if outcome.new_opponent_score == context.player_score {
                score -= TIE_PENALTY;
            } else {
                // score < player score
                score -= LOSE_PENALTY;
            }
        } else {
            let distance = (outcome.new_opponent_score - SETUP_TARGET).abs();
            score -= distance * POSITION_WEIGHT;

            // TODO:  Optional extra shaping:
            // slight reward for higher scores up to 18, discourages hovering too low.
            // score += outcome.new_opponent_score * 5;
        }

        score
    }

    fn score_moves(&self, context: GameContext) -> Vec<ScoredOpponentMove> {
        // Generate candidate moves from context
        let candidate_moves = self.generate_candidate_moves(&context);
        let mut scored_moves: Vec<ScoredOpponentMove> = vec![];

        // for each candidate move, generate an outcome and score it
        for (index, action) in candidate_moves.iter().enumerate() {
            let outcome = MoveOutcome::new(&context, *action);
            let score = self.score_outcome(&context, &outcome);
            scored_moves.push((score, candidate_moves[index]));
        }

        scored_moves
    }

    fn choose_opponent_move(&self, context: GameContext) -> OpponentAction {
        // Score the moves
        let scored_opponent_moves = self.score_moves(context);
        // Will always have something here so unwrap it
        let max_scored_move = scored_opponent_moves
            .iter()
            .max_by_key(|&(value, _)| value)
            .unwrap();

        max_scored_move.1 // (i32, OpponentAction)
    }

    /// Opponent's play logic:
    /// Return an OpponentAction based on opponent's hand and state
    ///
    fn decide_opponent_move(&self) -> OpponentAction {
        // build context
        let player_score = self.player.score();
        let opponent_score = self.opponent.score();
        let player_stood = self.player.stood;
        let opponent_hand: Vec<PlayableCard> = self
            .opponent
            .hand
            .iter()
            .enumerate()
            .filter_map(|(index, card_optional)| {
                card_optional.as_ref().map(|card| (index, card.value))
            })
            .collect();

        let context = GameContext::new(opponent_score, player_score, player_stood, opponent_hand);
        self.choose_opponent_move(context)

        // let score = self.opponent.score();
        // let target = 20 - score;
        //
        // let card_hits_twenty = |card: &LogicCard| -> bool { card.value == target };
        //
        // if let Some(index) = self.first_hand_index(card_hits_twenty) {
        //     return OpponentAction::PlayHand { index };
        // }
        //
        // // if score is >= threshold, stand
        // if score >= STAND_THRESHOLD as i32 {
        //     return OpponentAction::Stand;
        // }
        //
        // OpponentAction::Hit
    }

    /// Helper to finds first occurrence of card in hand that matches predicate
    ///
    fn first_hand_index<P>(&self, mut pred: P) -> Option<usize>
    where
        P: FnMut(&LogicCard) -> bool,
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
        match self.decide_opponent_move() {
            OpponentAction::Hit => {
                self.opponent_hit();
            }
            OpponentAction::Stand => {
                self.opponent_stand();
            }
            OpponentAction::PlayHand { index } => {
                self.opponent_play_card(index);
            }
        }

        self.game_phase = GamePhase::PlayerTurn;
        self.resolve_after_action();
    }

    /// Opponent hits (gets dealer card)
    ///
    fn opponent_hit(&mut self) {
        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.opponent.dealer_row.push(LogicCard {
            value: new_dealer_card_val,
        });
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

    ///  Remove card from player hand and add it to played_row
    ///
    fn play_card(&mut self, index: usize) {
        // Bounds checking already done before entering this fn
        let Some(Some(LogicCard { value: _ })) = self.player.hand.get(index) else {
            return;
        };

        if index < self.player.hand.len() {
            // "Remove" the card from the player's hand by setting value to 0
            let card_to_play = self.player.hand[index];
            self.player.hand[index] = None;
            self.player.played_row.push(card_to_play.unwrap());
        }
    }

    /// Opponent plays card
    ///
    fn opponent_play_card(&mut self, index: usize) {
        // Bounds checking already done before entering this fn
        let Some(Some(LogicCard { value: _ })) = self.opponent.hand.get(index) else {
            return;
        };

        if index < self.opponent.hand.len() {
            // "Remove" the card from the opponent's hand by setting value to 0
            let card_to_play = self.opponent.hand[index];
            self.opponent.hand[index] = None;
            self.opponent.played_row.push(card_to_play.unwrap());
        }
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
        // Reset round outcome
        self.round_outcome = None;
    }

    /// If in proper game phase, setup next round
    ///
    fn next_round(&mut self) {
        if let GamePhase::AwaitingNextRound = self.game_phase {
            self.setup_next_round();
        }
    }

    /// Reset all game stats
    ///
    fn new_game(&mut self) {
        if let GamePhase::GameOver { winner: _ } = self.game_phase {
            self.player.rounds_won = 0;
            self.opponent.rounds_won = 0;
            self.setup_next_round();
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
