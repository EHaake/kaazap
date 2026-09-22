//! Headless balance simulator (spec 022). Run the table with
//!   KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table -- --ignored --nocapture
//! Everything else in this file is an ordinary test (scripted-player rules,
//! termination, the balance guards). See docs/balance.md.

use kaazap::campaign::{CampaignRun, PLANETS, wins_needed};
use kaazap::card::{ALL_SIDE_CARDS, Card, DEFAULT_SIDE_DECK, FlipKind, PlayedCard};
use kaazap::economy::{
    RegionTier, SEED_PURSE, ante_floor, card_price, card_tier, region_tier, reserve_floor,
};
use kaazap::game::{GameAction, GamePhase, GameState};
use kaazap::opponent::{OPPONENTS, OpponentProfile, opponent_by_id};
use kaazap::player::Player;
use kaazap::profile::STARTER_SIDE_DECK;

const SCRIPTED_STAND_AT: i32 = 17;
const STEP_CAP: usize = 5_000; // a match that doesn't end in this many steps is a bug
const DEFAULT_N: usize = 10_000; // KAAZAP_SIM_N overrides (plan tension §3)
/// The guards' sample size (see the margin note above the guards).
const GUARD_N: usize = 600;
const TOL: f64 = 0.02; // ordering allowance (plan tension §3)

// Pool-best candidates (plan tension §5); T003 fixes, T004 may replace.
// All 10 cards, no flips (the scripted player never plays one). Each is the
// strongest of the three candidates T003 measured against its own region —
// see `specs/022-balance-pass/tuning-log.md` for the alternatives and rates.
const BEST_OUTER: [Card; 10] = [
    Card::Plus(3),
    Card::Plus(3),
    Card::Plus(3),
    Card::Minus(3),
    Card::Minus(3),
    Card::Minus(3),
    Card::Plus(2),
    Card::Minus(2),
    Card::PlusMinus(1),
    Card::PlusMinus(1),
];

const BEST_OUTER_MID: [Card; 10] = [
    Card::PlusMinus(3),
    Card::PlusMinus(3),
    Card::PlusMinus(3),
    Card::PlusMinus(3),
    Card::PlusMinus(2),
    Card::PlusMinus(2),
    Card::PlusMinus(2),
    Card::PlusMinus(1),
    Card::PlusMinus(1),
    Card::PlusMinus(1),
];

const BEST_FULL: [Card; 10] = [
    Card::PlusMinus(6),
    Card::PlusMinus(6),
    Card::PlusMinus(3),
    Card::PlusMinus(3),
    Card::PlusMinus(3),
    Card::PlusMinus(2),
    Card::PlusMinus(2),
    Card::PlusMinus(1),
    Card::PlusMinus(1),
    Card::Tiebreaker,
];

// ---------------------------------------------------------------- the player

#[derive(Debug, PartialEq)]
enum Move {
    Hit,
    Stand,
    Play { index: usize, value: i8 },
}

/// Every (hand index, signed value) the player could commit right now, in hand
/// order. Flips and empty slots contribute nothing — the scripted player never
/// plays a flip (a flip has no `playable_values`, and the AI doesn't play one
/// either), so the two Mid-tier flips are dead cards for it.
fn plays(gs: &GameState) -> Vec<(usize, i8)> {
    gs.player
        .hand
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.as_ref().map(|&card| (i, card)))
        .flat_map(|(i, card)| card.playable_values().into_iter().map(move |v| (i, v)))
        .collect()
}

