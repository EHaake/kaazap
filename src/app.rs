use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    SELECTION_PULSE_MS,
    audio::{Audio, AudioSnapshot, Sfx, audio_cues},
    board::BoardView,
    campaign::NodeRef,
    campaign_map::{CampaignMapState, MapOutcome},
    card::Card,
    config::Config,
    deck_builder::{BuildOutcome, DeckBuilderState},
    frame::{
        Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text, draw_text_centered,
        draw_text_in,
    },
    game::{GameAction, GamePhase, GameState},
    layout::OverlayLayout,
    menu::{MenuItem, MenuOutcome, MenuState},
    opponent::{OpponentProfile, opponent_by_id},
    opponent_select::{OpponentSelectState, SelectOutcome},
    overlay::{Overlay, OverlayKind},
    player::Player,
    profile::Profile,
    screen::Screen,
    settings::{SettingRow, Settings, SettingsAction, SettingsState},
};

/// How much one ←/→ press moves a volume slider on the settings screen.
const VOLUME_STEP: f32 = 0.1;

/// The one shared selection animation: a gentle two-phase breathe that
/// modulates the emphasis of whatever is currently selected (a hand
/// card's heavy border, the highlighted menu item). One cadence across
/// every screen, per design/brief.md — the only thing that moves.
#[derive(Debug)]
pub struct SelectionPulse {
    acc: Duration,
    on: bool,
}

impl Default for SelectionPulse {
    fn default() -> Self {
        Self {
            acc: Duration::ZERO,
            on: true,
        }
    }
}

impl SelectionPulse {
    pub fn tick(&mut self, dt: Duration) {
        self.acc += dt;
        let period = Duration::from_millis(SELECTION_PULSE_MS);
        while self.acc >= period {
            self.on = !self.on;
            self.acc -= period;
        }
    }

    /// Emphasis for the selected element at this instant. The structural
    /// anchor (heavy border / marker) stays constant; only this breathes,
    /// between Strong and Normal, so it reads as breathing not flicker.
    pub fn emphasis(&self) -> Emphasis {
        if self.on {
            Emphasis::Strong
        } else {
            Emphasis::Normal
        }
    }
}

/// The player's card-selection cursor: which hand slot is selected, and
/// the pending sign for a plus-or-minus / tiebreaker card. Pure logic
/// over the hand — the arrow-key/Enter interaction model deferred from
/// spec 001. Coexists with the direct number-key + h/l play path.
#[derive(Debug)]
pub struct HandCursor {
    index: usize,
    pending_positive: bool,
}

impl Default for HandCursor {
    fn default() -> Self {
        Self {
            index: 0,
            pending_positive: true,
        }
    }
}

impl HandCursor {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn pending_positive(&self) -> bool {
        self.pending_positive
    }

    /// Move to the next occupied slot to the right, wrapping. Resets the
    /// pending sign to positive.
    pub fn move_right(&mut self, hand: &[Option<Card>]) {
        if let Some(next) = next_occupied(hand, self.index, 1) {
            self.index = next;
        }
        self.pending_positive = true;
    }

    /// Move to the next occupied slot to the left, wrapping.
    pub fn move_left(&mut self, hand: &[Option<Card>]) {
        if let Some(next) = next_occupied(hand, self.index, -1) {
            self.index = next;
        }
        self.pending_positive = true;
    }

    /// Flip the pending sign — only meaningful on a sign-choice card.
    pub fn toggle_sign(&mut self, hand: &[Option<Card>]) {
        if is_sign_choice(hand, self.index) {
            self.pending_positive = !self.pending_positive;
        }
    }

    /// After the hand changes (a card was played), snap the cursor back
    /// onto an occupied slot and reset the pending sign.
    pub fn normalize(&mut self, hand: &[Option<Card>]) {
        if !matches!(hand.get(self.index), Some(Some(_)))
            && let Some(i) = first_occupied(hand)
        {
            self.index = i;
        }
        self.pending_positive = true;
    }
}

fn first_occupied(hand: &[Option<Card>]) -> Option<usize> {
    hand.iter().position(|slot| slot.is_some())
}

fn is_sign_choice(hand: &[Option<Card>], index: usize) -> bool {
    matches!(hand.get(index), Some(Some(c)) if c.sign_choice_magnitude().is_some())
}

/// The next occupied slot from `from`, scanning in `dir` (+1/-1) with
/// wraparound, excluding `from` itself. None if no other occupied slot.
fn next_occupied(hand: &[Option<Card>], from: usize, dir: isize) -> Option<usize> {
    let n = hand.len();
    if n == 0 {
        return None;
    }

    let mut i = from as isize;
    for _ in 0..n {
        i = (i + dir).rem_euclid(n as isize);
        let idx = i as usize;
        if idx == from {
            return None; // wrapped all the way around
        }
        if hand[idx].is_some() {
            return Some(idx);
        }
    }
    None
}

