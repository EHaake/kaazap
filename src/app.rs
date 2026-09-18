use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    SELECTION_PULSE_MS,
    audio::{Audio, AudioSnapshot, Sfx, audio_cues},
    banter::{
        BanterSnapshot, banter_event, banter_for, lines_for, match_restarted, pick, play_resumed,
    },
    board::BoardView,
    campaign::{NodeRef, PLANETS, planet_by_id},
    campaign_map::{CampaignMapState, MapBanner, MapOutcome},
    card::Card,
    config::Config,
    deck_builder::{BuildOutcome, BuilderOrigin, DeckBuilderState},
    economy::StakeOutcome,
    economy,
    frame::{
        Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text, draw_text_centered,
        draw_text_in,
    },
    game::{GameAction, GamePhase, GameState},
    layout::OverlayLayout,
    menu::{MenuItem, MenuOutcome, MenuState},
    opponent::{OpponentProfile, opponent_by_id},
    opponent_select::{OpponentSelectState, SelectOutcome},
    overlay::{Overlay, OverlayKind, draw_scrollable_overlay, draw_text_overlay, overlay_text},
    play_log::PlayLog,
    player::Player,
    profile::Profile,
    records::{RecordsOutcome, RecordsState},
    screen::Screen,
    settings::{SettingRow, Settings, SettingsAction, SettingsState},
    shop::{ShopOutcome, ShopState},
    stats::{RunStats, run_summary_lines},
    wager::{WagerOutcome, WagerState},
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
/// spec 001. Since spec 023 it is the only way a card is played: the
/// number keys select through it too.
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

    /// Select `index` if that slot is occupied (1–4, spec 023) — then does
    /// exactly what `move_left` / `move_right` do after landing: sets the index
    /// and resets the pending sign to positive (selecting the already-selected
    /// slot resets it too, as an arrow that lands on the same slot does). An
    /// empty or out-of-range slot changes nothing, sign included.
    pub fn select(&mut self, index: usize, hand: &[Option<Card>]) {
        if matches!(hand.get(index), Some(Some(_))) {
            self.index = index;
            self.pending_positive = true;
        }
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
    /// Shown at campaign entry when there is anything the choices would affect —
    /// the run has progress, or the pool differs from the starter: Continue /
    /// New Campaign / Reset Everything (spec 024, superseding spec 014's
    /// two-choice panel). `choice` is the highlighted one, defaulting to
    /// Continue — the safe option.
    CampaignEntry { choice: CampaignChoice },
    /// The destructive confirmation behind New Campaign (the map only) and Reset
    /// Everything (the full wipe back to the starter) — `scope` says which.
    /// `on_yes` is the highlighted choice, defaulting to No — the safe option
    /// (spec 014's `confirm_choice` seam, unchanged).
    ConfirmReset { on_yes: bool, scope: ResetScope },
    /// The in-match move-history overlay (spec 018). Unit-like — it carries no
    /// data; its content is rebuilt from live state on each draw.
    PlayLog,
    /// The read-only stats/records overlay (spec 020), opened from the start
    /// menu like How to Play / Settings and dismissed back to it. Holds its own
    /// view + scroll cursor; content is rebuilt from the profile each draw.
    Records(RecordsState),
    /// The wager prompt (spec 021), opened over the campaign map when a node is
    /// launched: it holds the match being staked and the chosen stake until the
    /// player commits (starting the match) or cancels back to the map.
    Wager(WagerState),
    /// The run-over notice (spec 021): raised over a freshly opened campaign map
    /// when the balance can no longer cover any ante. Unit-like — it carries no
    /// data. Acknowledging it wipes to a fresh starter run and returns to the
    /// start menu (chore 2026-09-13, supersedes spec 021's "a fresh map opens");
    /// nothing dismisses it (Esc does not), because the run really is over.
    RunOver,
    /// The victory notice (spec 024): raised over the campaign map when the
    /// player acknowledges the game-over popup of the match that completed the
    /// run. Unit-like — it carries no data; its content is rebuilt from the run
    /// tally on each draw. Enter, Space or Esc dismiss it; nothing else acts
    /// while it is up. Transient: no seen mark, so quitting under it loses the
    /// notice but not the completion or its payout, both already persisted.
    Victory,
    /// The first-run campaign primer (spec 023): raised over a freshly opened
    /// campaign map the first time the player reaches it from the start menu,
    /// naming the stake loop and the outfitter. Unit-like — it carries no data;
    /// its text is compiled in with `include_str!` and rebuilt into lines on
    /// each draw, not re-read from disk. Enter/Space/Esc dismiss it, marking
    /// the profile so it shows once.
    Primer,
    /// The first-match popup (spec 023): raised over the dealt board the first
    /// time this profile *starts* a match (never on a resume), naming the keys.
    /// Unit-like — it carries no data. The match is held while it is up (`tick`
    /// skips the engine update); Enter/Space/Esc dismiss it, marking the profile.
    FirstMatch,
}

/// A choice on the campaign-entry panel (spec 024): resume the run, replay the
/// map keeping the pool, or wipe everything back to the starter. Listed in the
/// order the panel draws them, Continue first — the safe option it opens on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CampaignChoice {
    Continue,
    NewCampaign,
    ResetEverything,
}

impl CampaignChoice {
    /// Every choice, in the order the panel draws them.
    const ALL: [Self; 3] = [Self::Continue, Self::NewCampaign, Self::ResetEverything];

    fn label(self) -> &'static str {
        match self {
            Self::Continue => "Continue",
            Self::NewCampaign => "New Campaign",
            Self::ResetEverything => "Reset Everything",
        }
    }

    /// The next choice in the given direction, wrapping at both ends — the
    /// two-choice panel's toggle generalized to three (spec 024). A pure mapping
    /// in the `confirm_choice` spirit, so the highlight's movement is
    /// unit-testable without an `App`.
    fn step(self, forward: bool) -> Self {
        let n = Self::ALL.len();
        let i = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        Self::ALL[(if forward { i + 1 } else { i + n - 1 }) % n]
    }
}

/// How much a confirmed reset takes (spec 024): New Campaign resets the map
/// only — credits, collection and deck stay — while Reset Everything wipes back
/// to the starter, which is what New Campaign did before this spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResetScope {
    MapOnly,
    Everything,
}

/// The effect of a key on a two-choice Yes/No confirmation — a pure mapping, so
/// the *irreversible* choice (Reset Everything's wipe, and New Campaign's map
/// reset — spec 024) is unit-testable without constructing an `App` (the
/// `cursor_confirm` pattern: a tested decision pulled out of a handler).
/// `Commit` is returned **only** for Enter/Space with Yes highlighted; No and
/// Esc `Cancel`, arrows `Toggle`, anything else is `Ignore`.
#[derive(Debug, PartialEq, Eq)]
enum ConfirmChoice {
    Toggle,
    Commit,
    Cancel,
    Ignore,
}

fn confirm_choice(on_yes: bool, key: KeyCode) -> ConfirmChoice {
    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Char('a') | KeyCode::Char('d') => {
            ConfirmChoice::Toggle
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if on_yes {
                ConfirmChoice::Commit
            } else {
                ConfirmChoice::Cancel
            }
        }
        KeyCode::Esc => ConfirmChoice::Cancel,
        _ => ConfirmChoice::Ignore,
    }
}

/// Whether a key acknowledges the run-over notice (spec 021) — a pure mapping in
/// the `confirm_choice` spirit, so the one-way reset is unit-testable without an
/// `App`. Only Enter/Space acknowledge: Esc deliberately does **not** dismiss the
/// notice, since there is nothing to go back to.
fn run_over_acknowledged(key: KeyCode) -> bool {
    matches!(key, KeyCode::Enter | KeyCode::Char(' '))
}

/// Whether a key dismisses a notice the player may wave away — the first-run
/// primer, the first-match popup (spec 023) and the victory notice (spec 024) —
/// a pure mapping in the `run_over_acknowledged` spirit, so the bindings are
/// unit-testable without an `App`. Enter, Space and Esc dismiss; every other key
/// is ignored. (Unlike the run-over notice, Esc *does* dismiss: none of these
/// asks the player for a decision.)
fn notice_dismissed(key: KeyCode) -> bool {
    matches!(key, KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc)
}

/// What a freshly opened campaign map raises (specs 021 + 023 + 024): the
/// run-over notice if the balance can no longer cover any ante, else the victory
/// notice if a completed run is waiting to be announced, else the first-run
/// primer if it is due, else nothing. One modal at a time, so a map that opens
/// broke shows the notice and the primer waits for the next open. A completing
/// win can never leave the player broke and the primer is menu-entry only, so
/// the precedence never actually arbitrates — it is written down so it can't
/// drift.
fn map_entry_modal(broke: bool, victory_due: bool, primer_due: bool) -> Option<Modal> {
    if broke {
        Some(Modal::RunOver)
    } else if victory_due {
        Some(Modal::Victory)
    } else if primer_due {
        Some(Modal::Primer)
    } else {
        None
    }
}

/// The victory notice's content (spec 024), title first and dismiss line last,
/// blank rows included — pure, so the wording, the breathing room and the fit
/// are testable without a terminal. The run summary is the same block the
/// run-over notice shows, built once in `stats`.
fn victory_notice_lines(run: &RunStats, cleared: usize, total: usize) -> Vec<String> {
    let mut lines = vec![
        "Campaign complete — the house's best has lost.".to_string(),
        String::new(),
        "Your cards and credits are yours to keep.".to_string(),
        "Rematches stay open; New Campaign replays the map with your deck.".to_string(),
        String::new(),
    ];
    lines.extend(run_summary_lines(run, cleared, total));
    // The dismiss line is the acted-on element, so it gets an empty row above
    // it; the box's own padding gives it the one below (design brief §Density
    // and breathing room).
    lines.push(String::new());
    lines.push("Enter  continue".to_string());
    lines
}