/// The scripted player: a deterministic function of the two boards, doing
/// exactly the four things the spec names, in order (plan tension §2).
fn scripted_move(gs: &GameState) -> Move {
    let s = gs.player.score();
    let p = gs.opponent.score();
    let plays = plays(gs);

    // 1. Recover: over 20, the play that leaves the highest total <= 20;
    //    none means accept the bust.
    if s > 20 {
        let mut best: Option<(usize, i8)> = None;
        let mut best_total = i32::MIN;
        for &(index, value) in &plays {
            let total = s + value as i32;
            if total <= 20 && total > best_total {
                best = Some((index, value));
                best_total = total;
            }
        }
        return match best {
            Some((index, value)) => Move::Play { index, value },
            None => Move::Stand,
        };
    }

    // 2. Reach 20 whenever a card lands it exactly.
    let need = 20 - s;
    if (-6..=6).contains(&need) {
        let need = need as i8;
        if let Some(&(index, value)) = plays.iter().find(|&&(_, v)| v == need) {
            return Move::Play { index, value };
        }
    }

    // 3. The opponent has stood: beat its total, steal the tie, or wait.
    if gs.opponent.stood && !gs.opponent.bust {
        if s > p {
            return Move::Stand;
        }
        if s == p && gs.player.has_tiebreaker_in_play() && !gs.opponent.has_tiebreaker_in_play() {
            return Move::Stand;
        }

        // The smallest total that beats it without busting.
        let mut best: Option<(usize, i8)> = None;
        let mut best_total = i32::MAX;
        for &(index, value) in &plays {
            let total = s + value as i32;
            if total > p && total <= 20 && total < best_total {
                best = Some((index, value));
                best_total = total;
            }
        }
        if let Some((index, value)) = best {
            return Move::Play { index, value };
        }

        // The tie-steal: a tiebreaker landing exactly its total, unanswered.
        if !gs.opponent.has_tiebreaker_in_play() {
            for &(index, value) in &plays {
                if gs.player.hand[index] == Some(Card::Tiebreaker) && s + value as i32 == p {
                    return Move::Play { index, value };
                }
            }
        }

        // Standing behind is a sure loss; a tie is worth taking only late.
        return if s < p {
            Move::Hit
        } else if s >= SCRIPTED_STAND_AT {
            Move::Stand
        } else {
            Move::Hit
        };
    }

    // 4. The opponent is still live: stand on a sensible total.
    if s >= SCRIPTED_STAND_AT {
        Move::Stand
    } else {
        Move::Hit
    }
}

// ---------------------------------------------------------------- the driver

/// One full match through the production state machine, no clock.
/// `true` when the scripted player won it.
fn play_match(opponent: OpponentProfile, deck: &[Card]) -> bool {
    let mut gs = GameState::with_opponent(opponent, deck.to_vec());
    for _ in 0..STEP_CAP {
        match gs.game_phase {
            GamePhase::PlayerTurn => {
                if gs.player.stood || gs.player.bust {
                    gs.update(); // stale frame: let the engine advance
                    continue;
                }
                match scripted_move(&gs) {
                    Move::Hit => gs.apply_game_action(GameAction::Hit),
                    Move::Stand => gs.apply_game_action(GameAction::Stand),
                    Move::Play { index, value } => {
                        gs.apply_game_action(GameAction::PlayHand { index });
                        if matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }) {
                            gs.apply_game_action(GameAction::ChooseSign { positive: value > 0 });
                        }
                    }
                }
            }
            GamePhase::AwaitingSignChoice { .. } => unreachable!("answered in the same step"),
            // Skip the wall-clock thinking delay
            GamePhase::OpponentThinking { .. } => gs.game_phase = GamePhase::OpponentTurn,
            GamePhase::OpponentTurn | GamePhase::RoundEnd => gs.update(),
            GamePhase::AwaitingNextRound => gs.apply_game_action(GameAction::NextRound),
            GamePhase::GameOver { winner } => return winner == Player::Player,
        }
    }
    panic!("{} vs {:?} did not finish in {STEP_CAP} steps", opponent.id, deck)
}

fn win_rate(opponent: OpponentProfile, deck: &[Card], n: usize) -> f64 {
    let wins = (0..n).filter(|_| play_match(opponent, deck)).count();
    wins as f64 / n as f64
}

// ------------------------------------------------------------------ the grid

struct Deck {
    name: &'static str,
    cards: Vec<Card>,
}