/// Draw the "terminal too small" recovery screen, centered and clipped
/// to whatever space exists. Uses the frame's own dimensions so it works
/// at any size.
fn draw_too_small(frame: &mut Frame, cols: usize, rows: usize) {
    let (min_cols, min_rows) = Config::min_size();
    let lines = [
        "Terminal too small".to_string(),
        format!("Need at least {min_cols} x {min_rows}"),
        format!("Now {cols} x {rows}"),
    ];

    let mid_x = frame.len() / 2;
    let mid_y = frame.first().map_or(0, Vec::len) / 2;

    for (i, line) in lines.iter().enumerate() {
        let y = (mid_y + i).saturating_sub(1);
        draw_text_centered(frame, mid_x, y, line, Emphasis::Alert);
    }
}

/// Commit the cursor-selected card: emit PlayHand, and for a sign-choice
/// card follow immediately with ChooseSign at the pending sign — the
/// engine passes through AwaitingSignChoice and back within one event.
/// Only acts during the player's turn.
fn cursor_confirm(game_state: &mut GameState, cursor: &mut HandCursor) {
    if !matches!(game_state.game_phase, GamePhase::PlayerTurn) {
        return;
    }

    let index = cursor.index();
    let card = match game_state.player.hand.get(index) {
        Some(Some(c)) => *c,
        _ => return,
    };

    game_state.apply_game_action(GameAction::PlayHand { index });
    if card.sign_choice_magnitude().is_some() {
        game_state.apply_game_action(GameAction::ChooseSign {
            positive: cursor.pending_positive(),
        });
    }

    cursor.normalize(&game_state.player.hand);
}

/// Translate the emacs navigation chords (`Ctrl+P/N/B/F`) into the arrow
/// `KeyCode`s they mirror, so every arrow-driven screen responds to them
/// with no per-screen code. Case-folded (terminals vary on the reported
/// case); only `Ctrl`-held keys are touched, so every plain key — and every
/// other `Ctrl` chord — passes through unchanged.
pub fn resolve_key(code: KeyCode, modifiers: KeyModifiers) -> KeyCode {
    if modifiers.contains(KeyModifiers::CONTROL)
        && let KeyCode::Char(c) = code
    {
        match c.to_ascii_lowercase() {
            'p' => return KeyCode::Up,
            'n' => return KeyCode::Down,
            'b' => return KeyCode::Left,
            'f' => return KeyCode::Right,
            _ => {}
        }
    }
    code
}

/// What a confirmed "discard your saved match?" should start. Both Quick Play
/// and Start Campaign raise the confirm over an existing save; this records
/// which one so the Yes branch does the right thing (open opponent select, or
/// discard the save and enter the campaign map).
#[derive(Debug, Clone)]
enum PendingStart {
    QuickPlay,
    Campaign,
}

/// A modal panel shown over the current screen. Exactly one is open at a
/// time — the type enforces what spec 004 spread across two `Option` fields
/// (the "only one is ever Some" invariant the T010 review flagged): the `?`
/// / How to Play help text, the settings panel, or the discard-a-save
/// confirmation.
enum Modal {
    Help(Overlay),
    Settings(SettingsState),
    /// The "discard your saved match?" confirmation shown when starting a new
    /// match (Quick Play or a campaign node) would replace an existing save.
    /// `on_yes` is the highlighted choice, defaulting to No — the safe option;
    /// `pending` is what to start if confirmed.
    ConfirmNewGame { on_yes: bool, pending: PendingStart },
}