/// The run-over notice's content (spec 021), now carrying the same run summary
/// between its title and its reset note (spec 024). Same shape and same rules as
/// `victory_notice_lines`.
fn run_over_notice_lines(run: &RunStats, cleared: usize, total: usize) -> Vec<String> {
    let mut lines = vec!["You're broke — the run is over.".to_string(), String::new()];
    lines.extend(run_summary_lines(run, cleared, total));
    lines.push(String::new());
    lines.push(
        "Deck, collection, and progress reset to the starter; your records stay.".to_string(),
    );
    lines.push(String::new());
    lines.push("Enter  continue".to_string());
    lines
}

/// The gap between adjacent choices on a choice panel's row.
const CHOICE_GAP: usize = 6;

/// The width of a choice panel's choice row: every label with its marker (▸, or
/// the two spaces that keep the marker from shifting the text), plus the gaps
/// between them. Pure, so the three-choice row's fit is testable without a
/// terminal (spec 024).
fn choice_row_width(labels: &[&str]) -> usize {
    let text: usize = labels.iter().map(|l| l.chars().count() + 2).sum();
    text + CHOICE_GAP * labels.len().saturating_sub(1)
}

/// How wide a choice panel's content is: the widest of its title, its note, its
/// hint and its choice row. Pure, so a panel's fit is measured by the test
/// through the same expression the draw uses (spec 024).
fn choice_panel_width(title: &str, note: Option<&str>, hint: &str, labels: &[&str]) -> usize {
    title
        .chars()
        .count()
        .max(note.map_or(0, |n| n.chars().count()))
        .max(hint.chars().count())
        .max(choice_row_width(labels))
}

/// Where a choice panel's rows sit, and how tall its content is (spec 024,
/// design brief §Density and breathing room): the choice row is the acted-on
/// element, so it keeps a blank row above and below it — which makes a panel
/// carrying a note one row taller rather than letting the note sit flush against
/// the choices. Returns `(note_row, choice_row, hint_row, height)`; `note_row`
/// is drawn on only when there is a note.
fn choice_rows(note_present: bool) -> (usize, usize, usize, usize) {
    if note_present { (1, 3, 5, 6) } else { (1, 2, 4, 5) }
}

/// What a key does in a match — the decision table behind the InGame arm,
/// pulled out so the spec-023 bindings are unit-testable without an `App`
/// (the `confirm_choice` pattern). On the player's turn: 1–4 select, ←/→ move,
/// ↑/↓ flip, Enter / P play; every other char (Space, d, s, n, g…) goes to the
/// engine's key map, which validates by phase. Esc / x always leave.
#[derive(Debug, PartialEq, Eq)]
enum TurnKey {
    Menu,
    MoveLeft,
    MoveRight,
    FlipSign,
    Select(usize),
    Play,
    Engine(char),
    Ignore,
}

fn turn_key(key: KeyCode, player_turn: bool) -> TurnKey {
    match key {
        KeyCode::Esc | KeyCode::Char('x') => TurnKey::Menu,
        KeyCode::Left if player_turn => TurnKey::MoveLeft,
        KeyCode::Right if player_turn => TurnKey::MoveRight,
        KeyCode::Up | KeyCode::Down if player_turn => TurnKey::FlipSign,
        KeyCode::Char(c @ '1'..='4') if player_turn => {
            TurnKey::Select(c as usize - '1' as usize)
        }
        KeyCode::Enter | KeyCode::Char('p') if player_turn => TurnKey::Play,
        KeyCode::Char(c) => TurnKey::Engine(c),
        _ => TurnKey::Ignore,
    }
}

/// Where the deck-builder's `Back` returns to — the menu or the campaign map.
/// A pure mapping from the [`BuilderOrigin`] it was opened with (the
/// `confirm_choice` pattern: a routing decision pulled out of the handler so it
/// is unit-testable without an `App`, whose `screen` is private). The
/// `BuildOutcome::Back` arm matches this to `start_menu()` vs. `open_campaign_map()`.
#[derive(Debug, PartialEq, Eq)]
enum BackTo {
    Menu,
    Map,
}