fn decks() -> [Deck; 5] {
    [
        Deck { name: "starter", cards: STARTER_SIDE_DECK.to_vec() },
        Deck { name: "standard", cards: DEFAULT_SIDE_DECK.to_vec() },
        Deck { name: "best_outer", cards: BEST_OUTER.to_vec() },
        Deck { name: "best_outer_mid", cards: BEST_OUTER_MID.to_vec() },
        Deck { name: "best_full", cards: BEST_FULL.to_vec() },
    ]
}

/// The roster opponents of a region, from the map rather than a second list.
fn opponents_in(tier: RegionTier) -> Vec<OpponentProfile> {
    PLANETS
        .iter()
        .filter(|p| region_tier(p.region) == tier)
        .flat_map(|p| p.opponents.iter())
        .filter_map(|id| opponent_by_id(id))
        .collect()
}

/// The shop price of a tier: the cheapest card in it (robust to re-tiering).
fn tier_price(tier: RegionTier) -> u32 {
    ALL_SIDE_CARDS
        .iter()
        .copied()
        .filter(|&c| card_tier(c) == tier)
        .map(card_price)
        .min()
        .unwrap_or(0)
}

fn floor_of(o: &OpponentProfile) -> u32 {
    ante_floor(o.stand_threshold)
}

/// Expected credits per match at a given stake, even money.
fn ev_per_match(floor: u32, w: f64) -> f64 {
    floor as f64 * (2.0 * w - 1.0)
}

/// The chance of taking a first-to-`needed` series at a per-match rate `w`
/// (matches are independent and a match always produces a winner). Best of
/// three: w²(3 − 2w). Best of five: w³(6w² − 15w + 10).
fn series_rate(w: f64, needed: u32) -> f64 {
    match needed {
        2 => w * w * (3.0 - 2.0 * w),
        3 => w * w * w * (6.0 * w * w - 15.0 * w + 10.0),
        _ => panic!("no closed form for a first-to-{needed} series"),
    }
}

struct Row {
    deck: &'static str,
    opponent: &'static str,
    n: usize,
    win: f64,
}

fn measure(n: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    for deck in decks() {
        for opponent in OPPONENTS {
            rows.push(Row {
                deck: deck.name,
                opponent: opponent.id,
                n,
                win: win_rate(opponent, &deck.cards, n),
            });
        }
    }
    rows
}

fn rate(rows: &[Row], deck: &str, opponent: &str) -> f64 {
    rows.iter()
        .find(|r| r.deck == deck && r.opponent == opponent)
        .unwrap_or_else(|| panic!("no measured row for {deck} vs {opponent}"))
        .win
}

fn min_over(rows: &[Row], deck: &str, tier: RegionTier) -> (f64, &'static str) {
    opponents_in(tier)
        .into_iter()
        .map(|o| (rate(rows, deck, o.id), o.id))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .expect("every region has opponents")
}

fn max_over(rows: &[Row], deck: &str, tier: RegionTier) -> (f64, &'static str) {
    opponents_in(tier)
        .into_iter()
        .map(|o| (rate(rows, deck, o.id), o.id))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .expect("every region has opponents")
}

fn pct(w: f64) -> String {
    format!("{:.1}", w * 100.0)
}

fn line(desc: &str, value: String) -> String {
    format!("{desc:<68}{value}")
}

fn verdict(ok: bool) -> &'static str {
    if ok { "PASS" } else { "FAIL" }
}

// ------------------------------------------------------------- the checks