pub struct App {
    pub config: Config,
    screen: Screen,
    board_view: BoardView,
    // The one open modal over the current screen, if any (help, settings).
    modal: Option<Modal>,
    // Whether a resumable saved match exists on disk — drives the menu's
    // Continue item. Kept in sync as the game saves/clears, so the menu never
    // does file I/O per frame.
    has_save: bool,
    pulse: SelectionPulse,
    settings: Settings,
    // The player's persistent profile — their card collection and built side
    // deck. Matches deal the player's hand from `profile.deck()`.
    profile: Profile,
    audio: Audio,
    // The last in-game audio snapshot; the next one is diffed against it to
    // decide which SFX to play. None outside a game.
    prev_audio: Option<AudioSnapshot>,
    // Some((cols, rows)) while the terminal is below the minimum size:
    // the game pauses and a recovery message shows until it grows back.
    too_small: Option<(usize, usize)>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let settings = Settings::load();
        let profile = Profile::load();
        let has_save = crate::save::exists();
        Self {
            config,
            screen: Screen::StartMenu {
                menu_state: MenuState::new(has_save),
            },
            board_view: BoardView::new(config),
            modal: None,
            has_save,
            pulse: SelectionPulse::default(),
            audio: Audio::new(settings),
            settings,
            profile,
            prev_audio: None,
            too_small: None,
        }
    }

    /// A fresh start-menu screen reflecting whether a save currently exists.
    fn start_menu(&self) -> Screen {
        Screen::StartMenu {
            menu_state: MenuState::new(self.has_save),
        }
    }

    /// Persist the in-progress match — or clear the save if it's over
    /// (`save` handles the `GameOver` → clear). A no-op off the InGame
    /// screen. `has_save` tracks whether a resumable file now exists, so the
    /// menu's Continue item stays correct without re-reading disk.
    fn save_game(&mut self) {
        if let Screen::InGame { game_state, .. } = &self.screen {
            crate::save::save(game_state);
            self.has_save = !matches!(game_state.game_phase, GamePhase::GameOver { .. });
        }
    }

    /// Open the opponent-select screen — the entry point to a new match
    /// (Start Game leads here, directly or via the discard-save confirm). A
    /// match needs a legal 10-card side deck, so if the built deck is
    /// incomplete the player is sent to the deck-builder instead (its
    /// "Deck: N/10" readout shows the shortfall). The starter deck is a full
    /// 10, so this only diverts a player who deliberately under-filled.
    fn open_opponent_select(&mut self) {
        if !self.profile.deck_is_valid() {
            self.open_deck_builder();
            return;
        }
        self.screen = Screen::OpponentSelect {
            state: OpponentSelectState::new(),
        };
    }

    /// Open the deck-builder screen (from the menu's Side Deck item, or when a
    /// match start finds the deck incomplete).
    fn open_deck_builder(&mut self) {
        self.screen = Screen::DeckBuilder {
            state: DeckBuilderState::new(),
        };
    }

    /// Open the campaign map at the run's current position. The Start Campaign
    /// entry handles discarding a saved match first (with a confirm); this just
    /// switches to the map screen.
    fn open_campaign_map(&mut self) {
        self.screen = Screen::CampaignMap {
            state: CampaignMapState::new(&self.profile),
        };
    }

    /// Launch a campaign match against `planet`/`opponent` (both ids), upholding
    /// `start_match`'s deck-valid precondition (diverting to the deck-builder if
    /// the deck is incomplete). Shared by the map's direct launch (no save) and
    /// the confirmed discard-and-launch.
    fn launch_campaign_node(&mut self, planet: &str, opponent: &str) {
        if !self.profile.deck_is_valid() {
            self.open_deck_builder();
        } else if let Some(opp) = opponent_by_id(opponent) {
            let node = NodeRef {
                planet: planet.to_string(),
                opponent: opponent.to_string(),
            };
            self.start_match(opp, Some(node));
        }
    }

    /// Begin a fresh match against `opponent`, dealing the player's hand from
    /// the profile's built deck, replacing any current match and persisting it.
    ///
    /// Precondition: the profile deck is valid (exactly `SIDE_DECK_SIZE`
    /// cards). This is the only match-start entry, reached solely via
    /// `open_opponent_select`, which enforces `deck_is_valid()` first — any new
    /// caller must uphold that guard, or an undersized deck would deal a short
    /// hand. Its two callers — the opponent-select Pick and the campaign map's
    /// Launch — both enforce it. `campaign` marks the match as a campaign node
    /// (persisted via the profile), or `None` for Quick Play.
    fn start_match(&mut self, opponent: OpponentProfile, campaign: Option<NodeRef>) {
        // Record whether this match belongs to the campaign, and against which
        // node — persisted, so a match resumed via Continue still routes back
        // to the map at game over. Quick Play passes None (clearing any stale
        // pointer).
        self.profile.campaign_mut().set_in_progress(campaign);
        self.profile.save();

        self.screen = Screen::InGame {
            game_state: Box::new(GameState::with_opponent(
                opponent,
                self.profile.deck().to_vec(),
            )),
            cursor: HandCursor::default(),
        };
        // Fresh game — the first snapshot seeds silently, so the empty
        // starting board plays no cues.
        self.prev_audio = None;
        // Persist immediately (overwriting any prior save), so quitting right
        // away still leaves a resumable game and Continue appears next launch.
        self.save_game();
    }

    /// Play the SFX for whatever just changed in the game, by diffing the
    /// current state against the previous snapshot. A no-op outside a game.
    /// The engine never makes a sound; this observes it from the outside.
    fn emit_audio_cues(&mut self) {
        let curr = match &self.screen {
            Screen::InGame { game_state, .. } => AudioSnapshot::of(game_state),
            _ => return,
        };
        if let Some(prev) = self.prev_audio {
            for cue in audio_cues(prev, curr) {
                self.audio.play_cue(cue);
            }
        }
        self.prev_audio = Some(curr);
    }

    /// Re-lay-out for a new (valid) terminal size and resume play. Game
    /// state is untouched — only the presentation is rebuilt.
    pub fn resize(&mut self, config: Config) {
        self.config = config;
        self.board_view = BoardView::new(config);
        if let Some(Modal::Help(overlay)) = &self.modal {
            self.modal = Some(Modal::Help(Overlay::new(overlay.kind(), config)));
        }
        self.too_small = None;
    }

    /// Enter the "terminal too small" state, pausing the game. State is
    /// preserved; `resize` restores it when the terminal grows back.
    pub fn set_too_small(&mut self, cols: usize, rows: usize) {
        self.too_small = Some((cols, rows));
    }

    pub fn is_too_small(&self) -> bool {
        self.too_small.is_some()
    }

    /// Route the input key to the appropriate handler
    ///
    pub fn handle_key(&mut self, key: KeyCode) {
        // Game paused while the terminal is too small — ignore input
        // (the global quit key is handled in the game loop)
        if self.too_small.is_some() {
            return;
        }

        // Global mute — works on every screen and under an overlay.
        if key == KeyCode::Char('m') {
            self.audio.toggle_mute();
            return;
        }

        // A modal (settings panel or help overlay) sits over the current
        // screen and takes all input while open. The settings panel routes
        // to its own handler; a help overlay is dismissed with ?, Esc, Enter,
        // or Space — the last so How to Play (opened from the menu with Space)
        // closes with the same key it opened on. Closing sounds the back cue.
        if matches!(self.modal, Some(Modal::Settings(_))) {
            self.handle_settings_input(key);
        } else if matches!(self.modal, Some(Modal::Help(_))) {
            if matches!(
                key,
                KeyCode::Char('?') | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Enter
            ) {
                self.modal = None;
                self.audio.play(Sfx::MenuBack);
            }
        } else if matches!(self.modal, Some(Modal::ConfirmNewGame { .. })) {
            self.handle_confirm_input(key);
        } else {
            // No modal open: ? opens the help overlay for the current screen.
            if let KeyCode::Char(c) = key
                && c == '?'
            {
                self.modal = match &self.screen {
                    Screen::StartMenu { .. } => {
                        Some(Modal::Help(Overlay::new(OverlayKind::MenuHelp, self.config)))
                    }
                    Screen::InGame { .. } => {
                        Some(Modal::Help(Overlay::new(OverlayKind::GameHelp, self.config)))
                    }
                    // The select and deck-builder screens carry their own
                    // on-screen hint lines, so ? opens no overlay there.
                    Screen::OpponentSelect { .. }
                    | Screen::DeckBuilder { .. }
                    | Screen::CampaignMap { .. } => None,
                };
            }

            // A finished campaign match returns to the map on an acknowledgement
            // key (the win was already recorded in tick); there's no quick-play
            // rematch, and the in-progress pointer is cleared here.
            if matches!(
                key,
                KeyCode::Enter
                    | KeyCode::Char(' ')
                    | KeyCode::Char('g')
                    | KeyCode::Char('x')
                    | KeyCode::Esc
            ) && matches!(&self.screen, Screen::InGame { game_state, .. }
                    if matches!(game_state.game_phase, GamePhase::GameOver { .. }))
                && self.profile.campaign().in_progress().is_some()
            {
                self.profile.campaign_mut().set_in_progress(None);
                self.profile.save();
                self.audio.play(Sfx::MenuSelect);
                self.open_campaign_map();
                return;
            }

            // Track whether a player action changed the game this key, so we
            // persist once afterward (cursor moves don't touch saved state).
            let mut game_changed = false;
            match &mut self.screen {
                // Route the Menu inputs only to Menu
                Screen::StartMenu { menu_state } => match menu_state.handle_input(key) {
                    Some(MenuOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                    Some(MenuOutcome::Activated(item)) => {
                        self.audio.play(Sfx::MenuSelect);
                        self.activate_menu_item(item);
                    }
                    None => {}
                },

                // Route the game inputs to game_state. The cursor model
                // (arrows + Enter/Space) and the direct keys (1-4, d/s, h/l)
                // coexist — cursor keys act only on the player's turn.
                Screen::InGame { game_state, cursor } => {
                    let player_turn = matches!(game_state.game_phase, GamePhase::PlayerTurn);
                    match key {
                        // Esc or X quits the game back to the main menu
                        KeyCode::Char('x') | KeyCode::Esc => {
                            self.screen = self.start_menu();
                        }
                        KeyCode::Left if player_turn => cursor.move_left(&game_state.player.hand),
                        KeyCode::Right if player_turn => cursor.move_right(&game_state.player.hand),
                        KeyCode::Up | KeyCode::Down if player_turn => {
                            cursor.toggle_sign(&game_state.player.hand)
                        }
                        // Space mirrors Enter here: it's the "select /
                        // confirm" key everywhere else, so on the player's
                        // turn it plays the highlighted card. Drawing has its
                        // own dedicated key (D).
                        KeyCode::Enter | KeyCode::Char(' ') if player_turn => {
                            cursor_confirm(game_state, cursor);
                            game_changed = true;
                        }
                        KeyCode::Char(c) => {
                            if let Some(game_action) = game_state.handle_game_input(c) {
                                game_state.apply_game_action(game_action);
                                cursor.normalize(&game_state.player.hand);
                                game_changed = true;
                            }
                        }
                        _ => {}
                    }
                }

                // The opponent-select screen: navigate the roster, pick to
                // start the match, or back out to the menu. The outcome is an
                // owned value, so the borrow of `state` ends before the
                // `&mut self` calls below (same NLL pattern as the menu arm).
                Screen::OpponentSelect { state } => match state.handle_input(key) {
                    Some(SelectOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                    Some(SelectOutcome::Picked(opponent)) => {
                        self.audio.play(Sfx::MenuSelect);
                        self.start_match(opponent, None); // Quick Play — not a campaign match
                    }
                    Some(SelectOutcome::Back) => {
                        self.audio.play(Sfx::MenuBack);
                        self.screen = self.start_menu();
                    }
                    None => {}
                },

                // The deck-builder: move over the collection grid, add/remove a
                // copy of the highlighted card (applied through the profile,
                // which enforces the own-a-copy + 10-card rules), or leave. The
                // outcome is owned, so `state`'s borrow ends before the
                // `&mut self` edits below (same NLL pattern as the arms above);
                // `&self.profile` is a disjoint field, so reading it for the
                // scrutinee is fine alongside `&mut self.screen`.
                Screen::DeckBuilder { state } => match state.handle_input(key, &self.profile) {
                    Some(BuildOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                    Some(BuildOutcome::Add(card)) => {
                        // Persist only edits that took effect — a rejected add
                        // (deck full or no spare copy owned) changes nothing.
                        if self.profile.try_add_to_deck(card) {
                            self.audio.play(Sfx::MenuSelect);
                            self.profile.save();
                        } else {
                            self.audio.play(Sfx::MenuBack);
                        }
                    }
                    Some(BuildOutcome::Remove(card)) => {
                        if self.profile.remove_from_deck(card) {
                            self.audio.play(Sfx::MenuSelect);
                            self.profile.save();
                        } else {
                            self.audio.play(Sfx::MenuBack); // none in the deck
                        }
                    }
                    Some(BuildOutcome::Back) => {
                        self.audio.play(Sfx::MenuBack);
                        self.screen = self.start_menu();
                    }
                    None => {}
                },

                // The campaign map: travel between unlocked planets, launch a
                // match against a planet's next opponent, or back out. (T002
                // launches the match; the campaign progress spine — the
                // in-progress pointer and the win seam — arrives in T003.)
                Screen::CampaignMap { state } => match state.handle_input(key, &self.profile) {
                    Some(MapOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                    Some(MapOutcome::Launch { planet, opponent }) => {
                        // No save-guard here: entering the campaign already
                        // discarded any saved match (the prompt lives at entry),
                        // so a launch from the map never has one to overwrite.
                        self.audio.play(Sfx::MenuSelect);
                        self.launch_campaign_node(planet, opponent);
                    }
                    Some(MapOutcome::Back) => {
                        self.audio.play(Sfx::MenuBack);
                        self.screen = self.start_menu();
                    }
                    None => {}
                },
            }

            if game_changed {
                self.save_game();
            }
        }

        // After any input, sound whatever just changed in the game.
        self.emit_audio_cues();
    }

    /// Route a key to the open settings panel: move between rows, adjust the
    /// selected channel's volume (updating audio + persisting immediately),
    /// or close the panel back to the menu with the back cue. The menu
    /// underneath is untouched, so its selection survives.
    fn handle_settings_input(&mut self, key: KeyCode) {
        let Some(Modal::Settings(state)) = self.modal.as_ref() else {
            return;
        };
        let Some(action) = state.handle_input(key) else {
            return;
        };
        match action {
            SettingsAction::Up => {
                if let Some(Modal::Settings(s)) = self.modal.as_mut() {
                    s.move_up();
                }
                self.audio.play(Sfx::MenuMove);
            }
            SettingsAction::Down => {
                if let Some(Modal::Settings(s)) = self.modal.as_mut() {
                    s.move_down();
                }
                self.audio.play(Sfx::MenuMove);
            }
            SettingsAction::Louder | SettingsAction::Quieter => {
                let delta = if matches!(action, SettingsAction::Louder) {
                    VOLUME_STEP
                } else {
                    -VOLUME_STEP
                };
                if let Some(Modal::Settings(s)) = self.modal.as_ref() {
                    let vol = match s.selected() {
                        SettingRow::Music => &mut self.settings.music_volume,
                        SettingRow::Sfx => &mut self.settings.sfx_volume,
                    };
                    *vol = (*vol + delta).clamp(0.0, 1.0);
                }
                self.audio.set_settings(self.settings);
                self.settings.save();
                // A tick after set_settings so you hear the new SFX level
                // (the music change is already live).
                self.audio.play(Sfx::MenuMove);
            }
            SettingsAction::Back => {
                self.modal = None;
                self.audio.play(Sfx::MenuBack);
            }
        }
    }

    /// Route a key to the discard-a-save confirmation: ←/→ (a/d) toggle
    /// between No and Yes, Enter/Space commit the highlighted choice, Esc
    /// cancels. Yes carries out the pending start — Quick Play opens opponent
    /// select, Start Campaign discards the save and opens the map; No / Esc
    /// close with the save intact.
    fn handle_confirm_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left | KeyCode::Right | KeyCode::Char('a') | KeyCode::Char('d') => {
                if let Some(Modal::ConfirmNewGame { on_yes, .. }) = self.modal.as_mut() {
                    *on_yes = !*on_yes;
                }
                self.audio.play(Sfx::MenuMove);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Only when Yes is highlighted do we carry out the pending start
                // (which overwrites the save); No / Esc leave it intact.
                let pending = match &self.modal {
                    Some(Modal::ConfirmNewGame { on_yes: true, pending }) => Some(pending.clone()),
                    _ => None,
                };
                self.modal = None;
                match pending {
                    Some(PendingStart::QuickPlay) => {
                        self.audio.play(Sfx::MenuSelect);
                        self.open_opponent_select();
                    }
                    Some(PendingStart::Campaign) => {
                        // Discard the saved match, then enter the map.
                        self.audio.play(Sfx::MenuSelect);
                        crate::save::clear();
                        self.has_save = false;
                        self.open_campaign_map();
                    }
                    None => self.audio.play(Sfx::MenuBack),
                }
            }
            KeyCode::Esc => {
                self.modal = None;
                self.audio.play(Sfx::MenuBack);
            }
            _ => {}
        }
    }

    /// Act on an activated start-menu item — open a screen or a modal.
    fn activate_menu_item(&mut self, menu_item: MenuItem) {
        match menu_item {
            MenuItem::Continue => {
                // Resume the saved match. Continue only appears when a save
                // exists, but a race or corruption could still yield None —
                // then it's a no-op, not a crash.
                if let Some(game) = crate::save::load() {
                    // The cursor isn't saved. Snap it onto a real hand card:
                    // the default index 0 may now be empty (that card was
                    // played), which would show the empty-hand prompt on the
                    // resumed board instead of a selection.
                    let mut cursor = HandCursor::default();
                    cursor.normalize(&game.player.hand);
                    self.screen = Screen::InGame {
                        game_state: Box::new(game),
                        cursor,
                    };
                    // Resuming mid-match: seed the audio snapshot silently so
                    // the restored board doesn't replay cues for cards already
                    // on the table.
                    self.prev_audio = None;
                }
            }
            MenuItem::StartCampaign => {
                if self.has_save {
                    // The discard prompt lives at campaign entry (human-ruled):
                    // entering discards the saved match, so a launch from the
                    // map later never has one to overwrite.
                    self.modal = Some(Modal::ConfirmNewGame {
                        on_yes: false,
                        pending: PendingStart::Campaign,
                    });
                } else {
                    self.open_campaign_map();
                }
            }
            MenuItem::QuickPlay => {
                if self.has_save {
                    // Starting fresh would discard the saved match — confirm.
                    self.modal = Some(Modal::ConfirmNewGame {
                        on_yes: false,
                        pending: PendingStart::QuickPlay,
                    });
                } else {
                    self.open_opponent_select();
                }
            }
            MenuItem::SideDeck => {
                // Build your side deck — independent of any match; the deck
                // persists and the next match deals from it.
                self.open_deck_builder();
            }
            MenuItem::HowToPlay => {
                self.modal = Some(Modal::Help(Overlay::new(OverlayKind::HowToPlay, self.config)));
            }
            MenuItem::Settings => {
                // Open Settings as an overlay over the menu — the menu (and
                // its selection) stays put underneath, like How to Play.
                self.modal = Some(Modal::Settings(SettingsState::default()));
            }
        }
    }

    /// Call tick on each sub-screen
    ///
    pub fn tick(&mut self, dt: Duration) {
        // One pulse drives every screen's selection breathe
        self.pulse.tick(dt);

        let mut phase_changed = false;
        if let Screen::InGame { game_state, .. } = &mut self.screen {
            let before = std::mem::discriminant(&game_state.game_phase);
            game_state.update();
            phase_changed = std::mem::discriminant(&game_state.game_phase) != before;
        }

        // Advance the campaign map's starfield twinkle — its own slow clock,
        // separate from the selection pulse (per the amended Motion rule).
        if let Screen::CampaignMap { state } = &mut self.screen {
            state.tick(dt);
        }

        // A phase change means the opponent moved or the round/game resolved
        // — persist the new position (save clears the file on GameOver).
        if phase_changed {
            self.save_game();
        }

        // Record a campaign win the first time game over is seen with the player
        // as winner. Checked every tick (not only the phase transition), so it
        // can't be missed however game over is reached; the is_opponent_beaten
        // guard fires it exactly once and mark_beaten is idempotent regardless.
        // `in_progress` is cleared only on the player's acknowledgement (the
        // InGame input arm); a loss records nothing and keeps the node open.
        if let Screen::InGame { game_state, .. } = &self.screen
            && matches!(game_state.game_phase, GamePhase::GameOver { winner: Player::Player })
            && let Some(node) = self.profile.campaign().in_progress().cloned()
            && !self.profile.campaign().is_opponent_beaten(&node.planet, &node.opponent)
        {
            self.profile.campaign_mut().mark_beaten(&node.planet, &node.opponent);
            self.profile.save();
        }

        // Sound the opponent's moves and round/game resolutions, which
        // happen here in the update rather than from a player keypress.
        self.emit_audio_cues();
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if let Some((cols, rows)) = self.too_small {
            draw_too_small(frame, cols, rows);
            return;
        }

        let pulse = self.pulse.emphasis();
        match &self.screen {
            Screen::StartMenu { menu_state } => menu_state.draw(frame, &self.config, pulse),
            Screen::InGame { game_state, cursor } => {
                self.board_view.draw(game_state, cursor, pulse, frame)
            }
            Screen::OpponentSelect { state } => state.draw(frame, &self.config, pulse),
            Screen::DeckBuilder { state } => state.draw(frame, &self.config, &self.profile, pulse),
            Screen::CampaignMap { state } => state.draw(frame, &self.config, &self.profile, pulse),
        }

        // The one open modal draws over the screen.
        match &self.modal {
            Some(Modal::Settings(state)) => {
                state.draw_overlay(frame, &self.config, self.settings, pulse)
            }
            Some(Modal::Help(overlay)) => overlay.draw(frame),
            Some(Modal::ConfirmNewGame { on_yes, .. }) => {
                self.draw_confirm_new_game(*on_yes, pulse, frame)
            }
            None => {}
        }
    }

    /// Draw the discard-a-save confirmation as a bordered overlay over the
    /// menu, matching How to Play / Settings: a prompt, the Yes / No choices
    /// (the highlighted one marked and breathing with the pulse), and a hint.
    fn draw_confirm_new_game(&self, on_yes: bool, pulse: Emphasis, frame: &mut Frame) {
        let title = "Discard your saved match?";
        let hint = "←/→ choose  ·  Enter confirm  ·  Esc cancel";

        let content_width = title.chars().count().max(hint.chars().count());
        let layout = OverlayLayout::new(self.config, content_width, 5);

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
        draw_text_in(frame, layout.inner, 0, Align::Center, title, Emphasis::Normal);

        // Yes and No centered on one row; the highlighted choice carries the
        // ▸ marker (constant, in place) and the shared pulse. The unselected
        // choice keeps two leading spaces so the marker never shifts the text.
        let inner = layout.inner;
        let row_y = inner.y0 + 2;
        let block_w = "▸ Yes".chars().count() + 6 + "▸ No".chars().count();
        let start_x = inner.x0 + inner.width().saturating_sub(block_w) / 2;
        let no_x = start_x + "▸ Yes".chars().count() + 6;

        let yes = if on_yes { "▸ Yes" } else { "  Yes" };
        let no = if on_yes { "  No" } else { "▸ No" };
        draw_text(frame, start_x, row_y, yes, if on_yes { pulse } else { Emphasis::Normal });
        draw_text(frame, no_x, row_y, no, if on_yes { Emphasis::Normal } else { pulse });

        draw_text_in(frame, inner, 4, Align::Center, hint, Emphasis::Muted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, FlipKind};
    use crate::game::GamePhase;

    fn hand(cards: &[Option<Card>]) -> Vec<Option<Card>> {
        cards.to_vec()
    }

    #[test]
    fn resolve_key_maps_emacs_chords_to_arrows() {
        let ctrl = KeyModifiers::CONTROL;
        assert_eq!(resolve_key(KeyCode::Char('p'), ctrl), KeyCode::Up);
        assert_eq!(resolve_key(KeyCode::Char('n'), ctrl), KeyCode::Down);
        assert_eq!(resolve_key(KeyCode::Char('b'), ctrl), KeyCode::Left);
        assert_eq!(resolve_key(KeyCode::Char('f'), ctrl), KeyCode::Right);
        // Case-folded — some terminals report the chord's letter uppercase.
        assert_eq!(resolve_key(KeyCode::Char('P'), ctrl), KeyCode::Up);
    }

    #[test]
    fn resolve_key_passes_through_everything_else() {
        // A bare letter (no Ctrl) is unchanged...
        assert_eq!(
            resolve_key(KeyCode::Char('p'), KeyModifiers::NONE),
            KeyCode::Char('p')
        );
        // ...an arrow key is unchanged...
        assert_eq!(resolve_key(KeyCode::Up, KeyModifiers::NONE), KeyCode::Up);
        // ...and a Ctrl chord that isn't one of p/n/b/f passes through.
        assert_eq!(
            resolve_key(KeyCode::Char('x'), KeyModifiers::CONTROL),
            KeyCode::Char('x')
        );
    }

    #[test]
    fn cursor_move_right_skips_empty_slots_and_wraps() {
        // occupied at 0 and 2; 1 and 3 empty
        let h = hand(&[Some(Card::Plus(2)), None, Some(Card::Minus(4)), None]);
        let mut c = HandCursor::default(); // index 0

        c.move_right(&h);
        assert_eq!(c.index(), 2);
        c.move_right(&h); // wraps past 3 back to 0
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn cursor_move_left_skips_empty_and_wraps() {
        let h = hand(&[Some(Card::Plus(2)), None, Some(Card::Minus(4)), None]);
        let mut c = HandCursor::default();

        c.move_left(&h); // from 0 wraps left to 2
        assert_eq!(c.index(), 2);
        c.move_left(&h);
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn cursor_single_occupied_slot_does_not_move() {
        let h = hand(&[None, Some(Card::Plus(2)), None, None]);
        let mut c = HandCursor::default();
        c.normalize(&h); // snaps to index 1
        assert_eq!(c.index(), 1);

        c.move_right(&h);
        assert_eq!(c.index(), 1);
        c.move_left(&h);
        assert_eq!(c.index(), 1);
    }

    #[test]
    fn cursor_empty_hand_moves_are_noops() {
        let h = hand(&[None, None, None, None]);
        let mut c = HandCursor::default();
        c.move_right(&h);
        c.move_left(&h);
        assert_eq!(c.index(), 0); // unchanged, no panic
    }

    #[test]
    fn cursor_toggle_sign_only_on_sign_choice_cards() {
        let h = hand(&[Some(Card::PlusMinus(3)), Some(Card::Plus(2)), None, None]);
        let mut c = HandCursor::default(); // index 0, PlusMinus

        assert!(c.pending_positive());
        c.toggle_sign(&h);
        assert!(!c.pending_positive());

        // move to a fixed card — toggling does nothing
        c.move_right(&h); // to index 1 (Plus), resets sign to positive
        assert!(c.pending_positive());
        c.toggle_sign(&h);
        assert!(c.pending_positive());
    }

    #[test]
    fn cursor_moving_resets_pending_sign() {
        let h = hand(&[Some(Card::PlusMinus(3)), Some(Card::Tiebreaker), None, None]);
        let mut c = HandCursor::default();
        c.toggle_sign(&h); // negative
        assert!(!c.pending_positive());
        c.move_right(&h);
        assert!(c.pending_positive()); // reset on move
    }

    #[test]
    fn cursor_normalize_snaps_off_an_emptied_slot() {
        let mut h = hand(&[Some(Card::Plus(2)), Some(Card::Minus(4)), None, None]);
        let mut c = HandCursor::default();
        c.move_right(&h); // index 1
        assert_eq!(c.index(), 1);

        h[1] = None; // that card was played
        c.normalize(&h);
        assert_eq!(c.index(), 0); // snapped back to the remaining card
    }

    // --- confirm emits the right actions through the engine ---

    fn game_with_hand(cards: &[Option<Card>]) -> GameState {
        let mut gs = GameState::new();
        gs.player.hand = cards.to_vec();
        gs
    }

    #[test]
    fn cursor_confirm_plays_a_fixed_card_immediately() {
        let mut gs = game_with_hand(&[Some(Card::Plus(5)), None, None, None]);
        let mut c = HandCursor::default();

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_none());
        assert_eq!(gs.player.played_row[0].value, 5);
        // fixed card commits without lingering in a sign phase
        assert!(!matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
    }

    #[test]
    fn cursor_confirm_plays_sign_card_at_the_pending_sign() {
        let mut gs = game_with_hand(&[Some(Card::PlusMinus(3)), None, None, None]);
        let mut c = HandCursor::default();
        c.toggle_sign(&gs.player.hand); // choose negative

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_none());
        assert_eq!(gs.player.played_row[0].value, -3);
        // committed in one event — not left waiting on h/l, and playing a
        // card keeps the player's turn (spec-001 ruling)
        assert!(!matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn cursor_confirm_tiebreaker_commits_as_pending_sign() {
        let mut gs = game_with_hand(&[Some(Card::Tiebreaker), None, None, None]);
        let mut c = HandCursor::default(); // positive

        cursor_confirm(&mut gs, &mut c);

        assert_eq!(gs.player.played_row[0].card, Card::Tiebreaker);
        assert_eq!(gs.player.played_row[0].value, 1);
    }

    #[test]
    fn cursor_confirm_flip_card_applies_and_does_not_prompt() {
        let mut gs = game_with_hand(&[Some(Card::Flip(FlipKind::TwoFour)), None, None, None]);
        let mut c = HandCursor::default();

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_none());
        assert!(!matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
    }

    #[test]
    fn cursor_confirm_does_nothing_off_the_players_turn() {
        let mut gs = game_with_hand(&[Some(Card::Plus(5)), None, None, None]);
        gs.game_phase = GamePhase::OpponentThinking {
            until: std::time::Instant::now(),
        };
        let mut c = HandCursor::default();

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_some()); // untouched
        assert!(gs.player.played_row.is_empty());
    }

    // --- selection pulse ---

    fn period() -> Duration {
        Duration::from_millis(crate::SELECTION_PULSE_MS)
    }

    #[test]
    fn pulse_toggles_phase_each_period() {
        let mut p = SelectionPulse::default();
        assert_eq!(p.emphasis(), Emphasis::Strong); // starts "on"
        p.tick(period());
        assert_eq!(p.emphasis(), Emphasis::Normal);
        p.tick(period());
        assert_eq!(p.emphasis(), Emphasis::Strong);
    }

    #[test]
    fn pulse_accumulates_small_ticks_and_carries_remainder() {
        let mut p = SelectionPulse::default();
        let half = period() / 2;

        p.tick(half); // half a period — no toggle yet
        assert_eq!(p.emphasis(), Emphasis::Strong);
        p.tick(half); // full period reached — toggles
        assert_eq!(p.emphasis(), Emphasis::Normal);
        p.tick(half); // remainder was ~0; half alone doesn't toggle
        assert_eq!(p.emphasis(), Emphasis::Normal);
        p.tick(half);
        assert_eq!(p.emphasis(), Emphasis::Strong);
    }

    #[test]
    fn pulse_large_tick_lands_on_the_right_phase() {
        let mut p = SelectionPulse::default();
        p.tick(period() * 3); // odd number of toggles
        assert_eq!(p.emphasis(), Emphasis::Normal);
    }

    // --- resize / too-small state ---

    #[test]
    fn resize_too_small_pauses_then_a_valid_resize_resumes() {
        let big = Config { num_cols: 120, num_rows: 40 };
        let mut app = App::new(big);
        assert!(!app.is_too_small());

        app.set_too_small(30, 10);
        assert!(app.is_too_small());
        // Input is ignored while too small (no panic, no state change)
        app.handle_key(KeyCode::Enter);
        assert!(app.is_too_small());

        app.resize(big);
        assert!(!app.is_too_small());
    }
}