fn back_destination(origin: BuilderOrigin) -> BackTo {
    match origin {
        BuilderOrigin::Menu => BackTo::Menu,
        BuilderOrigin::Map => BackTo::Map,
    }
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
    // The start-menu item to restore the cursor to when a screen backs out to
    // the menu (Erik ruling, spec 020): every Back funnels through start_menu(),
    // which re-selects this. Set to whatever item last opened a screen.
    menu_selection: MenuItem,
    pulse: SelectionPulse,
    settings: Settings,
    // The player's persistent profile — their card collection and built side
    // deck. Matches deal the player's hand from `profile.deck()`.
    profile: Profile,
    audio: Audio,
    // The last in-game audio snapshot; the next one is diffed against it to
    // decide which SFX to play. None outside a game.
    prev_audio: Option<AudioSnapshot>,
    // The opponent's current banter line, shown in the portrait panel. None
    // outside a game, or when there's no appropriate line for the state yet.
    banter: Option<&'static str>,
    // The most recently shown banter line, kept independently of `banter` so
    // the no-back-to-back-repeat rule survives the phase-clear (spec 017 §1):
    // `pick` is fed this, and it is never cleared by the phase-clear.
    banter_last: Option<&'static str>,
    // The last in-game banter snapshot; the next is diffed against it to
    // decide which line class fires. None outside a game.
    prev_banter: Option<BanterSnapshot>,
    // The in-game play log: an ordered record of the current round's moves and
    // the match's resolved round outcomes, built by diffing successive game
    // states. Empty outside a game; reset at match entry, mirroring the banter
    // seeding above.
    play_log: PlayLog,
    // The play-log overlay's scroll offset (top visible body line) and whether
    // it is pinned to follow the latest moves. `follow` opens true so the log
    // shows the newest moves; scrolling up unpins it, scrolling back to the
    // bottom re-pins it (spec 019). `scroll` is persisted (clamped) each draw so
    // a key press always starts from a real offset.
    play_log_scroll: usize,
    play_log_follow: bool,
    // Some((cols, rows)) while the terminal is below the minimum size:
    // the game pauses and a recovery message shows until it grows back.
    too_small: Option<(usize, usize)>,
    // The transient campaign-map banner: how the last staked match settled, or
    // why a launch was refused. Shown until the player navigates the map. UI
    // state only — not saved (the credits it reports are already persisted).
    banner: Option<MapBanner>,
    // Whether a completed campaign run is owed its victory notice (spec 024):
    // set at settlement from `Settlement::completed_run` and taken by the next
    // campaign map entry, which in that window is the game-over acknowledgement.
    // UI state only — never saved, since the completion and its payout are
    // already persisted and only the notice is lost by quitting under it.
    victory_due: bool,
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
            menu_selection: MenuItem::StartCampaign,
            pulse: SelectionPulse::default(),
            audio: Audio::new(settings),
            settings,
            profile,
            prev_audio: None,
            banter: None,
            banter_last: None,
            prev_banter: None,
            play_log: PlayLog::default(),
            play_log_scroll: 0,
            play_log_follow: true,
            too_small: None,
            banner: None,
            victory_due: false,
        }
    }

    /// A fresh start-menu screen reflecting whether a save currently exists.
    fn start_menu(&self) -> Screen {
        let mut menu_state = MenuState::new(self.has_save);
        menu_state.select_item(self.menu_selection);
        Screen::StartMenu { menu_state }
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
            self.open_deck_builder(BuilderOrigin::Menu);
            return;
        }
        self.screen = Screen::OpponentSelect {
            state: OpponentSelectState::new(),
        };
    }

    /// Open the read-only Records overlay (lifetime + run stats) over the menu,
    /// like How to Play / Settings — dismissed back to the menu, whose selection
    /// is preserved because the menu is never left. Needs no deck.
    fn open_records(&mut self) {
        self.modal = Some(Modal::Records(RecordsState::new()));
    }

    /// Open the deck-builder screen from `origin` (the menu's Side Deck item, the
    /// campaign map, or an incomplete-deck divert), which `Back` returns to.
    fn open_deck_builder(&mut self, origin: BuilderOrigin) {
        self.screen = Screen::DeckBuilder {
            state: DeckBuilderState::new(origin),
        };
    }

    /// Open the shop (the campaign-map outfitter): a fresh cursor over the
    /// current depth-gated pool. Back returns to the map.
    fn open_shop(&mut self) {
        self.screen = Screen::Shop {
            state: ShopState::new(),
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

    /// Enter (resume) the campaign: discard a stray in-progress match save first
    /// (with a confirm) if one exists, else open the map. The pre-spec-014
    /// Start Campaign behavior, now shared by the no-progress path and
    /// `CampaignEntry`'s Continue choice.
    fn enter_campaign_continue(&mut self) {
        if self.has_save {
            // The discard prompt lives at campaign entry (human-ruled): entering
            // discards the saved match, so a launch from the map later never has
            // one to overwrite.
            self.modal = Some(Modal::ConfirmNewGame {
                on_yes: false,
                pending: PendingStart::Campaign,
            });
        } else {
            // A pointer with no save behind it (killed mid-match) is a forfeit:
            // drop it — and its escrowed stake — before the broke check, so the
            // confirm notes never name a stake with no match behind it.
            if self.profile.campaign().in_progress().is_some() {
                self.profile.campaign_mut().set_in_progress(None);
                self.profile.save();
            }
            self.enter_campaign_map(true);
        }
    }

    /// Open the campaign map, then raise whatever the entry calls for (specs 021
    /// + 023 + 024): the run-over notice if the balance can no longer cover any
    /// ante, else the victory notice if a run was just completed, else the
    /// first-run primer when the map was reached from the start menu
    /// (`from_menu`) and the primer is still unseen. The one seam all three
    /// checks run at: the three menu-entry paths pass `from_menu: true`, the
    /// game-over acknowledgement passes `false` (a match's game-over is not a
    /// menu entry, spec 023), while Back from the shop or deck builder — which
    /// returns to a map already seen and cannot create a broke state — keeps
    /// using `open_campaign_map`. The victory flag is **taken** here: the notice
    /// is owed exactly once per completion, and the acknowledgement is the only
    /// entry that can happen in the window between settling and showing it.
    fn enter_campaign_map(&mut self, from_menu: bool) {
        self.open_campaign_map();
        let victory_due = std::mem::take(&mut self.victory_due);
        let primer_due = from_menu && !self.profile.primer_seen();
        self.modal = map_entry_modal(self.profile.is_broke(), victory_due, primer_due);
    }

    /// Route a key to the run-over notice: Enter/Space acknowledge, wiping to a
    /// fresh starter run and returning to the start menu (chore 2026-09-13,
    /// supersedes spec 021's "a fresh map opens" — a lost run should hand the
    /// player back to the menu, not drop them straight into another campaign);
    /// everything else (Esc included) is ignored — the notice is the only way
    /// out of a broke run (spec 021). Leaving for the menu plays `MenuBack`,
    /// matching `MapOutcome::Back`.
    fn handle_run_over_input(&mut self, key: KeyCode) {
        if run_over_acknowledged(key) {
            self.modal = None;
            self.reset_run();
            self.audio.play(Sfx::MenuBack);
            self.screen = self.start_menu();
        }
    }

    /// Route a key to the first-run primer or the first-match popup (spec 023):
    /// Enter/Space/Esc dismiss — marking the matching profile flag and saving,
    /// so each piece shows exactly once — and every other key is swallowed, so
    /// nothing underneath can be acted on. Dismissing the popup re-arms any
    /// opponent thinking pause that elapsed while the match was held, so the
    /// usual pause runs from the dismissal rather than expiring instantly.
    /// Quitting with either up leaves its mark unset.
    fn handle_onboarding_input(&mut self, key: KeyCode) {
        if !notice_dismissed(key) {
            return;
        }
        match self.modal {
            Some(Modal::Primer) => self.profile.mark_primer_seen(),
            Some(Modal::FirstMatch) => {
                self.profile.mark_first_match_seen();
                if let Screen::InGame { game_state, .. } = &mut self.screen {
                    game_state.restart_opponent_pause();
                }
            }
            _ => return,
        }
        self.profile.save();
        self.modal = None;
        self.audio.play(Sfx::MenuSelect);
    }

    /// Wipe to a fresh starter profile — the full reset, shared by Reset
    /// Everything (spec 014's New Campaign, rescoped by spec 024) and the
    /// run-over acknowledgement (spec 021), which differ only in where they land
    /// afterwards. The reset core is `Profile::reset_to_starter`; the app-side
    /// tail it shares with the map-only reset is `discard_match_and_banner`.
    /// Callers play their own sfx and set the screen.
    fn reset_run(&mut self) {
        self.profile.reset_to_starter();
        self.profile.save();
        self.discard_match_and_banner();
    }

    /// Reset the campaign map only — spec 024's New Campaign: the beaten set,
    /// the in-flight pointer (and its escrowed stake) and the run tally go with
    /// `Profile::reset_campaign_run`, while credits, collection, deck, lifetime
    /// records and the onboarding marks stay. Same app-side tail as the full
    /// reset. Callers play their own sfx and set the screen.
    fn reset_map_only(&mut self) {
        self.profile.reset_campaign_run();
        self.profile.save();
        self.discard_match_and_banner();
    }

    /// The tail both resets share (spec 024): discard the saved match — the run
    /// it belonged to is gone — and the now-stale map banner.
    fn discard_match_and_banner(&mut self) {
        crate::save::clear();
        self.has_save = false;
        self.banner = None;
    }

    /// Reset per `scope` and open a fresh campaign map — the New Campaign and
    /// Reset Everything actions (spec 024), reachable only from the start menu's
    /// `ConfirmReset`. A menu entry, so it goes through
    /// `enter_campaign_map(true)`: the broke check runs there (a map-only reset
    /// keeps the purse, which may no longer cover the fresh map's cheapest ante),
    /// and the primer shows here if it has not been seen yet.
    fn start_fresh_campaign(&mut self, scope: ResetScope) {
        match scope {
            ResetScope::MapOnly => self.reset_map_only(),
            ResetScope::Everything => self.reset_run(),
        }
        self.audio.play(Sfx::MenuSelect);
        self.enter_campaign_map(true);
    }

    /// The pre-match gate for a campaign node (both ids): uphold `start_match`'s
    /// deck-valid precondition (diverting to the deck-builder if the deck is
    /// incomplete), refuse the launch outright when the balance can't cover the
    /// node's ante floor (spec 021), and otherwise open the wager prompt — the
    /// match itself starts when the player commits a stake there.
    fn launch_campaign_node(&mut self, planet: &str, opponent: &str) {
        if !self.profile.deck_is_valid() {
            // Origin is the map: fixing an incomplete deck mid-campaign returns
            // to the campaign map, not the menu (spec 015 return-path fix).
            self.open_deck_builder(BuilderOrigin::Map);
        } else if let Some(opp) = opponent_by_id(opponent)
            && let Some(planet) = planet_by_id(planet)
        {
            let floor = economy::ante_floor(opp.stand_threshold);
            if self.profile.credits() < floor {
                // Unaffordable: no prompt opens and nothing is staked — the map
                // says why (spec 021, AC2).
                self.banner = Some(MapBanner::CantCover { floor });
                self.audio.play(Sfx::MenuBack);
            } else {
                // The prompt gates on the full balance, so an all-in stake is
                // always reachable and `stake_match` can never refuse.
                self.modal = Some(Modal::Wager(WagerState::new(
                    planet,
                    opp,
                    self.profile.credits(),
                    economy::cheapest_floor(self.profile.campaign()),
                )));
            }
        }
    }

    /// Route a key to the open wager prompt: ←/→ walk the stake, Esc backs out
    /// to the map with nothing staked, Enter commits — closing the prompt and
    /// starting the match against the chosen node with the stake escrowed
    /// (spec 021).
    fn handle_wager_input(&mut self, key: KeyCode) {
        let Some(Modal::Wager(state)) = self.modal.as_mut() else {
            return;
        };
        match state.handle_input(key) {
            Some(WagerOutcome::Moved) => self.audio.play(Sfx::MenuMove),
            Some(WagerOutcome::Cancel) => {
                self.modal = None;
                self.audio.play(Sfx::MenuBack);
            }
            Some(WagerOutcome::Commit) => {
                let node = NodeRef {
                    planet: state.planet_id().to_string(),
                    opponent: state.opponent().id.to_string(),
                    stake: state.stake(),
                };
                let opponent = state.opponent();
                self.modal = None;
                self.audio.play(Sfx::MenuSelect);
                self.start_match(opponent, Some(node));
            }
            None => {}
        }
    }

    /// Begin a fresh match against `opponent`, dealing the player's hand from
    /// the profile's built deck — every match, campaign or Quick Play (spec
    /// 024) — replacing any current match and persisting it.
    ///
    /// Precondition: the profile deck is valid (exactly `SIDE_DECK_SIZE`
    /// cards). Since spec 024 an undersized deck would short-deal **any**
    /// match, Quick Play included, and this is still the only match-start
    /// entry: both of its callers uphold the guard — the opponent-select Pick
    /// (reached via `open_opponent_select`, whose existing divert sends an
    /// invalid deck to the builder first) and the campaign map's Launch. Any
    /// new caller must uphold it too. `campaign` marks the match as a campaign
    /// node (persisted via the profile), or `None` for Quick Play.
    fn start_match(&mut self, opponent: OpponentProfile, campaign: Option<NodeRef>) {
        // Record whether this match belongs to the campaign, and against which
        // node — persisted, so a match resumed via Continue still routes back
        // to the map at game over. Quick Play passes None (clearing any stale
        // pointer).
        match campaign {
            // Escrow (spec 021): the stake leaves the balance now. The prompt
            // clamps to the balance, so this cannot fail; if it ever did,
            // launch nothing.
            Some(node) => {
                if !self.profile.stake_match(node) {
                    return;
                }
            }
            None => self.profile.campaign_mut().set_in_progress(None),
        }
        self.profile.save();

        // Capture the id (and name) before `opponent` is moved into the game
        // state; the greeting seeds the opening banter line, and the name seeds
        // the play log's opponent label.
        let opp_id = opponent.id;
        let opp_name = opponent.name;
        self.screen = Screen::InGame {
            game_state: Box::new(GameState::with_opponent(
                opponent,
                // One deal for both modes (spec 024): every match, campaign or
                // Quick Play, is dealt from the deck the player built.
                self.profile.deck().to_vec(),
            )),
            cursor: HandCursor::default(),
        };
        // Fresh game — the first snapshot seeds silently, so the empty
        // starting board plays no cues.
        self.prev_audio = None;
        // Seed the banter with the opponent's greeting; the first snapshot
        // seeds the diff silently.
        self.prev_banter = None;
        let line = pick(banter_for(opp_id).match_start, None, &mut rand::rng());
        self.banter = Some(line);
        self.banter_last = Some(line);
        // Fresh match — reset the play log; the first snapshot seeds its diff
        // silently, mirroring the banter/audio seeding above.
        self.play_log.reset(opp_name);
        // The profile's first match *start* (spec 023): raise the controls popup
        // over the dealt board. The save below still happens, so quitting under
        // the popup leaves a resumable match and an unset mark — and Continue,
        // a resume rather than a start, never raises it.
        if !self.profile.first_match_seen() {
            self.modal = Some(Modal::FirstMatch);
        }
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

    /// Update the opponent's banter line for whatever just changed, by diffing
    /// the current state against the previous snapshot — mirroring
    /// `emit_audio_cues`. A no-op outside a game. On a fired event, picks a line
    /// from the opponent's voice, never repeating the currently-shown one.
    fn update_banter(&mut self) {
        let (curr, id) = match &self.screen {
            Screen::InGame { game_state, .. } => {
                (BanterSnapshot::of(game_state), game_state.opponent_profile.id)
            }
            _ => return,
        };
        if let Some(prev) = self.prev_banter {
            if match_restarted(&prev, &curr) {
                // A rematch began in place (game_over true→false): seed a fresh
                // match-start greeting so it fires like a match entered from the
                // menu, not the lingering closing line (spec 017 §8 rematch
                // note). A match start outranks the round-level branches.
                let line = pick(banter_for(id).match_start, self.banter_last, &mut rand::rng());
                self.banter = Some(line);
                self.banter_last = Some(line);
            } else if let Some(ev) = banter_event(&prev, &curr) {
                // A new event: pick a line, avoiding the last one shown, and
                // record it in both fields.
                let line = pick(lines_for(banter_for(id), ev), self.banter_last, &mut rand::rng());
                self.banter = Some(line);
                self.banter_last = Some(line);
            } else if play_resumed(&prev, &curr) {
                // The next round's play has begun and no new line fired: clear
                // the shown line (spec 017 §8), but keep `banter_last` so the
                // no-repeat rule still holds across the blank (§1).
                self.banter = None;
            }
        }
        self.prev_banter = Some(curr);
    }

    /// Update the play log for whatever just changed, by feeding the current
    /// game state to the log's diff — mirroring `update_banter`. A no-op
    /// outside a game. The log observes the state; it never mutates it.
    fn update_play_log(&mut self) {
        let game_state = match &self.screen {
            Screen::InGame { game_state, .. } => game_state,
            _ => return,
        };
        self.play_log.observe(game_state);
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
        } else if matches!(self.modal, Some(Modal::PlayLog)) {
            // The play-log overlay is dismissed with L (the same key that opens
            // it) or Esc. It scrolls the transcript with the arrows (Ctrl+P/N
            // already mirror to Up/Down in resolve_key) and PgUp/PgDn; any
            // scroll unpins "follow", and the draw fn re-pins it when scrolled
            // back to the bottom. The game keeps ticking underneath and all
            // other keys are captured, exactly as the help overlay ignores
            // non-dismiss keys. Closing sounds the back cue. Spec 018/019.
            const PLAY_LOG_PAGE: usize = 10;
            match key {
                KeyCode::Char('L') | KeyCode::Esc => {
                    self.modal = None;
                    self.audio.play(Sfx::MenuBack);
                }
                KeyCode::Up => {
                    self.play_log_follow = false;
                    self.play_log_scroll = self.play_log_scroll.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.play_log_follow = false;
                    self.play_log_scroll = self.play_log_scroll.saturating_add(1);
                }
                KeyCode::PageUp => {
                    self.play_log_follow = false;
                    self.play_log_scroll = self.play_log_scroll.saturating_sub(PLAY_LOG_PAGE);
                }
                KeyCode::PageDown => {
                    self.play_log_follow = false;
                    self.play_log_scroll = self.play_log_scroll.saturating_add(PLAY_LOG_PAGE);
                }
                _ => {}
            }
        } else if matches!(self.modal, Some(Modal::Records(_))) {
            // The read-only Records overlay: ◂/▸ page views, ↑/↓ · PgUp/PgDn scroll,
            // Esc/x dismiss back to the menu (whose selection is preserved because the
            // menu was never left). Its own footer hint documents the keys.
            let outcome = if let Some(Modal::Records(state)) = self.modal.as_mut() {
                state.handle_input(key)
            } else {
                None
            };
            match outcome {
                Some(RecordsOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                Some(RecordsOutcome::Back) => {
                    self.modal = None;
                    self.audio.play(Sfx::MenuBack);
                }
                None => {}
            }
        } else if matches!(self.modal, Some(Modal::RunOver)) {
            // The run-over notice takes all input while open (spec 021): only
            // Enter/Space get past it, and they reset the run.
            self.handle_run_over_input(key);
        } else if matches!(self.modal, Some(Modal::Victory)) {
            // The victory notice takes all input while open (spec 024): only
            // Enter/Space/Esc get past it, and they dismiss it back to a live
            // map. Nothing on the map underneath can be acted on meanwhile.
            if notice_dismissed(key) {
                self.modal = None;
                self.audio.play(Sfx::MenuSelect);
            }
        } else if matches!(self.modal, Some(Modal::Primer | Modal::FirstMatch)) {
            // The first-run pieces take all input while open (spec 023): only
            // Enter/Space/Esc get past them, and they dismiss. Nothing on the
            // map or the board underneath can be acted on meanwhile.
            self.handle_onboarding_input(key);
        } else if matches!(self.modal, Some(Modal::Wager(_))) {
            // The wager prompt takes all input while open (spec 021): ←/→ set
            // the stake, Enter commits and launches, Esc returns to the map.
            self.handle_wager_input(key);
        } else if matches!(self.modal, Some(Modal::CampaignEntry { .. })) {
            self.handle_campaign_entry_input(key);
        } else if matches!(self.modal, Some(Modal::ConfirmReset { .. })) {
            self.handle_confirm_reset_input(key);
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
                    // The select, deck-builder, map, and shop screens carry
                    // their own on-screen hint lines, so ? opens no overlay there.
                    Screen::OpponentSelect { .. }
                    | Screen::DeckBuilder { .. }
                    | Screen::CampaignMap { .. }
                    | Screen::Shop { .. } => None,
                };
            }

            // Capital L opens the in-match play-log overlay (spec 018). Only
            // in-game, and only L — lowercase l stays the sign-minus binding in
            // the game-action path below. Return so it doesn't fall through.
            if key == KeyCode::Char('L') && matches!(&self.screen, Screen::InGame { .. }) {
                self.modal = Some(Modal::PlayLog);
                // Open pinned to the latest moves (spec 019); the draw fn
                // recomputes the real bottom offset while following.
                self.play_log_follow = true;
                self.play_log_scroll = 0;
                return;
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
                self.enter_campaign_map(false);
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

                // Route the game inputs to game_state through the `turn_key`
                // table (spec 023): the cursor keys — 1-4, arrows, Enter/P —
                // act only on the player's turn; every other char goes to the
                // engine's key map, which validates it by phase.
                Screen::InGame { game_state, cursor } => {
                    let player_turn = matches!(game_state.game_phase, GamePhase::PlayerTurn);
                    match turn_key(key, player_turn) {
                        // Esc or X quits the game back to the main menu
                        TurnKey::Menu => {
                            self.screen = self.start_menu();
                        }
                        TurnKey::MoveLeft => cursor.move_left(&game_state.player.hand),
                        TurnKey::MoveRight => cursor.move_right(&game_state.player.hand),
                        TurnKey::FlipSign => cursor.toggle_sign(&game_state.player.hand),
                        TurnKey::Select(i) => cursor.select(i, &game_state.player.hand),
                        TurnKey::Play => {
                            cursor_confirm(game_state, cursor);
                            game_changed = true;
                        }
                        TurnKey::Engine(c) => {
                            if let Some(game_action) = game_state.handle_game_input(c) {
                                game_state.apply_game_action(game_action);
                                cursor.normalize(&game_state.player.hand);
                                game_changed = true;
                            }
                        }
                        TurnKey::Ignore => {}
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
                // scrutinee is fine alongside `&mut self.screen`. `origin` (a
                // Copy) is read up front so `Back` can route on it after
                // `state`'s borrow is done — the menu vs. the campaign map.
                Screen::DeckBuilder { state } => {
                    let origin = state.origin();
                    match state.handle_input(key, &self.profile) {
                        Some(BuildOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                        // A nav key that couldn't move (blocked arrow or Tab)
                        // plays the "declined" cue instead of the move cue.
                        Some(BuildOutcome::Blocked) => self.audio.play(Sfx::MenuBack),
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
                        // Return to wherever the builder was opened from: the
                        // menu, or — correcting a pre-existing bug — the campaign
                        // map when a mid-campaign divert sent us here.
                        Some(BuildOutcome::Back) => {
                            self.audio.play(Sfx::MenuBack);
                            match back_destination(origin) {
                                BackTo::Menu => self.screen = self.start_menu(),
                                BackTo::Map => self.open_campaign_map(),
                            }
                        }
                        None => {}
                    }
                }

                // The campaign map: travel between unlocked planets, launch a
                // match against a planet's next opponent, open the shop or deck
                // builder, or back out. Launching a match records the in-progress
                // node so game-over routes back to the map.
                Screen::CampaignMap { state } => {
                    let outcome = state.handle_input(key, &self.profile);
                    // The post-win reward banner shows on arrival and clears on
                    // the player's first navigation.
                    if outcome.is_some() {
                        self.banner = None;
                    }
                    match outcome {
                        Some(MapOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                        Some(MapOutcome::Launch { planet, opponent }) => {
                            // No save-guard here: entering the campaign already
                            // discarded any saved match (the prompt lives at
                            // entry), so a launch from the map never overwrites one.
                            self.audio.play(Sfx::MenuSelect);
                            self.launch_campaign_node(planet, opponent);
                        }
                        Some(MapOutcome::OpenShop) => {
                            self.audio.play(Sfx::MenuSelect);
                            self.open_shop();
                        }
                        Some(MapOutcome::OpenDeckBuilder) => {
                            self.audio.play(Sfx::MenuSelect);
                            self.open_deck_builder(BuilderOrigin::Map);
                        }
                        Some(MapOutcome::Back) => {
                            self.audio.play(Sfx::MenuBack);
                            self.screen = self.start_menu();
                        }
                        None => {}
                    }
                }
                Screen::Shop { state } => match state.handle_input(key, &self.profile) {
                    Some(ShopOutcome::Moved) => self.audio.play(Sfx::MenuMove),
                    Some(ShopOutcome::Buy(card)) => {
                        // The profile decides affordability; persist only if the
                        // buy took (the deck-edit pattern). A refused buy is a
                        // soft "no", not an error.
                        if self.profile.try_purchase(card, economy::card_price(card)) {
                            self.profile.save();
                            self.audio.play(Sfx::MenuSelect);
                        } else {
                            self.audio.play(Sfx::MenuBack);
                        }
                    }
                    Some(ShopOutcome::Back) => {
                        self.audio.play(Sfx::MenuBack);
                        self.open_campaign_map();
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
        self.update_banter();
        self.update_play_log();
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
                        // Discard the saved match, then enter the map. Discarding
                        // a staked match forfeits the stake: the pointer goes with
                        // the save, so the escrow is never returned (spec 021).
                        // The confirm is already closed above, so a run-over
                        // notice raised here isn't overwritten.
                        self.audio.play(Sfx::MenuSelect);
                        crate::save::clear();
                        self.profile.campaign_mut().set_in_progress(None);
                        self.profile.save();
                        self.has_save = false;
                        self.enter_campaign_map(true);
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

    /// Route a key to the Continue / New Campaign / Reset Everything choice:
    /// ←/→ (a/d) step the highlight, wrapping; Enter/Space commit — Continue
    /// resumes, New Campaign opens the map-only confirm, Reset Everything the
    /// full-wipe one — and Esc closes. Mirrors `handle_confirm_input`.
    /// (spec 024, superseding spec 014's two-choice panel)
    fn handle_campaign_entry_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left | KeyCode::Right | KeyCode::Char('a') | KeyCode::Char('d') => {
                let forward = matches!(key, KeyCode::Right | KeyCode::Char('d'));
                if let Some(Modal::CampaignEntry { choice }) = self.modal.as_mut() {
                    *choice = choice.step(forward);
                }
                self.audio.play(Sfx::MenuMove);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let Some(Modal::CampaignEntry { choice }) = self.modal else {
                    return;
                };
                self.modal = None;
                self.audio.play(Sfx::MenuSelect);
                match choice {
                    CampaignChoice::Continue => self.enter_campaign_continue(),
                    // Both destructive choices go behind the same default-No
                    // confirm, which the scope tells apart.
                    CampaignChoice::NewCampaign => {
                        self.modal = Some(Modal::ConfirmReset {
                            on_yes: false,
                            scope: ResetScope::MapOnly,
                        });
                    }
                    CampaignChoice::ResetEverything => {
                        self.modal = Some(Modal::ConfirmReset {
                            on_yes: false,
                            scope: ResetScope::Everything,
                        });
                    }
                }
            }
            KeyCode::Esc => {
                self.modal = None;
                self.audio.play(Sfx::MenuBack);
            }
            _ => {}
        }
    }

    /// Route a key to a reset confirmation: ←/→ (a/d) toggle, Enter/Space commit
    /// (Yes performs the `scope`'s reset and opens a fresh map; No cancels), Esc
    /// cancels. Mirrors `handle_confirm_input`. (spec 014, rescoped by spec 024)
    fn handle_confirm_reset_input(&mut self, key: KeyCode) {
        let on_yes = matches!(self.modal, Some(Modal::ConfirmReset { on_yes: true, .. }));
        let Some(Modal::ConfirmReset { scope, .. }) = self.modal else {
            return;
        };
        match confirm_choice(on_yes, key) {
            ConfirmChoice::Toggle => {
                if let Some(Modal::ConfirmReset { on_yes, .. }) = self.modal.as_mut() {
                    *on_yes = !*on_yes;
                }
                self.audio.play(Sfx::MenuMove);
            }
            // The only path to either irreversible reset — guarded by
            // `confirm_choice` (Commit iff Enter/Space with Yes), which
            // `confirm_choice_commits_only_on_enter_with_yes` pins.
            ConfirmChoice::Commit => {
                self.modal = None;
                self.start_fresh_campaign(scope);
            }
            ConfirmChoice::Cancel => {
                self.modal = None;
                self.audio.play(Sfx::MenuBack);
            }
            ConfirmChoice::Ignore => {}
        }
    }

    /// Act on an activated start-menu item — open a screen or a modal.
    fn activate_menu_item(&mut self, menu_item: MenuItem) {
        self.menu_selection = menu_item;
        match menu_item {
            MenuItem::Continue => {
                // Resume the saved match. Continue only appears when a save
                // exists, but a race or corruption could still yield None —
                // then it's a no-op, not a crash.
                if let Some(mut game) = crate::save::load() {
                    // A pre-023 save can sit in AwaitingSignChoice (the old `1`
                    // on a ± card saved there). No key answers that phase any
                    // more, so cancel it on load: a no-op everywhere else, and
                    // there it returns the card to hand at PlayerTurn.
                    game.apply_game_action(GameAction::CancelSignChoice);
                    // The cursor isn't saved. Snap it onto a real hand card:
                    // the default index 0 may now be empty (that card was
                    // played), which would show the empty-hand prompt on the
                    // resumed board instead of a selection.
                    let mut cursor = HandCursor::default();
                    cursor.normalize(&game.player.hand);
                    // Read the opponent's name before `game` is moved into the
                    // Box; it seeds the play log's opponent label below.
                    let opp_name = game.opponent_profile.name;
                    self.screen = Screen::InGame {
                        game_state: Box::new(game),
                        cursor,
                    };
                    // Resuming mid-match: seed the audio snapshot silently so
                    // the restored board doesn't replay cues for cards already
                    // on the table.
                    self.prev_audio = None;
                    // Blank the banter on resume — no greeting for a match
                    // already underway; the next event supplies a line.
                    self.prev_banter = None;
                    self.banter = None;
                    self.banter_last = None;
                    // Reset the play log for the resumed match; its first
                    // observe seeds silently, so a resumed match is not
                    // back-logged.
                    self.play_log.reset(opp_name);
                }
            }
            MenuItem::StartCampaign => {
                // Offer the choices whenever there is something they would
                // affect — progress, or a pool that differs from the starter
                // (spec 024); on a truly fresh profile enter directly, where
                // every choice would be a no-op.
                if self.profile.differs_from_starter() {
                    self.modal = Some(Modal::CampaignEntry {
                        choice: CampaignChoice::Continue,
                    });
                } else {
                    self.enter_campaign_continue();
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
                self.open_deck_builder(BuilderOrigin::Menu);
            }
            MenuItem::Records => self.open_records(),
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

        // The first-match popup holds the match (spec 023): skipping the engine
        // update freezes whatever phase it opened on — including an opponent
        // thinking pause whose deadline passes underneath — until it is
        // dismissed, which re-arms that pause. The observers below still run;
        // nothing changed, so none of them fire.
        let held = matches!(self.modal, Some(Modal::FirstMatch));
        let mut phase_changed = false;
        if !held
            && let Screen::InGame { game_state, .. } = &mut self.screen
        {
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

        // Resolve the finished match exactly once, on the tick the phase
        // enters GameOver — the one resolution seam (spec 021 folded the old
        // every-tick campaign-win block into this edge, since a rematch makes
        // `!is_opponent_beaten` useless as a once-guard; `GameOver` is only
        // ever entered from `GameState::update()` here, and a saved match is
        // never at `GameOver`). `Profile::resolve_match` owns the whole
        // resolution: it writes the lifetime and run-tally statistics first
        // and only then settles the stake, because the first-clear record is
        // taken from the run tally on the completion edge and so must already
        // count the completing match (spec 024) — an order the app used to get
        // backwards, and which now lives in one unit-tested method instead of
        // spanning two calls from here. It also picks the mode itself from the
        // in-flight pointer. Quick Play has no pointer, so it settles nothing
        // and records to Quick Play; the pointer is cleared only on the
        // player's acknowledgement (the InGame input arm). Abandoned matches
        // never reach a GameOver tick, so they resolve nothing.
        if phase_changed
            && let Screen::InGame { game_state, .. } = &self.screen
            && matches!(game_state.game_phase, GamePhase::GameOver { .. })
        {
            let player_won =
                matches!(game_state.game_phase, GamePhase::GameOver { winner: Player::Player });
            let opponent_id = game_state.opponent_profile.id;
            let player_rounds = game_state.player.rounds_won as u32;
            let opp_rounds = game_state.opponent.rounds_won as u32;
            if let Some(settlement) =
                self.profile.resolve_match(opponent_id, player_won, player_rounds, opp_rounds)
            {
                self.banner = Some(MapBanner::Settled(settlement.outcome));
                // The completion edge is a one-shot signal, not a saved flag
                // (spec 024): it waits here until the acknowledgement opens the
                // map, which is what raises the victory notice.
                self.victory_due |= settlement.completed_run;
            }
            self.profile.save();
        }

        // Sound the opponent's moves and round/game resolutions, which
        // happen here in the update rather than from a player keypress.
        self.emit_audio_cues();
        self.update_banter();
        self.update_play_log();
    }

    /// The stake the board shows (spec 026, Q6 A): the escrowed stake while the
    /// match runs, or — at `GameOver`, when `tick` has already settled it on
    /// this same iteration and `stake_at_risk()` is `None` — the settled amount
    /// the map banner holds. The campaign pointer is still set at `GameOver`
    /// (it clears on the acknowledgement), so a Quick Play game over never
    /// picks up a banner left from an earlier campaign settlement.
    fn stake_to_show(&self) -> Option<u32> {
        self.profile.campaign().stake_at_risk().or_else(|| {
            let Screen::InGame { game_state, .. } = &self.screen else {
                return None;
            };
            if !matches!(game_state.game_phase, GamePhase::GameOver { .. })
                || self.profile.campaign().in_progress().is_none()
            {
                return None;
            }
            match &self.banner {
                Some(MapBanner::Settled(StakeOutcome::Won(n) | StakeOutcome::Lost(n))) => Some(*n),
                _ => None,
            }
        })
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
                self.board_view.draw(
                    game_state,
                    cursor,
                    self.banter,
                    self.stake_to_show(),
                    pulse,
                    frame,
                )
            }
            Screen::OpponentSelect { state } => state.draw(frame, &self.config, pulse),
            Screen::DeckBuilder { state } => state.draw(frame, &self.config, &self.profile, pulse),
            Screen::CampaignMap { state } => {
                state.draw(frame, &self.config, &self.profile, self.banner.as_ref(), pulse)
            }
            Screen::Shop { state } => state.draw(frame, &self.config, &self.profile, pulse),
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
            Some(Modal::Wager(state)) => state.draw(frame, &self.config, pulse),
            Some(Modal::RunOver) => {
                let lines =
                    run_over_notice_lines(self.run_tally(), self.worlds_cleared(), PLANETS.len());
                self.draw_notice(frame, &lines)
            }
            Some(Modal::Victory) => {
                let lines =
                    victory_notice_lines(self.run_tally(), self.worlds_cleared(), PLANETS.len());
                self.draw_notice(frame, &lines)
            }
            // The first-run primer and the first-match popup (spec 023): the same
            // bordered text box the other overlays use, rebuilt from its asset
            // every frame — so, like the play log, they need no resize arm.
            Some(Modal::Primer) => {
                draw_text_overlay(self.config, &overlay_text(OverlayKind::Primer), frame)
            }
            Some(Modal::FirstMatch) => {
                draw_text_overlay(self.config, &overlay_text(OverlayKind::FirstMatch), frame)
            }
            Some(Modal::CampaignEntry { choice }) => {
                self.draw_campaign_entry(*choice, pulse, frame)
            }
            Some(Modal::ConfirmReset { on_yes, scope }) => {
                self.draw_confirm_reset(*on_yes, *scope, pulse, frame)
            }
            // The play log draws after this match (it writes back scroll state,
            // which would conflict with the shared borrow the match holds).
            Some(Modal::PlayLog) => {}
            // Records likewise draws after this match — its draw writes the
            // clamped scroll back, needing a mutable borrow of self.modal.
            Some(Modal::Records(_)) => {}
            None => {}
        }

        // The play-log overlay (spec 019): a larger, padded, scrollable window
        // showing the full match transcript. Only meaningful in-match; the L
        // toggle can only open it there, but guard anyway. No resize arm is
        // needed (unlike Modal::Help, which caches an Overlay): the content is
        // rebuilt every draw from self.play_log and self.config, and resize()
        // already keeps self.config current. We pass usize::MAX while following
        // so the draw fn pins to the bottom, then persist the clamped scroll and
        // re-pin follow from whether it landed at the bottom.
        if matches!(self.modal, Some(Modal::PlayLog))
            && matches!(self.screen, Screen::InGame { .. })
        {
            let body = self.play_log.render_body();
            let scroll = if self.play_log_follow {
                usize::MAX
            } else {
                self.play_log_scroll
            };
            let result = draw_scrollable_overlay(self.config, "Play Log", &body, scroll, frame);
            self.play_log_scroll = result.scroll;
            self.play_log_follow = result.at_bottom;
        }

        // The Records overlay draws here rather than in the immutable match above:
        // its draw writes the clamped scroll back into the state, which needs a
        // mutable borrow of self.modal (mirrors the play-log block).
        if let Some(Modal::Records(state)) = &mut self.modal {
            state.draw(frame, &self.config, &self.profile, pulse);
        }
    }

    /// Draw a choice panel as a bordered overlay over the menu, matching How to
    /// Play / Settings: a title, an optional Muted note under it, the `labels`
    /// side by side on one row (the selected one marked with ▸ and breathing
    /// with the pulse; the others keep two leading spaces so the marker never
    /// shifts the text), and a hint. Shared by the discard-a-save confirm, the
    /// campaign-entry panel and the reset confirms — two labels or three (spec
    /// 024). `note` warns that confirming also forfeits an escrowed stake (spec
    /// 021); the rows come from `choice_rows`, which keeps a blank row above and
    /// below the acted-on choice row whether or not the note is showing.
    fn draw_choice_panel(
        &self,
        frame: &mut Frame,
        title: &str,
        note: Option<&str>,
        labels: &[&str],
        selected: usize,
        hint: &str,
        pulse: Emphasis,
    ) {
        let block_w = choice_row_width(labels);
        let (note_row, choice_row, hint_row, height) = choice_rows(note.is_some());

        // The box widens to fit the widest of title, note, hint, and choice row.
        let content_width = choice_panel_width(title, note, hint, labels);
        let layout = OverlayLayout::new(self.config, content_width, height);

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
        draw_text_in(frame, layout.inner, 0, Align::Center, title, Emphasis::Normal);
        if let Some(note) = note {
            draw_text_in(frame, layout.inner, note_row, Align::Center, note, Emphasis::Muted);
        }

        let inner = layout.inner;
        let row_y = inner.y0 + choice_row;
        let mut x = inner.x0 + inner.width().saturating_sub(block_w) / 2;
        for (i, label) in labels.iter().enumerate() {
            let text = format!("{} {}", if i == selected { "▸" } else { " " }, label);
            let emphasis = if i == selected { pulse } else { Emphasis::Normal };
            draw_text(frame, x, row_y, &text, emphasis);
            x += text.chars().count() + CHOICE_GAP;
        }

        draw_text_in(frame, inner, hint_row, Align::Center, hint, Emphasis::Muted);
    }

    /// The discard-a-save confirmation (Yes left / No right, default No). When
    /// the saved match carries a stake, a note says confirming forfeits it.
    fn draw_confirm_new_game(&self, on_yes: bool, pulse: Emphasis, frame: &mut Frame) {
        let note = self.stake_forfeit_note();
        self.draw_choice_panel(
            frame,
            "Discard your saved match?",
            note.as_deref(),
            &["Yes", "No"],
            if on_yes { 0 } else { 1 },
            "←/→ choose  ·  Enter confirm  ·  Esc cancel",
            pulse,
        );
    }

    /// The Continue / New Campaign / Reset Everything choice at campaign entry
    /// (default Continue, spec 024).
    fn draw_campaign_entry(&self, choice: CampaignChoice, pulse: Emphasis, frame: &mut Frame) {
        let labels: Vec<&str> = CampaignChoice::ALL.iter().map(|c| c.label()).collect();
        let selected = CampaignChoice::ALL.iter().position(|c| *c == choice).unwrap_or(0);
        self.draw_choice_panel(
            frame,
            "Campaign",
            None,
            &labels,
            selected,
            "←/→ choose  ·  Enter select  ·  Esc back",
            pulse,
        );
    }

    /// The destructive reset confirmation (Yes left / No right, default No),
    /// titled by what its `scope` takes (spec 024) and noting an escrowed stake
    /// the reset would forfeit.
    fn draw_confirm_reset(
        &self,
        on_yes: bool,
        scope: ResetScope,
        pulse: Emphasis,
        frame: &mut Frame,
    ) {
        let note = self.stake_forfeit_note();
        let title = match scope {
            ResetScope::MapOnly => "New campaign? Resets the map; you keep your cards and credits.",
            ResetScope::Everything => "Reset everything? Erases progress, credits & cards.",
        };
        self.draw_choice_panel(
            frame,
            title,
            note.as_deref(),
            &["Yes", "No"],
            if on_yes { 0 } else { 1 },
            "←/→ choose  ·  Enter confirm  ·  Esc cancel",
            pulse,
        );
    }

    /// The second line the discard confirmations carry when a match is in flight
    /// with a stake escrowed — confirming drops the pointer, and the stake with
    /// it (spec 021). `None` when nothing is at risk.
    fn stake_forfeit_note(&self) -> Option<String> {
        self.profile
            .campaign()
            .stake_at_risk()
            .map(|s| format!("…and forfeit your {s}-credit stake."))
    }

    /// The run tally both notices report (spec 024).
    fn run_tally(&self) -> &RunStats {
        self.profile.campaign().run_stats()
    }

    /// How many worlds this run has cleared — the summary's last figure.
    fn worlds_cleared(&self) -> usize {
        self.profile.campaign().worlds_cleared()
    }

    /// Draw a one-way notice — the run-over notice (spec 021) and the victory
    /// notice (spec 024) — as a bordered box over whatever is underneath.
    /// Borrows `draw_choice_panel`'s shape without its choices: the title row is
    /// Strong, the dismiss line (always the last row) is Muted, everything
    /// between is Normal, and every row is centred. The blank rows are part of
    /// the content the caller built, so the breathing room is testable without a
    /// terminal; the box pads itself evenly above and below (`OverlayLayout`).
    fn draw_notice(&self, frame: &mut Frame, lines: &[String]) {
        let content_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let layout = OverlayLayout::new(self.config, content_width, lines.len());

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
        let last = lines.len().saturating_sub(1);
        for (row, line) in lines.iter().enumerate() {
            let emphasis = match row {
                0 => Emphasis::Strong,
                r if r == last => Emphasis::Muted,
                _ => Emphasis::Normal,
            };
            draw_text_in(frame, layout.inner, row, Align::Center, line, emphasis);
        }
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
    fn confirm_choice_commits_only_on_enter_with_yes() {
        use ConfirmChoice::*;
        // The irreversible-reset guard (spec 014, now serving both scopes of
        // spec 024's confirm — New Campaign's map reset and Reset Everything's
        // wipe): a reset commits iff Enter/Space is pressed with Yes
        // highlighted — never on No, never on Esc, never on a toggle. A
        // refactor that flipped a condition (reset on No/Esc, or a default-Yes
        // confirm) would be caught here rather than in play.
        assert_eq!(confirm_choice(true, KeyCode::Enter), Commit);
        assert_eq!(confirm_choice(true, KeyCode::Char(' ')), Commit);
        assert_eq!(confirm_choice(false, KeyCode::Enter), Cancel);
        assert_eq!(confirm_choice(false, KeyCode::Char(' ')), Cancel);
        assert_eq!(confirm_choice(true, KeyCode::Esc), Cancel);
        assert_eq!(confirm_choice(false, KeyCode::Esc), Cancel);
        for k in [KeyCode::Left, KeyCode::Right, KeyCode::Char('a'), KeyCode::Char('d')] {
            assert_eq!(confirm_choice(true, k), Toggle);
            assert_eq!(confirm_choice(false, k), Toggle);
        }
        assert_eq!(confirm_choice(true, KeyCode::Char('z')), Ignore);
    }

    #[test]
    fn campaign_choice_steps_and_wraps_in_both_directions() {
        use CampaignChoice::*;
        // Spec 024: the entry panel's highlight moves with ←/→ and wraps at both
        // ends — the two-choice toggle generalized to three. Continue is first,
        // so the panel opens on the safe option, and the destructive choices are
        // reached deliberately rather than by drifting off the end of the row.
        assert_eq!(CampaignChoice::ALL, [Continue, NewCampaign, ResetEverything]);
        assert_eq!(Continue.step(true), NewCampaign);
        assert_eq!(NewCampaign.step(true), ResetEverything);
        assert_eq!(ResetEverything.step(true), Continue);
        assert_eq!(ResetEverything.step(false), NewCampaign);
        assert_eq!(NewCampaign.step(false), Continue);
        assert_eq!(Continue.step(false), ResetEverything);
    }

    #[test]
    fn a_choice_panel_keeps_a_blank_row_around_the_choice_row() {
        // Design brief §Density and breathing room (spec 024 tension §4): the
        // choice row is the acted-on element, so nothing is drawn on the row
        // above or below it. That makes a panel carrying a stake note one row
        // taller rather than letting the note sit flush against the choices —
        // which also corrects the spec-021 discard confirm, the other panel that
        // passes a note.
        assert_eq!(choice_rows(false), (1, 2, 4, 5));
        assert_eq!(choice_rows(true), (1, 3, 5, 6));

        for note_present in [false, true] {
            let (note_row, choice_row, hint_row, height) = choice_rows(note_present);
            let mut drawn = vec![0, choice_row, hint_row];
            if note_present {
                drawn.push(note_row);
            }
            assert!(
                !drawn.contains(&(choice_row - 1)),
                "note {note_present}: a row is drawn directly above the choices"
            );
            assert!(
                !drawn.contains(&(choice_row + 1)),
                "note {note_present}: a row is drawn directly below the choices"
            );
            assert!(hint_row < height, "the hint row is outside the content");
        }
    }

    #[test]
    fn the_campaign_entry_panel_fits_the_minimum_terminal() {
        // Spec 024: three labels on one row at campaign entry, and the widest
        // reset confirm (title plus a stake note, so the taller layout) — both
        // measured through the same layout the draw uses, at 139x31, so neither
        // box is ever clamped over the menu.
        let labels: Vec<&str> = CampaignChoice::ALL.iter().map(|c| c.label()).collect();
        assert_eq!(labels, ["Continue", "New Campaign", "Reset Everything"]);

        let (cols, rows) = Config::min_size();
        let config = Config { num_cols: cols, num_rows: rows };
        let yes_no = ["Yes", "No"];

        // Each panel's own strings, verbatim from its draw fn: the entry panel
        // has no note and its own "select / back" hint, the confirm has a stake
        // note and the "confirm / cancel" hint.
        for (title, note, row_labels, hint) in [
            (
                "Campaign",
                None,
                labels.as_slice(),
                "←/→ choose  ·  Enter select  ·  Esc back",
            ),
            (
                "New campaign? Resets the map; you keep your cards and credits.",
                Some("…and forfeit your 999-credit stake."),
                yes_no.as_slice(),
                "←/→ choose  ·  Enter confirm  ·  Esc cancel",
            ),
        ] {
            let width = choice_panel_width(title, note, hint, row_labels);
            let (_, _, _, height) = choice_rows(note.is_some());
            let layout = OverlayLayout::new(config, width, height);
            assert_eq!(layout.outer.width(), width + 2 * crate::H_PAD, "box width clamped");
            assert_eq!(layout.outer.height(), height + crate::V_PAD, "box height clamped");
            assert!(layout.outer.y1 < rows && layout.outer.x1 < cols, "box off-frame");
        }
    }

    #[test]
    fn run_over_acknowledged_only_on_enter_or_space() {
        // The run-over notice is one-way (spec 021): Enter/Space accept the reset
        // and the return to the start menu (chore 2026-09-13), and nothing else
        // gets past it — Esc especially, since dismissing it would leave the
        // player on a map with no affordable node.
        assert!(run_over_acknowledged(KeyCode::Enter));
        assert!(run_over_acknowledged(KeyCode::Char(' ')));
        assert!(!run_over_acknowledged(KeyCode::Esc));
        assert!(!run_over_acknowledged(KeyCode::Char('x')));
        for k in [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Char('g')] {
            assert!(!run_over_acknowledged(k));
        }
    }

    #[test]
    fn back_destination_routes_each_origin_to_its_screen() {
        // The deck-builder's `Back` returns to where it was opened from (spec
        // 015). This pure seam is the whole routing decision — the arm's match
        // on it is a fixed dispatch — so flipping either arm here (Menu → Map or
        // Map → Menu) is caught, including the campaign-divert-returns-to-map fix.
        assert_eq!(back_destination(BuilderOrigin::Menu), BackTo::Menu);
        assert_eq!(back_destination(BuilderOrigin::Map), BackTo::Map);
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
    fn turn_key_binds_the_spec_023_keys() {
        use TurnKey::*;

        // On the player's turn: 1-4 select that slot...
        assert_eq!(turn_key(KeyCode::Char('1'), true), Select(0));
        assert_eq!(turn_key(KeyCode::Char('4'), true), Select(3));
        // ...Enter and P play the selection...
        assert_eq!(turn_key(KeyCode::Enter, true), Play);
        assert_eq!(turn_key(KeyCode::Char('p'), true), Play);
        // ...the arrows move and flip...
        assert_eq!(turn_key(KeyCode::Left, true), MoveLeft);
        assert_eq!(turn_key(KeyCode::Right, true), MoveRight);
        assert_eq!(turn_key(KeyCode::Up, true), FlipSign);
        assert_eq!(turn_key(KeyCode::Down, true), FlipSign);
        // ...and every other char is the engine's (Space draws there).
        assert_eq!(turn_key(KeyCode::Char(' '), true), Engine(' '));
        assert_eq!(turn_key(KeyCode::Char('d'), true), Engine('d'));
        assert_eq!(turn_key(KeyCode::Char('s'), true), Engine('s'));

        // Off the player's turn the cursor keys do nothing, but Space — and
        // the number keys, which the engine maps to nothing — still pass.
        assert_eq!(turn_key(KeyCode::Enter, false), Ignore);
        assert_eq!(turn_key(KeyCode::Left, false), Ignore);
        assert_eq!(turn_key(KeyCode::Up, false), Ignore);
        assert_eq!(turn_key(KeyCode::Char(' '), false), Engine(' '));
        assert_eq!(turn_key(KeyCode::Char('1'), false), Engine('1'));

        // Esc / x leave the match either way.
        for on_turn in [true, false] {
            assert_eq!(turn_key(KeyCode::Esc, on_turn), Menu);
            assert_eq!(turn_key(KeyCode::Char('x'), on_turn), Menu);
        }
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

    #[test]
    fn cursor_select_lands_on_an_occupied_slot_and_resets_the_sign_like_a_move() {
        let h = hand(&[Some(Card::PlusMinus(3)), Some(Card::Minus(4)), None, None]);

        // Selecting slot 2 from a flipped slot 1 leaves exactly the state an
        // arrow landing there would.
        let mut c = HandCursor::default();
        c.toggle_sign(&h); // negative on the ±3
        c.select(1, &h);
        assert_eq!(c.index(), 1);
        assert!(c.pending_positive());

        let mut moved = HandCursor::default();
        moved.toggle_sign(&h);
        moved.move_right(&h);
        assert_eq!((c.index(), c.pending_positive()), (moved.index(), moved.pending_positive()));

        // Selecting the already-selected slot resets the sign too, as an arrow
        // landing on the same slot does.
        let mut c = HandCursor::default();
        c.toggle_sign(&h);
        assert!(!c.pending_positive());
        c.select(0, &h);
        assert_eq!(c.index(), 0);
        assert!(c.pending_positive());
    }

    #[test]
    fn cursor_select_ignores_an_empty_or_out_of_range_slot() {
        let h = hand(&[Some(Card::PlusMinus(3)), Some(Card::Minus(4)), None, None]);
        let mut c = HandCursor::default();
        c.toggle_sign(&h); // negative on the ±3

        c.select(2, &h); // empty slot
        assert_eq!(c.index(), 0);
        assert!(!c.pending_positive()); // sign untouched as well

        c.select(8, &h); // out of range
        assert_eq!(c.index(), 0);
        assert!(!c.pending_positive());
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

    #[test]
    fn notice_dismissed_on_enter_space_or_esc_only() {
        // The first-run pieces (spec 023) and the victory notice (spec 024) are
        // notices, not choices: Enter, Space and Esc all wave them away, and
        // nothing else does — so a stray key over the map or the board can never
        // mark a first-run piece seen, or wave off the victory notice, by
        // accident.
        assert!(notice_dismissed(KeyCode::Enter));
        assert!(notice_dismissed(KeyCode::Char(' ')));
        assert!(notice_dismissed(KeyCode::Esc));
        for k in [
            KeyCode::Char('x'),
            KeyCode::Char('d'),
            KeyCode::Char('1'),
            KeyCode::Char('?'),
            KeyCode::Up,
            KeyCode::Left,
        ] {
            assert!(!notice_dismissed(k), "{k:?} must not dismiss");
        }
    }

    #[test]
    fn map_entry_modal_prefers_run_over_then_victory_then_primer() {
        // One modal at a time (specs 023 + 024): broke wins over everything, a
        // completed run wins over the primer, and the primer shows only while it
        // is due. The first two cases never actually arise together in play — a
        // completing win can't leave the player broke, and the primer is
        // menu-entry only — so the order is pinned here rather than left to
        // drift.
        assert!(matches!(map_entry_modal(true, true, true), Some(Modal::RunOver)));
        assert!(matches!(map_entry_modal(true, false, false), Some(Modal::RunOver)));
        assert!(matches!(map_entry_modal(false, true, true), Some(Modal::Victory)));
        assert!(matches!(map_entry_modal(false, false, true), Some(Modal::Primer)));
        assert!(map_entry_modal(false, false, false).is_none());
    }

    #[test]
    fn both_notices_read_right_breathe_and_fit_the_minimum_terminal() {
        // Spec 024: the two notices share one summary block, both end on the
        // dismiss line with an empty row above it (the acted-on element gets its
        // air — the box's own padding gives the row below), neither doubles a
        // blank row, and both fit 139x31 unclamped so no row is ever eaten.
        let mut run = RunStats::default();
        for _ in 0..4 {
            run.record_match(true, 3, 1);
        }
        for _ in 0..2 {
            run.record_match(false, 1, 3);
        }
        run.record_credits_won(640);
        run.record_credits_lost(210);

        let summary = run_summary_lines(&run, 7, 8);
        let victory = victory_notice_lines(&run, 7, 8);
        let run_over = run_over_notice_lines(&run, 7, 8);

        assert_eq!(victory[0], "Campaign complete — the house's best has lost.");
        assert_eq!(run_over[0], "You're broke — the run is over.");

        let (cols, rows) = Config::min_size();
        let config = Config { num_cols: cols, num_rows: rows };

        for lines in [&victory, &run_over] {
            let summary_at = lines
                .iter()
                .position(|l| l == &summary[0])
                .expect("the summary block is on the notice");
            assert_eq!(
                &lines[summary_at..summary_at + 3],
                &summary[..],
                "the three summary lines run in order"
            );
            assert_eq!(lines.last().unwrap(), "Enter  continue", "the dismiss line is last");
            assert_eq!(
                lines[lines.len() - 2],
                "",
                "the dismiss line needs an empty row above it"
            );
            assert!(
                lines.windows(2).all(|w| !(w[0].is_empty() && w[1].is_empty())),
                "no slab of empty rows: {lines:?}"
            );

            let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            let layout = OverlayLayout::new(config, width, lines.len());
            assert_eq!(
                layout.outer.height(),
                lines.len() + crate::V_PAD,
                "box height clamped — the notice outgrew the minimum terminal"
            );
            assert_eq!(layout.outer.width(), width + 2 * crate::H_PAD, "box width clamped");
            assert!(layout.outer.y1 < rows && layout.outer.x1 < cols, "box off-frame");
        }

        // The victory notice's two spec'd lines sit above the summary...
        let victory_at = victory.iter().position(|l| l == &summary[0]).unwrap();
        assert_eq!(victory[2], "Your cards and credits are yours to keep.");
        assert_eq!(
            victory[3],
            "Rematches stay open; New Campaign replays the map with your deck."
        );
        assert!(victory_at > 3, "the summary follows the victory text");

        // ...and the run-over notice keeps its reset note *after* the summary.
        let run_over_at = run_over.iter().position(|l| l == &summary[0]).unwrap();
        let reset_note = "Deck, collection, and progress reset to the starter; your records stay.";
        let note_at = run_over
            .iter()
            .position(|l| l == reset_note)
            .expect("the run-over notice keeps its reset note");
        assert!(note_at > run_over_at + 2, "the reset note follows the summary");
    }

    #[test]
    fn the_first_match_popup_holds_the_match_and_swallows_play_keys() {
        // Spec 023: while the popup is up the match does not advance — an
        // opponent thinking pause whose deadline has already passed still does
        // not fire on a tick — and no play key reaches the game. Only
        // non-dismiss keys are pressed and no phase changes, so nothing here
        // saves the game or the profile to disk.
        use std::time::Instant;

        let mut app = App::new(Config { num_cols: 120, num_rows: 40 });
        let mut game_state = GameState::new();
        game_state.game_phase = GamePhase::OpponentThinking { until: Instant::now() };
        app.screen = Screen::InGame {
            game_state: Box::new(game_state),
            cursor: HandCursor::default(),
        };
        app.modal = Some(Modal::FirstMatch);

        app.tick(Duration::from_millis(500));
        let Screen::InGame { game_state, .. } = &mut app.screen else {
            panic!("still in the match");
        };
        assert!(
            matches!(game_state.game_phase, GamePhase::OpponentThinking { .. }),
            "an elapsed thinking pause must not resolve under the popup"
        );

        // Back on the player's turn: the play keys are swallowed too.
        game_state.game_phase = GamePhase::PlayerTurn;
        let hand_before = game_state.player.hand.clone();
        let dealer_before = game_state.player.dealer_row.len();
        let played_before = game_state.player.played_row.len();

        for k in [KeyCode::Char('d'), KeyCode::Char('1'), KeyCode::Char('s'), KeyCode::Char('p')] {
            app.handle_key(k);
        }

        assert!(matches!(app.modal, Some(Modal::FirstMatch)), "the popup stays up");
        let Screen::InGame { game_state, .. } = &app.screen else {
            panic!("still in the match");
        };
        assert!(matches!(game_state.game_phase, GamePhase::PlayerTurn), "phase unchanged");
        assert_eq!(game_state.player.hand, hand_before, "hand unchanged");
        assert_eq!(game_state.player.dealer_row.len(), dealer_before, "no card drawn");
        assert_eq!(game_state.player.played_row.len(), played_before, "no card played");
    }

    #[test]
    fn the_game_over_frame_shows_the_settled_stake_on_both_layouts() {
        // Spec 026 T002a (Q6 A): the match settles on the tick that draws the
        // game-over frame, so `stake_at_risk()` is already `None` there — the
        // board is handed the settled amount from the map banner instead, on
        // the compact band (89) and the wide panel (139). The App holds a
        // fresh profile with the campaign pointer still set, as it is at
        // `GameOver` before the acknowledgement; nothing here writes to disk.
        use crate::layout::BoardLayout;
        use crate::portrait::stake_line;

        fn row_text(frame: &Frame, y: usize) -> String {
            frame.iter().map(|col| col[y].ch).collect()
        }

        for (cols, wide) in [(89, false), (139, true)] {
            let config = Config { num_cols: cols, num_rows: 31 };
            let mut app = App::new(config);
            app.profile = Profile::default();
            app.profile.campaign_mut().set_in_progress(Some(NodeRef {
                planet: "cinder".to_string(),
                opponent: "greeb".to_string(),
                stake: 0, // settled: the escrow is already taken
            }));
            app.banner = Some(MapBanner::Settled(StakeOutcome::Won(30)));
            let mut game_state = GameState::new();
            game_state.game_phase = GamePhase::GameOver { winner: Player::Player };
            app.screen = Screen::InGame {
                game_state: Box::new(game_state),
                cursor: HandCursor::default(),
            };
            assert_eq!(app.stake_to_show(), Some(30));

            let mut frame = crate::frame::new_frame(&config);
            app.draw(&mut frame);
            let layout = BoardLayout::new(config);
            assert_eq!(layout.opponent_panel.is_some(), wide);
            if wide {
                let panel = layout.opponent_panel.unwrap();
                let row = row_text(&frame, panel.y0 + 18);
                assert!(row.contains("◈ 30"), "panel stake row at {cols}: {row:?}");
            } else {
                let status = layout.status;
                let stake = stake_line(30);
                let chars: Vec<char> = row_text(&frame, status.y0).chars().collect();
                let stake_x0 = status.x1 + 1 - stake.chars().count();
                let right: String = chars[stake_x0..=status.x1].iter().collect();
                assert_eq!(right, stake, "band ends in the stake at game over");
            }

            // A Quick Play game over — no campaign pointer — shows nothing,
            // even with a settlement banner still around.
            app.profile.campaign_mut().set_in_progress(None);
            assert_eq!(app.stake_to_show(), None);
        }
    }

    #[test]
    fn the_primer_swallows_map_keys() {
        // Spec 023: nothing on the campaign map can be acted on while the primer
        // is up. The map keys that would open the outfitter or the deck builder,
        // move the cursor, or go Back all leave the screen and the modal exactly
        // as they were — which is what is asserted here (the map's cursor field
        // is private to campaign_map.rs, so it is covered by that module's own
        // tests rather than here). Only non-dismiss keys are pressed, so nothing
        // writes to disk.
        let mut app = App::new(Config { num_cols: 120, num_rows: 40 });
        app.open_campaign_map();
        app.modal = Some(Modal::Primer);

        for k in [KeyCode::Char('b'), KeyCode::Char('c'), KeyCode::Up, KeyCode::Char('x')] {
            app.handle_key(k);
            assert!(
                matches!(app.screen, Screen::CampaignMap { .. }),
                "{k:?} must not leave the map"
            );
            assert!(matches!(app.modal, Some(Modal::Primer)), "{k:?} must not dismiss the primer");
        }
    }
}