/// T1–T8, plus the coupled `C` line at index 8 (its own verdicts are inline).
fn targets(rows: &[Row]) -> Vec<(String, bool)> {
    let mut out = Vec::new();

    let greeb = rate(rows, "starter", "greeb");
    out.push((line("T1 starter vs greeb >= 65%", pct(greeb)), greeb >= 0.65));

    let (w, who) = min_over(rows, "starter", RegionTier::Outer);
    out.push((
        line("T2 starter vs each Outer Rim opponent >= 50%", format!("min {} ({who})", pct(w))),
        w >= 0.50,
    ));

    let (w, who) = max_over(rows, "starter", RegionTier::Mid);
    out.push((
        line("T3 starter vs each Mid Rim opponent < 50%", format!("max {} ({who})", pct(w))),
        w < 0.50,
    ));

    let (w, who) = max_over(rows, "starter", RegionTier::Core);
    out.push((
        line("T4 starter vs each Core opponent < 33%", format!("max {} ({who})", pct(w))),
        w < 0.33,
    ));

    let (w, who) = min_over(rows, "best_outer", RegionTier::Outer);
    out.push((
        line("T5 best_outer vs each Outer Rim opponent >= 55%", format!("min {} ({who})", pct(w))),
        w >= 0.55,
    ));

    let (w, who) = min_over(rows, "best_outer_mid", RegionTier::Mid);
    out.push((
        line("T6 best_outer_mid vs each Mid Rim opponent >= 50%", format!("min {} ({who})", pct(w))),
        w >= 0.50,
    ));

    let (w, who) = min_over(rows, "best_full", RegionTier::Core);
    let sovereign = rate(rows, "best_full", "sovereign");
    out.push((
        line(
            "T7 best_full vs each Core opponent >= 45%; sovereign 45-55%",
            format!("min {} ({who}), sovereign {}", pct(w), pct(sovereign)),
        ),
        w >= 0.45 && (0.45..=0.55).contains(&sovereign),
    ));

    // T8: each better pool wins at least as often, per opponent (within TOL);
    // and the Sovereign is the hardest opponent for the full-pool deck.
    let mut worst_drop = 0.0_f64;
    let mut worst_pair = "-";
    for o in OPPONENTS {
        let outer = rate(rows, "best_outer", o.id);
        let outer_mid = rate(rows, "best_outer_mid", o.id);
        let full = rate(rows, "best_full", o.id);
        for (lower, higher) in [(outer, outer_mid), (outer_mid, full)] {
            if lower - higher > worst_drop {
                worst_drop = lower - higher;
                worst_pair = o.id;
            }
        }
    }
    // The lowest full-pool rate among the others: the hardest opponent the
    // Sovereign has to beat to be the hardest of all.
    let (hardest_other, hardest_id) = OPPONENTS
        .iter()
        .filter(|o| o.id != "sovereign")
        .map(|o| (rate(rows, "best_full", o.id), o.id))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .expect("the roster has more than the Sovereign");
    out.push((
        line(
            "T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest",
            format!(
                "worst drop {} ({worst_pair}); sovereign {} vs hardest other {} ({hardest_id})",
                pct(worst_drop),
                pct(sovereign),
                pct(hardest_other)
            ),
        ),
        worst_drop <= TOL && sovereign <= hardest_other + TOL,
    ));

    // C: B4's inequality, EV_m > 2·EV_g, printed where T004 can read it.
    let greeb_profile = opponent_by_id("greeb").expect("greeb is on the roster");
    let ev_g = ev_per_match(floor_of(&greeb_profile), rate(rows, "starter", "greeb"));
    let (ev_m, ev_m2, who) = best_mid_ev(rows);
    let ok_floor = ev_m > 2.0 * ev_g;
    let ok_2x = ev_m2 > 2.0 * ev_g;
    let at_2x = if ok_2x && !ok_floor { "PASS (above floor)" } else { verdict(ok_2x) };
    out.push((
        format!(
            "C  B4 coupling (T004): best EV_m@floor {ev_m:.1} ({who}) vs 2·EV_g {:.1}  {}; \
             @2×floor {ev_m2:.1}  {at_2x}",
            2.0 * ev_g,
            verdict(ok_floor)
        ),
        ok_floor || ok_2x,
    ));

    out
}

