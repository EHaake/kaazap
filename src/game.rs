use crate::{
    OPPONENT_THINKING_TIME_MS, STAND_THRESHOLD, card::{Card, FlipKind, PlayedCard}, player::{Player, PlayerState}
};
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

#[derive(Debug, Clone, Copy)]
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
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: PlayerState {
                name: "Your Name".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                // Interim fixed hand, same values as before — random
                // dealing from DEFAULT_SIDE_DECK arrives in T007
                hand: vec![
                    Some(Card::Plus(5)),
                    Some(Card::Plus(3)),
                    Some(Card::Plus(6)),
                    Some(Card::Plus(2)),
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
            },
            opponent: PlayerState {
                name: "Opponent".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                hand: vec![
                    Some(Card::Plus(2)),
                    Some(Card::Plus(6)),
                    Some(Card::Plus(1)),
                    Some(Card::Plus(4)),
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
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
        // While a sign choice is pending, the only meaningful keys are
        // the choice itself and cancel — everything else is ignored
        if matches!(self.game_phase, GamePhase::AwaitingSignChoice { .. }) {
            return match key {
                '+' | '1' => Some(GameAction::ChooseSign { positive: true }),
                '-' | '2' => Some(GameAction::ChooseSign { positive: false }),
                'c' => Some(GameAction::CancelSignChoice),
                _ => None,
            };
        }

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
                if matches!(self.game_phase, GamePhase::GameOver { winner }) {
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
            OpponentAction::PlayHand { index, value } => {
                self.commit_play(Player::Opponent, index, value);
                self.resolve_after_action();
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

        // Set gamephase to opponent's turn
        if self.opponent_can_act() {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
        }
    }

    /// Opponent's play logic:
    /// Return an OpponentAction based on opponent's hand and state
    ///
    fn decide_opponent_move(&self) -> OpponentAction {
        let score = self.opponent.score();
        let target = 20 - score;

        // Interim: only fixed Plus cards are recognized, matching current
        // behavior — the generalized predicate arrives in T008
        let card_hits_twenty =
            |card: &Card| -> bool { matches!(card, Card::Plus(n) if *n as i32 == target) };

        if let Some(index) = self.first_hand_index(card_hits_twenty) {
            return OpponentAction::PlayHand {
                index,
                value: target as i8,
            };
        }

        // if score is >= 17, stand
        if score >= STAND_THRESHOLD as i32 {
            return OpponentAction::Stand;
        }

        OpponentAction::Hit
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
        match self.decide_opponent_move() {
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

        self.game_phase = GamePhase::PlayerTurn;
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

    fn pc(card: Card, value: i8) -> PlayedCard {
        PlayedCard { card, value }
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
        gs.opponent.dealer_row = vec![pc(Card::Dealer(10), 10), pc(Card::Dealer(10), 10)];
        gs.opponent.played_row = vec![pc(Card::Plus(1), 1)]; // also 21 — equal totals
        gs.opponent.stood = true;
        gs.game_phase = GamePhase::RoundEnd;

        gs.update();

        assert!(matches!(gs.round_outcome, Some(RoundOutcome::OpponentWon)));
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
        // Player hand slot 0 holds Plus(5)
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
        // Opponent hand slot 1 holds Plus(6); a negative committed value
        // must land as passed — this is the path ± signs ride on
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
    fn commit_via_opponent_action_carries_the_action_value() {
        let mut gs = GameState::new();
        // Opponent hand slot 3 holds Plus(4)
        gs.apply_opponent_action(OpponentAction::PlayHand { index: 3, value: 4 });

        let pc = gs.opponent.played_row[0];
        assert_eq!(pc.card, Card::Plus(4));
        assert_eq!(pc.value, 4);
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
}