/// The Mid Rim opponent the Outer+Mid deck earns most against: its EV per match
/// at the floor, at twice the floor, and who it is.
fn best_mid_ev(rows: &[Row]) -> (f64, f64, &'static str) {
    opponents_in(RegionTier::Mid)
        .into_iter()
        .map(|m| {
            let w = rate(rows, "best_outer_mid", m.id);
            let floor = floor_of(&m);
            (ev_per_match(floor, w), ev_per_match(2 * floor, w), m.id)
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .expect("the Mid Rim has opponents")
}

/// Expected floor-stake matches to earn `need` credits, or `None` when the
/// deck doesn't earn (EV <= 0, i.e. never).
fn matches_for(need: f64, ev: f64) -> Option<usize> {
    if ev <= 0.0 {
        None
    } else {
        Some((need / ev).ceil().max(0.0) as usize)
    }
}

fn count_text(k: Option<usize>) -> String {
    match k {
        Some(k) => k.to_string(),
        None => "inf".to_string(),
    }
}

/// B1–B5: arithmetic on the measured rates and the live constants.
fn bounds(rows: &[Row]) -> Vec<(String, bool)> {
    let reserve = reserve_floor(&CampaignRun::default());
    let p_outer = tier_price(RegionTier::Outer);
    let p_mid = tier_price(RegionTier::Mid);
    let p_core = tier_price(RegionTier::Core);
    let greeb = opponent_by_id("greeb").expect("greeb is on the roster");
    let floor_greeb = floor_of(&greeb);
    let ev_g = ev_per_match(floor_greeb, rate(rows, "starter", "greeb"));
    let need_core = (p_core + reserve).saturating_sub(SEED_PURSE) as f64;

    let mut out = Vec::new();

    // B1: the first Outer card, from the seed purse plus Greeb floor wins.
    let want_outer = p_outer + reserve;
    let (k_first, b1) = if SEED_PURSE >= want_outer {
        (Some(0), true)
    } else {
        let k = matches_for((want_outer - SEED_PURSE) as f64, ev_g);
        (k, k.is_some_and(|k| k <= 5))
    };
    out.push((
        line(
            "B1 first Outer card within 5 Greeb floor matches",
            format!("k={}", count_text(k_first)),
        ),
        b1,
    ));

    // B2: the seed purse plus a clean Outer Rim clear at the floor must not
    // reach the first Mid card — but grinding from there must.
    let after_outer =
        SEED_PURSE + opponents_in(RegionTier::Outer).iter().map(floor_of).sum::<u32>();
    let want_mid = p_mid + reserve;
    let k_mid = matches_for(want_mid.saturating_sub(after_outer) as f64, ev_g);
    out.push((
        line(
            "B2 first Mid card not affordable after one Outer Rim clear",
            format!("({after_outer} vs {want_mid}); grind k={}", count_text(k_mid)),
        ),
        after_outer < want_mid && ev_g > 0.0,
    ));

    // B3: funding a Core card by Greeb floor grinding is the long road.
    let k_grind = matches_for(need_core, ev_g);
    out.push((
        line(
            "B3 Core card by Greeb grind >= 20 matches",
            format!("k_grind={}", count_text(k_grind)),
        ),
        k_grind.is_none_or(|k| k >= 20),
    ));

    // B4: betting in the Mid Rim with the Outer+Mid deck gets there in fewer
    // than half as many expected matches — at the floor, or above it.
    let half = k_grind.map_or(f64::INFINITY, |k| k as f64 / 2.0);
    let best_bet = opponents_in(RegionTier::Mid)
        .into_iter()
        .filter(|m| rate(rows, "best_outer_mid", m.id) > 0.5)
        .map(|m| {
            let w = rate(rows, "best_outer_mid", m.id);
            let floor = floor_of(&m);
            (
                matches_for(need_core, ev_per_match(floor, w)),
                matches_for(need_core, ev_per_match(2 * floor, w)),
                m.id,
            )
        })
        .min_by_key(|(k_floor, _, _)| k_floor.unwrap_or(usize::MAX));
    let (b4_text, b4) = match best_bet {
        Some((k_floor, k_2x, who)) => {
            let at_floor = k_floor.is_some_and(|k| (k as f64) < half);
            let above = k_2x.is_some_and(|k| (k as f64) < half);
            (
                format!(
                    "best k_bet@floor={} ({who}), @2×floor={}{}",
                    count_text(k_floor),
                    count_text(k_2x),
                    if above && !at_floor { " (above floor)" } else { "" }
                ),
                at_floor || above,
            )
        }
        None => ("no Mid Rim opponent above 50%".to_string(), false),
    };
    out.push((line("B4 Core card by Mid Rim bets < k_grind/2", b4_text), b4));

    // B5: ruin is a small number of floor losses; the "a staked win never
    // leaves you broke" half is spec 021's pinned property, cited not re-derived.
    let k_ruin = if floor_greeb == 0 {
        None
    } else {
        Some((SEED_PURSE as f64 / floor_greeb as f64).ceil() as usize)
    };
    out.push((
        line(
            "B5 ruin within 8 floor losses; staked win never broke (021)",
            format!("k_ruin={}", count_text(k_ruin)),
        ),
        k_ruin.is_some_and(|k| k <= 8),
    ));

    out
}

// -------------------------------------------------------------- the report

/// The simulator itself: a report, never an assertion, so a failing curve
/// prints instead of panicking. Ignored by default — it plays 50 × N matches.
#[test]
#[ignore]
fn balance_table() {
    let n = std::env::var("KAAZAP_SIM_N")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_N);
    let rows = measure(n);

    println!();
    println!(
        "{:<16}{:<12}{:>7}{:>8}{:>17}{:>9}",
        "deck", "opponent", "n", "win%", "ev/match@floor", "series%"
    );
    for r in &rows {
        let floor = floor_of(&opponent_by_id(r.opponent).expect("row opponents are on the roster"));
        println!(
            "{:<16}{:<12}{:>7}{:>8}{:>17}{:>9}",
            r.deck,
            r.opponent,
            r.n,
            pct(r.win),
            format!("{:+.1}", ev_per_match(floor, r.win)),
            pct(series_rate(r.win, wins_needed(r.opponent)))
        );
    }

    let targets = targets(&rows);
    println!("targets");
    for (text, ok) in &targets[..8] {
        println!("  {text}  {}", verdict(*ok));
    }
    println!("  {}", targets[8].0);

    let bounds = bounds(&rows);
    println!(
        "bounds (SEED {SEED_PURSE}, reserve {}, P_outer {}, P_mid {}, P_core {})",
        reserve_floor(&CampaignRun::default()),
        tier_price(RegionTier::Outer),
        tier_price(RegionTier::Mid),
        tier_price(RegionTier::Core)
    );
    for (text, ok) in &bounds {
        println!("  {text}  {}", verdict(*ok));
    }

    let met = targets[..8].iter().filter(|(_, ok)| *ok).count();
    let coupling = usize::from(targets[8].1);
    let held = bounds.iter().filter(|(_, ok)| *ok).count();
    println!("summary: targets {met}/8, coupling {coupling}/1, bounds {held}/5");
}

// ------------------------------------------------------------- the guards

/// Dealer cards summing to `total` (the `opponent_at` pattern in `game.rs`).
fn dealer_row(total: u8) -> Vec<PlayedCard> {
    let mut row = Vec::new();
    let mut remaining = total;
    while remaining > 0 {
        let n = remaining.min(10);
        row.push(PlayedCard { card: Card::Dealer(n), value: n as i8 });
        remaining -= n;
    }
    row
}

/// A fixed board: both dealer rows, the player's hand, and whether the
/// opponent has stood. Mirrors `opponent_at` in `game.rs` to the player's side.
fn board(player_total: u8, hand: Vec<Card>, opponent_total: u8, opponent_stood: bool) -> GameState {
    let mut gs = GameState::new();
    gs.player.dealer_row = dealer_row(player_total);
    gs.player.played_row = vec![];
    gs.player.hand = hand.into_iter().map(Some).collect();
    gs.opponent.dealer_row = dealer_row(opponent_total);
    gs.opponent.played_row = vec![];
    gs.opponent.stood = opponent_stood;
    gs
}

#[test]
fn scripted_player_follows_its_rules_on_fixed_boards() {
    // 1. Recover: over 20, the play that leaves the highest total <= 20.
    let gs = board(23, vec![Card::Minus(2), Card::Minus(4), Card::Minus(3)], 0, false);
    assert_eq!(scripted_move(&gs), Move::Play { index: 2, value: -3 }, "23 - 3 = 20 is the best");
    // …and with nothing that fits back under, accept the bust.
    let gs = board(23, vec![Card::Minus(2)], 0, false);
    assert_eq!(scripted_move(&gs), Move::Stand, "23 with only -2 cannot recover");

    // 2. Reach 20 exactly, with a fixed card or the right sign of a ±.
    let gs = board(17, vec![Card::Plus(3)], 0, false);
    assert_eq!(scripted_move(&gs), Move::Play { index: 0, value: 3 });
    let gs = board(17, vec![Card::PlusMinus(3)], 0, false);
    assert_eq!(scripted_move(&gs), Move::Play { index: 0, value: 3 });

    // 3. The opponent has stood.
    let gs = board(19, vec![Card::Plus(3)], 18, true);
    assert_eq!(scripted_move(&gs), Move::Stand, "already ahead of a stood opponent");

    // Reaching 20 outranks the smallest-winning-total scan (rules run in order).
    let gs = board(16, vec![Card::Plus(4), Card::Plus(3)], 18, true);
    assert_eq!(scripted_move(&gs), Move::Play { index: 0, value: 4 }, "16 + 4 = 20");

    // With no card that lands 20, take the smallest total that still wins.
    let gs = board(15, vec![Card::Plus(4), Card::Plus(3)], 17, true);
    assert_eq!(
        scripted_move(&gs),
        Move::Play { index: 1, value: 3 },
        "the smallest winning total, not the biggest"
    );

    let gs = board(17, vec![Card::Tiebreaker], 18, true);
    assert_eq!(scripted_move(&gs), Move::Play { index: 0, value: 1 }, "the tie-steal");

    let gs = board(12, vec![Card::Plus(1)], 18, true);
    assert_eq!(scripted_move(&gs), Move::Hit, "standing behind is a sure loss");

    let gs = board(18, vec![], 18, true);
    assert_eq!(scripted_move(&gs), Move::Stand, "take the tie late");

    let gs = board(12, vec![], 12, true);
    assert_eq!(scripted_move(&gs), Move::Hit, "chase the tie early");

    // 4. The opponent is live: the stand threshold decides.
    let gs = board(17, vec![], 10, false);
    assert_eq!(scripted_move(&gs), Move::Stand);
    let gs = board(16, vec![], 10, false);
    assert_eq!(scripted_move(&gs), Move::Hit);

    // Flips are never played — a hand of nothing else hits instead.
    let flips = vec![Card::Flip(FlipKind::TwoFour), Card::Flip(FlipKind::ThreeSix)];
    let gs = board(16, flips.clone(), 10, false);
    assert_eq!(scripted_move(&gs), Move::Hit);
    let gs = board(16, flips, 18, true);
    assert_eq!(scripted_move(&gs), Move::Hit);
}

#[test]
fn named_decks_are_ten_cards_from_their_pools() {
    for deck in decks() {
        assert_eq!(deck.cards.len(), 10, "{} is not a legal deck", deck.name);
        for card in &deck.cards {
            assert!(
                ALL_SIDE_CARDS.contains(card),
                "{} holds {}, which is not in the universe",
                deck.name,
                card.label()
            );
        }
    }

    let pools: [(&str, Vec<Card>, RegionTier); 3] = [
        ("starter", STARTER_SIDE_DECK.to_vec(), RegionTier::Outer),
        ("best_outer", BEST_OUTER.to_vec(), RegionTier::Outer),
        ("best_outer_mid", BEST_OUTER_MID.to_vec(), RegionTier::Mid),
    ];
    for (name, cards, depth) in pools {
        for card in cards {
            assert!(
                card_tier(card) <= depth,
                "{name} holds {}, above its pool",
                card.label()
            );
        }
    }

    // No candidate carries a flip: the scripted player can never play one.
    for deck in [BEST_OUTER, BEST_OUTER_MID, BEST_FULL] {
        assert!(!deck.iter().any(|c| matches!(c, Card::Flip(_))));
    }
}

#[test]
fn a_scripted_match_terminates_against_every_roster_opponent() {
    // One match per pair through the real state machine: no hang, no
    // unreachable phase, a winner every time (`play_match` panics otherwise).
    for deck in decks() {
        for opponent in OPPONENTS {
            play_match(opponent, &deck.cards);
        }
    }
}

// The guard margins, in standard errors at `GUARD_N` = 600, computed from the
// T004a final table (N = 10 000, `specs/022-balance-pass/tuning-log.md`).
// Single rate: (w − bound) / √(w(1−w)/N). Difference: gap / √((w₁(1−w₁) +
// w₂(1−w₂))/N). Sizing is from the SE at GUARD_N, not from the run-to-run
// spread of the N = 10 000 table.
//
//   starter vs Greeb >= 50%   w = .690  (.690−.500)/√(.690·.310/600)
//                             = .190/.01888 = 10.1 SE
//   starter vs Core <= 50%    binding rix, w = .299
//                             (.500−.299)/√(.299·.701/600) = .201/.01869 = 10.8 SE
//                             (magistrate 12.4, sovereign 15.1)
//   full > starter            binding greeb, gap .890−.690 = .200
//                             .200/√((.890·.110 + .690·.310)/600) = .200/.02280 = 8.8 SE
//                             (every other opponent is 9.0 SE or more)
//
// Smallest margin 8.8 SE (full vs starter against Greeb), so GUARD_N stays at
// 600: a false failure is well under 1e-6 per guard, and the three guards
// together run ~15 000 matches, seconds in debug. (No guard covers the Mid Rim
// because the smallest margin there is too thin: against a 50% bar, toran
// w = .402 gives .098/√(.402·.598/600) = .098/.02002 = 4.9 SE, under the 5 SE
// bar — nima 6.0, brakka 6.4, kesh 6.5.)

#[test]
fn starter_deck_beats_greeb_above_the_floor() {
    let greeb = opponent_by_id("greeb").expect("greeb is on the roster");
    let w = win_rate(greeb, &STARTER_SIDE_DECK, GUARD_N);
    assert!(w >= 0.50, "starter vs greeb is {}, below the 50% floor", pct(w));
}

#[test]
fn starter_deck_cannot_credibly_take_the_core() {
    for o in opponents_in(RegionTier::Core) {
        let w = win_rate(o, &STARTER_SIDE_DECK, GUARD_N);
        assert!(w <= 0.50, "starter vs {} is {}, above the 50% ceiling", o.id, pct(w));
    }
}

#[test]
fn the_full_pool_deck_outperforms_the_starter_against_every_opponent() {
    for o in OPPONENTS {
        let starter = win_rate(o, &STARTER_SIDE_DECK, GUARD_N);
        let full = win_rate(o, &BEST_FULL, GUARD_N);
        assert!(
            full > starter,
            "vs {}: full pool {} does not beat starter {}",
            o.id,
            pct(full),
            pct(starter)
        );
    }
}

#[test]
fn series_rate_matches_the_closed_form() {
    // The figures spec 029 flagged to the person: best of three, then best of five.
    for (w, needed, expected) in [
        (0.45, 2, 0.425),
        (0.60, 2, 0.648),
        (0.50, 2, 0.50),
        (0.33, 3, 0.205),
        (0.50, 3, 0.50),
    ] {
        let got = series_rate(w, needed);
        assert!(
            (got - expected).abs() < 1e-3,
            "series_rate({w}, {needed}) is {got}, expected {expected}"
        );
    }
}
